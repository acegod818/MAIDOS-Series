# Non-Functional Requirements - maidos-shared

## 1. Performance

| ID | Requirement | Target | Measurement |
|----|-------------|--------|-------------|
| NFR-P1 | Token issuance latency | < 1ms at p99 | Benchmark suite: auth_bench |
| NFR-P2 | Token verification latency | < 1ms at p99 | Benchmark suite: auth_bench |
| NFR-P3 | Event bus publish latency | < 5ms at p99 | Benchmark suite: bus_bench |
| NFR-P4 | Event bus subscribe latency | < 10ms at p99 | Benchmark suite: bus_bench |
| NFR-P5 | Config load time | < 50ms for 100KB TOML | Benchmark suite: config_bench |
| NFR-P6 | LLM request overhead | < 100ms (excluding provider latency) | Benchmark suite: llm_bench |
| NFR-P7 | FFI call overhead | < 100μs per call | Benchmark suite: ffi_bench |
| NFR-P8 | Hot reload detection | < 500ms after file change | Integration test: hot_reload_test |
| NFR-P9 | Memory footprint | < 50 MB with all crates loaded | Runtime measurement |
| NFR-P10 | Streaming throughput | >= 1000 tokens/sec | Integration test: streaming_test |

## 2. Reliability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-R1 | Event bus uptime | 99.9% (8.76 hours downtime/year) | Monitoring: connection failure rate |
| NFR-R2 | Token store consistency | Zero data loss under concurrent access | Stress test: 1000 concurrent token ops |
| NFR-R3 | Config hot reload | No service interruption during reload | Integration test: reload_under_load |
| NFR-R4 | LLM provider failover | < 5 second recovery with fallback router | Integration test: provider_failure_test |
| NFR-R5 | Bus reconnection | Automatic reconnect within 30 seconds | Integration test: bus_disconnect_test |
| NFR-R6 | Crash resilience | No panics in production code | Audit: Zero unwrap() in non-test code |

## 3. Security

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-S1 | Token signing algorithm | HMAC-SHA256 with >= 256-bit key | Code audit: maidos-auth/src/token.rs |
| NFR-S2 | Token comparison | Constant-time verification | Code audit: ring::constant_time::verify_slices_are_equal |
| NFR-S3 | Secret storage | No plaintext secrets in logs or disk | Audit: grep -r "password\\|secret\\|key" src/ |
| NFR-S4 | HTTPS enforcement | All HTTP clients require TLS | Code audit: reqwest rustls-tls-webpki-roots |
| NFR-S5 | Input validation | All FFI inputs validated before use | Code audit: FFI functions check null pointers |
| NFR-S6 | Memory safety | Zero unsafe blocks outside FFI boundary | Audit: unsafe count < 20 (FFI only) |
| NFR-S7 | API key protection | API keys never logged or exposed in errors | Integration test: key_redaction_test |
| NFR-S8 | Token expiration | All tokens have finite lifetime (max 24 hours) | Unit test: token expiration test |

## 4. Scalability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-SC1 | Concurrent token operations | >= 10,000 ops/sec | Stress test: concurrent_token_test |
| NFR-SC2 | Bus message throughput | >= 5,000 messages/sec | Stress test: bus_throughput_test |
| NFR-SC3 | Config reload under load | No failures during 100 concurrent reads | Stress test: config_reload_stress |
| NFR-SC4 | LLM request concurrency | >= 100 concurrent requests per provider | Stress test: llm_concurrency_test |
| NFR-SC5 | P2P peer capacity | >= 1,000 concurrent peers | Stress test: p2p_peer_capacity_test |

## 5. Maintainability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-M1 | Code coverage | >= 80% line coverage | cargo tarpaulin |
| NFR-M2 | Documentation coverage | 100% public API documented | cargo doc --no-deps --document-private-items |
| NFR-M3 | Clippy warnings | Zero warnings in production code | cargo clippy --all-targets --all-features |
| NFR-M4 | Build warnings | Zero compiler warnings | cargo build --all-features 2>&1 \| grep warning |
| NFR-M5 | Code complexity | Cyclomatic complexity < 15 per function | Manual audit or tool |
| NFR-M6 | Dependency count | < 50 direct dependencies per crate | cargo tree --depth 1 |

## 6. Usability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-U1 | API learning curve | Developers can integrate in < 1 hour | User testing with new engineers |
| NFR-U2 | Error messages | All errors include actionable guidance | Manual audit: error messages |
| NFR-U3 | Example completeness | Each crate has >= 3 examples | Check examples/ directory |
| NFR-U4 | FFI documentation | C/C++/C# examples for all FFI functions | Check bindings/ directory |

## 7. Portability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-PT1 | Windows support | Windows 10 version 1809+ | CI: Windows runner |
| NFR-PT2 | Linux support | Kernel 4.4+ (glibc 2.27+) | CI: Ubuntu 20.04 runner |
| NFR-PT3 | macOS support | macOS 10.15+ | CI: macOS runner |
| NFR-PT4 | Cross-compilation | Support x64 and ARM64 targets | CI: Cross-compile matrix |

## 8. Observability

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-O1 | Structured logging | All logs in JSON format | Manual audit: log output |
| NFR-O2 | Trace correlation | Span IDs propagate across crates | Integration test: trace_propagation_test |
| NFR-O3 | Metrics exposure | Token ops, bus messages, LLM requests tracked | Check maidos-log metrics API |
| NFR-O4 | Health check | Each crate exposes health check function | Check health_check() in each lib.rs |

## 9. Legal and Compliance

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-L1 | License compliance | MIT license, all dependencies MIT/Apache-2.0 compatible | cargo license |
| NFR-L2 | Export control | No cryptography stronger than 256-bit symmetric | Manual audit |
| NFR-L3 | Privacy | No PII logged or persisted | Code audit: log statements |

## 10. Testing

| ID | Requirement | Target | Verification |
|----|-------------|--------|--------------|
| NFR-T1 | Unit test count | >= 200 unit tests across workspace | cargo test --lib |
| NFR-T2 | Integration test count | >= 50 integration tests | cargo test --test integration |
| NFR-T3 | Benchmark suite | >= 5 benchmark harnesses | ls benches/ |
| NFR-T4 | CI execution time | Full test suite < 10 minutes | GitHub Actions: CI workflow duration |
