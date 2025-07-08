use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use ulid::Ulid;

use crate::extract::FromRequest;

pub struct Correlation(pub Ulid);

impl Deref for Correlation {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Correlation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Correlation> for Ulid {
    fn from(value: Correlation) -> Self {
        value.0
    }
}

impl From<Ulid> for Correlation {
    fn from(value: Ulid) -> Self {
        Self(value)
    }
}

impl<S: Send + Sync> FromRequest<S> for Correlation {
    type Rejection = Infallible;

    async fn from_request(
        req: triomphe::Arc<sithra_transport::datapack::RequestDataPack>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(req.correlation()))
    }
}
