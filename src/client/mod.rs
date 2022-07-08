/// Defines Client

use libp2p::core::upgrade;
use libp2p::core::transport::OrTransport;
use libp2p::tcp::{GenTcpConfig, TcpTransport};
use libp2p::dns::DnsConfig;
use libp2p::Transport;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use libp2p::noise::NoiseConfig;
use libp2p::PeerId;
use libp2p::swarm::{Swarm, SwarmBuilder, SwarmEvent};
use libp2p::relay::v2::client::{Event as RelayClientEventKinds, Client as RelayClient};
use libp2p::identify::{IdentifyEvent as IdentifyEventKinds, IdentifyInfo};
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::time::Duration;
use log::{info, error, debug};
use async_std::sync::{Arc, Mutex};
use futures::{join, select};

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
    pub swarm1: Arc<Mutex<Swarm<Behaviour>>>,
    // listen
    pub swarm2: Arc<Mutex<Swarm<Behaviour>>>,
}

impl Client {
    pub fn new() -> Self {
        let local_keys = Keys::new();
        let local_public_key1 = local_keys.key.public();
        let local_public_key2 = local_keys.key.public();

        let (relay_transport1, client1) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport1 = OrTransport::new(
            relay_transport1,
            block_on(DnsConfig::system(TcpTransport::new(
                GenTcpConfig::default().port_reuse(true),
            )))
            .unwrap()
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(local_keys.noise_key.clone()).into_authenticated())
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .boxed();

        let (relay_transport2, client2) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport2 = OrTransport::new(
            relay_transport2,
            block_on(DnsConfig::system(TcpTransport::new(
                GenTcpConfig::default().port_reuse(true),
            )))
            .unwrap()
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
            swarm1: Arc::new(Mutex::new(swarm1)),
            swarm2: Arc::new(Mutex::new(swarm2)),
        }
    }

    pub fn set_peer_id(&mut self) {
        self.keys.peer_id = PeerId::random();
    }

    pub async fn bind(&self, addr: Multiaddr) {
        let dial = self.dialer_bind(addr.clone());
        let listen = self.listener_bind(addr);
        join!(dial, listen);
    }

    pub async fn relay(&self, addr: Multiaddr) {
        // Dial relay not for the reservation or relayed connection, but to
        // (a) learn our local public address
        // (b) enable a freshly started relay to learn its public address
        let mut guard = self.swarm2.lock_arc().await;
        guard.dial(addr.clone()).unwrap();
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match guard.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(PingEvent(_)) => {}
                SwarmEvent::Behaviour(IdentifyEvent(IdentifyEventKinds::Sent { .. })) => {
                    info!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(IdentifyEvent(IdentifyEventKinds::Received {
                    info: IdentifyInfo { observed_addr, .. },
                    ..
                })) => {
                    info!("Relay told us our public address: {:?}", observed_addr);
                    learned_observed_addr = true;
                }
                event => panic!("{:?}", event),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }

        // listen from relay server
        // let mut guard = self.swarm2.lock_arc().await;
        guard.listen_on(addr.with(Protocol::P2pCircuit)).unwrap();
    }

    pub async fn relay_peer(&self, addr: Multiaddr, peer_id: PeerId) {
        let mut guard = self.swarm1.lock_arc().await;
        info!("swarm1 ready to dial peer {:?}", peer_id);
        guard.dial(
            addr.clone()
                .with(Protocol::P2pCircuit)
                .with(Protocol::P2p(peer_id.into()))
        ).unwrap();
    }
    
    // wait dialer and listener concurrently, every loop lasts 100 micro seconds
    pub async fn wait(&self) {
        let dial = self.dialer_wait();
        let listen = self.listener_wait();
        join!(dial, listen);
    }
}


impl Client {
    pub async fn dialer_bind(&self, addr: Multiaddr) {
        let mut guard = self.swarm1.lock_arc().await;
        guard.listen_on(addr.clone()).unwrap();
        
        let mut delay = futures_timer::Delay::new(Duration::from_secs(1)).fuse();
        loop { select! {
            event = guard.next() => {
                match event.unwrap() {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("swarm1 Listening on {:?}", address);
                    }
                    event => panic!("{:?}", event),
                }
            }
            _ = delay => {
                // Likely listening on all interfaces now, thus continuing by breaking the loop.
                break;
            }
        } }
    }

    pub async fn listener_bind(&self, addr: Multiaddr) {
        let mut guard = self.swarm2.lock_arc().await;
        guard.listen_on(addr.clone()).unwrap();
        
        let mut delay = futures_timer::Delay::new(Duration::from_micros(100)).fuse();
        loop { select! {
            event = guard.next() => {
                match event.unwrap() {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("swarm2 Listening on {:?}", address);
                    }
                    event => panic!("{:?}", event),
                }
            }
            _ = delay => {
                // Likely listening on all interfaces now, thus continuing by breaking the loop.
                break;
            }
        } }
    }

    pub async fn dialer_wait(&self) {
        let mut guard = self.swarm1.lock_arc().await;
        
        let mut delay = futures_timer::Delay::new(Duration::from_micros(100)).fuse();
        loop { select! {
            event = guard.next() => { match event.unwrap() {
                SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                    info!("swarm1 Identify {event:?}")
                }
                SwarmEvent::Behaviour(PingEvent(event)) => {
                    debug!("swarm1 Ping {event:?}")
                }
                SwarmEvent::Behaviour(DcutrEvent(event)) => {
                    info!("swarm1 Dcutr {event:?}")
                }
                SwarmEvent::Behaviour(RelayClientEvent(event)) => {
                    info!("swarm1 Relay {event:?}")
                }
                SwarmEvent::Behaviour(_) => todo!(),
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, num_established: _, concurrent_dial_errors: _
                } => {
                    info!("swarm1 Established connection to {peer_id:?} via {endpoint:?}");
                },
                SwarmEvent::ConnectionClosed {
                    peer_id, endpoint, num_established: _, cause
                } => {
                    error!("Connection with {peer_id:?}@{endpoint:?} closed due to {cause:?}");
                },
                SwarmEvent::IncomingConnection {
                    local_addr: _, send_back_addr
                } => {
                    debug!("swarm1 Incoming connection from {send_back_addr}");
                },
                SwarmEvent::IncomingConnectionError {
                    local_addr, send_back_addr, error
                } => todo!(),
                SwarmEvent::OutgoingConnectionError {
                    peer_id, error
                } => {
                    error!("swarm1 Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
                SwarmEvent::BannedPeer {
                    peer_id, endpoint
                } => todo!(),
                SwarmEvent::NewListenAddr {
                    listener_id, address
                } => {
                    info!("swarm1 Listening on {listener_id:?}@{address}");
                },
                SwarmEvent::ExpiredListenAddr {
                    listener_id, address
                } => {
                    info!("swarm1 Stopped listening to {listener_id:?}@{address}");
                },
                SwarmEvent::ListenerClosed {
                    listener_id, addresses, reason
                } => todo!(),
                SwarmEvent::ListenerError {
                    listener_id, error
                } => todo!(),
                SwarmEvent::Dialing(peer_id) => {
                    info!("swarm1 Dailing {peer_id:?}");
                }
            } }
            _ = delay => {
                // Timeout invoked, thus stop listening to swarm events
                break;
            }
        } }
    }

    pub async fn listener_wait(&self) {
        let mut guard = self.swarm2.lock_arc().await;
        
        let mut delay = futures_timer::Delay::new(Duration::from_micros(100)).fuse();
        loop { select! {
            event = guard.next() => { match event.unwrap() {
                SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                    info!("swarm2 Identify {event:?}")
                }
                SwarmEvent::Behaviour(PingEvent(event)) => {
                    debug!("swarm2 Ping {event:?}")
                }
                SwarmEvent::Behaviour(DcutrEvent(event)) => {
                    info!("swarm2 Dcutr {event:?}")
                }
                SwarmEvent::Behaviour(RelayClientEvent(event)) => {
                    info!("swarm2 Relay {event:?}")
                }
                SwarmEvent::Behaviour(_) => todo!(),
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint,
                    num_established: _, concurrent_dial_errors: _
                } => {
                    info!("swarm2 Established connection to {peer_id:?}@{endpoint:?}");
                },
                SwarmEvent::ConnectionClosed {
                    peer_id, endpoint,
                    num_established: _, cause
                } => {
                    error!("swarm2 Connection with {peer_id:?}@{endpoint:?} closed due to {cause:?}");
                },
                SwarmEvent::IncomingConnection {
                    local_addr: _, send_back_addr
                } => {
                    debug!("swarm2 Incoming connection from {send_back_addr}");
                },
                SwarmEvent::IncomingConnectionError {
                    local_addr: _, send_back_addr, error
                } => {
                    error!("swarm2 Incoming connection from {send_back_addr} error: {error}");
                },
                SwarmEvent::OutgoingConnectionError {
                    peer_id, error
                } => {
                    error!("swarm2 Error when connecting {:?}: {:?}", peer_id, error);
                }
                SwarmEvent::BannedPeer {
                    peer_id, endpoint
                } => todo!(),
                SwarmEvent::NewListenAddr {
                    listener_id, address
                } => {
                    info!("swarm2 Listening on {listener_id:?}@{address}");
                },
                SwarmEvent::ExpiredListenAddr {
                    listener_id, address
                } => {
                    info!("swarm2 Stopped listening to {listener_id:?}@{address}");
                },
                SwarmEvent::ListenerClosed {
                    listener_id, addresses: _, reason
                } => {
                    error!("swarm2 Listener {listener_id:?} closed due to {reason:?}");
                },
                SwarmEvent::ListenerError {
                    listener_id, error
                } => todo!(),
                SwarmEvent::Dialing(peer_id) => {
                    info!("swarm2 Dailing {peer_id:?}");
                }
            } }
            _ = delay => {
                // Timeout invoked, thus stop listening to swarm events
                break;
            }
        } }
    }
}