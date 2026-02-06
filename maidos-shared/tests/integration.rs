//! Integration Tests for MAIDOS Shared Core
//!
//! <impl>
//! Phase 6 整合測試：驗證模組間互動
//! - Config → Auth: 配置驅動認證
//! - Auth Token 操作
//! - Config → LLM: 配置驅動 LLM 
//! - 模組錯誤處理
//! </impl>

use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use maidos_config::MaidosConfig;
use maidos_llm::{CompletionRequest, Message, Role};
use std::str::FromStr;
use std::time::Duration;

// ============================================================================
// Test 1: Config → Auth Integration
// ============================================================================

#[test]
fn test_config_driven_auth() {
    // <impl>
    // 從配置文件讀取認證參數，建立 Token
    // </impl>
    
    let config_toml = r#"
[maidos]
version = "1.0"

[auth]
secret_key = "maidos-test-secret-key-12345678"
token_ttl = 3600

[llm]
default_provider = "openai"
budget_daily = 10.0
budget_monthly = 100.0
"#;

    let config = MaidosConfig::from_str(config_toml).expect("Failed to parse config");
    
    let auth_section = config.auth();
    let llm_section = config.llm();
    
    assert_eq!(auth_section.token_ttl, 3600);
    assert_eq!(llm_section.default_provider, "openai");
    
    // 從配置建立 TokenIssuer
    let issuer = TokenIssuer::from_config(&config).expect("Failed to create issuer");
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmStream);
    
    let token = issuer.issue(caps).expect("Failed to issue token");
    let token_str = token.as_str();
    
    let verified = issuer.verify(token_str).expect("Verification failed");
    assert!(verified.has(Capability::LlmChat));
    assert!(verified.has(Capability::LlmStream));
    assert!(!verified.has(Capability::FileRead));
}

// ============================================================================
// Test 2: Auth Token Roundtrip
// ============================================================================

#[test]
fn test_auth_token_roundtrip() {
    // <impl>
    // 測試令牌的創建、序列化、驗證
    // </impl>
    
    let secret = b"auth-integration-secret-key-32b!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(60));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::FileRead);
    caps.grant(Capability::EventPublish);
    
    let token = issuer.issue(caps).expect("Failed to issue token");
    let token_str = token.as_str();
    
    assert!(token_str.contains('.'));
    
    let verified = issuer.verify(token_str).expect("Verification failed");
    assert!(verified.has(Capability::LlmChat));
    assert!(verified.has(Capability::FileRead));
    assert!(verified.has(Capability::EventPublish));
    assert!(!verified.has(Capability::ScreenCapture));
}

// ============================================================================
// Test 3: Capability Access Control
// ============================================================================

#[test]
fn test_capability_access_control() {
    // <impl>
    // 測試能力系統的訪問控制
    // </impl>
    
    let secret = b"access-control-secret-key!!!!!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(60));
    
    // 有限權限令牌
    let mut limited_caps = CapabilitySet::empty();
    limited_caps.grant(Capability::LlmChat);
    let limited_token = issuer.issue(limited_caps).expect("Issue failed");
    let limited_str = limited_token.as_str();
    
    // 完整權限令牌
    let mut full_caps = CapabilitySet::empty();
    full_caps.grant(Capability::LlmChat);
    full_caps.grant(Capability::LlmVision);
    full_caps.grant(Capability::FileRead);
    full_caps.grant(Capability::ShellExec);
    let full_token = issuer.issue(full_caps).expect("Issue failed");
    let full_str = full_token.as_str();
    
    // 測試 check 方法
    assert!(issuer.check(limited_str, Capability::LlmChat));
    assert!(!issuer.check(limited_str, Capability::LlmVision));
    
    assert!(issuer.check(full_str, Capability::LlmChat));
    assert!(issuer.check(full_str, Capability::ShellExec));
    
    // 測試 check_all 方法
    assert!(issuer.check_all(full_str, &[Capability::LlmChat, Capability::FileRead]));
    assert!(!issuer.check_all(limited_str, &[Capability::LlmChat, Capability::FileRead]));
}

// ============================================================================
// Test 4: Config → LLM Request
// ============================================================================

#[test]
fn test_config_driven_llm_request() {
    // <impl>
    // 從配置讀取 LLM 參數，構建請求
    // </impl>
    
    let config_toml = r#"
[llm]
default_provider = "openai"

[llm.providers.openai]
model = "gpt-4o"
"#;

    let config = MaidosConfig::from_str(config_toml).expect("Failed to parse config");
    
    let llm_section = config.llm();
    assert_eq!(llm_section.default_provider, "openai");
    
    let provider_config = config.provider("openai").expect("Provider not found");
    assert_eq!(provider_config.model, Some("gpt-4o".to_string()));
    
    let request = CompletionRequest {
        model: provider_config.model.clone().unwrap_or_default(),
        messages: vec![Message::user("Hello!")],
        system: None,
        temperature: None,
        max_tokens: None,
        stream: false,
        stop: None,
        top_p: None,
    };
    
    assert_eq!(request.model, "gpt-4o");
}

// ============================================================================
// Test 5: Multi-Provider LLM Config
// ============================================================================

#[test]
fn test_multi_provider_llm_config() {
    // <impl>
    // 測試配置多個 LLM 供應商
    // </impl>
    
    let config_toml = r#"
[llm]
default_provider = "anthropic"

[llm.providers.openai]
model = "gpt-4o"

[llm.providers.anthropic]
model = "claude-sonnet-4-20250514"

[llm.providers.ollama]
model = "llama3.2"
endpoint = "http://localhost:11434"
"#;

    let config = MaidosConfig::from_str(config_toml).expect("Failed to parse config");
    
    let default = config.default_provider().expect("No default provider");
    assert_eq!(default.model, Some("claude-sonnet-4-20250514".to_string()));
    
    let ollama = config.provider("ollama").expect("Ollama not found");
    assert_eq!(ollama.endpoint, Some("http://localhost:11434".to_string()));
}

// ============================================================================
// Test 6: Error Propagation
// ============================================================================

#[test]
fn test_error_propagation() {
    // <impl>
    // 測試錯誤在模組間的正確傳播
    // </impl>
    
    // Config 錯誤
    let config_result = MaidosConfig::from_str("invalid { toml");
    assert!(config_result.is_err());
    
    // Auth 錯誤
    let secret = b"error-test-secret-key-32bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(60));
    let verify_result = issuer.verify("not.a.valid.token");
    assert!(verify_result.is_err());
}

// ============================================================================
// Test 7: CapabilitySet Operations
// ============================================================================

#[test]
fn test_capability_set_operations() {
    // <impl>
    // 測試 CapabilitySet 的各種操作
    // </impl>
    
    let mut caps = CapabilitySet::empty();
    
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmVision);
    assert!(caps.has(Capability::LlmChat));
    
    caps.revoke(Capability::LlmVision);
    assert!(!caps.has(Capability::LlmVision));
    
    assert!(caps.has_any(&[Capability::LlmChat, Capability::FileRead]));
    assert!(!caps.has_any(&[Capability::FileRead, Capability::FileWrite]));
    
    caps.grant(Capability::FileRead);
    assert!(caps.has_all(&[Capability::LlmChat, Capability::FileRead]));
    assert!(!caps.has_all(&[Capability::LlmChat, Capability::ShellExec]));
}

// ============================================================================
// Test 8: Config Schema Validation
// ============================================================================

#[test]
fn test_config_schema_validation() {
    // <impl>
    // 測試配置的 Schema 驗證
    // </impl>
    
    let valid_config = r#"
[maidos]
version = "1.0"

[llm]
default_provider = "openai"
budget_daily = 10.0

[bus]
endpoint = "tcp://127.0.0.1:9000"
"#;
    
    let config = MaidosConfig::from_str(valid_config).expect("Valid config should parse");
    let schema = config.schema();
    
    assert_eq!(schema.maidos.version, "1.0");
    assert_eq!(schema.bus.endpoint, "tcp://127.0.0.1:9000");
}

// ============================================================================
// Test 9: Token TTL
// ============================================================================

#[test]
fn test_token_ttl() {
    // <impl>
    // 測試令牌的 TTL 設置
    // </impl>
    
    let secret = b"ttl-test-secret-key-32bytes!!!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    
    let token1 = issuer.issue(caps).expect("Issue failed");
    let token2 = issuer.issue_with_ttl(caps, Duration::from_secs(60)).expect("Issue with TTL failed");
    let token3 = issuer.issue_for_subject(caps, "user:12345").expect("Issue for subject failed");
    
    assert!(issuer.verify(token1.as_str()).is_ok());
    assert!(issuer.verify(token2.as_str()).is_ok());
    assert!(issuer.verify(token3.as_str()).is_ok());
}

// ============================================================================
// Test 10: LLM Message Building
// ============================================================================

#[test]
fn test_llm_message_building() {
    // <impl>
    // 測試 LLM 訊息的各種類型
    // </impl>
    
    let user_msg = Message::user("Hello, AI!");
    assert_eq!(user_msg.role, Role::User);
    
    let assistant_msg = Message::assistant("Hello!");
    assert_eq!(assistant_msg.role, Role::Assistant);
    
    let system_msg = Message::system("You are helpful.");
    assert_eq!(system_msg.role, Role::System);
    
    let request = CompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::user("What is 2+2?"),
        ],
        system: Some("You are helpful.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
        stop: None,
        top_p: None,
    };
    
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.system, Some("You are helpful.".to_string()));
}

// ============================================================================
// Test 11: Full Integration - Config + Auth + LLM
// ============================================================================

#[test]
fn test_full_config_auth_llm_integration() {
    // <impl>
    // 完整的模組整合測試
    // </impl>
    
    let config_toml = r#"
[auth]
secret_key = "integration-test-secret-key-32!"
token_ttl = 1800

[llm]
default_provider = "anthropic"

[llm.providers.anthropic]
model = "claude-sonnet-4-20250514"
"#;

    let config = MaidosConfig::from_str(config_toml).expect("Config failed");
    let issuer = TokenIssuer::from_config(&config).expect("Issuer failed");
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmStream);
    
    let token = issuer.issue(caps).expect("Issue failed");
    let token_str = token.as_str();
    
    assert!(issuer.check(token_str, Capability::LlmChat));
    assert!(!issuer.check(token_str, Capability::ShellExec));
    
    if issuer.check(token_str, Capability::LlmChat) {
        let provider = config.default_provider().expect("No provider");
        
        let request = CompletionRequest {
            model: provider.model.clone().unwrap_or_default(),
            messages: vec![Message::user("Hello!")],
            system: None,
            temperature: None,
            max_tokens: None,
            stream: issuer.check(token_str, Capability::LlmStream),
            stop: None,
            top_p: None,
        };
        
        assert_eq!(request.model, "claude-sonnet-4-20250514");
        assert!(request.stream);
    }
}

// ============================================================================
// Test 12: Bus Event Creation
// ============================================================================

#[test]
fn test_bus_event_creation() {
    // <impl>
    // 測試 Bus 事件的創建
    // </impl>
    
    use maidos_bus::Event;
    
    let payload = b"Hello, Bus!".to_vec();
    let event = Event::new("test.topic", "test-source", payload).expect("Event creation failed");
    
    assert_eq!(event.topic, "test.topic");
    assert_eq!(event.source, "test-source");
    assert!(event.id > 0);  // ID should be non-zero
    
    let payload2 = b"Another".to_vec();
    let event2 = Event::new("test.topic", "test-source", payload2).expect("Event creation failed");
    assert_ne!(event.id, event2.id);
}

// ============================================================================
// Test 13: Bus Config Creation
// ============================================================================

#[test]
fn test_bus_config_creation() {
    // <impl>
    // 測試 Bus 配置
    // </impl>
    
    use maidos_bus::{PublisherConfig, SubscriberConfig};
    
    let pub_config = PublisherConfig {
        bind_addr: "127.0.0.1:9000".to_string(),
        channel_capacity: 2048,
        max_connections: 200,
    };
    
    assert_eq!(pub_config.bind_addr, "127.0.0.1:9000");
    
    let default_pub = PublisherConfig::default();
    assert_eq!(default_pub.channel_capacity, 1024);
    
    let sub_config = SubscriberConfig {
        publisher_addr: "127.0.0.1:9000".to_string(),
        topics: vec!["test.*".to_string()],
        reconnect_delay_ms: 1000,
        auto_reconnect: true,
        buffer_capacity: 256,
    };
    
    assert_eq!(sub_config.publisher_addr, "127.0.0.1:9000");
}

// ============================================================================
// Test 14: Capability Iteration
// ============================================================================

#[test]
fn test_capability_iteration() {
    // <impl>
    // 測試能力集合的迭代
    // </impl>
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::FileRead);
    caps.grant(Capability::EventPublish);
    
    let cap_vec: Vec<_> = caps.iter().collect();
    assert!(cap_vec.contains(&Capability::LlmChat));
    assert!(cap_vec.contains(&Capability::FileRead));
    assert!(cap_vec.contains(&Capability::EventPublish));
    assert!(!cap_vec.contains(&Capability::ShellExec));
}

// ============================================================================
// Test 15: Token Remaining TTL
// ============================================================================

#[test]
fn test_token_remaining_ttl() {
    // <impl>
    // 測試令牌的剩餘 TTL
    // </impl>
    
    let secret = b"remaining-ttl-secret-key-32byte".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    
    let token = issuer.issue(caps).expect("Issue failed");
    
    // 剩餘 TTL 應該接近 3600 秒（允許一些執行時間誤差）
    let remaining = token.remaining_ttl();
    assert!(remaining.as_secs() > 3590);
    assert!(remaining.as_secs() <= 3600);
    
    // 令牌應該未過期
    assert!(!token.is_expired());
}

// ============================================================================
// v0.2.0 Integration Tests
// ============================================================================

// ============================================================================
// Test 16: Provider Type Enumeration (v0.2.0)
// ============================================================================

#[test]
fn test_provider_type_all_v020() {
    // <impl>
    // 驗證 v0.2.0 所有 13 個 Provider 類型
    // </impl>
    
    use maidos_llm::ProviderType;
    
    let all = ProviderType::all();
    assert_eq!(all.len(), 13, "v0.2.0 should have 13 providers");
    
    // 驗證 Tier 1 (原有)
    assert!(ProviderType::parse("openai").is_some());
    assert!(ProviderType::parse("anthropic").is_some());
    assert!(ProviderType::parse("google").is_some());
    assert!(ProviderType::parse("deepseek").is_some());
    assert!(ProviderType::parse("groq").is_some());
    
    // 驗證 Tier 2 (v0.2.0 新增)
    assert!(ProviderType::parse("mistral").is_some());
    assert!(ProviderType::parse("azure_openai").is_some());
    assert!(ProviderType::parse("cohere").is_some());
    assert!(ProviderType::parse("together").is_some());
    assert!(ProviderType::parse("replicate").is_some());
    
    // 驗證本地 Provider
    assert!(ProviderType::parse("ollama").is_some());
    assert!(ProviderType::parse("lmstudio").is_some());
    assert!(ProviderType::parse("vllm").is_some());
}

// ============================================================================
// Test 17: Tool Format Conversion (v0.2.0)
// ============================================================================

#[test]
fn test_tool_format_conversion() {
    // <impl>
    // 驗證 MaidosTool 跨 Provider 格式轉換
    // </impl>
    
    use maidos_llm::tool::{MaidosTool, ToolParameter, ToProviderFormat};
    
    let tool = MaidosTool::new("get_weather", "Get weather for a location")
        .parameter(
            ToolParameter::string("location")
                .description("City name")
                .required(true),
        )
        .parameter(
            ToolParameter::string("unit")
                .enum_values(vec!["celsius", "fahrenheit"]),
        );
    
    // OpenAI format
    let openai = tool.to_openai();
    assert_eq!(openai["type"], "function");
    assert_eq!(openai["function"]["name"], "get_weather");
    
    // Anthropic format
    let anthropic = tool.to_anthropic();
    assert_eq!(anthropic["name"], "get_weather");
    assert!(anthropic.get("input_schema").is_some());
    
    // Google format
    let google = tool.to_google();
    assert_eq!(google["name"], "get_weather");
    
    // Cohere format
    let cohere = tool.to_cohere();
    assert!(cohere.get("parameter_definitions").is_some());
    
    // Mistral format (OpenAI-compatible)
    let mistral = tool.to_mistral();
    assert_eq!(mistral["type"], "function");
}

// ============================================================================
// Test 18: Error Type Classification (v0.2.0)
// ============================================================================

#[test]
fn test_error_capability_classification() {
    // <impl>
    // 驗證 Vision/Tools 錯誤正確分類
    // </impl>
    
    use maidos_llm::LlmError;
    
    let vision_err = LlmError::vision_not_supported("DeepSeek");
    assert!(vision_err.is_capability_error());
    assert!(vision_err.to_string().contains("DeepSeek"));
    assert!(vision_err.to_string().contains("vision"));
    
    let tools_err = LlmError::tools_not_supported("Replicate");
    assert!(tools_err.is_capability_error());
    assert!(tools_err.to_string().contains("Replicate"));
    assert!(tools_err.to_string().contains("function calling"));
    
    let other_err = LlmError::Auth("test".to_string());
    assert!(!other_err.is_capability_error());
}

// ============================================================================
// Test 19: Streaming Types (v0.2.0)
// ============================================================================

#[test]
fn test_streaming_types() {
    // <impl>
    // 驗證 Streaming 統一類型
    // </impl>
    
    use maidos_llm::streaming::{StreamChunk, StreamUsage, SseParser, SseEvent};
    
    // StreamChunk
    let chunk = StreamChunk::text("Hello");
    assert_eq!(chunk.delta, "Hello");
    assert!(!chunk.is_final());
    
    let final_chunk = StreamChunk::finish("stop");
    assert!(final_chunk.is_final());
    
    // StreamUsage
    let usage = StreamUsage::new(100, 50);
    assert_eq!(usage.total_tokens, 150);
    
    // SseParser
    let mut parser = SseParser::new();
    let events = parser.parse(b"data: {\"text\": \"hello\"}\n\n");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], SseEvent::Data(_)));
    
    let done_events = parser.parse(b"data: [DONE]\n\n");
    assert_eq!(done_events.len(), 1);
    assert_eq!(done_events[0], SseEvent::Done);
}

// ============================================================================
// Test 20: Budget Controller (v0.2.0)
// ============================================================================

#[test]
fn test_budget_controller_v020() {
    // <impl>
    // 驗證預算控制功能
    // </impl>
    
    use maidos_llm::budget::{BudgetBuilder, BudgetLimit, ExceededAction};
    
    let controller = BudgetBuilder::new()
        .global_limit(BudgetLimit {
            max_cost: 10.0,
            ..Default::default()
        })
        .exceeded_action(ExceededAction::Block)
        .build();
    
    // 記錄用量
    controller.record_usage("openai", 100, 50, 0.01);
    
    let status = controller.get_global_status().expect("Status should exist");
    assert_eq!(status.usage.input_tokens, 100);
    assert_eq!(status.usage.output_tokens, 50);
    assert!(!status.exceeded);
    
    // 超過預算
    controller.record_usage("openai", 1000, 500, 15.0);
    assert!(controller.check_budget("openai").is_err());
}
