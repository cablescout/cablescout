use crate::WgKeyPair;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use serde_with::rust::StringWithSeparator;
use serde_with::skip_serializing_none;
use serde_with::SpaceSeparator;
use std::net::IpAddr;
use std::time::Duration;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireguardInterface {
    pub address: IpNetwork,
    pub dns: Option<IpAddr>,
    pub mtu: Option<u16>,
    pub listen_port: Option<u16>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireguardInterfaceScripts {
    post_up: Option<String>,
    post_down: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FullWireguardInterface {
    private_key: String,
    #[serde(flatten)]
    interface: WireguardInterface,
    #[serde(flatten)]
    scripts: WireguardInterfaceScripts,
}

impl FullWireguardInterface {
    pub fn new(key_pair: &WgKeyPair, interface: WireguardInterface) -> Self {
        Self {
            private_key: key_pair.private_key.clone(),
            interface,
            scripts: Default::default(),
        }
    }

    pub fn new_with_scripts(
        key_pair: WgKeyPair,
        interface: WireguardInterface,
        scripts: WireguardInterfaceScripts,
    ) -> Self {
        Self {
            private_key: key_pair.private_key,
            interface,
            scripts,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireguardPeer {
    pub public_key: String,
    #[serde(with = "StringWithSeparator::<SpaceSeparator>")]
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<Duration>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireguardConfig {
    interface: FullWireguardInterface,
    peer: WireguardPeer,
}

impl WireguardConfig {
    pub fn new(interface: FullWireguardInterface, peer: WireguardPeer) -> Self {
        Self { interface, peer }
    }
}
