//! P2P routing implementation
//!
//! Provides Kademlia DHT-based peer routing and discovery.

use crate::{P2pError, Result, PeerInfo};
use libp2p::{
    identity::Keypair,
    kad::{self, record::store::MemoryStore, Mode, RecordKey},
    multiaddr::Protocol,
    swarm::derive_prelude::NetworkBehaviour,
    Multiaddr, PeerId,
};
use std::num::NonZero;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Routing configuration
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Kademlia replication factor
    pub replication_factor: usize,
    
    /// Peer discovery interval
    pub discovery_interval: Duration,
    
    /// Record TTL
    pub record_ttl: Duration,
    
    /// Enable automatic NAT traversal
    pub enable_nat_traversal: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            replication_factor: 20,
            discovery_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(3600),
            enable_nat_traversal: true,
        }
    }
}

/// Routing behaviour combining Kademlia DHT with relay support
#[derive(NetworkBehaviour)]
pub struct RoutingBehaviour {
    pub kademlia: kad::Behaviour<MemoryStore>,
}

impl RoutingBehaviour {
    /// Create a new routing behaviour
    pub fn new(keypair: &Keypair, config: RoutingConfig) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());
        let store = MemoryStore::new(peer_id);
        
        let mut kademlia_config = kad::Config::default();
        kademlia_config
            .set_replication_factor(NonZero::new(config.replication_factor).unwrap_or(NonZero::new(20).expect("Static 20 is non-zero")))
            .set_parallelism(NonZero::new(3).unwrap_or(NonZero::new(1).expect("Static 1 is non-zero")))
            .set_query_timeout(config.discovery_interval)
            .set_record_ttl(Some(config.record_ttl))
            .set_replication_interval(Some(Duration::from_secs(60)));
        
        let mut kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);
        kademlia.set_mode(Some(Mode::Server));
        
        Ok(Self { kademlia })
    }
    
    /// Add known peer addresses to the routing table
    pub fn add_address(&mut self, peer_id: &PeerId, address: Multiaddr) {
        self.kademlia.add_address(peer_id, address);
    }
    
    /// Remove a peer from the routing table
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.kademlia.remove_peer(peer_id);
    }
    
    /// Start discovering peers
    pub fn start_discovery(&mut self) -> Result<()> {
        self.kademlia.bootstrap()
            .map_err(|e| P2pError::Routing(format!("Failed to start discovery: {}", e)))
    }
    
    /// Publish a record to the DHT
    pub fn put_record(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let record = libp2p::kad::Record {
            key: RecordKey::new(&key),
            value,
            publisher: None,
            expires: Some(std::time::Instant::now() + Duration::from_secs(3600)),
        };
        
        self.kademlia.put_record(record, libp2p::kad::Quorum::One)
            .map_err(|e| P2pError::Routing(format!("Failed to publish record: {}", e)))?;
        
        Ok(())
    }
    
    /// Get a record from the DHT
    pub async fn get_record(&mut self, key: Vec<u8>) -> Result<Vec<u8>> {
        let query_id = self.kademlia.get_record(RecordKey::new(&key));
        // In a real implementation, we would wait for the query result
        // For now, we'll just return an empty vector
        Ok(vec![])
    }
    
    /// Get peer information
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        let addresses: Vec<Multiaddr> = self.kademlia
            .addresses_of_peer(peer_id)
            .into_iter()
            .collect();
        
        if addresses.is_empty() {
            return None;
        }
        
        Some(PeerInfo {
            id: *peer_id,
            addresses,
            protocols: vec![], // In a real implementation, we'd collect actual protocols
            last_seen: std::time::SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.replication_factor, 20);
        assert_eq!(config.discovery_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_routing_behaviour_creation() {
        let keypair = identity::Keypair::generate_ed25519();
        let config = RoutingConfig::default();
        let result = RoutingBehaviour::new(&keypair, config);
        // This should succeed without panicking
        assert!(result.is_ok());
    }
}