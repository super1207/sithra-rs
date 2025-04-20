use ioevent::{prelude::*, rpc::*};
use log::info;
use sithra_common::prelude::*;

const SUBSCRIBERS: &[ioevent::Subscriber<CommonState>] =
    &[create_subscriber!(echo_msg), create_subscriber!(poke_reply)];

#[subscriber]
async fn echo_msg(state: State<CommonState>, msg: event::MessageEvent) -> Result {
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

#[subscriber]
async fn poke_reply(state: State<CommonState>, msg: event::NotifyEvent) -> Result {
    match msg {
        event::NotifyEvent::Poke {
            group_id,
            user_id,
            target_id,
        } => {
            info!(
                "收到戳一戳通知: 群ID: {}, 用户ID: {}, 目标ID: {}",
                group_id, user_id, target_id
            );
            if target_id == state.self_id() {
                info!("是在戳我！");
                let message = vec![MessageNode::Text("喵呜~".to_string())];
                let msg = SendGroupMsgParams::new(group_id.into(), message)?;
                let _ = state.call(&msg).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = CommonState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("echo 示例插件启动成功");
}
