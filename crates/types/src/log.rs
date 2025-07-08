use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub level:   Level,
    pub message: String,
    pub target:  String,
}

impl Log {
    #[must_use]
    pub fn new(level: impl Into<Level>, message: String, target: String) -> Self {
        Self {
            level: level.into(),
            message,
            target,
        }
    }
}

pub mod command {
    use sithra_server::typed;
    use sithra_transport::datapack::RequestDataPack;

    use super::Log;
    use crate::into_response;

    typed!("/log.create" => impl Log);
    into_response!("/log.create", Log);

    impl From<Log> for RequestDataPack {
        fn from(value: Log) -> Self {
            Self::default().payload(value).path("/log.create")
        }
    }
}
