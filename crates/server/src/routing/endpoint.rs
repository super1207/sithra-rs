use std::convert::Infallible;

use tower::{Layer, Service};

use crate::{
    boxed::BoxedIntoRoute, request::Request, response::IntoResponse, routing::route::Route,
};

#[derive(Clone, Debug)]
pub enum Endpoint<S, E = Infallible> {
    Route(Route<E>),
    BoxedHandler(BoxedIntoRoute<S, E>),
}

impl<S, E> Endpoint<S, E>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn layer<L>(self, layer: L) -> Endpoint<S>
    where
        L: Layer<Route<E>> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        E: 'static,
    {
        match self {
            Self::BoxedHandler(handler) => {
                let layer_fn = move |route: Route<E>| route.layer(layer.clone());

                Endpoint::BoxedHandler(handler.map(layer_fn))
            }
            Self::Route(route) => Endpoint::Route(route.layer(layer)),
        }
    }
}
