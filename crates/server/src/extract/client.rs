use std::convert::Infallible;

use crate::{
    extract::{FromRequest, context::Clientful},
    server::Client,
};

impl<S: Send + Sync + Clientful> FromRequest<S> for Client {
    type Rejection = Infallible;

    async fn from_request(
        _req: triomphe::Arc<sithra_transport::datapack::RequestDataPack>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.client().clone())
    }
}
