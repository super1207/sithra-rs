use std::time::Duration;

use ioevent::error::CallSubscribeError;
use ioevent::rpc::ProcedureCallExt;
use ioevent::{Event, State, subscriber};
use scraper::{Html, Selector};
use sithra_common::error::ApiError;
use sithra_common::event::MessageEventFlattened as Message;
use sithra_common::prelude::*;
use thiserror::Error;
use tokio::time::timeout;

use crate::McToolsState;

#[subscriber]
pub async fn search_mcmod(s: State<McToolsState>, msg: Message) -> Result {
    if !msg.starts_with("mcmod ") {
        return Ok(());
    }

    let message = msg.message.clone().trim_start_matches("mcmod ");
    let name = match message.first() {
        Some(MessageNode::Text(name)) => name.trim(),
        _ => return Ok(()),
    };

    let result = match search_mod(name).await {
        Ok(result) if !result.is_empty() => result,
        Ok(_) => {
            msg.reply(&s, vec![MessageNode::Text("啥都没搜到捏。".to_string())])
                .await?;
            return Ok(());
        }
        Err(e) => {
            log::error!("mcmod 搜索失败: {}", e);
            return Ok(());
        }
    };

    let forward = CreateForwardMsgParams::new(result.to_forward_message(s.self_id().into()));
    let forward_id = s.call(&forward).await??;
    let forward_msg = msg
        .reply(&s, vec![MessageNode::Forward(forward_id.0.into())])
        .await?;

    let sender_id = msg.sender.user_id;
    let next = s
        .wait_next(move |e| {
            let is_message = Message::SELECTOR.match_event(e);
            if !is_message {
                return None;
            }

            let message = match Message::try_from(e) {
                Ok(message) => message,
                Err(_) => return None,
            };

            if message.sender.user_id == sender_id {
                if message.message.len() != 1 {
                    return None;
                }
                match message.first() {
                    Some(MessageNode::Text(text)) => text.parse::<usize>().ok(),
                    _ => None,
                }
            } else {
                None
            }
        })
        .await;

    let next = match timeout(Duration::from_secs(15), next).await {
        Ok(next) => next?,
        Err(_) => {
            msg.reply(&s, vec![MessageNode::Text("超时了捏。".to_string())])
                .await?;
            let del = DeleteMsgParams::new(forward_msg.message_id);
            if let Ok(del) = del {
                if let Err(e) = s.call(&del).await {
                    log::error!("mcmod 撤回消息失败: {}", e);
                }
            }
            return Ok(());
        }
    };

    let result = result.get_content(next - 1, s.self_id().into()).await;

    let del = DeleteMsgParams::new(forward_msg.message_id);
    if let Ok(del) = del {
        if let Err(e) = s.call(&del).await {
            log::error!("mcmod 撤回消息失败: {}", e);
        }
    }

    match result {
        Ok(content) => {
            let forward = CreateForwardMsgParams::new(content);
            let forward_id = s.call(&forward).await??;
            msg.reply(&s, vec![MessageNode::Forward(forward_id.0.into())])
                .await?;
        }
        Err(e) => {
            log::error!("mcmod 获取 URL 失败: {}", e);
            msg.reply(
                &s,
                vec![MessageNode::Text(
                    "诶呀？好像是索引太大，没有这个东西捏。".to_string(),
                )],
            )
            .await?;
        }
    };

    Ok(())
}

pub async fn search_mod(name: &str) -> Result<Vec<SearchModResult>, McModError> {
    let url = format!("https://search.mcmod.cn/s?key={}", name);
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;
    let doc = Html::parse_document(&body);
    let selector = Selector::parse(".result-item > .head > a")?;
    let result = doc.select(&selector);
    let result = result.map(|e| {
        (
            e.text().fold(String::new(), |mut acc, s| {
                acc.push_str(s);
                acc
            }),
            e.attr("href").unwrap_or_default(),
        )
    });
    let result = result.map(|(name, url)| SearchModResult {
        name,
        url: url.to_string(),
    });
    Ok(result.collect())
}

pub struct SearchModResult {
    pub url: String,
    pub name: String,
}

pub trait SearchModResultExt {
    fn to_forward_message(&self, user_id: UserId) -> Vec<ForwardMessageNode>;
    fn get_url(&self, id: usize) -> Result<String, McModError>;
    fn get_content(
        &self,
        id: usize,
        self_id: UserId,
    ) -> impl Future<Output = Result<Vec<ForwardMessageNode>, McModError>>;
}

impl SearchModResultExt for Vec<SearchModResult> {
    fn to_forward_message(&self, user_id: UserId) -> Vec<ForwardMessageNode> {
        self.iter()
            .enumerate()
            .map(|(i, e)| {
                ForwardMessageNode::new(
                    user_id.clone(),
                    "".to_string(),
                    vec![MessageNode::Text(format!("{}: {}", i + 1, e.name))],
                )
            })
            .collect()
    }
    fn get_url(&self, id: usize) -> Result<String, McModError> {
        Ok(self
            .get(id)
            .ok_or(McModError::IndexOutOfBounds(id))?
            .url
            .clone())
    }
    async fn get_content(
        &self,
        id: usize,
        self_id: UserId,
    ) -> Result<Vec<ForwardMessageNode>, McModError> {
        let url = self.get_url(id)?;
        let resp = reqwest::get(url).await?;
        let body = resp.text().await?;
        let doc = Html::parse_document(&body);
        let selector = Selector::parse("li.text-area.common-text > p")?;
        let result = doc.select(&selector);
        // let img_selector = Selector::parse("img")?;
        let result = result.fold(Vec::new(), |mut msg, e| {
            /* let imgs = e.select(&img_selector);
            let mut srcs = Vec::new();
            for img in imgs {
                let src = img.attr("src");
                if let Some(src) = src {
                    srcs.push(src.to_string());
                }
            } */
            let text = e.text().fold(String::new(), |mut acc, s| {
                acc.push_str(s);
                acc
            });
            let mut imsg = Vec::new();
            /* for src in srcs {
                imsg.push(MessageNode::Image(src));
            } */
            if !text.trim().is_empty() {
                imsg.push(MessageNode::Text(text));
            }
            if !imsg.is_empty() {
                msg.push(ForwardMessageNode::new(
                    self_id.clone(),
                    "".to_string(),
                    imsg,
                ));
            }
            msg
        });
        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum McModError {
    #[error("request error: {0}")]
    RequestError(reqwest::Error),
    #[error("selector error: {0}")]
    SelectorError(String),
    #[error("api error: {0}")]
    ApiError(ApiError),
    #[error("call subscribe error: {0}")]
    CallSubscribeError(CallSubscribeError),
    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
impl From<reqwest::Error> for McModError {
    fn from(err: reqwest::Error) -> Self {
        McModError::RequestError(err)
    }
}
impl<'a> From<scraper::error::SelectorErrorKind<'a>> for McModError {
    fn from(err: scraper::error::SelectorErrorKind) -> Self {
        McModError::SelectorError(err.to_string())
    }
}
impl From<ApiError> for McModError {
    fn from(err: ApiError) -> Self {
        McModError::ApiError(err)
    }
}
impl From<CallSubscribeError> for McModError {
    fn from(err: CallSubscribeError) -> Self {
        McModError::CallSubscribeError(err)
    }
}
