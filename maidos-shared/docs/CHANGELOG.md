# maidos-shared -- Changelog

| Field     | Value              |
|-----------|--------------------|
| Product   | maidos-shared      |
| Version   | 0.2.0              |
| Type      | Changelog          |

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-07

### Added

#### New Workspace Members
- **maidos-log** - Structured logging framework using `tracing` and `tracing-subscriber` with environment-based filtering and chrono timestamps.
- **maidos-social** - Social media connector adapters for Twitter, Discord, and Telegram with rate limiting and webhook support.
- **maidos-google** - Google API integration covering Drive file operations, Sheets read/write, and Calendar event management. Uses OAuth2 and service account authentication.
- **maidos-p2p** - Peer-to-peer networking module with handcrafted discovery and transport over TCP/tokio. Includes NAT traversal support.
- **maidos-chain** - Blockchain interaction module built on ethers-rs. Supports smart contract ABI calls, HD wallet management, and JSON-RPC provider abstraction.

#### Tier 2 Cloud LLM Providers (maidos-llm)
- Mistral (9 models, vision via Pixtral, function calling, SSE streaming)
- Azure OpenAI (5 models, deployment-based routing, SSE streaming)
- Cohere (6 models, RAG support, fallback streaming)
- Together AI (8 models, vision via Llama Vision, SSE streaming)
- Replicate (5 models, async prediction polling, fallback streaming)

#### Streaming and Tool Enhancements (maidos-llm)
- Unified `StreamChunk` and `StreamUsage` types in `streaming.rs`
- `SseParser` for Server-Sent Events parsing
- `StreamingResponse` trait for provider-agnostic streaming
- `MaidosTool` unified tool format with `ToProviderFormat` trait
- `ToolParameter` typed parameter definitions with constraints
- `ToolCall` / `ToolResult` execution handling
- `VisionNotSupported` and `ToolsNotSupported` error variants with suggestions

#### Integration and Audit Tests
- Cross-crate integration tests in `tests/integration.rs`
- Audit and fake-check test suite in `tests/audit_and_fake_check.rs`

#### Benchmarks
- `ffi_bench.rs` - FFI overhead measurement (38 benchmarks)

### Changed
- Workspace expanded from 4 crates to 8 crates (+ maidos-log, maidos-social, maidos-google, maidos-p2p, maidos-chain)
- Total LLM providers: 8 to 13 (10 cloud + 3 local)
- Total unit tests: 218 to 307
- Total integration tests: 15 to 20

### Code Quality
- TODO/FIXME count: 0
- Compilation warnings: 0
- Clippy warnings: 0
- Test pass rate: 100%
- Production code unwrap count: 0

## [0.1.0] - 2026-01-04

### Added
- Initial workspace with 4 core crates: maidos-config, maidos-auth, maidos-bus, maidos-llm
- TOML configuration loading with environment variable expansion and schema validation
- Capability-based authentication with HMAC-SHA256 token signing
- TCP event bus with MessagePack serialization and topic filtering
- Multi-provider LLM interface (OpenAI, Anthropic, Ollama)
- C FFI exports (31 functions across 4 crates)
- C# P/Invoke bindings (31 matching functions)
- Integration test suite (15 tests)
- Performance benchmark suite (auth, bus, config, llm)
- 5 executable examples

### Performance Baseline

| Operation           | Latency | Throughput    |
|---------------------|--------:|---------------|
| Token generation    |  700 ns | 1.4M ops/sec  |
| Token validation    |  680 ns | 1.5M ops/sec  |
| Capability check    |  3.3 ns | 300M ops/sec  |
| Config parse (full) |  19 us  | 53K ops/sec   |
| Event creation      |  120 ns | 8.3M ops/sec  |
| FFI call overhead   |  < 15%  | -             |

*maidos-shared CHANGELOG v0.2.0 -- CodeQC Gate C Compliant*
