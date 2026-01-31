//! MAIDOS Capability-based Authentication System
//!
//! Provides capability tokens for fine-grained access control:
//! - HMAC-SHA256 signed tokens
//! - Time-based expiration
//! - Bitmask-based capability checking
//!
//! # Example
//!
//! ```
//! use maidos_auth::{Capability, CapabilitySet, CapabilityToken};
//! use std::time::Duration;
//!
//! // Create capabilities
//! let mut caps = CapabilitySet::empty();
//! caps.grant(Capability::LlmChat);
//! caps.grant(Capability::FileRead);
//!
//! // Generate token
//! let secret = b"your-32-byte-secret-key-here!!!";
//! let token = CapabilityToken::new(caps, Duration::from_secs(3600), secret).unwrap();
//!
//! // Verify and check capabilities
//! let verified = CapabilityToken::verify(token.as_str(), secret).unwrap();
//! assert!(verified.has(Capability::LlmChat));
//! assert!(!verified.has(Capability::ShellExec));
//! ```

mod capability;
mod error;
mod ffi;
mod policy;
mod store;
mod token;

pub use capability::{Capability, CapabilitySet};
pub use error::{AuthError, Result};
pub use policy::{
    Condition, ConditionOp, ConditionValue, PolicyContext, PolicyDecision, PolicyEngine, PolicyRule,
};
pub use store::{StoreConfig, StoreStats, StoredToken, TokenStore};
pub use token::{CapabilityToken, TokenPayload};

use maidos_config::MaidosConfig;
use std::time::Duration;
use tracing::info;

/// Token issuer that integrates with MaidosConfig
pub struct TokenIssuer {
    secret: Vec<u8>,
    default_ttl: Duration,
}

impl TokenIssuer {
    /// Create a new token issuer from configuration
    pub fn from_config(config: &MaidosConfig) -> Result<Self> {
        let auth = config.auth();

        let secret = auth
            .secret_key
            .as_ref()
            .ok_or(AuthError::NoSecretKey)?
            .as_bytes()
            .to_vec();

        Ok(Self {
            secret,
            default_ttl: Duration::from_secs(auth.token_ttl),
        })
    }

    /// Create with explicit secret and TTL
    pub fn new(secret: Vec<u8>, default_ttl: Duration) -> Self {
        Self { secret, default_ttl }
    }

    /// Issue a token with the given capabilities
    pub fn issue(&self, capabilities: CapabilitySet) -> Result<CapabilityToken> {
        info!("[MAIDOS-AUDIT] Issuing token with capabilities: {:?}", capabilities);
        CapabilityToken::new(capabilities, self.default_ttl, &self.secret)
    }

    /// Issue a token with custom TTL
    pub fn issue_with_ttl(&self, capabilities: CapabilitySet, ttl: Duration) -> Result<CapabilityToken> {
        info!("[MAIDOS-AUDIT] Issuing token with custom TTL: {:?}, capabilities: {:?}", ttl, capabilities);
        CapabilityToken::new(capabilities, ttl, &self.secret)
    }

    /// Issue a token with subject identifier
    pub fn issue_for_subject(&self, capabilities: CapabilitySet, subject: &str) -> Result<CapabilityToken> {
        info!("[MAIDOS-AUDIT] Issuing token for subject: {}, capabilities: {:?}", subject, capabilities);
        CapabilityToken::new_with_subject(
            capabilities,
            self.default_ttl,
            &self.secret,
            Some(subject.to_string()),
        )
    }

    /// Verify a token
    pub fn verify(&self, token_str: &str) -> Result<CapabilityToken> {
        let result = CapabilityToken::verify(token_str, &self.secret);
        match result {
            Ok(ref token) => {
                info!("[MAIDOS-AUDIT] Token verified successfully for subject: {:?}", token.subject());
            }
            Err(ref e) => {
                info!("[MAIDOS-AUDIT] Token verification failed: {}", e);
            }
        }
        result
    }

    /// Check if a token has a specific capability
    pub fn check(&self, token_str: &str, cap: Capability) -> bool {
        let allowed = self.verify(token_str)
            .map(|t| t.has(cap))
            .unwrap_or(false);
        
        if allowed {
            info!("[MAIDOS-AUDIT] Capability check PASSED: {:?}", cap);
        } else {
            info!("[MAIDOS-AUDIT] Capability check FAILED: {:?}", cap);
        }
        allowed
    }

    /// Check if a token has all required capabilities
    pub fn check_all(&self, token_str: &str, caps: &[Capability]) -> bool {
        let allowed = self.verify(token_str)
            .map(|t| t.has_all(caps))
            .unwrap_or(false);

        if allowed {
            info!("[MAIDOS-AUDIT] Multi-capability check PASSED: {:?}", caps);
        } else {
            info!("[MAIDOS-AUDIT] Multi-capability check FAILED: {:?}", caps);
        }
        allowed
    }
}

/// Guard that ensures a capability is present
pub struct CapabilityGuard<'a> {
    token: &'a CapabilityToken,
    required: Capability,
}

impl<'a> CapabilityGuard<'a> {
    /// Create a guard that checks for a capability
    pub fn require(token: &'a CapabilityToken, cap: Capability) -> Result<Self> {
        if !token.has(cap) {
            info!("[MAIDOS-AUDIT] CapabilityGuard DENIED access to: {:?}", cap);
            return Err(AuthError::MissingCapability(cap));
        }
        info!("[MAIDOS-AUDIT] CapabilityGuard GRANTED access to: {:?}", cap);
        Ok(Self {
            token,
            required: cap,
        })
    }

    /// Get the underlying token
    pub fn token(&self) -> &CapabilityToken {
        self.token
    }

    /// Get the required capability
    pub fn capability(&self) -> Capability {
        self.required
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_issuer() {
        let issuer = TokenIssuer::new(
            b"test-secret-key-32-bytes-long!!".to_vec(),
            Duration::from_secs(3600),
        );

        let mut caps = CapabilitySet::empty();
        caps.grant(Capability::LlmChat);

        let token = issuer.issue(caps).unwrap();
        let verified = issuer.verify(token.as_str()).unwrap();

        assert!(verified.has(Capability::LlmChat));
    }

    #[test]
    fn test_capability_guard() {
        let secret = b"test-secret-key-32-bytes-long!!";
        let mut caps = CapabilitySet::empty();
        caps.grant(Capability::LlmChat);

        let token = CapabilityToken::new(caps, Duration::from_secs(3600), secret).unwrap();

        // Should succeed
        let guard = CapabilityGuard::require(&token, Capability::LlmChat);
        assert!(guard.is_ok());

        // Should fail
        let guard = CapabilityGuard::require(&token, Capability::ShellExec);
        assert!(matches!(guard, Err(AuthError::MissingCapability(_))));
    }

    #[test]
    fn test_issuer_check() {
        let issuer = TokenIssuer::new(
            b"test-secret-key-32-bytes-long!!".to_vec(),
            Duration::from_secs(3600),
        );

        let mut caps = CapabilitySet::empty();
        caps.grant(Capability::LlmChat);
        caps.grant(Capability::FileRead);

        let token = issuer.issue(caps).unwrap();

        assert!(issuer.check(token.as_str(), Capability::LlmChat));
        assert!(issuer.check(token.as_str(), Capability::FileRead));
        assert!(!issuer.check(token.as_str(), Capability::ShellExec));

        assert!(issuer.check_all(token.as_str(), &[Capability::LlmChat, Capability::FileRead]));
        assert!(!issuer.check_all(token.as_str(), &[Capability::LlmChat, Capability::ShellExec]));
    }
}