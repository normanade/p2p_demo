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
};
use libp2p::relay::v2::relay::Relay;
use libp2p::autonat::{Config as AutoNatConfig, Behaviour as AutoNat};

use crate::Event;

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
                AutoNatConfig::default(),
            ),
        }
    }
}
