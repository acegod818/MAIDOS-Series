use crate::{Peer, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Simple peer discovery service
pub struct DiscoveryService {
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    config: DiscoveryConfig,
}

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

impl DiscoveryService {
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn discovery_interval(&self) -> u64 {
        self.config.discovery_interval
    }

    /// Add a new peer to the discovery service
    #[allow(dead_code)]
    pub async fn add_peer(&self, peer: Peer) {
        let peer_clone = peer.clone();
        let mut peers = self.peers.write().await;
        peers.insert(peer.id.clone(), peer_clone);
        info!("Added peer: {} ({})", peer.id, peer.addr);
    }

    /// Remove an expired peer
    #[allow(dead_code)]
    pub async fn remove_expired_peer(&self, peer_id: &str) -> bool {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get(peer_id) {
            if peer.is_expired(self.config.peer_timeout) {
                peers.remove(peer_id);
                info!("Removed expired peer: {}", peer_id);
                return true;
            }
        }
        false
    }

    /// Get all active peers
    #[allow(dead_code)]
    pub async fn get_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Get a specific peer by ID
    #[allow(dead_code)]
    pub async fn get_peer(&self, peer_id: &str) -> Option<Peer> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// Discover peers from bootstrap nodes
    pub async fn discover_peers(&self) -> Result<Vec<SocketAddr>> {
        let mut discovered = Vec::new();
        
        for node in &self.config.bootstrap_nodes {
            if let Ok(addr) = node.parse::<SocketAddr>() {
                if self.check_peer_online(&addr).await {
                    discovered.push(addr);
                }
            }
        }
        
        Ok(discovered)
    }

    /// Simple peer online check
    async fn check_peer_online(&self, addr: &SocketAddr) -> bool {
        // Simple TCP connection check
        tokio::net::TcpStream::connect(addr).await.is_ok()
    }

    /// Clean up expired peers
    #[allow(dead_code)]
    pub async fn cleanup_expired(&self) -> usize {
        let mut removed_count = 0;
        let mut peers = self.peers.write().await;
        
        let expired_peers: Vec<String> = peers.values()
            .filter(|peer| peer.is_expired(self.config.peer_timeout))
            .map(|peer| peer.id.clone())
            .collect();
            
        for peer_id in expired_peers {
            peers.remove(&peer_id);
            removed_count += 1;
        }
        
        if removed_count > 0 {
            info!("Cleaned up {} expired peers", removed_count);
        }
        
        removed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_add_get_and_remove_peer() {
        let config = DiscoveryConfig {
            bootstrap_nodes: vec![],
            discovery_interval: 1,
            peer_timeout: 1,
        };
        let service = DiscoveryService::new(config);

        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let mut peer = Peer::new("peer-1".to_string(), addr);
        peer.last_seen = 0;
        service.add_peer(peer).await;

        let fetched = service.get_peer("peer-1").await;
        assert!(fetched.is_some());

        let removed = service.remove_expired_peer("peer-1").await;
        assert!(removed);

        let fetched = service.get_peer("peer-1").await;
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_discover_peers_filters_invalid() {
        let config = DiscoveryConfig {
            bootstrap_nodes: vec!["127.0.0.1:0".to_string(), "invalid".to_string()],
            discovery_interval: 1,
            peer_timeout: 1,
        };
        let service = DiscoveryService::new(config);
        let peers = service.discover_peers().await.unwrap();
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = DiscoveryConfig {
            bootstrap_nodes: vec![],
            discovery_interval: 1,
            peer_timeout: 1,
        };
        let service = DiscoveryService::new(config);
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let mut peer = Peer::new("peer-1".to_string(), addr);
        peer.last_seen = 0;
        service.add_peer(peer).await;
        let removed = service.cleanup_expired().await;
        assert_eq!(removed, 1);
    }
}
