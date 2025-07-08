use std::{process::Stdio, time::Duration};

use sithra_server::{extract::payload::Payload, on, routing::router::Router, server::Server};
use sithra_transport::{datapack::RequestDataPack, peer::Peer};
use tokio::process::Command;

async fn print(Payload(content): Payload<String>) -> Payload<String> {
    println!("{content}");
    Payload(content)
}

#[tokio::main]
async fn main() {
    let child = Command::new("./server_b")
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let peer = Peer::from_child(child).unwrap();
    let (writer, reader) = peer.split();
    let router = Router::new().route("/print", on(print));
    let server = Server::new();
    let client = server.client().sink();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            client.send(RequestDataPack::default().path("/hello_world")).unwrap();
            // let Ok(response) =
            // client.post(RequestDataPack::default().path("/hello_world")) else
            // {     continue;
            // };
            // drop(response);
            // response.await.ok();
        }
    });
    server.service(router).serve(writer, reader).join_all().await;
}
