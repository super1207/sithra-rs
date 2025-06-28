use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use serde::Deserialize;
use sithra_transport::datapack::RequestDataPack;
use triomphe::Arc;

use crate::{
    extract::{FromRequest, from_ref::FromRef},
    request::Request,
    response::Error,
    server::Client,
};

pub struct Context<T: for<'de> Deserialize<'de>, S> {
    pub state:         S,
    pub request:       Request,
    pub payload_cache: T,
    _marker:           PhantomData<T>,
}

impl<T, S> Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    /// # Errors
    ///
    /// Returns an error if the payload cannot be deserialized.
    pub const fn payload(&self) -> &T {
        &self.payload_cache
    }
}

impl<T, S> Deref for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T, S> DerefMut for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<OuterState, InnerState, T> FromRequest<OuterState> for Context<T, InnerState>
where
    InnerState: FromRef<OuterState>,
    OuterState: Send + Sync,
    T: for<'de> Deserialize<'de>,
{
    type Rejection = Error<rmpv::ext::Error>;

    async fn from_request(
        parts: Arc<RequestDataPack>,
        state: &OuterState,
    ) -> Result<Self, Self::Rejection> {
        let request = Request::from(parts);
        let payload_cache = request.payload()?;
        Ok(Self {
            state: InnerState::from_ref(state),
            request,
            payload_cache,
            _marker: PhantomData,
        })
    }
}

pub trait Clientful {
    fn client(&self) -> &Client;
}

impl Clientful for Client {
    fn client(&self) -> &Client {
        self
    }
}

impl<T, S> Clientful for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
    S: Clientful,
{
    fn client(&self) -> &Client {
        self.state.client()
    }
}
