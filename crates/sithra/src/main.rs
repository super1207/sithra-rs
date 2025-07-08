use sithra::{conf, loader};
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = conf::load_config();
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Err(err.into());
        }
    };
    let mut loader = loader::Loader::new(config);
    loader.load();

    signal::ctrl_c().await?;

    loader.abort_all();
    Ok(())
}
