use std::{
    fs::File,
    io::Read,
    net::Ipv4Addr,
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use daemon_ipv4_issuer::Ipv4Issuer;
use daemon_messages::Messages;
use daemon_netns::Namespace;
use log::info;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
use welds::connections::any::AnyClient;

const DEFAULT_LOCK_PATH: &str = "/tmp/rst.lock";

pub struct DaemonSocket {
    pub path: PathBuf,
    pub listener: Option<UnixListener>,
    pub lock_created: bool,
    pub database_client: AnyClient,
    pub tailscale_device_ip_range: Arc<Mutex<Ipv4Issuer>>,
    pub veth_ip_range: Arc<Mutex<Ipv4Issuer>>,
}

impl DaemonSocket {
    pub fn new(database_client: AnyClient, path: &Path) -> Self {
        DaemonSocket {
            path: path.to_path_buf(),
            listener: None,
            lock_created: false,
            database_client,
            tailscale_device_ip_range: Arc::new(Mutex::new(Ipv4Issuer::default())),
            veth_ip_range: Arc::new(Mutex::new(
                Ipv4Issuer::new(Ipv4Addr::new(10, 0, 1, 0), 24)
                    .expect("Failed to create default veth ip range issuer"),
            )),
        }
    }

    pub fn start(&mut self) -> std::io::Result<()> {
        self.check_for_lock()?;
        if std::fs::exists(&self.path)? {
            std::fs::remove_file(&self.path)?;
        }
        if let Ok(_) = std::fs::write(DEFAULT_LOCK_PATH, std::process::id().to_ne_bytes()) {
            info!("Running as process: {}", std::process::id());
            self.lock_created = true;
        }
        let listener = UnixListener::bind(&self.path)?;
        self.listener = Some(listener);
        if let Some(listener) = &self.listener {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        /* connection succeeded */
                        let ts_ip_issuer = self.tailscale_device_ip_range.clone();
                        let veth_ip_issuer = self.tailscale_device_ip_range.clone();
                        thread::spawn(move || {
                            let stream = &mut stream;
                            Self::handle_client(stream, ts_ip_issuer, veth_ip_issuer)
                        });
                    }
                    Err(_) => {
                        /* connection failed */
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_client(
        stream: &mut UnixStream,
        ts_ip_issuer: Arc<Mutex<Ipv4Issuer>>,
        veth_ip_issuer: Arc<Mutex<Ipv4Issuer>>,
    ) {
        let mut message = String::new();
        stream.read_to_string(&mut message).unwrap();
        info!("Recieved message: {}", message);
        let decoded_message: Messages = serde_json::from_str(&message).unwrap();
        info!("Recieved message: {:?}", decoded_message);

        match decoded_message {
            Messages::AddInstance { name } => {
                Namespace::create(&name, ts_ip_issuer, veth_ip_issuer);
            }
            Messages::DeleteInstance { name } => todo!(),
            Messages::Status { namespace } => todo!(),
            Messages::Up { name } => todo!(),
            Messages::Down { name } => todo!(),
        }
    }

    fn check_for_lock(&self) -> Result<(), std::io::Error> {
        if std::fs::exists(DEFAULT_LOCK_PATH)? {
            let mut file = File::open(DEFAULT_LOCK_PATH)?;
            let mut buffer = [0; 4];
            file.read_exact(&mut buffer[..])?;
            let process_id = u32::from_ne_bytes(buffer);
            let refresh = RefreshKind::nothing();
            let system =
                System::new_with_specifics(refresh.with_processes(ProcessRefreshKind::nothing()))
                    .expect("Failed to gather processes, cannot safely run");
            let process = system.process(Pid::from_u32(process_id));

            if process.is_some() {
                panic!("Service already running");
            } else {
                // No matching process lets clean up the lock file
                std::fs::remove_file(DEFAULT_LOCK_PATH)?
            }
        }
        Ok(())
    }
}

impl Drop for DaemonSocket {
    fn drop(&mut self) {
        if self.lock_created {
            let _ = std::fs::remove_file(DEFAULT_LOCK_PATH);
        }
        if self.listener.is_some() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
