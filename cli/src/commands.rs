use crate::config::Config;
use crate::tunnel::{Tunnel, TunnelConfig};
use anyhow::Result;

impl Config {
    pub async fn cmd_status(&self) -> Result<()> {
        let mut names = self.get_tunnel_names().peekable();
        if names.peek().is_none() {
            println!("No tunnels configured 😭");
        } else {
            println!("Cablescout tunnel status:");
            println!();
            for name in names {
                println!("   {}: 😴 Not connected", name);
            }
            println!();
        }
        Ok(())
    }

    pub async fn cmd_up(&self, name: Option<String>) -> Result<()> {
        let (name, config) = self.find(name)?;
        let tunnel = Tunnel::new(name, config);
        println!("🚀 Connecting to {}", name);
        tunnel.connect().await?;
        println!("🎉 Successfully connected to {}", name);
        Ok(())
    }

    pub async fn cmd_down(&self, name: Option<String>) -> Result<()> {
        let (name, config) = self.find(name)?;
        let tunnel = Tunnel::new(name, config);
        println!("✋ Disconnecting from {}", name);
        tunnel.disconnect().await?;
        println!("🛑 Disconnected from {}", name);
        Ok(())
    }

    pub async fn cmd_add(&mut self, name: String, tunnel_config: TunnelConfig) -> Result<()> {
        self.add_tunnel(&name, tunnel_config)?;
        self.save()?;
        println!("✨ Tunnel {} added", name);
        Ok(())
    }

    pub async fn cmd_remove(&mut self, name: String) -> Result<()> {
        self.remove_tunnel(&name)?;
        self.save()?;
        println!("🪓 Removed tunnel {}", name);
        Ok(())
    }
}
