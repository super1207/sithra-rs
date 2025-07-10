use std::process;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sithra_adapter_onebot::{
    AdapterState, OneBotMessage, api::request::ApiCall, message::OneBotSegment, util::send_req,
};
use sithra_kit::{
    layers::BotId,
    plugin::Plugin,
    server::{
        extract::{correlation::Correlation, payload::Payload, state::State},
        response::Response,
    },
    transport::channel::Channel,
    types::{
        channel::SetMute,
        message::{Segments, SendMessage},
    },
};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

// type Context<T> = RawContext<T, AdapterState>;

#[derive(Clone, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "ws-url")]
    ws_url: String,
}

#[tokio::main]
async fn main() {
    let (plugin, config) = Plugin::<Config>::new().await.expect("Init plugin failed");

    let (ws_stream, _) = connect_async(&config.ws_url).await.unwrap();
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<WsMessage>();
    let send_loop = tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            ws_write.send(msg).await.expect("Send message to channel Error");
        }
    });

    let bot_id = format!("{}-{}", "onebot", process::id());

    let client = plugin.server.client();
    let sink = client.sink();
    let bot_id_ = bot_id.clone();
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
            if message.is_empty() {
                continue;
            }
            let message = match serde_json::from_str::<OneBotMessage>(&message) {
                Ok(message) => message,
                Err(err) => {
                    log::error!("Parse message from ws Error: {err}\traw: {message:?}");
                    continue;
                }
            };
            let message = match message {
                OneBotMessage::Api(api) => Some(api.into_rep(&bot_id_)),
                OneBotMessage::Event(event) => event.into_req(&bot_id_),
            };
            let Some(message) = message else {
                continue;
            };
            sink.send(message).unwrap();
        }
    });

    let state = AdapterState { ws_tx };

    let plugin = plugin.map(|r| {
        r.route_typed(SendMessage::on(send_message))
            .route_typed(SetMute::on(set_mute))
            .layer(BotId::new(bot_id))
            .with_state(state)
    });

    tokio::select! {
        _ = send_loop => {}
        _ = recv_loop => {}
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn send_message(
    Payload(payload): Payload<SendMessage>,
    State(state): State<AdapterState>,
    Correlation(id): Correlation,
    channel: Channel,
) -> Option<Response> {
    let segments = payload.content.into_iter().filter_map(|s| match OneBotSegment::try_from(s) {
        Ok(segment) => match segment {
            OneBotSegment(segment) => Some(segment),
        },
        Err(_err) => None,
    });
    let req = if let Some(group_id) = channel.parent_id {
        ApiCall::new(
            "send_msg",
            json!({
                "message_type": "group",
                "group_id": group_id,
                "message": segments.collect::<Segments<_>>()
            }),
            id,
        )
    } else {
        ApiCall::new(
            "send_msg",
            json!({
                "message_type": "private",
                "user_id": channel.id,
                "message": segments.collect::<Segments<_>>()
            }),
            id,
        )
    };
    send_req(&state, id, &req, "send_msg")
}

async fn set_mute(
    Payload(payload): Payload<SetMute>,
    State(state): State<AdapterState>,
    Correlation(id): Correlation,
) -> Option<Response> {
    let SetMute { channel, duration } = payload;
    let Channel {
        id: user_id,
        ty: _,
        name: _,
        parent_id,
        self_id: _,
    } = channel;
    let Some(parent_id) = parent_id else {
        log::error!("Set Mute Failed to get parent_id");
        let mut response = Response::error("Failed to get parent_id");
        response.correlate(id);
        return Some(response);
    };
    let duration = duration.as_secs();
    let req = ApiCall::new(
        "set_group_ban",
        json!({
            "group_id": parent_id,
            "user_id": user_id,
            "duration": duration
        }),
        id,
    );
    send_req(&state, id, &req, "set_mute")
}
