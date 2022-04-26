/// Node start options

use clap::Parser;
use libp2p::{PeerId, Multiaddr};

#[derive(Debug, Parser)]
#[clap(name = "libp2p relay")]
pub struct HubOpt {
    /// Starts as client or as hub
    #[clap(long)]
    pub as_client: Option<bool>,

    /// Determine if the relay listen on ipv6 or ipv4 loopback address. the default is ipv4
    #[clap(long)]
    pub use_ipv6: Option<bool>,

    /// The port used to listen on all interfaces
    #[clap(long)]
    pub port: u16,
}

#[derive(Debug, Parser)]
#[clap(name = "libp2p autonat")]
pub struct ClientOpt {
    #[clap(long)]
    pub port: u16,

    #[clap(long)]
    pub server_address: Multiaddr,

    #[clap(long)]
    pub server_peer_id: PeerId,
}
