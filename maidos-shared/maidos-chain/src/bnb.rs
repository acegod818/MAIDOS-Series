//! Binance Smart Chain implementation

use crate::{Blockchain, ChainInfo, ChainType, ChainError, Result, MncContract, PaymentProcessor, Transaction};
use async_trait::async_trait;
use ethers::{
    prelude::*,
    providers::{Http, Provider},
};
use std::sync::Arc;

/// Binance Smart Chain implementation
pub struct BnbChain {
    provider: Arc<Provider<Http>>,
    mnc_contract: BnbMncContract,
    payment_processor: BnbPaymentProcessor,
    chain_id: u64,
}

/// BNB MNC contract implementation
pub struct BnbMncContract {
    // In a real implementation, this would contain contract addresses and ABI
}

/// BNB payment processor implementation
pub struct BnbPaymentProcessor {
    provider: Arc<Provider<Http>>,
}

impl BnbChain {
    /// Create a new BNB chain instance
    pub fn new(rpc_endpoint: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_endpoint)
            .map_err(|e| ChainError::Network(format!("Failed to connect to BNB node: {}", e)))?;
        let provider = Arc::new(provider);
        
        let chain_id = provider.get_chainid()
            .await
            .map_err(|e| ChainError::EthProvider(e))?
            .as_u64();
        
        Ok(Self {
            provider: provider.clone(),
            mnc_contract: BnbMncContract {},
            payment_processor: BnbPaymentProcessor { provider },
            chain_id,
        })
    }
}

#[async_trait]
impl Blockchain for BnbChain {
    fn chain_info(&self) -> ChainInfo {
        ChainInfo {
            chain_type: ChainType::BNB,
            network: match self.chain_id {
                56 => "Binance Smart Chain Mainnet".to_string(),
                97 => "Binance Smart Chain Testnet".to_string(),
                _ => format!("BNB Chain #{}", self.chain_id),
            },
            rpc_endpoint: self.provider.url().to_string(),
            chain_id: Some(self.chain_id),
            native_currency: "BNB".to_string(),
        }
    }

    async fn get_balance(&self, address: &str) -> Result<U256> {
        let address: Address = address.parse()
            .map_err(|_| ChainError::InvalidAddress(address.to_string()))?;
        
        let balance = self.provider.get_balance(address, None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;
        
        Ok(balance)
    }

    async fn send_transaction(&self, tx: &Transaction) -> Result<H256> {
        // Send the transaction
        let tx_request = ethers::types::TransactionRequest::new()
            .to(tx.to.parse().map_err(|_| ChainError::InvalidAddress(tx.to.clone()))?)
            .value(tx.amount)
            .data(tx.data.clone())
            .gas(tx.gas_limit)
            .gas_price(tx.gas_price);
        
        let pending_tx = self.provider.send_transaction(tx_request, None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;
        
        Ok(pending_tx.tx_hash())
    }

    async fn get_transaction_receipt(&self, tx_hash: &H256) -> Result<Option<ethers::types::TransactionReceipt>> {
        let receipt = self.provider.get_transaction_receipt(*tx_hash)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;
        
        Ok(receipt)
    }

    fn mnc(&self) -> &dyn MncContract {
        &self.mnc_contract
    }

    fn payment(&self) -> &dyn PaymentProcessor {
        &self.payment_processor
    }
}

#[async_trait]
impl MncContract for BnbMncContract {
    async fn get_locked(&self, _address: &str) -> Result<U256> {
        // In a real implementation, this would call the MNC contract
        // For now, we'll just return zero
        Ok(U256::zero())
    }

    async fn lock(&self, _wallet: &str, _amount: U256) -> Result<H256> {
        // In a real implementation, this would call the MNC contract
        // For now, we'll just return a dummy hash
        Ok(H256::random())
    }

    async fn unlock(&self, _wallet: &str, _amount: U256) -> Result<H256> {
        // In a real implementation, this would call the MNC contract
        // For now, we'll just return a dummy hash
        Ok(H256::random())
    }
}

#[async_trait]
impl PaymentProcessor for BnbPaymentProcessor {
    async fn create_payment(&self, _from: &str, _to: &str, _amount: U256) -> Result<Transaction> {
        // In a real implementation, this would create a payment transaction
        // For now, we'll just return a dummy transaction
        Ok(Transaction {
            from: "0x0000000000000000000000000000000000000000".to_string(),
            to: "0x0000000000000000000000000000000000000000".to_string(),
            amount: U256::zero(),
            data: vec![],
            gas_limit: U256::from(21000),
            gas_price: U256::from(5_000_000_000u64), // 5 Gwei
        })
    }

    async fn estimate_gas(&self, _tx: &Transaction) -> Result<U256> {
        // In a real implementation, this would estimate gas
        // For now, we'll just return a default value
        Ok(U256::from(21000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bnb_chain_creation() {
        // Just test that the code compiles
        // Actual instantiation requires network access
    }

    #[test]
    fn test_bnb_chain_info_mapping() {
        let provider = Arc::new(Provider::<Http>::try_from("http://127.0.0.1:0").unwrap());
        let chain = BnbChain {
            provider: provider.clone(),
            mnc_contract: BnbMncContract {},
            payment_processor: BnbPaymentProcessor { provider },
            chain_id: 97,
        };

        let info = chain.chain_info();
        assert_eq!(info.chain_type, ChainType::BNB);
        assert!(info.network.contains("Testnet"));
    }

    #[tokio::test]
    async fn test_bnb_mnc_contract_dummy() {
        let contract = BnbMncContract {};
        let locked = contract.get_locked("0x123").await.unwrap();
        assert_eq!(locked, U256::zero());
    }

    #[tokio::test]
    async fn test_bnb_payment_processor_defaults() {
        let provider = Arc::new(Provider::<Http>::try_from("http://127.0.0.1:0").unwrap());
        let processor = BnbPaymentProcessor { provider };
        let tx = processor.create_payment("a", "b", U256::from(1u64)).await.unwrap();
        assert_eq!(tx.gas_limit, U256::from(21000));
    }
}
