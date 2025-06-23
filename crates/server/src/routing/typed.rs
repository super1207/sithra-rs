#[macro_export]
macro_rules! typed {
    ($route:expr => impl $typed:ty; $($T:ty),*)=>{
        $(impl<Sta: ::std::marker::Send + ::std::marker::Sync> AllowedParams<Sta> for $T {})*
        pub trait AllowedParams<Sta> {}
        typed!(@private A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
        #[allow(dead_code)]
        impl $typed {
            #[doc = "Create a new endpoint for the given route and handler.\n\n"]
            #[doc = concat!("Path: `", $route, "`\n\n")]
            #[doc = "Allowed parameters:\n\n"]
            $(#[doc = concat!(" - `", stringify!($T), "`\n")])*
            pub fn on<H, T, S>(handler: H) -> (&'static str, $crate::routing::endpoint::Endpoint<S, ::std::convert::Infallible>)
            where
                H: $crate::handler::Handler<T, S>,
                T: AllowedParams<S> + 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                (
                    $route,
                    $crate::routing::endpoint::Endpoint::BoxedHandler($crate::boxed::BoxedIntoRoute::from_handler(handler)),
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
            $($T: AllowedParams<Sta> + $crate::extract::FromRequest<Sta> + 'static,)*
        {
        }
    };
}

#[cfg(test)]
fn _typed() {
    use crate::{
        extract::{payload::Payload, state::State},
        routing::{endpoint::Endpoint, router::Router},
    };

    struct Message;
    typed!("/message" => impl Message; Payload<String>, State<()>);
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
