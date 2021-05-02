use anyhow::{anyhow, Result};
use log::*;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub(crate) struct WgKeyPair {
    pub(crate) public_key: String,
    pub(crate) private_key: String,
}

impl WgKeyPair {
    async fn make_private_key() -> Result<String> {
        debug!("Generating private key");
        let child = Command::new("wg").arg("genkey").output().await?;
        if !child.status.success() {
            return Err(anyhow!("Running \"wg genkey\" failed"));
        }
        Ok(String::from_utf8(child.stdout)?)
    }

    async fn make_public_key(private_key: &[u8]) -> Result<String> {
        debug!("Getting public key");
        let mut child = Command::new("wg")
            .arg("pubkey")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child.stdin.take().unwrap().write_all(private_key).await?;
        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Err(anyhow!(
                "Running \"wg pubkey\" failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    pub async fn new() -> Result<Self> {
        let private_key = Self::make_private_key().await?;
        let public_key = Self::make_public_key(private_key.as_bytes()).await?;
        Ok(Self {
            private_key,
            public_key,
        })
    }
}
