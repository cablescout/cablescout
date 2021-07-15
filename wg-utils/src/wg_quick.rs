use crate::WireguardConfig;
use anyhow::{anyhow, Result};
use log::*;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[cfg(target_family = "windows")]
fn config_dir() -> PathBuf {
    PathBuf::from(r"C:\Program Files\WireGuard\Tunnels")
}

#[cfg(target_family = "unix")]
fn config_dir() -> PathBuf {
    PathBuf::from("/etc/wireguard")
}

async fn run_command(command: &mut Command) -> Result<()> {
    debug!("Running: {:?}", command);
    let output = command.output().await?;
    if !output.status.success() {
        let msg = format!(
            "Running command failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        error!("Command failed: {}", msg);
        return Err(anyhow!(msg));
    }
    debug!("Output: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

async fn write_config_file(name: &str, config: WireguardConfig) -> Result<String> {
    let config_dir = config_dir();
    create_dir_all(&config_dir).await?;

    let config_pathbuf = config_dir.join(format!("{}.conf", name));
    let config_path = config_pathbuf
        .to_str()
        .expect("Could not format tunnel config path")
        .to_string();

    debug!("Writing {}", config_path);
    let mut config_file = File::create(&config_pathbuf).await?;
    let config_data = format!("{}", config);
    config_file.write_all(config_data.as_bytes()).await?;

    Ok(config_path)
}

#[cfg(target_family = "unix")]
pub async fn wg_quick_up(name: &str, config: WireguardConfig) -> Result<()> {
    if let Err(err) = wg_quick_down(name).await {
        debug!(
            "Error while running 'wg-quick down' before 'wg-quick up': {}",
            err
        );
    }
    info!("Bringing {} up", name);
    let config_path = write_config_file(name, config).await?;
    run_command(Command::new("wg-quick").args(["up", &config_path])).await
}

#[cfg(target_family = "unix")]
pub async fn wg_quick_down(name: &str) -> Result<()> {
    info!("Taking {} down", name);
    run_command(Command::new("wg-quick").args(["down", name])).await
}

#[cfg(target_family = "windows")]
pub async fn wg_quick_up(name: &str, config: WireguardConfig) -> Result<()> {
    info!("Bringing {} up", name);
    let config_path = write_config_file(name, config).await?;
    run_command(
        Command::new(r"C:\Program Files\WireGuard\wireguard.exe")
            .args(["/installtunnelservice", &config_path]),
    )
    .await
}

#[cfg(target_family = "windows")]
pub async fn wg_quick_down(name: &str) -> Result<()> {
    info!("Taking {} down", name);
    run_command(
        Command::new(r"C:\Program Files\WireGuard\wireguard.exe")
            .args(["/uninstalltunnelservice", name]),
    )
    .await
}
