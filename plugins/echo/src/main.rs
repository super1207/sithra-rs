use event::MessageEventFlattened as Message;
use ioevent::{prelude::*, rpc::*};
use log::info;
use sithra_common::prelude::*;

const SUBSCRIBERS: &[ioevent::Subscriber<CommonState>] =
    &[create_subscriber!(poke_reply), create_subscriber!(echo_msg)];

#[subscriber]
async fn echo_msg(state: State<CommonState>, msg: Message) -> Result {
    if msg.starts_with("echo ") {
        info!("echo 插件收到{}发送的消息", msg.sender.call_name());
        let message = msg.message.clone().trim_start_matches("echo ");
        let id = msg.reply(&state, message.clone()).await?;
        info!("echo 插件回复消息: {:?}", id);
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
            if target_id == state.self_id() {
                info!("是在戳我！");
                let message = vec![
                    MessageNode::At(user_id.into()),
                    MessageNode::Text(" 你要干嘛~".to_string()),
                ];
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
