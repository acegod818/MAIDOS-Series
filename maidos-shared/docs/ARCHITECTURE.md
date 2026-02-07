# maidos-shared -- Architecture Document

| Field     | Value                  |
|-----------|------------------------|
| Product   | maidos-shared          |
| Version   | 0.2.0                  |
| Type      | Architecture Overview  |

## Workspace Structure

```
maidos-shared/
  Cargo.toml           # Workspace root
  maidos-config/       # Configuration management
  maidos-auth/         # Authentication (JWT/OAuth)
  maidos-bus/          # Event bus
  maidos-llm/          # LLM integration
  maidos-log/          # Structured logging
  maidos-social/       # Social connectors
  maidos-google/       # Google API
  maidos-p2p/          # P2P networking
  maidos-chain/        # Blockchain module
  tests/               # Integration tests
  examples/            # Usage examples
  benches/             # Benchmarks
  bindings/            # FFI bindings
  scripts/             # Utility scripts
  include/             # C header files
```

## Dependency Graph

```
maidos-config  <--- (all crates depend on config)
    |
    +--- maidos-log    <--- (most crates depend on log)
    |        |
    +--------+--- maidos-auth
    |        |
    +--------+--- maidos-bus
    |        |
    +--------+--- maidos-llm
    |        |
    +--------+--- maidos-social
    |        |
    +--------+--- maidos-google
    |        |
    +--------+--- maidos-p2p
    |        |
    +--------+--- maidos-chain
```

## Crate Responsibilities

| Crate          | Key Modules                | External Dependencies          |
|----------------|----------------------------|--------------------------------|
| maidos-config  | loader, schema, env        | toml, serde, directories       |
| maidos-auth    | jwt, oauth, session        | jsonwebtoken, oauth2, reqwest  |
| maidos-bus     | dispatcher, subscriber     | tokio, async-trait             |
| maidos-llm     | client, prompt, models     | reqwest, serde_json            |
| maidos-log     | logger, format, sink       | tracing, tracing-subscriber    |
| maidos-social  | twitter, discord, telegram | reqwest, serde                 |
| maidos-google  | drive, sheets, calendar    | google-apis, reqwest           |
| maidos-p2p     | discovery, transport       | libp2p, tokio                  |
| maidos-chain   | contract, wallet, rpc      | ethers, web3                   |

## Cross-Crate Communication

Crates communicate through well-defined Rust trait interfaces. The event bus (maidos-bus) provides async pub/sub messaging for loosely-coupled interaction between modules at runtime.

*maidos-shared ARCHITECTURE v0.2.0 -- CodeQC Gate C Compliant*
