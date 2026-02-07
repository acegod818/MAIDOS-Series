# maidos-shared -- Build Instructions

| Field     | Value                |
|-----------|----------------------|
| Product   | maidos-shared        |
| Version   | 0.2.0                |
| Type      | Build Guide          |

## Prerequisites

| Requirement    | Minimum Version | Purpose                       |
|----------------|-----------------|-------------------------------|
| Rust toolchain | 1.75+           | Compiler and cargo             |
| CMake          | 3.20+           | C header generation (optional) |
| .NET SDK       | 8.0+            | C# bindings (optional)         |
| OpenSSL dev    | 1.1+            | TLS for reqwest (Linux only)   |

## Quick Build

```bash
# Build all workspace members in debug mode
cargo build --workspace

# Build in release mode with optimizations
cargo build --workspace --release

# Build a specific crate only
cargo build -p maidos-config
cargo build -p maidos-auth
cargo build -p maidos-llm
```

## Workspace Members

The workspace contains 8 crates built together by default:

| Crate          | Lib Type   | Output Artifact              |
|----------------|------------|------------------------------|
| maidos-config  | lib+cdylib | libmaidos_config.so / .dll   |
| maidos-auth    | lib+cdylib | libmaidos_auth.so / .dll     |
| maidos-bus     | lib+cdylib | libmaidos_bus.so / .dll      |
| maidos-llm     | lib+cdylib | libmaidos_llm.so / .dll      |
| maidos-log     | rlib       | (static, Rust only)          |
| maidos-social  | rlib       | (static, Rust only)          |
| maidos-google  | rlib       | (static, Rust only)          |
| maidos-p2p     | rlib       | (static, Rust only)          |
| maidos-chain   | rlib       | (static, Rust only)          |

## Feature Flags

### maidos-config

| Feature   | Default | Description                          |
|-----------|---------|--------------------------------------|
| watcher   | Yes     | File watcher for hot-reload support  |

To build without the file watcher:

```bash
cargo build -p maidos-config --no-default-features
```

### Workspace-Level Defaults

The workspace Cargo.toml centralizes dependency versions. Individual crates inherit from [workspace.dependencies] to ensure consistency.

## Linting

```bash
# Run clippy on all crates with deny on warnings
cargo clippy --workspace -- -D warnings

# Format check (no modification)
cargo fmt --all -- --check

# Format in place
cargo fmt --all
```

## Documentation Generation

```bash
# Generate rustdoc for all crates
cargo doc --workspace --no-deps

# Generate and open in browser
cargo doc --workspace --no-deps --open
```

## Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run a specific benchmark
cargo bench --bench auth_bench
cargo bench --bench bus_bench
cargo bench --bench config_bench
cargo bench --bench llm_bench
cargo bench --bench ffi_bench
```

## Cross-Compilation

### Linux to Windows

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --workspace --release --target x86_64-pc-windows-gnu
```

### Linux to macOS (requires osxcross)

```bash
rustup target add x86_64-apple-darwin
cargo build --workspace --release --target x86_64-apple-darwin
```

## Release Build Checklist

1. Run cargo build --workspace --release
2. Run cargo clippy --workspace -- -D warnings
3. Run cargo test --workspace
4. Run cargo bench --workspace
5. Verify zero warnings, zero test failures
6. Tag the release: git tag v0.2.0

## Troubleshooting

### OpenSSL Not Found (Linux)

```bash
sudo apt install libssl-dev pkg-config   # Debian/Ubuntu
sudo dnf install openssl-devel           # Fedora
```

### Long Compile Times

The ethers crate (used by maidos-chain) and reqwest can increase compile times. Use cargo build -p to build only the crate you are working on during development.

### Windows MSVC Linker Errors

Ensure the Visual Studio Build Tools are installed with the Desktop development with C++ workload.

*maidos-shared BUILD v0.2.0 -- CodeQC Gate C Compliant*
