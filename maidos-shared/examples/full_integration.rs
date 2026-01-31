//! Full Integration Example
//!
//! <impl>
//! WHAT: 展示所有模組協同工作
//! WHY: 讓開發者理解完整工作流程
//! HOW: Config → Auth → LLM → Bus 完整流程
//! </impl>

use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use maidos_bus::{Event, PublisherConfig};
use maidos_config::MaidosConfig;
use maidos_llm::{CompletionRequest, Message};
use std::str::FromStr;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MAIDOS Full Integration Example ===\n");

    // Step 1: 載入配置
    println!("【Step 1: 載入配置】");
    
    let config_str = r#"
[maidos]
version = "1.0"

[auth]
secret_key = "production-secret-32-bytes-key!"
token_ttl = 7200

[llm]
default_provider = "anthropic"
budget_daily = 50.0
budget_monthly = 500.0

[llm.providers.openai]
api_key = "sk-openai-key"
model = "gpt-4o"
endpoint = "https://api.openai.com/v1"

[llm.providers.anthropic]
api_key = "sk-anthropic-key"
model = "claude-sonnet-4-20250514"
endpoint = "https://api.anthropic.com/v1"

[bus]
endpoint = "tcp://127.0.0.1:9000"
buffer_size = 2048
"#;

    let config = MaidosConfig::from_str(config_str)?;
    println!("  ✓ 配置載入成功");
    println!("  Version: {}", config.schema().maidos.version);

    // Step 2: 從配置創建 TokenIssuer
    println!("\n【Step 2: 創建認證系統】");
    
    let auth_config = config.auth();
    let secret = auth_config.secret_key.as_ref()
        .expect("Secret key required");
    let issuer = TokenIssuer::new(
        secret.as_bytes().to_vec(),
        Duration::from_secs(auth_config.token_ttl),
    );
    println!("  ✓ TokenIssuer 創建成功");
    println!("  TTL: {} 秒", auth_config.token_ttl);

    // Step 3: 發行服務令牌
    println!("\n【Step 3: 發行服務令牌】");
    
    // LLM 服務令牌
    let mut llm_caps = CapabilitySet::empty();
    llm_caps.grant(Capability::LlmChat);
    llm_caps.grant(Capability::LlmVision);
    let llm_token = issuer.issue(llm_caps)?;
    let token_preview = &llm_token.as_str()[..40.min(llm_token.as_str().len())];
    println!("  ✓ LLM 服務令牌: {}...", token_preview);

    // Bus 服務令牌
    let mut bus_caps = CapabilitySet::empty();
    bus_caps.grant(Capability::EventPublish);
    bus_caps.grant(Capability::EventSubscribe);
    let bus_token = issuer.issue(bus_caps)?;
    let token_preview = &bus_token.as_str()[..40.min(bus_token.as_str().len())];
    println!("  ✓ Bus 服務令牌: {}...", token_preview);

    // Step 4: 驗證令牌權限
    println!("\n【Step 4: 驗證令牌權限】");
    
    let incoming_token = llm_token.as_str();
    
    let can_chat = issuer.check(incoming_token, Capability::LlmChat);
    let can_shell = issuer.check(incoming_token, Capability::ShellExec);
    
    println!("  LLM 令牌權限檢查:");
    println!("    LlmChat: {}", if can_chat { "✓ 允許" } else { "✗ 拒絕" });
    println!("    ShellExec: {}", if can_shell { "✓ 允許" } else { "✗ 拒絕" });

    // Step 5: 構建 LLM 請求
    println!("\n【Step 5: 構建 LLM 請求】");
    
    let provider = config.default_provider()
        .expect("Default provider not found");
    
    // model 是 Option<String>，需要處理 None 的情況
    let model = provider.model.clone().unwrap_or_else(|| "gpt-4o".to_string());
    
    let request = CompletionRequest {
        model: model.clone(),
        messages: vec![
            Message::user("Explain the concept of zero-trust security."),
        ],
        system: Some("You are a cybersecurity expert.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2048),
        stream: false,
        stop: None,
        top_p: None,
    };
    
    println!("  ✓ LLM 請求構建成功");
    println!("    Model: {}", request.model);
    println!("    Messages: {} 條", request.messages.len());

    // Step 6: 發布事件到 Bus
    println!("\n【Step 6: 發布事件到 Bus】");
    
    let request_json = serde_json::to_vec(&request)?;
    
    let event = Event::new(
        "llm.request.queued",
        "llm-gateway",
        request_json,
    )?;
    
    println!("  ✓ Event 創建成功");
    println!("    ID: {}", event.id);
    println!("    Topic: {}", event.topic);
    println!("    Payload: {} bytes", event.payload.len());

    // Bus 配置 - endpoint 和 buffer_size 不是 Option
    let bus_config = config.bus();
    let endpoint = bus_config.endpoint.replace("tcp://", "");
    
    let pub_config = PublisherConfig {
        bind_addr: endpoint.clone(),
        channel_capacity: bus_config.buffer_size,
        max_connections: 100,
    };
    
    println!("    Publisher: {}", pub_config.bind_addr);

    // Step 7: 模擬響應事件
    println!("\n【Step 7: 模擬響應事件】");
    
    #[derive(serde::Serialize)]
    struct LlmResponse {
        request_id: String,
        model: String,
        content: String,
        tokens_used: u32,
        latency_ms: u32,
    }
    
    let response = LlmResponse {
        request_id: event.id.to_string(),  // u64 -> String
        model: request.model.clone(),
        content: "Zero-trust security is a framework...".to_string(),
        tokens_used: 256,
        latency_ms: 1500,
    };
    
    let response_json = serde_json::to_vec(&response)?;
    let response_event = Event::new(
        "llm.response.completed",
        "llm-worker-01",
        response_json,
    )?;
    
    println!("  ✓ Response Event 創建成功");
    println!("    ID: {}", response_event.id);
    println!("    Topic: {}", response_event.topic);

    // 統計摘要
    println!("\n【整合統計】");
    println!("  配置 Sections: maidos, auth, llm, bus");
    println!("  發行令牌: 2 個");
    println!("  LLM Providers: {}", config.llm().providers.len());
    println!("  Events 發布: 2 條");
    println!("  總 Payload: {} bytes", 
        event.payload.len() + response_event.payload.len());

    println!("\n=== 整合範例完成 ===");
    Ok(())
}
