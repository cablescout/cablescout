use anyhow::Result;
use cablescout_api::{
    FinishLoginRequest, FinishLoginResponse, StartLoginRequest, StartLoginResponse,
};
use log::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use url::Url;
use wg_utils::{wg_quick_down, wg_quick_up, FullWireguardInterface, WgKeyPair, WireguardConfig};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// Remote Cablescout endpoint (<hostname>, <hostname:port>, <ip>, or <ip:port>)
    #[structopt(short, long)]
    endpoint: Url,
}

impl TunnelConfig {
    fn start_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/start")?)
    }

    fn finish_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/finish")?)
    }
}

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

pub struct Tunnel<'a> {
    name: &'a str,
    config: &'a TunnelConfig,
}

impl<'a> Tunnel<'a> {
    pub fn new(name: &'a str, config: &'a TunnelConfig) -> Self {
        Self { name, config }
    }

    async fn login(&self) -> Result<WireguardConfig> {
        let key_pair = WgKeyPair::new().await?;

        debug!("Sending login start request");
        let start_res: StartLoginResponse = http_client()?
            .post(self.config.start_url()?)
            .json(&StartLoginRequest {
                client_public_key: key_pair.public_key.clone(),
            })
            .send()
            .await?
            .json()
            .await?;
        debug!("Got login start response: {:#?}", start_res);

        // TODO: Login

        debug!("Sending login finish request");
        let finish_res: FinishLoginResponse = http_client()?
            .post(self.config.finish_url()?)
            .json(&FinishLoginRequest {
                login_token: start_res.login_token,
                id_token: "".to_owned(),
            })
            .send()
            .await?
            .json()
            .await?;
        debug!("Got login finish response: {:#?}", finish_res);

        Ok(WireguardConfig::new(
            FullWireguardInterface::new(&key_pair, finish_res.interface),
            finish_res.peer,
        ))
    }

    pub async fn connect(&self) -> Result<()> {
        let config = self.login().await?;
        wg_quick_up(self.name, config).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        wg_quick_down(self.name).await?;
        Ok(())
    }
}
