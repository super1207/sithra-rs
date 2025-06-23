use sithra_transport::datapack::RequestDataPack;
use triomphe::Arc;
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct Request {
    pub data: Arc<RequestDataPack>,
}

impl Request {
    #[must_use]
    pub fn correlation(&self) -> Ulid {
        self.data.correlation()
    }

    #[must_use]
    pub fn into_raw(self) -> Arc<RequestDataPack> {
        self.data
    }

    #[must_use]
    pub const fn from_raw(data: Arc<RequestDataPack>) -> Self {
        Self { data }
    }

    #[must_use]
    pub fn new(data: RequestDataPack) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
}
