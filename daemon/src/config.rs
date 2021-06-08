use anyhow::Result;
use async_std::fs;
use async_std::prelude::*;
use cablescout_api::daemon::TunnelInfo;
use log::*;
use notify::{watcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task;
use url::Url;

const CONFIG_SUFFIX: &str = ".tunnel.json";

pub type ConfigTunnels = HashMap<String, TunnelConfig>;

pub struct Config {
    path: PathBuf,
    inner: RwLock<Inner>,
}

struct Inner {
    tunnels: ConfigTunnels,
}

impl Inner {
    async fn new(path: &Path) -> Result<Self> {
        Ok(Self {
            tunnels: Self::read_tunnels(path).await?,
        })
    }

    async fn read_tunnels(path: &Path) -> Result<ConfigTunnels> {
        info!(
            "Reading tunnels from {}",
            path.to_str().unwrap_or("ERROR PARSING PATH")
        );

        let mut entries = fs::read_dir(path).await?;
        let mut tunnels: ConfigTunnels = Default::default();

        while let Some(res) = entries.next().await {
            let entry = res?;
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();
            if !entry.file_type().await?.is_file() {
                debug!("Skipping {} (not a file)", filename);
                continue;
            }
            if let Some(name) = filename.strip_suffix(CONFIG_SUFFIX) {
                let raw = fs::read(entry.path()).await?;
                let tunnel_config: TunnelConfig = match serde_json::from_slice(&raw) {
                    Ok(tunnel_config) => tunnel_config,
                    Err(err) => {
                        error!("Could not parse {:?}: {}", filename, err);
                        continue;
                    }
                };
                info!("Found tunnel {}: {:#?}", name, tunnel_config);
                tunnels.insert(name.to_owned(), tunnel_config);
            } else {
                debug!("Skipping {} (suffix doesn't match)", filename);
            }
        }

        info!("Found {} configured tunnels", tunnels.len());
        Ok(tunnels)
    }
}

impl Config {
    pub async fn new(path: PathBuf) -> Result<Arc<Self>> {
        let inner = RwLock::new(Inner::new(&path).await?);
        let self_ = Arc::new(Self { path, inner });
        self_.watch();
        Ok(self_)
    }

    pub async fn get_tunnels_info(self: &Arc<Self>) -> HashMap<String, TunnelInfo> {
        self.inner
            .read()
            .await
            .tunnels
            .iter()
            .map(|(key, value)| (key.to_owned(), value.into()))
            .collect()
    }

    pub async fn find(self: &Arc<Self>, name: &str) -> Option<TunnelConfig> {
        self.inner.read().await.tunnels.get(name).cloned()
    }

    fn watch(self: &Arc<Self>) {
        debug!("Watching for changes in {:?}", self.path);

        let (refresh_tx, mut refresh_rx) = tokio::sync::mpsc::unbounded_channel();
        let path = self.path.clone();

        task::spawn_blocking(move || {
            let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
            let mut watcher = watcher(watcher_tx, Duration::from_millis(200)).unwrap();
            watcher.watch(path, RecursiveMode::Recursive).unwrap();

            loop {
                watcher_rx.recv().unwrap();
                refresh_tx.send(()).unwrap();
            }
        });

        let self_ = self.clone();
        task::spawn(async move {
            loop {
                match self_.clone().refresh().await {
                    Ok(()) => (),
                    Err(err) => error!("Error refreshing config: {:?}", err),
                }
                if refresh_rx.recv().await.is_none() {
                    break;
                }
            }
        });
    }

    async fn refresh(self: Arc<Self>) -> Result<()> {
        let mut writer = self.inner.write().await;
        *writer = Inner::new(&self.path).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub endpoint: Url,
}

impl From<&TunnelConfig> for TunnelInfo {
    fn from(config: &TunnelConfig) -> Self {
        Self {
            endpoint: config.endpoint.to_string(),
        }
    }
}

impl TunnelConfig {
    pub fn start_api_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/start")?)
    }

    pub fn finish_api_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/api/v1/login/finish")?)
    }

    pub fn finish_url(&self) -> Result<Url> {
        Ok(self.endpoint.join("/finish")?)
    }
}
