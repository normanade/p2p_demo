/// Defines Hub

use libp2p::core::upgrade;
use libp2p::tcp::TcpConfig;
use libp2p::Transport;
use libp2p::Multiaddr;
use libp2p::noise::NoiseConfig;
use libp2p::swarm::{Swarm, SwarmEvent};
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::error::Error;
use log::info;

pub mod behaviour;

use super::keys::Keys;
use behaviour::Behaviour;
use crate::Event::Relay as RelayEvent;
use crate::Event::Identify as IdentifyEvent;

pub struct Hub {
    pub keys: Keys,
    pub swarm: Swarm<Behaviour>,
}

impl Hub {
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
        self.keys.peer_id = libp2p::PeerId::random();
    }

    pub fn listen(&mut self, addr: Multiaddr) {
        self.swarm.listen_on(addr).unwrap();
        
        // Wait to listen on all interfaces.
        block_on(async {
            let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
            loop {
                futures::select! {
                    event = self.swarm.next() => {
                        match event.unwrap() {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                info!("Listening on {:?}", address);
                            }
                            event => panic!("{:?}", event),
                        }
                    }
                    _ = delay => {
                        // Likely listening on all interfaces now, thus continuing by breaking the loop.
                        break;
                    }
                }
            }
        });
    }

    pub async fn wait(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(RelayEvent(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                    info!("{:?}", event)
                }
                _ => {}
            }
        }
    }
}
