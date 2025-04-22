mod api;
mod data;
mod error;

use std::time::Duration;

use ioevent::error::CallSubscribeError;
use ioevent::rpc::ProcedureCallExt;
use ioevent::{Event, State, create_subscriber, subscriber};
use log::info;
use sithra_common::event::MessageEventFlattened as Message;
use sithra_common::prelude::*;
use sithra_headless_common::TakeScreenshot;
use tokio::time::timeout;

const SUBSCRIBERS: &[ioevent::Subscriber<CommonState>] = &[create_subscriber!(search_cratesio)];

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
            "P" => Some(Self::PrevPage),
            str => str.parse::<usize>().map(Self::GetCrate).ok(),
        }
    }
}

// TODO: 搜索 crates.io 上的包
#[subscriber]
pub async fn search_cratesio(state: State<CommonState>, msg: Message) -> Result {
    // 检查消息是否以 "crate " 开头
    if !msg.starts_with("crate ") {
        return Ok(());
    }

    // 提取搜索关键字
    let message = msg.clone().trim_start_matches("crate ");
    if message.len() != 1 {
        return Ok(());
    }

    let Some(MessageNode::Text(text)) = message.first() else {
        return Ok(());
    };

    // 执行搜索
    let query = text.trim();
    let mut result = match api::search_cratesio(query).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("crates.io 搜索失败: {}", e);
            msg.reply(&state, vec![MessageNode::Text("搜索失败捏。".to_string())])
                .await?;
            return Ok(());
        }
    };

    // 检查搜索结果是否为空
    if result.is_empty() {
        msg.reply(
            &state,
            vec![MessageNode::Text("啥都没搜到捏。".to_string())],
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
                msg.reply(&state, vec![MessageNode::Text("操作超时捏。".to_string())])
                    .await?;
                break;
            }
        };

        match user_cmd {
            // 下一页
            Action::NextPage => {
                match result.next_page().await {
                    Ok(Some(new_result)) => {
                        result = new_result;
                        let new_msg_id = send_search_results(&state, &msg, &result).await?;
                        delete_previous_message(&state, &prev_output).await?;
                        prev_output = new_msg_id;
                    }
                    Ok(None) => {
                        // 没有下一页，不做任何操作
                    }
                    Err(_) => {
                        msg.reply(
                            &state,
                            vec![MessageNode::Text("获取数据失败捏。".to_string())],
                        )
                        .await?;
                        break;
                    }
                }
            }

            // 上一页
            Action::PrevPage => {
                match result.prev_page().await {
                    Ok(Some(new_result)) => {
                        result = new_result;
                        let new_msg_id = send_search_results(&state, &msg, &result).await?;
                        delete_previous_message(&state, &prev_output).await?;
                        prev_output = new_msg_id;
                    }
                    Ok(None) => {
                        // 没有上一页，不做任何操作
                    }
                    Err(_) => {
                        msg.reply(
                            &state,
                            vec![MessageNode::Text("获取数据失败捏。".to_string())],
                        )
                        .await?;
                        break;
                    }
                }
            }

            // 超时
            Action::Timeout => {
                msg.reply(&state, vec![MessageNode::Text("操作超时捏。".to_string())])
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
                if let Some(url) = result.get_n_page_url(i) {
                    log::debug!("尝试网页截图: {}", url);
                    let screenshot_params = TakeScreenshot {
                        url,
                        selector: None,
                    };
                    let img = state.call(&screenshot_params).await?;
                    let img_url = format!("file://{}", img.file_path);
                    let img_msg = vec![MessageNode::Image(img_url)];
                    let _ = msg.reply(&state, img_msg).await?;
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

/// 发送搜索结果并返回消息ID
async fn send_search_results(
    state: &State<CommonState>,
    msg: &Message,
    result: &data::CratesioSearchResult,
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
    state: &State<CommonState>,
    response: &MessageIdResponse,
) -> Result<(), CallSubscribeError> {
    let delete_params = DeleteMsgParams::new(response.message_id.clone())
        .map_err(|e| CallSubscribeError::Other(e.to_string()))?;
    state.call(&delete_params).await??;
    Ok(())
}

/// 等待用户命令，带超时
async fn wait_user_command(
    state: &State<CommonState>,
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
                let text = text.trim().to_string();
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

#[sithra_common::main(subscribers = SUBSCRIBERS, state = CommonState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("crates.io 搜索插件启动成功");
}
