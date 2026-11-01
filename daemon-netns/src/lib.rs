use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use cidr::Ipv4Inet;
use daemon_ipv4_issuer::Ipv4Issuer;
use futures::future;
use welds::{connections::any::AnyClient, prelude::*};

#[cfg(test)]
mod tests;

pub(crate) const SYSTEMD_PATH: &str = "/etc/systemd/system";
pub(crate) const NETNS_PATH: &str = "/etc/netns";

pub enum NamespaceError {
    IoError(std::io::Error),
    Ipv4RangeExhausted,
}

impl From<std::io::Error> for NamespaceError {
    fn from(value: std::io::Error) -> Self {
        NamespaceError::IoError(value)
    }
}

pub struct State {}

impl State {
    pub fn get_namespaces() -> Vec<Namespace> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum NamespaceState {
    Running,
    Stopped,
    Errored,
    #[default]
    Unknown,
}

#[derive(Debug, Default, WeldsModel)]
#[welds(table = "namespaces")]
pub struct Namespace {
    #[welds(primary_key)]
    id: u32,
    systemd_service: String,
    namespace: String,
    #[welds(ignore)]
    status: NamespaceState,
}

impl Namespace {
    pub async fn get_namespaces(client: &AnyClient) -> std::io::Result<Vec<Self>> {
        let entries = future::try_join_all(
            std::fs::read_dir(NETNS_PATH)?
                .filter(|m| {
                    m.as_ref()
                        .unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .contains("lerna-")
                })
                .map(async |m| -> std::io::Result<Namespace> {
                    let namespace_name = m.unwrap().file_name().into_string().unwrap();
                    let mut namespace = Namespace::all()
                        .where_col(|namespace| namespace.namespace.equal(&namespace_name))
                        .limit(1)
                        .run(client)
                        .await
                        .unwrap();

                    Ok(namespace.remove(0).into_inner())
                }),
        );
        Ok(entries.await.unwrap())
    }

    // To create a namespace we need to do the following:
    // - create namespace with: ip netns add lerna-NAME
    // - setup veth pair
    // - set routing rules both inside and outside the namespace
    // - allow ipv4 forwarding
    // - allow veth pair through the firewall
    pub async fn create(
        name: &str,
        ts_ip_issuer: Arc<Mutex<Ipv4Issuer>>,
        veth_ip_issuer: Arc<Mutex<Ipv4Issuer>>,
    ) -> Result<(), NamespaceError> {
        Command::new("ip")
            .args(["netns", "add", &format!("lerna-{name}")])
            .output()?;

        let mut veth_ip_index: Vec<(usize, Ipv4Inet)> = Vec::new();
        for _ in 0..2 {
            if let Some(pair) = veth_ip_issuer.lock().unwrap().next_with_index() {
                veth_ip_index.push(pair);
            } else {
                return Err(NamespaceError::Ipv4RangeExhausted);
            }
        }
        let veth0_index = veth_ip_index.get(0).unwrap().0;
        let veth1_index = veth_ip_index.get(1).unwrap().0;

        // sudo ip link add veth0 type veth peer name veth1
        // TODO: Find the existing veth's and iterate up
        Command::new("ip")
            .args([
                "link",
                "add",
                &format!("lerna-veth{}", veth0_index),
                "type",
                "veth",
                "peer",
                "name",
                &format!("lerna-veth{}", veth1_index),
            ])
            .output()?;

        // sudo ip addr add 10.0.1.1/24 dev veth1

        for index_pair in veth_ip_index.iter() {
            let (index, ip) = index_pair;
            Command::new("ip")
                .args([
                    "addr",
                    "add",
                    &format!("{ip}/32"),
                    "dev",
                    &format!("lerna-veth{index}"),
                ])
                .output()?;
        }

        // sudo ip link set veth0 netns tailns
        Command::new("ip")
            .args([
                "link",
                "set",
                &format!("lerna-veth{}", veth0_index),
                "netns",
                &format!("lerna-{name}"),
            ])
            .output()?;

        Ok(())
    }
}
