use chrono::prelude::*;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartLoginRequest {
    pub client_public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartLoginResponse {
    pub login_token: String,
    pub oidc_client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishLoginRequest {
    pub login_token: String,
    pub id_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_address: IpNetwork,
    pub dns_server: Option<IpAddr>,
    pub mtu: Option<u16>,
    pub server_public_key: String,
    pub server_port: u16,
    pub networks: Vec<IpNetwork>,
    pub persistent_keepalive: Option<Duration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishLoginResponse {
    pub session_ends_at: DateTime<Utc>,
    pub client_config: ClientConfig,
}
