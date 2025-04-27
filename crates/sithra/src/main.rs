mod client;
mod config;
mod subscribers;
mod util;

use std::{env, process::Stdio};

use client::*;
use config::Config;
use ioevent::{
    error::{BusError, BusRecvError},
    prelude::*,
};
use log::*;
use subscribers::SUBSCRIBERS;
use tokio::{
    fs::{self, create_dir_all},
    process::Command,
};
use tracing_subscriber::prelude::*;

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

    let mut childs = Vec::new();

    let mut plugin_dir = fs::read_dir(&plugin_path).await?;
    while let Some(entry) = plugin_dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let mut child = Command::new(&path);
            let plugin_name = path.file_stem();
            if let Some(Some(name)) = plugin_name.map(|s| s.to_str()) {
                let data_path = plugin_path.join(format!("{}.data", name));
                create_dir_all(&data_path).await?;
                let data_path = fs::canonicalize(&data_path).await?;
                child.arg(data_path);
            } else {
                error!(target: "plugin_loader", "插件名称获取失败: {:?}", path);
                continue;
            }
            child.stdout(Stdio::piped());
            child.stdin(Stdio::piped());
            let mut child = child.spawn()?;
            let io: IoPair<_, _> = IoPair {
                reader: child.stdout.take().unwrap(),
                writer: child.stdin.take().unwrap(),
            };
            builder.add_pair(io);
            childs.push(child);
            if let Some(Some(name)) = path.file_name().map(|s| s.to_str()) {
                info!(target: "plugin_loader", "成功加载插件: {}", name);
            }
        }
    }

    let (bus, wright) = builder.build();
    let state = State::new(ClientState::new(), wright);
    let bus_handle = bus
        .run(state, &|e| {
            error!(target: "main", "总线错误: {:?}", e);
            match e {
                BusError::BusRecv(BusRecvError::Recv(ioevent::error::RecvError::Io(_))) => {
                    std::process::exit(1);
                }
                _ => {}
            }
        })
        .await;
    let (_, close_headle) = bus_handle.spawn();
    let _ = tokio::signal::ctrl_c().await;
    close_headle.close();
    /* old version
    let App {
        state,
        mut msg_receiver,
        mut api_sender,
        mut api_receiver,
    } = App::new(
        &join_url(&config.base.ws_url, "event/"),
        &join_url(&config.base.ws_url, "api/"),
        wright,
        config.base.self_id,
    )
    .await?; */

    /* old version
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
            match e {
                BusError::BusRecv(BusRecvError::Recv(ioevent::error::RecvError::Io(_))) => {
                    std::process::exit(1);
                }
                _ => {}
            }
        })
        .await;
    let (_, close_handle) = bus_handle.spawn();

    loop {
        select! {
            _ = tick_api_receiver(&state_clone, &mut api_receiver) => {}
            _ = tokio::signal::ctrl_c() => {
                log::info!("正在关闭...");
                for mut child in childs {
                    while let None = child.try_wait()? {
                        let _ = child.start_kill();
                    }
                }
                cancel_token.cancel();
                msg_receiver_handle.abort();
                api_sender_handle.abort();
                close_handle.close();
                break;
            }
        }
    }
    */
    Ok(())
}
