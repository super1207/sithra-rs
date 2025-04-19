mod client;
mod config;
mod subscribers;
mod util;

use std::env;

use client::*;
use config::Config;
use ioevent::prelude::*;
use log::*;
use subscribers::API_PROCEDURES;
use tokio::{fs, process::Command, select, signal};
use tracing_subscriber::prelude::*;
use util::join_url;

const SUBSCRIBERS: &[Subscriber<ClientState>] = API_PROCEDURES;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    log::set_max_level(log::LevelFilter::Debug);

    let config = Config::init().await?;

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
            info!("成功加载插件: {:?}", path.file_name().unwrap().to_str());
        }
    }

    let Bus {
        mut subscribe_ticker,
        mut effect_ticker,
        effect_wright,
    } = builder.build();
    let App {
        state,
        mut msg_receiver,
        mut api_sender,
        mut api_receiver,
    } = App::new(
        &join_url(&config.base.ws_url, "event/"),
        &join_url(&config.base.ws_url, "api/"),
        effect_wright,
    )
    .await?;

    log::info!("成功连接到 WebSocket 服务器");

    let state_clone = state.clone();
    let handle = tokio::spawn(async move {
        loop {
            select! {
            _ = tick_msg_receiver(&mut msg_receiver) => {}
            _ = tick_api_sender(&mut api_sender) => {}
            _ = tick_api_receiver(&state_clone, &mut api_receiver) => {}
            }
        }
    });

    loop {
        select! {
            err = subscribe_ticker.tick(&state) => {
                match err {
                    Ok(e) => {
                        for err in e {
                            error!("订阅者错误: {:?}", err);
                        }
                    }
                    Err(e) => {
                        error!("总线错误: {:?}", e);
                    }
                }
            }
            errs = effect_ticker.tick() => {
                for err in errs {
                    error!("副作用总线错误: {:?}", err);
                }
            }
            _ = signal::ctrl_c() => {
                log::info!("正在关闭...");
                break;
            }
        }
    }

    handle.abort();

    Ok(())
}
