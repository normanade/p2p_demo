use async_std::task::block_on;
/// Relay server setup for hole punching.
/// Refer https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html
/// or https://blog.ipfs.io/2022-01-20-libp2p-hole-punching/
/// for concrete guide of usage.

use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use libp2p::PeerId;
use std::str::FromStr;
use std::net::{Ipv4Addr, Ipv6Addr};
use log::{info, debug};

use async_std::sync::Mutex;
use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};

use p2p_demo::conf::Conf;
use p2p_demo::Node;

const CONFIG_PATH: &str = "node.ini";

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    block_on(async_main());
}

async fn async_main() {
    let conf = Conf::new(CONFIG_PATH);
    debug!("Config file from {}: {:?}", CONFIG_PATH, conf);

    let mut node = Mutex::new(Node::new(conf.role.clone()));
    info!("Local peer id: {:?}", node.get_mut().get_peer_id());

    bind_local_address(&conf, node.get_mut());

    // only effective when `role' is client
    // connect to configured relay server
    if let "client" = conf.role.as_str() {
        let relay_addr = conf.get_relay_address();
        node.get_mut().relay(relay_addr.clone());

        let f1 = get_peer_id().fuse();
        let f2 = wait_response(&node).fuse();
        pin_mut!(f1, f2);
        
        select! {
            peer = f1 => {
                let mut guard = node.lock().await;
                guard.dial(relay_addr.clone(), peer);
                drop(guard);
            },
            _ = f2 => unreachable!(),
        }
    }
    else {
        node.get_mut().wait().await.unwrap();
    }
}

fn bind_local_address(conf: &Conf, node: &mut Node) {
    // Listen on all interfaces
    let port = conf.get_bind_port();
    let listen_addr = match conf.use_ipv6 {
        true => Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(port))
            .with(Protocol::from(Ipv6Addr::UNSPECIFIED)),
        false => Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(port)),
    };

    node.listen(listen_addr);
}

async fn get_peer_id() -> PeerId {
    println!("Please input relay client PeerID:");
    let mut input = String::new();
    async_std::io::stdin().read_line(&mut input).await.unwrap();
    PeerId::from_str(input.trim()).expect("Invalid PeerID")
}

async fn wait_response(node: &Mutex<Node>) {
    let mut guard = node.lock().await;
    guard.wait().await.unwrap();
    drop(guard);
}
