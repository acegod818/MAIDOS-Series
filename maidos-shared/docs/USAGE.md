# MAIDOS Shared Core - 使用說明書

> 版本：0.2.0
> 日期：2026-01-09
> 授權：MIT

---

## 目錄

0. [發布包選擇指南](#0-發布包選擇指南)
1. [概述](#1-概述)
2. [安裝方式](#2-安裝方式)
3. [快速開始](#3-快速開始)
4. [模組詳解](#4-模組詳解)
5. [FFI 綁定](#5-ffi-綁定)
6. [最佳實踐](#6-最佳實踐)
7. [故障排除](#7-故障排除)
8. [v0.2.0 新功能](#8-v020-新功能)

---

## 0. 發布包選擇指南

### 0.1 發布包一覽

| 包名 | 大小 | 內容 | 適用對象 |
|------|-----:|------|----------|
| `maidos-shared-x.x.x-source.zip` | ~200 KB | 純源碼 | Rust 開發者、想自己編譯的人 |
| `maidos-shared-x.x.x-{os}-{arch}.zip` | ~3 MB | 預編譯動態庫 + 頭文件 | C/C++ 開發者 |
| `MaidosShared.x.x.x.nupkg` | ~3 MB | NuGet 套件 | C# / .NET 開發者 |
| `maidos-shared-x.x.x-full.zip` | ~6.5 MB | 以上全部 | 不確定要哪個就下這個 |

### 0.2 如何選擇？

```
你用什麼語言開發？
│
├─ Rust
│   └─ 下載: source.zip 或直接 cargo add
│
├─ C / C++
│   └─ 下載: {os}-{arch}.zip (預編譯版)
│       └─ 沒有你的 OS？下載 source.zip 自己編譯
│
├─ C# / .NET
│   └─ 下載: .nupkg 或 dotnet add package MaidosShared
│
├─ Python / Go / 其他
│   └─ 下載: {os}-{arch}.zip，透過 FFI 調用
│
└─ 不確定
    └─ 下載: full.zip (包含所有版本)
```

### 0.3 OS 相容性

| 作業系統 | 架構 | 預編譯版 | 源碼編譯 |
|----------|------|:--------:|:--------:|
| Linux | x86_64 | ✅ 提供 | ✅ |
| Linux | ARM64 | ❌ 需自編譯 | ✅ |
| macOS | x86_64 | ❌ 需自編譯 | ✅ |
| macOS | ARM64 (M1/M2) | ❌ 需自編譯 | ✅ |
| Windows | x86_64 | ❌ 需自編譯 | ✅ |

**沒有預編譯版？** 下載 `source.zip`，然後：

```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 編譯
cd maidos-shared-0.1.0-source
cargo build --release

# 產物位置
ls target/release/*.so      # Linux
ls target/release/*.dylib   # macOS
ls target/release/*.dll     # Windows
```

### 0.4 發布包內容對照

#### source.zip (源碼版)
```
maidos-shared-0.1.0-source/
├── maidos-config/      # 配置管理模組源碼
├── maidos-auth/        # 認證模組源碼
├── maidos-bus/         # 事件總線模組源碼
├── maidos-llm/         # LLM 模組源碼
├── bindings/csharp/    # C# 綁定源碼
├── include/maidos.h    # C 頭文件
├── tests/              # 測試
├── benches/            # 效能基準
├── examples/           # 範例程式
├── Cargo.toml          # Rust 建置配置
└── README.md
```

#### {os}-{arch}.zip (預編譯版)
```
maidos-shared-0.1.0-linux-x64/
├── lib/
│   ├── libmaidos_config.so
│   ├── libmaidos_auth.so
│   ├── libmaidos_bus.so
│   └── libmaidos_llm.so
├── include/
│   └── maidos.h        # C API 頭文件
├── USAGE.md            # 使用說明
└── SPEC-Shared-Core.md # 技術規格（位於 /documentation）
```

#### .nupkg (NuGet 版)
```
MaidosShared.0.1.0.nupkg
├── lib/net8.0/         # C# 綁定類別
│   ├── MaidosConfig.cs
│   ├── MaidosAuth.cs
│   ├── MaidosBus.cs
│   └── MaidosLlm.cs
└── runtimes/{os}-{arch}/native/
    └── *.so / *.dll / *.dylib
```

---

## 1. 概述

### 1.1 什麼是 MAIDOS Shared Core？

MAIDOS Shared Core 是一套 Rust 跨語言共享核心庫，提供：

- **maidos-config**: 配置管理（TOML 解析、環境變數、熱重載）
- **maidos-auth**: 認證授權（Capability Token、策略引擎）
- **maidos-bus**: 事件總線（ZeroMQ Pub/Sub）
- **maidos-llm**: LLM 抽象層（13 種 Provider、路由、預算）

### 1.2 支援的語言

| 語言 | 綁定方式 | 說明 |
|------|----------|------|
| Rust | 原生 | `cargo add maidos-*` |
| C/C++ | FFI | `#include "maidos.h"` |
| C# | P/Invoke | NuGet 套件 |
| Python | FFI | 透過 cffi/ctypes |
| 其他 | FFI | 任何支援 C ABI 的語言 |

### 1.3 系統需求

- **Rust**: 1.75+
- **OS**: Linux, macOS, Windows
- **Arch**: x86_64, ARM64

---

## 2. 安裝方式

### 2.1 Rust (crates.io)

```toml
# Cargo.toml
[dependencies]
maidos-config = "0.1"
maidos-auth = "0.1"
maidos-bus = "0.1"
maidos-llm = "0.1"
```

### 2.2 從源碼編譯

```bash
# 克隆倉庫
git clone https://github.com/maidos/maidos-shared.git
cd maidos-shared

# 編譯
cargo build --release

# 產生動態庫
ls target/release/*.so      # Linux
ls target/release/*.dylib   # macOS
ls target/release/*.dll     # Windows
```

### 2.3 C/C++ 鏈接

```bash
# 編譯你的程式
gcc -o myapp myapp.c -L./lib -lmaidos_shared -I./include

# 設置動態庫路徑
export LD_LIBRARY_PATH=./lib:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=./lib:$DYLD_LIBRARY_PATH  # macOS
```

### 2.4 C# NuGet

```bash
dotnet add package MaidosShared
```

---

## 3. 快速開始

### 3.1 配置管理

```rust
use maidos_config::MaidosConfig;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 載入配置
    let config = MaidosConfig::load(Path::new("config.toml"))?;
    
    // 讀取值
    println!("Provider: {}", config.llm().default_provider);
    
    // 熱重載
    let _handle = config.watch(|_| {
        println!("Config changed!");
    })?;
    
    Ok(())
}
```

配置檔案格式：

```toml
# config.toml
[maidos]
version = "1.0"

[llm]
default_provider = "ollama"
budget_daily = 10.0

[llm.providers.ollama]
base_url = "http://localhost:11434"
model = "llama3"

[llm.providers.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4o"

[bus]
endpoint = "tcp://127.0.0.1:5555"

[auth]
token_ttl = 3600
secret = "${AUTH_SECRET:-default_secret}"
```

### 3.2 認證授權

```rust
use maidos_auth::{CapabilitySet, Capability, TokenIssuer};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立簽發器
    let issuer = TokenIssuer::new(
        b"my-secret-key",
        Duration::from_secs(3600)
    );
    
    // 簽發 Token
    let caps = CapabilitySet::new()
        .grant(Capability::LlmChat)
        .grant(Capability::LlmComplete);
    let token = issuer.issue(caps)?;
    
    println!("Token: {}", token.as_str());
    
    // 驗證 Token
    let verified = issuer.verify(token.as_str())?;
    assert!(verified.has(Capability::LlmChat));
    
    Ok(())
}
```

### 3.3 事件總線

```rust
use maidos_bus::{Publisher, Subscriber, Event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 發布者
    let mut publisher = Publisher::bind("tcp://127.0.0.1:5555")?;
    publisher.start().await?;
    
    // 訂閱者
    let mut subscriber = Subscriber::connect("tcp://127.0.0.1:5555")?;
    subscriber.subscribe("events.*")?;
    subscriber.start().await?;
    
    // 發布事件
    let event = Event::new("events.test", "my-service", vec![1, 2, 3])?;
    publisher.publish(event).await?;
    
    // 接收事件
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    if let Some(event) = subscriber.try_recv().await? {
        println!("Received: {} from {}", event.topic, event.source);
    }
    
    Ok(())
}
```

### 3.4 LLM 請求

```rust
use maidos_llm::{create_provider, ProviderType, CompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立 Provider (本地 Ollama)
    let provider = create_provider(ProviderType::Ollama, None, None)?;
    
    // 簡單請求
    let request = CompletionRequest::quick("Why is the sky blue?");
    let response = provider.complete(request).await?;
    println!("{}", response.text);
    
    // 對話請求
    let request = CompletionRequest::builder()
        .model("llama3")
        .message(Message::system("You are a helpful assistant."))
        .message(Message::user("Hello!"))
        .temperature(0.7)
        .max_tokens(500)
        .build();
    
    let response = provider.complete(request).await?;
    println!("{}", response.text);
    
    Ok(())
}
```

---

## 4. 模組詳解

### 4.1 maidos-config

#### 環境變數展開

```toml
# 必須存在
api_key = "${API_KEY}"

# 帶預設值
timeout = "${TIMEOUT:-30}"

# 巢狀
url = "https://${HOST:-localhost}:${PORT:-8080}"
```

#### 熱重載

```rust
let handle = config.watch(|new_config| {
    // 配置已自動更新
    println!("New provider: {}", new_config.llm().default_provider);
})?;

// 手動重載
config.reload()?;

// 停止監聽
handle.stop();
```

### 4.2 maidos-auth

#### 權限類型

| 權限 | 值 | 說明 |
|------|-----|------|
| LlmChat | 0x0001 | LLM 對話 |
| LlmComplete | 0x0002 | LLM 補全 |
| LlmEmbed | 0x0004 | LLM 嵌入 |
| LlmVision | 0x0008 | LLM 視覺 |
| LlmFunction | 0x0010 | 函數調用 |
| ConfigRead | 0x0020 | 讀配置 |
| ConfigWrite | 0x0040 | 寫配置 |
| BusPublish | 0x0080 | 發布事件 |
| BusSubscribe | 0x0100 | 訂閱事件 |
| AuthIssue | 0x0200 | 簽發 Token |
| AuthRevoke | 0x0400 | 撤銷 Token |
| Admin | 0x8000 | 管理權限 |

#### 策略引擎

```rust
use maidos_auth::{PolicyEngine, PolicyRule, Condition, Decision, PolicyContext};

let mut engine = PolicyEngine::new(Decision::Deny);

// 添加規則
engine.add_rule(PolicyRule::new("allow-users")
    .capability(Capability::LlmChat)
    .condition(Condition::eq("role", "user"))
    .condition(Condition::lt("request_count", 100))
    .decision(Decision::Allow));

// 評估
let ctx = PolicyContext::new()
    .set("role", "user")
    .set("request_count", 50);

match engine.evaluate(&ctx) {
    Decision::Allow => println!("Allowed"),
    Decision::Deny => println!("Denied"),
}
```

### 4.3 maidos-bus

#### 主題格式

```
service.event.type     # 精確匹配
service.event.*        # 匹配 service.event.* 下所有
service.*              # 匹配 service.* 下所有
*                      # 匹配所有
```

#### 型別化事件

```rust
#[derive(Serialize, Deserialize)]
struct UserCreated {
    id: u64,
    name: String,
}

// 發布
let event = Event::with_data("users.created", "auth-service", &UserCreated {
    id: 123,
    name: "Alice".into(),
})?;

// 接收
let user: UserCreated = received_event.data()?;
```

### 4.4 maidos-llm

#### Provider 選擇

```rust
// 雲端 Provider (需 API Key)
let openai = create_provider(ProviderType::OpenAi, Some(api_key), None)?;
let anthropic = create_provider(ProviderType::Anthropic, Some(api_key), None)?;
let google = create_provider(ProviderType::Google, Some(api_key), None)?;

// 本地 Provider (無需 API Key)
let ollama = create_provider(ProviderType::Ollama, None, None)?;
let lmstudio = create_provider(ProviderType::LmStudio, None, None)?;

// 自定義 URL
let custom = create_provider(
    ProviderType::Ollama,
    None,
    Some("http://192.168.1.100:11434".into())
)?;
```

#### 路由器

```rust
use maidos_llm::{Router, RoutingStrategy};

let router = Router::builder()
    .add_provider("primary", openai, 10)
    .add_provider("fallback", ollama, 1)
    .strategy(RoutingStrategy::Fallback)
    .build();

// 自動選擇
let provider = router.select_provider()?;

// 記錄失敗
router.record_failure("primary");

// 健康狀態
for status in router.health_status() {
    println!("{}: {} ({}ms)", status.name, 
        if status.healthy { "✓" } else { "✗" },
        status.avg_latency_ms);
}
```

#### 預算控制

```rust
use maidos_llm::BudgetController;

let budget = BudgetController::builder()
    .daily_limit(10.0)
    .monthly_limit(100.0)
    .per_request_limit(0.50)
    .warning_threshold(0.8)
    .exceeded_action(ExceededAction::Block)
    .build();

// 檢查+記錄
if budget.check_budget("openai", estimated_cost)? {
    let response = provider.complete(request).await?;
    budget.record_usage("openai", actual_cost, 
        response.usage.prompt_tokens,
        response.usage.completion_tokens)?;
}

// 查看狀態
let status = budget.get_global_status()?;
println!("Today: ${:.2} / ${:.2}", status.daily_usage, status.daily_limit);
```

---

## 5. FFI 綁定

### 5.1 C 範例

```c
#include "maidos.h"
#include <stdio.h>

int main() {
    // 載入配置
    MaidosConfig* config = maidos_config_load("config.toml");
    if (!config) {
        printf("Error: %s\n", maidos_last_error());
        return 1;
    }
    
    // 讀取值
    const char* provider = maidos_config_get_string(config, "llm.default_provider");
    printf("Provider: %s\n", provider);
    maidos_string_free(provider);
    
    // 建立 LLM
    MaidosLlmProvider* llm = maidos_llm_create("ollama", NULL, NULL);
    
    // 請求
    MaidosLlmResponse response;
    if (maidos_llm_complete(llm, "Hello!", &response) == MAIDOS_OK) {
        printf("Response: %s\n", response.text);
        printf("Tokens: %u\n", response.total_tokens);
        maidos_llm_response_free(&response);
    }
    
    // 清理
    maidos_llm_free(llm);
    maidos_config_free(config);
    
    return 0;
}
```

### 5.2 C# 範例

```csharp
using MaidosShared;

class Program
{
    static async Task Main()
    {
        // 載入配置
        using var config = MaidosConfig.Load("config.toml");
        Console.WriteLine($"Provider: {config.GetString("llm.default_provider")}");
        
        // 建立 LLM
        using var llm = new MaidosLlm("ollama");
        
        // 請求
        var response = await llm.CompleteAsync("Hello!");
        Console.WriteLine($"Response: {response.Text}");
        Console.WriteLine($"Tokens: {response.TotalTokens}");
    }
}
```

---

## 6. 最佳實踐

### 6.1 配置管理

```
✅ 敏感資訊使用環境變數
✅ 提供合理的預設值
✅ 驗證配置完整性
✅ 使用熱重載避免重啟

❌ 硬編碼 API Key
❌ 忽略配置驗證錯誤
```

### 6.2 認證授權

```
✅ 最小權限原則
✅ 定期輪換密鑰
✅ 設置合理的 TTL
✅ 記錄授權決策

❌ 使用弱密鑰
❌ 過長的 Token 有效期
❌ 授予過多權限
```

### 6.3 事件總線

```
✅ 明確的主題命名規範
✅ 處理訂閱者斷線
✅ 設置合理的超時
✅ 使用型別化事件

❌ 過大的事件負載
❌ 忽略發布失敗
```

### 6.4 LLM 請求

```
✅ 設置預算限制
✅ 使用路由器做負載均衡
✅ 處理 Rate Limit
✅ 記錄 Token 使用量

❌ 無限制的 max_tokens
❌ 忽略錯誤處理
❌ 不設預算控制
```

---

## 7. 故障排除

### 7.1 常見錯誤

| 錯誤碼 | 說明 | 解決方案 |
|--------|------|----------|
| `MAIDOS_ERR_NULL_POINTER` | 空指針 | 檢查參數非 NULL |
| `MAIDOS_ERR_INVALID_UTF8` | 無效 UTF-8 | 確保字串編碼正確 |
| `MAIDOS_ERR_NOT_FOUND` | 找不到資源 | 檢查路徑/鍵名 |
| `MAIDOS_ERR_AUTH` | 認證失敗 | 檢查 Token/API Key |
| `MAIDOS_ERR_NETWORK` | 網路錯誤 | 檢查連線/防火牆 |
| `MAIDOS_ERR_PROVIDER` | Provider 錯誤 | 檢查 Provider 狀態 |
| `MAIDOS_ERR_BUDGET` | 預算超限 | 增加預算或等待重置 |

### 7.2 除錯技巧

```rust
// 啟用追蹤
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// 檢查 Provider 資訊
let info = provider.info();
println!("Provider: {} ({})", info.name, info.default_model);

// 檢查預算狀態
let status = budget.get_global_status()?;
println!("Usage: {:.1}%", status.usage_percentage * 100.0);
```

### 7.3 效能調優

```rust
// 重用 Provider（線程安全）
lazy_static! {
    static ref PROVIDER: Arc<dyn LlmProvider> = 
        create_provider(ProviderType::Ollama, None, None).unwrap();
}

// 批量請求
let futures = prompts.iter()
    .map(|p| provider.complete(CompletionRequest::quick(p)));
let results = futures::future::join_all(futures).await;

// 調整超時
let request = CompletionRequest::builder()
    .timeout(Duration::from_secs(60))
    .build();
```

---

## 附錄

### A. 環境變數

| 變數 | 說明 | 預設值 |
|------|------|--------|
| `MAIDOS_CONFIG_PATH` | 配置檔路徑 | `./config.toml` |
| `MAIDOS_LOG_LEVEL` | 日誌級別 | `info` |
| `OPENAI_API_KEY` | OpenAI API Key | - |
| `ANTHROPIC_API_KEY` | Anthropic API Key | - |
| `GOOGLE_API_KEY` | Google API Key | - |

### B. 效能基準

| 操作 | 延遲 (µs) | 說明 |
|------|----------:|------|
| Token 簽發 | ~5 | HMAC-SHA256 |
| Token 驗證 | ~3 | 簽名比對 |
| Config 讀取 | ~0.1 | RwLock 讀鎖 |
| Event 序列化 | ~2 | MessagePack |
| FFI 調用開銷 | ~0.5 | C ABI |

---

*MAIDOS Shared Core v0.2.0 - 使用說明書*

---

## 8. v0.2.0 新功能

### 8.1 Tier 2 Cloud Providers

v0.2.0 新增 5 個雲端 Provider：

```rust
use maidos_llm::{create_provider, ProviderType};

// Mistral (Vision via Pixtral)
let mistral = create_provider(
    ProviderType::Mistral, 
    Some("your-mistral-api-key"),
    None
)?;

// Azure OpenAI (Deployment-based)
let azure = create_provider(
    ProviderType::AzureOpenAi,
    Some("your-azure-api-key"),
    Some("https://your-resource.openai.azure.com/your-deployment")
)?;

// Cohere (RAG support)
let cohere = create_provider(
    ProviderType::Cohere,
    Some("your-cohere-api-key"),
    None
)?;

// Together AI (Open-source models)
let together = create_provider(
    ProviderType::Together,
    Some("your-together-api-key"),
    None
)?;

// Replicate (Async polling)
let replicate = create_provider(
    ProviderType::Replicate,
    Some("your-replicate-api-token"),
    None
)?;
```

### 8.2 統一 Streaming 介面

```rust
use maidos_llm::{CompletionRequest, streaming::StreamChunk};
use futures::StreamExt;

// 啟用 streaming
let request = CompletionRequest::builder()
    .model("gpt-4o")
    .message(Message::user("Tell me a story"))
    .stream(true)
    .build();

// 處理 stream
let mut stream = provider.complete_stream(request).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(StreamChunk { delta, is_final, .. }) => {
            print!("{}", delta);
            if is_final {
                println!("\n--- Done ---");
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 8.3 MaidosTool 跨 Provider 格式

```rust
use maidos_llm::tool::{MaidosTool, ToolParameter, ToProviderFormat};

// 定義工具（Provider 無關）
let tool = MaidosTool::new("get_weather", "Get current weather")
    .parameter(
        ToolParameter::string("location")
            .description("City name, e.g. 'Tokyo'")
            .required(true)
    )
    .parameter(
        ToolParameter::string("unit")
            .enum_values(vec!["celsius", "fahrenheit"])
    );

// 轉換為各 Provider 格式
let openai_format = tool.to_openai();       // OpenAI function calling
let anthropic_format = tool.to_anthropic(); // Anthropic tool use
let google_format = tool.to_google();       // Google function calling
let mistral_format = tool.to_mistral();     // Mistral (OpenAI-compatible)
let cohere_format = tool.to_cohere();       // Cohere native format
```

### 8.4 Vision/Tools 錯誤處理

```rust
use maidos_llm::LlmError;

match provider.complete(request).await {
    Err(LlmError::VisionNotSupported { provider, suggestion }) => {
        println!("Provider '{}' doesn't support vision.", provider);
        println!("Suggestion: {}", suggestion);
    }
    Err(LlmError::ToolsNotSupported { provider, suggestion }) => {
        println!("Provider '{}' doesn't support function calling.", provider);
        println!("Suggestion: {}", suggestion);
    }
    Ok(response) => println!("{}", response.text),
    Err(e) => eprintln!("Other error: {}", e),
}

// 檢查是否為能力錯誤
if error.is_capability_error() {
    // 切換到支援該功能的 Provider
}
```

### 8.5 Provider 能力一覽 (v0.2.0)

| Provider | Vision | Tools | Streaming | 備註 |
|----------|:------:|:-----:|:---------:|------|
| OpenAI | ✅ | ✅ | ✅ Real | |
| Anthropic | ✅ | ✅ | ✅ Real | |
| Google | ✅ | ✅ | ✅ Real | |
| DeepSeek | ❌ | ✅ | ✅ Real | |
| Groq | ✅ | ✅ | ✅ Real | |
| Mistral | ✅ | ✅ | ✅ Real | Pixtral models |
| Azure OpenAI | ✅ | ✅ | ✅ Real | Deployment-based |
| Cohere | ❌ | ✅ | 🔄 Fallback | RAG support |
| Together AI | ✅ | ✅ | ✅ Real | Llama Vision |
| Replicate | ✅ | ❌ | 🔄 Fallback | Async polling |
| Ollama | ✅ | ✅ | ✅ Real | |
| LM Studio | ✅ | ✅ | ✅ Real | |
| vLLM | ❌ | ❌ | ✅ Real | |
