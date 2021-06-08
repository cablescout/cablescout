use crate::config::{DaemonConfig, TunnelConfig};
use crate::http::http_post;
use anyhow::Result;
use cablescout_api::daemon::TunnelStatus;
use cablescout_api::server::{
    FinishLoginRequest, FinishLoginResponse, StartLoginRequest, StartLoginResponse,
};
use log::*;
use std::sync::Arc;
use url::Url;
use wg_utils::{wg_quick_down, wg_quick_up, FullWireguardInterface, WgKeyPair, WireguardConfig};

pub struct Tunnel {
    name: String,
    daemon_config: Arc<DaemonConfig>,
    tunnel_config: TunnelConfig,
    status: TunnelStatus,
    key_pair: Option<WgKeyPair>,
    login_token: Option<String>,
    error: Option<String>,
}

impl Tunnel {
    pub fn new(
        name: String,
        daemon_config: Arc<DaemonConfig>,
        tunnel_config: TunnelConfig,
    ) -> Self {
        Self {
            name,
            daemon_config,
            tunnel_config,
            status: TunnelStatus::Disconnected,
            key_pair: None,
            login_token: None,
            error: None,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn status(&self) -> TunnelStatus {
        self.status
    }

    async fn start_login(&self) -> Result<(WgKeyPair, StartLoginResponse)> {
        let key_pair = WgKeyPair::new().await?;

        let req = StartLoginRequest {
            device_id: self.daemon_config.get_device_id().await,
            client_public_key: key_pair.public_key.clone(),
        };
        debug!("Sending login start request: {:#?}", req);
        let start_res: StartLoginResponse =
            http_post(self.tunnel_config.start_api_url()?, req).await?;
        debug!("Got login start response: {:#?}", start_res);

        Ok((key_pair, start_res))
    }

    async fn finish_login(
        &self,
        login_token: String,
        key_pair: WgKeyPair,
        auth_code: String,
    ) -> Result<()> {
        let req = FinishLoginRequest {
            login_token,
            auth_code,
        };
        debug!("Sending login finish request: {:#?}", req);
        let finish_res: FinishLoginResponse =
            http_post(self.tunnel_config.finish_api_url()?, req).await?;
        debug!("Got login finish response: {:#?}", finish_res);

        let wg_config = WireguardConfig::new(
            FullWireguardInterface::new(&key_pair, finish_res.interface),
            vec![finish_res.peer],
        );

        wg_quick_up(&self.name, wg_config).await?;

        Ok(())
    }

    pub async fn start_connect(&mut self) -> Result<Url> {
        self.status = TunnelStatus::Connecting;
        self.error = None;

        match self.start_login().await {
            Ok((key_pair, start_res)) => {
                self.key_pair = Some(key_pair);
                self.login_token = Some(start_res.login_token);
                Ok(start_res.auth_url)
            }
            Err(err) => {
                self.status = TunnelStatus::Error;
                self.error = Some(err.to_string());
                Err(err)
            }
        }
    }

    pub async fn finish_connect(&mut self, auth_code: String) -> Result<()> {
        let login_token = self
            .login_token
            .take()
            .expect("No login_token while calling finish_connect");
        let key_pair = self
            .key_pair
            .take()
            .expect("No key_pair while calling finish_connect");
        match self.finish_login(login_token, key_pair, auth_code).await {
            Ok(()) => {
                self.status = TunnelStatus::Connected;
                Ok(())
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.status = TunnelStatus::Error;
                Err(err)
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.status = TunnelStatus::Disconnecting;

        match wg_quick_down(&self.name).await {
            Ok(_) => {
                self.status = TunnelStatus::Disconnected;
                Ok(())
            }
            Err(err) => {
                self.status = TunnelStatus::Error;
                self.error = Some(err.to_string());
                Err(err)
            }
        }
    }
}
