use crate::WireguardConfig;
use anyhow::{anyhow, Result};
use log::*;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

async fn run_wg_quick<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    debug!("Running wg-quick");
    let output = Command::new("wg-quick").args(args).output().await?;
    if !output.status.success() {
        return Err(anyhow!(
            "Running \"wg-quick\" failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    debug!("Output: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub async fn wg_quick_up(name: &str, config: WireguardConfig) -> Result<()> {
    let config_dir = PathBuf::from("/etc/wireguard");
    create_dir_all(&config_dir).await?;

    let config_pathbuf = config_dir.join(format!("{}.conf", name));
    let config_path = config_pathbuf
        .to_str()
        .expect("Could not format tunnel config path");

    debug!("Writing {}", config_path);
    let mut config_file = File::create(&config_pathbuf).await?;
    let config_data = serde_ini::to_string(&config)?;
    config_file.write_all(config_data.as_bytes()).await?;

    run_wg_quick(&["up", &config_path]).await
}

pub async fn wg_quick_down(name: &str) -> Result<()> {
    run_wg_quick(&["down", name]).await
}
