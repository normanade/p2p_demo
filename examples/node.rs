/// Relay server setup for hole punching.
/// Refer https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html
/// or https://blog.ipfs.io/2022-01-20-libp2p-hole-punching/
/// for concrete guide of usage.

use futures::join;
use std::io::Write;
use std::process::exit;
use std::time::Duration;
use log::{info, warn};

use async_std::task::block_on;
use async_std::task;
use async_std::channel;

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
    let role = conf.role.clone();
    conf.print_detail();

    let node = Node::new(conf);
    info!("Local peer id: {:?}", node.get_peer_id());

    node.bind().await;

    if let "hub" = role.as_str() {
        wait_response(&node).await
    }
    else {
        let (sender, receiver) = channel::bounded(1);

        // sets handler which respondes to user input in a spawned process
        ctrlc::set_handler(move || {
            if sender.is_closed() || sender.is_full() {
                return;
            }
            let mut user_input = String::new();
            let stdout_handle = std::io::stdout();
            let mut guard = stdout_handle.lock();
            guard.write(b"\r> ").expect("Write to stdout failed");
            guard.flush().expect("Flush stdout failed");
            std::io::stdin().read_line(&mut user_input).unwrap();
    
            block_on(async {
                sender.send(user_input).await.expect("Channel error")
            });
        }).expect("Error setting Ctrl-C handler");
    
        let f1 = async {
            loop {
                if let Ok(user_input) = receiver.recv().await {
                    match node.execute(user_input).await {
                        Ok(true) => break,
                        Ok(false) => task::sleep(Duration::from_micros(100)).await,
                        Err(err) => warn!("{}", err),
                    }
                }
            }
            exit(0);
        };
        let f2 = wait_response(&node);
        join!(f1, f2);
    }
}

async fn wait_response(node: &Node) {
    loop {
        task::sleep(Duration::from_micros(100)).await;
        node.wait().await;
    }
}
