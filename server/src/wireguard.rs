use crate::login::UserData;
use anyhow::{anyhow, Result};
use cablescout_api::ClientConfig;
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::time;

#[derive(Debug, StructOpt)]
pub struct WireguardSettings {
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

pub struct Session {
    ends_at: DateTime<Utc>,
    user_data: UserData,
    client_public_key: String,
    client_address: IpAddr,
}

pub struct Wireguard {
    inner: RwLock<Inner>,
}

struct Inner {
    settings: WireguardSettings,
    sessions: Vec<Session>,
    sessions_notify: Arc<Notify>,
    server_public_key: String,
}

impl Inner {
    fn allocate_client_address(&mut self) -> Result<IpAddr> {
        let mut in_use =
            itertools::sorted(self.sessions.iter().map(|session| session.client_address));
        self.settings
            .wg_client_cidr
            .iter()
            .skip_while(|ip| Some(ip) == in_use.next().as_ref())
            .next()
            .ok_or_else(|| anyhow!("Out of client addresses"))
    }
}

impl Wireguard {
    pub fn new(settings: WireguardSettings) -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(Inner {
                settings,
                sessions: Default::default(),
                sessions_notify: Default::default(),
                server_public_key: "".to_owned(),
            }),
        })
    }

    pub fn run(self: Arc<Self>) {
        tokio::spawn(self.clone().run_server());
        tokio::spawn(self.expire_old_sessions());
    }

    pub async fn start_session(
        self: Arc<Self>,
        client_public_key: String,
        user_data: UserData,
    ) -> Result<(ClientConfig, DateTime<Utc>)> {
        let mut inner = self.inner.write().await;

        let ends_at = Utc::now()
            .checked_add_signed(chrono::Duration::from_std(
                inner.settings.session_duration.into(),
            )?)
            .ok_or_else(|| anyhow!("Overflow while calculating session end time"))?;
        let client_address = inner.allocate_client_address()?;

        let session = Session {
            ends_at,
            user_data,
            client_public_key,
            client_address,
        };

        let client_config = ClientConfig {
            client_address: IpNetwork::new(
                client_address,
                match client_address {
                    IpAddr::V4(_) => 32,
                    IpAddr::V6(_) => 128,
                },
            )?,
            dns_server: inner.settings.wg_dns_server,
            mtu: inner.settings.wg_mtu,
            server_public_key: inner.server_public_key.clone(),
            server_port: inner.settings.wg_port,
            networks: vec![inner.settings.wg_client_cidr]
                .into_iter()
                .chain(inner.settings.wg_additional_networks.iter().copied())
                .collect(),
            persistent_keepalive: inner.settings.wg_client_keepalive.map(|value| value.into()),
        };

        inner.sessions.push(session);
        inner.sessions_notify.notify_waiters();
        Ok((client_config, ends_at))
    }

    async fn run_server(self: Arc<Self>) {
        let sessions_notify = {
            let inner = self.inner.read().await;
            inner.sessions_notify.clone()
        };

        loop {
            sessions_notify.notified().await;
        }
    }

    async fn next_expiring_session(self: Arc<Self>) -> Option<DateTime<Utc>> {
        let inner = self.inner.read().await;
        inner.sessions.iter().map(|session| session.ends_at).max()
    }

    async fn expire_old_sessions(self: Arc<Self>) {
        let sessions_notify = {
            let inner = self.inner.read().await;
            inner.sessions_notify.clone()
        };

        loop {
            let timeout = time::sleep(match self.clone().next_expiring_session().await {
                None => Duration::from_secs(10000000000),
                Some(datetime) => {
                    let duration_until = datetime - Utc::now();
                    duration_until.to_std().unwrap_or_else(|_| Duration::from_nanos(0))
                }
            });

            select! {
                _ = sessions_notify.notified() => {
                    // Restart loop to calculate the next session to expire
                    break;
                }

                _ = timeout => {
                    let mut inner = self.inner.write().await;
                    let now = Utc::now();

                    // TODO: Switch to drain_filter
                    let mut i = 0;
                    while i != inner.sessions.len() {
                        if inner.sessions[i].ends_at < now {
                            inner.sessions.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
    }
}
