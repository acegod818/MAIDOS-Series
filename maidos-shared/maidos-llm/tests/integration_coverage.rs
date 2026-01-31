//! Integration coverage test – forces execution of many public APIs
//! to increase line‑coverage for the core LLM crate.

use maidos_llm::{
    providers::{create_provider, ProviderType},
    router::{RouterBuilder, ProviderConfig},
    budget::{BudgetBuilder, BudgetController},
    message::{Message, Role, Content},
};

#[tokio::test]
async fn cover_llm_core_paths() {
    // 1. Parse all provider types – exercises ProviderType::parse & name()
    for name in [
        "openai", "anthropic", "google", "deepseek", "groq", "mistral",
        "azure", "cohere", "together", "replicate", "ollama", "lmstudio", "vllm",
    ] {
        let pt = ProviderType::parse(name).expect("known provider");
        assert!(!pt.name().is_empty());
    }

    // 2. Create a dummy local provider (Ollama) – no real network call
    let provider = create_provider(ProviderType::Ollama, Some("key".to_string()), Some("http://localhost:11434".to_string())).expect("create provider");
    // We don't call chat here to avoid actual network/async hangs in tarpaulin if not mocked
    assert_eq!(provider.name().to_lowercase(), "ollama");

    // 3. Build a simple router with a single provider
    let router = RouterBuilder::default()
        .add_provider(ProviderConfig {
            name: "ollama".into(),
            ..Default::default()
        })
        .build();
    assert!(!router.get_available_providers().is_empty());

    // 4. Use BudgetController – create a simple budget and check status
    let budget_config = BudgetBuilder::default().build().config().clone();
    let controller = BudgetController::new(budget_config);
    controller.record_usage("ollama", 100, 50, 0.0);

    // 5. Message utilities – ensure all constructors and helpers run
    let msg = Message::with_content(
        Role::User,
        vec![
            Content::text("Hello"), 
            Content::image_url("https://example.com/img.png")
        ],
    );
    assert!(msg.has_image());
    assert_eq!(msg.text(), "Hello");
}