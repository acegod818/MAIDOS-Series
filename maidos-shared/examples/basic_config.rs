//! Basic Configuration Example
//!
//! <impl>
//! WHAT: 展示 maidos-config 基本使用
//! WHY: 讓開發者快速上手配置模組
//! HOW: 載入、訪問、驗證配置
//! </impl>

use maidos_config::MaidosConfig;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MAIDOS Config Basic Example ===\n");

    let config_str = r#"
[maidos]
version = "1.0"

[auth]
secret_key = "my-super-secret-key-32-bytes!!"
token_ttl = 3600

[llm]
default_provider = "openai"
budget_daily = 10.0
budget_monthly = 100.0

[llm.providers.openai]
api_key = "sk-test-key"
model = "gpt-4o"
endpoint = "https://api.openai.com/v1"

[bus]
endpoint = "tcp://127.0.0.1:9000"
buffer_size = 1024
"#;

    let config = MaidosConfig::from_str(config_str)?;
    println!("✓ 配置載入成功\n");

    // 訪問 Auth 配置
    println!("【Auth 配置】");
    let auth = config.auth();
    if let Some(ref key) = auth.secret_key {
        let display_len = 16.min(key.len());
        println!("  Secret Key: {}...", &key[..display_len]);
    }
    println!("  Token TTL: {} 秒", auth.token_ttl);

    // 訪問 LLM 配置
    println!("\n【LLM 配置】");
    let llm = config.llm();
    println!("  默認供應商: {}", llm.default_provider);
    println!("  每日預算: ${:.2}", llm.budget_daily);
    println!("  每月預算: ${:.2}", llm.budget_monthly);

    // 獲取 Provider
    println!("\n【OpenAI Provider】");
    if let Some(openai) = config.provider("openai") {
        if let Some(ref model) = openai.model {
            println!("  Model: {}", model);
        }
        if let Some(ref endpoint) = openai.endpoint {
            println!("  Endpoint: {}", endpoint);
        }
        println!("  Timeout: {} 秒", openai.timeout_secs);
        println!("  Max Retries: {}", openai.max_retries);
    }

    // 訪問 Bus 配置
    println!("\n【Bus 配置】");
    let bus = config.bus();
    println!("  Endpoint: {}", bus.endpoint);
    println!("  Buffer Size: {}", bus.buffer_size);
    println!("  Reconnect: {} ms", bus.reconnect_ms);

    println!("\n=== 範例完成 ===");
    Ok(())
}
