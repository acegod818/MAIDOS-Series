# Deployment Guide - maidos-shared

## Overview

maidos-shared is a Rust workspace library distributed via crates.io for Rust consumers and as compiled binaries (DLL/SO) for FFI consumers (C, C++, C#). This guide covers building, testing, publishing, and integration.

## Prerequisites

- **Rust Toolchain**: 1.75 or later (2021 edition)
- **Cargo**: Bundled with Rust
- **C/C++ Compiler**: MSVC (Windows), GCC/Clang (Linux/macOS) for FFI bindings
- **CMake**: 3.20+ (optional, for C examples)
- **.NET SDK**: 8.0+ (optional, for C# bindings)

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup default stable
```

## Building from Source

### 1. Clone Repository

```bash
git clone https://github.com/maidos/maidos-shared.git
cd maidos-shared
```

### 2. Build All Crates

```bash
# Debug build
cargo build --workspace

# Release build (optimized)
cargo build --workspace --release
```

**Output**: Static libraries (`.rlib`) in `target/release/` for each crate.

### 3. Build FFI Dynamic Library

```bash
# Add cdylib target to workspace Cargo.toml
[lib]
crate-type = ["rlib", "cdylib"]

# Build
cargo build --release
```

**Output**:
- Windows: `target/release/maidos_shared.dll`
- Linux: `target/release/libmaidos_shared.so`
- macOS: `target/release/libmaidos_shared.dylib`

### 4. Generate C Header Files

```bash
cargo install cbindgen
cbindgen --config cbindgen.toml --output include/maidos.h
```

**Output**: `include/maidos.h` with all FFI function declarations.

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test --workspace --lib

# Run specific crate tests
cargo test -p maidos-auth
```

### Integration Tests

```bash
# Run all integration tests
cargo test --workspace --test integration

# Run with environment variables (for API key tests)
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
cargo test --test integration -- --ignored
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --bench auth_bench
```

### Code Quality Checks

```bash
# Clippy (linter)
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check

# Security audit
cargo install cargo-audit
cargo audit

# License check
cargo install cargo-license
cargo license
```

## Publishing to crates.io

### Prerequisites

1. Create account at https://crates.io
2. Obtain API token: `cargo login <YOUR_TOKEN>`

### Publish Workflow

```bash
# 1. Update version in all Cargo.toml files
# Workspace version is in root Cargo.toml [workspace.package]
# Edit version = "0.2.0" → "0.3.0"

# 2. Update CHANGELOG.md
# Add release notes for 0.3.0

# 3. Commit version bump
git add Cargo.toml */Cargo.toml CHANGELOG.md
git commit -m "Bump version to 0.3.0"
git tag v0.3.0
git push origin main --tags

# 4. Publish crates in dependency order
cargo publish -p maidos-config
cargo publish -p maidos-log
cargo publish -p maidos-auth
cargo publish -p maidos-bus
cargo publish -p maidos-llm
cargo publish -p maidos-social
cargo publish -p maidos-google
cargo publish -p maidos-p2p
cargo publish -p maidos-chain

# Wait ~10 seconds between publishes for crates.io to index
```

### Yanking a Release

If a critical bug is found:

```bash
cargo yank --vers 0.3.0 -p maidos-auth
```

This prevents new projects from using 0.3.0 but does not break existing builds.

## Integration as Rust Dependency

### Add to Cargo.toml

```toml
[dependencies]
maidos-config = "0.2"
maidos-auth = "0.2"
maidos-bus = "0.2"
maidos-llm = "0.2"
maidos-log = "0.2"
```

### Use in Code

```rust
use maidos_auth::{TokenIssuer, Capability};
use maidos_llm::{create_provider, ProviderType, CompletionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auth
    let issuer = TokenIssuer::new(b"my-secret");
    let token = issuer.issue("user", vec![Capability::ReadUser], Duration::from_secs(3600))?;

    // LLM
    let provider = create_provider(ProviderType::OpenAI, Some("sk-..."), None)?;
    let request = CompletionRequest::quick("Hello");
    let response = provider.complete(request).await?;
    println!("{}", response.text);

    Ok(())
}
```

## Integration as C/C++ Library

### 1. Copy Compiled Library and Header

```bash
# From maidos-shared repository
cp target/release/libmaidos_shared.so /usr/local/lib/
cp include/maidos.h /usr/local/include/

# Update library cache (Linux)
sudo ldconfig
```

### 2. Link in C Project

**CMakeLists.txt**:
```cmake
cmake_minimum_required(VERSION 3.20)
project(my_app C)

find_library(MAIDOS_LIB maidos_shared PATHS /usr/local/lib)
include_directories(/usr/local/include)

add_executable(my_app main.c)
target_link_libraries(my_app ${MAIDOS_LIB})
```

**main.c**:
```c
#include <maidos.h>
#include <stdio.h>

int main() {
    void* llm = maidos_llm_create_provider("ollama", NULL, NULL);
    if (llm == NULL) {
        fprintf(stderr, "Failed to create provider: %s\n", maidos_get_last_error());
        return 1;
    }

    char* response = maidos_llm_complete(llm, "llama3.2", "Why is the sky blue?");
    printf("Response: %s\n", response);

    maidos_llm_free_string(response);
    maidos_llm_free_provider(llm);
    return 0;
}
```

**Build**:
```bash
mkdir build && cd build
cmake ..
make
./my_app
```

## Integration as C# Library

### 1. Copy DLL to Project

```bash
# Copy DLL to C# project bin directory
cp target/release/maidos_shared.dll MyApp/bin/Debug/net8.0/
```

**MyApp.csproj**:
```xml
<ItemGroup>
  <None Include="maidos_shared.dll">
    <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
  </None>
</ItemGroup>
```

### 2. Define P/Invoke Bindings

**MaidosBindings.cs**:
```csharp
using System.Runtime.InteropServices;

public static class MaidosAuth {
    [DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr maidos_auth_issue_token(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string userId,
        [MarshalAs(UnmanagedType.LPArray, ArraySubType = UnmanagedType.LPUTF8Str)] string[] capabilities,
        int capabilitiesLen,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string secret,
        int ttlSeconds
    );

    [DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern void maidos_auth_free_string(IntPtr s);
}
```

### 3. Use in C# Code

```csharp
string[] caps = { "read_user", "write_driver" };
IntPtr tokenPtr = MaidosAuth.maidos_auth_issue_token("user_123", caps, caps.Length, "my-secret", 3600);
string token = Marshal.PtrToStringUTF8(tokenPtr);
Console.WriteLine($"Token: {token}");
MaidosAuth.maidos_auth_free_string(tokenPtr);
```

## Containerized Deployment (Docker)

**Dockerfile**:
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/libmaidos_shared.so /usr/local/lib/
COPY --from=builder /build/include/maidos.h /usr/local/include/
RUN ldconfig
CMD ["/bin/bash"]
```

**Build and Run**:
```bash
docker build -t maidos-shared:0.2.0 .
docker run -it maidos-shared:0.2.0
```

## Continuous Integration (GitHub Actions)

**.github/workflows/ci.yml**:
```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --workspace --release
      - run: cargo test --workspace
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo bench --no-run
```

## Version Matrix

| maidos-shared | Rust MSRV | Windows | Linux | macOS |
|---------------|-----------|---------|-------|-------|
| 0.1.0 | 1.70 | 10+ | glibc 2.27+ | 10.15+ |
| 0.2.0 | 1.75 | 10+ | glibc 2.27+ | 10.15+ |

## Troubleshooting

### Build Fails with "linker not found"

Install build essentials:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora
sudo dnf install gcc gcc-c++

# macOS
xcode-select --install
```

### FFI Linker Error on Windows

Install Visual Studio with "Desktop development with C++" workload:
https://visualstudio.microsoft.com/downloads/

### C# DllNotFoundException

Ensure `maidos_shared.dll` is in:
- Same directory as .exe
- `bin/Debug/net8.0/` or `bin/Release/net8.0/`
- System PATH

Check with:
```powershell
where maidos_shared.dll
```

### OpenSSL Not Found (Linux)

Install OpenSSL development headers:
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora
sudo dnf install openssl-devel
```

## Support

- **Documentation**: https://docs.rs/maidos-shared
- **Issues**: https://github.com/maidos/maidos-shared/issues
- **Discussions**: https://github.com/maidos/maidos-shared/discussions
