#![feature(btree_drain_filter)]

mod api;
mod api_result;
mod login;
mod sessions;
mod tokens;
mod wireguard;

use crate::api::ApiServer;
use crate::wireguard::Wireguard;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    #[structopt(flatten)]
    api: api::ApiSettings,

    #[structopt(flatten)]
    login: login::LoginSettings,

    #[structopt(flatten)]
    wireguard: wireguard::WireguardSettings,
}

async fn _main(options: Options) -> Result<()> {
    let wireguard = Wireguard::new(options.wireguard).await?;
    wireguard.clone().run();
    let api = ApiServer::new(options.api, options.login, wireguard)?;
    api.run().await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    let options = Options::from_args();

    env_logger::Builder::new()
        .filter(
            Some(env!("CARGO_CRATE_NAME")),
            match options.debug {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            },
        )
        .filter(None, log::LevelFilter::Info)
        .init();

    if let Err(err) = _main(options).await {
        log::error!("{:?}", err);
        std::process::exit(1);
    }
}
