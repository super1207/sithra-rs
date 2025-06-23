use std::{
    convert::Infallible,
    fmt,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::future::Map;
use pin_project::pin_project;
use tower::{Layer, Service, ServiceExt, util::Oneshot};

use crate::{
    extract::FromRequest,
    request::Request,
    response::{IntoResponse, Response},
};

pub trait Handler<T, S>: Clone + Send + Sync + Sized + 'static {
    /// The type of future calling this handler returns.
    type Future: Future<Output = Response> + Send + 'static;

    /// Call the handler with the given request.
    fn call(self, req: Request, state: S) -> Self::Future;

    /// Apply a [`tower::Layer`] to the handler.
    ///
    /// All requests to the handler will be processed by the layer's
    /// corresponding middleware.
    ///
    /// This can be used to add additional processing to a request for a single
    /// handler.
    ///
    /// Note this differs from
    /// [`routing::Router::layer`](crate::routing::Router::layer) which adds
    /// a middleware to a group of routes.
    ///
    /// If you're applying middleware that produces errors you have to handle
    /// the errors so they're converted into responses. You can learn more
    /// about doing that [here](crate::error_handling).
    ///
    /// # Example
    ///
    /// Adding the [`tower::limit::ConcurrencyLimit`] middleware to a handler
    /// can be done like so:
    ///
    /// ```rust
    /// use axum::{Router, handler::Handler, routing::get};
    /// use tower::limit::{ConcurrencyLimit, ConcurrencyLimitLayer};
    ///
    /// async fn handler() { /* ... */
    /// }
    ///
    /// let layered_handler = handler.layer(ConcurrencyLimitLayer::new(64));
    /// let app = Router::new().route("/", get(layered_handler));
    /// # let _: Router = app;
    /// ```
    fn layer<L>(self, layer: L) -> Layered<L, Self, T, S>
    where
        L: Layer<HandlerService<Self, T, S>> + Clone,
        L::Service: Service<Request>,
    {
        Layered {
            layer,
            handler: self,
            _marker: PhantomData,
        }
    }

    /// Convert the handler into a [`Service`] by providing the state
    fn with_state(self, state: S) -> HandlerService<Self, T, S> {
        HandlerService::new(self, state)
    }
}

/// A [`Service`] created from a [`Handler`] by applying a Tower middleware.
///
/// Created with [`Handler::layer`]. See that method for more details.
pub struct Layered<L, H, T, S> {
    layer:   L,
    handler: H,
    _marker: PhantomData<fn() -> (T, S)>,
}

#[allow(clippy::missing_fields_in_debug)]
impl<L, H, T, S> fmt::Debug for Layered<L, H, T, S>
where
    L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layered").field("layer", &self.layer).finish()
    }
}

impl<L, H, T, S> Clone for Layered<L, H, T, S>
where
    L: Clone,
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            layer:   self.layer.clone(),
            handler: self.handler.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, S, T, L> Handler<T, S> for Layered<L, H, T, S>
where
    L: Layer<HandlerService<H, T, S>> + Clone + Send + Sync + 'static,
    H: Handler<T, S>,
    L::Service: Service<Request, Error = Infallible> + Clone + Send + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse,
    <L::Service as Service<Request>>::Future: Send,
    T: 'static,
    S: 'static,
{
    type Future = LayeredFuture<L::Service>;

    fn call(self, req: Request, state: S) -> Self::Future {
        use futures_util::future::{FutureExt, Map};

        let svc = self.handler.with_state(state);
        let svc = self.layer.layer(svc);

        let future: Map<
            _,
            fn(
                Result<
                    <L::Service as Service<Request>>::Response,
                    <L::Service as Service<Request>>::Error,
                >,
            ) -> _,
        > = svc.oneshot(req).map(|result| match result {
            Ok(response) => response.into_response(),
            Err(err) => match err {},
        });

        LayeredFuture::new(future)
    }
}

/// An adapter that makes a [`Handler`] into a [`Service`].
///
/// Created with [`Handler::with_state`] or
/// [`HandlerWithoutStateExt::into_service`].
///
/// [`HandlerWithoutStateExt::into_service`]: super::HandlerWithoutStateExt::into_service
pub struct HandlerService<H, T, S> {
    handler: H,
    state:   S,
    _marker: PhantomData<fn() -> T>,
}

impl<H, T, S> HandlerService<H, T, S> {
    /// Get a reference to the state.
    pub const fn state(&self) -> &S {
        &self.state
    }
}

impl<H, T, S> HandlerService<H, T, S> {
    pub(super) fn new(handler: H, state: S) -> Self {
        Self {
            handler,
            state,
            _marker: PhantomData,
        }
    }
}

impl<H, T, S> fmt::Debug for HandlerService<H, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoService").finish_non_exhaustive()
    }
}

impl<H, T, S> Clone for HandlerService<H, T, S>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            state:   self.state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, T, S> Service<Request> for HandlerService<H, T, S>
where
    H: Handler<T, S> + Clone + Send + 'static,
    S: Clone + Send + Sync,
{
    type Error = Infallible;
    type Future = IntoServiceFuture<H::Future>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // `IntoService` can only be constructed from async functions which are always
        // ready, or from `Layered` which buffers in `<Layered as
        // Handler>::call` and is therefore also always ready.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        use futures_util::future::FutureExt;

        let handler = self.handler.clone();
        let future = Handler::call(handler, req, self.state.clone());
        let future = future.map(Ok as _);

        IntoServiceFuture::new(future)
    }
}

#[pin_project]
/// The response future for [`IntoService`](super::IntoService).
pub struct IntoServiceFuture<F> {
    #[pin]
    future: Map<F, fn(Response) -> Result<Response, Infallible>>,
}
impl<F> IntoServiceFuture<F> {
    pub(crate) fn new(future: Map<F, fn(Response) -> Result<Response, Infallible>>) -> Self {
        Self { future }
    }
}
impl<F> fmt::Debug for IntoServiceFuture<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(IntoServiceFuture)).finish_non_exhaustive()
    }
}
impl<F> Future for IntoServiceFuture<F>
where
    Map<F, fn(Response) -> Result<Response, Infallible>>: Future,
{
    type Output = <Map<F, fn(Response) -> Result<Response, Infallible>> as Future>::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

#[pin_project]
/// The response future for [`Layered`](super::Layered).
pub struct LayeredFuture<S>
where
    S: Service<Request>,
{
    #[pin]
    inner: Map<Oneshot<S, Request>, fn(Result<S::Response, S::Error>) -> Response>,
}

impl<S> LayeredFuture<S>
where
    S: Service<Request>,
{
    pub(super) fn new(
        inner: Map<Oneshot<S, Request>, fn(Result<S::Response, S::Error>) -> Response>,
    ) -> Self {
        Self { inner }
    }
}

impl<S> Future for LayeredFuture<S>
where
    S: Service<Request>,
{
    type Output = Response;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

macro_rules! handler_for_tuple {
    (@impl) => {
        impl<F, Fut, Res, S> Handler<(), S> for F
        where
            F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Res> + Send,
            Res: IntoResponse,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, _req: Request, _state: S) -> Self::Future {
                Box::pin(async move { self().await.into_response() })
            }
        }
    };
    (@impl $first:ident $(, $rest:ident)*)=>{
        handler_for_tuple!(@inner $first $(, $rest)*);
        handler_for_tuple!(@impl $($rest),*);
    };
    (@inner $($T:ident),*)=> {
        #[allow(non_snake_case, unused_mut)]
        impl<Fun, Fut, Sta, Res, $($T,)*> Handler<($($T,)*), Sta> for Fun
        where
            Fun: FnOnce($($T,)*) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Res> + Send,
            Sta: Send + Sync + 'static,
            Res: IntoResponse,
            $( $T: FromRequest<Sta> + Send, )*
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: Request, state: Sta) -> Self::Future {
                let raw = req.into_raw();
                Box::pin(async move {
                    $(
                        let $T = match $T::from_request(raw.clone(), &state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*


                    self($($T,)*).await.into_response()
                })
            }
        }
    };
}
handler_for_tuple!(@impl A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
