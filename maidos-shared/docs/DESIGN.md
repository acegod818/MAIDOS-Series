# maidos-shared -- Design Document

| Field     | Value              |
|-----------|--------------------|
| Product   | maidos-shared      |
| Version   | 0.2.0              |
| Type      | Design Decisions   |

## Design Principles

| Principle            | Description                                              |
|----------------------|----------------------------------------------------------|
| Modular Crates       | Each concern is a separate crate with minimal coupling   |
| Trait-Based APIs     | Public interfaces defined via traits for testability     |
| Async First          | All I/O operations use async/await (tokio runtime)       |
| Config Driven        | Behavior controlled by configuration, not hard-coding    |
| Zero Unsafe          | No unsafe code outside of FFI boundary modules           |
| Error Propagation    | Custom error types with thiserror, ? propagation         |

## Crate Design Details

### maidos-config
- Loads TOML configuration files
- Merges with environment variables
- Provides typed access via serde deserialization

### maidos-auth
- JWT token creation and validation with configurable claims
- OAuth2 authorization code flow with PKCE support
- Session management with refresh token rotation

### maidos-bus
- Async event dispatcher with topic-based routing
- Type-safe event payloads via generic subscribers
- Backpressure handling with bounded channels

### maidos-llm
- Unified interface for multiple LLM providers
- Prompt template system with variable substitution
- Streaming response support

### maidos-log
- Structured JSON logging via tracing
- Multiple output sinks (console, file, remote)
- Context-aware span tracking

### maidos-social
- Adapter pattern for social platform APIs
- Rate limiting per platform
- Webhook receiver for incoming events

### maidos-google
- Service account and OAuth2 authentication
- Drive file operations (CRUD, sharing)
- Sheets read/write with typed cell access

### maidos-p2p
- libp2p-based peer discovery and transport
- NAT traversal with hole punching
- Encrypted channels (Noise protocol)

### maidos-chain
- Smart contract interaction via ABI
- Wallet management (HD wallets)
- RPC provider abstraction

## Error Handling Strategy

Each crate defines its own error enum using thiserror:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

*maidos-shared DESIGN v0.2.0 -- CodeQC Gate C Compliant*
