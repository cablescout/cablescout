mod config;
mod http;
mod server;
mod tunnel;

use anyhow::anyhow;
use async_std::fs::create_dir_all;
use config::Config;
use log::*;
use server::Server;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "51889")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::from_args();

    let default_level = match options.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter(Some(env!("CARGO_CRATE_NAME")), default_level)
        .filter(Some("wg_utils"), default_level)
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Daemon starting: {:?}", options);

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not find config directory"))?
        .join("cablescout");

    debug!("Creating {:?}", config_dir);
    create_dir_all(config_dir.clone()).await?;

    let config = Config::new(config_dir);
    config.watch();

    Server::new(options.port, config).run().await?;

    Ok(())
}
