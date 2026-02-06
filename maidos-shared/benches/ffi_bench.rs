//! FFI Performance Benchmarks
//!
//! <impl>
//! WHAT: 跨語言 FFI 調用效能基準測試
//! WHY: 測量 C FFI 調用開銷，驗證跨語言效能可接受
//! HOW: 使用 Criterion 比較 Rust 原生 vs FFI 調用延遲
//! METRICS: 調用開銷 (ns)、吞吐量 (ops/sec)、記憶體分配開銷
//! NOTE: Bus/LLM FFI 涉及 async runtime，本測試專注於同步 FFI 操作
//! </impl>

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::ffi::CString;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

// Re-export from crates for comparison
use maidos_auth::{Capability, CapabilitySet, CapabilityToken, TokenIssuer};
use maidos_bus::Event;
use maidos_config::MaidosConfig;
use maidos_llm::{CompletionRequest, Message};

// ============================================================================
// Test Fixture
// ============================================================================

struct TestFixture {
    _temp_dir: TempDir,
    config_path: std::path::PathBuf,
}

impl TestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("bench.toml");

        fs::write(
            &config_path,
            r#"
[maidos]
version = "1.0"

[llm]
default_provider = "anthropic"
budget_daily = 100.0
budget_monthly = 2000.0

[llm.providers.anthropic]
model = "claude-3-opus"
timeout_secs = 60

[llm.providers.openai]
model = "gpt-4"
timeout_secs = 30

[bus]
endpoint = "tcp://127.0.0.1:5555"
buffer_size = 1024
reconnect_ms = 5000

[auth]
token_ttl = 86400
"#,
        )
        .expect("Failed to write config");

        Self {
            _temp_dir: temp_dir,
            config_path,
        }
    }
}

// ============================================================================
// Config: Native vs FFI Comparison
// ============================================================================

fn bench_config_native_vs_ffi(c: &mut Criterion) {
    let fixture = TestFixture::new();
    let mut group = c.benchmark_group("config_native_vs_ffi");

    // --- Native Config Load ---
    group.bench_function("native_load", |b| {
        b.iter(|| {
            let config = MaidosConfig::load(black_box(&fixture.config_path)).unwrap();
            black_box(config);
        })
    });

    // --- FFI-style Config Load (simulated via direct call) ---
    // Note: We call the native function but through the same path FFI would take
    group.bench_function("ffi_style_load", |b| {
        b.iter(|| {
            // Simulate FFI: convert path string, call function, return result
            let path_str = fixture.config_path.to_str().unwrap();
            let config = MaidosConfig::load(black_box(std::path::Path::new(path_str))).unwrap();
            black_box(config);
        })
    });

    // --- Native Config Access ---
    let config = MaidosConfig::load(&fixture.config_path).unwrap();

    group.bench_function("native_access_string", |b| {
        b.iter(|| {
            let value = black_box(config.llm().default_provider);
            black_box(value.clone());
        })
    });

    group.bench_function("native_access_f64", |b| {
        b.iter(|| {
            let value = black_box(config.llm().budget_daily);
            black_box(value);
        })
    });

    group.bench_function("native_access_u64", |b| {
        b.iter(|| {
            let value = black_box(config.auth().token_ttl);
            black_box(value);
        })
    });

    // --- Native JSON Serialization ---
    let schema = config.schema();
    group.bench_function("native_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&schema)).unwrap();
            black_box(json);
        })
    });

    group.finish();
}

// ============================================================================
// Auth: Native vs FFI Token Operations
// ============================================================================

fn bench_auth_native_vs_ffi(c: &mut Criterion) {
    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret.clone(), Duration::from_secs(3600));

    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmVision);

    let mut group = c.benchmark_group("auth_native_vs_ffi");

    // --- Native Token Issue ---
    group.bench_function("native_token_issue", |b| {
        b.iter(|| {
            let token = issuer.issue(black_box(caps)).unwrap();
            black_box(token);
        })
    });

    // --- FFI-style Token Issue (simulates crossing FFI boundary) ---
    group.bench_function("ffi_style_token_issue", |b| {
        let caps_u32 = caps.as_u32();
        b.iter(|| {
            // Simulate FFI: receive raw values, convert, call, convert back
            let caps_from_ffi = CapabilitySet::from_u32(black_box(caps_u32));
            let token = CapabilityToken::new(
                caps_from_ffi,
                Duration::from_secs(black_box(3600)),
                black_box(&secret),
            )
            .unwrap();
            // Simulate returning string across FFI
            let token_str = token.as_str().to_string();
            black_box(token_str);
        })
    });

    // --- Native Token Verify ---
    let token = issuer.issue(caps).unwrap();
    let token_str = token.as_str().to_string();

    group.bench_function("native_token_verify", |b| {
        b.iter(|| {
            let verified = issuer.verify(black_box(&token_str)).unwrap();
            black_box(verified);
        })
    });

    // --- FFI-style Token Verify ---
    group.bench_function("ffi_style_token_verify", |b| {
        b.iter(|| {
            // Simulate FFI: receive string pointer, call verify, return u32
            let verified = CapabilityToken::verify(black_box(&token_str), black_box(&secret)).unwrap();
            let caps_out = verified.capabilities().as_u32();
            black_box(caps_out);
        })
    });

    // --- Native Capability Check ---
    group.bench_function("native_capability_check", |b| {
        b.iter(|| {
            let has = issuer
                .check(black_box(&token_str), black_box(Capability::LlmChat));
            black_box(has);
        })
    });

    // --- FFI-style Capability Check ---
    group.bench_function("ffi_style_capability_check", |b| {
        let cap_u32 = Capability::LlmChat as u32;
        b.iter(|| {
            let verified =
                CapabilityToken::verify(black_box(&token_str), black_box(&secret)).unwrap();
            let has = (verified.capabilities().as_u32() & black_box(cap_u32)) != 0;
            black_box(has);
        })
    });

    // --- Token Roundtrip Comparison ---
    group.bench_function("native_token_roundtrip", |b| {
        b.iter(|| {
            let token = issuer.issue(black_box(caps)).unwrap();
            let verified = issuer.verify(token.as_str()).unwrap();
            black_box(verified);
        })
    });

    group.bench_function("ffi_style_token_roundtrip", |b| {
        let caps_u32 = caps.as_u32();
        b.iter(|| {
            // Issue
            let caps_from_ffi = CapabilitySet::from_u32(black_box(caps_u32));
            let token = CapabilityToken::new(
                caps_from_ffi,
                Duration::from_secs(black_box(3600)),
                black_box(&secret),
            )
            .unwrap();
            let token_str = token.as_str().to_string();
            // Verify
            let verified =
                CapabilityToken::verify(black_box(&token_str), black_box(&secret)).unwrap();
            let caps_out = verified.capabilities().as_u32();
            black_box(caps_out);
        })
    });

    group.finish();
}

// ============================================================================
// Bus: Event Creation and Serialization
// ============================================================================

fn bench_bus_event_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bus_event_ops");

    // --- Small Event Creation ---
    let small_payload = b"small payload".to_vec();

    group.bench_function("event_create_small", |b| {
        b.iter(|| {
            let event = Event::new(
                black_box("test.event"),
                black_box("bench"),
                black_box(small_payload.clone()),
            )
            .unwrap();
            black_box(event);
        })
    });

    // --- Large Event Creation ---
    let large_payload = vec![0u8; 64 * 1024]; // 64KB

    group.bench_function("event_create_64kb", |b| {
        b.iter(|| {
            let event = Event::new(
                black_box("test.event"),
                black_box("bench"),
                black_box(large_payload.clone()),
            )
            .unwrap();
            black_box(event);
        })
    });

    // --- Event Serialization ---
    let event = Event::new("test.event", "bench", small_payload.clone()).unwrap();

    group.bench_function("event_serialize_small", |b| {
        b.iter(|| {
            let bytes = event.to_bytes().unwrap();
            black_box(bytes);
        })
    });

    let large_event = Event::new("test.event", "bench", large_payload.clone()).unwrap();

    group.bench_function("event_serialize_64kb", |b| {
        b.iter(|| {
            let bytes = large_event.to_bytes().unwrap();
            black_box(bytes);
        })
    });

    // --- Event Deserialization ---
    let event_bytes = event.to_bytes().unwrap();

    group.bench_function("event_deserialize_small", |b| {
        b.iter(|| {
            let event = Event::from_bytes(black_box(&event_bytes)).unwrap();
            black_box(event);
        })
    });

    let large_event_bytes = large_event.to_bytes().unwrap();

    group.bench_function("event_deserialize_64kb", |b| {
        b.iter(|| {
            let event = Event::from_bytes(black_box(&large_event_bytes)).unwrap();
            black_box(event);
        })
    });

    // --- Event Roundtrip ---
    group.bench_function("event_roundtrip_small", |b| {
        b.iter(|| {
            let event = Event::new("test.event", "bench", small_payload.clone()).unwrap();
            let bytes = event.to_bytes().unwrap();
            let decoded = Event::from_bytes(&bytes).unwrap();
            black_box(decoded);
        })
    });

    group.finish();
}

// ============================================================================
// LLM: Request Building
// ============================================================================

fn bench_llm_request_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_request_ops");

    // --- Simple Request ---
    group.bench_function("request_create_simple", |b| {
        b.iter(|| {
            let request = CompletionRequest::new(black_box("claude-3-opus"))
                .system(black_box("You are a helpful assistant."))
                .message(Message::user(black_box("Hello")));
            black_box(request);
        })
    });

    // --- Multi-turn Request ---
    group.bench_function("request_create_multiturn", |b| {
        b.iter(|| {
            let request = CompletionRequest::new(black_box("claude-3-opus"))
                .system(black_box("You are a helpful assistant."))
                .message(Message::user(black_box("Hello")))
                .message(Message::assistant(black_box("Hi! How can I help?")))
                .message(Message::user(black_box("Tell me about Rust")))
                .message(Message::assistant(black_box("Rust is a systems programming language...")))
                .message(Message::user(black_box("What about async?")));
            black_box(request);
        })
    });

    // --- Request Serialization ---
    let request = CompletionRequest::new("claude-3-opus")
        .system("You are a helpful assistant.")
        .message(Message::user("Hello"))
        .max_tokens(1024)
        .temperature(0.7);

    group.bench_function("request_to_json", |b| {
        b.iter(|| {
            // Simulate what FFI would do: serialize to JSON
            let json = serde_json::to_string(black_box(&request)).unwrap();
            black_box(json);
        })
    });

    group.finish();
}

// ============================================================================
// String Conversion Overhead (FFI Critical Path)
// ============================================================================

fn bench_string_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_conversion");

    // --- CString Creation ---
    let rust_string = "hello world from rust".to_string();

    group.bench_function("cstring_from_string", |b| {
        b.iter(|| {
            let cstring = CString::new(black_box(rust_string.clone())).unwrap();
            black_box(cstring);
        })
    });

    // --- CString to String ---
    let cstring = CString::new("hello world from C").unwrap();

    group.bench_function("cstring_to_string", |b| {
        b.iter(|| {
            let rust_str = black_box(&cstring).to_str().unwrap().to_string();
            black_box(rust_str);
        })
    });

    // --- Pointer to String (FFI pattern) ---
    group.bench_function("ptr_to_string", |b| {
        let ptr = cstring.as_ptr();
        b.iter(|| unsafe {
            let c_str = std::ffi::CStr::from_ptr(black_box(ptr));
            let rust_str = c_str.to_str().unwrap().to_string();
            black_box(rust_str);
        })
    });

    // --- Long String Conversion ---
    let long_string = "x".repeat(4096);

    group.throughput(Throughput::Bytes(4096));
    group.bench_function("cstring_4kb", |b| {
        b.iter(|| {
            let cstring = CString::new(black_box(long_string.clone())).unwrap();
            let ptr = cstring.as_ptr();
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(ptr);
                let back = c_str.to_str().unwrap().to_string();
                black_box(back);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Memory Allocation Patterns
// ============================================================================

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // --- Box Allocation (FFI handle pattern) ---
    #[derive(Clone)]
    #[allow(dead_code)]
    struct LargeStruct {
        data: [u8; 1024],
        name: String,
    }

    let large = LargeStruct {
        data: [0u8; 1024],
        name: "test".to_string(),
    };

    group.bench_function("box_alloc_free", |b| {
        b.iter(|| {
            let boxed = Box::new(black_box(large.clone()));
            let ptr = Box::into_raw(boxed);
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        })
    });

    // --- Vec Allocation and Transfer ---
    for size in [64, 1024, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("vec_alloc_copy", size), size, |b, &size| {
            let data = vec![0u8; size];
            b.iter(|| {
                // Simulate FFI: allocate, copy, return pointer
                let mut copy = vec![0u8; size];
                copy.copy_from_slice(black_box(&data));
                let ptr = copy.as_mut_ptr();
                let len = copy.len();
                std::mem::forget(copy);
                // Simulate C caller freeing
                unsafe {
                    let _ = Vec::from_raw_parts(ptr, len, len);
                }
            })
        });
    }

    group.finish();
}

// ============================================================================
// FFI Overhead Summary
// ============================================================================

fn bench_ffi_overhead_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_overhead_summary");

    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();

    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);
    caps.grant(Capability::LlmVision);

    // Measure the actual FFI overhead components
    
    // 1. Pure function call (baseline)
    group.bench_function("baseline_fn_call", |b| {
        #[inline(never)]
        fn pure_add(a: u32, b: u32) -> u32 {
            a.wrapping_add(b)
        }
        b.iter(|| {
            let result = pure_add(black_box(42), black_box(100));
            black_box(result);
        })
    });

    // 2. extern "C" function call overhead
    group.bench_function("extern_c_fn_call", |b| {
        #[no_mangle]
        extern "C" fn extern_add(a: u32, b: u32) -> u32 {
            a.wrapping_add(b)
        }
        b.iter(|| {
            let result = extern_add(black_box(42), black_box(100));
            black_box(result);
        })
    });

    // 3. String parameter passing
    group.bench_function("string_param_overhead", |b| {
        let cstring = CString::new("test string").unwrap();
        let ptr = cstring.as_ptr();
        b.iter(|| unsafe {
            let s = std::ffi::CStr::from_ptr(black_box(ptr));
            let len = s.to_bytes().len();
            black_box(len);
        })
    });

    // 4. Box allocation overhead
    group.bench_function("handle_alloc_overhead", |b| {
        b.iter(|| {
            let handle = Box::new(black_box(42u64));
            let ptr = Box::into_raw(handle);
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        })
    });

    // 5. Complete token operation (represents typical FFI call)
    group.bench_function("typical_ffi_operation", |b| {
        let caps_u32 = caps.as_u32();
        b.iter(|| {
            // Simulate complete FFI cycle:
            // 1. Receive parameters
            let caps_param = black_box(caps_u32);
            let ttl_param = black_box(3600u64);
            // 2. Convert to Rust types
            let caps_set = CapabilitySet::from_u32(caps_param);
            // 3. Execute operation
            let token = CapabilityToken::new(caps_set, Duration::from_secs(ttl_param), &secret).unwrap();
            // 4. Convert result to FFI format
            let result_str = token.as_str().to_string();
            let cstring = CString::new(result_str).unwrap();
            // 5. Return (would be into_raw in real FFI)
            black_box(cstring);
        })
    });

    group.finish();
}

// ============================================================================
// Throughput Tests
// ============================================================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let secret = b"benchmark-secret-key-32-bytes!!".to_vec();
    let issuer = TokenIssuer::new(secret.clone(), Duration::from_secs(3600));

    let mut caps = CapabilitySet::empty();
    caps.grant(Capability::LlmChat);

    // Token operations throughput
    group.throughput(Throughput::Elements(1));

    group.bench_function("tokens_per_sec", |b| {
        b.iter(|| {
            let token = issuer.issue(black_box(caps)).unwrap();
            let _ = issuer.verify(token.as_str()).unwrap();
        })
    });

    // Event serialization throughput
    let payload_sizes = [64, 1024, 4096, 16384];

    for size in payload_sizes.iter() {
        let payload = vec![0u8; *size];
        let event = Event::new("test.event", "bench", payload).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("event_serialize_bytes", size),
            size,
            |b, _| {
                b.iter(|| {
                    let bytes = event.to_bytes().unwrap();
                    black_box(bytes);
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    benches,
    bench_config_native_vs_ffi,
    bench_auth_native_vs_ffi,
    bench_bus_event_operations,
    bench_llm_request_building,
    bench_string_conversion,
    bench_memory_patterns,
    bench_ffi_overhead_summary,
    bench_throughput,
);

criterion_main!(benches);
