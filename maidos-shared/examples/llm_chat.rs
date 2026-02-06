//! LLM Chat Example
//!
//! <impl>
//! WHAT: 展示 maidos-llm 請求構建
//! WHY: 讓開發者快速整合 LLM
//! HOW: 構建訊息、請求、多供應商支持
//! </impl>

use maidos_llm::{CompletionRequest, Message, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MAIDOS LLM Chat Example ===\n");

    // 創建訊息
    println!("【創建訊息】");
    let _user_msg = Message::user("Hello, how are you?");
    let _assistant_msg = Message::assistant("I'm doing well, thank you!");
    let _system_msg = Message::system("You are a helpful assistant.");
    
    println!("  User message created");
    println!("  Assistant message created");
    println!("  System message created");

    // 構建基本請求
    println!("\n【基本請求】");
    let basic_request = CompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::user("What is 2 + 2?"),
        ],
        system: None,
        temperature: None,
        max_tokens: None,
        stream: false,
        stop: None,
        top_p: None,
    };
    
    let json = serde_json::to_string_pretty(&basic_request)?;
    println!("{}", json);

    // 構建完整請求
    println!("\n【完整請求】");
    let full_request = CompletionRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        messages: vec![
            Message::user("Explain quantum computing."),
            Message::assistant("Quantum computing uses quantum mechanics..."),
            Message::user("Can you give an example?"),
        ],
        system: Some("You are a physics professor.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2048),
        stream: true,
        stop: Some(vec!["END".to_string(), "STOP".to_string()]),
        top_p: Some(0.9),
    };
    
    println!("  Model: {}", full_request.model);
    println!("  Messages: {} 條", full_request.messages.len());
    println!("  System: {:?}", full_request.system);
    println!("  Temperature: {:?}", full_request.temperature);
    println!("  Max Tokens: {:?}", full_request.max_tokens);
    println!("  Stream: {}", full_request.stream);

    // 多供應商配置
    println!("\n【多供應商模型】");
    let providers = [
        ("OpenAI", "gpt-4o"),
        ("OpenAI", "gpt-4o-mini"),
        ("Anthropic", "claude-sonnet-4-20250514"),
        ("Anthropic", "claude-opus-4-20250514"),
        ("Ollama", "llama3.2"),
    ];
    
    for (provider, model) in providers {
        println!("  {} / {}: ✓", provider, model);
    }

    // Role 枚舉
    println!("\n【Role 類型】");
    let roles = [Role::User, Role::Assistant, Role::System];
    for role in roles {
        println!("  {:?}", role);
    }

    // 對話歷史構建
    println!("\n【對話歷史】");
    let conversation = [
        Message::user("Hi there!"),
        Message::assistant("Hello! How can I help?"),
        Message::user("What's the weather like?"),
        Message::assistant("I don't have weather data."),
        Message::user("Okay, thanks anyway."),
    ];
    
    println!("  對話輪數: {}", conversation.len());
    for (i, msg) in conversation.iter().enumerate() {
        let role_str = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            Role::System => "System",
            Role::Tool => "Tool",
        };
        println!("  [{}] {}", i + 1, role_str);
    }

    println!("\n=== 範例完成 ===");
    Ok(())
}
