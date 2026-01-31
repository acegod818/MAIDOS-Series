//! Bus Module Benchmarks
//!
//! <impl>
//! WHAT: 訊息匯流排效能基準測試
//! WHY: 測量 Pub/Sub 吞吐量和延遲
//! HOW: 使用 Criterion + Tokio runtime
//! METRICS: 訊息發布、序列化、事件創建 ops/sec
//! </impl>

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use maidos_bus::{Event, PublisherConfig, SubscriberConfig};

/// 基準測試：Event 創建
fn bench_event_creation(c: &mut Criterion) {
    c.bench_function("event_create_small", |b| {
        let payload = b"Hello, World!".to_vec();
        b.iter(|| {
            Event::new(
                black_box("test.topic"),
                black_box("bench-source"),
                black_box(payload.clone())
            ).unwrap()
        })
    });
    
    c.bench_function("event_create_medium", |b| {
        let payload = vec![0u8; 1024]; // 1KB
        b.iter(|| {
            Event::new(
                black_box("test.topic.medium"),
                black_box("bench-source"),
                black_box(payload.clone())
            ).unwrap()
        })
    });
    
    c.bench_function("event_create_large", |b| {
        let payload = vec![0u8; 65536]; // 64KB
        b.iter(|| {
            Event::new(
                black_box("test.topic.large"),
                black_box("bench-source"),
                black_box(payload.clone())
            ).unwrap()
        })
    });
}

/// 基準測試：Event 序列化
fn bench_event_serialization(c: &mut Criterion) {
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize, Clone)]
    struct TestPayload {
        id: u64,
        name: String,
        values: Vec<f64>,
        nested: NestedData,
    }
    
    #[derive(Serialize, Deserialize, Clone)]
    struct NestedData {
        flag: bool,
        count: i32,
        tags: Vec<String>,
    }
    
    let test_data = TestPayload {
        id: 12345,
        name: "benchmark-test".to_string(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        nested: NestedData {
            flag: true,
            count: 100,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        },
    };
    
    c.bench_function("event_serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&test_data)).unwrap();
            black_box(json)
        })
    });
    
    c.bench_function("event_serialize_msgpack", |b| {
        b.iter(|| {
            let msgpack = rmp_serde::to_vec(black_box(&test_data)).unwrap();
            black_box(msgpack)
        })
    });
    
    // 預先序列化用於反序列化測試
    let json_bytes = serde_json::to_vec(&test_data).unwrap();
    let msgpack_bytes = rmp_serde::to_vec(&test_data).unwrap();
    
    c.bench_function("event_deserialize_json", |b| {
        b.iter(|| {
            let data: TestPayload = serde_json::from_slice(black_box(&json_bytes)).unwrap();
            black_box(data)
        })
    });
    
    c.bench_function("event_deserialize_msgpack", |b| {
        b.iter(|| {
            let data: TestPayload = rmp_serde::from_slice(black_box(&msgpack_bytes)).unwrap();
            black_box(data)
        })
    });
}

/// 基準測試：不同 Payload 大小的吞吐量
fn bench_payload_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_scaling");
    
    for size in [64, 256, 1024, 4096, 16384, 65536].iter() {
        let payload = vec![0u8; *size];
        
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("event_create", size),
            size,
            |b, _| {
                b.iter(|| {
                    Event::new(
                        "bench.topic",
                        "bench-source",
                        black_box(payload.clone())
                    ).unwrap()
                })
            }
        );
    }
    
    group.finish();
}

/// 基準測試：Topic 驗證
fn bench_topic_validation(c: &mut Criterion) {
    c.bench_function("topic_valid_short", |b| {
        b.iter(|| {
            Event::new(
                black_box("a.b"),
                "src",
                vec![]
            ).unwrap()
        })
    });
    
    c.bench_function("topic_valid_long", |b| {
        b.iter(|| {
            Event::new(
                black_box("system.module.submodule.component.action"),
                "src",
                vec![]
            ).unwrap()
        })
    });
}

/// 基準測試：Config 創建
fn bench_config_creation(c: &mut Criterion) {
    c.bench_function("publisher_config_default", |b| {
        b.iter(|| {
            black_box(PublisherConfig::default())
        })
    });
    
    c.bench_function("publisher_config_custom", |b| {
        b.iter(|| {
            black_box(PublisherConfig {
                bind_addr: "127.0.0.1:9000".to_string(),
                channel_capacity: 2048,
                max_connections: 500,
            })
        })
    });
    
    c.bench_function("subscriber_config_custom", |b| {
        b.iter(|| {
            black_box(SubscriberConfig {
                publisher_addr: "127.0.0.1:9000".to_string(),
                topics: vec![
                    "system.*".to_string(),
                    "llm.*".to_string(),
                    "auth.*".to_string(),
                ],
                reconnect_delay_ms: 1000,
                auto_reconnect: true,
                buffer_capacity: 512,
            })
        })
    });
}

/// 基準測試：Event ID 生成（唯一性）
fn bench_event_id_generation(c: &mut Criterion) {
    c.bench_function("event_id_uniqueness", |b| {
        b.iter(|| {
            let e1 = Event::new("t", "s", vec![]).unwrap();
            let e2 = Event::new("t", "s", vec![]).unwrap();
            black_box(e1.id != e2.id)
        })
    });
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_serialization,
    bench_payload_scaling,
    bench_topic_validation,
    bench_config_creation,
    bench_event_id_generation,
);

criterion_main!(benches);
