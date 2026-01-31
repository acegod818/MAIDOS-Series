use crate::ChainError;
use rand::Rng;
use std::fmt;

/// Simple wallet for blockchain operations
#[derive(Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    /// Create a new wallet with random key pair
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let private_key: [u8; 32] = rng.gen();
        let public_key = Self::derive_public_key(&private_key);
        
        Self {
            private_key: hex::encode(private_key),
            public_key: hex::encode(public_key),
        }
    }

    /// Create wallet from private key
    pub fn from_private_key(private_key: &str) -> Result<Self, ChainError> {
        let key_bytes = hex::decode(private_key)
            .map_err(|e| ChainError::Wallet(format!("Invalid private key: {}", e)))?;
        
        if key_bytes.len() != 32 {
            return Err(ChainError::Wallet("Private key must be 32 bytes".to_string()));
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        
        let public_key = Self::derive_public_key(&key_array);
        
        Ok(Self {
            private_key: private_key.to_string(),
            public_key: hex::encode(public_key),
        })
    }

    /// Derive public key from private key (simple XOR for demo)
    fn derive_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let mut public_key = [0u8; 32];
        for (i, &byte) in private_key.iter().enumerate() {
            public_key[i] = byte ^ 0xFF; // Simple transformation for demo
        }
        public_key
    }

    /// Get wallet address (simple hash of public key)
    pub fn address(&self) -> String {
        let public_key_bytes = hex::decode(&self.public_key).unwrap_or_default();
        let hash = Self::simple_hash(&public_key_bytes);
        format!("0x{}", hex::encode(&hash[..20])) // Ethereum-like address
    }

    /// Simple hash function for demo purposes
    fn simple_hash(data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Sign a message (demo implementation)
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let private_key_bytes = hex::decode(&self.private_key).unwrap_or_default();
        let mut signature = Vec::with_capacity(message.len() + private_key_bytes.len());
        signature.extend_from_slice(message);
        signature.extend_from_slice(&private_key_bytes);
        signature
    }

    /// Verify a signature (demo implementation)
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != message.len() + 32 {
            return false;
        }
        
        let expected = self.sign(message);
        signature == expected
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Wallet(address: {})", self.address())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.private_key.is_empty());
        assert!(!wallet.public_key.is_empty());
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_wallet_from_private_key() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::from_private_key(&wallet1.private_key).unwrap();
        assert_eq!(wallet1.private_key, wallet2.private_key);
        assert_eq!(wallet1.public_key, wallet2.public_key);
    }

    #[test]
    fn test_signature() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign(message);
        assert!(wallet.verify(message, &signature));
    }

    #[test]
    fn test_signature_invalid() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign(message);
        let mut bad = signature.clone();
        bad.push(0);
        assert!(!wallet.verify(message, &bad));
    }

    #[test]
    fn test_wallet_display() {
        let wallet = Wallet::new();
        let text = format!("{}", wallet);
        assert!(text.contains("Wallet(address: 0x"));
    }

    #[test]
    fn test_wallet_invalid_private_key_length() {
        let err = Wallet::from_private_key("abc").unwrap_err();
        assert!(matches!(err, ChainError::Wallet(_)));
    }
}
