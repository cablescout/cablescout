mod config;
mod tunnel;

use config::Config;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cablescout")]
struct Options {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Directory where cablescout saves its configuration
    #[structopt(name = "PATH", default_value = "~/.cablescout")]
    config_dir: PathBuf,

    #[structopt(subcommand)]
    cmd: Command,
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

        /// Remote Cablescout address
        address: String,
    },

    /// Remove a tunnel
    Remove {
        /// Tunnel name to remove
        name: String,
    },
}

fn main() {
    let options = Options::from_args();
    let config = Config::new(options.config_dir);
}
