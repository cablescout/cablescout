use crate::login::UserData;
use crate::sessions::{ip_address_as_ip_network, SessionManager};
use anyhow::Result;
use ipnetwork::IpNetwork;
use log::*;
use std::net::IpAddr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::time::Instant;
use uuid::Uuid;
use wg_utils::{
    wg_quick_up, FullWireguardInterface, WgKeyPair, WireguardConfig, WireguardInterface,
    WireguardInterfaceScripts, WireguardPeer,
};

#[derive(Debug, StructOpt)]
pub(crate) struct WireguardSettings {
    /// Session duration, after which a client that was successfully
    /// connected is not authorized anymore and have to login again
    #[structopt(long, env = "SESSION_DURATION", default_value = "1d")]
    session_duration: humantime::Duration,

    /// WireGuard server bind address, use default value to listen on all interfaces
    #[structopt(long, env = "WG_BIND_IP", default_value = "0.0.0.0")]
    wg_bind_ip: IpAddr,

    /// WireGuard server port
    #[structopt(long, env = "WG_PORT", default_value = "51820")]
    wg_port: u16,

    /// Client address CIDR. The server allocates one address for itself,
    /// then clients get addresses following this first address.
    #[structopt(long, env = "WG_CLIENT_CIDR", default_value = "172.25.0.0/24")]
    wg_client_cidr: IpNetwork,

    /// Additional networks to route traffic to
    #[structopt(long, env = "WG_ADDITIONAL_NETWORKS")]
    wg_additional_networks: Vec<IpNetwork>,

    /// Optional DNS server for clients
    #[structopt(long, env = "WG_DNS_SERVER")]
    wg_dns_server: Option<IpAddr>,

    /// MTU to set for configuration returned to clients
    #[structopt(long, env = "WG_MTU")]
    wg_mtu: Option<u16>,

    /// Persistent keepalive for the clients
    #[structopt(long, env = "WG_CLIENT_KEEPALIVE")]
    wg_client_keepalive: Option<humantime::Duration>,

    /// Persistent keepalive for the server
    #[structopt(long, env = "WG_SERVER_KEEPALIVE")]
    wg_server_keepalive: Option<humantime::Duration>,

    /// Post up script
    #[structopt(long, env = "WG_POST_UP_SCRIPT")]
    wg_post_up_script: Option<String>,

    /// Post down script
    #[structopt(long, env = "WG_POST_DOWN_SCRIPT")]
    wg_post_down_script: Option<String>,
}

pub(crate) struct Wireguard {
    settings: WireguardSettings,
    session_manager: Arc<SessionManager<UserData>>,
    key_pair: WgKeyPair,
}

impl Wireguard {
    pub(crate) async fn new(settings: WireguardSettings) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            session_manager: SessionManager::new(
                settings.wg_client_cidr,
                settings.session_duration.into(),
            ),
            settings,
            key_pair: WgKeyPair::new().await?,
        }))
    }

    pub(crate) fn run(self: Arc<Self>) {
        self.session_manager.clone().run();
        tokio::spawn(self.run_server());
    }

    pub(crate) async fn start_session(
        self: Arc<Self>,
        hostname: &str,
        device_id: Uuid,
        client_public_key: String,
        user_data: UserData,
    ) -> Result<(WireguardInterface, WireguardPeer, Instant)> {
        let session = self
            .session_manager
            .create(device_id, client_public_key, user_data)
            .await?;

        let interface = WireguardInterface {
            address: ip_address_as_ip_network(session.client_address)?,
            dns: self.settings.wg_dns_server,
            mtu: self.settings.wg_mtu,
            listen_port: None,
        };

        let peer = WireguardPeer {
            public_key: self.key_pair.public_key.clone(),
            endpoint: Some(format!("{}:{}", hostname, self.settings.wg_port)),
            allowed_ips: vec![self.settings.wg_client_cidr]
                .into_iter()
                .chain(self.settings.wg_additional_networks.iter().copied())
                .collect(),
            persistent_keepalive: self.settings.wg_client_keepalive.map(|value| value.into()),
        };

        Ok((interface, peer, session.ends_at))
    }

    async fn run_server(self: Arc<Self>) {
        let sessions_notify = self.session_manager.clone().get_notify();

        loop {
            sessions_notify.notified().await;
            debug!("Client sessions have changed, updating server");

            if let Err(err) = self.clone().update_server().await {
                error!("Error updating server configuration: {}", err);
            }
        }
    }

    async fn update_server(self: Arc<Self>) -> Result<()> {
        let interface = FullWireguardInterface::new_with_scripts(
            &self.key_pair,
            WireguardInterface {
                address: ip_address_as_ip_network(self.session_manager.server_address())?,
                listen_port: Some(self.settings.wg_port),
                mtu: self.settings.wg_mtu,
                dns: None,
            },
            WireguardInterfaceScripts {
                post_up: self.settings.wg_post_up_script.clone(),
                post_down: self.settings.wg_post_down_script.clone(),
            },
        );

        let peers = self.session_manager.get_peers().await?;

        let config = WireguardConfig::new(interface, peers);
        wg_quick_up("server", config).await?;

        Ok(())
    }
}
