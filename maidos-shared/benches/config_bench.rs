//! Config Module Benchmarks
//!
//! <impl>
//! WHAT: 配置模組效能基準測試
//! WHY: 測量配置載入、解析、訪問效能
//! HOW: 使用 Criterion 框架
//! METRICS: TOML 解析、值訪問、序列化 ops/sec
//! </impl>

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::str::FromStr;
use maidos_config::MaidosConfig;

/// 最小配置
const MINIMAL_CONFIG: &str = r#"
[maidos]
version = "1.0"
"#;

/// 標準配置
const STANDARD_CONFIG: &str = r#"
[maidos]
version = "1.0"

[auth]
secret_key = "test-secret-key-32-bytes-long!!"
token_ttl = 3600

[llm]
default_provider = "openai"
budget_daily = 10.0
budget_monthly = 100.0

[llm.providers.openai]
model = "gpt-4o"

[bus]
endpoint = "tcp://127.0.0.1:9000"
buffer_size = 1024
"#;

/// 完整配置
const FULL_CONFIG: &str = r#"
[maidos]
version = "1.0"

[auth]
secret_key = "production-secret-key-32-bytes!"
token_ttl = 7200

[llm]
default_provider = "anthropic"
budget_daily = 50.0
budget_monthly = 500.0

[llm.providers.openai]
api_key = "sk-benchmark-test-key-not-real"
model = "gpt-4o"
endpoint = "https://api.openai.com/v1"
timeout_secs = 30
max_retries = 3

[llm.providers.anthropic]
api_key = "sk-ant-benchmark-test-key-not-real"
model = "claude-sonnet-4-20250514"
endpoint = "https://api.anthropic.com/v1"
timeout_secs = 60
max_retries = 3

[llm.providers.ollama]
model = "llama3.2"
endpoint = "http://localhost:11434"
timeout_secs = 120

[bus]
endpoint = "tcp://127.0.0.1:9000"
buffer_size = 4096
reconnect_ms = 5000
"#;

/// 基準測試：配置解析
fn bench_config_parse(c: &mut Criterion) {
    c.bench_function("config_parse_minimal", |b| {
        b.iter(|| {
            MaidosConfig::from_str(black_box(MINIMAL_CONFIG)).unwrap()
        })
    });
    
    c.bench_function("config_parse_standard", |b| {
        b.iter(|| {
            MaidosConfig::from_str(black_box(STANDARD_CONFIG)).unwrap()
        })
    });
    
    c.bench_function("config_parse_full", |b| {
        b.iter(|| {
            MaidosConfig::from_str(black_box(FULL_CONFIG)).unwrap()
        })
    });
}

/// 基準測試：配置值訪問
fn bench_config_access(c: &mut Criterion) {
    let config = MaidosConfig::from_str(FULL_CONFIG).unwrap();
    
    c.bench_function("config_access_section", |b| {
        b.iter(|| {
            let llm = config.llm();
            black_box(llm.default_provider.clone())
        })
    });
    
    c.bench_function("config_access_provider", |b| {
        b.iter(|| {
            let provider = config.provider(black_box("openai"));
            black_box(provider)
        })
    });
    
    c.bench_function("config_access_default_provider", |b| {
        b.iter(|| {
            let provider = config.default_provider();
            black_box(provider)
        })
    });
    
    c.bench_function("config_access_schema", |b| {
        b.iter(|| {
            let schema = config.schema();
            black_box(schema.maidos.version.clone())
        })
    });
}

/// 基準測試：配置序列化（到 JSON）
fn bench_config_serialize(c: &mut Criterion) {
    let config = MaidosConfig::from_str(FULL_CONFIG).unwrap();
    let schema = config.schema();
    
    c.bench_function("config_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&schema)).unwrap();
            black_box(json)
        })
    });
    
    c.bench_function("config_to_json_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(black_box(&schema)).unwrap();
            black_box(json)
        })
    });
}

/// 基準測試：配置大小擴展性
fn bench_config_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_scaling");
    
    // 不同配置複雜度
    let configs = [
        ("minimal", MINIMAL_CONFIG),
        ("standard", STANDARD_CONFIG),
        ("full", FULL_CONFIG),
    ];
    
    for (name, config_str) in configs.iter() {
        group.throughput(Throughput::Bytes(config_str.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            config_str,
            |b, config| {
                b.iter(|| MaidosConfig::from_str(black_box(config)).unwrap())
            }
        );
    }
    
    group.finish();
}

/// 基準測試：多次訪問（模擬實際使用）
fn bench_config_realistic_usage(c: &mut Criterion) {
    let config = MaidosConfig::from_str(FULL_CONFIG).unwrap();
    
    c.bench_function("config_realistic_usage", |b| {
        b.iter(|| {
            // 模擬實際使用模式：多次訪問不同 section
            let llm = config.llm();
            let auth = config.auth();
            let bus = config.bus();
            
            let _provider = llm.default_provider.clone();
            let _ttl = auth.token_ttl;
            let _endpoint = bus.endpoint.clone();
            
            // 獲取特定 provider
            let openai = config.provider("openai");
            let anthropic = config.provider("anthropic");
            
            black_box((openai, anthropic))
        })
    });
}

/// 基準測試：配置重新解析（模擬熱重載）
fn bench_config_reload(c: &mut Criterion) {
    c.bench_function("config_reload_simulation", |b| {
        b.iter(|| {
            // 模擬配置變更後的重新解析
            let config1 = MaidosConfig::from_str(black_box(STANDARD_CONFIG)).unwrap();
            let config2 = MaidosConfig::from_str(black_box(FULL_CONFIG)).unwrap();
            
            // 比較關鍵值
            let changed = config1.llm().default_provider != config2.llm().default_provider;
            black_box(changed)
        })
    });
}

/// 基準測試：Default 配置創建
fn bench_config_default(c: &mut Criterion) {
    c.bench_function("config_default", |b| {
        b.iter(|| {
            black_box(MaidosConfig::default_config())
        })
    });
}

criterion_group!(
    benches,
    bench_config_parse,
    bench_config_access,
    bench_config_serialize,
    bench_config_scaling,
    bench_config_realistic_usage,
    bench_config_reload,
    bench_config_default,
);

criterion_main!(benches);
