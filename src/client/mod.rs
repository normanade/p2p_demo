/// Defines Client

use libp2p::core::upgrade;
use libp2p::core::transport::OrTransport;
use libp2p::tcp::{GenTcpConfig, TcpTransport};
use libp2p::dns::DnsConfig;
use libp2p::Transport;
use libp2p::multiaddr::Protocol;
use libp2p::noise::NoiseConfig;
use libp2p::PeerId;
use libp2p::swarm::{Swarm, SwarmBuilder, SwarmEvent};
// use libp2p::relay::v2::client::{Event as RelayClientEventKinds, Client as RelayClient};
use libp2p::relay::v2::client::Client as RelayClient;
use libp2p::identify::{IdentifyEvent as IdentifyEventKinds, IdentifyInfo};
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use std::time::Duration;
use std::str::FromStr;
use log::{info, error, debug};
use async_std::sync::{Arc, Mutex, RwLock};
use futures::select;

pub mod behaviour;

use super::conf;
use super::keys::Keys;
use behaviour::Behaviour;
use crate::Event::RelayClient as RelayClientEvent;
use crate::Event::Identify as IdentifyEvent;
use crate::Event::Ping as PingEvent;
use crate::Event::Dcutr as DcutrEvent;

pub struct Client {
    pub keys: Keys,
    pub swarm: Arc<Mutex<Swarm<Behaviour>>>,
    conf: conf::Conf,
    relay_id: RwLock<Option<PeerId>>,
}

impl Client {
    pub fn new(conf: conf::Conf) -> Self {
        let local_keys = Keys::new();
        let local_public_key = local_keys.key.public();

        let (relay_transport, client) = RelayClient::new_transport_and_behaviour(local_keys.peer_id);
        let transport = OrTransport::new(
            relay_transport,
            block_on(DnsConfig::system(TcpTransport::new(
                GenTcpConfig::default().port_reuse(true),
            )))
            .unwrap()
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
            swarm: Arc::new(Mutex::new(swarm)),
            conf: conf,
            relay_id: RwLock::new(None),
        }
    }

    pub fn set_peer_id(&mut self) {
        self.keys.peer_id = PeerId::random();
    }

    pub async fn bind(&self) {
        let listen_addr = self.conf.get_bind_address();

        let mut guard = self.swarm.lock_arc().await;
        guard.listen_on(listen_addr).unwrap();
        
        let mut delay = futures_timer::Delay::new(Duration::from_secs(1)).fuse();
        loop { select! {
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
        } }
    }

    pub async fn execute(&self, user_input: String) -> Result<bool, String> {
        let mut iter = user_input.split_whitespace();
        match iter.next() {
            Some("relay") | Some("r") => {
                if let Some(peer_id) = iter.next() {
                    match PeerId::from_str(peer_id) {
                        Ok(peer_id) => {
                            self.relay(peer_id).await;
                            Ok(false)
                        },
                        Err(err) => {
                            Err(err.to_string() + " - PeerId invalid!")
                        },
                    }
                }
                else {
                    Err("Please input peerid as the second param.".to_string())
                }
            },
            Some("dial") | Some("d") => {
                if let Some(peer_id) = iter.next() {
                    match PeerId::from_str(peer_id) {
                        Ok(peer_id) => {
                            self.relay_peer(peer_id).await;
                            Ok(false)
                        },
                        Err(err) => {
                            Err(err.to_string() + " - PeerId invalid!")
                        }
                    }
                }
                else {
                    Err("Please input peerid as the second param.".to_string())
                }
            },
            Some("quit") | Some("q") => Ok(true),
            None => Ok(false),
            _ => {
                Err("Command invalid!".to_string())
            }
        }
    }

    pub async fn relay(&self, relay_id: PeerId) {
        let mut writer = self.relay_id.write().await;
        *writer = Some(relay_id);
        drop(writer);
        let addr = self.conf.get_relay_address(relay_id).unwrap();
        // Dial relay not for the reservation or relayed connection, but to:
        // (a) learn our local public address,
        // (b) enable a freshly started relay to learn its public address.
        // If reservation is requested when relay hasn't acknowledged
        // its public address yet, the reservation will fail.
        info!("Getting swarm dial lock");
        let mut guard = self.swarm.lock_arc().await;
        info!("swarm dial lock success");
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
                SwarmEvent::OutgoingConnectionError {
                    peer_id, error
                } => {
                    error!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
                event => panic!("{:?}", event),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }

        // listen from relay server
        guard.listen_on(addr.with(Protocol::P2pCircuit)).unwrap();
    }

    pub async fn relay_peer(&self, peer_id: PeerId) {
        let reader = self.relay_id.read().await;
        if let Some(relay_id) = *reader {
            let addr = self.conf.get_relay_address(relay_id).unwrap();
            let mut guard = self.swarm.lock_arc().await;
            info!("Ready to dial peer {:?}", peer_id);
            guard.dial(
                addr.clone()
                    .with(Protocol::P2pCircuit)
                    .with(Protocol::P2p(peer_id.into()))
            ).unwrap();
        }
        else {
            error!("Relay not found, can't dial peer!");
        }
    }
    
    // wait dialer and listener concurrently, every loop lasts 100 micro seconds
    pub async fn wait(&self) {
        let mut guard = self.swarm.lock_arc().await;
        
        let mut delay = futures_timer::Delay::new(Duration::from_micros(100)).fuse();
        loop { select! {
            event = guard.next() => { match event.unwrap() {
                SwarmEvent::Behaviour(IdentifyEvent(event)) => {
                    info!("Identify {event:?}")
                }
                SwarmEvent::Behaviour(PingEvent(event)) => {
                    info!("Ping {event:?}")
                }
                SwarmEvent::Behaviour(DcutrEvent(event)) => {
                    info!("Dcutr {event:?}")
                }
                SwarmEvent::Behaviour(RelayClientEvent(event)) => {
                    info!("Relay {event:?}")
                }
                SwarmEvent::Behaviour(_) => todo!(),
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, num_established: _, concurrent_dial_errors: _
                } => {
                    info!("Established connection to {peer_id:?} via {endpoint:?}");
                },
                SwarmEvent::ConnectionClosed {
                    peer_id, endpoint, num_established: _, cause
                } => {
                    error!("Connection with {peer_id:?}@{endpoint:?} closed due to {cause:?}");
                },
                SwarmEvent::IncomingConnection {
                    local_addr: _, send_back_addr
                } => {
                    debug!("Incoming connection from {send_back_addr}");
                },
                SwarmEvent::IncomingConnectionError {
                    local_addr: _, send_back_addr, error
                } => {
                    error!("Incoming connection from {send_back_addr} error: {error}");
                },
                SwarmEvent::OutgoingConnectionError {
                    peer_id, error
                } => {
                    error!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
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
                    listener_id, addresses: _, reason
                } => {
                    error!("Listener {listener_id:?} closed due to {reason:?}");
                },
                SwarmEvent::ListenerError {
                    // listener_id, error
                    ..
                } => todo!(),
                SwarmEvent::Dialing(peer_id) => {
                    info!("Dailing {peer_id:?}");
                }
            } }
            _ = delay => {
                // Timeout invoked, thus stop listening to swarm events
                break;
            }
        } }
    }
}
