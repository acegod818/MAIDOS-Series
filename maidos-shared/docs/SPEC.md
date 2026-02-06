# maidos-shared — Technical Specification

## 1. Workspace Layout

```
maidos-shared/
  Cargo.toml          # Workspace root, feature re-exports
  maidos-auth/        # Authentication module
  maidos-bus/         # Message bus module
  maidos-chain/       # Blockchain audit module
  maidos-config/      # Configuration management
  maidos-google/      # Google API integration
  maidos-llm/         # LLM client (Ollama)
  maidos-log/         # Structured logging
  maidos-p2p/         # Peer-to-peer networking
  maidos-social/      # Social media integration
```

## 2. Sub-Crate Specifications

### 2.1 maidos-auth
- **Purpose:** OAuth2 and token-based authentication.
- **Key types:** `AuthClient`, `AuthConfig`, `AuthToken`, `AuthError`.
- **Dependencies:** `reqwest`, `serde`, `keyring` (credential storage).
- **Async:** All network methods are `async fn`.

### 2.2 maidos-bus
- **Purpose:** Typed publish/subscribe message bus.
- **Key types:** `MessageBus`, `Subscription`, `Topic`, `BusError`.
- **Modes:** In-process (tokio broadcast) and networked (TCP/TLS).
- **Guarantees:** Ordered delivery per topic; at-most-once semantics.

### 2.3 maidos-chain
- **Purpose:** Append-only hash chain for audit logs.
- **Key types:** `AuditChain`, `AuditEntry`, `ChainError`.
- **Hashing:** SHA-256 for entry linking.

### 2.4 maidos-config
- **Purpose:** Layered configuration with file, env, and runtime overrides.
- **Key types:** `Config`, `ConfigBuilder`, `ConfigError`.
- **Format:** TOML primary, JSON fallback.
- **Hot reload:** Filesystem watcher triggers `on_change` callback.

### 2.5 maidos-google
- **Purpose:** Google Workspace and Cloud API wrappers.
- **Key types:** `GoogleClient`, `ServiceAccount`, `GoogleError`.
- **APIs:** Drive, Sheets, Calendar (extensible).

### 2.6 maidos-llm
- **Purpose:** Local LLM inference via Ollama HTTP API.
- **Key types:** `OllamaClient`, `GenerateRequest`, `GenerateResponse`, `LlmError`.
- **Streaming:** Supports chunked streaming via `Stream<Item = String>`.
- **Privacy:** All inference is local; no data leaves the machine.

### 2.7 maidos-log
- **Purpose:** Unified structured logging for all MAIDOS products.
- **Key types:** `LogConfig`, `LogLevel`.
- **Backend:** `tracing` + `tracing-subscriber`.
- **Outputs:** stdout, file, both. JSON or human-readable format.

### 2.8 maidos-p2p
- **Purpose:** Peer discovery and message exchange on local or wide-area networks.
- **Key types:** `PeerNode`, `PeerConfig`, `PeerEvent`, `P2pError`.
- **Transport:** QUIC (via `quinn`) with TLS 1.3.
- **Discovery:** mDNS for LAN; configurable bootstrap nodes for WAN.

### 2.9 maidos-social
- **Purpose:** Post and read from social media platforms.
- **Key types:** `SocialClient`, `Post`, `SocialError`.
- **Platforms:** Twitter/X, Discord webhook (extensible).
- **Rate limiting:** Built-in retry with exponential backoff.

## 3. Error Handling Strategy

All sub-crates define a crate-level error enum implementing `std::error::Error` and
`thiserror::Error`. Errors are propagated via `Result<T, SubCrateError>` and can be
converted into a unified `maidos_shared::Error` at the workspace level.

## 4. Feature Flags

The root `maidos-shared` crate re-exports each sub-crate behind a Cargo feature flag:
`auth`, `bus`, `chain`, `config`, `google`, `llm`, `log`, `p2p`, `social`. Consumers
enable only what they need.
