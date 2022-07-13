/// Defines Hub

use libp2p::core::upgrade;
use libp2p::tcp::TcpTransport;
use libp2p::Transport;
use libp2p::noise::NoiseConfig;
use libp2p::swarm::{Swarm, SwarmEvent};
use futures::future::FutureExt;
use futures::stream::StreamExt;
use log::{info, debug, error};
use async_std::sync::{Arc, Mutex};
use std::time::Duration;
use futures::select;

pub mod behaviour;

use super::conf;
use super::keys::Keys;
use behaviour::Behaviour;
use crate::Event::Relay as RelayEvent;
use crate::Event::Ping as PingEvent;
use crate::Event::Identify as IdentifyEvent;

pub struct Hub {
    pub keys: Keys,
    pub swarm: Arc<Mutex<Swarm<Behaviour>>>,
    conf: conf::Conf,
}

impl Hub {
    pub fn new(conf: conf::Conf) -> Self {
        let local_keys = Keys::new();
        let local_public_key = local_keys.key.public();

        let tcp_transport = TcpTransport::default();
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
            swarm: Arc::new(Mutex::new(swarm)),
            conf: conf,
        }
    }

    pub fn set_peer_id(&mut self) {
        self.keys.peer_id = libp2p::PeerId::random();
    }

    pub async fn bind(&self) {
        let listen_addr = self.conf.get_bind_address();

        let mut guard = self.swarm.lock_arc().await;
        guard.listen_on(listen_addr).unwrap();
        
        // Wait to listen on all interfaces.
        let mut delay = futures_timer::Delay::new(Duration::from_secs(1)).fuse();
        loop {
            futures::select! {
                event = guard.next() => {
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
    }

    // every loop lasts 100 micro seconds, preserves time for other tasks
    pub async fn wait(&self) {
        let mut guard = self.swarm.lock_arc().await;

        let mut delay = futures_timer::Delay::new(Duration::from_micros(100)).fuse();
        loop { select! {
            event = guard.next() => { match event.unwrap() {
                SwarmEvent::Behaviour(RelayEvent(event)) => {
                    info!("Relay {:?}", event)
                }
                SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                    debug!("Identify {:?}", event)
                }
                SwarmEvent::Behaviour(PingEvent(event)) => {
                    debug!("Ping {event:?}")
                }
                SwarmEvent::Behaviour(e) => {
                    info!("Event {:?}", e)
                },
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint,
                    // num_established, concurrent_dial_errors
                    ..
                } => {
                    debug!("Established connection to {peer_id:?}@{endpoint:?}");
                },
                SwarmEvent::ConnectionClosed {
                    peer_id, endpoint, num_established: _, cause
                } => {
                    error!("Connection with {peer_id:?}@{endpoint:?} closed due to {cause:?}");
                },
                SwarmEvent::IncomingConnection { local_addr, send_back_addr } => {
                    debug!("Received connection from {send_back_addr} to {local_addr}");
                },
                SwarmEvent::IncomingConnectionError {
                    // local_addr, send_back_addr, error
                    ..
                } => todo!(),
                SwarmEvent::OutgoingConnectionError {
                    // peer_id, error
                    ..
                } => todo!(),
                SwarmEvent::BannedPeer {
                    // peer_id, endpoint
                    ..
                } => todo!(),
                SwarmEvent::NewListenAddr {
                    listener_id, address
                } => {
                    info!("Listening on {listener_id:?}@{address}");
                },
                SwarmEvent::ExpiredListenAddr {
                    listener_id, address
                } => {
                    info!("Stopped listening to {listener_id:?}@{address}");
                },
                SwarmEvent::ListenerClosed {
                    // listener_id, addresses, reason
                    ..
                } => todo!(),
                SwarmEvent::ListenerError {
                    // listener_id, error
                    ..
                } => todo!(),
                SwarmEvent::Dialing(_) => todo!(),
            } }
            _ = delay => {
                // Timeout invoked, thus stop listening to swarm events
                break;
            }
        } }
    }
}
