use crate::login::UserData;
use crate::sessions::SessionManager;
use anyhow::Result;
use cablescout_api::ClientConfig;
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::sync::Arc;
use structopt::StructOpt;

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
}

pub(crate) struct Wireguard {
    settings: WireguardSettings,
    session_manager: Arc<SessionManager>,
    server_public_key: String,
}

impl Wireguard {
    pub(crate) fn new(settings: WireguardSettings) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            session_manager: SessionManager::new(
                settings.wg_client_cidr,
                chrono::Duration::from_std(settings.session_duration.into())?,
            ),
            settings,
            server_public_key: "".to_owned(),
        }))
    }

    pub(crate) fn run(self: Arc<Self>) {
        self.session_manager.clone().run();
        tokio::spawn(self.run_server());
    }

    pub(crate) async fn start_session(
        self: Arc<Self>,
        client_public_key: String,
        user_data: UserData,
    ) -> Result<(ClientConfig, DateTime<Utc>)> {
        let session = self
            .session_manager
            .create(client_public_key, user_data)
            .await?;

        let client_config = ClientConfig {
            client_address: session.client_address_as_ip_network()?,
            dns_server: self.settings.wg_dns_server,
            mtu: self.settings.wg_mtu,
            server_public_key: self.server_public_key.clone(),
            server_port: self.settings.wg_port,
            networks: vec![self.settings.wg_client_cidr]
                .into_iter()
                .chain(self.settings.wg_additional_networks.iter().copied())
                .collect(),
            persistent_keepalive: self.settings.wg_client_keepalive.map(|value| value.into()),
        };

        Ok((client_config, session.ends_at))
    }

    async fn run_server(self: Arc<Self>) {
        let sessions_notify = self.session_manager.clone().get_notify();

        loop {
            sessions_notify.notified().await;
        }
    }
}
