///

use libp2p::PeerId;
use libp2p::Multiaddr;

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

    pub async fn bind(&self, addr: Multiaddr) {
        match self {
            Node::Hub(x) => x.bind(addr).await,
            Node::Client(x) => x.bind(addr).await,
        };
    }

    pub async fn relay(&self, addr: Option<Multiaddr>) {
        match self {
            Node::Client(x) => x.relay(addr.unwrap()).await,
            _ => unreachable!(),
        };
    }

    pub async fn dial(&self, addr: Option<Multiaddr>, peer_id: PeerId) {
        match self {
            Node::Client(x) => x.relay_peer(addr.unwrap(), peer_id).await,
            _ => unreachable!(),
        };
    }

    pub async fn wait(&self) {
        match self {
            Node::Hub(x) => {
                x.wait().await;
            }
            Node::Client(x) => {
                x.wait().await;
            }
        }
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
