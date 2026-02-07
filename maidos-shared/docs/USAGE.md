# Usage Guide — maidos-shared

## Adding as Dependency

```toml
[dependencies]
maidos-config = { git = "https://github.com/maidos/maidos-shared", features = ["watch"] }
maidos-auth = { git = "https://github.com/maidos/maidos-shared" }
maidos-bus = { git = "https://github.com/maidos/maidos-shared" }
maidos-llm = { git = "https://github.com/maidos/maidos-shared" }
```

## maidos-config
```rust
use maidos_config::Config;
let config = Config::load("config.toml")?;
let value = config.get::<String>("app.name")?;
```

## maidos-auth
```rust
use maidos_auth::{Token, Verifier};
let token = Token::create(&secret, claims)?;
let verified = Verifier::verify(&secret, &token)?;
```

## maidos-bus
```rust
use maidos_bus::EventBus;
let bus = EventBus::new();
bus.subscribe("topic", |msg| { /* handle */ });
bus.publish("topic", payload)?;
```

## maidos-llm
```rust
use maidos_llm::Provider;
let provider = Provider::from_config(&config)?;
let response = provider.complete("prompt").await?;
```

## maidos-chain
```rust
use maidos_chain::Client;
let client = Client::connect(&rpc_url).await?;
let tx = client.send_transaction(params).await?;
```

## Feature Flags

| Crate | Flag | Description |
|-------|------|-------------|
| maidos-config | `watch` | File system hot-reload via notify |
| maidos-config | `default` | Basic config without watch |
