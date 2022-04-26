/// Defines Client

use libp2p::core::upgrade;
use libp2p::tcp::TcpConfig;
use libp2p::Transport;
use libp2p::Multiaddr;
use libp2p::noise::NoiseConfig;
use libp2p::PeerId;
use libp2p::swarm::{Swarm, SwarmEvent};
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

        let tcp_transport = TcpConfig::new();
        let transport = tcp_transport
            .upgrade(upgrade::Version::V1)
            .authenticate(NoiseConfig::xx(local_keys.noise_key.clone()).into_authenticated())
            .multiplex(libp2p::yamux::YamuxConfig::default())
            .boxed();
        let swarm = Swarm::new(
            transport,
            Behaviour::new(local_public_key, local_keys.peer_id),
            local_keys.peer_id,
        );

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
    
    pub fn add_nat_server(&mut self, peer_id: PeerId, server_addr: Multiaddr) {
        self.swarm.behaviour_mut().add_nat_server(peer_id, server_addr);
    }
}
