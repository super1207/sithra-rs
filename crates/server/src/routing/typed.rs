#[macro_export]
macro_rules! typed {
    ($route:expr => impl $typed:ty; $($T:ty),*)=>{
        $(impl<Sta: ::std::marker::Send + ::std::marker::Sync> AllowedParams<Sta> for $crate::extract::payload::Payload<$T> {})*
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

            #[doc(hidden)]
            pub const fn path() -> &'static str {
                $route
            }

            #[doc(hidden)]
            pub const fn _check<H, T, S>(_handler: &H) -> &'static str
            where
                H: $crate::handler::Handler<T, S>,
                T: AllowedParams<S> + 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                $route
            }

            #[doc(hidden)]
            pub const fn __check<H, T, S>(handler: H) -> H
            where
                H: $crate::handler::Handler<T, S>,
                T: AllowedParams<S> + 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                handler
            }
        }
        typed!(@default);
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
    (@default) => {
        impl<Sta> AllowedParams<Sta> for $crate::extract::state::State<Sta> {}
        impl<Sta> AllowedParams<Sta> for $crate::transport::channel::Channel {}
        impl<T, Sta> AllowedParams<Sta> for $crate::extract::context::Context<T, Sta>
        where
            T: AllowedParams<Sta> + for<'de> $crate::__private::Deserialize<'de>,
        {
        }
    };
}

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! on {
    ($handler:ident) => {
        ($crate::path_!($handler), $crate::on($handler))
    };
}

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! router {
    { $router:expr; $($typed:ty[$($handler:expr),*]),* } => {
        ($router)
        $(.route(<$typed>::path(), $crate::multi([$($crate::on(<$typed>::__check($handler)),)*])))*
    }
}

#[cfg(test)]
fn _typed() {
    use sithra_server_macros::on;

    use crate::{
        extract::{payload::Payload, state::State},
        routing::{endpoint::Endpoint, router::Router},
    };

    mod message {
        pub struct Message;
        typed!("/message" => impl Message; String, ());
    }
    mod other {
        pub struct Other;
        typed!("/other" => impl Other; String, ());
    }

    #[on(message::Message)]
    async fn message_handler(Payload(_str): Payload<String>, State(()): State<()>) {}

    let _: (_, Endpoint<()>) = message::Message::on(async || {});
    let _: (_, Endpoint<()>) =
        message::Message::on(async |Payload(_str): Payload<String>, State(()): State<()>| {});
    let _: (_, Endpoint<()>) = message::Message::on(async |Payload(_str): Payload<String>| {});
    let _: (_, Endpoint<()>) = message::Message::on(async |State(_unit): State<()>| {});

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

    let router: Router = Router::new()
        .route_typed(message::Message::on(async || {}))
        .route_typed(on!(message_handler));
    let _ = router! {router;
        message::Message[message_handler, message_handler],
        other::Other[message_handler, message_handler]
    };
}
