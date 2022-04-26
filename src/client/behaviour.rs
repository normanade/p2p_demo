/// Defines hub's behaviour when `Event` is happening

use libp2p::identify::{
    Identify,
    IdentifyConfig,
};
use libp2p::ping::{
    Ping,
    PingConfig,
};
use libp2p::{
    identity::PublicKey,
    NetworkBehaviour,
    PeerId,
    Multiaddr,
};
use libp2p::relay::v2::relay::Relay;
use libp2p::autonat::{Config as AutoNatConfig, Behaviour as AutoNat};

use crate::Event;
use std::time::Duration;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    relay: Relay,
    ping: Ping,
    identify: Identify,
    autonat: AutoNat,
}

impl Behaviour {
    pub fn new(public_key: PublicKey, peer_id: PeerId) -> Self {
        Self {
            relay: Relay::new(peer_id, Default::default()),
            ping: Ping::new(PingConfig::new()),
            identify: Identify::new(IdentifyConfig::new(
                "/TODO/0.0.1".to_string(),
                public_key,
            )),
            autonat: AutoNat::new(
                peer_id,
                AutoNatConfig {
                    retry_interval: Duration::from_secs(10),
                    refresh_interval: Duration::from_secs(30),
                    boot_delay: Duration::from_secs(5),
                    throttle_server_period: Duration::ZERO,
                    ..Default::default()
                },
            ),
        }
    }

    pub fn add_nat_server(&mut self, peer_id: PeerId, server_addr: Multiaddr) {
        self.autonat.add_server(peer_id, Some(server_addr));
    }
}
