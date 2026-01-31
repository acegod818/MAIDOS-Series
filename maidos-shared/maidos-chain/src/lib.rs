//! MAIDOS Handcrafted Blockchain Implementation
//!
//! Simple blockchain operations without complex dependencies.
//! Features: Multi-chain support, transaction handling, wallet management.

mod error;
mod eth;
mod sol;
mod wallet;
mod transaction;
pub mod mnc;
pub mod payment;

pub use error::{ChainError, Result};
pub use wallet::Wallet;
pub use transaction::Transaction;
pub use mnc::{MncContract, EthMncContract, BnbMncContract, SolMncContract};
pub use payment::{PaymentProcessor, EthPaymentProcessor, BnbPaymentProcessor, SolPaymentProcessor};

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainType {
    Ethereum,
    Solana,
    BNB,
}

impl ChainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainType::Ethereum => "ethereum",
            ChainType::Solana => "solana",
            ChainType::BNB => "bnb",
        }
    }
}

/// Blockchain client configuration
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_type: ChainType,
    pub rpc_url: String,
    pub timeout_secs: u64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_type: ChainType::Ethereum,
            rpc_url: "http://localhost:8545".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Main blockchain client
pub struct ChainClient {
    config: ChainConfig,
}

impl ChainClient {
    pub fn new(config: ChainConfig) -> Self {
        Self { config }
    }

    /// Get current chain type
    pub fn chain_type(&self) -> ChainType {
        self.config.chain_type
    }

    /// Simple chain health check
    pub async fn health_check(&self) -> Result<bool> {
        match self.config.chain_type {
            ChainType::Ethereum => eth::health_check(&self.config.rpc_url).await,
            ChainType::Solana => sol::health_check(&self.config.rpc_url).await,
            ChainType::BNB => eth::health_check(&self.config.rpc_url).await, // BNB uses Ethereum-like RPC
        }
    }

    /// Send a transaction
    pub async fn send_transaction(&self, tx: &Transaction) -> Result<String> {
        match self.config.chain_type {
            ChainType::Ethereum => eth::send_transaction(&self.config.rpc_url, tx).await,
            ChainType::Solana => sol::send_transaction(&self.config.rpc_url, tx).await,
            ChainType::BNB => eth::send_transaction(&self.config.rpc_url, tx).await,
        }
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<String> {
        match self.config.chain_type {
            ChainType::Ethereum => eth::get_balance(&self.config.rpc_url, address).await,
            ChainType::Solana => sol::get_balance(&self.config.rpc_url, address).await,
            ChainType::BNB => eth::get_balance(&self.config.rpc_url, address).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn spawn_rpc_server(body: &str) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = body.to_string();

        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        (format!("http://{}", addr), handle)
    }

    #[test]
    fn test_chain_type_conversion() {
        assert_eq!(ChainType::Ethereum.as_str(), "ethereum");
        assert_eq!(ChainType::Solana.as_str(), "solana");
    }

    #[test]
    fn test_chain_client_creation() {
        let config = ChainConfig::default();
        let client = ChainClient::new(config);
        assert_eq!(client.chain_type(), ChainType::Ethereum);
    }

    #[tokio::test]
    async fn test_chain_client_health_check_false_on_invalid_rpc() {
        let config = ChainConfig {
            chain_type: ChainType::Ethereum,
            rpc_url: "http://127.0.0.1:0".to_string(),
            timeout_secs: 1,
        };
        let client = ChainClient::new(config);
        let ok = client.health_check().await.unwrap();
        assert!(!ok);
    }

    #[tokio::test]
    async fn test_chain_client_get_balance_sol() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":{"value":"123"}}"#);
        let config = ChainConfig {
            chain_type: ChainType::Solana,
            rpc_url: url,
            timeout_secs: 1,
        };
        let client = ChainClient::new(config);
        let balance = client.get_balance("addr").await.unwrap();
        assert_eq!(balance, "123");
    }

    #[tokio::test]
    async fn test_chain_client_send_transaction_sol() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":"sig"}"#);
        let config = ChainConfig {
            chain_type: ChainType::Solana,
            rpc_url: url,
            timeout_secs: 1,
        };
        let client = ChainClient::new(config);
        let tx = Transaction::new("from", "to", "1");
        let sig = client.send_transaction(&tx).await.unwrap();
        assert_eq!(sig, "sig");
    }
}
