#![allow(unused)]
use std::{collections::HashMap, sync::Arc, sync::LazyLock};

use anyhow;
use dashmap::DashMap;
use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use ioevent::{
    EffectWright, State,
    error::CallSubscribeError,
    rpc::{DefaultProcedureWright, ProcedureCallWright, procedure},
};
use log::error;
use rand::{RngCore, SeedableRng, rngs::SmallRng};
/* old version
use sithra_common::{
    api::{
        ApiRequest, ApiRequestKind,
        data::request,
        api_internal::{ApiResponse, ApiResponseKind},
    },
    error::ApiError,
    event::{
        self,
        event_internal::{InternalOnebotEvent, InternalOnebotEventKind},
    },
    message,
}; */
use tokio::{
    net::TcpStream,
    sync::{Mutex, mpsc, oneshot},
};
/* pub use api_receiver::*;
pub use api_sender::*;
pub use msg_receiver::*; */

#[derive(Clone)]
pub struct ClientState {
    pub pcw: DefaultProcedureWright,
}
impl ProcedureCallWright for ClientState {
    fn next_echo(&self) -> impl Future<Output = u64> + Send {
        self.pcw.next_echo()
    }
}
impl ClientState {
    pub fn new() -> Self {
        Self {
            pcw: DefaultProcedureWright::default(),
        }
    }
}

/* old version
#[derive(Clone)]
pub struct ClientState {
    pub api_sender: mpsc::UnboundedSender<ApiRequest>,
    pub api_shooters: Arc<DashMap<String, oneshot::Sender<Result<ApiResponse, ApiError>>>>,
    pub pcw: DefaultProcedureWright,
    pub self_id: u64,
}
impl ProcedureCallWright for ClientState {
    fn next_echo(&self) -> impl Future<Output = u64> + Send {
        self.pcw.next_echo()
    }
}

macro_rules! ignore_none {
    ($expr:expr) => {
        match $expr {
            Some(value) => value,
            None => {
                error!("异常空值：{}", stringify!($expr));
                return;
            }
        }
    };
}
macro_rules! ignore_err {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(e) => {
                error!("{}", e);
                return;
            }
        }
    };
}
macro_rules! ignore_err_out {
    ($expr:expr) => {
        let _ = match $expr {
            Ok(_) => (),
            Err(e) => {
                error!("{}", e);
            }
        };
    };
}

pub const ECHO_GENERATOR: LazyLock<Mutex<SmallRng>> =
    LazyLock::new(|| Mutex::new(SmallRng::from_os_rng()));

mod msg_receiver {
    use log::debug;

    use super::*;
    pub struct MsgReceiver {
        pub stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        pub state: State<ClientState>,
    }

    pub async fn tick_msg_receiver(msg_receiver: &mut MsgReceiver) {
        let message = msg_receiver.stream.next().await;
        let message = if let Some(message) = message {
            message
        } else {
            return;
        };
        let message = message;
        let message = ignore_err!(message);
        if message.is_ping() {
            msg_receiver.stream.send(Message::Pong(message.into_data()));
            return;
        }
        if message.is_pong() {
            msg_receiver.stream.send(Message::Ping(message.into_data()));
            return;
        }
        let message = ignore_err!(message.to_text());
        let message = ignore_err!(serde_json::from_str::<InternalOnebotEvent>(message));
        let (event, errors) = message.into();
        if let Some(errors) = errors {
            for error in errors {
                error!("消息解析错误: {:?}", error);
            }
        }
        ignore_err_out!(msg_receiver.state.wright.emit(&event));
        match event.kind {
            event::EventKind::Message(message_detail) => {
                debug!("消息事件: {:?}", message_detail);
                ignore_err_out!(msg_receiver.state.wright.emit(&message_detail));
                ignore_err_out!(msg_receiver.state.wright.emit(&message_detail.flatten()));
            }
            event::EventKind::Notice(notice_detail) => {
                debug!("通知事件: {:?}", notice_detail);
                ignore_err_out!(msg_receiver.state.wright.emit(&notice_detail));
                match notice_detail {
                    event::NoticeEvent::Notify(notify) => {
                        ignore_err_out!(msg_receiver.state.wright.emit(&notify));
                    }
                    _ => {}
                }
            }
            event::EventKind::Request(request_detail) => {
                debug!("请求事件: {:?}", request_detail);
                ignore_err_out!(msg_receiver.state.wright.emit(&request_detail));
            }
            event::EventKind::Meta(meta_detail) => {
                debug!("元事件: {:?}", meta_detail);
                ignore_err_out!(msg_receiver.state.wright.emit(&meta_detail));
            }
            event::EventKind::Unknown(value) => {
                error!("未知事件: {:?}", value);
            }
        }
    }
}

mod api_sender {
    use log::debug;

    use super::*;
    pub struct ApiSenderWsState {
        pub ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        pub api_receiver: mpsc::UnboundedReceiver<ApiRequest>,
    }
    pub async fn tick_api_sender(ws_state: &mut ApiSenderWsState) {
        let api_request = ignore_none!(ws_state.api_receiver.recv().await);
        let api_request = ignore_err!(serde_json::to_string(&api_request));
        log::debug!("发送 onebot 请求: {:?}", api_request);
        ignore_err_out!(
            ws_state
                .ws_sender
                .send(Message::Text(api_request.into()))
                .await
        );
    }
}

mod api_receiver {
    use futures_util::future::err;
    use log::{debug, info};
    /* use sithra_common::api::api_internal::OnlyEcho; */

    use super::*;
    pub struct ApiReceiverWsState {
        pub ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    }
    pub async fn tick_api_receiver(state: &State<ClientState>, ws_state: &mut ApiReceiverWsState) {
        let message = ignore_none!(ws_state.ws_receiver.next().await);
        let message = ignore_err!(message);
        if message.is_ping() || message.is_pong() {
            return;
        }
        let message = ignore_err!(message.to_text());
        let echo = ignore_err!(serde_json::from_str::<OnlyEcho>(message));
        let message = serde_json::from_str::<ApiResponse>(message);
        let (_, sender) = ignore_none!(state.api_shooters.remove(&echo.echo));
        match message {
            Ok(message) => {
                if let Err(e) = sender.send(Ok(message)) {
                    error!("发送响应失败: {:?}", e);
                }
            }
            Err(e) => {
                if let Err(e) = sender.send(Err(ApiError::ResponseParseFailed(e.to_string()))) {
                    error!("发送响应失败: {:?}", e);
                }
            }
        }
    }
}

pub async fn build_ws_split(
    url: &str,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
)> {
    let ws_stream = build_ws_stream(url).await?;
    Ok(ws_stream.split())
}

pub async fn build_ws_stream(
    url: &str,
) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let result = tokio_tungstenite::connect_async(url).await?;
    Ok(result.0)
}
pub struct App {
    pub state: State<ClientState>,
    pub msg_receiver: MsgReceiver,
    pub api_sender: ApiSenderWsState,
    pub api_receiver: ApiReceiverWsState,
}

impl App {
    pub async fn new(
        url: &str,
        api_url: &str,
        bus: EffectWright,
        self_id: u64,
    ) -> anyhow::Result<Self> {
        let stream = build_ws_stream(url).await?;
        let (ws_sender, ws_receiver) = build_ws_split(api_url).await?;
        let (api_sender, api_receiver) = mpsc::unbounded_channel::<ApiRequest>();
        let state = ClientState {
            api_sender,
            api_shooters: Arc::new(DashMap::new()),
            pcw: DefaultProcedureWright::default(),
            self_id,
        };
        let state = State::new(state, bus);
        let msg_receiver = MsgReceiver {
            stream,
            state: state.clone(),
        };
        let api_sender = ApiSenderWsState {
            ws_sender,
            api_receiver,
        };
        let api_receiver = ApiReceiverWsState { ws_receiver };
        Ok(Self {
            state,
            msg_receiver,
            api_sender,
            api_receiver,
        })
    }
}
 */
