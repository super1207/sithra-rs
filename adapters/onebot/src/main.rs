use std::process;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sithra_adapter_onebot::{
    OneBotMessage,
    api::request::{ApiCall, SendMessage as OneBotSendMessage, SendMessageKind},
    message::OneBotSegment,
};
use sithra_kit::{
    init,
    layers::BotId,
    logger::init_log,
    server::{
        extract::{
            context::Context as RawContext, correlation::Correlation, payload::Payload,
            state::State,
        },
        response::Response,
        routing::router::Router,
        server::Server,
    },
    transport::{channel::Channel, peer::Peer},
    types::{initialize::Initialize, message::SendMessage},
};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use triomphe::Arc;

type SharedConfig = Arc<Config>;

// type Context<T> = RawContext<T, AdapterState>;

#[derive(Clone)]
struct AdapterState {
    config: SharedConfig,
    ws_tx:  mpsc::UnboundedSender<WsMessage>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "ws-url")]
    ws_url: String,
}

#[tokio::main]
async fn main() {
    let peer = Peer::new();

    let (peer, config) = init!(peer, Config);

    let config = config.unwrap();

    let (peer_write, peer_read) = peer.split();

    let (ws_stream, _) = connect_async(&config.ws_url).await.unwrap();
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<WsMessage>();
    let send_loop = tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            ws_write.send(msg).await.expect("Send message to channel Error");
        }
    });

    let server = Server::new();
    let client = server.client();
    init_log(client.sink());
    let sink = client.sink();
    let recv_loop = tokio::spawn(async move {
        while let Some(message) = ws_read.next().await {
            let message = message.expect("Recv message from ws Error");
            let message = match message.into_text() {
                Ok(message) => message,
                Err(err) => {
                    log::error!("Recv message from ws Error: {err}");
                    continue;
                }
            };
            let message = match serde_json::from_str::<OneBotMessage>(&message) {
                Ok(message) => message,
                Err(err) => {
                    log::error!("Parse message from ws Error: {err}");
                    continue;
                }
            };
            let message = match message {
                OneBotMessage::Api(api) => Some(api.into_rep()),
                OneBotMessage::Event(event) => event.into_req(),
            };
            let Some(message) = message else {
                continue;
            };
            sink.send(message).unwrap();
        }
    });

    let state = AdapterState {
        config: Arc::new(config),
        ws_tx,
    };

    let bot_id = format!("{}-{}", "onebot", process::id());
    let router = Router::new()
        .route_typed(SendMessage::on(send_message))
        .layer(BotId::new(&bot_id))
        .with_state(state);

    let mut serve = server.service(router).serve(peer_write, peer_read);

    tokio::select! {
        _ = send_loop => {}
        _ = recv_loop => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    serve.abort_all();
}

async fn send_message(
    Payload(payload): Payload<SendMessage>,
    State(state): State<AdapterState>,
    Correlation(id): Correlation,
    channel: Channel,
) -> Option<Response> {
    let segments = payload.content.into_iter().filter_map(|s| match OneBotSegment::try_from(s) {
        Ok(segment) => match segment {
            OneBotSegment::Typed(segment) => Some(segment),
            OneBotSegment::Unknown(_) => None,
        },
        Err(_err) => None,
    });
    let params = if let Some(group_id) = channel.parent_id {
        OneBotSendMessage {
            message_type: SendMessageKind::Group { group_id },
            message:      segments.collect(),
        }
    } else {
        OneBotSendMessage {
            message_type: SendMessageKind::Private {
                user_id: channel.id,
            },
            message:      segments.collect(),
        }
    };
    let req = ApiCall::new(&"send_msg", params, id);
    let req = serde_json::to_string(&req);
    let Ok(req) = req else {
        log::error!("Failed to serialize send_msg request");
        let mut response = Response::error(&"Failed to serialize send_msg request");
        response.correlate(id);
        return Some(response);
    };
    let result = state.ws_tx.send(WsMessage::Text(req.into()));
    if let Err(err) = result {
        log::error!("Failed to send send_msg request: {err}");
        let mut response = Response::error(&"Failed to send send_msg request");
        response.correlate(id);
        return Some(response);
    }
    None
}
