#[macro_export]
macro_rules! typed {
    ($route:expr => @impl $typed:ty; $($T:ty),*)=>{
        $(impl<Sta: ::std::marker::Send + ::std::marker::Sync> AllowedParams<Sta> for $T {})*
        pub trait AllowedParams<Sta> {}
        typed!(@private A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
        #[allow(dead_code)]
        impl $typed {
            pub fn on<H, T, S>(handler: H) -> (&'static str, $crate::server::routing::endpoint::Endpoint<S, ::std::convert::Infallible>)
            where
                H: $crate::server::handler::Handler<T, S>,
                T: AllowedParams<S> + 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                (
                    $route,
                    $crate::server::routing::endpoint::Endpoint::BoxedHandler($crate::server::boxed::BoxedIntoRoute::from_handler(handler)),
                )
            }
        }
    };
    (@private $first:ident $(, $rest:ident)*)=> {
        typed!(@inner $first $( ,$rest)*);
        typed!(@private $($rest),*);
    };
    (@private) => {
        impl<Sta: ::std::marker::Sync> AllowedParams<Sta> for () {}
    };
    (@inner $($T:ident),*)=> {
        impl<Sta, $($T,)*> AllowedParams<Sta> for ($($T),*,)
        where
            Sta: ::std::marker::Send + ::std::marker::Sync,
            $($T: AllowedParams<Sta> + $crate::server::extract::FromRequest<Sta> + 'static,)*
        {
        }
    };
}

#[cfg(test)]
fn _typed() {
    use crate::server::{
        extract::{payload::Payload, state::State},
        routing::{endpoint::Endpoint, router::Router},
    };

    struct Message;
    typed!("/message" => @impl Message; Payload<String>, State<()>);
    let _: (_, Endpoint<()>) = Message::on(async || {});
    let _: (_, Endpoint<()>) =
        Message::on(async |Payload(_str): Payload<String>, State(()): State<()>| {});
    let _: (_, Endpoint<()>) = Message::on(async |Payload(_str): Payload<String>| {});
    let _: (_, Endpoint<()>) = Message::on(async |State(_unit): State<()>| {});

    // Type Error
    // ```rust
    // let _: (_, Endpoint<String>) = TestTyped::on(
    //     async |Payload(_str): Payload<()>, State(_str): State<String>| {},
    // );
    // let _: (_, Endpoint<()>) =
    //     TestTyped::on(async |Payload(_): Payload<()>| {});
    // let _: (_, Endpoint<String>) =
    //     TestTyped::on(async |State(_str): State<String>| {});
    // ```

    let _: Router = Router::new().route_typed(Message::on(async || {}));
}
