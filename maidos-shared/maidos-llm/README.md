# maidos-llm

> 多 LLM Provider 抽象層 | 路由策略 | 預算控制

[![crates.io](https://img.shields.io/crates/v/maidos-llm.svg)](https://crates.io/crates/maidos-llm)
[![docs.rs](https://docs.rs/maidos-llm/badge.svg)](https://docs.rs/maidos-llm)

## 功能

- ✅ 8 種 Provider 支援 (5 雲端 + 3 本地)
- ✅ 統一 API 介面
- ✅ 6 種路由策略
- ✅ 預算控制 (Daily/Monthly/Per-Request)
- ✅ Vision 支援
- ✅ Function Calling 支援
- ✅ C FFI 支援

## 使用

```rust
use maidos_llm::{create_provider, ProviderType, CompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立 Provider
    let provider = create_provider(
        ProviderType::OpenAi,
        Some("sk-...".to_string()),
        None,
    )?;

    // 簡單請求
    let request = CompletionRequest::quick("Explain quantum computing");
    let response = provider.complete(request).await?;
    println!("{}", response.text);

    // 對話請求
    let request = CompletionRequest::builder()
        .model("gpt-4o")
        .message(Message::system("You are a helpful assistant."))
        .message(Message::user("Hello!"))
        .temperature(0.7)
        .max_tokens(1000)
        .build();
    
    let response = provider.complete(request).await?;
    println!("{}", response.text);

    Ok(())
}
```

## 支援的 Provider

### 雲端 (需 API Key)

| Provider | 建立方式 | 模型範例 |
|----------|----------|----------|
| OpenAI | `ProviderType::OpenAi` | gpt-4o, gpt-4o-mini |
| Anthropic | `ProviderType::Anthropic` | claude-sonnet-4-20250514 |
| Google | `ProviderType::Google` | gemini-1.5-pro |
| DeepSeek | `ProviderType::DeepSeek` | deepseek-chat |
| Groq | `ProviderType::Groq` | llama-3.3-70b-versatile |

### 本地 (無需 API Key)

| Provider | 建立方式 | 預設 URL |
|----------|----------|----------|
| Ollama | `ProviderType::Ollama` | localhost:11434 |
| LM Studio | `ProviderType::LmStudio` | localhost:1234 |
| vLLM | `ProviderType::Vllm` | localhost:8000 |

## 路由器

```rust
use maidos_llm::{Router, RoutingStrategy};

let router = Router::builder()
    .add_provider("openai", openai_provider, 10)  // priority 10
    .add_provider("anthropic", anthropic_provider, 5)
    .strategy(RoutingStrategy::Priority)
    .build();

// 自動選擇最高優先級的可用 Provider
let provider = router.select_provider()?;
```

### 路由策略

| 策略 | 說明 |
|------|------|
| `Priority` | 按優先級選擇 |
| `RoundRobin` | 輪詢 |
| `Weighted` | 加權隨機 |
| `Cost` | 最低成本優先 |
| `Speed` | 最快回應優先 |
| `Fallback` | 失敗時降級 |

## 預算控制

```rust
use maidos_llm::BudgetController;

let budget = BudgetController::builder()
    .daily_limit(10.0)        // $10/天
    .monthly_limit(100.0)     // $100/月
    .per_request_limit(0.50)  // $0.50/請求
    .warning_threshold(0.8)   // 80% 警告
    .build();

// 檢查預算
if budget.check_budget("openai", 0.10)? {
    // 執行請求
    budget.record_usage("openai", 0.10, 1000, 500)?;
}
```

## Vision 支援

```rust
let request = CompletionRequest::builder()
    .message(Message::user_with_image(
        "What's in this image?",
        "data:image/png;base64,..."
    ))
    .build();
```

## FFI

```c
MaidosLlmProvider* llm = maidos_llm_create("openai", api_key, NULL);
const char* response = maidos_llm_complete(llm, "Hello!");
maidos_llm_free(llm);
```

## License

MIT
