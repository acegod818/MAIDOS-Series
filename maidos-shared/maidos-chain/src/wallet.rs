use crate::ChainError;
use ethers::signers::{LocalWallet, Signer};
use ethers::core::types::Signature;
use std::fmt;
use std::str::FromStr;

/// Ethereum-compatible wallet using secp256k1 ECDSA
///
/// Wraps ethers::LocalWallet for real cryptographic operations:
/// - Key generation: CSPRNG via k256
/// - Address derivation: Keccak256 of uncompressed public key
/// - Signing: secp256k1 ECDSA with recovery ID (EIP-155 compatible)
/// - Verification: ECDSA signature verification against public key
#[derive(Clone)]
pub struct Wallet {
    inner: LocalWallet,
}

impl Wallet {
    /// Create a new wallet with a cryptographically random key pair
    pub fn new() -> Self {
        Self {
            inner: LocalWallet::new(&mut rand::thread_rng()),
        }
    }

    /// Create wallet from a hex-encoded private key
    pub fn from_private_key(private_key: &str) -> Result<Self, ChainError> {
        let clean = private_key.strip_prefix("0x").unwrap_or(private_key);
        let inner = LocalWallet::from_str(clean)
            .map_err(|e| ChainError::Wallet(format!("Invalid private key: {}", e)))?;
        Ok(Self { inner })
    }

    /// Get the Ethereum address (0x-prefixed, checksummed)
    pub fn address(&self) -> String {
        format!("{:?}", self.inner.address())
    }

    /// Get the hex-encoded private key
    pub fn private_key(&self) -> String {
        hex::encode(self.inner.signer().to_bytes())
    }

    /// Get the hex-encoded public key (uncompressed, 65 bytes)
    pub fn public_key(&self) -> String {
        let verifying_key = self.inner.signer().verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        hex::encode(encoded.as_bytes())
    }

    /// Sign a message using EIP-191 personal_sign (secp256k1 ECDSA)
    ///
    /// Returns a 65-byte signature (r: 32 bytes, s: 32 bytes, v: 1 byte)
    pub async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ChainError> {
        let signature = self.inner.sign_message(message).await
            .map_err(|e| ChainError::Wallet(format!("Signing failed: {}", e)))?;
        Ok(signature.to_vec())
    }

    /// Sign a message synchronously (blocking)
    pub fn sign_sync(&self, message: &[u8]) -> Result<Vec<u8>, ChainError> {
        // ethers LocalWallet sign_hash is CPU-bound (no I/O)
        let hash = ethers::core::utils::hash_message(message);
        let signature: Signature = self.inner.sign_hash(hash)
            .map_err(|e| ChainError::Wallet(format!("Signing failed: {}", e)))?;
        Ok(signature.to_vec())
    }

    /// Verify a signature against this wallet's public key
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8]) -> bool {
        if signature_bytes.len() != 65 {
            return false;
        }

        let signature = match Signature::try_from(signature_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        signature.verify(message, self.inner.address()).is_ok()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address())
            .finish()
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
        assert!(wallet.address().starts_with("0x"));
        assert_eq!(wallet.address().len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_wallet_from_private_key() {
        let wallet1 = Wallet::new();
        let pk = wallet1.private_key();
        let wallet2 = Wallet::from_private_key(&pk).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_signature_sync() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign_sync(message).unwrap();
        assert_eq!(signature.len(), 65); // r(32) + s(32) + v(1)
        assert!(wallet.verify(message, &signature));
    }

    #[test]
    fn test_signature_invalid() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign_sync(message).unwrap();
        let mut bad = signature.clone();
        bad[0] ^= 0xFF; // Corrupt first byte
        assert!(!wallet.verify(message, &bad));
    }

    #[test]
    fn test_cross_wallet_verify_fails() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let message = b"test message";
        let signature = wallet1.sign_sync(message).unwrap();
        assert!(!wallet2.verify(message, &signature));
    }

    #[test]
    fn test_wallet_display() {
        let wallet = Wallet::new();
        let text = format!("{}", wallet);
        assert!(text.contains("Wallet(address: 0x"));
    }

    #[test]
    fn test_wallet_invalid_private_key() {
        let err = Wallet::from_private_key("not_hex").unwrap_err();
        assert!(matches!(err, ChainError::Wallet(_)));
    }

    #[test]
    fn test_wallet_with_0x_prefix() {
        let wallet1 = Wallet::new();
        let pk = format!("0x{}", wallet1.private_key());
        let wallet2 = Wallet::from_private_key(&pk).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }
}
