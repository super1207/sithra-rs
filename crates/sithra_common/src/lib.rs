/* old version
pub mod api;
pub mod event;
pub mod message;
pub mod traits;
pub use sithra_macro::*;
pub mod state;

pub mod hello {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    pub enum Hello {
        H1(String),
        #[serde(other)]
        H2,
    }
}

pub mod error {
    pub use crate::api::error::*;
    pub use ioevent::error::*;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum BotError {
        #[error("api error: {0}")]
        ApiError(ApiError),
        #[error("call subscribe error: {0}")]
        CallSubscribeError(CallSubscribeError),
    }
    impl From<ApiError> for BotError {
        fn from(value: ApiError) -> Self {
            BotError::ApiError(value)
        }
    }
    impl From<CallSubscribeError> for BotError {
        fn from(value: CallSubscribeError) -> Self {
            BotError::CallSubscribeError(value)
        }
    }
    impl From<BotError> for CallSubscribeError {
        fn from(value: BotError) -> Self {
            match value {
                BotError::ApiError(e) => e.into(),
                BotError::CallSubscribeError(e) => e,
            }
        }
    }
}

pub mod prelude {
    pub use crate::event;
    pub use crate::message::*;
    pub use crate::traits::*;
    pub use crate::api::*;
    pub use crate::state::*;
    pub use sithra_macro::*;
}
 */

pub mod log;