use ioevent::{prelude::*, rpc::*};
use log::info;
use sithra_common::{event, prelude::MessageNode};

const SUBSCRIBERS: &[ioevent::Subscriber<DefaultProcedureWright>] = &[create_subscriber!(echo_msg)];

#[subscriber]
async fn echo_msg(state: State<DefaultProcedureWright>, msg: event::MessageDetail) -> Result {
    let msg = msg.flatten();
    if msg.message.len() > 0 {
        if let Some(MessageNode::Text(text)) = msg.message.first() {
            if text.starts_with("echo ") {
                info!("echo 插件收到消息: {}", text);
                let mut message = Vec::new();
                let text = text.trim_start_matches("echo ");
                message.push(MessageNode::Text(text.to_string()));
                msg.reply(&state, message.clone()).await.unwrap();
                info!("echo 插件回复消息: {}", text);
            }
        }
    }
    Ok(())
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = DefaultProcedureWright::default())]
async fn main(_effect_wright: &ioevent::EffectWright) {
    log::info!("echo 示例插件启动成功");
}