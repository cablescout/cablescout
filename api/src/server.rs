use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use wg_utils::{WireguardInterface, WireguardPeer};

#[derive(Debug, Serialize, Deserialize)]
pub struct StartLoginRequest {
    pub device_id: Uuid,
    pub client_public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartLoginResponse {
    pub auth_url: Url,
    pub login_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishLoginRequest {
    pub login_token: String,
    pub auth_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishLoginResponse {
    pub session_ends_at: DateTime<Utc>,
    pub interface: WireguardInterface,
    pub peer: WireguardPeer,
}
