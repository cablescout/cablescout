use crate::login::UserData;
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use log::*;
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::time;

#[derive(Debug, Clone)]
pub(crate) struct Session {
    pub(crate) ends_at: DateTime<Utc>,
    pub(crate) user_data: UserData,
    pub(crate) client_public_key: String,
    pub(crate) client_address: IpAddr,
}

impl Session {
    pub fn client_address_as_ip_network(&self) -> Result<IpNetwork> {
        Ok(IpNetwork::new(
            self.client_address,
            match self.client_address {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            },
        )?)
    }
}

pub(crate) struct SessionManager {
    client_network: IpNetwork,
    session_duration: chrono::Duration,
    sessions: RwLock<BTreeMap<IpAddr, Session>>,
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

    pub async fn create(&self, client_public_key: String, user_data: UserData) -> Result<Session> {
        let mut sessions = self.sessions.write().await;

        let ends_at = Utc::now()
            .checked_add_signed(self.session_duration)
            .ok_or_else(|| anyhow!("Overflow while calculating session end time"))?;

        let mut addresses_in_use = sessions.keys();
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

        // TODO: Use expect_none()
        if sessions.insert(client_address, session.clone()).is_some() {
            panic!("Inserted session overrides existing IpAddr, this shouldn't happen");
        }

        self.notify.notify_waiters();
        Ok(session)
    }

    async fn next_expiring_session(self: Arc<Self>) -> Option<DateTime<Utc>> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|session| session.ends_at).max()
    }

    async fn expire_old_sessions(self: Arc<Self>) {
        let notify = self.clone().get_notify();

        loop {
            let timeout = time::sleep(match self.clone().next_expiring_session().await {
                None => Duration::from_secs(10000000000),
                Some(datetime) => {
                    let duration_until = datetime - Utc::now();
                    duration_until
                        .to_std()
                        .unwrap_or_else(|_| Duration::from_nanos(0))
                }
            });

            select! {
                _ = notify.notified() => {
                    // Restart loop to calculate the next session to expire
                    break;
                }

                _ = timeout => {
                    let mut sessions = self.sessions.write().await;
                    let now = Utc::now();
                    for (_ip, expired) in sessions.drain_filter(|_ip, session| session.ends_at < now) {
                        info!("Session expired: {:?}", expired);
                    }
                }
            }
        }
    }
}
