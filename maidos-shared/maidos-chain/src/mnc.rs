//! MNC token contract interface

use crate::{ChainError, Result};
use async_trait::async_trait;
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Locked amount information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedAmount {
    /// Total locked amount
    pub total: U256,

    /// Amount locked for voting
    pub voting: U256,

    /// Amount locked for staking
    pub staking: U256,

    /// Amount locked for other purposes
    pub other: U256,
}

/// MNC contract interface
#[async_trait]
pub trait MncContract: Send + Sync {
    /// Get locked amount for an address
    async fn get_locked(&self, address: &str) -> Result<U256>;

    /// Lock MNC tokens
    async fn lock(&self, wallet: &str, amount: U256) -> Result<H256>;

    /// Unlock MNC tokens
    async fn unlock(&self, wallet: &str, amount: U256) -> Result<H256>;

    /// Get detailed lock information
    async fn get_lock_info(&self, address: &str) -> Result<LockedAmount> {
        let total = self.get_locked(address).await?;
        Ok(LockedAmount {
            total,
            voting: U256::zero(),
            staking: U256::zero(),
            other: total,
        })
    }

    /// Check if address has sufficient locked tokens
    async fn has_minimum_locked(&self, address: &str, minimum: U256) -> Result<bool> {
        let locked = self.get_locked(address).await?;
        Ok(locked >= minimum)
    }
}

/// Ethereum MNC contract implementation
pub struct EthMncContract {
    #[allow(dead_code)]
    contract_address: Address,
}

impl EthMncContract {
    /// Create new Ethereum MNC contract interface
    pub fn new(contract_address: Address) -> Self {
        Self { contract_address }
    }
}

#[async_trait]
impl MncContract for EthMncContract {
    async fn get_locked(&self, address: &str) -> Result<U256> {
        info!("[MAIDOS-AUDIT] ETH MNC: Querying locked amount for {}", address);
        Err(ChainError::ProviderNotConfigured("ETH MNC get_locked requires a configured RPC provider".into()))
    }

    async fn lock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] ETH MNC: Locking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("ETH MNC lock requires a configured RPC provider".into()))
    }

    async fn unlock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] ETH MNC: Unlocking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("ETH MNC unlock requires a configured RPC provider".into()))
    }
}

/// BNB MNC contract implementation
pub struct BnbMncContract {
    #[allow(dead_code)]
    contract_address: Address,
}

impl BnbMncContract {
    /// Create new BNB MNC contract interface
    pub fn new(contract_address: Address) -> Self {
        Self { contract_address }
    }
}

#[async_trait]
impl MncContract for BnbMncContract {
    async fn get_locked(&self, address: &str) -> Result<U256> {
        info!("[MAIDOS-AUDIT] BNB MNC: Querying locked amount for {}", address);
        Err(ChainError::ProviderNotConfigured("BNB MNC get_locked requires a configured RPC provider".into()))
    }

    async fn lock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] BNB MNC: Locking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("BNB MNC lock requires a configured RPC provider".into()))
    }

    async fn unlock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] BNB MNC: Unlocking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("BNB MNC unlock requires a configured RPC provider".into()))
    }
}

/// Solana MNC contract implementation
pub struct SolMncContract {
    #[allow(dead_code)]
    program_id: String,
}

impl SolMncContract {
    /// Create new Solana MNC contract interface
    pub fn new(program_id: String) -> Self {
        Self { program_id }
    }
}

#[async_trait]
impl MncContract for SolMncContract {
    async fn get_locked(&self, address: &str) -> Result<U256> {
        info!("[MAIDOS-AUDIT] SOL MNC: Querying locked amount for {}", address);
        Err(ChainError::ProviderNotConfigured("SOL MNC get_locked requires a configured RPC provider".into()))
    }

    async fn lock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] SOL MNC: Locking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("SOL MNC lock requires a configured RPC provider".into()))
    }

    async fn unlock(&self, wallet: &str, amount: U256) -> Result<H256> {
        info!("[MAIDOS-AUDIT] SOL MNC: Unlocking {} tokens for {}", amount, wallet);
        Err(ChainError::ProviderNotConfigured("SOL MNC unlock requires a configured RPC provider".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locked_amount_struct() {
        let locked = LockedAmount {
            total: U256::from(1000u64),
            voting: U256::from(100u64),
            staking: U256::from(200u64),
            other: U256::from(700u64),
        };

        assert_eq!(locked.total, U256::from(1000u64));
        assert_eq!(locked.voting, U256::from(100u64));
    }

    #[tokio::test]
    async fn test_mnc_contract_provider_not_configured() {
        let contract = EthMncContract::new(Address::zero());
        let result = contract.get_locked("0x123").await;
        assert!(matches!(result, Err(ChainError::ProviderNotConfigured(_))));
    }

    #[tokio::test]
    async fn test_mnc_contract_defaults() {
        struct Dummy;

        #[async_trait]
        impl MncContract for Dummy {
            async fn get_locked(&self, _address: &str) -> Result<U256> {
                Ok(U256::from(42u64))
            }

            async fn lock(&self, _wallet: &str, _amount: U256) -> Result<H256> {
                Ok(H256::zero())
            }

            async fn unlock(&self, _wallet: &str, _amount: U256) -> Result<H256> {
                Ok(H256::zero())
            }
        }

        let contract = Dummy;
        let info = contract.get_lock_info("addr").await.unwrap();
        assert_eq!(info.total, U256::from(42u64));
        assert_eq!(info.other, U256::from(42u64));
        assert!(contract.has_minimum_locked("addr", U256::from(40u64)).await.unwrap());
        assert!(!contract.has_minimum_locked("addr", U256::from(100u64)).await.unwrap());
    }
}
