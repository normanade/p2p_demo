///

use config::{Config, File};
use serde::Deserialize;
use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::multiaddr::Protocol;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

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
    hub_ip: String,
    hub_port: u16,
}

impl Conf {
    pub fn new(config_path: &str) -> Self {
        let setting = Config::builder().add_source(File::with_name(config_path)).build().unwrap();
        setting.try_deserialize().unwrap()
    }

    pub fn get_bind_port(&self) -> u16 {
        if let "hub" = self.role.as_str() {
            self.hub.listen_port
        } else {
            0
        }
    }

    pub fn get_relay_address(&self) -> Option<Multiaddr> {
        if let "client" = self.role.as_str() {
            let relay_ip = match self.use_ipv6 {
                true => self.client.hub_ip.parse::<Ipv6Addr>().unwrap().into(),
                false => self.client.hub_ip.parse::<Ipv4Addr>().unwrap().into(),
            };
            println!(
                "Preparing to connect relay server at {}:{}",
                self.client.hub_ip,
                self.client.hub_port
            );
            
            println!("Please input relay server peerid:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let peer_id = PeerId::from_str(input.trim()).expect("Invalid PeerId");
            Some(
                Multiaddr::empty()
                .with(relay_ip)
                .with(Protocol::Tcp(self.client.hub_port))
                .with(Protocol::P2p(peer_id.into()))
            )
        } else {
            None
        }
    }
}
