///

use libp2p::PeerId;
use libp2p::Multiaddr;
use std::time::Duration;
use async_std::future;

pub mod keys;
pub mod conf;

mod hub;
mod client;
mod event;
pub use event::Event;

pub enum Node {
    Hub(hub::Hub),
    Client(client::Client),
}

impl Node {
    pub fn new(role: String) -> Self {
        match role.as_str() {
            "hub" => Node::Hub(hub::Hub::new()),
            "client" => Node::Client(client::Client::new()),
            _ => panic!("No such role!")
        }
    }
    
    pub fn get_peer_id(&self) -> PeerId {
        match self {
            Node::Hub(x) => x.keys.peer_id,
            Node::Client(x) => x.keys.peer_id,
        }
    }

    pub fn listen(&mut self, addr: Multiaddr) {
        match self {
            Node::Hub(x) => x.listen(addr),
            Node::Client(x) => x.listen(addr),
        };
    }

    pub fn relay(&mut self, addr: Option<Multiaddr>) {
        match self {
            Node::Client(x) => x.relay(addr.unwrap()),
            _ => (),
        };
    }

    pub fn dial(&mut self, addr: Option<Multiaddr>, peer_id: PeerId) {
        match self {
            Node::Client(x) => x.relay_peer(addr.unwrap(), peer_id),
            _ => (),
        };
    }

    pub async fn wait(&mut self) {
        let dur = Duration::from_millis(500);
        match self {
            Node::Hub(x) => future::timeout(dur, x.wait()).await.unwrap_or(()),
            Node::Client(x) => future::timeout(dur, x.wait()).await.unwrap_or(()),
        };
        ()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
