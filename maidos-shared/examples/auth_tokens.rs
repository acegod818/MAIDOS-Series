//! Authentication Token Example
//!
//! <impl>
//! WHAT: 展示 maidos-auth 令牌操作
//! WHY: 讓開發者理解認證流程
//! HOW: 發行、驗證、權限檢查
//! </impl>

use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MAIDOS Auth Token Example ===\n");

    // 創建 TokenIssuer
    let secret = b"my-super-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    println!("✓ TokenIssuer 創建成功（TTL: 1小時）\n");

    // 創建 CapabilitySet
    println!("【創建權限集】");
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmVision);
    caps.grant(Capability::FileRead);
    
    println!("  已授予權限:");
    for cap in caps.iter() {
        println!("    - {:?}", cap);
    }

    // 發行令牌
    println!("\n【發行令牌】");
    let token = issuer.issue(caps)?;
    let token_str = token.as_str();
    let display_len = 50.min(token_str.len());
    println!("  Token: {}...", &token_str[..display_len]);
    println!("  剩餘有效時間: {:?}", token.remaining_ttl());

    // 驗證令牌
    println!("\n【驗證令牌】");
    let verified = issuer.verify(token_str)?;
    println!("  ✓ 令牌驗證成功");
    println!("  權限數量: {}", verified.capabilities().iter().count());

    // 權限檢查（返回 bool）
    println!("\n【權限檢查】");
    
    let has_chat = issuer.check(token_str, Capability::LlmChat);
    println!("  LlmChat: {}", if has_chat { "✓" } else { "✗" });
    
    let has_shell = issuer.check(token_str, Capability::ShellExec);
    println!("  ShellExec: {}", if has_shell { "✓" } else { "✗" });

    let has_all = issuer.check_all(
        token_str, 
        &[Capability::LlmChat, Capability::FileRead]
    );
    println!("  LlmChat + FileRead: {}", if has_all { "✓" } else { "✗" });

    // 所有可用權限
    println!("\n【所有可用權限】");
    let all_caps = Capability::all();
    println!("  共 {} 種權限:", all_caps.len());
    for (i, cap) in all_caps.iter().enumerate().take(6) {
        println!("    {:2}. {:?}", i + 1, cap);
    }
    if all_caps.len() > 6 {
        println!("    ... ({} more)", all_caps.len() - 6);
    }

    // CapabilitySet 操作
    println!("\n【CapabilitySet 操作】");
    let mut demo_caps = CapabilitySet::empty();
    
    demo_caps.grant(Capability::EventPublish);
    println!("  grant(EventPublish): has={}", demo_caps.has(Capability::EventPublish));
    
    demo_caps.revoke(Capability::EventPublish);
    println!("  revoke(EventPublish): has={}", demo_caps.has(Capability::EventPublish));
    
    demo_caps.grant(Capability::FileRead);
    let has_any = demo_caps.has_any(&[Capability::ShellExec, Capability::FileRead]);
    println!("  has_any([ShellExec, FileRead]): {}", has_any);

    // 帶自定義 TTL 的令牌
    println!("\n【自定義 TTL 令牌】");
    let short_token = issuer.issue_with_ttl(caps, Duration::from_secs(300))?;
    println!("  TTL: 5 分鐘");
    println!("  剩餘時間: {:?}", short_token.remaining_ttl());

    println!("\n=== 範例完成 ===");
    Ok(())
}
