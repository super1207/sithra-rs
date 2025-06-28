use sithra_transport::channel::Channel;

use crate::{extract::FromRequest, response};

impl<S: Send + Sync> FromRequest<S> for Channel {
    type Rejection = response::Error<&'static str>;

    async fn from_request(
        req: triomphe::Arc<sithra_transport::datapack::RequestDataPack>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(req.channel.clone().ok_or("Expected channel in request")?)
    }
}
