use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use std::error::Error;
use std::net::{Ipv4Addr};
use clap::Parser;

use p2p_demo::client::Client;
use p2p_demo::opt::ClientOpt;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = ClientOpt::parse();
    println!("opt: {:?}", opt);

    let mut client = Client::new();
    println!("Local peer id: {:?}", client.keys.peer_id);
    
    let listen_addr = Multiaddr::empty()
        .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Tcp(opt.port));
    
    client.listen(listen_addr);
    client.add_nat_server(opt.server_peer_id, opt.server_address);

    client.wait()
}
