use std::{
    num::{IntErrorKind, ParseIntError},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sithra_kit::{
    plugin::Plugin,
    server::{
        extract::context::{Clientful, Context},
        server::Client,
    },
    transport::channel::Channel,
    types::{
        channel::SetMute,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use triomphe::Arc;

macro_rules! tap_err {
    ($val:ident, $action:expr) => {
        match $val {
            Ok(res) => res,
            Err(err) => {
                log::error!(concat!("Failed to ", $action, " channel: {:?}"), err);
                return Some(msg!(H[text: concat!($action, "失败喵，请通过日志查看错误信息喵。")]).into());
            }
        }
    };
}

type Ctx<T> = Context<T, AppState>;

#[derive(Clone)]
struct AppState {
    admins: Arc<Vec<String>>,
    client: Client,
}

impl Clientful for AppState {
    fn client(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    admins: Vec<String>,
}

#[tokio::main]
async fn main() {
    let (plugin, config) = Plugin::<Config>::new().await.unwrap();
    let client = plugin.server.client();
    let state = AppState {
        admins: Arc::new(config.admins),
        client,
    };
    let plugin = plugin.map(move |r| r.route_typed(Message::on(mute)).with_state(state));
    log::info!("Management Tools plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn mute(ctx: Ctx<Message<H>>, mut channel: Channel) -> Option<SendMessage> {
    let params = parse_cmd(&ctx.content);

    let (id, duration) = match params {
        Ok(ok) => ok,
        Err(ParseErr::InvalidNumber) => return Some(msg!(H[text: "无效的数字喵"]).into()),
        Err(ParseErr::NotEnoughParams) => {
            return Some(msg!(H[text: "需要俩参数喵, 用户ID和时间喵"]).into());
        }
        Err(ParseErr::NotMatch) => return None,
    };

    if !auth(&channel.id, &ctx.state.admins) {
        return Some(msg!(H[text: "你没有权限喵"]).into());
    }

    let is_unmute = duration.is_zero();
    let duration_secs = duration.as_secs();

    id.clone_into(&mut channel.id);

    let set_mute = SetMute { channel, duration };
    let res = ctx.post(set_mute);
    let res = tap_err!(res, "禁言").await;
    tap_err!(res, "禁言");
    log::info!("mute user {id} for {duration_secs} seconds");
    Some(
        msg!(H [
            text: if is_unmute {"解禁成功喵"} else {"禁言成功喵"}
        ])
        .into(),
    )
}

fn auth(user: &String, admins: &[String]) -> bool {
    admins.contains(user)
}

fn parse_cmd(segs: &[H]) -> Result<(&str, Duration), ParseErr> {
    match segs {
        [H::Text(cmd), H::At(user_id), H::Text(duration)] if cmd.trim() == "mute" => {
            let duration = Duration::from_secs(duration.trim().parse()?);
            Ok((user_id.trim(), duration))
        }
        [H::Text(cmd), ..] if cmd.trim() == "mute" => Err(ParseErr::NotEnoughParams),
        [H::Text(cmd)] => {
            if let Some(params) = cmd.strip_prefix("mute ") {
                let params = params.split_whitespace().map(str::trim).collect::<Vec<_>>();
                if params.len() != 2 {
                    return Err(ParseErr::NotEnoughParams);
                }
                let duration = Duration::from_secs(params[1].parse()?);
                let id = params[0];
                Ok((id, duration))
            } else {
                Err(ParseErr::NotMatch)
            }
        }
        _ => Err(ParseErr::NotMatch),
    }
}

enum ParseErr {
    InvalidNumber,
    NotEnoughParams,
    NotMatch,
}

impl From<ParseIntError> for ParseErr {
    fn from(e: ParseIntError) -> Self {
        match e.kind() {
            IntErrorKind::Empty => Self::NotEnoughParams,
            _ => Self::InvalidNumber,
        }
    }
}
