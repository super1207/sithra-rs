use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    task::{Context, Poll},
};

use matchit::MatchError;
pub use matchit::Router as RouteRouter;
use tower::{Layer, Service};
use triomphe::Arc;

use crate::server::{
    request::Request,
    response::{IntoResponse, Response},
    routing::{
        endpoint::Endpoint,
        route::{Route, RouteFuture},
    },
    try_downcast,
};

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteId(u32);

#[derive(Debug)]
pub struct RouterInner<S> {
    routes:        HashMap<RouteId, Endpoint<S>>,
    route_router:  RouteRouter<RouteId>,
    prev_route_id: RouteId,
}

impl<S> Default for RouterInner<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> RouterInner<S> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes:        HashMap::new(),
            route_router:  RouteRouter::new(),
            prev_route_id: RouteId(0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Router<S = ()> {
    inner: Arc<RouterInner<S>>,
}

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Router<S> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RouterInner::new()),
        }
    }
}

macro_rules! panic_on_err {
    ($expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(err) => panic!("{err}"),
        }
    };
}

macro_rules! tap_inner {
    ( $self_:ident, mut $inner:ident => { $($stmt:stmt)* } ) => {
        #[allow(redundant_semicolons)]
        {
            let mut $inner = $self_.into_inner();
            $($stmt)*;
            Router {
                inner: Arc::new($inner),
            }
        }
    };
}

macro_rules! map_inner {
    ($self_:ident, $inner:pat_param => $expr:expr) => {
        #[allow(redundant_semicolons)]
        {
            let $inner = $self_.into_inner();
            Router {
                inner: Arc::new($expr),
            }
        }
    };
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn into_inner(self) -> RouterInner<S> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            Err(arc) => RouterInner {
                routes:        arc.routes.clone(),
                route_router:  arc.route_router.clone(),
                prev_route_id: arc.prev_route_id,
            },
        }
    }

    #[track_caller]
    #[must_use]
    pub fn route(self, path: &str, method_router: Endpoint<S>) -> Self {
        tap_inner!(self, mut this => {
            panic_on_err!(this.route(path, method_router));
        })
    }

    /// # Panics
    /// Panics if the service is a `Router`.
    #[must_use]
    pub fn route_service<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        let Err(service) = try_downcast::<Self, _>(service) else {
            panic!("Invalid route: `Router::route_service` cannot be used with `Router`s.");
        };

        tap_inner!(self, mut this => {
            panic_on_err!(this.route_service(path, service));
        })
    }

    #[must_use]
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        map_inner!(self, this => this.layer(layer))
    }

    #[must_use]
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        map_inner!(self, this => this.route_layer(layer))
    }

    #[must_use]
    pub fn has_routes(&self) -> bool {
        !self.inner.routes.is_empty()
    }

    pub fn with_state<S2>(self, state: S) -> Router<S2> {
        map_inner!(self, this => this.with_state(state))
    }

    pub(crate) fn call_with_state(&self, req: Request, state: S) -> RouteFuture<Infallible> {
        if let Ok(future) = self.inner.call_with_state(req, state) {
            return future;
        }
        RouteFuture::ready(Response::none())
    }
}

impl Service<Request> for Router<()> {
    type Error = Infallible;
    type Future = RouteFuture<Infallible>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.call_with_state(req, ())
    }
}

impl<S> RouterInner<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn set_node(&mut self, path: &str, id: RouteId) -> Result<(), String> {
        self.route_router
            .insert(path, id)
            .map_err(|err| format!("Invalid route {path:?}: {err}"))
    }

    /// # Errors
    /// Returns an error if the route already exists.
    pub fn route(&mut self, path: &str, endpoint: Endpoint<S>) -> Result<(), Cow<'static, str>> {
        let id = self.next_route_id();
        self.set_node(path, id)?;
        self.routes.insert(id, endpoint);

        Ok(())
    }

    /// # Errors
    /// Returns an error if the route already exists.
    pub fn route_service<T>(&mut self, path: &str, service: T) -> Result<(), Cow<'static, str>>
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        self.route_endpoint(path, Endpoint::Route(Route::new(service)))
    }

    pub(super) fn route_endpoint(
        &mut self,
        path: &str,
        endpoint: Endpoint<S>,
    ) -> Result<(), Cow<'static, str>> {
        let id = self.next_route_id();
        self.set_node(path, id)?;
        self.routes.insert(id, endpoint);

        Ok(())
    }

    #[must_use]
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let route = endpoint.layer(layer.clone());
                (id, route)
            })
            .collect();

        Self {
            routes,
            route_router: self.route_router,
            prev_route_id: self.prev_route_id,
        }
    }

    /// # Panics
    /// Panics if no routes have been added yet.
    #[track_caller]
    #[must_use]
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        assert!(
            !self.routes.is_empty(),
            "Adding a route_layer before any routes is a no-op. Add the routes you want the layer \
             to apply to first."
        );

        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let route = endpoint.layer(layer.clone());
                (id, route)
            })
            .collect();

        Self {
            routes,
            route_router: self.route_router,
            prev_route_id: self.prev_route_id,
        }
    }

    pub fn has_routes(&self) -> bool {
        !self.routes.is_empty()
    }

    pub(super) fn with_state<S2>(self, state: S) -> RouterInner<S2> {
        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let endpoint: Endpoint<S2> = match endpoint {
                    Endpoint::BoxedHandler(handler) => {
                        Endpoint::Route(handler.into_route(state.clone()))
                    }
                    Endpoint::Route(route) => Endpoint::Route(route),
                };
                (id, endpoint)
            })
            .collect();

        RouterInner {
            routes,
            route_router: self.route_router,
            prev_route_id: self.prev_route_id,
        }
    }

    #[allow(clippy::result_large_err)]
    pub(super) fn call_with_state(
        &self,
        req: Request,
        state: S,
    ) -> Result<RouteFuture<Infallible>, (Request, S)> {
        let raw = req.into_raw();
        match self.route_router.at(&raw.path) {
            Ok(match_) => {
                let id = *match_.value;

                // url_params::insert_url_params(&mut parts.extensions, match_.params);

                let endpoint = self
                    .routes
                    .get(&id)
                    .expect("no route for id. This is a bug in sithra. Please file an issue");

                let req = Request::from_raw(raw);
                match endpoint {
                    Endpoint::BoxedHandler(handler) => {
                        let route = handler.clone().into_route(state);
                        Ok(route.oneshot_inner_owned(req))
                    }
                    Endpoint::Route(route) => Ok(route.clone().call_owned(req)),
                }
            }
            // explicitly handle all variants in case matchit adds
            // new ones we need to handle differently
            Err(MatchError::NotFound) => Err((Request::from_raw(raw), state)),
        }
    }

    const fn next_route_id(&mut self) -> RouteId {
        let next_id = self
            .prev_route_id
            .0
            .checked_add(1)
            .expect("Over `u32::MAX` routes created. If you need this, please file an issue.");
        self.prev_route_id = RouteId(next_id);
        self.prev_route_id
    }
}
