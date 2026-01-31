# MAIDOS Shared Core - 架構設計文檔

> **版本**: 0.1.0  
> **更新日期**: 2026-01-04  
> **狀態**: Production Ready

---

## 概述

MAIDOS Shared Core 是一套高效能 Rust 核心庫，提供 AI 應用程式所需的基礎設施模組。設計目標是提供零成本抽象、類型安全、跨語言互操作的基礎服務。

```
┌─────────────────────────────────────────────────────────────┐
│                    MAIDOS Ecosystem                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │ Config  │  │  Auth   │  │   Bus   │  │   LLM   │        │
│  │         │  │         │  │         │  │         │        │
│  │  TOML   │  │ Token   │  │ Pub/Sub │  │ Multi-  │        │
│  │  Loader │  │ Issuer  │  │   TCP   │  │ Provider│        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
│       │            │            │            │              │
│       └────────────┴─────┬──────┴────────────┘              │
│                          │                                   │
│                    ┌─────┴─────┐                             │
│                    │    FFI    │                             │
│                    │   Layer   │                             │
│                    └─────┬─────┘                             │
│                          │                                   │
│              ┌───────────┼───────────┐                       │
│              │           │           │                       │
│         ┌────┴────┐ ┌────┴────┐ ┌────┴────┐                 │
│         │   C#    │ │ Python  │ │  Node   │                 │
│         │ Binding │ │ Binding │ │ Binding │                 │
│         └─────────┘ └─────────┘ └─────────┘                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 模組架構

### 1. maidos-config

**職責**: 配置管理、TOML 解析、環境變數展開、熱重載

```rust
┌─────────────────────────────────────┐
│            MaidosConfig             │
├─────────────────────────────────────┤
│ + from_file(path) -> Result         │
│ + from_str(toml) -> Result          │
│ + auth() -> &AuthConfig             │
│ + llm() -> &LlmConfig               │
│ + bus() -> &BusConfig               │
│ + provider(name) -> Option          │
│ + default_provider() -> Option      │
│ + schema() -> &ConfigSchema         │
├─────────────────────────────────────┤
│              Loader                  │
│ + load_with_env() -> Result         │
│ + expand_env_vars()                 │
├─────────────────────────────────────┤
│              Watcher                 │
│ + watch(path, callback)             │
│ + stop()                            │
└─────────────────────────────────────┘
```

**配置結構**:
```toml
[maidos]
version = "1.0"

[auth]
secret_key = "..."
token_ttl = 3600

[llm]
default_provider = "openai"
budget_daily = 10.0

[llm.providers.openai]
model = "gpt-4o"
api_key = "${OPENAI_API_KEY}"

[bus]
endpoint = "tcp://127.0.0.1:9000"
```

---

### 2. maidos-auth

**職責**: 能力認證、令牌發行/驗證、HMAC-SHA256 簽名

```rust
┌─────────────────────────────────────┐
│            TokenIssuer              │
├─────────────────────────────────────┤
│ + new(secret, ttl) -> Self          │
│ + issue(caps) -> Result<Token>      │
│ + issue_with_subject(caps, sub)     │
│ + verify(token_str) -> Result       │
│ + check(token, cap) -> Result<bool> │
│ + check_all(token, caps) -> Result  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│           CapabilitySet             │
├─────────────────────────────────────┤
│ + empty() -> Self                   │
│ + grant(cap)                        │
│ + revoke(cap)                       │
│ + has(cap) -> bool                  │
│ + has_any(caps) -> bool             │
│ + has_all(caps) -> bool             │
│ + iter() -> impl Iterator           │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│            Capability               │
├─────────────────────────────────────┤
│ LlmChat, LlmVision, LlmAudio        │
│ FileRead, FileWrite, FileDelete     │
│ ShellExec, NetworkAccess            │
│ EventPublish, EventSubscribe        │
│ ConfigRead, ConfigWrite             │
│ TokenIssue, TokenRevoke             │
│ PluginLoad, PluginUnload            │
│ SystemInfo, SystemControl           │
└─────────────────────────────────────┘
```

**令牌格式**:
```
<base64(payload)>.<base64(hmac_signature)>

Payload = {
    caps: u32,           // Bitmask
    exp: u64,            // Unix timestamp
    iat: u64,            // Issued at
    sub: Option<String>, // Subject
}
```

---

### 3. maidos-bus

**職責**: 事件匯流排、Pub/Sub、TCP 傳輸、MessagePack 序列化

```rust
┌─────────────────────────────────────┐
│              Event                  │
├─────────────────────────────────────┤
│ + id: String                        │
│ + topic: String                     │
│ + source: String                    │
│ + timestamp: u64                    │
│ + payload: Vec<u8>                  │
├─────────────────────────────────────┤
│ + new(topic, source, payload)       │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│            Publisher                │
├─────────────────────────────────────┤
│ + new(config) -> Self               │
│ + start() -> Result                 │
│ + publish(event) -> Result          │
│ + stop() -> Result                  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│            Subscriber               │
├─────────────────────────────────────┤
│ + new(config) -> Self               │
│ + connect() -> Result               │
│ + subscribe(topics) -> Result       │
│ + recv() -> Result<Event>           │
│ + disconnect() -> Result            │
└─────────────────────────────────────┘
```

**Topic 命名規範**:
```
<module>.<submodule>.<action>

Examples:
- system.status
- llm.chat.request
- auth.token.issued
- bus.subscriber.connected
```

---

### 4. maidos-llm

**職責**: 統一 LLM API、多供應商支持、Streaming

```rust
┌─────────────────────────────────────┐
│         CompletionRequest           │
├─────────────────────────────────────┤
│ + model: String                     │
│ + messages: Vec<Message>            │
│ + system: Option<String>            │
│ + temperature: Option<f32>          │
│ + max_tokens: Option<u32>           │
│ + stream: bool                      │
│ + stop: Option<Vec<String>>         │
│ + top_p: Option<f32>                │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│             Message                 │
├─────────────────────────────────────┤
│ + role: Role                        │
│ + content: String                   │
├─────────────────────────────────────┤
│ + user(content) -> Self             │
│ + assistant(content) -> Self        │
│ + system(content) -> Self           │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│         Provider Trait              │
├─────────────────────────────────────┤
│ + complete(req) -> Result           │
│ + complete_stream(req) -> Stream    │
└─────────────────────────────────────┘

Implementations:
├── OpenAiProvider
├── AnthropicProvider
└── OllamaProvider
```

---

## FFI 設計

### C ABI 導出

所有模組提供 C ABI 相容的函數導出：

```c
// Config
maidos_config_t* maidos_config_load(const char* path);
const char* maidos_config_get(maidos_config_t* cfg, const char* key);
void maidos_config_free(maidos_config_t* cfg);

// Auth
maidos_issuer_t* maidos_issuer_new(const uint8_t* secret, size_t len, uint64_t ttl);
char* maidos_issuer_issue(maidos_issuer_t* issuer, uint32_t caps);
int maidos_issuer_verify(maidos_issuer_t* issuer, const char* token);
void maidos_issuer_free(maidos_issuer_t* issuer);

// Bus
maidos_event_t* maidos_event_new(const char* topic, const char* source, ...);
int maidos_publisher_publish(maidos_publisher_t* pub, maidos_event_t* event);

// LLM
char* maidos_llm_complete(const char* provider, const char* request_json);
```

### C# P/Invoke

```csharp
[DllImport("maidos_shared")]
private static extern IntPtr maidos_config_load(string path);

[DllImport("maidos_shared")]
private static extern IntPtr maidos_issuer_new(
    byte[] secret, UIntPtr len, ulong ttl);
```

---

## 效能特性

### 基準測試結果

| 操作 | 延遲 | 吞吐量 |
|------|------|--------|
| Token 生成 | 700 ns | 1.4M ops/sec |
| Token 驗證 | 680 ns | 1.5M ops/sec |
| Config 解析 (標準) | 9 µs | 111K ops/sec |
| Event 創建 | 120 ns | 8.3M ops/sec |
| Event 創建 (64KB) | 1.7 µs | 36 GB/s |
| Message 創建 | 32-125 ns | 8-31M ops/sec |
| Request 序列化 | 525 ns | 1.9M ops/sec |

### 設計決策

1. **零拷貝序列化**: MessagePack 用於 Bus，減少記憶體分配
2. **Bitmask 權限**: CapabilitySet 使用 u32 bitmask，O(1) 權限檢查
3. **預分配緩衝區**: 避免動態分配在熱路徑
4. **Arc 共享配置**: 配置物件使用 Arc 共享，避免重複載入

---

## 錯誤處理

每個模組定義獨立的錯誤類型：

```rust
// maidos-config
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    EnvVarNotSet(String),
    ValidationError(String),
}

// maidos-auth
pub enum AuthError {
    InvalidSecret,
    InvalidToken,
    TokenExpired,
    InsufficientCapabilities,
    SignatureVerificationFailed,
}

// maidos-bus
pub enum BusError {
    InvalidTopic(String),
    ConnectionFailed(String),
    SerializationError(String),
    ChannelClosed,
}

// maidos-llm
pub enum LlmError {
    ProviderNotFound(String),
    RequestFailed(String),
    InvalidResponse(String),
    RateLimited,
    QuotaExceeded,
}
```

---

## 測試策略

### 測試金字塔

```
        ╱╲
       ╱  ╲         Integration Tests (15)
      ╱────╲        跨模組交互驗證
     ╱      ╲
    ╱────────╲      Unit Tests (122)
   ╱          ╲     單一模組功能驗證
  ╱────────────╲
 ╱              ╲   Benchmark Tests (30+)
╱────────────────╲  效能基線建立
```

### 覆蓋範圍

- **maidos-config**: 19 單元測試
- **maidos-auth**: 21 單元測試
- **maidos-bus**: 25 單元測試
- **maidos-llm**: 57 單元測試
- **整合測試**: 15 跨模組測試
- **基準測試**: 4 套件 30+ 測試

---

## 部署指南

### 編譯

```bash
# Debug 版本
cargo build

# Release 版本 (優化)
cargo build --release

# 生成動態庫
cargo build --release --lib
```

### 輸出產物

```
target/release/
├── libmaidos_config.so    # Linux
├── libmaidos_config.dylib # macOS
├── maidos_config.dll      # Windows
├── libmaidos_auth.so
├── libmaidos_bus.so
└── libmaidos_llm.so
```

### 整合到 C# 專案

1. 複製動態庫到專案目錄
2. 引用 `MaidosShared.csproj`
3. 使用 P/Invoke wrapper 類

```csharp
using MaidosShared;

var config = MaidosConfig.Load("config.toml");
var issuer = new TokenIssuer(secretKey, ttl);
var token = issuer.Issue(Capability.LlmChat | Capability.FileRead);
```

---

## 版本歷史

| 版本 | 日期 | 變更 |
|------|------|------|
| 0.1.0 | 2026-01-04 | 初始版本，4 模組 + C# 綁定 |

---

## 授權

Proprietary - MAIDOS Project

---

*Generated by Code-QC v2.1B3 Internal*
