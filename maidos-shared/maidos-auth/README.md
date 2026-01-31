# maidos-auth

> Capability-based 認證庫 | HMAC-SHA256 Token | 策略引擎

[![crates.io](https://img.shields.io/crates/v/maidos-auth.svg)](https://crates.io/crates/maidos-auth)
[![docs.rs](https://docs.rs/maidos-auth/badge.svg)](https://docs.rs/maidos-auth)

## 功能

- ✅ Capability-based Access Control
- ✅ HMAC-SHA256 Token 簽發/驗證
- ✅ 18 種預定義權限
- ✅ 策略引擎 (Policy Engine)
- ✅ Token 儲存 (In-Memory Store)
- ✅ C FFI 支援

## 使用

```rust
use maidos_auth::{CapabilitySet, Capability, TokenIssuer};
use std::time::Duration;

// 建立 Token 簽發器
let secret = b"your-secret-key";
let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));

// 簽發 Token
let caps = CapabilitySet::new()
    .grant(Capability::LlmChat)
    .grant(Capability::LlmComplete);
let token = issuer.issue(caps)?;

// 驗證 Token
let verified = issuer.verify(token.as_str())?;
assert!(verified.has(Capability::LlmChat));
```

## 權限類型

| 權限 | 說明 |
|------|------|
| `LlmChat` | LLM 對話 |
| `LlmComplete` | LLM 補全 |
| `LlmEmbed` | LLM 嵌入 |
| `ConfigRead` | 讀取配置 |
| `ConfigWrite` | 寫入配置 |
| `BusPublish` | 發布事件 |
| `BusSubscribe` | 訂閱事件 |
| `Admin` | 管理權限 |
| ... | 共 18 種 |

## 策略引擎

```rust
use maidos_auth::{PolicyEngine, PolicyRule, Condition, Decision};

let mut engine = PolicyEngine::new(Decision::Deny);

engine.add_rule(PolicyRule::new("allow-chat")
    .capability(Capability::LlmChat)
    .condition(Condition::eq("role", "user"))
    .decision(Decision::Allow));

let ctx = PolicyContext::new()
    .set("role", "user");

let decision = engine.evaluate(&ctx);
```

## FFI

```c
MaidosToken* token = maidos_auth_create_token(caps, 3600, secret);
bool valid = maidos_auth_verify_token(token_str, secret);
maidos_auth_free_token(token);
```

## License

MIT
