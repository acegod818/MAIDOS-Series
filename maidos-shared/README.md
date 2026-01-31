# MAIDOS Shared Core

> 🦀 Rust 跨語言共享核心庫 | LLM 抽象層 + 認證 + 事件總線 + 配置管理

[![CI](https://github.com/user/maidos-shared/actions/workflows/ci.yml/badge.svg)](https://github.com/user/maidos-shared/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Code-QC: SS](https://img.shields.io/badge/Code--QC-SS-gold.svg)](./CODE-QC-REPORT.md)

---

## 📦 組件

| Crate | 描述 | crates.io |
|-------|------|-----------|
| `maidos-config` | TOML 配置管理、環境變數展開、熱重載 | [![](https://img.shields.io/crates/v/maidos-config.svg)](https://crates.io/crates/maidos-config) |
| `maidos-auth` | Capability-based 認證、HMAC-SHA256 Token | [![](https://img.shields.io/crates/v/maidos-auth.svg)](https://crates.io/crates/maidos-auth) |
| `maidos-bus` | ZeroMQ 事件總線、Pub/Sub 模式 | [![](https://img.shields.io/crates/v/maidos-bus.svg)](https://crates.io/crates/maidos-bus) |
| `maidos-llm` | 多 LLM Provider 抽象、路由、預算控制 | [![](https://img.shields.io/crates/v/maidos-llm.svg)](https://crates.io/crates/maidos-llm) |

---

## 🚀 快速開始

### Rust

```toml
# Cargo.toml
[dependencies]
maidos-config = "0.1"
maidos-auth = "0.1"
maidos-bus = "0.1"
maidos-llm = "0.1"
```

```rust
use maidos_llm::{Message, Role, CompletionRequest, ProviderType, create_provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立 Provider
    let provider = create_provider(
        ProviderType::Ollama,
        None,  // 本地不需 API Key
        None,
    )?;

    // 建立請求
    let request = CompletionRequest::quick("Why is the sky blue?");

    // 執行
    let response = provider.complete(request).await?;
    println!("{}", response.text);

    Ok(())
}
```

### C / C++

```c
#include "maidos.h"

int main() {
    // 載入配置
    MaidosConfig* config = maidos_config_load("config.toml");
    
    // 建立 LLM Provider
    MaidosLlmProvider* llm = maidos_llm_create("ollama", NULL, NULL);
    
    // 發送請求
    const char* response = maidos_llm_complete(llm, "Hello, world!");
    printf("%s\n", response);
    
    // 清理
    maidos_llm_free(llm);
    maidos_config_free(config);
    return 0;
}
```

### C# / .NET

```csharp
using MaidosShared;

// 建立 Provider
using var llm = new MaidosLlm("ollama");

// 發送請求
string response = llm.Complete("Hello, world!");
Console.WriteLine(response);
```

---

## 🔌 支援的 LLM Provider

### 雲端 Tier 1 (需 API Key)

| Provider | 模型 | Vision | Function Calling |
|----------|------|:------:|:----------------:|
| OpenAI | GPT-4, GPT-4o, GPT-4o-mini | ✅ | ✅ |
| Anthropic | Claude 3.5/3 Opus/Sonnet/Haiku | ✅ | ✅ |
| Google | Gemini 1.5 Pro/Flash, 2.0 | ✅ | ✅ |
| DeepSeek | Chat, Coder, Reasoner (R1) | ❌ | ✅ |
| Groq | Llama 3.3, Mixtral, Gemma | ✅ | ✅ |

### 雲端 Tier 2 (v0.2.0 新增)

| Provider | 模型 | Vision | Function Calling |
|----------|------|:------:|:----------------:|
| Mistral | Large, Medium, Pixtral | ✅ | ✅ |
| Azure OpenAI | GPT-4o (Deployment) | ✅ | ✅ |
| Cohere | Command R+, R | ❌ | ✅ + RAG |
| Together AI | Llama, Mixtral, Qwen | ✅ | ✅ |
| Replicate | Llama 2, LLaVA | ✅ | ❌ |

### 本地 (無需 API Key)

| Provider | 預設 URL | 說明 |
|----------|----------|------|
| Ollama | `localhost:11434` | 本地模型運行 |
| LM Studio | `localhost:1234` | 桌面應用 |
| vLLM | `localhost:8000` | 高吞吐量服務 |

---

## 🛠️ 功能特性

### maidos-config
- ✅ TOML 配置解析
- ✅ 環境變數展開 (`${VAR}`, `${VAR:-default}`)
- ✅ Schema 驗證
- ✅ 熱重載 (File Watch)
- ✅ 執行緒安全

### maidos-auth
- ✅ Capability-based Access Control
- ✅ HMAC-SHA256 Token 簽發/驗證
- ✅ 18 種預定義權限
- ✅ 策略引擎 (Policy Engine)
- ✅ Token 儲存 (In-Memory Store)

### maidos-bus
- ✅ ZeroMQ Pub/Sub
- ✅ 主題過濾 (Topic Filtering)
- ✅ MessagePack 序列化
- ✅ 非同步 (Tokio)

### maidos-llm
- ✅ 13 種 Provider 支援 (10 雲端 + 3 本地)
- ✅ 統一 API 介面
- ✅ 6 種路由策略 (Priority, RoundRobin, Weighted, Cost, Speed, Fallback)
- ✅ 預算控制 (Daily/Monthly/Per-Request)
- ✅ Vision 支援
- ✅ Function Calling 支援
- ✅ 統一 Streaming 介面 (v0.2.0)
- ✅ MaidosTool 跨 Provider 格式轉換 (v0.2.0)

---

## 📁 專案結構

```
maidos-shared/
├── maidos-config/     # 配置管理
├── maidos-auth/       # 認證授權
├── maidos-bus/        # 事件總線
├── maidos-llm/        # LLM 抽象層
│   └── src/
│       ├── providers/
│       │   ├── cloud/     # 10 雲端 Provider
│       │   └── local/     # 3 本地 Provider
│       ├── streaming.rs   # 統一 Streaming (v0.2.0)
│       └── tool.rs        # MaidosTool 格式 (v0.2.0)
├── bindings/
│   └── csharp/        # C# P/Invoke 綁定
├── tests/             # 整合測試
├── benches/           # 效能基準
├── examples/          # 範例程式
└── include/           # C 頭文件
```

---

## 📊 品質保證

```
Grade SS - 卓越品質認證
├── 零 unwrap (生產代碼)
├── 零 Clippy 警告
├── 零 TODO/FIXME
├── 307 測試全過 (v0.2.0)
└── Code-QC v2.2B/C 合規
```

---

## 🔧 建置

### 需求
- Rust 1.75+
- CMake 3.20+ (C 綁定)
- .NET 8.0+ (C# 綁定)

### 編譯

```bash
# Rust
cargo build --release

# 產生動態庫
cargo build --release --lib

# 執行測試
cargo test --workspace

# 執行基準測試
cargo bench
```

---

## 📄 授權

MIT License - 詳見 [LICENSE](./LICENSE)

---

## 🤝 貢獻

歡迎 PR！請參閱 [CONTRIBUTING.md](./CONTRIBUTING.md)

---

*MAIDOS Shared Core - Zero Defects, Zero Fakes.*
