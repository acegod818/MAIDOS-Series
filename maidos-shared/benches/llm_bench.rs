//! LLM Module Benchmarks
//!
//! <impl>
//! WHAT: LLM 模組效能基準測試
//! WHY: 測量請求構建、訊息處理效能
//! HOW: 使用 Criterion 框架
//! METRICS: Message 創建、Request 構建、序列化 ops/sec
//! </impl>

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use maidos_llm::{CompletionRequest, Message, Role};
use maidos_llm::streaming::{SseParser, StreamChunk, StreamUsage};
use maidos_llm::tool::{MaidosTool, ToolParameter, ToProviderFormat};

/// 基準測試：Message 創建
fn bench_message_creation(c: &mut Criterion) {
    c.bench_function("message_user_short", |b| {
        b.iter(|| {
            Message::user(black_box("Hello, how are you?"))
        })
    });
    
    c.bench_function("message_user_long", |b| {
        let content = "a".repeat(1000);
        b.iter(|| {
            Message::user(black_box(&content))
        })
    });
    
    c.bench_function("message_assistant", |b| {
        b.iter(|| {
            Message::assistant(black_box("I'm doing well, thank you for asking!"))
        })
    });
    
    c.bench_function("message_system", |b| {
        b.iter(|| {
            Message::system(black_box("You are a helpful AI assistant."))
        })
    });
}

/// 基準測試：Request 構建
fn bench_request_building(c: &mut Criterion) {
    c.bench_function("request_minimal", |b| {
        b.iter(|| {
            CompletionRequest {
                model: black_box("gpt-4o".to_string()),
                messages: vec![Message::user("Hello")],
                system: None,
                temperature: None,
                max_tokens: None,
                stream: false,
                stop: None,
                top_p: None,
            }
        })
    });
    
    c.bench_function("request_full", |b| {
        b.iter(|| {
            CompletionRequest {
                model: black_box("gpt-4o".to_string()),
                messages: vec![
                    Message::user("What is 2+2?"),
                    Message::assistant("4"),
                    Message::user("And 3+3?"),
                ],
                system: Some("You are a math tutor.".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(2048),
                stream: true,
                stop: Some(vec!["END".to_string()]),
                top_p: Some(0.9),
            }
        })
    });
}

/// 基準測試：多輪對話構建
fn bench_conversation_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversation");
    
    for msg_count in [1, 5, 10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*msg_count as u64));
        group.bench_with_input(
            BenchmarkId::new("build", msg_count),
            msg_count,
            |b, &count| {
                b.iter(|| {
                    let mut messages = Vec::with_capacity(count);
                    for i in 0..count {
                        if i % 2 == 0 {
                            messages.push(Message::user(format!("User message {}", i)));
                        } else {
                            messages.push(Message::assistant(format!("Assistant response {}", i)));
                        }
                    }
                    CompletionRequest {
                        model: "gpt-4o".to_string(),
                        messages,
                        system: Some("System prompt".to_string()),
                        temperature: Some(0.7),
                        max_tokens: Some(2048),
                        stream: false,
                        stop: None,
                        top_p: None,
                    }
                })
            }
        );
    }
    
    group.finish();
}

/// 基準測試：Request 序列化
fn bench_request_serialization(c: &mut Criterion) {
    let request = CompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            Message::user("Hello, how are you?"),
            Message::assistant("I'm doing well!"),
            Message::user("Can you help me with something?"),
        ],
        system: Some("You are a helpful assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2048),
        stream: false,
        stop: None,
        top_p: Some(0.9),
    };
    
    c.bench_function("request_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&request)).unwrap();
            black_box(json)
        })
    });
    
    let json_str = serde_json::to_string(&request).unwrap();
    
    c.bench_function("request_from_json", |b| {
        b.iter(|| {
            let req: CompletionRequest = serde_json::from_str(black_box(&json_str)).unwrap();
            black_box(req)
        })
    });
}

/// 基準測試：Message 內容大小擴展性
fn bench_message_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_scaling");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let content = "x".repeat(*size);
        
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("create", size),
            &content,
            |b, content| {
                b.iter(|| {
                    Message::user(black_box(content))
                })
            }
        );
    }
    
    group.finish();
}

/// 基準測試：Role 操作
fn bench_role_operations(c: &mut Criterion) {
    c.bench_function("role_comparison", |b| {
        let role1 = Role::User;
        let role2 = Role::Assistant;
        b.iter(|| {
            black_box(role1 == role2)
        })
    });
    
    c.bench_function("role_clone", |b| {
        let role = Role::System;
        b.iter(|| {
            black_box(role.clone())
        })
    });
}

/// 基準測試：完整請求處理流程
fn bench_full_request_flow(c: &mut Criterion) {
    c.bench_function("full_request_flow", |b| {
        b.iter(|| {
            // 構建請求
            let request = CompletionRequest {
                model: "claude-sonnet-4-20250514".to_string(),
                messages: vec![
                    Message::user("Explain quantum computing in simple terms."),
                ],
                system: Some("You are a science educator.".to_string()),
                temperature: Some(0.5),
                max_tokens: Some(4096),
                stream: true,
                stop: None,
                top_p: None,
            };
            
            // 序列化
            let json = serde_json::to_string(&request).unwrap();
            
            // 反序列化（模擬接收響應後的處理）
            let _: CompletionRequest = serde_json::from_str(&json).unwrap();
            
            black_box(json.len())
        })
    });
}

// ============================================================================
// v0.2.0 Benchmarks
// ============================================================================

/// 基準測試：SSE 解析 (v0.2.0)
fn bench_sse_parsing(c: &mut Criterion) {
    c.bench_function("sse_parse_single_chunk", |b| {
        let data = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
        b.iter(|| {
            let mut parser = SseParser::new();
            let events = parser.parse(black_box(data));
            black_box(events)
        })
    });
    
    c.bench_function("sse_parse_multiple_chunks", |b| {
        let data = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\ndata: [DONE]\n\n";
        b.iter(|| {
            let mut parser = SseParser::new();
            let events = parser.parse(black_box(data));
            black_box(events)
        })
    });
    
    let mut group = c.benchmark_group("sse_scaling");
    for chunk_count in [10, 50, 100].iter() {
        let mut data = Vec::new();
        for i in 0..*chunk_count {
            data.extend_from_slice(format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":\"word{}\"}}}}]}}\n\n", i
            ).as_bytes());
        }
        data.extend_from_slice(b"data: [DONE]\n\n");
        
        group.throughput(Throughput::Elements(*chunk_count as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", chunk_count),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut parser = SseParser::new();
                    let events = parser.parse(black_box(data));
                    black_box(events)
                })
            }
        );
    }
    group.finish();
}

/// 基準測試：StreamChunk 操作 (v0.2.0)
fn bench_stream_chunk(c: &mut Criterion) {
    c.bench_function("stream_chunk_text", |b| {
        b.iter(|| {
            StreamChunk::text(black_box("Hello, world!"))
        })
    });
    
    c.bench_function("stream_chunk_with_usage", |b| {
        b.iter(|| {
            StreamChunk {
                delta: "response".to_string(),
                finish_reason: Some("stop".to_string()),
                usage: Some(StreamUsage::new(100, 50)),
                tool_call: None,
            }
        })
    });
    
    c.bench_function("stream_usage_new", |b| {
        b.iter(|| {
            StreamUsage::new(black_box(1000), black_box(500))
        })
    });
}

/// 基準測試：MaidosTool 格式轉換 (v0.2.0)
fn bench_tool_format(c: &mut Criterion) {
    let tool = MaidosTool::new("get_weather", "Get current weather for a location")
        .parameter(
            ToolParameter::string("location")
                .description("City name, e.g. 'Tokyo'")
                .required(true)
        )
        .parameter(
            ToolParameter::string("unit")
                .enum_values(vec!["celsius", "fahrenheit"])
        );
    
    c.bench_function("tool_to_openai", |b| {
        b.iter(|| {
            black_box(tool.to_openai())
        })
    });
    
    c.bench_function("tool_to_anthropic", |b| {
        b.iter(|| {
            black_box(tool.to_anthropic())
        })
    });
    
    c.bench_function("tool_to_google", |b| {
        b.iter(|| {
            black_box(tool.to_google())
        })
    });
    
    c.bench_function("tool_to_cohere", |b| {
        b.iter(|| {
            black_box(tool.to_cohere())
        })
    });
    
    c.bench_function("tool_to_mistral", |b| {
        b.iter(|| {
            black_box(tool.to_mistral())
        })
    });
}

/// 基準測試：Tool 構建 (v0.2.0)
fn bench_tool_building(c: &mut Criterion) {
    c.bench_function("tool_minimal", |b| {
        b.iter(|| {
            MaidosTool::new(
                black_box("simple_tool"),
                black_box("A simple tool")
            )
        })
    });
    
    c.bench_function("tool_with_params", |b| {
        b.iter(|| {
            MaidosTool::new("complex_tool", "A complex tool")
                .parameter(ToolParameter::string("name").required(true))
                .parameter(ToolParameter::number("count").min(0.0).max(100.0))
                .parameter(ToolParameter::boolean("enabled"))
                .parameter(ToolParameter::array("items", ToolParameter::string("item")))
        })
    });
    
    let mut group = c.benchmark_group("tool_params_scaling");
    for param_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*param_count as u64));
        group.bench_with_input(
            BenchmarkId::new("build", param_count),
            param_count,
            |b, &count| {
                b.iter(|| {
                    let mut tool = MaidosTool::new("scaled_tool", "Tool with many params");
                    for i in 0..count {
                        tool = tool.parameter(
                            ToolParameter::string(format!("param_{}", i))
                                .description(format!("Parameter {}", i))
                                .required(i < 3)
                        );
                    }
                    black_box(tool)
                })
            }
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_request_building,
    bench_conversation_building,
    bench_request_serialization,
    bench_message_scaling,
    bench_role_operations,
    bench_full_request_flow,
    // v0.2.0 benchmarks
    bench_sse_parsing,
    bench_stream_chunk,
    bench_tool_format,
    bench_tool_building,
);

criterion_main!(benches);
