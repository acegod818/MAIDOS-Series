//! Simple P2P example demonstrating handcrafted implementation

use maidos_p2p::{P2pConfig, P2pNode};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MAIDOS Handcrafted P2P Example");

    // Create P2P node configuration
    let config = P2pConfig {
        listen_addr: "127.0.0.1:5000".to_string(),
        bootstrap_nodes: vec!["127.0.0.1:5001".to_string()],
        max_peers: 10,
        heartbeat_interval: 30,
    };

    // Start the P2P node
    let mut node = P2pNode::new(config);
    node.start().await?;
    println!("Node {} started successfully", node.node_id());

    // Run for a while to demonstrate functionality
    println!("Running P2P node for 30 seconds...");
    sleep(Duration::from_secs(30)).await;

    // Stop the node
    node.stop().await;
    println!("Node stopped");

    Ok(())
}
