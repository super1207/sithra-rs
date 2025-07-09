use ahash::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Config {
    pub raw:    String,
    pub config: HashMap<String, BaseConfig>,
}

/// # Errors
///
/// * `ReadError` - Failed to read config file
/// * `ParseError` - Failed to parse config file
pub fn load_config() -> Result<Config, LoadConfigError> {
    let config_file = std::fs::read_to_string("config.toml")?;
    let config: HashMap<String, BaseConfig> = toml::from_str(&config_file)?;

    Ok(Config {
        raw: config_file,
        config,
    })
}

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("Failed to read config file")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseConfig {
    pub path:   String,
    #[serde(default)]
    pub args:   Vec<String>,
    pub config: Option<toml::Value>,
}

impl Config {
    pub fn iter(&self) -> impl Iterator<Item = (&str, &BaseConfig)> {
        self.config.iter().map(|(key, value)| (key.as_str(), value))
    }
}
