use futures::future;
use welds::{connections::any::AnyClient, prelude::*};

#[cfg(test)]
mod tests;

pub(crate) const SYSTEMD_PATH: &str = "/etc/systemd/system";
pub(crate) const NETNS_PATH: &str = "/etc/netns";

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

    /// To create a namespace we need to do the following:
    /// - create namespace with: ip netns add lerna-NAME
    /// - setup veth pair
    /// - set routing rules both inside and outside the namespace
    /// - allow ipv4 forwarding
    /// - allow veth pair through the firewall
    pub fn create() {}
}
