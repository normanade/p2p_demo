/// Relay server setup for hole punching.
/// Refer https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html
/// or https://blog.ipfs.io/2022-01-20-libp2p-hole-punching/
/// for concrete guide of usage.

use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};

// use p2p_demo::hub::Hub;
use p2p_demo::conf::Conf;
use p2p_demo::Node;

const CONFIG_PATH: &str = "node.ini";

fn main() -> Result<(), Box<dyn Error>> {
    // std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let conf = Conf::new(CONFIG_PATH);
    println!("conf: {:?}", conf);

    let mut node = Node::new(conf.role.clone());
    println!("Local peer id: {:?}", node.get_peer_id());
        
    // Listen on all interfaces
    let port = conf.get_bind_port();
    let listen_addr = if let true = conf.use_ipv6 {
        Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(port))
            .with(Protocol::from(Ipv6Addr::UNSPECIFIED))
    } else {
        Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(port))
    };
    node.listen(listen_addr);
    node.wait()
}
