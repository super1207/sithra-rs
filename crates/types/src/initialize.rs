use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Initialize<C> {
    pub config: C,
}

impl<C> Initialize<C> {
    pub const fn new(config: C) -> Self {
        Self { config }
    }
}

impl<C> Initialize<C>
where
    C: for<'de> Deserialize<'de>,
{
    /// # Errors
    /// Returns an error if the provided value cannot be deserialized into the
    /// config type.
    pub fn from_value(value: rmpv::Value) -> Result<Self, rmpv::ext::Error> {
        let config = rmpv::ext::from_value(value)?;
        Ok(Self { config })
    }
}

pub mod command {
    use super::Initialize;

    #[allow(dead_code)]
    impl<C> Initialize<C> {
        /// Create a new endpoint for the given route and handler.
        #[doc = concat!("Path: `","/initialize","`\n\n")]
        /// Allowed payload:
        pub fn on<H, T, S>(
            handler: H,
        ) -> (
            &'static str,
            sithra_server::routing::endpoint::Endpoint<S, ::std::convert::Infallible>,
        )
        where
            H: sithra_server::handler::Handler<T, S>,
            T: 'static,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            (
                "/initialize",
                sithra_server::routing::endpoint::Endpoint::BoxedHandler(
                    sithra_server::boxed::BoxedIntoRoute::from_handler(handler),
                ),
            )
        }

        #[doc(hidden)]
        pub fn __on<H, T, S>(
            handler: H,
        ) -> sithra_server::routing::endpoint::Endpoint<S, ::std::convert::Infallible>
        where
            H: sithra_server::handler::Handler<T, S>,
            T: 'static,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            sithra_server::routing::endpoint::Endpoint::BoxedHandler(
                sithra_server::boxed::BoxedIntoRoute::from_handler(handler),
            )
        }

        #[doc(hidden)]
        #[must_use]
        pub const fn path() -> &'static str {
            "/initialize"
        }

        #[doc(hidden)]
        pub const fn _check<H, T, S>(_handler: &H) -> &'static str
        where
            H: sithra_server::handler::Handler<T, S>,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            "/initialize"
        }

        #[doc(hidden)]
        pub const fn __check<H, T, S>(handler: H) -> H
        where
            H: sithra_server::handler::Handler<T, S>,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            handler
        }
    }
}
