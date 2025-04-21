use std::sync::Arc;

use event::MessageEventFlattened as Message;
use ioevent::{
    prelude::*,
    rpc::{DefaultProcedureWright, ProcedureCallWright},
};
use log::info;
use sithra_common::prelude::*;

const SUBSCRIBERS: &[ioevent::Subscriber<McToolsState>] = &[create_subscriber!(mcbody)];

#[derive(Clone)]
struct McToolsState {
    self_id: u64,
    http_client: Arc<reqwest::Client>,
    pcw: DefaultProcedureWright,
}
impl SithraState for McToolsState {
    fn create(self_id: u64) -> Self {
        Self {
            self_id,
            http_client: Arc::new(reqwest::Client::new()),
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

#[subscribe_message]
async fn mcbody(s: State<McToolsState>, msg: &Message) -> Option<Vec<MessageNode>> {
    if msg.starts_with("mcbody ") {
        let message = msg.message.clone().trim_start_matches("mcbody ");
        if let Some(MessageNode::Text(name)) = message.first() {
            let name = name.trim();
            let url = format!("https://nmsr.nickac.dev/fullbody/{}", name);
            let message = if check_url_availability(&s, &url).await {
                vec![MessageNode::Image(url)]
            } else {
                vec![MessageNode::Text("找不着你的皮肤捏。".to_string())]
            };
            return Some(message);
        }
    }
    None
}

async fn check_url_availability(s: &State<McToolsState>, url: &str) -> bool {
    match s.http_client.get(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = McToolsState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("mc-tools 插件启动成功");
}
