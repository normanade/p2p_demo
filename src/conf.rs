///

use config::{Config, File};
use serde::Deserialize;
use libp2p::Multiaddr;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Conf {
    pub role: String,
    pub use_ipv6: bool,
    hub: HubOpt,
    client: ClientOpt,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct HubOpt {
    listen_port: u16,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ClientOpt {
    username: String,
    hub_ip: Multiaddr,
    hub_port: u16,
    peer_users: String,
}

impl Conf {
    pub fn new(config_path: &str) -> Self {
        let setting = Config::builder().add_source(File::with_name(config_path)).build().unwrap();
        setting.try_deserialize().unwrap()
    }

    pub fn get_bind_port(&self) -> u16 {
        0
    }
}
