use crate::tunnel::TunnelConfig;
use anyhow::{anyhow, Result};
use shellexpand;
use std::fs::{create_dir_all, read_dir};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    tunnels_path: PathBuf,
    tunnels: Vec<TunnelConfig>,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid directory name"))?;
        let base_path: PathBuf = shellexpand::full(path)?.into_owned().into();

        let mut tunnels_path = base_path.clone();
        tunnels_path.push("tunnels");
        create_dir_all(&tunnels_path)?;

        let tunnels = Self::load_tunnels(&tunnels_path)?;
        Ok(Self {
            tunnels_path,
            tunnels,
        })
    }

    fn load_tunnels(path: &Path) -> Result<Vec<TunnelConfig>> {
        let mut tunnels: Vec<_> = Default::default();
        for entry in read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if !entry
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow!("Can't parse filename in tunnels config directory"))?
                .ends_with(".yaml")
            {
                continue;
            }
            tunnels.push(TunnelConfig::from_file(&entry.path())?);
        }
        Ok(tunnels)
    }
}
