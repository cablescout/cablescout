use anyhow::{anyhow, Result};
use cablescout_api::{
    FinishLoginRequest, FinishLoginResponse, StartLoginRequest, StartLoginResponse,
};
use log::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use structopt::StructOpt;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use url::Url;
use wg_utils::{wg_quick_down, wg_quick_up, FullWireguardInterface, WgKeyPair, WireguardConfig};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// Remote Cablescout endpoint (<hostname>, <hostname:port>, <ip>, or <ip:port>)
    #[structopt(short, long)]
    endpoint: Url,
}

impl TunnelConfig {
    fn start_api_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/start")?)
    }

    fn finish_api_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/finish")?)
    }
}

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

async fn http_post<Req, Res>(url: Url, req: Req) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    Ok(http_client()?
        .post(url)
        .json(&req)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub struct Tunnel<'a> {
    name: &'a str,
    config: &'a TunnelConfig,
}

async fn read_line() -> Result<String> {
    BufReader::new(stdin())
        .lines()
        .next_line()
        .await?
        .ok_or_else(|| anyhow!("Error reading from stdin"))
}

impl<'a> Tunnel<'a> {
    pub fn new(name: &'a str, config: &'a TunnelConfig) -> Self {
        Self { name, config }
    }

    async fn login(&self) -> Result<WireguardConfig> {
        let key_pair = WgKeyPair::new().await?;

        debug!("Sending login start request");
        let start_res: StartLoginResponse = http_post(
            self.config.start_api_url()?,
            StartLoginRequest {
                client_public_key: key_pair.public_key.clone(),
            },
        )
        .await?;
        debug!("Got login start response: {:#?}", start_res);

        Command::new("open")
            .arg(start_res.auth_url.to_string())
            .spawn()?;

        println!("--------------------------------------------------");
        println!("               Enter code below                   ");
        println!("--------------------------------------------------");
        let auth_code = read_line().await?.trim().to_owned();

        debug!("Sending login finish request");
        let finish_res: FinishLoginResponse = http_post(
            self.config.finish_api_url()?,
            FinishLoginRequest {
                login_token: start_res.login_token,
                auth_code,
            },
        )
        .await?;
        debug!("Got login finish response: {:#?}", finish_res);

        Ok(WireguardConfig::new(
            FullWireguardInterface::new(&key_pair, finish_res.interface),
            vec![finish_res.peer],
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
