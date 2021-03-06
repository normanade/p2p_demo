///

use config::{Config, File};
use log::info;
use serde::Deserialize;
use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::multiaddr::Protocol;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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
    hub_ip: IpAddr,
    hub_port: u16,
}

impl Conf {
    pub fn new(config_path: &str) -> Self {
        let setting = Config::builder().add_source(File::with_name(config_path)).build().unwrap();
        setting.try_deserialize().unwrap()
    }
}

impl Conf {
    pub fn print_detail(&self) {
        if let "client" = self.role.as_str() {
            info!("Start as {}, relay server at {:?}", self.role, self.client);
        } else {
            info!("Start as {}, opening port [{}]", self.role, self.hub.listen_port);
        }
    }

    pub fn get_bind_port(&self) -> u16 {
        if let "hub" = self.role.as_str() {
            self.hub.listen_port
        } else {
            0
        }
    }

    pub fn get_bind_address(&self) -> Multiaddr {
        // Listen on all interfaces
        let port = self.get_bind_port();
        match self.use_ipv6 {
            true => Multiaddr::empty()
                .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(port))
                .with(Protocol::from(Ipv6Addr::UNSPECIFIED)),
            false => Multiaddr::empty()
                .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(port)),
        }
    }

    pub fn get_relay_address(&self, relay_id: PeerId) -> Option<Multiaddr> {
        if let "client" = self.role.as_str() {
            // let relay_ip = match self.use_ipv6 {
            //     true => self.client.hub_ip.parse::<Ipv6Addr>().unwrap().into(),
            //     false => self.client.hub_ip.parse::<Ipv4Addr>().unwrap().into(),
            // };
            let relay_ip = self.client.hub_ip.into();
            Some(
                Multiaddr::empty()
                .with(relay_ip)
                .with(Protocol::Tcp(self.client.hub_port))
                .with(Protocol::P2p(relay_id.into()))
            )
        } else {
            None
        }
    }
}
