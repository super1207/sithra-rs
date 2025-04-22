use crate::data::*;
use crate::error::*;

pub async fn search_cratesio(query: &str) -> Result<CratesioSearchResult, SearchCratesioError> {
    // 从第一页开始，每页10个结果
    let url = format!(
        "https://crates.io/api/v1/crates?page=1&per_page=10&q={}",
        query
    );
    let response = reqwest::get(url).await?;
    let body = response.json::<CratesioSearchResult>().await?;
    Ok(body)
}

pub async fn next_page(
    result: &CratesioSearchResult,
) -> Result<Option<CratesioSearchResult>, SearchCratesioError> {
    if let Some(next_page) = &result.meta.next_page {
        let url = format!("https://crates.io/api/v1/crates{}", next_page);
        let response = reqwest::get(url).await?;
        let body = response.json::<CratesioSearchResult>().await?;
        Ok(Some(body))
    } else {
        Ok(None)
    }
}

pub async fn prev_page(
    result: &CratesioSearchResult,
) -> Result<Option<CratesioSearchResult>, SearchCratesioError> {
    if let Some(prev_page) = &result.meta.prev_page {
        let url = format!("https://crates.io/api/v1/crates{}", prev_page);
        let response = reqwest::get(url).await?;
        let body = response.json::<CratesioSearchResult>().await?;
        Ok(Some(body))
    } else {
        Ok(None)
    }
}

pub async fn get_readme(for_crate: &CratesioCrate) -> Result<String, SearchCratesioError> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/readme",
        for_crate.id, for_crate.newest_version
    );
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}
