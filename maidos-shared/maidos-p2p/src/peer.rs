use std::net::SocketAddr;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Represents a peer in the P2P network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub addr: SocketAddr,
    pub last_seen: u64,
    pub is_connected: bool,
}

impl Peer {
    pub fn new(id: String, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            last_seen: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            is_connected: false,
        }
    }

    pub fn update_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.last_seen) > timeout_secs
    }
}

/// Simple peer discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub bootstrap_nodes: Vec<String>,
    pub discovery_interval: u64,
    pub peer_timeout: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: Vec::new(),
            discovery_interval: 60,
            peer_timeout: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_new_and_expired() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let mut peer = Peer::new("peer-1".to_string(), addr);
        assert!(!peer.is_connected);
        peer.last_seen = 0;
        assert!(peer.is_expired(1));
    }

    #[test]
    fn test_peer_update_seen() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let mut peer = Peer::new("peer-1".to_string(), addr);
        let before = peer.last_seen;
        peer.update_seen();
        assert!(peer.last_seen >= before);
    }
}
