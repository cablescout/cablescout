mod commands;
mod config;
mod tunnel;

use anyhow::Result;
use config::Config;
use std::path::PathBuf;
use structopt::StructOpt;
use tunnel::TunnelConfig;

#[derive(Debug, StructOpt)]
#[structopt(name = "cablescout")]
struct Options {
    /// Activate debug mode
    #[structopt(global = true, short, long)]
    debug: bool,

    /// Directory where cablescout saves its configuration
    #[structopt(global = true, long, name = "PATH", default_value = "~/.cablescout")]
    config_dir: PathBuf,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Print tunnels and their connectivity status
    Status {},

    /// Connect a tunnel
    Up {
        /// Name of tunnel, can be skipped if only one tunnel is defined
        name: Option<String>,
    },

    /// Disconnect a tunnel
    Down {
        /// Name of tunnel, can be skipped if only one tunnel is defined
        name: Option<String>,
    },

    /// Add a new tunnel
    Add {
        /// New tunnel name
        name: String,

        #[structopt(flatten)]
        tunnel_config: TunnelConfig,
    },

    /// Remove a tunnel
    Remove {
        /// Tunnel name to remove
        name: String,
    },
}

impl Options {
    fn run(self) -> Result<()> {
        let mut config = Config::new(self.config_dir)?;

        match self.command {
            Command::Status {} => {
                config.cmd_status()?;
            }
            Command::Up { name } => {
                config.cmd_up(name)?;
            }
            Command::Down { name } => {
                config.cmd_down(name)?;
            }
            Command::Add {
                name,
                tunnel_config,
            } => {
                config.cmd_add(name, tunnel_config)?;
            }
            Command::Remove { name } => {
                config.cmd_remove(name)?;
            }
        }
        Ok(())
    }
}

fn main() {
    let options = Options::from_args();
    let is_debug = options.debug;

    env_logger::Builder::new()
        .filter(
            Some(env!("CARGO_CRATE_NAME")),
            match is_debug {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            },
        )
        .filter(None, log::LevelFilter::Info)
        .init();

    let result = options.run().await;
    match is_debug {
        true => result.unwrap(),
        false => {
            if let Err(err) = result {
                println!("error: {}", err.to_string());
                std::process::exit(1);
            }
        }
    }
}
