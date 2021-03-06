use anyhow::{anyhow, Result};
use chrono::prelude::*;
use ipnetwork::{IpNetwork, IpNetworkError};
use log::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{Notify, RwLock};
use tokio::time;
use uuid::Uuid;
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
pub(crate) struct Session<U>
where
    U: Send,
{
    pub(crate) ends_at: DateTime<Utc>,
    pub(crate) user_data: U,
    pub(crate) device_id: Uuid,
    pub(crate) client_public_key: String,
    pub(crate) client_address: IpAddr,
}

impl<U> TryFrom<&Session<U>> for WireguardPeer
where
    U: Send,
{
    type Error = anyhow::Error;

    fn try_from(session: &Session<U>) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: session.client_public_key.clone(),
            allowed_ips: vec![ip_address_as_ip_network(session.client_address)?],
            endpoint: None,
            persistent_keepalive: None,
        })
    }
}

pub(crate) struct SessionManager<U>
where
    U: Send,
{
    client_network: IpNetwork,
    session_duration: chrono::Duration,
    sessions: RwLock<HashMap<Uuid, Session<U>>>,
    notify: Arc<Notify>,
}

impl<U> SessionManager<U>
where
    U: Send + Sync + Clone + 'static,
{
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

    pub async fn create(
        &self,
        device_id: Uuid,
        client_public_key: String,
        user_data: U,
    ) -> Result<Session<U>> {
        let mut sessions = self.sessions.write().await;

        let ends_at = Utc::now()
            .checked_add_signed(self.session_duration)
            .ok_or_else(|| anyhow!("Overflow while calculating session end time"))?;

        let session = if let Some(session) = sessions.get_mut(&device_id) {
            info!(
                "Updating existing session of device {} to end at {}",
                device_id, ends_at
            );
            session.ends_at = ends_at;
            session.client_public_key = client_public_key;
            session.clone()
        } else {
            info!(
                "Creating new session for device {}, ends at {}",
                device_id, ends_at
            );
            let server_addresses = [self.client_network.network(), self.server_address()];
            let mut addresses_in_use = itertools::sorted(itertools::chain(
                &server_addresses,
                sessions.values().map(|session| &session.client_address),
            ));
            let client_address = self
                .client_network
                .iter()
                .find(|ip| Some(ip) != addresses_in_use.next())
                .ok_or_else(|| anyhow!("Out of client addresses"))?;

            let session = Session {
                ends_at,
                user_data,
                device_id,
                client_public_key,
                client_address,
            };

            sessions.insert(device_id, session.clone());
            session
        };

        self.notify.notify_waiters();
        Ok(session)
    }

    pub async fn get_peers(&self) -> Result<Vec<WireguardPeer>> {
        Ok(self
            .sessions
            .read()
            .await
            .values()
            .map(WireguardPeer::try_from)
            .collect::<Result<_>>()?)
    }

    async fn next_expiring_session(self: Arc<Self>) -> Option<DateTime<Utc>> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|session| session.ends_at).max()
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
                    sessions.retain(|_, session| session.ends_at >= now);
                    info!("Removed {} sessions", sessions.len() - len_before);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_env_log::test;

    #[derive(Clone)]
    struct TestUserData {}

    type TestSessionManager = Arc<SessionManager<TestUserData>>;

    fn create_session_manager() -> Result<TestSessionManager> {
        let client_network: IpNetwork = "192.168.1.0/24".parse()?;
        let manager = SessionManager::new(client_network, chrono::Duration::minutes(10));
        manager.clone().run();
        Ok(manager)
    }

    #[test(tokio::test)]
    async fn test_create_session() -> Result<()> {
        let manager = create_session_manager()?;
        assert_eq!(manager.server_address(), "192.168.1.1".parse::<IpAddr>()?);

        let device_id1 = Uuid::new_v4();
        let session1 = manager
            .create(device_id1, "key1".to_owned(), TestUserData {})
            .await?;
        assert_eq!(session1.client_address, "192.168.1.2".parse::<IpAddr>()?);

        let device_id2 = Uuid::new_v4();
        let session2 = manager
            .create(device_id2, "key2".to_owned(), TestUserData {})
            .await?;
        assert_eq!(session2.client_address, "192.168.1.3".parse::<IpAddr>()?);
        Ok(())
    }

    #[test(tokio::test)]
    async fn test_session_reuse() -> Result<()> {
        let manager = create_session_manager()?;
        assert_eq!(manager.server_address(), "192.168.1.1".parse::<IpAddr>()?);

        let device_id = Uuid::new_v4();
        let session1 = manager
            .create(device_id, "key1".to_owned(), TestUserData {})
            .await?;
        assert_eq!(session1.client_address, "192.168.1.2".parse::<IpAddr>()?);

        let session2 = manager
            .create(device_id, "key2".to_owned(), TestUserData {})
            .await?;
        assert_eq!(session2.client_address, session1.client_address);
        Ok(())
    }
}
