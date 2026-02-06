# maidos-shared — Architecture Document

## 1. High-Level Architecture

```
+-------------------------------------------------------+
|                   MAIDOS Products                      |
|  Driver  |   IME   |  Forge  |  CodeQC  |  Others    |
+----------+---------+---------+----------+-------------+
           |         |         |          |
           v         v         v          v
+-------------------------------------------------------+
|               maidos-shared (workspace)                |
|  +----------+  +---------+  +----------+  +---------+ |
|  |  auth    |  |   bus   |  |  config  |  |   log   | |
|  +----------+  +---------+  +----------+  +---------+ |
|  +----------+  +---------+  +----------+  +---------+ |
|  |  chain   |  | google  |  |   llm    |  |   p2p   | |
|  +----------+  +---------+  +----------+  +---------+ |
|  +----------+                                         |
|  |  social  |                                         |
|  +----------+                                         |
+-------------------------------------------------------+
```

## 2. Dependency Graph Between Sub-Crates

```
maidos-auth ──────> maidos-config (reads auth provider settings)
maidos-auth ──────> maidos-log   (logs auth events)
maidos-bus ───────> maidos-log   (logs bus lifecycle)
maidos-google ───-> maidos-auth  (uses OAuth2 tokens)
maidos-google ───-> maidos-config
maidos-llm ───────> maidos-config (reads Ollama endpoint)
maidos-llm ───────> maidos-log
maidos-p2p ───────> maidos-config (reads peer settings)
maidos-p2p ───────> maidos-log
maidos-social ───-> maidos-auth  (uses API tokens)
maidos-social ───-> maidos-config
maidos-chain ────-> maidos-log
```

## 3. Layering Rules

1. **maidos-config** and **maidos-log** are leaf dependencies — they depend on no other
   sub-crate within the workspace.
2. **maidos-auth** depends only on config and log.
3. Higher-level sub-crates (google, social, llm, p2p) may depend on auth, config, and log
   but never on each other.
4. **maidos-bus** is independent of auth and can be used standalone.
5. Circular dependencies between sub-crates are forbidden.

## 4. Async Runtime

All async sub-crates target **tokio** as the runtime. The workspace does not bundle a
runtime; consumers provide `#[tokio::main]` or equivalent. Sub-crates expose async
functions and return `impl Future` or `impl Stream` where appropriate.

## 5. Data Flow Example (LLM Query)

```
Product Code
    |
    v
maidos-llm::OllamaClient::generate(prompt)
    |
    +--> maidos-config: read endpoint URL, model name, timeout
    +--> maidos-log: trace request start
    |
    v
HTTP POST to Ollama /api/generate
    |
    v
Stream<GenerateResponse> returned to caller
    |
    +--> maidos-log: trace request completion
```

## 6. Build Architecture

- **Workspace root** `Cargo.toml` declares all members.
- Each sub-crate has its own `Cargo.toml` with independent versioning.
- CI builds the entire workspace; individual sub-crates are publishable to crates.io.
- Feature flags at the root level control which sub-crates are compiled.

## 7. Testing Architecture

| Layer | Scope | Tool |
|-------|-------|------|
| Unit | Per-function, mocked dependencies | `cargo test` |
| Integration | Sub-crate interaction | `cargo test --test integration` |
| Cross-product | Verify against consumer crates | CI matrix with pinned versions |
