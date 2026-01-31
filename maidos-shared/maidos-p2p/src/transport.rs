use crate::{Message, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

/// Simple TCP transport for P2P communication
pub struct TcpTransport {
    listener: Option<TcpListener>,
}

impl TcpTransport {
    pub fn new() -> Self {
        Self { listener: None }
    }

    /// Start listening on a specific address
    pub async fn listen(&mut self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| crate::P2pError::Network(format!("Failed to bind to {}: {}", addr, e)))?;
        
        self.listener = Some(listener);
        info!("TCP transport listening on {}", addr);
        Ok(())
    }

    /// Accept incoming connections
    pub async fn accept(&self) -> Result<(TcpStream, std::net::SocketAddr)> {
        if let Some(ref listener) = self.listener {
            let (stream, addr) = listener.accept().await
                .map_err(|e| crate::P2pError::Network(format!("Accept failed: {}", e)))?;
            info!("Accepted connection from {}", addr);
            Ok((stream, addr))
        } else {
            Err(crate::P2pError::Network("Not listening".to_string()))
        }
    }

    /// Connect to a remote peer
    pub async fn connect(&self, addr: &str) -> Result<TcpStream> {
        let stream = TcpStream::connect(addr).await
            .map_err(|e| crate::P2pError::Network(format!("Connect to {} failed: {}", addr, e)))?;
        
        info!("Connected to {}", addr);
        Ok(stream)
    }

    /// Send a message over a TCP stream
    pub async fn send_message(stream: &mut TcpStream, message: &Message) -> Result<()> {
        let data = message.serialize()
            .map_err(|e| crate::P2pError::Serialization(e.to_string()))?;
        
        // Send message length first
        let len_bytes = (data.len() as u32).to_be_bytes();
        stream.write_all(&len_bytes).await
            .map_err(|e| crate::P2pError::Network(format!("Write length failed: {}", e)))?;
        
        // Send message data
        stream.write_all(&data).await
            .map_err(|e| crate::P2pError::Network(format!("Write data failed: {}", e)))?;
        
        stream.flush().await
            .map_err(|e| crate::P2pError::Network(format!("Flush failed: {}", e)))?;
        
        Ok(())
    }

    /// Receive a message from a TCP stream
    pub async fn receive_message(stream: &mut TcpStream) -> Result<Message> {
        // Read message length
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await
            .map_err(|e| crate::P2pError::Network(format!("Read length failed: {}", e)))?;
        
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // Read message data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await
            .map_err(|e| crate::P2pError::Network(format!("Read data failed: {}", e)))?;
        
        // Deserialize message
        Message::deserialize(&data)
            .map_err(|e| crate::P2pError::Serialization(e.to_string()))
    }

    /// Simple connection health check
    pub async fn check_health(&self, stream: &mut TcpStream) -> bool {
        match Self::send_message(stream, &Message::heartbeat("health_check")).await {
            Ok(_) => true,
            Err(e) => {
                warn!("Connection health check failed: {}", e);
                false
            }
        }
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageKind;

    #[tokio::test]
    async fn test_transport_send_receive() {
        let mut server = TcpTransport::new();
        server.listen("127.0.0.1:0").await.unwrap();
        let addr = server.listener.as_ref().unwrap().local_addr().unwrap();

        let msg = Message::new_text("a", "b", "hello");

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = server.accept().await.unwrap();
            TcpTransport::receive_message(&mut stream).await.unwrap()
        });

        let client = TcpTransport::new();
        let mut stream = client.connect(&addr.to_string()).await.unwrap();
        TcpTransport::send_message(&mut stream, &msg).await.unwrap();

        let received = server_task.await.unwrap();
        assert_eq!(received.payload, msg.payload);
        assert!(matches!(received.kind, MessageKind::Text));
    }

    #[tokio::test]
    async fn test_transport_accept_not_listening() {
        let transport = TcpTransport::new();
        let result = transport.accept().await;
        assert!(matches!(result, Err(crate::P2pError::Network(_))));
    }
}
