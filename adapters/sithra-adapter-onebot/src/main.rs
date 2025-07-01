use std::process;

use futures_util::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use sithra_kit::{
    init,
    layers::BotId,
    logger::init_log,
    server::{extract::context::Context as RawContext, routing::router::Router, server::Server},
    transport::peer::Peer,
    types::{initialize::Initialize, message::SendMessage},
};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use triomphe::Arc;

type SharedConfig = Arc<Config>;

type Context<T> = RawContext<T, AdapterState>;

#[derive(Clone)]
struct AdapterState {
    config: OnceCell<SharedConfig>,
    ws_tx:  mpsc::UnboundedSender<WsMessage>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Config {}

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("").await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
    let send_loop = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            write.send(msg).await.expect("Send message to channel Error");
        }
    });

    let peer = Peer::new();

    let (peer, config) = init!(peer, Config);

    let server = Server::new();
    let client = server.client();
    init_log(client.sink());
    let sink = client.sink();
    let recv_loop = tokio::spawn(async move {
        while let Some(event) = read.next().await {
            let event = event.expect("Recv message from ws Error");
        }
    });

    let bot_id = format!("{}-{}", "onebot", process::id());
    let _router = Router::new()
        .route_typed(SendMessage::on(send_message))
        .route_typed(Initialize::<Config>::on(init))
        .layer(BotId::new(&bot_id));

    tokio::select! {
        _ = send_loop => {}
        _ = recv_loop => {}
    }
}

async fn send_message(ctx: Context<SendMessage>) {
    let payload = ctx.payload();
}

async fn init(ctx: Context<Initialize<Config>>) {
    let config = ctx.payload().config.clone();
    ctx.config.set(Arc::new(config)).ok();
}
