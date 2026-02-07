# maidos-shared -- Dependency Overview

| Field     | Value                     |
|-----------|---------------------------|
| Product   | maidos-shared              |
| Version   | 0.2.0                     |
| Type      | Dependency Documentation   |

## Overview

All external dependencies are declared in the workspace-level `Cargo.toml` under `[workspace.dependencies]` to ensure version consistency across all 8 crates. Individual crate manifests reference these with `{ workspace = true }`.

## Core Dependencies

### Serialization

| Crate       | Version | Used By                          | Rationale                                |
|-------------|---------|----------------------------------|------------------------------------------|
| serde       | 1.0     | All crates                       | Standard Rust serialization framework    |
| serde_json  | 1.0     | All crates                       | JSON parsing for APIs and configuration  |
| toml        | 0.7     | maidos-config                    | TOML configuration file parsing          |
| rmp-serde   | 1.1     | maidos-bus                       | MessagePack binary serialization for event bus messages |

### Async Runtime

| Crate       | Version | Used By                              | Rationale                                |
|-------------|---------|--------------------------------------|------------------------------------------|
| tokio       | 1.32    | bus, llm, social, google, p2p, chain | Industry-standard async runtime          |
| async-trait | 0.1     | llm, social, google, p2p            | Enables async methods in trait definitions |
| futures     | 0.3     | bus, llm                             | Future combinators and stream utilities  |

### Cryptography

| Crate    | Version | Used By                    | Rationale                                   |
|----------|---------|----------------------------|---------------------------------------------|
| hmac     | 0.12.1  | maidos-auth, maidos-google | HMAC message authentication codes           |
| sha2     | 0.10    | maidos-auth, maidos-chain  | SHA-256 hashing for token signing           |
| ring     | 0.17    | (workspace-level)          | TLS and general-purpose cryptography        |
| zeroize  | 1.8.2   | (transitive, pinned)       | Secure memory zeroing for secret material   |

### HTTP

| Crate   | Version | Used By                    | Rationale                                       |
|---------|---------|----------------------------|-------------------------------------------------|
| reqwest | 0.11    | llm, social, google, chain | HTTP client with JSON, streaming, and rustls-tls |

The `reqwest` dependency uses `rustls-tls-webpki-roots` instead of native TLS to enable consistent cross-compilation without requiring system OpenSSL.

### Logging and Tracing

| Crate               | Version | Used By      | Rationale                             |
|----------------------|---------|--------------|---------------------------------------|
| tracing              | 0.1     | All crates   | Structured, context-aware logging     |
| tracing-subscriber   | 0.3     | maidos-log   | Log output formatting and filtering   |

### Error Handling

| Crate     | Version | Used By     | Rationale                              |
|-----------|---------|-------------|----------------------------------------|
| thiserror | 1.0     | All crates  | Derive macro for custom error types    |

### Synchronization

| Crate       | Version | Used By    | Rationale                              |
|-------------|---------|------------|----------------------------------------|
| parking_lot | 0.12    | auth, llm  | Faster mutex/rwlock than std           |

### Time

| Crate  | Version | Used By      | Rationale                              |
|--------|---------|--------------|----------------------------------------|
| chrono | 0.4     | log, google  | Date/time handling with serde support  |

### File Watching

| Crate  | Version | Used By       | Rationale                               |
|--------|---------|---------------|-----------------------------------------|
| notify | 6.0     | maidos-config | Cross-platform file system event watcher |

### Bytes

| Crate | Version | Used By    | Rationale                                |
|-------|---------|------------|------------------------------------------|
| bytes | 1.0     | maidos-llm | Efficient byte buffer for streaming data |

## Crate-Specific Dependencies

### maidos-google

| Crate       | Version | Rationale                                      |
|-------------|---------|------------------------------------------------|
| base64      | 0.21    | Base64 encoding for OAuth2 JWT assertions      |
| rand        | 0.8     | Random nonce generation for OAuth2             |
| url         | 2.5     | URL parsing and construction                   |
| urlencoding | 2.1     | URL percent-encoding for query parameters      |

### maidos-social

| Crate | Version | Rationale                                 |
|-------|---------|-------------------------------------------|
| url   | 2.5     | URL parsing for API endpoint construction |

### maidos-p2p

| Crate | Version | Rationale                               |
|-------|---------|------------------------------------------|
| rand  | 0.8     | Peer ID generation and randomized delays |

### maidos-chain

| Crate  | Version | Rationale                                      |
|--------|---------|------------------------------------------------|
| ethers | 2.0     | Ethereum JSON-RPC, contract ABI, wallet support |
| hex    | 0.4     | Hex encoding/decoding for addresses and hashes  |
| rand   | 0.8     | Key generation and nonce randomization          |

## Dev Dependencies

| Crate      | Version | Used By            | Rationale                          |
|------------|---------|--------------------|------------------------------------|
| tempfile   | 3.10    | config, bus, tests | Temporary files for test isolation |
| tokio-test | 0.4     | maidos-llm         | Async test utilities               |
| wiremock   | 0.5     | maidos-llm         | HTTP mock server for provider tests |
| criterion  | 0.5     | benchmarks         | Statistical benchmarking framework  |

## Dependency Policy

1. **Minimize transitive dependencies.** Prefer crates with small dependency trees.
2. **Pin workspace versions.** All shared dependency versions are declared once at the workspace level.
3. **Prefer pure-Rust implementations.** Use `rustls` over `native-tls` to avoid system library dependencies.
4. **Audit regularly.** Run `cargo audit` to check for known vulnerabilities.
5. **No duplicates.** Use `cargo deny` or manual checks to prevent multiple versions of the same crate.

## Dependency Count Summary

| Category                | Count |
|-------------------------|-------|
| Direct (workspace)      | 18    |
| Direct (crate-specific) | 7     |
| Dev dependencies        | 4     |

*maidos-shared DEPS v0.2.0 -- CodeQC Gate C Compliant*
