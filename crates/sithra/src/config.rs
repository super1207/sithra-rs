use std::env;

use log::info;
use serde::Deserialize;
use sithra_common::log::LogLevel;
use tokio::{
    fs,
    io::{self, AsyncReadExt},
};

const DEFAULT_CONFIG: &str = include_str!("../static/config.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub base: BaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct BaseConfig {
    #[serde(rename = "log-level")]
    pub log_level: LogLevel,
}

impl Config {
    pub async fn init() -> anyhow::Result<Self> {
        let current_dir = env::current_dir()?;
        let config_path = current_dir.join("config.toml");
        if !config_path.exists() {
            fs::write(&config_path, DEFAULT_CONFIG).await?;
            info!("请填写 config.toml 文件后按下回车继续");
            io::stdin().read(&mut [0; 1]).await?;
            let config_str = fs::read_to_string(&config_path).await?;
            Ok(toml::from_str(&config_str)?)
        } else {
            Ok(toml::from_str(&fs::read_to_string(&config_path).await?)?)
        }
    }
}
