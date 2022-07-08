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

use async_std::task::block_on;
use futures::join;
use std::time::Duration;
use async_std::task;

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

    let node = Node::new(conf.role.clone());
    info!("Local peer id: {:?}", node.get_peer_id());

    bind_local_address(&conf, &node).await;

    // only effective when `role' is client
    // connect to configured relay server
    if let "client" = conf.role.as_str() {
        let relay_addr = conf.get_relay_address();
        node.relay(relay_addr.clone()).await;

        let f1 = dial_peer_with_relay(&node, relay_addr);
        let f2 = wait_response(&node);
        join!(f1, f2);
    }
    else {
        wait_response(&node).await
    }
}

async fn bind_local_address(conf: &Conf, node: &Node) {
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

    node.bind(listen_addr).await;
}


async fn dial_peer_with_relay(node: &Node, relay_addr: Option<Multiaddr>) {
    let peer = input_peer_id().await;
    node.dial(relay_addr, peer).await;
}

async fn input_peer_id() -> PeerId {
    // wait 3 seconds till swarms connected to relay server
    task::sleep(Duration::from_secs(5)).await;
    println!("Please input relay client PeerID:");
    let mut input = String::new();
    async_std::io::stdin().read_line(&mut input).await.unwrap();
    PeerId::from_str(input.trim()).expect("Invalid PeerID")
}

async fn wait_response(node: &Node) {
    loop {
        // task::sleep(Duration::from_micros(100)).await;
        node.wait().await;
    }
}
