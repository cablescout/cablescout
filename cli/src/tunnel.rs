use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// Remote Cablescout endpoint (<hostname>, <hostname:port>, <ip>, or <ip:port>)
    #[structopt(short, long)]
    endpoint: String,
}

pub struct Tunnel<'a> {
    name: &'a str,
    config: &'a TunnelConfig,
}

impl<'a> Tunnel<'a> {
    pub fn new(name: &'a str, config: &'a TunnelConfig) -> Self {
        Self { name, config }
    }

    pub fn connect(&self) -> Result<()> {
        Ok(())
    }
}
