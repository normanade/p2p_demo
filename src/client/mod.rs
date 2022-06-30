/// Defines Client

use libp2p::core::upgrade;
use libp2p::core::transport::OrTransport;
use libp2p::tcp::TcpConfig;
use libp2p::dns::DnsConfig;
use libp2p::Transport;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use libp2p::noise::NoiseConfig;
use libp2p::PeerId;
use libp2p::swarm::{Swarm, SwarmBuilder, SwarmEvent};
use libp2p::relay::v2::client::Client as RelayClient;
use libp2p::identify::{IdentifyEvent as IdentifyEventKinds, IdentifyInfo};
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::error::Error;

pub mod behaviour;

use super::keys::Keys;
use behaviour::Behaviour;
use crate::Event::RelayClient as RelayClientEvent;
use crate::Event::Identify as IdentifyEvent;
use crate::Event::Ping as PingEvent;
use crate::Event::Dcutr as DcutrEvent;

pub struct Client {
    pub keys: Keys,
    // dial
    pub swarm1: Swarm<Behaviour>,
    // listen
    pub swarm2: Swarm<Behaviour>,
}

impl Client {
    pub fn new() -> Self {
        let local_keys = Keys::new();
        let local_public_key1 = local_keys.key.public();
        let local_public_key2 = local_keys.key.public();

        let (relay_transport1, client1) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport1 = OrTransport::new(
            relay_transport1,
            block_on(DnsConfig::system(TcpConfig::new().port_reuse(true))).unwrap(),
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(local_keys.noise_key.clone()).into_authenticated())
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .boxed();

        let (relay_transport2, client2) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport2 = OrTransport::new(
            relay_transport2,
            block_on(DnsConfig::system(TcpConfig::new().port_reuse(true))).unwrap(),
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(local_keys.noise_key.clone()).into_authenticated())
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .boxed();

        let swarm1 = SwarmBuilder::new(
            transport1,
            Behaviour::new(local_public_key1, client1),
            local_keys.peer_id,
        )
        .dial_concurrency_factor(10_u8.try_into().unwrap())
        .build();
        let swarm2 = SwarmBuilder::new(
            transport2,
            Behaviour::new(local_public_key2, client2),
            local_keys.peer_id,
        )
        .dial_concurrency_factor(10_u8.try_into().unwrap())
        .build();

        Self {
            keys: local_keys,
            swarm1: swarm1,
            swarm2: swarm2,
        }
    }

    pub fn set_peer_id(&mut self) {
        self.keys.peer_id = PeerId::random();
    }

    pub fn listen(&mut self, addr: Multiaddr) {
        self.swarm1.listen_on(addr.clone()).unwrap();
        self.swarm2.listen_on(addr).unwrap();

        // Wait to listen on all interfaces.
        block_on(async {
            let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
            loop {
                futures::select! {
                    event = self.swarm1.next() => {
                        match event.unwrap() {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                println!("swarm1 Listening on {:?}", address);
                            }
                            event => panic!("{:?}", event),
                        }
                    }
                    event = self.swarm2.next() => {
                        match event.unwrap() {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                println!("swarm2 Listening on {:?}", address);
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

    pub fn relay(&mut self, addr: Multiaddr) {
        // dial the relay server
        self.swarm1.dial(addr.clone()).unwrap();
        // Wait till connected to relay to learn external address.
        block_on(async {
            loop {
                match self.swarm1.next().await.unwrap() {
                    SwarmEvent::NewListenAddr { .. } => {}
                    SwarmEvent::Dialing { .. } => {}
                    SwarmEvent::ConnectionEstablished { .. } => {}
                    SwarmEvent::Behaviour(PingEvent(_)) => {}
                    SwarmEvent::Behaviour(IdentifyEvent(IdentifyEventKinds::Sent { .. })) => {}
                    SwarmEvent::Behaviour(IdentifyEvent(IdentifyEventKinds::Received {
                        info: IdentifyInfo { observed_addr, .. },
                        ..
                    })) => {
                        println!("swarm1 Observed address through relay: {:?}", observed_addr);
                        break;
                    }
                    event => panic!("{:?}", event),
                }
            }
        });

        // listen from relay server
        self.swarm2.listen_on(addr.with(Protocol::P2pCircuit)).unwrap();
    }

    pub fn relay_peer(&mut self, addr: Multiaddr, peer_id: PeerId) {
        // peer with smaller id, dials the other side
        if self.keys.peer_id < peer_id {
            self.swarm1.dial(
                addr.clone()
                    .with(Protocol::P2pCircuit)
                    .with(Protocol::P2p(peer_id.into()))
            ).unwrap();
        }
    }

    pub fn wait(&mut self) -> Result<(), Box<dyn Error>> {
        block_on(async {
            loop {
                match self.swarm1.next().await.expect("Infinite Stream.") {
                    SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                        println!("swarm1 Identify {:?}", event)
                    }
                    SwarmEvent::Behaviour(DcutrEvent(event)) => {
                        println!("swarm1 Dcutr {:?}", event)
                    }
                    SwarmEvent::Behaviour(RelayClientEvent(event)) => {
                        println!("swarm1 Relay {:?}", event)
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        println!("swarm1 Established connection to {:?} via {:?}", peer_id, endpoint);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                        println!("swarm1 Outgoing connection error to {:?}: {:?}", peer_id, error);
                    }
                    _ => {}
                }
                match self.swarm2.next().await.expect("Infinite Stream.") {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("swarm2 Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(PingEvent(_)) => {}
                    SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                        println!("swarm2 Identify {:?}", event)
                    }
                    SwarmEvent::Behaviour(DcutrEvent(event)) => {
                        println!("swarm2 Dcutr {:?}", event)
                    }
                    SwarmEvent::Behaviour(RelayClientEvent(event)) => {
                        println!("swarm2 Relay {:?}", event)
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        println!("swarm2 Established connection to {:?} via {:?}", peer_id, endpoint);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                        println!("swarm2 Outgoing connection error to {:?}: {:?}", peer_id, error);
                    }
                    _ => {}
                }
            }
        })
    }
}
