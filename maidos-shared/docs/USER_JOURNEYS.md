# maidos-shared — User Journeys

## J-001: Add maidos-shared as a Dependency

**Persona:** MAIDOS product developer starting a new product or feature.

1. Developer opens their product's `Cargo.toml`.
2. Adds `maidos-shared` with desired feature flags (e.g., `features = ["auth", "log"]`).
3. Runs `cargo check` to verify resolution.
4. Imports the required modules: `use maidos_shared::auth::*;`
5. Builds successfully with only the selected sub-crates compiled.

**Success:** The developer compiles with zero unused dependency warnings and only the
needed sub-crate code is included.

---

## J-002: Configure Authentication

**Persona:** Developer integrating OAuth2 login into a MAIDOS product.

1. Developer enables the `auth` feature in `Cargo.toml`.
2. Creates an `AuthConfig` with provider, client ID, and redirect URI.
3. Calls `AuthClient::new(config)` to initialize the client.
4. Invokes `client.authenticate()` which opens the browser flow or reads cached tokens.
5. Receives an `AuthToken` with access token, refresh token, and expiry.
6. On subsequent calls, `client.ensure_valid_token()` auto-refreshes if needed.

**Success:** User authenticates once; tokens persist and refresh transparently.

---

## J-003: Use LLM Client via Ollama

**Persona:** Developer adding AI-assisted features to a MAIDOS product.

1. Developer enables the `llm` feature in `Cargo.toml`.
2. Constructs an `OllamaClient` with endpoint URL and model name.
3. Calls `client.generate(prompt, options)` with a text prompt.
4. Receives a streamed or buffered response.
5. Handles errors: connection refused (Ollama not running), timeout, model not found.

**Success:** LLM responses arrive within configured timeout; all data stays local.

---

## J-004: Set Up Structured Logging

**Persona:** Developer instrumenting a MAIDOS product for observability.

1. Developer enables the `log` feature.
2. Calls `maidos_log::init(LogConfig { level, format, output })` at application start.
3. Uses `tracing` macros (`info!`, `warn!`, `error!`) throughout the codebase.
4. Log output appears in the configured sink (stdout, file, or both).
5. In production builds, log level is configurable at runtime via config reload.

**Success:** Structured JSON logs flow to the designated output with correct levels.

---

## J-005: Use the Message Bus

**Persona:** Developer connecting components within a MAIDOS product.

1. Developer enables the `bus` feature.
2. Creates a `MessageBus` instance (in-process or networked mode).
3. Subscribes to a topic: `bus.subscribe("driver.update", handler)`.
4. Another component publishes: `bus.publish("driver.update", payload)`.
5. The handler receives the message with guaranteed ordering per topic.
6. On shutdown, `bus.close()` drains pending messages before teardown.

**Success:** Components communicate asynchronously without direct coupling.
