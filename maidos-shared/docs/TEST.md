# Testing Strategy — maidos-shared

## Test Levels

### Unit Tests
- Per-crate tests in each module
- Run: `cargo test --workspace`
- Coverage target: >= 70% per crate

### Integration Tests
- Cross-crate interaction tests
- File: `tests/integration.rs`
- Run: `cargo test --test integration`

### Audit Tests
- Dependency vulnerability scanning
- Fake implementation detection
- File: `tests/audit_and_fake_check.rs`
- Run: `cargo test --test audit_and_fake_check`

### Benchmarks
- Performance regression detection
- Files: `benches/*.rs` (auth, bus, config, llm, ffi)
- Run: `cargo bench`

## CI Pipeline

1. `cargo fmt --check` — formatting
2. `cargo clippy --workspace -- -D warnings` — lint
3. `cargo test --workspace` — all tests
4. `cargo bench` — performance (nightly)
5. `cargo audit` — vulnerability scan

## Test Conventions

- Use `#[cfg(test)]` modules for unit tests
- Use `tempfile` crate for filesystem tests
- Async tests use `#[tokio::test]`
- No network-dependent tests in CI (mock providers)
