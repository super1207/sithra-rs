use std::{
    convert::Infallible,
    fmt,
    task::{Context, Poll},
};

use tower::Service;

use crate::server::{
    request::Request,
    response::Response,
    routing::{route::RouteFuture, router::Router},
};

pub mod endpoint;
pub mod route;
pub mod router;

/// A [`Router`] converted into a borrowed [`Service`] with a fixed body type.
///
/// See [`Router::as_service`] for more details.
pub struct RouterAsService<'a, S = ()> {
    router: &'a mut Router<S>,
}

impl Service<Request> for RouterAsService<'_, ()> {
    type Error = Infallible;
    type Future = RouteFuture<Infallible>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <Router as Service<Request>>::poll_ready(self.router, cx)
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.router.call(req)
    }
}

impl<S> fmt::Debug for RouterAsService<'_, S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouterAsService").field("router", &self.router).finish()
    }
}

/// A [`Router`] converted into an owned [`Service`] with a fixed body type.
///
/// See [`Router::into_service`] for more details.
pub struct RouterIntoService<S = ()> {
    router: Router<S>,
}

impl<S> Clone for RouterIntoService<S>
where
    Router<S>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
        }
    }
}

impl Service<Request> for RouterIntoService<()> {
    type Error = Infallible;
    type Future = RouteFuture<Infallible>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <Router as Service<Request>>::poll_ready(&mut self.router, cx)
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.router.call(req)
    }
}

impl<S> fmt::Debug for RouterIntoService<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouterIntoService").field("router", &self.router).finish()
    }
}
