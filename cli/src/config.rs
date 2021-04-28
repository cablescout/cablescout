use crate::tunnel::TunnelConfig;
use anyhow::{anyhow, Result};
use log::*;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    tunnels: HashMap<String, TunnelConfig>,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        debug!(
            "Loading config from {}",
            path.to_str().unwrap_or("ERROR PARSING PATH")
        );

        let mut self_ = Self {
            path,
            tunnels: Default::default(),
        };

        self_.load()?;
        Ok(self_)
    }

    pub fn save(&self) -> Result<()> {
        let path = self.tunnels_file_path()?;
        let mut file = File::create(path)?;
        let tunnels = serde_yaml::to_string(&self.tunnels)?;
        file.write_all(tunnels.as_bytes())?;
        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        let tunnels_file = match File::open(self.tunnels_file_path()?) {
            Ok(file) => file,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    return Ok(());
                } else {
                    return Err(err.into());
                }
            }
        };
        let tunnels_reader = BufReader::new(tunnels_file);
        self.tunnels = serde_yaml::from_reader(tunnels_reader)?;
        Ok(())
    }

    fn tunnels_file_path(&self) -> Result<PathBuf> {
        let mut path = self.ensure_path()?;
        path.push("tunnels.yaml");
        Ok(path)
    }

    fn ensure_path(&self) -> Result<PathBuf> {
        let path = self
            .path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid directory name"))?;
        debug!("Ensuring path: {}", path);
        let path: PathBuf = shellexpand::full(path)?.into_owned().into();
        create_dir_all(&path)?;
        Ok(path)
    }

    pub fn num_tunnels(&self) -> usize {
        self.tunnels.len()
    }

    pub fn find(&self, name: Option<String>) -> Result<(&String, &TunnelConfig)> {
        match name {
            Some(name) => self
                .tunnels
                .get_key_value(&name)
                .ok_or_else(|| anyhow!("🤷‍♂️ No such tunnel: {}", name)),
            None => match self.num_tunnels() {
                0 => Err(anyhow!("No tunnels configured 😭")),
                1 => Ok(self.tunnels.iter().next().unwrap()),
                _ => Err(anyhow!("Please specify tunnel name 🙄")),
            },
        }
    }

    pub fn get_tunnel_names<'a>(&'a self) -> Box<dyn Iterator<Item = &String> + 'a> {
        Box::new(self.tunnels.keys())
    }

    pub fn add_tunnel(&mut self, name: &str, tunnel_config: TunnelConfig) -> Result<()> {
        match self.tunnels.insert(name.to_owned(), tunnel_config) {
            None => Ok(()),
            Some(_) => Err(anyhow!("🚧 Tunnel {} already exists", name)),
        }
    }

    pub fn remove_tunnel(&mut self, name: &str) -> Result<()> {
        match self.tunnels.remove(name) {
            None => Err(anyhow!("🤷‍♂️ No such tunnel: {}", name)),
            Some(_) => Ok(()),
        }
    }
}
