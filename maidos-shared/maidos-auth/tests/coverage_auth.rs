use maidos_auth::{Capability, CapabilitySet, CapabilityToken};
use std::time::Duration;

#[test]
fn cover_auth_paths() {
    // 1. Capability construction
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::FileRead);
    assert!(caps.has(Capability::LlmChat));
    assert!(caps.has(Capability::FileRead));

    // 2. Token creation & verification
    let secret = b"your-32-byte-secret-key-here!!!";
    let token = CapabilityToken::new(caps, Duration::from_secs(3600), secret).expect("jwt");
    let verified = CapabilityToken::verify(token.as_str(), secret).expect("verify");
    assert!(verified.has(Capability::LlmChat));
    assert_eq!(verified.subject(), None);

    // 3. Token with subject
    let token_sub = CapabilityToken::new_with_subject(
        CapabilitySet::from_u32(1), 
        Duration::from_secs(3600), 
        secret, 
        Some("user123".into())
    ).unwrap();
    assert_eq!(token_sub.subject(), Some("user123"));
}