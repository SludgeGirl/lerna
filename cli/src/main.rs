use std::{io::Write, os::unix::net::UnixStream};

use clap::{Parser, Subcommand};
use daemon_messages::Messages;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// See the status of the tailscale instances. Similar to tailscale status
    Status {
        #[arg(short, long)]
        tailscale_instance: Option<String>,
    },
    /// Adds a tailscale instance isolated in it's own namespace
    Add { name: String },
    /// Setup the routing and tailscale instances in namespaces
    Up {
        #[arg(short, long)]
        tailscale_instance: Option<String>,
    },
    /// Teardown the routing and stop the tailscale instances
    Down {
        #[arg(short, long)]
        tailscale_instance: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut stream = UnixStream::connect("/tmp/rst.sock").unwrap();
    match cli.command {
        Commands::Add { name } => {
            stream
                .write_all(
                    serde_json::to_string(&Messages::AddInstance { name: name })
                        .unwrap()
                        .as_bytes(),
                )
                .unwrap();
        }
        Commands::Status { tailscale_instance } => {
            stream
                .write_all(
                    serde_json::to_string(&Messages::Status {
                        namespace: tailscale_instance,
                    })
                    .unwrap()
                    .as_bytes(),
                )
                .unwrap();
        }
        Commands::Up { tailscale_instance } => todo!("woa: {}", tailscale_instance.unwrap()),
        Commands::Down { tailscale_instance } => {}
    }
}
