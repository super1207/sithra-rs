mod client;
mod config;
mod subscribers;
mod util;

use std::env;

use client::*;
use config::Config;
use ioevent::prelude::*;
use log::*;
use subscribers::SUBSCRIBERS;
use tokio::{fs, process::Command, select};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::prelude::*;
use util::join_url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::init().await?;

    log::set_max_level(config.base.log_level.into());

    let subscribers = Subscribers::init(SUBSCRIBERS);
    let mut builder = BusBuilder::new(subscribers);
    let current_dir = env::current_dir()?;
    let plugin_path = current_dir.join("plugins");
    fs::create_dir_all(&plugin_path).await?;

    let mut plugin_dir = fs::read_dir(plugin_path).await?;
    while let Some(entry) = plugin_dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let child = Command::new(&path);
            let io: IoPair<_, _> = child.try_into()?;
            builder.add_pair(io);
            info!(target: "plugin_loader", "成功加载插件: {:?}", path.file_name().unwrap().to_str());
        }
    }

    let (bus, wright) = builder.build();
    let App {
        state,
        mut msg_receiver,
        mut api_sender,
        mut api_receiver,
    } = App::new(
        &join_url(&config.base.ws_url, "event/"),
        &join_url(&config.base.ws_url, "api/"),
        wright,
    )
    .await?;

    log::info!("成功连接到 WebSocket 服务器");
    
    let cancel_token = CancellationToken::new();

    let cancel_token_clone = cancel_token.clone();
    let msg_receiver_handle = tokio::spawn(async move {
        loop {
            if cancel_token_clone.is_cancelled() {
                break;
            }
            tick_msg_receiver(&mut msg_receiver).await;
        }
    });
    let cancel_token_clone = cancel_token.clone();
    let api_sender_handle = tokio::spawn(async move {
        loop {
            if cancel_token_clone.is_cancelled() {
                break;
            }
            tick_api_sender(&mut api_sender).await;
        }
    });
    let state_clone = state.clone();
    let bus_handle = bus
        .run(state, &|e| {
            error!("总线错误: {:?}", e);
        })
        .await;
    let (join_handle, close_handle) = bus_handle.spawn();

    loop {
        select! {
            _ = tick_api_receiver(&state_clone, &mut api_receiver) => {}
            _ = tokio::signal::ctrl_c() => {
                log::info!("正在关闭...");
                cancel_token.cancel();
                msg_receiver_handle.abort();
                api_sender_handle.abort();
                close_handle.close();
                break;
            }
        }
    }
    join_handle.await.unwrap();
    Ok(())
}
