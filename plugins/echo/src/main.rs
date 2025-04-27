use ioevent::{prelude::*, rpc::*};
use log::info;
use sithra_common::prelude::{
    common::{CommonMessage, CommonMessageExt},
    *,
};

const SUBSCRIBERS: &[ioevent::Subscriber<DefaultProcedureWright>] = &[create_subscriber!(echo_msg)];

#[subscriber]
async fn echo_msg(
    state: State<DefaultProcedureWright>,
    msg: MessageEvent<CommonMessage>,
) -> Result {
    if msg.message().starts_with("echo ") {
        info!("echo 插件收到{}发送的消息", msg.fetch_user().call_name());
        let message = msg.message().clone().trim_start_matches("echo ");
        let reply = msg.build_reply(message);
        state.call(&reply).await?;
    }
    Ok(())
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = DefaultProcedureWright::default())]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("echo 示例插件启动成功");
}
