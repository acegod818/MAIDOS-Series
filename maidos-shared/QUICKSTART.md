# MAIDOS Shared Core - 快速開始指南

> 5 分鐘內開始使用 MAIDOS Shared Core

---

## 安裝

### Rust (Cargo)

在 `Cargo.toml` 中添加依賴：

```toml
[dependencies]
maidos-config = { path = "path/to/maidos-shared/maidos-config" }
maidos-auth = { path = "path/to/maidos-shared/maidos-auth" }
maidos-bus = { path = "path/to/maidos-shared/maidos-bus" }
maidos-llm = { path = "path/to/maidos-shared/maidos-llm" }
```

### C# (.NET)

1. 編譯動態庫：
```bash
cd maidos-shared
cargo build --release
```

2. 複製 `.so/.dll/.dylib` 到專案目錄

3. 引用 C# 綁定：
```xml
<ItemGroup>
  <ProjectReference Include="path/to/MaidosShared.csproj" />
</ItemGroup>
```

---

## 快速範例

### 1. 配置載入

```rust
use maidos_config::MaidosConfig;

// 從字串載入
let config = MaidosConfig::from_str(r#"
[maidos]
version = "1.0"

[auth]
secret_key = "my-secret-key-32-bytes-long!!!"
token_ttl = 3600

[llm]
default_provider = "openai"

[llm.providers.openai]
model = "gpt-4o"
api_key = "sk-..."
"#)?;

// 訪問配置
let auth = config.auth();
println!("TTL: {} 秒", auth.token_ttl);

let provider = config.default_provider().unwrap();
println!("Model: {}", provider.model);
```

### 2. 令牌認證

```rust
use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use std::time::Duration;

// 創建 issuer
let issuer = TokenIssuer::new(
    b"my-secret-key-32-bytes-long!!!".to_vec(),
    Duration::from_secs(3600),
);

// 創建權限集
let mut caps = CapabilitySet::empty();
caps.grant(Capability::LlmChat);
caps.grant(Capability::FileRead);

// 發行令牌
let token = issuer.issue(caps)?;
println!("Token: {}", token.as_str());

// 驗證令牌
let verified = issuer.verify(token.as_str())?;
assert!(verified.capabilities().has(Capability::LlmChat));

// 權限檢查
let can_chat = issuer.check(token.as_str(), Capability::LlmChat)?;
assert!(can_chat);
```

### 3. LLM 請求

```rust
use maidos_llm::{CompletionRequest, Message};

// 構建請求
let request = CompletionRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        Message::user("What is 2 + 2?"),
    ],
    system: Some("You are a math tutor.".to_string()),
    temperature: Some(0.7),
    max_tokens: Some(1024),
    stream: false,
    stop: None,
    top_p: None,
};

// 序列化為 JSON
let json = serde_json::to_string(&request)?;
println!("{}", json);
```

### 4. 事件匯流排

```rust
use maidos_bus::{Event, PublisherConfig, SubscriberConfig};

// 創建事件
let event = Event::new(
    "system.status",      // topic
    "health-monitor",     // source
    b"OK".to_vec(),       // payload
)?;

println!("Event ID: {}", event.id);
println!("Topic: {}", event.topic);

// Publisher 配置
let pub_config = PublisherConfig {
    bind_addr: "127.0.0.1:9000".to_string(),
    channel_capacity: 1024,
    max_connections: 100,
};

// Subscriber 配置
let sub_config = SubscriberConfig {
    publisher_addr: "127.0.0.1:9000".to_string(),
    topics: vec!["system.*".to_string()],
    reconnect_delay_ms: 5000,
    auto_reconnect: true,
    buffer_capacity: 256,
};
```

---

## 完整整合範例

```rust
use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use maidos_bus::Event;
use maidos_config::MaidosConfig;
use maidos_llm::{CompletionRequest, Message};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 載入配置
    let config = MaidosConfig::from_str(r#"
[maidos]
version = "1.0"

[auth]
secret_key = "production-secret-key-32-bytes!"
token_ttl = 7200

[llm]
default_provider = "anthropic"

[llm.providers.anthropic]
model = "claude-sonnet-4-20250514"
api_key = "sk-ant-..."
"#)?;

    // 2. 創建認證系統
    let auth = config.auth();
    let issuer = TokenIssuer::new(
        auth.secret_key.as_bytes().to_vec(),
        Duration::from_secs(auth.token_ttl),
    );

    // 3. 發行服務令牌
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::EventPublish);
    let token = issuer.issue(caps)?;

    // 4. 驗證權限後構建 LLM 請求
    if issuer.check(token.as_str(), Capability::LlmChat)? {
        let provider = config.default_provider().unwrap();
        let request = CompletionRequest {
            model: provider.model.clone(),
            messages: vec![Message::user("Hello!")],
            system: None,
            temperature: Some(0.7),
            max_tokens: Some(1024),
            stream: false,
            stop: None,
            top_p: None,
        };

        // 5. 發布事件
        let event = Event::new(
            "llm.request.queued",
            "api-gateway",
            serde_json::to_vec(&request)?,
        )?;
        
        println!("Request queued: {}", event.id);
    }

    Ok(())
}
```

---

## 運行範例

```bash
# 基本配置
cargo run --example basic_config

# 認證令牌
cargo run --example auth_tokens

# LLM 對話
cargo run --example llm_chat

# 事件匯流排
cargo run --example bus_pubsub

# 完整整合
cargo run --example full_integration
```

---

## 運行測試

```bash
# 所有測試
cargo test --workspace

# 特定模組
cargo test -p maidos-auth

# 整合測試
cargo test --test integration
```

---

## 運行基準測試

```bash
# 所有基準
cargo bench

# 特定基準
cargo bench --bench auth_bench

# 快速模式
cargo bench -- --quick
```

---

## 下一步

- 閱讀 [ARCHITECTURE.md](ARCHITECTURE.md) 了解設計細節
- 查看 [examples/](examples/) 目錄獲取更多範例
- 運行 `cargo doc --open` 查看 API 文檔

---

## 常見問題

### Q: 如何處理環境變數？

```toml
[llm.providers.openai]
api_key = "${OPENAI_API_KEY}"
```

設置環境變數後載入配置即可。

### Q: 令牌過期怎麼辦？

```rust
let token = issuer.verify(token_str);
match token {
    Ok(t) => println!("Valid"),
    Err(AuthError::TokenExpired) => println!("Expired, please refresh"),
    Err(e) => println!("Error: {:?}", e),
}
```

### Q: 如何自定義權限組合？

```rust
let mut caps = CapabilitySet::empty();
caps.grant(Capability::LlmChat);
caps.grant(Capability::LlmVision);
caps.grant(Capability::FileRead);
// 根據需要添加更多...
```

---

*Happy Coding with MAIDOS!*
