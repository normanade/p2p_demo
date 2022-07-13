///

use libp2p::PeerId;

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
    pub fn new(conf: conf::Conf) -> Self {
        match conf.role.trim() {
            "hub" => Node::Hub(hub::Hub::new(conf)),
            "client" => Node::Client(client::Client::new(conf)),
            _ => panic!("No such role!")
        }
    }
    
    pub fn get_peer_id(&self) -> PeerId {
        match self {
            Node::Hub(x) => x.keys.peer_id,
            Node::Client(x) => x.keys.peer_id,
        }
    }

    pub async fn bind(&self) {
        match self {
            Node::Hub(x) => x.bind().await,
            Node::Client(x) => x.bind().await,
        };
    }
    
    pub async fn execute(&self, user_input: String) -> Result<bool, String> {
        match self {
            Node::Client(x) => x.execute(user_input).await,
            _ => unreachable!(),
        }
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
