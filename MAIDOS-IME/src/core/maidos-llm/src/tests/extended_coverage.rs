//! LLM extended tests

use crate::{LlmClient, models::ModelConfig, ProviderType, LlmRequest, Message};

#[tokio::test]
async fn test_local_client_edge_cases() {
    let client = LlmClient::new("local");
    
    // Test empty message request
    let empty_request = LlmRequest {
        provider: ProviderType::Ollama,
        model: "local".to_string(),
        system: None,
        messages: vec![],
        max_tokens: Some(10),
        temperature: Some(0.7),
    };
    
    let empty_result = client.complete(&empty_request).await;
    assert!(empty_result.is_ok());
    
    // Test single message
    let single_request = LlmRequest {
        provider: ProviderType::Ollama,
        model: "local".to_string(),
        system: None,
        messages: vec![Message::user("test")],
        max_tokens: Some(10),
        temperature: Some(0.7),
    };
    
    let single_result = client.complete(&single_request).await;
    assert!(single_result.is_ok());
    
    // Test multiple messages
    let multi_request = LlmRequest {
        provider: ProviderType::Ollama,
        model: "local".to_string(),
        system: Some("system prompt".to_string()),
        messages: vec![
            Message::system("This is a system prompt"),
            Message::user("User question"),
            Message::assistant("Assistant response")
        ],
        max_tokens: Some(20),
        temperature: Some(0.5),
    };
    
    let multi_result = client.complete(&multi_request).await;
    assert!(multi_result.is_ok());
}

#[test]
fn test_local_llm_inference_edge_cases() {
    use crate::local::LocalLlm;
    
    let llm = LocalLlm::new();
    
    // Test empty input
    assert_eq!(llm.simple_inference(""), "ok");
    
    // Test whitespace-only input
    assert_eq!(llm.simple_inference("   "), "ok");
    
    // Test unknown keyword
    assert_eq!(llm.simple_inference("xyzabc"), "ok");
    
    // Test mixed content
    let mixed_response = llm.simple_inference("hello world test unknown");
    assert!(!mixed_response.is_empty());
    
    // Test selection format
    let select_response = llm.simple_inference("select the most appropriate character");
    assert_eq!(select_response, "ok");

    // Test correction format
    let correct_response = llm.simple_inference("correct errors");
    assert_eq!(correct_response, "ok");
}

#[test]
fn test_model_config_validation() {
    use crate::models::ModelConfig;
    
    // Test valid config
    let valid_config = ModelConfig {
        provider: "ollama".to_string(),
        model: "llama3.2".to_string(),
        max_tokens: 100,
    };
    
    let client_result = LlmClient::from_config(&valid_config);
    assert!(client_result.is_ok());
    
    // Test invalid provider
    let invalid_provider = ModelConfig {
        provider: "invalid".to_string(),
        model: "model".to_string(),
        max_tokens: 10,
    };
    
    let invalid_result = LlmClient::from_config(&invalid_provider);
    assert!(invalid_result.is_err());
}

#[test]
fn test_provider_types() {
    // Test provider type comparison
    let ollama = ProviderType::Ollama;
    let openai = ProviderType::OpenAI;
    let anthropic = ProviderType::Anthropic;
    
    assert_eq!(ollama, ProviderType::Ollama);
    assert_eq!(openai, ProviderType::OpenAI);
    assert_eq!(anthropic, ProviderType::Anthropic);
    
    // Test provider type clone
    let cloned_ollama = ollama.clone();
    assert_eq!(cloned_ollama, ProviderType::Ollama);
}

#[test]
fn test_message_types() {
    // Test message creation
    let system_msg = Message::system("System message");
    let user_msg = Message::user("User message");
    let assistant_msg = Message::assistant("Assistant message");
    
    assert_eq!(system_msg.role, crate::Role::System);
    assert_eq!(user_msg.role, crate::Role::User);
    assert_eq!(assistant_msg.role, crate::Role::Assistant);
    
    // Test message clone
    let cloned_user = user_msg.clone();
    assert_eq!(cloned_user.content, "User message");

    // Test message content
    assert_eq!(system_msg.content, "System message");
    assert_eq!(user_msg.content, "User message");
    assert_eq!(assistant_msg.content, "Assistant message");
}