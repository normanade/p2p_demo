/// Relay server setup for hole punching.
/// Refer https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html
/// or https://blog.ipfs.io/2022-01-20-libp2p-hole-punching/
/// for concrete guide of usage.

use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use clap::Parser;

use p2p_demo::hub::Hub;
use p2p_demo::opt::HubOpt;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = HubOpt::parse();
    println!("opt: {:?}", opt);
    let mut hub = Hub::new();
    println!("Local peer id: {:?}", hub.keys.peer_id);

    // Listen on all interfaces
    let listen_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));

    hub.listen(listen_addr);
    hub.wait()
}
