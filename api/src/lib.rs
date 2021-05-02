use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use wg_utils::{WireguardInterface, WireguardPeer};

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
pub struct FinishLoginResponse {
    pub session_ends_at: DateTime<Utc>,
    pub interface: WireguardInterface,
    pub peer: WireguardPeer,
}
