use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;
use tower::{
    Layer, Service, ServiceExt,
    util::{BoxCloneSyncService, MapErrLayer, Oneshot},
};
use ulid::Ulid;

use crate::server::{
    request::Request,
    response::{IntoResponse, MapIntoResponse, Response},
};

#[derive(Clone, Debug)]
pub struct Route<E = Infallible>(BoxCloneSyncService<Request, Response, E>);

impl<E> Route<E> {
    pub(crate) fn new<T>(svc: T) -> Self
    where
        T: Service<Request, Error = E> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse + 'static,
        T::Future: Send + 'static,
    {
        Self(BoxCloneSyncService::new(MapIntoResponse::new(svc)))
    }

    /// Variant of [`Route::call`] that takes ownership of the route to avoid
    /// cloning.
    pub(crate) fn call_owned(self, req: Request) -> RouteFuture<E> {
        self.oneshot_inner_owned(req)
    }

    pub(crate) fn oneshot_inner(&self, req: Request) -> RouteFuture<E> {
        let correlation = req.correlation();
        RouteFuture::new(self.0.clone().oneshot(req), correlation)
    }

    /// Variant of [`Route::oneshot_inner`] that takes ownership of the route to
    /// avoid cloning.
    pub(crate) fn oneshot_inner_owned(self, req: Request) -> RouteFuture<E> {
        let correlation = req.correlation();
        RouteFuture::new(self.0.oneshot(req), correlation)
    }

    pub(crate) fn layer<L, NewError>(self, layer: L) -> Route<NewError>
    where
        L: Layer<Self> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<NewError> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        NewError: 'static,
    {
        let layer = (MapErrLayer::new(Into::into), layer);

        Route::new(layer.layer(self))
    }
}

impl<E> Service<Request> for Route<E> {
    type Error = E;
    type Future = RouteFuture<E>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.oneshot_inner(req)
    }
}

#[pin_project(project = RouteFutureProj)]
pub enum RouteFuture<E> {
    Oneshot(#[pin] RouteFutureOneshot<E>),
    Ready(Option<Response>),
}

#[pin_project]
pub struct RouteFutureOneshot<E> {
    #[pin]
    inner:       Oneshot<BoxCloneSyncService<Request, Response, E>, Request>,
    correlation: Ulid,
}

impl<E> RouteFuture<E> {
    const fn new(
        inner: Oneshot<BoxCloneSyncService<Request, Response, E>, Request>,
        correlation: Ulid,
    ) -> Self {
        Self::Oneshot(RouteFutureOneshot { inner, correlation })
    }

    #[must_use]
    pub const fn ready(response: Response) -> Self {
        Self::Ready(Some(response))
    }
}

impl<E> Future for RouteFuture<E> {
    type Output = Result<Response, E>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            RouteFutureProj::Oneshot(route_future_oneshot) => {
                let this = route_future_oneshot.project();
                let mut res = ready!(this.inner.poll(cx))?;
                res.correlate(*this.correlation);
                Poll::Ready(Ok(res))
            }
            RouteFutureProj::Ready(response) => {
                Poll::Ready(Ok(response.take().expect("unreachable")))
            }
        }
    }
}
