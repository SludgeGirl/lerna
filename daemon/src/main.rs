mod daemon_socket;

use std::path::Path;

use crate::daemon_socket::DaemonSocket;

const DEFAULT_SOCK_PATH: &str = "/tmp/rst.sock";
const DEFAULT_DATABASE_PATH: &str = "/var/lib/lerna/database.sqlite";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let connection_string = format!("sqlite::file:{}", DEFAULT_DATABASE_PATH);
    let client = welds::connections::connect(connection_string)
        .await
        .unwrap();

    DaemonSocket::new(client, Path::new(DEFAULT_SOCK_PATH)).start()?;
    Ok(())
}
