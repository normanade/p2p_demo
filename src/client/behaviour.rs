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
};
use libp2p::relay::v2::client::Client;
use libp2p::dcutr::behaviour::Behaviour as Dcutr;

use crate::Event;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    ping: Ping,
    identify: Identify,
    relay_client: Client,
    dcutr: Dcutr,
}

impl Behaviour {
    pub fn new(public_key: PublicKey, client: Client) -> Self {
        Self {
            ping: Ping::new(PingConfig::new()),
            identify: Identify::new(IdentifyConfig::new(
                "/TODO/0.0.1".to_string(),
                public_key,
            )),
            relay_client: client,
            dcutr: Dcutr::new(),
        }
    }
}
