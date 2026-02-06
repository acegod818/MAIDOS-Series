# maidos-shared — Non-Functional Requirements

## 1. Performance

| ID | Requirement | Target |
|----|------------|--------|
| NFR-P01 | Auth token validation latency | < 5 ms (cached), < 200 ms (network) |
| NFR-P02 | Message bus publish latency | < 1 ms for in-process messages |
| NFR-P03 | Config loading time | < 50 ms for standard config files |
| NFR-P04 | LLM client initialization | < 500 ms to Ollama connection |
| NFR-P05 | Log write throughput | > 100,000 structured events/sec |

## 2. Compatibility

| ID | Requirement | Target |
|----|------------|--------|
| NFR-C01 | Rust edition | 2024 (stable toolchain) |
| NFR-C02 | MSRV (minimum supported Rust version) | 1.80.0 |
| NFR-C03 | Platform support | Windows 10+, Linux (Ubuntu 22.04+), macOS 13+ |
| NFR-C04 | Cross-product ABI stability | Identical behavior across all MAIDOS consumers |

## 3. Dependency Management

| ID | Requirement | Target |
|----|------------|--------|
| NFR-D01 | Direct dependency count per sub-crate | <= 10 |
| NFR-D02 | No unsafe code in shared crate | Zero `unsafe` blocks unless FFI-mandated |
| NFR-D03 | License compatibility | All dependencies must be MIT / Apache-2.0 compatible |
| NFR-D04 | Audit compliance | `cargo audit` must pass with zero known vulnerabilities |

## 4. Reliability

| ID | Requirement | Target |
|----|------------|--------|
| NFR-R01 | Unit test coverage | >= 80% line coverage per sub-crate |
| NFR-R02 | Integration test pass rate | 100% on CI before merge |
| NFR-R03 | Graceful degradation | Network-dependent modules must handle offline mode |
| NFR-R04 | Error propagation | All public APIs return `Result<T, E>` with typed errors |

## 5. Security

| ID | Requirement | Target |
|----|------------|--------|
| NFR-S01 | Credential storage | No plaintext secrets in config files |
| NFR-S02 | TLS enforcement | All HTTP clients default to TLS 1.2+ |
| NFR-S03 | Token rotation | Auth tokens auto-refresh before expiry |
| NFR-S04 | Dependency supply chain | Cargo-vet or equivalent for third-party crates |

## 6. Maintainability

| ID | Requirement | Target |
|----|------------|--------|
| NFR-M01 | Public API documentation | 100% of public items have doc comments |
| NFR-M02 | Clippy compliance | Zero warnings with `clippy::pedantic` |
| NFR-M03 | Changelog discipline | Every PR updates CHANGELOG.md |
| NFR-M04 | Build time (clean) | < 120 seconds on standard CI runner |
