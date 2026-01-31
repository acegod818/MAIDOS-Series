//! Configuration schema definitions
//!
//! <impl>
//! WHAT: Strongly-typed structs for MAIDOS configuration
//! WHY: Type safety + serde validation at load time
//! HOW: Nested structs with serde derive, defaults via Default trait
//! TEST: Deserialization tests with valid/invalid TOML
//! </impl>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaidosConfigSchema {
    /// MAIDOS metadata
    #[serde(default)]
    pub maidos: MaidosSection,

    /// LLM provider settings
    #[serde(default)]
    pub llm: LlmSection,

    /// Event bus settings
    #[serde(default)]
    pub bus: BusSection,

    /// Authentication settings
    #[serde(default)]
    pub auth: AuthSection,
}

/// MAIDOS metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaidosSection {
    /// Config schema version
    #[serde(default = "default_version")]
    pub version: String,
}

impl Default for MaidosSection {
    fn default() -> Self {
        Self {
            version: default_version(),
        }
    }
}

fn default_version() -> String {
    "1.0".to_string()
}

/// LLM configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSection {
    /// Default provider name
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Daily budget in USD
    #[serde(default = "default_budget_daily")]
    pub budget_daily: f64,

    /// Monthly budget in USD
    #[serde(default = "default_budget_monthly")]
    pub budget_monthly: f64,

    /// Provider-specific configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for LlmSection {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            budget_daily: default_budget_daily(),
            budget_monthly: default_budget_monthly(),
            providers: HashMap::new(),
        }
    }
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_budget_daily() -> f64 {
    10.0
}

fn default_budget_monthly() -> f64 {
    100.0
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// API key (supports ${ENV_VAR} syntax)
    #[serde(default)]
    pub api_key: Option<String>,

    /// API endpoint URL
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Model name to use
    #[serde(default)]
    pub model: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Max retries on failure
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

/// Event bus configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusSection {
    /// ZeroMQ endpoint
    #[serde(default = "default_bus_endpoint")]
    pub endpoint: String,

    /// Message buffer size
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// Reconnect interval in milliseconds
    #[serde(default = "default_reconnect_ms")]
    pub reconnect_ms: u64,
}

impl Default for BusSection {
    fn default() -> Self {
        Self {
            endpoint: default_bus_endpoint(),
            buffer_size: default_buffer_size(),
            reconnect_ms: default_reconnect_ms(),
        }
    }
}

fn default_bus_endpoint() -> String {
    "tcp://127.0.0.1:5555".to_string()
}

fn default_buffer_size() -> usize {
    1000
}

fn default_reconnect_ms() -> u64 {
    1000
}

/// Authentication configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSection {
    /// Token time-to-live in seconds
    #[serde(default = "default_token_ttl")]
    pub token_ttl: u64,

    /// Secret key for HMAC signing (supports ${ENV_VAR} syntax)
    #[serde(default)]
    pub secret_key: Option<String>,
}

impl Default for AuthSection {
    fn default() -> Self {
        Self {
            token_ttl: default_token_ttl(),
            secret_key: None,
        }
    }
}

fn default_token_ttl() -> u64 {
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema() {
        let schema = MaidosConfigSchema::default();
        assert_eq!(schema.maidos.version, "1.0");
        assert_eq!(schema.llm.default_provider, "ollama");
        assert_eq!(schema.llm.budget_daily, 10.0);
        assert_eq!(schema.bus.endpoint, "tcp://127.0.0.1:5555");
        assert_eq!(schema.auth.token_ttl, 3600);
    }

    #[test]
    fn test_deserialize_minimal() {
        let toml_str = r#"
[maidos]
version = "1.0"
"#;
        let schema: MaidosConfigSchema = toml::from_str(toml_str).unwrap();
        assert_eq!(schema.maidos.version, "1.0");
        // Defaults should be applied
        assert_eq!(schema.llm.default_provider, "ollama");
    }

    #[test]
    fn test_deserialize_full() {
        let toml_str = r#"
[maidos]
version = "1.0"

[llm]
default_provider = "anthropic"
budget_daily = 50.0
budget_monthly = 500.0

[llm.providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-20250514"
timeout_secs = 60

[llm.providers.ollama]
endpoint = "http://localhost:11434"
model = "llama3.2"

[bus]
endpoint = "tcp://127.0.0.1:5556"
buffer_size = 2000

[auth]
token_ttl = 7200
"#;
        let schema: MaidosConfigSchema = toml::from_str(toml_str).unwrap();
        assert_eq!(schema.llm.default_provider, "anthropic");
        assert_eq!(schema.llm.budget_daily, 50.0);
        assert_eq!(schema.llm.providers.len(), 2);
        assert_eq!(
            schema.llm.providers.get("anthropic").unwrap().model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert_eq!(schema.bus.buffer_size, 2000);
        assert_eq!(schema.auth.token_ttl, 7200);
    }
}
