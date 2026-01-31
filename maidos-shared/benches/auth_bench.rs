//! Auth Module Benchmarks
//!
//! <impl>
//! WHAT: 認證模組效能基準測試
//! WHY: 建立效能基線，識別熱路徑優化機會
//! HOW: 使用 Criterion 框架測量關鍵操作
//! METRICS: Token 生成、驗證、Capability 操作 ops/sec
//! </impl>

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use maidos_auth::{Capability, CapabilitySet, TokenIssuer};
use std::time::Duration;

/// 基準測試：Token 生成
fn bench_token_issue(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::FileRead);
    
    c.bench_function("token_issue", |b| {
        b.iter(|| {
            issuer.issue(black_box(caps.clone())).unwrap()
        })
    });
}

/// 基準測試：Token 驗證
fn bench_token_verify(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    let token = issuer.issue(caps).unwrap();
    let token_str = token.as_str().to_string();
    
    c.bench_function("token_verify", |b| {
        b.iter(|| {
            issuer.verify(black_box(&token_str)).unwrap()
        })
    });
}

/// 基準測試：Token 生成+驗證完整流程
fn bench_token_roundtrip(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::FileRead);
    caps.grant(Capability::EventPublish);
    
    c.bench_function("token_roundtrip", |b| {
        b.iter(|| {
            let token = issuer.issue(black_box(caps.clone())).unwrap();
            issuer.verify(token.as_str()).unwrap()
        })
    });
}

/// 基準測試：Capability 權限檢查
fn bench_capability_check(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmVision);
    caps.grant(Capability::FileRead);
    caps.grant(Capability::EventPublish);
    
    let token = issuer.issue(caps).unwrap();
    let token_str = token.as_str().to_string();
    
    c.bench_function("capability_check_single", |b| {
        b.iter(|| {
            issuer.check(black_box(&token_str), black_box(Capability::LlmChat))
        })
    });
    
    c.bench_function("capability_check_all", |b| {
        b.iter(|| {
            issuer.check_all(
                black_box(&token_str), 
                black_box(&[Capability::LlmChat, Capability::FileRead])
            )
        })
    });
}

/// 基準測試：CapabilitySet 操作
fn bench_capability_set(c: &mut Criterion) {
    c.bench_function("capset_grant_revoke", |b| {
        b.iter(|| {
            let mut caps = CapabilitySet::empty();
            caps.grant(black_box(Capability::LlmChat));
            caps.grant(black_box(Capability::FileRead));
            caps.grant(black_box(Capability::EventPublish));
            caps.revoke(black_box(Capability::FileRead));
            caps.has(black_box(Capability::LlmChat))
        })
    });
    
    c.bench_function("capset_has_all", |b| {
        let mut caps = CapabilitySet::empty();
        caps.grant(Capability::LlmChat);
        caps.grant(Capability::LlmVision);
        caps.grant(Capability::FileRead);
        caps.grant(Capability::FileWrite);
        caps.grant(Capability::EventPublish);
        
        let check = [
            Capability::LlmChat, 
            Capability::FileRead, 
            Capability::EventPublish
        ];
        
        b.iter(|| {
            caps.has_all(black_box(&check))
        })
    });
    
    c.bench_function("capset_iteration", |b| {
        let mut caps = CapabilitySet::empty();
        caps.grant(Capability::LlmChat);
        caps.grant(Capability::LlmVision);
        caps.grant(Capability::FileRead);
        caps.grant(Capability::ShellExec);
        caps.grant(Capability::EventPublish);
        
        b.iter(|| {
            let count: usize = caps.iter().count();
            black_box(count)
        })
    });
}

/// 基準測試：不同能力數量的 Token 效能
fn bench_token_scaling(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret, Duration::from_secs(3600));
    
    let mut group = c.benchmark_group("token_scaling");
    
    for cap_count in [1, 5, 10, 18].iter() {
        let mut caps = CapabilitySet::empty();
        for i in 0..*cap_count {
            // 使用 Capability::all() 的前 N 個
            if let Some(cap) = Capability::all().get(i) {
                caps.grant(*cap);
            }
        }
        
        group.throughput(Throughput::Elements(*cap_count as u64));
        group.bench_with_input(
            BenchmarkId::new("issue", cap_count),
            cap_count,
            |b, _| {
                b.iter(|| issuer.issue(black_box(caps.clone())).unwrap())
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_token_issue,
    bench_token_verify,
    bench_token_roundtrip,
    bench_capability_check,
    bench_capability_set,
    bench_token_scaling,
);

criterion_main!(benches);
