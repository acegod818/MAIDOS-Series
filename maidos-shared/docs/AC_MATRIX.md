# Acceptance Criteria Matrix - maidos-shared

## maidos-auth

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-001 | Token issuance with user ID and capabilities returns valid JWT | Unit test: `test_issue_token` | ✅ |
| AC-002 | Token verification with correct secret returns claims | Unit test: `test_verify_token` | ✅ |
| AC-003 | Revoked token verification returns `TokenRevoked` error | Unit test: `test_revoke_token` | ✅ |
| AC-004 | Expired token verification returns `TokenExpired` error | Unit test: `test_expired_token` | ✅ |
| AC-005 | Token with missing capability fails policy check | Unit test: `test_policy_deny` | ✅ |
| AC-006 | Token with required capability passes policy check | Unit test: `test_policy_allow` | ✅ |
| AC-007 | Token store tracks 10,000 tokens without memory leak | Stress test: `test_token_store_capacity` | ✅ |
| AC-008 | Token verification uses constant-time comparison | Code audit: `ring::constant_time::verify_slices_are_equal` | ✅ |
| AC-009 | FFI `maidos_auth_verify_token` returns null for invalid token | FFI test: `test_ffi_verify_invalid` | ✅ |
| AC-010 | FFI `maidos_auth_free_result` does not leak memory | FFI test: valgrind / AddressSanitizer | ✅ |

## maidos-bus

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-011 | Publisher publishes message and returns success | Unit test: `test_publish` | ✅ |
| AC-012 | Subscriber receives message within 10ms | Integration test: `test_subscribe_latency` | ✅ |
| AC-013 | Subscriber with topic filter only receives matching messages | Unit test: `test_topic_filtering` | ✅ |
| AC-014 | Bus reconnects automatically after disconnect | Integration test: `test_reconnection` | ✅ |
| AC-015 | MessagePack serialization round-trips correctly | Unit test: `test_serialization` | ✅ |
| AC-016 | Publisher handles 5,000 messages/sec throughput | Benchmark: `bus_bench` | ✅ |
| AC-017 | Subscriber handles 5,000 messages/sec throughput | Benchmark: `bus_bench` | ✅ |
| AC-018 | FFI `maidos_bus_publish` returns 0 on success | FFI test: `test_ffi_publish` | ✅ |

## maidos-llm

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-019 | `create_provider` with valid API key returns provider | Unit test: `test_create_openai_provider` | ✅ |
| AC-020 | `complete()` sends request and returns response text | Integration test: `test_openai_completion` (requires API key) | ✅ |
| AC-021 | Streaming returns incremental `TextDelta` items | Integration test: `test_streaming` | ✅ |
| AC-022 | Router fallback strategy retries next provider on failure | Unit test: `test_router_fallback` | ✅ |
| AC-023 | Budget controller blocks request when limit exceeded | Unit test: `test_budget_limit` | ✅ |
| AC-024 | Vision request with image returns description | Integration test: `test_vision` (requires GPT-4o key) | ✅ |
| AC-025 | Function calling request returns tool call response | Integration test: `test_function_calling` | ✅ |
| AC-026 | MaidosTool format translates to OpenAI tool spec | Unit test: `test_tool_conversion` | ✅ |
| AC-027 | Provider error includes actionable error message | Unit test: `test_error_message` | ✅ |
| AC-028 | Ollama provider works without API key | Integration test: `test_ollama_local` | ✅ |
| AC-029 | FFI `maidos_llm_complete` returns UTF-8 string | FFI test: `test_ffi_complete` | ✅ |

## maidos-config

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-030 | `ConfigLoader::load` parses valid TOML file | Unit test: `test_load_toml` | ✅ |
| AC-031 | Environment variable expansion replaces `${VAR}` | Unit test: `test_env_expansion` | ✅ |
| AC-032 | Missing required field returns validation error | Unit test: `test_schema_validation` | ✅ |
| AC-033 | Hot reload detects file change within 500ms | Integration test: `test_hot_reload` | ✅ |
| AC-034 | Concurrent reads during reload do not block | Stress test: `test_concurrent_reads` | ✅ |
| AC-035 | Invalid TOML syntax returns parse error | Unit test: `test_invalid_toml` | ✅ |
| AC-036 | Default value syntax `${VAR:-default}` works | Unit test: `test_default_value` | ✅ |
| AC-037 | FFI `maidos_config_load` returns config pointer | FFI test: `test_ffi_load` | ✅ |

## maidos-log

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-038 | `init_logger` configures JSON output | Unit test: `test_json_format` | ✅ |
| AC-039 | Log rotation triggers at size limit | Integration test: `test_log_rotation` | ✅ |
| AC-040 | Per-module log levels filter correctly | Unit test: `test_log_filtering` | ✅ |
| AC-041 | Trace span IDs propagate across functions | Integration test: `test_trace_propagation` | ✅ |
| AC-042 | No PII appears in log output | Audit: grep for email/password patterns | ✅ |

## maidos-social

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-043 | Twitter client posts tweet successfully | Integration test: `test_post_tweet` (requires API key) | ✅ |
| AC-044 | Discord webhook sends message successfully | Integration test: `test_discord_webhook` (requires webhook URL) | ✅ |
| AC-045 | Telegram bot sends message successfully | Integration test: `test_telegram_send` (requires bot token) | ✅ |
| AC-046 | OAuth flow completes and returns access token | Integration test: `test_oauth_flow` | ✅ |
| AC-047 | Rate limit error includes retry-after header | Unit test: `test_rate_limit_error` | ✅ |

## maidos-google

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-048 | Drive client uploads file successfully | Integration test: `test_drive_upload` (requires OAuth) | ✅ |
| AC-049 | Sheets client reads spreadsheet data | Integration test: `test_sheets_read` (requires OAuth) | ✅ |
| AC-050 | Calendar client creates event successfully | Integration test: `test_calendar_create` (requires OAuth) | ✅ |
| AC-051 | OAuth2 refresh token renews expired access token | Integration test: `test_oauth_refresh` | ✅ |

## maidos-p2p

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-052 | Peer discovery finds >= 1 peer on local network | Integration test: `test_peer_discovery` | ✅ |
| AC-053 | Peer-to-peer message send/receive succeeds | Integration test: `test_p2p_messaging` | ✅ |
| AC-054 | NAT traversal establishes connection via relay | Integration test: `test_nat_traversal` | ✅ |
| AC-055 | Encrypted connection uses Noise protocol | Code audit: libp2p noise transport | ✅ |

## maidos-chain

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-056 | Wallet creation returns valid address | Unit test: `test_create_wallet` | ✅ |
| AC-057 | RPC client queries block number successfully | Integration test: `test_rpc_query` (requires RPC endpoint) | ✅ |
| AC-058 | Contract call sends transaction and returns receipt | Integration test: `test_contract_call` (requires testnet) | ✅ |
| AC-059 | Event listener receives contract event | Integration test: `test_event_listener` | ✅ |
| AC-060 | Transaction signing uses private key correctly | Unit test: `test_sign_transaction` | ✅ |

## Cross-Crate Integration

| ID | Criterion | Verification Method | Status |
|----|-----------|---------------------|--------|
| AC-061 | All crates use `maidos-config` for configuration | Code audit: Cargo.toml dependencies | ✅ |
| AC-062 | All crates use `maidos-log` for structured logging | Code audit: tracing macros | ✅ |
| AC-063 | Integration test suite passes all 50 tests | `cargo test --test integration` | ✅ |
| AC-064 | Benchmark suite runs without errors | `cargo bench` | ✅ |
| AC-065 | FFI test suite passes all tests | `cargo test --test ffi` | ✅ |
| AC-066 | No Clippy warnings in workspace | `cargo clippy --all-targets --all-features` | ✅ |
| AC-067 | Documentation builds without warnings | `cargo doc --no-deps --document-private-items` | ✅ |
| AC-068 | All public APIs have doc comments | Manual audit | ✅ |
| AC-069 | All crates pass audit for unsafe code | Audit: < 20 unsafe blocks (FFI only) | ✅ |
| AC-070 | Zero unwrap() in production code paths | Audit: grep -r "unwrap()" src/ --exclude tests | ✅ |
