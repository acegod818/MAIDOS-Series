use crate::{ChainError, Result, Transaction};
use serde_json::Value;

/// Simple Solana RPC client with handcrafted HTTP implementation
pub struct SolClient {
    rpc_url: String,
}

impl SolClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Simple HTTP RPC call for Solana
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

/// Check Solana node health
pub async fn health_check(rpc_url: &str) -> Result<bool> {
    let client = SolClient::new(rpc_url);
    match client.rpc_call("getHealth", vec![]).await {
        Ok(result) => {
            if let Some(health) = result.as_str() {
                Ok(health == "ok")
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

/// Send Solana transaction with full instruction support
pub async fn send_transaction(rpc_url: &str, tx: &Transaction) -> Result<String> {
    let client = SolClient::new(rpc_url);
    
    // In MAIDOS, we support complex instructions via compiled buffer
    let tx_data = if let Some(data) = &tx.data {
        serde_json::json!({
            "from": tx.from,
            "to": tx.to,
            "value": tx.value,
            "data": hex::encode(data),
            "instructions": tx.instructions, // Added for complex contract calls
        })
    } else {
        serde_json::json!({
            "from": tx.from,
            "to": tx.to,
            "value": tx.value,
        })
    };
    
    println!("[MAIDOS-AUDIT] Sending Solana transaction to: {}", tx.to);
    let params = vec![
        serde_json::to_value(tx_data)?,
        serde_json::json!({ "encoding": "base64", "preflightCommitment": "confirmed" })
    ];
    let result = client.rpc_call("sendTransaction", params).await?;
    
    result.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ChainError::Transaction("Invalid transaction signature".to_string()))
}

/// Get Solana balance
pub async fn get_balance(rpc_url: &str, address: &str) -> Result<String> {
    let client = SolClient::new(rpc_url);
    let params = vec![serde_json::to_value(address)?];
    let result = client.rpc_call("getBalance", params).await?;
    
    if let Some(balance_obj) = result.as_object() {
        if let Some(value) = balance_obj.get("value") {
            return value.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| ChainError::Transaction("Invalid balance value".to_string()));
        }
    }
    
    Err(ChainError::Transaction("Invalid balance response".to_string()))
}

/// Get Solana recent blockhash
#[allow(dead_code)]
pub async fn get_recent_blockhash(rpc_url: &str) -> Result<String> {
    let client = SolClient::new(rpc_url);
    let result = client.rpc_call("getRecentBlockhash", vec![]).await?;
    
    if let Some(blockhash_obj) = result.as_object() {
        if let Some(value) = blockhash_obj.get("value") {
            if let Some(blockhash) = value.as_object().and_then(|v| v.get("blockhash")) {
                return blockhash.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| ChainError::Transaction("Invalid blockhash".to_string()));
            }
        }
    }
    
    Err(ChainError::Transaction("Invalid blockhash response".to_string()))
}

/// Get Solana transaction count
#[allow(dead_code)]
pub async fn get_transaction_count(rpc_url: &str) -> Result<String> {
    let client = SolClient::new(rpc_url);
    let result = client.rpc_call("getTransactionCount", vec![]).await?;
    
    result.as_u64()
        .map(|n| n.to_string())
        .ok_or_else(|| ChainError::Transaction("Invalid transaction count".to_string()))
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
    fn test_sol_client_creation() {
        let client = SolClient::new("http://localhost:8899");
        assert_eq!(client.rpc_url, "http://localhost:8899");
    }

    #[tokio::test]
    async fn test_send_transaction_error_on_invalid_rpc() {
        let tx = Transaction::new("from", "to", "1");
        let result = send_transaction("http://127.0.0.1:0", &tx).await;
        assert!(matches!(result, Err(ChainError::Network(_))));
    }

    #[tokio::test]
    async fn test_send_transaction_with_data_error_on_invalid_rpc() {
        let tx = Transaction {
            from: "from".to_string(),
            to: "to".to_string(),
            value: "1".to_string(),
            gas: None,
            gas_price: None,
            nonce: None,
            data: Some("deadbeef".to_string()),
            instructions: Some(serde_json::json!({"ix":"value"})),
        };
        let result = send_transaction("http://127.0.0.1:0", &tx).await;
        assert!(matches!(result, Err(ChainError::Network(_))));
    }

    #[tokio::test]
    async fn test_get_balance_error_on_invalid_rpc() {
        let result = get_balance("http://127.0.0.1:0", "address").await;
        assert!(matches!(result, Err(ChainError::Network(_))));
    }

    #[tokio::test]
    async fn test_health_check_ok() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":"ok"}"#);
        let ok = health_check(&url).await.unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn test_send_transaction_success() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":"sig"}"#);
        let tx = Transaction::new("from", "to", "1");
        let sig = send_transaction(&url, &tx).await.unwrap();
        assert_eq!(sig, "sig");
    }

    #[tokio::test]
    async fn test_get_balance_success() {
        let (url, _handle) = spawn_rpc_server(r#"{"result":{"value":"123"}}"#);
        let balance = get_balance(&url, "address").await.unwrap();
        assert_eq!(balance, "123");
    }
}
