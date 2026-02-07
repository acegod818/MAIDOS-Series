# maidos-shared -- CI/CD Pipeline Configuration

| Field     | Value                     |
|-----------|---------------------------|
| Product   | maidos-shared              |
| Version   | 0.2.0                     |
| Type      | CI/CD Documentation        |

## Overview

The CI/CD pipeline runs on GitHub Actions and enforces the CodeQC quality gates on every push and pull request. Two workflow files control the pipeline:

| Workflow       | Trigger                  | Purpose                    |
|----------------|--------------------------|----------------------------|
| `ci.yml`       | push, pull_request       | Build, lint, test          |
| `release.yml`  | tag push (`v*`)          | Build artifacts, publish   |

## CI Workflow (ci.yml)

### Jobs

| Job        | Runs On          | Steps                                        |
|------------|------------------|----------------------------------------------|
| build      | ubuntu-latest    | checkout, toolchain, cargo build --workspace  |
| lint       | ubuntu-latest    | cargo fmt --check, cargo clippy -D warnings   |
| test       | ubuntu-latest    | cargo test --workspace                        |
| bench      | ubuntu-latest    | cargo bench --workspace (no fail on regression) |
| windows    | windows-latest   | cargo build --workspace, cargo test --workspace |
| macos      | macos-latest     | cargo build --workspace, cargo test --workspace |

### Matrix Strategy

The build and test jobs use a matrix to cover multiple platforms:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, 1.75.0]
```

### Quality Gates Enforced

| Gate | Check                                | Failure Policy |
|------|--------------------------------------|----------------|
| G1   | `cargo build --workspace`            | Block merge    |
| G1   | `cargo clippy -- -D warnings`        | Block merge    |
| G2   | `cargo test --workspace`             | Block merge    |
| G3   | `cargo test --test integration`      | Block merge    |
| -    | `cargo fmt --all -- --check`         | Block merge    |

### Caching

The pipeline caches the following directories to speed up builds:

- `~/.cargo/registry`
- `~/.cargo/git`
- `target/`

Cache key is based on `Cargo.lock` hash with OS prefix.

## Release Workflow (release.yml)

### Trigger

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Build Matrix

| Target                        | OS             | Artifact                  |
|-------------------------------|----------------|---------------------------|
| x86_64-unknown-linux-gnu      | ubuntu-latest  | .so libraries + tar.gz    |
| x86_64-pc-windows-msvc        | windows-latest | .dll libraries + zip      |
| x86_64-apple-darwin            | macos-latest   | .dylib libraries + tar.gz |
| aarch64-apple-darwin           | macos-latest   | .dylib libraries + tar.gz |

### Release Artifacts

Each platform build produces:

1. Shared libraries (`.so` / `.dll` / `.dylib`) for cdylib crates
2. C header file (`include/maidos.h`)
3. Source archive
4. NuGet package (`.nupkg`) for C# bindings
5. SHA256 checksums

### Steps

1. Checkout source at tagged commit
2. Install Rust stable toolchain
3. Run full test suite
4. Build in release mode for each target
5. Package artifacts with `scripts/release.sh`
6. Create GitHub Release with artifacts attached
7. Compute and attach SHA256 checksums

## Local CI Simulation

Developers can run the equivalent of CI locally using the QC scripts:

| Script              | Gate | Description                  |
|---------------------|------|------------------------------|
| qc\proof.bat      | All  | Run all four gates           |
| qc\build.bat      | G1   | Build + Clippy               |
| qc\unit.bat       | G2   | Unit tests                   |
| qc\integration.bat| G3   | Integration tests            |
| qc\e2e.bat        | G4   | Release build + audit        |

## Branch Protection Rules

| Rule                          | Setting  |
|-------------------------------|----------|
| Require status checks         | Yes      |
| Required checks               | build, lint, test |
| Require up-to-date branch     | Yes      |
| Require linear history        | Yes      |
| Restrict force pushes         | Yes      |

## Environment Variables

| Variable            | Usage                        | Required |
|---------------------|------------------------------|----------|
| `CARGO_TERM_COLOR`  | Force colored output in CI   | No       |
| `RUSTFLAGS`         | Additional compiler flags    | No       |
| `GITHUB_TOKEN`      | Release artifact upload      | Yes (release only) |

*maidos-shared CI v0.2.0 -- CodeQC Gate C Compliant*
