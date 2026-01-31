use crate::{Message, P2pConfig, Peer, Result, TcpTransport};
use crate::discovery::DiscoveryService;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// Main P2P node implementation
pub struct P2pNode {
    config: P2pConfig,
    node_id: String,
    transport: Arc<TcpTransport>,
    discovery: Arc<DiscoveryService>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    #[allow(dead_code)]
    message_queue: mpsc::Sender<Message>,
    #[allow(dead_code)]
    message_receiver: Option<mpsc::Receiver<Message>>,
    running: Arc<RwLock<bool>>,
    tasks: Vec<JoinHandle<()>>,
}

impl P2pNode {
    pub fn new(config: P2pConfig) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        let discovery_config = crate::discovery::DiscoveryConfig {
            bootstrap_nodes: config.bootstrap_nodes.clone(),
            discovery_interval: 60,
            peer_timeout: config.heartbeat_interval * 2,
        };

        Self {
            config,
            node_id: Self::generate_node_id(),
            transport: Arc::new(TcpTransport::new()),
            discovery: Arc::new(DiscoveryService::new(discovery_config)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_queue: tx,
            message_receiver: Some(rx),
            running: Arc::new(RwLock::new(false)),
            tasks: Vec::new(),
        }
    }

    fn generate_node_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("node-{:016x}", rng.gen::<u64>())
    }

    /// Start the P2P node
    pub async fn start(&mut self) -> Result<()> {
        *self.running.write().await = true;
        info!("Starting P2P node: {}", self.node_id);

        // Start listening
        let mut transport = TcpTransport::new();
        transport.listen(&self.config.listen_addr).await?;
        self.transport = Arc::new(transport);

        // Start background tasks
        self.start_background_tasks().await;

        info!("P2P node {} started successfully", self.node_id);
        Ok(())
    }

    /// Stop the P2P node
    pub async fn stop(&mut self) {
        *self.running.write().await = false;
        info!("Stopping P2P node: {}", self.node_id);

        // Cancel all background tasks
        for task in self.tasks.drain(..) {
            task.abort();
        }

        info!("P2P node {} stopped", self.node_id);
    }

    async fn start_background_tasks(&mut self) {
        let running = self.running.clone();
        let transport = self.transport.clone();
        let _node_id = self.node_id.clone();
        let _listen_addr = self.config.listen_addr.clone();

        // Task 1: Accept incoming connections
        let task1 = tokio::spawn(async move {
            while *running.read().await {
                if let Ok((stream, addr)) = transport.accept().await {
                    info!("Accepted connection from {}", addr);
                    // Handle connection in a separate task
                    let running_clone = running.clone();
                    let transport_clone = transport.clone();
                    tokio::spawn(async move {
                        Self::handle_connection(stream, addr, running_clone, transport_clone).await;
                    });
                }
            }
        });

        // Task 2: Peer discovery
        let task2 = {
            let discovery = self.discovery.clone();
            let discovery_interval = self.discovery.discovery_interval();
            let running = self.running.clone();
            tokio::spawn(async move {
                while *running.read().await {
                    if let Ok(peers) = discovery.discover_peers().await {
                        info!("Discovered {} peers", peers.len());
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(discovery_interval)).await;
                }
            })
        };

        self.tasks.push(task1);
        self.tasks.push(task2);
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        addr: SocketAddr,
        running: Arc<RwLock<bool>>,
        _transport: Arc<TcpTransport>,
    ) {
        while *running.read().await {
            match TcpTransport::receive_message(&mut stream).await {
                Ok(message) => {
                    info!("Received message from {}: {:?}", addr, message.kind);
                    // Handle message based on type
                    match message.kind {
                        crate::message::MessageKind::Heartbeat => {
                            info!("Heartbeat received from {}", addr);
                        }
                        _ => {
                            info!("Processing message from {}", addr);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to receive message from {}: {}", addr, e);
                    break;
                }
            }
        }
    }

    /// Send message to a peer
    pub async fn send_message(&self, peer_id: &str, message: Message) -> Result<()> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            match self.transport.connect(&peer.addr.to_string()).await {
                Ok(mut stream) => {
                    TcpTransport::send_message(&mut stream, &message).await?;
                    info!("Sent message to {}", peer_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to connect to {}: {}", peer_id, e);
                    Err(e)
                }
            }
        } else {
            Err(crate::P2pError::PeerConnection(format!("Peer {} not found", peer_id)))
        }
    }

    /// Add a peer manually
    pub async fn add_peer(&self, peer: Peer) {
        let peer_clone = peer.clone();
        let mut peers = self.peers.write().await;
        peers.insert(peer.id.clone(), peer_clone);
        info!("Added peer: {}", peer.id);
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Get current peers count
    pub async fn peers_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
}

impl Drop for P2pNode {
    fn drop(&mut self) {
        if let Ok(running) = self.running.try_read() {
            if *running {
                info!("Dropping running P2P node, manual shutdown recommended");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_add_peer_and_count() {
        let config = P2pConfig::default();
        let node = P2pNode::new(config);
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let peer = Peer::new("peer-1".to_string(), addr);
        node.add_peer(peer).await;
        let count = node.peers_count().await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_send_message_peer_not_found() {
        let config = P2pConfig::default();
        let node = P2pNode::new(config);
        let msg = Message::new_text("a", "b", "hello");
        let result = node.send_message("missing", msg).await;
        assert!(matches!(result, Err(crate::P2pError::PeerConnection(_))));
    }

    #[tokio::test]
    async fn test_node_start_stop() {
        let config = P2pConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            bootstrap_nodes: vec![],
            max_peers: 10,
            heartbeat_interval: 1,
        };
        let mut node = P2pNode::new(config);
        node.start().await.unwrap();
        node.stop().await;
    }
}
