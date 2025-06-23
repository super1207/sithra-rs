use std::convert::Infallible;

use crate::{boxed::BoxedIntoRoute, handler::Handler, routing::endpoint::Endpoint};

pub mod boxed;
pub mod extract;
pub mod handler;
pub mod multi;
pub mod request;
pub mod response;
pub mod routing;
pub mod service;

pub(crate) fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

pub fn on<H, T, S>(handler: H) -> Endpoint<S, Infallible>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    Endpoint::BoxedHandler(BoxedIntoRoute::from_handler(handler))
}

#[must_use]
pub fn multi<S, const N: usize>(endpoints: [Endpoint<S, Infallible>; N]) -> Endpoint<S, Infallible>
where
    S: Clone + Send + Sync + 'static,
{
    Endpoint::BoxedHandler(BoxedIntoRoute::from_multi(endpoints))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sithra_transport::datapack::{RequestDataPack, ResponseDataPack};
    use tokio::sync::Mutex;
    use tower::Service;
    use triomphe::Arc;

    use crate::{
        extract::{payload::Payload, state::State},
        multi, on,
        request::Request,
        routing::router::Router,
    };

    #[derive(Default, Clone)]
    struct AppState {
        counter: Arc<AtomicUsize>,
    }

    fn test_data(path: impl Into<String>) -> RequestDataPack {
        RequestDataPack::default().path(path)
    }

    async fn count2(State(state): State<AppState>) -> ResponseDataPack {
        state.counter.fetch_add(2, Ordering::Relaxed);
        ResponseDataPack::default().payload_value("")
    }
    async fn on_message(Payload(_message): Payload<String>, State(state): State<AppState>) {
        state.counter.fetch_add(1, Ordering::Relaxed);
    }

    #[tokio::test]
    async fn router() {
        let state = AppState::default();
        let a = Arc::new(Mutex::new(String::new()));
        let a_ = a.clone();
        let mut router: Router = Router::new()
            .route("/message", multi([on(async || {}), on(on_message)]))
            .route(
                "/count",
                on(async move |State(state): State<AppState>| {
                    {
                        *a_.lock().await = "xixi".to_owned();
                    }
                    state.counter.fetch_add(1, Ordering::Relaxed);
                }),
            )
            .route("/count2", on(count2))
            .with_state(state.clone());

        router
            .call(Request::new(test_data("/message").payload("Hello World.")))
            .await
            .unwrap();

        let response = router.call(Request::new(test_data("/count"))).await.unwrap();
        assert_eq!(state.counter.load(Ordering::SeqCst), 1);
        assert!(response.is_none());
        assert_eq!(*a.lock().await, "xixi");

        let request = Request::new(test_data("/count2"));
        let correlation = request.correlation();
        let response = router.call(request).await.unwrap();
        tokio::task::yield_now().await;
        assert_eq!(state.counter.load(Ordering::SeqCst), 4);
        assert_eq!(response.data.map(|r| r.correlation), Some(correlation));
    }
}
