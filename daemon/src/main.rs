mod daemon_socket;

use std::path::Path;

use crate::daemon_socket::DaemonSocket;

const DEFAULT_SOCK_PATH: &str = "/tmp/rst.sock";

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    DaemonSocket::new(Path::new(DEFAULT_SOCK_PATH)).start()?;
    Ok(())
}
