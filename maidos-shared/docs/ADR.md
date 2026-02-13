# Architecture Decision Records - maidos-shared

## ADR-001: Cargo Workspace Structure

**Status**: Accepted
**Date**: 2025-08-10
**Deciders**: MAIDOS Architecture Team

### Context

The MAIDOS ecosystem requires shared functionality across 6 products (Driver, IME, CodeQC, Forge, Office, PDF). Duplication of authentication, configuration, and LLM integration logic leads to inconsistent behavior and security vulnerabilities. We need a maintainable structure for shared code.

### Decision

We will use a Cargo workspace with 9 member crates, each providing a focused capability. All crates share common dependencies declared at workspace level. Workspace structure:

```
maidos-shared/
  Cargo.toml              # Workspace root
  maidos-config/          # Configuration management
  maidos-auth/            # Authentication
  maidos-bus/             # Event bus
  maidos-llm/             # LLM integration
  maidos-log/             # Logging
  maidos-social/          # Social connectors
  maidos-google/          # Google APIs
  maidos-p2p/             # P2P networking
  maidos-chain/           # Blockchain
```

### Consequences

**Positive**:
- Single source of truth for shared logic
- Version consistency across products
- Unified testing and benchmarking
- Easier dependency management

**Negative**:
- Increased initial setup complexity
- Breaking changes affect all products
- Requires coordination across product teams

---

## ADR-002: Capability-Based Authentication Model

**Status**: Accepted
**Date**: 2025-08-15
**Deciders**: Security Team

### Context

Traditional role-based access control (RBAC) is too coarse-grained for MAIDOS products. A user may need to read driver data but not install drivers. OAuth2 scopes are external-facing, but we need internal authorization.

### Decision

We will implement capability-based access control in `maidos-auth`. Capabilities are fine-grained permissions (e.g., `read_user`, `write_driver`, `manage_config`). Tokens contain a list of granted capabilities. Policy checks verify that a token has required capabilities before allowing operations.

Capabilities list:
- `read_user`, `write_user`, `delete_user`
- `read_driver`, `write_driver`, `delete_driver`
- `read_hardware`, `write_hardware`
- `read_config`, `write_config`, `manage_config`
- `read_log`, `write_log`
- `admin`, `audit`
- `publish_event`, `subscribe_event`
- `invoke_llm`, `manage_llm`

### Consequences

**Positive**:
- Fine-grained least-privilege access control
- Explicit capability requirements in code
- Easier security audits

**Negative**:
- More complex token payloads
- Requires careful capability design

---

## ADR-003: Provider Abstraction for LLM Integration

**Status**: Accepted
**Date**: 2025-09-01
**Deciders**: AI Integration Team

### Context

MAIDOS products require LLM integration (CodeQC for code review, Office for document generation, Driver for hardware knowledge). OpenAI, Anthropic, Google, and local providers (Ollama, LM Studio) have different APIs, authentication methods, and response formats. Hard-coding provider logic in each product leads to vendor lock-in.

### Decision

We will define a `Provider` trait in `maidos-llm` with a unified interface:

```rust
#[async_trait]
pub trait Provider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn complete_streaming(&self, request: CompletionRequest) -> Result<StreamingResponse>;
}
```

Each provider implements this trait with provider-specific logic. A `Router` allows fallback, weighted routing, and cost optimization across multiple providers.

Supported providers: OpenAI, Anthropic, Google, DeepSeek, Groq, Mistral, Azure OpenAI, Cohere, Together AI, Replicate, Ollama (local), LM Studio (local), vLLM (local).

### Consequences

**Positive**:
- Vendor-agnostic application code
- Easy provider swapping
- Multi-provider fallback for reliability
- Cost optimization via routing

**Negative**:
- Lowest common denominator API
- Provider-specific features require extensions

---

## ADR-004: FFI Design with C ABI and C# Bindings

**Status**: Accepted
**Date**: 2025-09-10
**Deciders**: Interop Team

### Context

MAIDOS-Driver and MAIDOS-IME are C# WPF applications. MAIDOS-Forge uses C++ for performance. We need to expose Rust functionality to these languages. Rust's native ABI is unstable, so direct FFI is not viable.

### Decision

We will expose `extern "C"` functions for core operations in each crate. These functions use C-compatible types (pointers, integers, UTF-8 strings). C# bindings use `DllImport` with P/Invoke. Memory management follows ownership rules:

- Rust allocates, caller frees (via `_free` functions)
- Strings passed as `*const c_char` (UTF-8)
- Structs passed as opaque pointers (`*mut c_void`)

Example FFI function:
```rust
#[no_mangle]
pub extern "C" fn maidos_auth_verify_token(
    token: *const c_char,
    secret: *const c_char,
) -> *mut c_char {
    // Implementation
}
```

C# binding:
```csharp
[DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
public static extern IntPtr maidos_auth_verify_token(
    [MarshalAs(UnmanagedType.LPUTF8Str)] string token,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string secret
);
```

### Consequences

**Positive**:
- Stable ABI for cross-language interop
- Works with C, C++, C#, Python, etc.
- Rust safety benefits with minimal FFI surface

**Negative**:
- Manual memory management at FFI boundary
- Requires `unsafe` blocks
- Error handling via return codes (not Result)

---

## ADR-005: ZeroMQ for Event Bus

**Status**: Accepted
**Date**: 2025-09-15
**Deciders**: Architecture Team

### Context

MAIDOS products need inter-process communication (e.g., Driver UI → Driver Service, Forge Scheduler → Build Workers). Options include gRPC, message queues (RabbitMQ, Kafka), and ZeroMQ. Requirements: low latency, simple deployment, topic-based pub/sub.

### Decision

We will use ZeroMQ for `maidos-bus`. ZeroMQ is a lightweight messaging library with no broker (no deployment complexity). It supports pub/sub, push/pull, and request/reply patterns. We will use MessagePack for serialization (compact, schema-free).

Architecture:
- Publisher binds to `tcp://*:5555`
- Subscribers connect and filter by topic prefix
- Messages are `BusMessage { topic, payload }`

### Consequences

**Positive**:
- Zero broker deployment overhead
- Sub-10ms latency for local IPC
- Language bindings for C, C++, C#, Python
- Automatic reconnection support

**Negative**:
- No message persistence (vs Kafka)
- No guaranteed delivery (at-most-once semantics)
- Requires manual topic management

---

## ADR-006: HMAC-SHA256 for Token Signing

**Status**: Accepted
**Date**: 2025-09-20
**Deciders**: Security Team

### Context

`maidos-auth` needs to issue tamper-proof tokens for internal authentication. Options include JWT with RS256 (asymmetric), JWT with HS256 (symmetric), custom HMAC, or session IDs with server-side storage.

### Decision

We will use HMAC-SHA256 for token signing with a 256-bit shared secret. Tokens are JWTs with standard claims (sub, exp, iat, capabilities). Verification uses constant-time comparison via `ring::constant_time::verify_slices_are_equal` to prevent timing attacks.

Token format:
```json
{
  "sub": "user_123",
  "exp": 1706889600,
  "iat": 1706803200,
  "caps": ["read_user", "write_driver"]
}
```

### Consequences

**Positive**:
- Fast signing and verification (< 1ms)
- No public key infrastructure overhead
- Industry-standard JWT format
- Constant-time verification prevents timing attacks

**Negative**:
- Shared secret must be distributed securely
- Token revocation requires server-side store
- Asymmetric keys (RS256) offer stronger security

---

## ADR-007: Hot Reload with File Watcher

**Status**: Accepted
**Date**: 2025-09-25
**Deciders**: DevOps Team

### Context

MAIDOS services (Driver, Forge) run continuously and need configuration changes without restart. Manual restart disrupts service availability. Options include polling, inotify (Linux), and cross-platform file watchers.

### Decision

We will use the `notify` crate in `maidos-config` to watch config files for changes. On file modification, the config is reloaded and published to registered listeners. Reloads are debounced (500ms) to avoid thrashing on bulk edits. Thread-safe access uses `Arc<RwLock<Config>>`.

### Consequences

**Positive**:
- Zero-downtime config updates
- Cross-platform (inotify on Linux, FSEvents on macOS, ReadDirectoryChangesW on Windows)
- Automatic debouncing

**Negative**:
- File watcher may miss rapid changes
- Reload errors require fallback to old config
- Increased memory for dual-buffer config

---

## ADR-008: Streaming with Unified MaidosStreamItem

**Status**: Accepted
**Date**: 2026-01-15
**Deciders**: AI Integration Team

### Context

LLM providers (OpenAI, Anthropic, etc.) return streaming responses via Server-Sent Events (SSE). Each provider has a different response format (data-only, delta, function_call, done). Application code must handle each provider's format separately.

### Decision

We will define a unified `MaidosStreamItem` enum in `maidos-llm`:

```rust
pub enum MaidosStreamItem {
    TextDelta(String),
    FunctionCall { name: String, arguments: String },
    ToolCall { id: String, name: String, arguments: String },
    Done(TokenUsage),
    Error(String),
}
```

Each provider's `complete_streaming` method translates provider-specific SSE events to `MaidosStreamItem`. Application code consumes a single stream type.

### Consequences

**Positive**:
- Application code provider-agnostic
- Easier testing with mock providers
- Consistent error handling

**Negative**:
- Translation overhead for each provider
- Lowest common denominator feature set

---

## ADR-009: libp2p for P2P Networking

**Status**: Accepted
**Date**: 2026-02-01
**Deciders**: Networking Team

### Context

MAIDOS-Forge requires peer-to-peer communication for distributed builds. Requirements: peer discovery, NAT traversal, encrypted connections, and DHT-based routing. Options include raw sockets, QUIC, and libp2p.

### Decision

We will use libp2p in `maidos-p2p`. libp2p is a modular networking stack with built-in peer discovery (mDNS, DHT), NAT traversal (AutoNAT, Relay), and encryption (Noise protocol). It supports multiple transports (TCP, QUIC, WebSockets).

### Consequences

**Positive**:
- Battle-tested P2P stack (IPFS, Polkadot)
- Built-in NAT traversal and encryption
- Supports WebRTC for browser peers (future)

**Negative**:
- Large dependency tree
- Complex API with steep learning curve
- Overkill for simple use cases

---

## ADR-010: Ethers for Blockchain Integration

**Status**: Accepted
**Date**: 2026-02-05
**Deciders**: Blockchain Team

### Context

MAIDOS-Forge uses blockchain for build artifact attestation. We need to interact with Ethereum-compatible chains (Ethereum, Polygon, Arbitrum). Options include web3.rs, ethers-rs, and direct JSON-RPC.

### Decision

We will use ethers-rs in `maidos-chain`. Ethers provides high-level abstractions for wallets, contracts, and RPC clients. It supports EIP-1559 transactions, ENS resolution, and event listening.

### Consequences

**Positive**:
- Production-ready Ethereum library
- Strong typing for contracts (via abigen)
- Support for hardware wallets (Ledger, Trezor)

**Negative**:
- Ethereum-only (no Bitcoin, Solana)
- Large dependency (tokio, serde, etc.)
- Breaking changes between major versions
