use crate::key_pair::WgKeyPair;
use cablescout_api::ClientConfig;
use ipnetwork::IpNetwork;
use serde::Serialize;
use std::net::IpAddr;
use std::time::Duration;

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct WgQuickInterface {
    private_key: String,
    address: IpNetwork,
    dns: Option<IpAddr>,
    mtu: Option<u16>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct WgQuickPeer {
    public_key: String,
    #[serde(with = "serde_with::rust::StringWithSeparator::<serde_with::SpaceSeparator>")]
    allowed_ips: Vec<IpNetwork>,
    endpoint: String,
    persistent_keepalive: Option<Duration>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct WgQuickConfig {
    interface: WgQuickInterface,
    peer: WgQuickPeer,
}

impl WgQuickConfig {
    pub fn new(endpoint: String, key_pair: WgKeyPair, client_config: ClientConfig) -> Self {
        Self {
            interface: WgQuickInterface {
                private_key: key_pair.private_key,
                address: client_config.client_address,
                dns: client_config.dns_server,
                mtu: client_config.mtu,
            },
            peer: WgQuickPeer {
                public_key: client_config.server_public_key,
                allowed_ips: client_config.networks,
                endpoint,
                persistent_keepalive: client_config.persistent_keepalive,
            },
        }
    }
}
