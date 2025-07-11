use std::str::FromStr;
use std::{collections::BTreeMap, process};
use std::time::Duration;
use std::pin::Pin;

use futures_util::{SinkExt, StreamExt};
use hyper::header::HeaderValue;
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
use tokio::time::{interval, sleep, Sleep};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use ulid::Ulid; 

#[derive(Clone, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "ws-url")]
    ws_url: String,
}



pub fn get_params_from_uri(uri:&hyper::Uri) -> BTreeMap<String,String> {
    let mut ret_map = BTreeMap::new();
    if uri.query().is_none() {
        return ret_map;
    }
    let query_str = uri.query().unwrap();
    let query_vec = query_str.split("&");
    for it in query_vec {
        if it == "" {
            continue;
        }
        let index_opt = it.find("=");
        if index_opt.is_some() {
            let k_rst: String = url::form_urlencoded::parse(it.get(0..index_opt.unwrap()).unwrap().as_bytes())
                .map(|(key, val)| [key, val].concat())
                .collect::<String>();
            let v_rst: String = url::form_urlencoded::parse(it.get(index_opt.unwrap() + 1..).unwrap().as_bytes())
                .map(|(key, val)| [key, val].concat())
                .collect::<String>();
            ret_map.insert(k_rst, v_rst);
        }
        else {
            let k_rst: String = url::form_urlencoded::parse(it.as_bytes())
                .map(|(key, val)| [key, val].concat())
                .collect::<String>();
            ret_map.insert(k_rst,"".to_owned());
        }
    }
    ret_map
}

#[tokio::main]
async fn main() {
    let (plugin, config) = Plugin::<Config>::new().await.expect("Init plugin failed");

    let (ws_tx, ws_rx) = mpsc::unbounded_channel::<WsMessage>();
    let bot_id = format!("{}-{}", "onebot", process::id());
    let client = plugin.server.client();
    let sink = client.sink();
    let bot_id_ = bot_id.clone();
    let config_ = config.clone();

    let mp = get_params_from_uri(&hyper::Uri::from_str(&config_.ws_url).expect("Parse URI failed"));
    use tungstenite::client::IntoClientRequest;
    let url = url::Url::parse(&config_.ws_url).expect("Parse WebSocket URL failed");
    let mut request = url.as_str().into_client_request().expect("Create WebSocket request failed");
    if let Some(access_token) = mp.get("access_token") {
        request.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    }
    // log::info!("request: {:?}", request);
    let connection_manager = tokio::spawn(async move {
        let mut ws_rx = ws_rx; 

        'reconnect: loop {
            log::info!("Attempting to connect to WebSocket: {}", &config_.ws_url);
            let ws_stream = match connect_async(request.clone()).await {
                Ok((stream, _)) => {
                    log::info!("WebSocket connected successfully.");
                    stream
                }
                Err(e) => {
                    log::error!("Failed to connect to WebSocket: {}. Retrying in 5 seconds...", e);
                    sleep(Duration::from_secs(5)).await;
                    continue 'reconnect;
                }
            };

            let (mut ws_write, mut ws_read) = ws_stream.split();
            let bot_id_ = bot_id_.clone();
            
            let mut heartbeat_interval = interval(Duration::from_secs(30));
            let mut timeout_sleep: Option<Pin<Box<Sleep>>> = None;
            let mut last_heartbeat_echo: Option<Ulid> = None;

            loop {
                tokio::select! {
                    _ = async { timeout_sleep.as_mut().unwrap().await }, if timeout_sleep.is_some() => {
                        log::error!("Heartbeat timeout. No response received in 5 seconds. Reconnecting...");
                        timeout_sleep.take();
                        break;
                    }

                    _ = heartbeat_interval.tick() => {
                        let echo = Ulid::new();
                        let heartbeat_req = ApiCall::new(
                            "get_version_info",
                            json!({}),
                            echo,
                        );
                        let msg = WsMessage::Text(serde_json::to_string(&heartbeat_req).unwrap().into());
                        
                        log::debug!("Sending heartbeat with echo: {}", &echo);
                        if let Err(e) = ws_write.send(msg).await {
                            log::error!("Failed to send heartbeat: {}. Connection lost.", e);
                            break;
                        }
                        
                        last_heartbeat_echo = Some(echo);
                        timeout_sleep = Some(Box::pin(sleep(Duration::from_secs(5))));
                    }

                    Some(msg) = ws_rx.recv() => {
                        if let Err(e) = ws_write.send(msg).await {
                            log::error!("Send message to WebSocket failed: {}. Connection lost.", e);
                            break;
                        }
                    },
                    Some(message) = ws_read.next() => {
                        let message = match message {
                            Ok(message) => message,
                            Err(e) => {
                                log::error!("Recv message from WebSocket failed: {}. Connection lost.", e);
                                break;
                            }
                        };
                        
                        if let Some(echo_to_check) = &last_heartbeat_echo {
                             if let Ok(text) = message.to_text() {
                                 if text.contains(&echo_to_check.to_string()) {
                                     log::debug!("Heartbeat ACK received for echo: {}", echo_to_check);
                                     timeout_sleep = None;
                                     last_heartbeat_echo = None;
                                     continue;
                                 }
                             }
                        }

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
                        if let Some(message) = message {
                            if let Err(e) = sink.send(message) {
                                log::error!("Failed to send message to Sithra core: {}", e);
                            }
                        }
                    },
                    else => {
                        break;
                    }
                }
            }
            log::warn!("WebSocket connection lost. Reconnecting in 5 seconds...");
            sleep(Duration::from_secs(5)).await;
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
        _ = connection_manager => {
            log::error!("Connection manager task exited unexpectedly.");
        }
        _ = plugin.run().join_all() => {
             log::info!("Sithra plugin server exited.");
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Ctrl-C received, shutting down.");
        }
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