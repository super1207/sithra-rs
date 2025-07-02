use std::process;

use futures_util::{SinkExt, StreamExt};
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
    config: SharedConfig,
    ws_tx:  mpsc::UnboundedSender<WsMessage>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Config {}

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("").await.unwrap();
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<WsMessage>();
    let send_loop = tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            ws_write.send(msg).await.expect("Send message to channel Error");
        }
    });

    let peer = Peer::new();

    let (peer, config) = init!(peer, Config);

    let config = config.unwrap();

    let (peer_write, peer_read) = peer.split();

    let server = Server::new();
    let client = server.client();
    init_log(client.sink());
    let sink = client.sink();
    let recv_loop = tokio::spawn(async move {
        while let Some(event) = ws_read.next().await {
            let event = event.expect("Recv message from ws Error");
            // TODO: Handle event & Send to sink
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

async fn send_message(ctx: Context<SendMessage>) {
    let payload = ctx.payload();
}
