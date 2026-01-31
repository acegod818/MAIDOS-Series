//! MAIDOS Handcrafted P2P Implementation
//!
//! Simple P2P networking without complex dependencies.
//! Features: TCP-based messaging, peer discovery, message routing.

mod error;
mod node;
mod peer;
mod discovery;
mod message;
mod transport;

pub use error::{P2pError, Result};
pub use node::P2pNode;
pub use peer::{Peer, DiscoveryConfig};
pub use message::Message;
pub use transport::TcpTransport;

/// Simple P2P configuration
#[derive(Debug, Clone)]
pub struct P2pConfig {
    pub listen_addr: String,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
    pub heartbeat_interval: u64,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:5000".to_string(),
            bootstrap_nodes: Vec::new(),
            max_peers: 100,
            heartbeat_interval: 30,
        }
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_node_creation() {
        let config = P2pConfig::default();
        let node = P2pNode::new(config);
        assert!(!node.node_id().is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new_text("node1", "node2", "Hello");
        let data = msg.serialize().expect("Serialization failed");
        let msg2 = Message::deserialize(&data).expect("Deserialization failed");
        assert_eq!(msg.from, msg2.from);
        assert_eq!(msg.to, msg2.to);
    }

    #[test]
    fn test_peer_management() {
        let addr = "127.0.0.1:8080".parse().expect("Invalid address");
        let peer = Peer::new("test-peer".to_string(), addr);
        assert_eq!(peer.id, "test-peer");
        assert!(!peer.is_connected);
    }
}
