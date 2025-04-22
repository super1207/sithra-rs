mod mcmod; // 用于搜索 mcmod 站
mod skin; // 用于查询玩家皮肤
use mcmod::*;
use skin::*;

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
#[sithra_common::main(subscribers = SUBSCRIBERS, state = McToolsState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("mc-tools 插件启动成功");
    log::set_max_level(log::LevelFilter::Info);
}
