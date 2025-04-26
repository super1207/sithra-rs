use scraper::{Html, Selector};
use serde::Deserialize;
use sithra_common::prelude::{ForwardMessageNode, MessageNode, UserId};

use crate::{api, error::SearchCratesioError};

#[derive(Debug, Deserialize)]
pub struct CratesioSearchResult {
    pub crates: Vec<CratesioCrate>,
    pub meta: CratesioSearchMeta,
}

#[allow(unused)]
impl CratesioSearchResult {
    pub fn is_empty(&self) -> bool {
        self.crates.is_empty()
    }

    pub fn has_next_page(&self) -> bool {
        self.meta.next_page.is_some()
    }

    pub fn has_prev_page(&self) -> bool {
        self.meta.prev_page.is_some()
    }

    pub async fn next_page(&self) -> Result<Option<CratesioSearchResult>, SearchCratesioError> {
        api::next_page(self).await
    }

    pub async fn prev_page(&self) -> Result<Option<CratesioSearchResult>, SearchCratesioError> {
        api::prev_page(self).await
    }

    pub fn total(&self) -> u64 {
        self.meta.total
    }

    pub fn to_forward_message(&self, user_id: UserId) -> Vec<ForwardMessageNode> {
        let mut nodes = Vec::new();
        let total = self.total();
        let total_str = format!(
            "一共找到了 {} 个结果捏，15 喵内回复编号可以查看具体信息哦",
            total
        );
        let total_message = MessageNode::Text(total_str);
        let total_node =
            ForwardMessageNode::new(user_id.clone(), "total".to_string(), vec![total_message]);
        nodes.push(total_node);
        for (index, scrate) in self.crates.iter().enumerate() {
            let message = format!(
                "{}. {}:\n介绍: {}\n最新版本: {}\n下载量: {}",
                index + 1,
                scrate.name,
                scrate.description,
                scrate.newest_version,
                scrate.downloads
            );
            let message = MessageNode::Text(message);
            let node =
                ForwardMessageNode::new(user_id.clone(), format!("{}", index + 1), vec![message]);
            nodes.push(node);
        }
        let (has_next, has_prev) = (self.has_next_page(), self.has_prev_page());
        let page_str = match (has_next, has_prev) {
            (true, true) => "回复[P/N]可以查看[上一页/下一页]喵",
            (true, false) => "回复[N]可以查看[下一页]喵",
            (false, true) => "回复[P]可以查看[上一页]喵",
            (false, false) => "没有更多了捏",
        };
        let page_message = MessageNode::Text(page_str.to_string());
        let page_node =
            ForwardMessageNode::new(user_id.clone(), "page".to_string(), vec![page_message]);
        nodes.push(page_node);

        nodes
    }

    pub async fn get_n_crate_readme(&self, n: usize) -> Option<Vec<MessageNode>> {
        let scrate = self.crates.get(n - 1)?;
        let readme = api::get_readme(scrate).await.ok()?;
        let doc = Html::parse_document(&readme);
        let selector = Selector::parse("body > *").ok()?;
        let result = doc.select(&selector);
        let result = result.fold(Vec::new(), |mut acc, e| {
            let text = e
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .as_str()
                .trim()
                .to_string();
            if !text.is_empty() {
                acc.push(MessageNode::Text(text));
            }
            acc
        });
        Some(result)
    }

    pub async fn get_n_crate_readme_forward(
        &self,
        user_id: UserId,
        n: usize,
    ) -> Option<Vec<ForwardMessageNode>> {
        let readme = self.get_n_crate_readme(n).await?;
        let scrate = self.crates.get(n - 1)?;
        let mut messages = Vec::new();
        let message = MessageNode::Text(format!(
            "包名: {}\n版本: {}",
            scrate.name, scrate.newest_version
        ));
        messages.push(message);
        messages.extend(readme);
        let result = messages.into_iter().map(|msg| {
            ForwardMessageNode::new(user_id.clone(), format!("{}", user_id.0), vec![msg])
        });
        Some(result.collect())
    }

    pub fn get_n_page_url(&self, n: usize) -> Option<String> {
        let page = self.crates.get(n - 1)?;
        let url = format!(
            "https://crates.io/crates/{}/{}",
            page.id, page.newest_version
        );
        Some(url)
    }
}

#[derive(Debug, Deserialize)]
pub struct CratesioCrate {
    pub id: String,
    pub name: String,
    pub newest_version: String,
    pub description: String,
    pub downloads: u64,
}

#[derive(Debug, Deserialize)]
pub struct CratesioSearchMeta {
    pub total: u64,
    pub next_page: Option<String>,
    pub prev_page: Option<String>,
}
