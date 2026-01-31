//! Token generation and verification
//!
//! <impl>
//! WHAT: HMAC-SHA256 signed capability tokens with TTL
//! WHY: Secure, stateless authentication for capability-based access control
//! HOW: Token = base64(json_payload) + "." + base64(hmac_signature)
//! TEST: Generation, verification, expiration, tampering detection
//! </impl>

use crate::capability::CapabilitySet;
use crate::error::{AuthError, Result};
use crate::Capability;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Get current Unix timestamp in seconds.
/// 
/// # Panics
/// Panics if system time is before UNIX_EPOCH (1970-01-01), which should never
/// happen on any reasonable system.
#[inline]
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX epoch")
        .as_secs()
}

/// Token payload (signed content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    /// Granted capabilities (as bitmask)
    pub caps: u32,
    /// Token issue time (Unix timestamp)
    pub iat: u64,
    /// Token expiration time (Unix timestamp)
    pub exp: u64,
    /// Optional subject identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
}

/// A capability token
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    payload: TokenPayload,
    raw: String,
}

impl CapabilityToken {
    /// Create a new token with the given capabilities and TTL
    pub fn new(capabilities: CapabilitySet, ttl: Duration, secret: &[u8]) -> Result<Self> {
        Self::new_with_subject(capabilities, ttl, secret, None)
    }

    /// Create a new token with subject identifier
    pub fn new_with_subject(
        capabilities: CapabilitySet,
        ttl: Duration,
        secret: &[u8],
        subject: Option<String>,
    ) -> Result<Self> {
        let now = current_timestamp();

        let payload = TokenPayload {
            caps: capabilities.as_u32(),
            iat: now,
            exp: now + ttl.as_secs(),
            sub: subject,
        };

        let raw = Self::encode_token(&payload, secret)?;
        Ok(Self { payload, raw })
    }

    /// Parse and verify a token string
    pub fn verify(token_str: &str, secret: &[u8]) -> Result<Self> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 2 {
            return Err(AuthError::MalformedToken(
                "Expected format: payload.signature".to_string(),
            ));
        }

        let payload_b64 = parts[0];
        let signature_b64 = parts[1];

        // Decode and verify signature first
        let expected_sig = Self::compute_signature(payload_b64.as_bytes(), secret)?;
        let actual_sig = base64_decode(signature_b64)?;

        if !constant_time_eq(&expected_sig, &actual_sig) {
            return Err(AuthError::InvalidSignature);
        }

        // Decode payload
        let payload_bytes = base64_decode(payload_b64)?;
        let payload: TokenPayload = serde_json::from_slice(&payload_bytes)
            .map_err(|e| AuthError::MalformedToken(format!("Invalid payload: {}", e)))?;

        // Check expiration
        let now = current_timestamp();

        if now >= payload.exp {
            return Err(AuthError::TokenExpired);
        }

        Ok(Self {
            payload,
            raw: token_str.to_string(),
        })
    }

    /// Check if token has a specific capability
    pub fn has(&self, cap: Capability) -> bool {
        let caps = CapabilitySet::from_u32(self.payload.caps);
        caps.has(cap)
    }

    /// Check if token has all specified capabilities
    pub fn has_all(&self, caps: &[Capability]) -> bool {
        let set = CapabilitySet::from_u32(self.payload.caps);
        set.has_all(caps)
    }

    /// Get remaining TTL
    pub fn remaining_ttl(&self) -> Duration {
        let now = current_timestamp();

        if now >= self.payload.exp {
            Duration::ZERO
        } else {
            Duration::from_secs(self.payload.exp - now)
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now > self.payload.exp
    }

    /// Get the raw token string
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get granted capabilities
    pub fn capabilities(&self) -> CapabilitySet {
        CapabilitySet::from_u32(self.payload.caps)
    }

    /// Get subject identifier
    pub fn subject(&self) -> Option<&str> {
        self.payload.sub.as_deref()
    }

    fn encode_token(payload: &TokenPayload, secret: &[u8]) -> Result<String> {
        let payload_json = serde_json::to_vec(payload)
            .map_err(|e| AuthError::SerializationError(e.to_string()))?;

        let payload_b64 = base64_encode(&payload_json);
        let signature = Self::compute_signature(payload_b64.as_bytes(), secret)?;
        let signature_b64 = base64_encode(&signature);

        Ok(format!("{}.{}", payload_b64, signature_b64))
    }

    fn compute_signature(data: &[u8], secret: &[u8]) -> Result<Vec<u8>> {
        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|e| AuthError::SerializationError(format!("HMAC error: {}", e)))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// URL-safe base64 encoding (no padding)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::with_capacity((data.len() * 4).div_ceil(3));
    
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        }
    }
    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    fn decode_char(c: u8) -> Result<u8> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'-' => Ok(62),
            b'_' => Ok(63),
            _ => Err(AuthError::MalformedToken("Invalid base64".to_string())),
        }
    }

    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity((bytes.len() * 3) / 4);
    let mut i = 0;

    while i < bytes.len() {
        let remaining = bytes.len() - i;
        if remaining < 2 {
            return Err(AuthError::MalformedToken("Truncated base64".to_string()));
        }

        let v0 = decode_char(bytes[i])?;
        let v1 = decode_char(bytes[i + 1])?;
        result.push((v0 << 2) | (v1 >> 4));

        if remaining > 2 {
            let v2 = decode_char(bytes[i + 2])?;
            result.push((v1 << 4) | (v2 >> 2));
            
            if remaining > 3 {
                let v3 = decode_char(bytes[i + 3])?;
                result.push((v2 << 6) | v3);
            }
        }
        i += 4;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-secret-key-32-bytes-long!!";

    #[test]
    fn test_token_create_and_verify() {
        let caps: CapabilitySet = vec![Capability::LlmChat, Capability::FileRead]
            .into_iter()
            .collect();

        let token = CapabilityToken::new(caps, Duration::from_secs(3600), TEST_SECRET).unwrap();
        let verified = CapabilityToken::verify(token.as_str(), TEST_SECRET).unwrap();

        assert!(verified.has(Capability::LlmChat));
        assert!(verified.has(Capability::FileRead));
        assert!(!verified.has(Capability::ShellExec));
    }

    #[test]
    fn test_token_expiration() {
        let caps = CapabilitySet::empty();
        // Create token that expires in the past by manipulating payload directly
        let secret = TEST_SECRET;
        
        // Create with 1 second TTL, then sleep
        let token = CapabilityToken::new(caps, Duration::from_secs(1), secret).unwrap();
        std::thread::sleep(Duration::from_millis(1100));

        let result = CapabilityToken::verify(token.as_str(), secret);
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_token_invalid_signature() {
        let caps = CapabilitySet::empty();
        let token = CapabilityToken::new(caps, Duration::from_secs(3600), TEST_SECRET).unwrap();

        let wrong_secret = b"wrong-secret-key-32-bytes-long!";
        let result = CapabilityToken::verify(token.as_str(), wrong_secret);
        assert!(matches!(result, Err(AuthError::InvalidSignature)));
    }

    #[test]
    fn test_token_tampering() {
        let caps: CapabilitySet = vec![Capability::LlmChat].into_iter().collect();
        let token = CapabilityToken::new(caps, Duration::from_secs(3600), TEST_SECRET).unwrap();

        // Tamper with payload
        let parts: Vec<&str> = token.as_str().split('.').collect();
        let tampered = format!("{}x.{}", parts[0], parts[1]);

        let result = CapabilityToken::verify(&tampered, TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_subject() {
        let caps = CapabilitySet::empty();
        let token = CapabilityToken::new_with_subject(
            caps,
            Duration::from_secs(3600),
            TEST_SECRET,
            Some("user-123".to_string()),
        )
        .unwrap();

        let verified = CapabilityToken::verify(token.as_str(), TEST_SECRET).unwrap();
        assert_eq!(verified.subject(), Some("user-123"));
    }

    #[test]
    fn test_remaining_ttl() {
        let caps = CapabilitySet::empty();
        let token = CapabilityToken::new(caps, Duration::from_secs(3600), TEST_SECRET).unwrap();

        let remaining = token.remaining_ttl();
        assert!(remaining.as_secs() > 3590);
        assert!(remaining.as_secs() <= 3600);
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World! 123";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
