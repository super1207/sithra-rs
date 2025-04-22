use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchCratesioError {
    #[error("请求失败: {0}")]
    RequestError(#[from] reqwest::Error),
}
