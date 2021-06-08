use crate::login::UserData;
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use ipnetwork::{IpNetwork, IpNetworkError};
use log::*;
use std::convert::TryFrom;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::time;
use wg_utils::WireguardPeer;

pub fn ip_address_as_ip_network(ip: IpAddr) -> Result<IpNetwork, IpNetworkError> {
    IpNetwork::new(
        ip,
        match ip {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        },
    )
}

#[derive(Debug, Clone)]
pub(crate) struct Session {
    pub(crate) ends_at: DateTime<Utc>,
    pub(crate) user_data: UserData,
    pub(crate) client_public_key: String,
    pub(crate) client_address: IpAddr,
}

impl TryFrom<&Session> for WireguardPeer {
    type Error = anyhow::Error;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: session.client_public_key.clone(),
            allowed_ips: vec![ip_address_as_ip_network(session.client_address)?],
            endpoint: None,
            persistent_keepalive: None,
        })
    }
}

pub(crate) struct SessionManager {
    client_network: IpNetwork,
    session_duration: chrono::Duration,
    sessions: RwLock<Vec<Session>>,
    notify: Arc<Notify>,
}

impl SessionManager {
    pub fn new(client_network: IpNetwork, session_duration: chrono::Duration) -> Arc<Self> {
        Arc::new(Self {
            client_network,
            session_duration,
            sessions: Default::default(),
            notify: Default::default(),
        })
    }

    pub fn run(self: Arc<Self>) {
        tokio::spawn(self.expire_old_sessions());
    }

    pub fn get_notify(self: Arc<Self>) -> Arc<Notify> {
        self.notify.clone()
    }

    /// The first address in the client network is reserved for the server
    pub fn server_address(&self) -> IpAddr {
        self.client_network
            .iter()
            .find(|ip| ip != &self.client_network.network())
            .expect("Client network is too small")
    }

    pub async fn create(&self, client_public_key: String, user_data: UserData) -> Result<Session> {
        let mut sessions = self.sessions.write().await;

        let ends_at = Utc::now()
            .checked_add_signed(self.session_duration)
            .ok_or_else(|| anyhow!("Overflow while calculating session end time"))?;

        let server_addresses = [self.client_network.network(), self.server_address()];
        let mut addresses_in_use = itertools::sorted(itertools::chain(
            &server_addresses,
            sessions.iter().map(|session| &session.client_address),
        ));
        let client_address = self
            .client_network
            .iter()
            .find(|ip| Some(ip) != addresses_in_use.next())
            .ok_or_else(|| anyhow!("Out of client addresses"))?;

        let session = Session {
            ends_at,
            user_data,
            client_public_key,
            client_address,
        };

        sessions.push(session.clone());
        self.notify.notify_waiters();
        Ok(session)
    }

    pub async fn get_peers(&self) -> Result<Vec<WireguardPeer>> {
        Ok(self
            .sessions
            .read()
            .await
            .iter()
            .map(WireguardPeer::try_from)
            .collect::<Result<_>>()?)
    }

    async fn next_expiring_session(self: Arc<Self>) -> Option<DateTime<Utc>> {
        let sessions = self.sessions.read().await;
        sessions.iter().map(|session| session.ends_at).max()
    }

    async fn expire_old_sessions(self: Arc<Self>) {
        let notify = self.clone().get_notify();

        loop {
            let until = match self.clone().next_expiring_session().await {
                None => Duration::from_secs(10000000000),
                Some(datetime) => {
                    let duration_until = datetime - Utc::now();
                    duration_until
                        .to_std()
                        .unwrap_or_else(|_| Duration::from_nanos(0))
                }
            };
            info!("Next session is set to expire in {:?}", until);

            let timeout = time::sleep(until);
            select! {
                _ = notify.notified() => {
                    // Restart loop to calculate the next session to expire
                    break;
                }

                _ = timeout => {
                    debug!("Removing old sessions");
                    let mut sessions = self.sessions.write().await;
                    let len_before = sessions.len();
                    let now = Utc::now();
                    sessions.retain(|session| session.ends_at >= now);
                    info!("Removed {} sessions", sessions.len() - len_before);
                }
            }
        }
    }
}
