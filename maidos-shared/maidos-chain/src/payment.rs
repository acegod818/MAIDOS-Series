//! Payment processing utilities

use crate::{ChainError, Result};
use async_trait::async_trait;
use ethers::types::{U256, H256};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Sender address
    pub from: String,

    /// Recipient address
    pub to: String,

    /// Amount to transfer
    pub amount: U256,

    /// Transaction data (optional)
    pub data: Vec<u8>,

    /// Gas limit
    pub gas_limit: U256,

    /// Gas price
    pub gas_price: U256,
}

/// Payment processor interface
#[async_trait]
pub trait PaymentProcessor: Send + Sync {
    /// Create a payment transaction
    async fn create_payment(&self, from: &str, to: &str, amount: U256) -> Result<Transaction>;

    /// Estimate gas for a transaction
    async fn estimate_gas(&self, tx: &Transaction) -> Result<U256>;

    /// Send a transaction
    async fn send_transaction(&self, tx: &Transaction) -> Result<H256> {
        info!("[MAIDOS-AUDIT] Sending transaction from {} to {}", tx.from, tx.to);
        Err(ChainError::ProviderNotConfigured("send_transaction requires a configured RPC provider".into()))
    }

    /// Get transaction status
    async fn get_transaction_status(&self, tx_hash: &H256) -> Result<Option<bool>> {
        info!("[MAIDOS-AUDIT] Querying transaction status for hash: {:?}", tx_hash);
        Err(ChainError::ProviderNotConfigured("get_transaction_status requires a configured RPC provider".into()))
    }
}

/// Ethereum payment processor
pub struct EthPaymentProcessor {
    // Provider would be stored here
}

impl EthPaymentProcessor {
    /// Create new Ethereum payment processor
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EthPaymentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProcessor for EthPaymentProcessor {
    async fn create_payment(&self, from: &str, to: &str, amount: U256) -> Result<Transaction> {
        info!("[MAIDOS-AUDIT] Creating ETH payment: {} -> {} amount: {}", from, to, amount);
        Ok(Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            data: vec![],
            gas_limit: U256::from(21000), // Standard ETH transfer gas limit
            gas_price: U256::from(20_000_000_000u64), // 20 Gwei default
        })
    }

    async fn estimate_gas(&self, tx: &Transaction) -> Result<U256> {
        info!("[MAIDOS-AUDIT] Estimating gas for ETH transaction to {}", tx.to);
        Err(ChainError::ProviderNotConfigured("ETH gas estimation requires a configured RPC provider".into()))
    }
}

/// BNB payment processor
pub struct BnbPaymentProcessor {
    // Provider would be stored here
}

impl BnbPaymentProcessor {
    /// Create new BNB payment processor
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BnbPaymentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProcessor for BnbPaymentProcessor {
    async fn create_payment(&self, from: &str, to: &str, amount: U256) -> Result<Transaction> {
        info!("[MAIDOS-AUDIT] Creating BNB payment: {} -> {} amount: {}", from, to, amount);
        Ok(Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            data: vec![],
            gas_limit: U256::from(21000), // Standard BNB transfer gas limit
            gas_price: U256::from(5_000_000_000u64), // 5 Gwei default
        })
    }

    async fn estimate_gas(&self, tx: &Transaction) -> Result<U256> {
        info!("[MAIDOS-AUDIT] Estimating gas for BNB transaction to {}", tx.to);
        Err(ChainError::ProviderNotConfigured("BNB gas estimation requires a configured RPC provider".into()))
    }
}

/// Solana payment processor
pub struct SolPaymentProcessor {
    // Client would be stored here
}

impl SolPaymentProcessor {
    /// Create new Solana payment processor
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SolPaymentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProcessor for SolPaymentProcessor {
    async fn create_payment(&self, from: &str, to: &str, amount: U256) -> Result<Transaction> {
        info!("[MAIDOS-AUDIT] Creating SOL payment: {} -> {} amount: {}", from, to, amount);
        Ok(Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            data: vec![],
            gas_limit: U256::from(0), // Solana doesn't use gas in the same way
            gas_price: U256::from(0),
        })
    }

    async fn estimate_gas(&self, tx: &Transaction) -> Result<U256> {
        info!("[MAIDOS-AUDIT] Estimating compute units for SOL transaction to {}", tx.to);
        Err(ChainError::ProviderNotConfigured("Solana compute estimation requires a configured RPC provider".into()))
    }
}

/// Payment utility functions
pub struct PaymentUtils;

impl PaymentUtils {
    /// Validate payment amount
    pub fn validate_amount(amount: &U256) -> Result<()> {
        if amount.is_zero() {
            return Err(ChainError::InvalidInput("Payment amount cannot be zero".to_string()));
        }

        if *amount > U256::from(u128::MAX) {
            return Err(ChainError::InvalidInput("Payment amount too large".to_string()));
        }

        Ok(())
    }

    /// Validate address format
    pub fn validate_address(address: &str) -> Result<()> {
        if address.is_empty() {
            return Err(ChainError::InvalidAddress("Address cannot be empty".to_string()));
        }

        if address.len() < 10 {
            return Err(ChainError::InvalidAddress("Address too short".to_string()));
        }

        Ok(())
    }

    /// Calculate total cost including gas
    pub fn calculate_total_cost(amount: &U256, gas_limit: &U256, gas_price: &U256) -> U256 {
        let gas_cost = gas_limit * gas_price;
        *amount + gas_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_struct() {
        let tx = Transaction {
            from: "0x1234...".to_string(),
            to: "0x5678...".to_string(),
            amount: U256::from(100u64),
            data: vec![1, 2, 3],
            gas_limit: U256::from(21000u64),
            gas_price: U256::from(20_000_000_000u64),
        };

        assert_eq!(tx.amount, U256::from(100u64));
        assert_eq!(tx.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_payment_utils() {
        let amount = U256::from(100u64);
        let gas_limit = U256::from(21000u64);
        let gas_price = U256::from(20_000_000_000u64);

        let total_cost = PaymentUtils::calculate_total_cost(&amount, &gas_limit, &gas_price);
        let expected_gas_cost = U256::from(21000u64 * 20_000_000_000u64);
        let expected_total = amount + expected_gas_cost;

        assert_eq!(total_cost, expected_total);
    }

    #[test]
    fn test_payment_utils_validation_errors() {
        let zero = U256::zero();
        let err = PaymentUtils::validate_amount(&zero).unwrap_err();
        assert!(matches!(err, ChainError::InvalidInput(_)));

        let err = PaymentUtils::validate_address("").unwrap_err();
        assert!(matches!(err, ChainError::InvalidAddress(_)));

        let err = PaymentUtils::validate_address("short").unwrap_err();
        assert!(matches!(err, ChainError::InvalidAddress(_)));
    }

    #[test]
    fn test_payment_utils_validation_ok() {
        let amount = U256::from(1u64);
        assert!(PaymentUtils::validate_amount(&amount).is_ok());
        assert!(PaymentUtils::validate_address("0x1234567890").is_ok());
    }

    #[tokio::test]
    async fn test_payment_processors_provider_not_configured() {
        let processor = EthPaymentProcessor::new();
        let tx = processor.create_payment("from", "to", U256::one()).await.unwrap();
        let result = processor.send_transaction(&tx).await;
        assert!(matches!(result, Err(ChainError::ProviderNotConfigured(_))));
    }

    #[tokio::test]
    async fn test_payment_processors_create_payment_defaults() {
        let eth = EthPaymentProcessor::new();
        let bnb = BnbPaymentProcessor::new();
        let sol = SolPaymentProcessor::new();

        let tx_eth = eth.create_payment("from", "to", U256::from(10u64)).await.unwrap();
        let tx_bnb = bnb.create_payment("from", "to", U256::from(10u64)).await.unwrap();
        let tx_sol = sol.create_payment("from", "to", U256::from(10u64)).await.unwrap();

        assert_eq!(tx_eth.gas_limit, U256::from(21000u64));
        assert_eq!(tx_bnb.gas_price, U256::from(5_000_000_000u64));
        assert_eq!(tx_sol.gas_limit, U256::zero());
    }
}
