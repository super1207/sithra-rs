mod mcmod;
use mcmod::*;

use base64::{Engine, prelude::BASE64_STANDARD};
use event::MessageEventFlattened as Message;
use ioevent::{
    prelude::*,
    rpc::{DefaultProcedureWright, ProcedureCallWright},
};
use log::info;
use sithra_common::prelude::*;

const SUBSCRIBERS: &[ioevent::Subscriber<McToolsState>] = &[
    create_subscriber!(mcbody),
    create_subscriber!(mchead),
    create_subscriber!(mcface),
    create_subscriber!(mcskin),
    create_subscriber!(search_mcmod),
];

#[derive(Clone)]
struct McToolsState {
    self_id: u64,
    pcw: DefaultProcedureWright,
}
impl SithraState for McToolsState {
    fn create(self_id: u64) -> Self {
        Self {
            self_id,
            pcw: DefaultProcedureWright::default(),
        }
    }
    fn self_id(&self) -> u64 {
        self.self_id
    }
}

impl ProcedureCallWright for McToolsState {
    async fn next_echo(&self) -> u64 {
        self.pcw.next_echo().await
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
async fn mcbody(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcbody ", "fullbody", "找不着你的皮肤捏。").await
}

#[subscriber]
async fn mchead(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mchead ", "head", "摸不着头脑捏。").await
}

#[subscriber]
async fn mcface(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcface ", "face", "捏不到你的脸捏。").await
}

#[subscriber]
async fn mcskin(state: State<McToolsState>, msg: Message) -> Result {
    handle_mc_command(state, msg, "mcskin ", "skin", "摸不着你的皮肤捏。").await
}

async fn get_image(url: &str) -> Option<String> {
    match reqwest::get(url).await {
        Ok(response) => response.bytes().await.ok().map(|bytes| {
            let base64 = BASE64_STANDARD.encode(bytes);
            format!("base64://{}", base64)
        }),
        Err(_) => None,
    }
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = McToolsState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("mc-tools 插件启动成功");
    log::set_max_level(log::LevelFilter::Info);
}
