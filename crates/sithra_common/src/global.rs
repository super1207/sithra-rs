use std::{path::PathBuf, sync::OnceLock};

use crate::error::BotError;

pub static DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_data_path(path: PathBuf) -> Result<(), BotError> {
    DATA_PATH.set(path).map_err(|_| BotError::InitializeError)
}

#[macro_export]
macro_rules! data_path {
    () => {
        $crate::global::DATA_PATH
            .get_or_init(|| ::std::path::PathBuf::from(format!("./{}", env!("CARGO_PKG_NAME"))))
    };
}
