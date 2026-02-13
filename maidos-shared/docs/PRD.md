# Product Requirements Document - maidos-shared

## 1. Overview

maidos-shared is a Rust workspace library providing foundational cross-product capabilities for the MAIDOS ecosystem. It consists of 9 specialized crates covering authentication, event bus, blockchain, configuration, Google APIs, LLM integration (11 providers), logging, P2P networking, and social media connectors. The library is designed for consumption by Rust, C, C++, C#, and other languages via FFI bindings.

## 2. Problem Statement

MAIDOS products (Driver, IME, CodeQC, Forge, Office, PDF) require common infrastructure: authentication, configuration management, LLM integration, event messaging, and external service connectors. Duplicating these capabilities across products leads to inconsistent behavior, security vulnerabilities, increased maintenance burden, and wasted engineering effort. A unified shared library reduces duplication, ensures security best practices, and accelerates product development.

## 3. Target Users

- **MAIDOS Product Teams**: Engineers building Driver, IME, CodeQC, Forge, Office, PDF
- **Third-Party Integrators**: External developers consuming MAIDOS APIs
- **DevOps Engineers**: Operators managing configuration and monitoring
- **Security Teams**: Auditors verifying authentication and token handling

## 4. Product Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| PG-1 | Eliminate duplicate auth/config/LLM code across products | >= 90% code reuse in new MAIDOS products |
| PG-2 | Provide unified LLM abstraction for 11+ providers | Single API supports OpenAI, Claude, Gemini, DeepSeek, Ollama, etc. |
| PG-3 | Deliver sub-millisecond token validation | Token ops complete in < 1ms at p99 |
| PG-4 | Enable secure cross-product messaging | Event bus achieves 99.9% uptime |
| PG-5 | Support cross-language FFI (C#, C, C++) | All crates expose C ABI with bindings |

## 5. Core Features

### 5.1 Authentication (maidos-auth)
- **Capability-Based Access Control**: 18 predefined capabilities (read_user, write_driver, manage_config, etc.)
- **HMAC-SHA256 Token Issuance**: Time-limited tokens with constant-time verification
- **Policy Engine**: Evaluate capability requirements against token grants
- **Token Store**: In-memory store with expiration tracking
- **FFI Exposure**: C ABI for token creation, verification, and revocation

**Acceptance Criteria**: AC-001 (Token issuance), AC-002 (Token verification), AC-003 (Revocation), AC-004 (Expiration)

### 5.2 Event Bus (maidos-bus)
- **ZeroMQ Pub/Sub**: Topic-based message routing across processes
- **MessagePack Serialization**: Compact binary format with schema evolution
- **Async I/O**: Tokio-based non-blocking publisher/subscriber
- **Topic Filtering**: Subscribers receive only messages matching topic prefixes
- **Reconnection Logic**: Automatic reconnect with exponential backoff

**Acceptance Criteria**: AC-005 (Publish), AC-006 (Subscribe), AC-007 (Topic filtering), AC-008 (Reconnection)

### 5.3 LLM Integration (maidos-llm)
- **11 Provider Support**: OpenAI, Anthropic, Google, DeepSeek, Groq, Mistral, Azure OpenAI, Cohere, Together AI, Replicate, Ollama, LM Studio, vLLM
- **Unified API**: Single `complete()` method across all providers
- **Streaming Interface**: Unified streaming with `MaidosStreamItem` (v0.2.0)
- **Vision Support**: Multi-modal inputs for GPT-4o, Claude, Gemini, etc.
- **Function Calling**: MaidosTool format with cross-provider translation
- **Router with 6 Strategies**: Priority, RoundRobin, Weighted, Cost, Speed, Fallback
- **Budget Control**: Daily/Monthly/Per-Request limits with tracking

**Acceptance Criteria**: AC-009 (Provider creation), AC-010 (Completion), AC-011 (Streaming), AC-012 (Router), AC-013 (Budget)

### 5.4 Configuration (maidos-config)
- **TOML Parsing**: Load structured config from .toml files
- **Environment Variable Expansion**: `${VAR}`, `${VAR:-default}` syntax
- **Schema Validation**: Type-checked config with required/optional fields
- **Hot Reload**: File watcher triggers config refresh on change
- **Thread Safety**: Arc + RwLock for concurrent access

**Acceptance Criteria**: AC-014 (Load TOML), AC-015 (Env expansion), AC-016 (Hot reload)

### 5.5 Logging (maidos-log)
- **Structured Logging**: JSON-formatted logs with tracing integration
- **Multiple Sinks**: File, stdout, stderr, syslog
- **Log Rotation**: Size-based and time-based rotation
- **Filtering**: Per-module log level configuration

**Acceptance Criteria**: AC-017 (Log output), AC-018 (Log rotation)

### 5.6 Social Connectors (maidos-social)
- **Twitter/X API**: Post tweets, read timeline, handle OAuth
- **Discord Bot**: Send messages, handle webhooks
- **Telegram Bot**: Send messages, receive updates

**Acceptance Criteria**: AC-019 (Post to Twitter), AC-020 (Send Discord message)

### 5.7 Google APIs (maidos-google)
- **Google Drive**: Upload, download, list files
- **Google Sheets**: Read/write spreadsheet data
- **Google Calendar**: Create/read events

**Acceptance Criteria**: AC-021 (Upload to Drive), AC-022 (Read from Sheets)

### 5.8 P2P Networking (maidos-p2p)
- **libp2p Integration**: Peer discovery, DHT, transport protocols
- **NAT Traversal**: Hole punching, relay support
- **Secure Connections**: Noise protocol encryption

**Acceptance Criteria**: AC-023 (Peer discovery), AC-024 (Send message)

### 5.9 Blockchain (maidos-chain)
- **Ethereum Integration**: Wallet management, contract interaction
- **RPC Client**: Query blockchain state, send transactions
- **Event Listening**: Monitor contract events

**Acceptance Criteria**: AC-025 (Connect wallet), AC-026 (Call contract)

## 6. Architecture Summary

maidos-shared is a Cargo workspace with 9 member crates. Common dependencies (serde, tokio, tracing) are declared at workspace level. Each crate exposes a public Rust API and optionally a C FFI layer (`extern "C"` functions). C# bindings (`bindings/csharp/`) provide P/Invoke wrappers. The compiled output is a set of `.rlib` files for Rust consumers and `maidos_shared.dll` (Windows) or `libmaidos_shared.so` (Linux) for FFI consumers.

**Dependency Order**: All crates depend on `maidos-config` and `maidos-log`. Higher-level crates (auth, bus, llm) have no inter-dependencies.

## 7. Constraints

- **Rust Version**: Minimum 1.75 (2021 edition)
- **Performance**: Token operations < 1ms, LLM streaming overhead < 100ms, bus message latency < 10ms
- **Memory**: Total footprint < 50 MB for all crates loaded
- **Security**: HMAC-SHA256 for tokens, constant-time comparison, no plaintext secrets in logs
- **Compatibility**: Windows 10+, Linux (kernel 4.4+), macOS 10.15+

## 8. Release Information

- **Current Version**: v0.2.0
- **License**: MIT
- **Repository**: https://github.com/maidos/maidos-shared
- **Crates.io**: Each crate published independently
- **Signing Identity**: MAIDOS Project

## 9. Dependencies

| Crate | External Dependencies |
|-------|-----------------------|
| maidos-config | toml, serde, notify, directories |
| maidos-auth | jsonwebtoken, hmac, sha2, ring |
| maidos-bus | zmq, rmp-serde, tokio |
| maidos-llm | reqwest, serde_json, tokio |
| maidos-log | tracing, tracing-subscriber |
| maidos-social | reqwest, serde |
| maidos-google | reqwest, serde, oauth2 |
| maidos-p2p | libp2p, tokio |
| maidos-chain | ethers, web3 |

## 10. Out of Scope

- **GUI Components**: UI is responsibility of consuming applications (Driver, IME, etc.)
- **Database ORM**: Persistence beyond in-memory token store
- **Advanced AI Features**: RAG, fine-tuning, model training (use LLM providers directly)
- **Cloud Deployment**: Deployment automation (handled by product teams)
- **Multi-Tenancy**: Tenant isolation (handled at application layer)
