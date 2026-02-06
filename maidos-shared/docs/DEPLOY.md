# maidos-shared — Deployment and Publishing Guide

## 1. Publishing Strategy

maidos-shared sub-crates are published individually to crates.io. The root `maidos-shared`
crate is also published and re-exports sub-crates via feature flags.

## 2. Version Management

| Rule | Details |
|------|---------|
| Versioning scheme | Semantic versioning (major.minor.patch) |
| Workspace version | All sub-crates share a workspace-level version in root `Cargo.toml` |
| Version bump trigger | Any public API change requires at minimum a minor bump |
| Pre-release | Use `-alpha.N` or `-rc.N` suffixes for testing |

## 3. Publishing Order

Sub-crates must be published bottom-up following the dependency graph:

```
1. maidos-config, maidos-log         (no internal deps)
2. maidos-auth                       (depends on config, log)
3. maidos-bus, maidos-chain          (depends on log only)
4. maidos-google, maidos-llm         (depends on auth, config, log)
   maidos-p2p, maidos-social
5. maidos-shared (root)              (re-exports all)
```

## 4. Publish Checklist

- [ ] All tests pass: `cargo test --workspace`
- [ ] Clippy clean: `cargo clippy --workspace -- -D warnings`
- [ ] CHANGELOG.md updated for each modified sub-crate
- [ ] Version bumped in all affected `Cargo.toml` files
- [ ] Dry run: `cargo publish --dry-run -p <crate>`
- [ ] Tag release: `git tag maidos-shared-v<version>`
- [ ] Publish: `cargo publish -p <crate>` in dependency order
- [ ] Verify on crates.io: documentation renders correctly

## 5. CI/CD Pipeline

```
push to main
    |
    v
cargo fmt --check
cargo clippy --workspace
cargo test --workspace
cargo doc --no-deps --workspace
    |
    v (on tag push)
cargo publish (per crate, in order)
```

## 6. Dependency Updates

- Run `cargo update` weekly in CI to check for new dependency versions.
- Run `cargo audit` on every PR to detect known vulnerabilities.
- Pin major versions of critical dependencies (tokio, reqwest, serde).

## 7. Rollback Procedure

If a published version has a critical defect:

1. Publish a patch release with the fix.
2. If the defect is a security issue, yank the affected version: `cargo yank --version <ver> -p <crate>`.
3. Notify downstream consumers via GitHub advisory.

## 8. Local Development

```bash
# Build all sub-crates
cargo build --workspace

# Build with specific features only
cargo build -p maidos-shared --features "auth,log"

# Run all tests
cargo test --workspace

# Generate docs
cargo doc --workspace --open
```
