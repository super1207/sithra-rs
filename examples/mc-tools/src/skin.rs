use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ioevent::{subscriber, State};
use sithra_common::prelude::*;
use sithra_common::event::MessageEventFlattened as Message;

use crate::McToolsState;

async fn get_image(url: &str) -> Option<String> {
    match reqwest::get(url).await {
        Ok(response) => response.bytes().await.ok().map(|bytes| {
            let base64 = BASE64_STANDARD.encode(bytes);
            format!("base64://{}", base64)
        }),
        Err(_) => None,
    }
}

async fn handle_mc_command(
    state: State<McToolsState>,
    msg: Message,
    command: &str,
    endpoint: &str,
    error_message: &str,
) -> Result<(), ioevent::error::CallSubscribeError> {
    if msg.starts_with(command) {
        let message = msg.message.clone().trim_start_matches(command);
        if let Some(MessageNode::Text(name)) = message.first() {
            let name = name.trim();
            let url = format!("https://nmsr.nickac.dev/{}/{}", endpoint, name);
            let message = if let Some(image) = get_image(&url).await {
                vec![MessageNode::Image(image)]
            } else {
                vec![MessageNode::Text(error_message.to_string())]
            };
            msg.reply(&state, message).await?;
        }
    }
    Ok(())
}

#[subscriber]
pub async fn mcbody(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcbody ", "fullbody", "找不着你的皮肤捏。").await
}

#[subscriber]
pub async fn mchead(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mchead ", "head", "摸不着头脑捏。").await
}

#[subscriber]
pub async fn mcface(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcface ", "face", "捏不到你的脸捏。").await
}

#[subscriber]
pub async fn mcskin(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcskin ", "skin", "摸不着你的皮肤捏。").await
}