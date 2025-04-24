use std::time::Duration;

use ioevent::error::CallSubscribeError;
use ioevent::rpc::ProcedureCallExt;
use ioevent::{Event, State, subscriber};
use scraper::{Html, Selector};
use sithra_common::error::ApiError;
use sithra_common::event::MessageEventFlattened as Message;
use sithra_common::prelude::*;
use sithra_headless_common::*;
use thiserror::Error;
use tokio::time::timeout;

use crate::McToolsState;

enum Action {
    PrevPage,
    NextPage,
    GetCrate(usize),
    Timeout,
}

impl Action {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "N" => Some(Self::NextPage),
            "n" => Some(Self::NextPage),
            "P" => Some(Self::PrevPage),
            "p" => Some(Self::PrevPage),
            str => str.parse::<usize>().map(Self::GetCrate).ok(),
        }
    }
}
#[subscriber]
pub async fn search_mcmod(state: State<McToolsState>, msg: Message) -> Result {
    // 检查消息是否以 "crate " 开头
    if !msg.starts_with("mcmod ") {
        return Ok(());
    }

    // 提取搜索关键字
    let message = msg.clone().trim_start_matches("mcmod ");
    if message.len() != 1 {
        return Ok(());
    }

    let Some(MessageNode::Text(text)) = message.first() else {
        return Ok(());
    };

    // 执行搜索
    let query = text.trim();
    let mut result = match search_mod(query).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("mcmod 搜索失败: {}", e);
            msg.reply(&state, vec![MessageNode::Text("搜索失败喵。".to_string())])
                .await?;
            return Ok(());
        }
    };

    // 检查搜索结果是否为空
    if result.results.is_empty() {
        msg.reply(
            &state,
            vec![MessageNode::Text("啥都没搜到喵。".to_string())],
        )
        .await?;
        return Ok(());
    }

    // 显示初始搜索结果
    let mut prev_output = send_search_results(&state, &msg, &result).await?;

    // 交互式循环处理用户命令
    loop {
        let user_cmd = match wait_user_command(&state, &msg, Duration::from_secs(15)).await {
            Ok(cmd) => cmd,
            Err(_) => {
                msg.reply(&state, vec![MessageNode::Text("操作超时喵。".to_string())])
                    .await?;
                break;
            }
        };

        match user_cmd {
            // 下一页
            Action::NextPage => {
                match result.get_next_page().await {
                    Some(Ok(new_result)) => {
                        result = new_result;
                        let new_msg_id = send_search_results(&state, &msg, &result).await?;
                        delete_previous_message(&state, &prev_output).await?;
                        prev_output = new_msg_id;
                    }
                    None => {
                        // 没有下一页，不做任何操作
                    }
                    Some(Err(e)) => {
                        msg.reply(
                            &state,
                            vec![MessageNode::Text(format!("获取数据失败喵: {}", e))],
                        )
                        .await?;
                        break;
                    }
                }
            }

            // 上一页
            Action::PrevPage => {
                match result.get_prev_page().await {
                    Some(Ok(new_result)) => {
                        result = new_result;
                        let new_msg_id = send_search_results(&state, &msg, &result).await?;
                        delete_previous_message(&state, &prev_output).await?;
                        prev_output = new_msg_id;
                    }
                    None => {
                        // 没有上一页，不做任何操作
                    }
                    Some(Err(e)) => {
                        msg.reply(
                            &state,
                            vec![MessageNode::Text(format!("获取数据失败喵: {}", e))],
                        )
                        .await?;
                        break;
                    }
                }
            }

            // 超时
            Action::Timeout => {
                msg.reply(&state, vec![MessageNode::Text("操作超时喵。".to_string())])
                    .await?;
                delete_previous_message(&state, &prev_output).await?;
                break;
            }

            // 数字索引 - 获取特定的 crate
            Action::GetCrate(i) => {
                /* if let Some(scrate) = result
                    .get_n_crate_readme_forward(state.self_id.into(), i)
                    .await
                {
                    // 发送 README
                    let forward = state.call(&CreateForwardMsgParams::new(scrate)).await??;
                    let forward_msg = vec![MessageNode::Forward(forward.into())];
                    let _ = msg.reply(&state, forward_msg).await?;

                    // 删除前一个消息
                    delete_previous_message(&state, &prev_output).await?;
                } else {
                    msg.reply(
                        &state,
                        vec![MessageNode::Text("你确定是这个索引喵？".to_string())],
                    )
                    .await?;
                } */
                if let Ok(url) = result.get_url(i) {
                    log::debug!("尝试网页截图: {}", url);
                    let screenshot_params = TakeScreenshot {
                        preprocess_script: Some(
                            "document.querySelector(\".common-text\").style.padding = \"15px\";"
                                .to_string(),
                        ),
                        url,
                        selector: Some(".common-text".to_string()),
                    };
                    let img = state.call(&screenshot_params).await?;
                    if let TakeScreenshotResponse::Success(img) = img {
                        let img_url = format!("file://{}", img);
                        let img_msg = vec![MessageNode::Image(img_url)];
                        let _ = msg.reply(&state, img_msg).await?;
                    } else {
                        msg.reply(
                            &state,
                            vec![MessageNode::Text("图片渲染失败喵。".to_string())],
                        )
                        .await?;
                    }
                    delete_previous_message(&state, &prev_output).await?;
                } else {
                    msg.reply(
                        &state,
                        vec![MessageNode::Text("你确定是这个索引喵？".to_string())],
                    )
                    .await?;
                }
                break;
            }
        }
    }

    Ok(())
}

pub async fn search_mod(name: &str) -> Result<SearchModData, McModError> {
    let url = format!("https://search.mcmod.cn/s?key={}", name);
    SearchModData::get_from_url(&url).await
}

pub struct SearchModData {
    pub results: Vec<SearchModResult>,
    pub next_page: Option<String>,
    pub prev_page: Option<String>,
}
pub struct SearchModResult {
    pub url: String,
    pub name: String,
}

#[allow(unused)]
impl SearchModData {
    pub async fn get_from_url(url: &str) -> Result<Self, McModError> {
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
        let page_selector = Selector::parse("a.page-link")?;
        let pages = doc.select(&page_selector);
        let (next_page, prev_page) =
            pages.fold((None, None), |(mut next_page, mut prev_page), e| {
                let url = e.attr("href").unwrap_or_default();
                if url.contains("后页") {
                    next_page = Some(url);
                }
                if url.contains("前页") {
                    prev_page = Some(url);
                }
                (next_page, prev_page)
            });
        let result = result
            .map(|(name, url)| SearchModResult {
                name,
                url: url.to_string(),
            })
            .filter(|f| f.url.contains("https://www.mcmod.cn/class"));
        Ok(SearchModData {
            results: result.collect(),
            next_page: next_page.map(|url| url.to_string()),
            prev_page: prev_page.map(|url| url.to_string()),
        })
    }
    pub async fn get_next_page(&self) -> Option<Result<Self, McModError>> {
        if let Some(url) = &self.next_page {
            let url = format!("https://search.mcmod.cn/{}", url);
            let data = Self::get_from_url(&url).await;
            Some(data)
        } else {
            None
        }
    }
    pub async fn get_prev_page(&self) -> Option<Result<Self, McModError>> {
        if let Some(url) = &self.prev_page {
            let url = format!("https://search.mcmod.cn/{}", url);
            let data = Self::get_from_url(&url).await;
            Some(data)
        } else {
            None
        }
    }
    fn to_forward_message(&self, user_id: UserId) -> Vec<ForwardMessageNode> {
        let mut msg = Vec::new();
        let total = format!("当前页有 {} 个结果喵。", self.results.len());
        msg.push(ForwardMessageNode::new(
            user_id.clone(),
            "".to_string(),
            vec![MessageNode::Text(total)],
        ));
        for (i, e) in self.results.iter().enumerate() {
            let forward = ForwardMessageNode::new(
                user_id.clone(),
                "".to_string(),
                vec![MessageNode::Text(format!("[{}] {}", i + 1, e.name))],
            );
            msg.push(forward);
        }
        let (has_next, has_prev) = (self.next_page.is_some(), self.prev_page.is_some());
        let page_str = match (has_next, has_prev) {
            (true, true) => "回复[P/N]可以查看[上一页/下一页]喵",
            (true, false) => "回复[N]可以查看[下一页]喵",
            (false, true) => "回复[P]可以查看[上一页]喵",
            (false, false) => "没有更多了捏",
        };
        msg.push(ForwardMessageNode::new(
            user_id.clone(),
            "".to_string(),
            vec![MessageNode::Text(page_str.to_string())],
        ));
        msg
    }
    fn get_url(&self, id: usize) -> Result<String, McModError> {
        Ok(self
            .results
            .get(id - 1)
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

/// 发送搜索结果并返回消息ID
async fn send_search_results(
    state: &State<McToolsState>,
    msg: &Message,
    result: &SearchModData,
) -> Result<MessageIdResponse, CallSubscribeError> {
    let forward_raw = result.to_forward_message(state.self_id.into());
    let forward = state
        .call(&CreateForwardMsgParams::new(forward_raw))
        .await??;
    let forward_msg = vec![MessageNode::Forward(forward.into())];
    let message_id = msg.reply(state, forward_msg).await?;
    Ok(message_id)
}

/// 删除之前的消息
async fn delete_previous_message(
    state: &State<McToolsState>,
    response: &MessageIdResponse,
) -> Result<(), CallSubscribeError> {
    let delete_params = DeleteMsgParams::new(response.message_id.clone())
        .map_err(|e| CallSubscribeError::Other(e.to_string()))?;
    state.call(&delete_params).await??;
    Ok(())
}

/// 等待用户命令，带超时
async fn wait_user_command(
    state: &State<McToolsState>,
    original_msg: &Message,
    timeout_duration: Duration,
) -> Result<Action, CallSubscribeError> {
    // 复制我们需要比较的用户ID，避免引用生命周期问题
    let user_id = original_msg.sender.user_id;

    // 创建用于等待用户消息的 Future
    let wait_future = state
        .wait_next(move |e| {
            // 检查事件是否是消息
            if !Message::SELECTOR.match_event(e) {
                return None;
            }

            // 尝试将事件转换为消息
            let next_msg = match Message::try_from(e) {
                Ok(msg) => msg,
                Err(_) => return None,
            };

            // 检查消息发送者是否相同（使用复制的ID）
            if user_id != next_msg.sender.user_id {
                return None;
            }

            if next_msg.len() != 1 {
                return None;
            }

            // 获取文本内容
            if let Some(MessageNode::Text(text)) = next_msg.first() {
                let text = text.trim();
                Action::parse(&text)
            } else {
                None
            }
        })
        .await;

    // 使用超时等待命令
    match timeout(timeout_duration, wait_future).await {
        Ok(Ok(action)) => Ok(action),
        Ok(Err(e)) => Err(CallSubscribeError::Other(format!(
            "等待用户操作失败: {}",
            e
        ))),
        Err(_) => Ok(Action::Timeout), // 超时情况
    }
}

#[derive(Debug, Error)]
pub enum McModError {
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("selector error: {0}")]
    SelectorError(String),
    #[error("api error: {0}")]
    ApiError(#[from] ApiError),
    #[error("call subscribe error: {0}")]
    CallSubscribeError(#[from] CallSubscribeError),
    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for McModError {
    fn from(err: scraper::error::SelectorErrorKind<'a>) -> Self {
        McModError::SelectorError(err.to_string())
    }
}
