# maidos-shared -- Glossary of Terms

| Field     | Value              |
|-----------|--------------------|
| Product   | maidos-shared      |
| Version   | 0.2.0              |
| Type      | Glossary           |

## General Terms

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| MAIDOS              | The overarching product ecosystem. maidos-shared provides foundational libraries used across all MAIDOS applications. |
| Workspace           | A Cargo workspace containing multiple crates that are built and tested together.                 |
| Crate               | A Rust compilation unit (library or binary). maidos-shared contains 8 library crates.           |
| cdylib              | A Cargo library type that produces a C-compatible dynamic library (.so, .dll, .dylib) for FFI.  |
| rlib                | A Cargo library type that produces a Rust-only static library, consumed by other Rust crates.   |
| FFI                 | Foreign Function Interface. The mechanism by which Rust code exposes C-compatible functions for use by other languages. |
| P/Invoke            | Platform Invocation Services. The .NET mechanism for calling native (C/Rust) functions from C#. |
| CodeQC              | The MAIDOS quality control framework that enforces build, test, and compliance gates.           |

## Authentication (maidos-auth)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Capability          | A discrete permission (e.g., ConfigRead, LlmInvoke) granted to a token holder.                 |
| CapabilitySet       | A bitmask representing a collection of capabilities assigned to a single token.                 |
| HMAC-SHA256         | Hash-based Message Authentication Code using SHA-256. Used to sign and verify tokens.           |
| Token               | A signed string containing a capability set and expiration timestamp, used for authorization.   |
| Token TTL           | Time-to-live. The duration in seconds before a token expires.                                   |
| Session             | A stateful authentication context with refresh token rotation support.                          |
| OAuth2              | An open authorization framework. maidos-auth supports the authorization code flow with PKCE.    |
| PKCE                | Proof Key for Code Exchange. A security extension for OAuth2 that prevents authorization code interception. |
| JWT                 | JSON Web Token. A compact, URL-safe token format for claims transmission.                       |

## Configuration (maidos-config)

| Term                  | Definition                                                                                    |
|-----------------------|-----------------------------------------------------------------------------------------------|
| Hot-Reload            | The ability to detect configuration file changes at runtime and reload without restarting.     |
| Environment Expansion | Substitution of `${VAR}` or `${VAR:-default}` placeholders in config files with environment variable values. |
| Schema Validation     | Verification that a configuration file conforms to the expected structure and types.           |
| TOML                  | Tom's Obvious, Minimal Language. The configuration file format used by maidos-config.         |

## Event Bus (maidos-bus)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Event Bus           | A publish/subscribe messaging system for decoupled communication between modules.               |
| Publisher           | A component that sends events to the bus on a specific topic.                                   |
| Subscriber          | A component that registers interest in a topic and receives matching events.                    |
| Topic               | A string-based routing key used to filter events. Supports wildcard matching.                   |
| MessagePack         | A binary serialization format used for compact, efficient event payloads on the bus.            |
| Backpressure        | Flow control mechanism using bounded channels to prevent producers from overwhelming consumers. |
| Bounded Channel     | A tokio channel with a fixed capacity that blocks or returns an error when full.                |

## LLM Integration (maidos-llm)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| LLM                 | Large Language Model. An AI model that generates text from prompts (e.g., GPT-4, Claude).       |
| Provider            | An implementation of the LlmProvider trait that connects to a specific LLM service.             |
| CompletionRequest   | A struct representing a request to an LLM, containing messages, model selection, and parameters.|
| StreamChunk         | A single piece of a streaming response from an LLM provider.                                   |
| SSE                 | Server-Sent Events. A protocol for streaming text data over HTTP, used by most LLM APIs.        |
| MaidosTool          | A provider-agnostic tool definition that can be converted to OpenAI, Anthropic, or Google format.|
| Function Calling    | The ability of an LLM to request execution of named functions with structured arguments.        |
| Vision              | The ability of an LLM to process image inputs alongside text.                                   |
| RAG                 | Retrieval-Augmented Generation. Providing external documents to an LLM for grounded responses.  |
| Budget Controller   | A component that tracks and enforces spending limits (daily, monthly, per-request) for LLM API calls. |
| Fallback            | A routing strategy that tries providers in order, falling back to the next on failure.          |

## Logging (maidos-log)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Span                | A tracing context representing a unit of work, with a start and end time.                       |
| Structured Logging  | Logging with machine-parseable key-value fields rather than plain text messages.                 |
| Sink                | An output destination for log records (console, file, or remote endpoint).                      |
| Env Filter          | A tracing-subscriber filter configured via the RUST_LOG environment variable.                   |

## Networking (maidos-p2p)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Peer                | A node in the P2P network identified by a unique peer ID.                                       |
| Discovery           | The process by which peers find and connect to each other.                                      |
| NAT Traversal       | Techniques for establishing connections between peers behind Network Address Translation devices.|
| Hole Punching       | A NAT traversal technique where both peers send packets to create firewall pinhole entries.     |
| Bootstrap Peer      | A well-known peer used as an initial contact point for network discovery.                       |
| Noise Protocol      | A framework for building cryptographic protocols used for encrypted peer-to-peer channels.      |

## Blockchain (maidos-chain)

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Smart Contract      | Self-executing code deployed on a blockchain, interacted with via ABI-encoded calls.            |
| ABI                 | Application Binary Interface. The specification for encoding function calls and data on Ethereum.|
| HD Wallet           | Hierarchical Deterministic Wallet. Generates a tree of key pairs from a single seed phrase.     |
| JSON-RPC            | The remote procedure call protocol used to communicate with Ethereum nodes.                     |
| Chain ID            | A numeric identifier for an Ethereum-compatible blockchain network.                             |
| ethers-rs           | The Rust library used by maidos-chain for Ethereum interaction.                                 |

## Quality Assurance

| Term                | Definition                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------|
| Gate (G1-G4)        | A quality checkpoint in the CodeQC process. G1=Build, G2=Unit, G3=Integration, G4=E2E.         |
| Proof Pack          | A combined run of all four gates (G1-G4) with a random nonce and timestamp for audit trail.     |
| Nonce               | A random value included in proof output to demonstrate the tests were actually executed.         |
| Grade SS            | The highest CodeQC quality grade, indicating zero defects across all gates.                     |

*maidos-shared GLOSSARY v0.2.0 -- CodeQC Gate C Compliant*
