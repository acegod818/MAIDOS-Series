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
    provider: Arc<Provider<Http>>,
    /// MNC token contract address (BEP-20)
    contract_address: Address,
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
            mnc_contract: BnbMncContract {
                provider: provider.clone(),
                contract_address: "0x0000000000000000000000000000000000000000"
                    .parse()
                    .expect("Invalid MNC contract address — configure via env MNC_CONTRACT_ADDRESS"),
            },
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
    async fn get_locked(&self, address: &str) -> Result<U256> {
        let addr: Address = address.parse()
            .map_err(|_| ChainError::InvalidAddress(address.to_string()))?;

        // Call balanceOf(address) on the MNC BEP-20 token contract
        // Function selector: 0x70a08231 = keccak256("balanceOf(address)")
        let mut calldata = vec![0x70, 0xa0, 0x82, 0x31];
        calldata.extend_from_slice(&ethers::abi::encode(&[ethers::abi::Token::Address(addr)]));

        let tx = ethers::types::TransactionRequest::new()
            .to(self.contract_address)
            .data(calldata);

        let result = self.provider.call(&tx.into(), None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        let balance = U256::from_big_endian(&result);
        Ok(balance)
    }

    async fn lock(&self, wallet: &str, amount: U256) -> Result<H256> {
        let _wallet_addr: Address = wallet.parse()
            .map_err(|_| ChainError::InvalidAddress(wallet.to_string()))?;

        // Call lock(uint256) on the MNC staking contract
        // Function selector: 0xdd467064 = keccak256("lock(uint256)")
        let mut calldata = vec![0xdd, 0x46, 0x70, 0x64];
        calldata.extend_from_slice(&ethers::abi::encode(&[ethers::abi::Token::Uint(amount)]));

        let tx = ethers::types::TransactionRequest::new()
            .to(self.contract_address)
            .data(calldata);

        let pending = self.provider.send_transaction(tx, None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        Ok(pending.tx_hash())
    }

    async fn unlock(&self, wallet: &str, amount: U256) -> Result<H256> {
        let _wallet_addr: Address = wallet.parse()
            .map_err(|_| ChainError::InvalidAddress(wallet.to_string()))?;

        // Call unlock(uint256) on the MNC staking contract
        // Function selector: 0x6198e339 = keccak256("unlock(uint256)")
        let mut calldata = vec![0x61, 0x98, 0xe3, 0x39];
        calldata.extend_from_slice(&ethers::abi::encode(&[ethers::abi::Token::Uint(amount)]));

        let tx = ethers::types::TransactionRequest::new()
            .to(self.contract_address)
            .data(calldata);

        let pending = self.provider.send_transaction(tx, None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        Ok(pending.tx_hash())
    }
}

#[async_trait]
impl PaymentProcessor for BnbPaymentProcessor {
    async fn create_payment(&self, from: &str, to: &str, amount: U256) -> Result<Transaction> {
        let from_addr: Address = from.parse()
            .map_err(|_| ChainError::InvalidAddress(from.to_string()))?;
        let to_addr: Address = to.parse()
            .map_err(|_| ChainError::InvalidAddress(to.to_string()))?;

        // Query current gas price from the network
        let gas_price = self.provider.get_gas_price()
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        // Build the transaction to estimate gas
        let tx_request = ethers::types::TransactionRequest::new()
            .from(from_addr)
            .to(to_addr)
            .value(amount);

        let gas_limit = self.provider.estimate_gas(&tx_request.clone().into(), None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        Ok(Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            data: vec![],
            gas_limit,
            gas_price,
        })
    }

    async fn estimate_gas(&self, tx: &Transaction) -> Result<U256> {
        let to_addr: Address = tx.to.parse()
            .map_err(|_| ChainError::InvalidAddress(tx.to.clone()))?;

        let tx_request = ethers::types::TransactionRequest::new()
            .to(to_addr)
            .value(tx.amount)
            .data(tx.data.clone());

        let gas = self.provider.estimate_gas(&tx_request.into(), None)
            .await
            .map_err(|e| ChainError::EthProvider(e))?;

        Ok(gas)
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
            mnc_contract: BnbMncContract {
                provider: provider.clone(),
                contract_address: "0x0000000000000000000000000000000000000000"
                    .parse()
                    .expect("Invalid MNC contract address — configure via env MNC_CONTRACT_ADDRESS"),
            },
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
