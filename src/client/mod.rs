/// Defines Client

use libp2p::core::upgrade;
use libp2p::core::transport::OrTransport;
use libp2p::tcp::TcpConfig;
use libp2p::dns::DnsConfig;
use libp2p::Transport;
use libp2p::Multiaddr;
use libp2p::noise::NoiseConfig;
use libp2p::PeerId;
use libp2p::swarm::{Swarm, SwarmBuilder, SwarmEvent};
use libp2p::relay::v2::client::Client as RelayClient;
use futures::executor::block_on;
use futures::stream::StreamExt;
use std::error::Error;

pub mod behaviour;

use super::keys::Keys;
use behaviour::Behaviour;
use crate::Event::Relay as RelayEvent;

pub struct Client {
    pub keys: Keys,
    pub swarm: Swarm<Behaviour>,
}

impl Client {
    pub fn new() -> Self {
        let local_keys = Keys::new();
        let local_public_key = local_keys.key.public();

        let (relay_transport, client) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport = OrTransport::new(
            relay_transport,
            block_on(DnsConfig::system(TcpConfig::new().port_reuse(true))).unwrap(),
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(local_keys.noise_key.clone()).into_authenticated())
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .boxed();

        let swarm = SwarmBuilder::new(
            transport,
            Behaviour::new(local_public_key, client),
            local_keys.peer_id,
        )
        .dial_concurrency_factor(10_u8.try_into().unwrap())
        .build();

        Self {
            keys: local_keys,
            swarm: swarm,
        }
    }

    pub fn set_peer_id(&mut self) {
        self.keys.peer_id = PeerId::random();
    }

    pub fn listen(&mut self, addr: Multiaddr) {
        self.swarm.listen_on(addr).unwrap();
    }

    pub fn wait(&mut self) -> Result<(), Box<dyn Error>> {
        block_on(async {
            loop {
                match self.swarm.next().await.expect("Infinite Stream.") {
                    SwarmEvent::Behaviour(RelayEvent(event)) => {
                        println!("{:?}", event)
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    _ => {}
                }
            }
        })
    }
}
