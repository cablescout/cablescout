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
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    debug!("Running wg-quick: args={:?}", args);
    let output = Command::new("wg-quick").args(args).output().await?;
    if !output.status.success() {
        let msg = format!(
            "Running \"wg-quick\" failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        error!("Command failed: {}", msg);
        return Err(anyhow!(msg));
    }
    debug!("Output: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub async fn wg_quick_up(name: &str, config: WireguardConfig) -> Result<()> {
    info!("Bringing {} up", name);

    let config_dir = PathBuf::from("/etc/wireguard");
    create_dir_all(&config_dir).await?;

    let config_pathbuf = config_dir.join(format!("{}.conf", name));
    let config_path = config_pathbuf
        .to_str()
        .expect("Could not format tunnel config path");

    debug!("Writing {}", config_path);
    let mut config_file = File::create(&config_pathbuf).await?;
    let config_data = format!("{}", config);
    config_file.write_all(config_data.as_bytes()).await?;

    if let Err(err) = run_wg_quick(&["down", name]).await {
        debug!(
            "Error while running 'wg-quick down' before 'wg-quick up': {}",
            err
        );
    }

    run_wg_quick(&["up", &config_path]).await
}

pub async fn wg_quick_down(name: &str) -> Result<()> {
    info!("Taking {} down", name);
    run_wg_quick(&["down", name]).await
}
