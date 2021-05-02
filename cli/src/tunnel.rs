use crate::key_pair::WgKeyPair;
use crate::wg_quick::WgQuickConfig;
use anyhow::{anyhow, Result};
use cablescout_api::{
    FinishLoginRequest, FinishLoginResponse, StartLoginRequest, StartLoginResponse,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use url::Url;

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

    fn wg_address(&self, port: u16) -> Result<String> {
        let host = self
            .endpoint
            .host()
            .ok_or_else(|| anyhow!("Endpoint has no host part"))?;
        Ok(format!("{}:{}", host, port))
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

    async fn login(&self) -> Result<WgQuickConfig> {
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

        Ok(WgQuickConfig::new(
            self.config
                .wg_address(finish_res.client_config.server_port)?,
            key_pair,
            finish_res.client_config,
        ))
    }

    async fn run_wg_quick<I, S>(args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        debug!("Running wg-quick");
        let output = Command::new("wg-quick").args(args).output().await?;
        if !output.status.success() {
            return Err(anyhow!(
                "Running \"wg-quick\" failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        debug!("Output: {}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    pub async fn connect(&self, wg_config_path: PathBuf) -> Result<()> {
        let config = self.login().await?;

        let config_pathbuf = wg_config_path.join(format!("{}.conf", self.name));
        let config_path = config_pathbuf
            .to_str()
            .expect("Could not format tunnel config path")
            .to_owned();
        debug!("Writing {}", config_path);
        let mut config_file = File::create(config_pathbuf).await?;
        let config_data = serde_ini::to_string(&config)?;
        config_file.write_all(config_data.as_bytes()).await?;

        Self::run_wg_quick(&["up", &config_path]).await?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        Self::run_wg_quick(&["down", self.name]).await?;
        Ok(())
    }
}
