//! Token Storage
//!
//! <impl>
//! WHAT: In-memory token storage with expiration management
//! WHY: Enable token lookup, revocation, and lifecycle management
//! HOW: HashMap with TTL tracking, background cleanup
//! TEST: Unit tests for store/retrieve/revoke/expiration
//! </impl>

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::capability::CapabilitySet;
use crate::error::{AuthError, Result};

/// Stored token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    /// Token string (or hash for security)
    pub token_hash: String,
    /// Subject (owner)
    pub subject: Option<String>,
    /// Capabilities granted
    pub capabilities: CapabilitySet,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
    /// Expiration timestamp (Unix seconds)
    pub expires_at: u64,
    /// Is token revoked
    pub revoked: bool,
    /// Revocation reason
    pub revoke_reason: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl StoredToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        self.expires_at <= now
    }

    /// Check if token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let now = current_timestamp();
        self.expires_at.saturating_sub(now)
    }
}

/// Token store statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreStats {
    /// Total tokens stored
    pub total_tokens: usize,
    /// Active (valid) tokens
    pub active_tokens: usize,
    /// Expired tokens
    pub expired_tokens: usize,
    /// Revoked tokens
    pub revoked_tokens: usize,
    /// Total storage operations
    pub store_count: u64,
    /// Total lookup operations
    pub lookup_count: u64,
    /// Total revoke operations
    pub revoke_count: u64,
}

/// Token store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    /// Maximum tokens to store
    pub max_tokens: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Keep expired tokens for this many seconds (for audit)
    pub expired_retention_secs: u64,
    /// Enable auto cleanup
    pub auto_cleanup: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            max_tokens: 10000,
            cleanup_interval_secs: 300, // 5 minutes
            expired_retention_secs: 3600, // 1 hour
            auto_cleanup: true,
        }
    }
}

/// In-memory token store
pub struct TokenStore {
    tokens: Arc<RwLock<HashMap<String, StoredToken>>>,
    config: StoreConfig,
    stats: Arc<RwLock<StoreStats>>,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl TokenStore {
    /// Create a new token store
    pub fn new(config: StoreConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(StoreStats::default())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Store a token
    pub fn store(
        &self,
        token_hash: String,
        subject: Option<String>,
        capabilities: CapabilitySet,
        ttl_secs: u64,
    ) -> Result<()> {
        self.maybe_cleanup();

        let mut tokens = self.tokens.write();

        // Check capacity
        if tokens.len() >= self.config.max_tokens {
            return Err(AuthError::Internal("Token store at capacity".into()));
        }

        let now = current_timestamp();
        let stored = StoredToken {
            token_hash: token_hash.clone(),
            subject,
            capabilities,
            created_at: now,
            expires_at: now + ttl_secs,
            revoked: false,
            revoke_reason: None,
            metadata: HashMap::new(),
        };

        tokens.insert(token_hash, stored);

        // Update stats
        let mut stats = self.stats.write();
        stats.store_count += 1;
        stats.total_tokens = tokens.len();
        stats.active_tokens = tokens.values().filter(|t| t.is_valid()).count();

        Ok(())
    }

    /// Store a token with metadata
    pub fn store_with_metadata(
        &self,
        token_hash: String,
        subject: Option<String>,
        capabilities: CapabilitySet,
        ttl_secs: u64,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        self.maybe_cleanup();

        let mut tokens = self.tokens.write();

        if tokens.len() >= self.config.max_tokens {
            return Err(AuthError::Internal("Token store at capacity".into()));
        }

        let now = current_timestamp();
        let stored = StoredToken {
            token_hash: token_hash.clone(),
            subject,
            capabilities,
            created_at: now,
            expires_at: now + ttl_secs,
            revoked: false,
            revoke_reason: None,
            metadata,
        };

        tokens.insert(token_hash, stored);

        let mut stats = self.stats.write();
        stats.store_count += 1;
        stats.total_tokens = tokens.len();

        Ok(())
    }

    /// Get a token by hash
    pub fn get(&self, token_hash: &str) -> Option<StoredToken> {
        self.maybe_cleanup();

        let tokens = self.tokens.read();
        let mut stats = self.stats.write();
        stats.lookup_count += 1;

        tokens.get(token_hash).cloned()
    }

    /// Check if a token is valid
    pub fn is_valid(&self, token_hash: &str) -> bool {
        self.get(token_hash).map(|t| t.is_valid()).unwrap_or(false)
    }

    /// Revoke a token
    pub fn revoke(&self, token_hash: &str, reason: Option<String>) -> Result<()> {
        let mut tokens = self.tokens.write();

        if let Some(token) = tokens.get_mut(token_hash) {
            token.revoked = true;
            token.revoke_reason = reason;

            let mut stats = self.stats.write();
            stats.revoke_count += 1;
            stats.revoked_tokens = tokens.values().filter(|t| t.revoked).count();

            Ok(())
        } else {
            Err(AuthError::InvalidToken("Token not found".into()))
        }
    }

    /// Revoke all tokens for a subject
    pub fn revoke_by_subject(&self, subject: &str, reason: Option<String>) -> usize {
        let mut tokens = self.tokens.write();
        let mut count = 0;

        for token in tokens.values_mut() {
            if token.subject.as_deref() == Some(subject) && !token.revoked {
                token.revoked = true;
                token.revoke_reason = reason.clone();
                count += 1;
            }
        }

        if count > 0 {
            let mut stats = self.stats.write();
            stats.revoke_count += count as u64;
            stats.revoked_tokens = tokens.values().filter(|t| t.revoked).count();
        }

        count
    }

    /// List tokens for a subject
    pub fn list_by_subject(&self, subject: &str) -> Vec<StoredToken> {
        let tokens = self.tokens.read();
        tokens
            .values()
            .filter(|t| t.subject.as_deref() == Some(subject))
            .cloned()
            .collect()
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        let tokens = self.tokens.read();
        let mut stats = self.stats.read().clone();

        stats.total_tokens = tokens.len();
        stats.active_tokens = tokens.values().filter(|t| t.is_valid()).count();
        stats.expired_tokens = tokens.values().filter(|t| t.is_expired()).count();
        stats.revoked_tokens = tokens.values().filter(|t| t.revoked).count();

        stats
    }

    /// Force cleanup of expired tokens
    pub fn cleanup(&self) -> usize {
        let mut tokens = self.tokens.write();
        let now = current_timestamp();
        let retention = self.config.expired_retention_secs;

        let before = tokens.len();
        tokens.retain(|_, t| {
            // Keep if not expired, or expired within retention period
            !t.is_expired() || (now - t.expires_at) < retention
        });

        *self.last_cleanup.write() = Instant::now();
        before - tokens.len()
    }

    /// Clear all tokens
    pub fn clear(&self) {
        let mut tokens = self.tokens.write();
        tokens.clear();

        let mut stats = self.stats.write();
        *stats = StoreStats::default();
    }

    fn maybe_cleanup(&self) {
        if !self.config.auto_cleanup {
            return;
        }

        let should_cleanup = {
            let last = self.last_cleanup.read();
            last.elapsed().as_secs() >= self.config.cleanup_interval_secs
        };

        if should_cleanup {
            self.cleanup();
        }
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new(StoreConfig::default())
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;

    #[test]
    fn test_store_and_get() {
        let store = TokenStore::default();
        let caps = CapabilitySet::from_iter([Capability::LlmChat]);

        store
            .store("token123".into(), Some("user1".into()), caps, 3600)
            .unwrap();

        let token = store.get("token123").unwrap();
        assert_eq!(token.subject, Some("user1".into()));
        assert!(token.is_valid());
    }

    #[test]
    fn test_token_not_found() {
        let store = TokenStore::default();
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn test_revoke_token() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store
            .store("token123".into(), None, caps, 3600)
            .unwrap();

        assert!(store.is_valid("token123"));

        store.revoke("token123", Some("testing".into())).unwrap();

        let token = store.get("token123").unwrap();
        assert!(token.revoked);
        assert!(!token.is_valid());
        assert_eq!(token.revoke_reason, Some("testing".into()));
    }

    #[test]
    fn test_revoke_by_subject() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store
            .store("token1".into(), Some("user1".into()), caps.clone(), 3600)
            .unwrap();
        store
            .store("token2".into(), Some("user1".into()), caps.clone(), 3600)
            .unwrap();
        store
            .store("token3".into(), Some("user2".into()), caps, 3600)
            .unwrap();

        let count = store.revoke_by_subject("user1", None);
        assert_eq!(count, 2);

        assert!(!store.is_valid("token1"));
        assert!(!store.is_valid("token2"));
        assert!(store.is_valid("token3"));
    }

    #[test]
    fn test_list_by_subject() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store
            .store("token1".into(), Some("user1".into()), caps.clone(), 3600)
            .unwrap();
        store
            .store("token2".into(), Some("user1".into()), caps.clone(), 3600)
            .unwrap();
        store
            .store("token3".into(), Some("user2".into()), caps, 3600)
            .unwrap();

        let user1_tokens = store.list_by_subject("user1");
        assert_eq!(user1_tokens.len(), 2);
    }

    #[test]
    fn test_store_with_metadata() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();
        let mut metadata = HashMap::new();
        metadata.insert("client".into(), "mobile".into());
        metadata.insert("version".into(), "1.0".into());

        store
            .store_with_metadata("token123".into(), None, caps, 3600, metadata)
            .unwrap();

        let token = store.get("token123").unwrap();
        assert_eq!(token.metadata.get("client"), Some(&"mobile".into()));
    }

    #[test]
    fn test_stats() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store.store("token1".into(), None, caps.clone(), 3600).unwrap();
        store.store("token2".into(), None, caps.clone(), 3600).unwrap();
        store.revoke("token2", None).unwrap();

        let stats = store.stats();
        assert_eq!(stats.total_tokens, 2);
        assert_eq!(stats.active_tokens, 1);
        assert_eq!(stats.revoked_tokens, 1);
        assert_eq!(stats.store_count, 2);
        assert_eq!(stats.revoke_count, 1);
    }

    #[test]
    fn test_capacity_limit() {
        let config = StoreConfig {
            max_tokens: 2,
            ..Default::default()
        };
        let store = TokenStore::new(config);
        let caps = CapabilitySet::empty();

        store.store("token1".into(), None, caps.clone(), 3600).unwrap();
        store.store("token2".into(), None, caps.clone(), 3600).unwrap();

        let result = store.store("token3".into(), None, caps, 3600);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store.store("token1".into(), None, caps.clone(), 3600).unwrap();
        store.store("token2".into(), None, caps, 3600).unwrap();

        assert_eq!(store.stats().total_tokens, 2);

        store.clear();

        assert_eq!(store.stats().total_tokens, 0);
    }

    #[test]
    fn test_remaining_ttl() {
        let store = TokenStore::default();
        let caps = CapabilitySet::empty();

        store.store("token1".into(), None, caps, 3600).unwrap();

        let token = store.get("token1").unwrap();
        // Should be close to 3600 (allowing some margin for execution time)
        assert!(token.remaining_ttl() > 3590);
        assert!(token.remaining_ttl() <= 3600);
    }
}
