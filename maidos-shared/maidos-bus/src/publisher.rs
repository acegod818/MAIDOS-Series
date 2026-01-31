//! Publisher for sending events to subscribers
//!
//! <impl>
//! WHAT: TCP-based publisher that broadcasts events to connected subscribers
//! WHY: Decoupled inter-process communication without external dependencies
//! HOW: Async TCP server with broadcast channel for fan-out
//! TEST: Connection handling, event broadcast, graceful shutdown
//! </impl>

use crate::error::{BusError, Result};
use crate::event::{Event, FrameHeader};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Publisher configuration
#[derive(Debug, Clone)]
pub struct PublisherConfig {
    /// Address to bind to
    pub bind_addr: String,
    /// Broadcast channel capacity
    pub channel_capacity: usize,
    /// Maximum concurrent connections
    pub max_connections: usize,
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".to_string(),
            channel_capacity: 1024,
            max_connections: 100,
        }
    }
}

/// Publisher state
struct PublisherState {
    /// Active connections count
    connection_count: usize,
    /// Event counter
    events_published: u64,
}

/// Event publisher (server side)
pub struct Publisher {
    config: PublisherConfig,
    /// Broadcast sender for events
    broadcast_tx: broadcast::Sender<Arc<Vec<u8>>>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// State
    state: Arc<RwLock<PublisherState>>,
    /// Bound address
    bound_addr: Arc<RwLock<Option<SocketAddr>>>,
}

impl Publisher {
    /// Create a new publisher with config
    pub fn new(config: PublisherConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.channel_capacity);
        Self {
            config,
            broadcast_tx,
            shutdown_tx: None,
            state: Arc::new(RwLock::new(PublisherState {
                connection_count: 0,
                events_published: 0,
            })),
            bound_addr: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(PublisherConfig::default())
    }

    /// Start the publisher server
    pub async fn start(&mut self) -> Result<()> {
        if self.shutdown_tx.is_some() {
            return Err(BusError::AlreadyRunning);
        }

        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        let actual_addr = listener.local_addr()?;

        {
            let mut addr = self.bound_addr.write().await;
            *addr = Some(actual_addr);
        }

        info!("[MAIDOS-AUDIT] Publisher starting on {}", actual_addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let broadcast_tx = self.broadcast_tx.clone();
        let state = self.state.clone();
        let max_conn = self.config.max_connections;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                let conn_count = {
                                    let s = state.read().await;
                                    s.connection_count
                                };

                                if conn_count >= max_conn {
                                    warn!("[MAIDOS-AUDIT] Max connections reached, rejecting {}", addr);
                                    continue;
                                }

                                let rx = broadcast_tx.subscribe();
                                let state_clone = state.clone();

                                {
                                    let mut s = state_clone.write().await;
                                    s.connection_count += 1;
                                }

                                info!("[MAIDOS-AUDIT] New subscriber connection from {}", addr);

                                tokio::spawn(async move {
                                    if let Err(e) = handle_subscriber(stream, rx).await {
                                        debug!("Subscriber {} disconnected: {}", addr, e);
                                    }
                                    let mut s = state_clone.write().await;
                                    s.connection_count = s.connection_count.saturating_sub(1);
                                    info!("[MAIDOS-AUDIT] Subscriber disconnected: {}", addr);
                                });
                            }
                            Err(e) => {
                                error!("[MAIDOS-AUDIT] Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("[MAIDOS-AUDIT] Publisher shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: Event) -> Result<()> {
        info!("[MAIDOS-AUDIT] Publishing event: topic={}, source={}", event.topic, event.source);
        let bytes = event.to_bytes()?;

        // Frame: 4-byte length header + payload
        let header = FrameHeader::new(bytes.len() as u32);
        let mut frame = Vec::with_capacity(FrameHeader::SIZE + bytes.len());
        frame.extend_from_slice(&header.to_bytes());
        frame.extend_from_slice(&bytes);

        // Broadcast to all subscribers (ignore if no receivers)
        let _ = self.broadcast_tx.send(Arc::new(frame));

        {
            let mut s = self.state.write().await;
            s.events_published += 1;
        }

        Ok(())
    }

    /// Get the bound address (available after start)
    pub async fn bound_addr(&self) -> Option<SocketAddr> {
        *self.bound_addr.read().await
    }

    /// Get current connection count
    pub async fn connection_count(&self) -> usize {
        self.state.read().await.connection_count
    }

    /// Get total events published
    pub async fn events_published(&self) -> u64 {
        self.state.read().await.events_published
    }

    /// Stop the publisher
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
            info!("[MAIDOS-AUDIT] Publisher stopped manually");
        }
        Ok(())
    }
}

/// Handle a single subscriber connection
async fn handle_subscriber(
    mut stream: TcpStream,
    mut rx: broadcast::Receiver<Arc<Vec<u8>>>,
) -> Result<()> {
    loop {
        match rx.recv().await {
            Ok(frame) => {
                if let Err(e) = stream.write_all(&frame).await {
                    return Err(BusError::Io(e));
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("[MAIDOS-AUDIT] Subscriber lagged behind by {} messages", n);
            }
            Err(broadcast::error::RecvError::Closed) => {
                return Err(BusError::ChannelClosed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publisher_start_stop() {
        let mut pub_handle = Publisher::with_defaults();
        pub_handle.start().await.unwrap();

        let addr = pub_handle.bound_addr().await;
        assert!(addr.is_some());

        pub_handle.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_publisher_already_running() {
        let mut pub_handle = Publisher::with_defaults();
        pub_handle.start().await.unwrap();

        let result = pub_handle.start().await;
        assert!(matches!(result, Err(BusError::AlreadyRunning)));

        pub_handle.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_publish_event() {
        let mut pub_handle = Publisher::with_defaults();
        pub_handle.start().await.unwrap();

        let event = Event::new("test.topic", "test-src", vec![1, 2, 3]).unwrap();
        pub_handle.publish(event).await.unwrap();

        assert_eq!(pub_handle.events_published().await, 1);

        pub_handle.stop().await.unwrap();
    }
}