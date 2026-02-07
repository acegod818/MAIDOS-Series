# maidos-shared -- Product Specification

| Field         | Value                                              |
|---------------|----------------------------------------------------|
| Product       | maidos-shared                                      |
| Version       | 0.2.0                                              |
| Type          | Rust Workspace (Multi-Crate Library)               |
| Description   | Shared foundational libraries for the MAIDOS ecosystem |
| Build System  | Cargo (Rust)                                       |
| Test Runner   | `cargo test`                                       |
| License       | Proprietary -- MAIDOS Project                      |

## Purpose

maidos-shared provides the foundational building blocks used across all MAIDOS products. It is organized as a Cargo workspace containing 8 specialized crates covering configuration, authentication, messaging, AI integration, logging, social connectors, Google API access, peer-to-peer networking, and blockchain capabilities.

## Crate Overview

| Crate          | Description                              |
|----------------|------------------------------------------|
| maidos-config  | Configuration management and loading     |
| maidos-auth    | Authentication (JWT, OAuth)              |
| maidos-bus     | Event bus / message passing              |
| maidos-llm     | LLM integration and API connectors       |
| maidos-log     | Structured logging framework             |
| maidos-social  | Social media connectors                  |
| maidos-google  | Google API integration                   |
| maidos-p2p     | Peer-to-peer networking                  |
| maidos-chain   | Blockchain module                        |

## Core Features

| Feature                    | Crate(s)           | Description                          |
|----------------------------|--------------------|--------------------------------------|
| Configuration Management   | maidos-config      | TOML/JSON config loading, env merge  |
| JWT Authentication         | maidos-auth        | Token generation and validation      |
| OAuth2 Flows               | maidos-auth        | OAuth2 authorization code, PKCE      |
| Event Bus                  | maidos-bus         | Pub/sub, async message dispatch      |
| LLM Integration            | maidos-llm         | OpenAI, Claude, local model support  |
| Structured Logging         | maidos-log         | JSON logs, tracing integration       |
| Social Connectors          | maidos-social      | Twitter, Discord, Telegram adapters  |
| Google API                 | maidos-google      | Drive, Sheets, Calendar access       |
| P2P Networking             | maidos-p2p         | libp2p-based peer discovery          |
| Blockchain                 | maidos-chain       | On-chain interaction utilities       |

## Build Commands

```bash
cargo build            # Build all crates
cargo test             # Run all tests
cargo clippy           # Lint all crates
cargo doc --no-deps    # Generate documentation
```

## Version History

| Version | Date       | Notes                                |
|---------|------------|--------------------------------------|
| 0.1.0   | 2025-08    | Initial workspace with core crates   |
| 0.2.0   | 2026-02    | Full 8-crate workspace, compliance   |

*maidos-shared SPEC v0.2.0 -- CodeQC Gate C Compliant*
