use std::future::pending;

use ioevent::{prelude::*, rpc::*};
use sithra_common::{event, main_loop, prelude::MessageNode};

const SUBSCRIBERS: &[ioevent::Subscriber<DefaultProcedureWright>] = &[create_subscriber!(echo_msg)];

#[subscriber]
async fn echo_msg(state: State<DefaultProcedureWright>, msg: event::MessageDetail) -> Result {
    let msg = msg.flatten();
    if msg.message.len() > 0 {
        if let Some(MessageNode::Text(text)) = msg.message.first() {
            if text.starts_with("echo ") {
                let mut message = Vec::new();
                let text = text.trim_start_matches("echo ");
                message.push(MessageNode::Text(text.to_string()));
                msg.reply(&state, message).await.unwrap();
            }
        }
    }
    Ok(())
}

#[main_loop(subscribers = SUBSCRIBERS, state = DefaultProcedureWright::default())]
async fn main_loop(_effect_wright: &ioevent::EffectWright) {
    pending::<()>().await
}
