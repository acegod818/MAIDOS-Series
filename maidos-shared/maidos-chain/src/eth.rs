use crate::{ChainError, Result, Transaction};
use serde_json::Value;
use std::collections::HashMap;

/// Simple Ethereum RPC client with handcrafted HTTP implementation
pub struct EthClient {
    rpc_url: String,
}

impl EthClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Simple HTTP RPC call
    async fn rpc_call(&self, method: &str, params: Vec<Value>) -> Result<Value> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChainError::Network(format!("RPC call failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::Rpc(format!("HTTP error: {}", response.status())));
        }

        let result: Value = response
            .json()
            .await
            .map_err(|e| ChainError::Serialization(format!("Parse error: {}", e)))?;

        if let Some(error) = result.get("error") {
            return Err(ChainError::Rpc(format!("RPC error: {}", error)));
        }

        result.get("result")
            .cloned()
            .ok_or_else(|| ChainError::Rpc("No result in response".to_string()))
    }
}

/// Check Ethereum node health
pub async fn health_check(rpc_url: &str) -> Result<bool> {
    let client = EthClient::new(rpc_url);
    match client.rpc_call("eth_blockNumber", vec![]).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Send Ethereum transaction
pub async fn send_transaction(rpc_url: &str, tx: &Transaction) -> Result<String> {
    let client = EthClient::new(rpc_url);
    
    let mut tx_params = HashMap::new();
    tx_params.insert("from", tx.from.as_str());
    tx_params.insert("to", tx.to.as_str());
    tx_params.insert("value", tx.value.as_str());
    
    if let Some(ref gas) = tx.gas {
        tx_params.insert("gas", gas.as_str());
    }
    
    if let Some(ref gas_price) = tx.gas_price {
        tx_params.insert("gasPrice", gas_price.as_str());
    }
    
    if let Some(ref data) = tx.data {
        tx_params.insert("data", data.as_str());
    }
    
    let params = vec![serde_json::to_value(tx_params)?];
    let result = client.rpc_call("eth_sendTransaction", params).await?;
    
    result.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ChainError::Transaction("Invalid transaction hash".to_string()))
}

/// Get Ethereum balance
pub async fn get_balance(rpc_url: &str, address: &str) -> Result<String> {
    let client = EthClient::new(rpc_url);
    let params = vec![
        serde_json::to_value(address)?,
        serde_json::to_value("latest")?
    ];
    let result = client.rpc_call("eth_getBalance", params).await?;
    
    result.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ChainError::Transaction("Invalid balance".to_string()))
}

/// Get gas price
#[allow(dead_code)]
pub async fn get_gas_price(rpc_url: &str) -> Result<String> {
    let client = EthClient::new(rpc_url);
    let result = client.rpc_call("eth_gasPrice", vec![]).await?;
    
    result.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ChainError::Transaction("Invalid gas price".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transaction;
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
    fn test_rpc_payload_creation() {
        let client = EthClient::new("http://localhost:8545");
        // Just test that the struct can be created
        assert_eq!(client.rpc_url, "http://localhost:8545");
    }

    #[tokio::test]
    async fn test_send_transaction_error_on_invalid_rpc() {
        let tx = Transaction::new("0xfrom", "0xto", "0x1")
            .with_gas("0x5208")
            .with_gas_price("0x3b9aca00")
            .with_data("0x");
        let result = send_transaction("http://127.0.0.1:0", &tx).await;
        assert!(matches!(result, Err(ChainError::Network(_))));
    }

    #[tokio::test]
    async fn test_get_balance_error_on_invalid_rpc() {
        let result = get_balance("http://127.0.0.1:0", "0xabc").await;
        assert!(matches!(result, Err(ChainError::Network(_))));
    }

    #[tokio::test]
    async fn test_get_balance_success() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":"0x10"}"#);
        let result = get_balance(&url, "0xabc").await.unwrap();
        assert_eq!(result, "0x10");
    }

    #[tokio::test]
    async fn test_send_transaction_success() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":"0xabc"}"#);
        let tx = Transaction::new("0xfrom", "0xto", "0x1");
        let result = send_transaction(&url, &tx).await.unwrap();
        assert_eq!(result, "0xabc");
    }
}
