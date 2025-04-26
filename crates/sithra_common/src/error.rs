use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum BotError {
    #[error("invalid channel type")]
    InvalidChannelType,
    #[error("initialize error")]
    InitializeError,
}
