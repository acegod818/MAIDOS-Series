//! Bus Pub/Sub Example
//!
//! <impl>
//! WHAT: 展示 maidos-bus 事件處理
//! WHY: 讓開發者理解訊息匯流排
//! HOW: Event 創建、配置、序列化
//! </impl>

use maidos_bus::{Event, PublisherConfig, SubscriberConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MAIDOS Bus Pub/Sub Example ===\n");

    // 創建 Event
    println!("【創建 Event】");
    let event = Event::new(
        "system.status",
        "health-monitor",
        b"System is healthy".to_vec(),
    )?;
    
    println!("  ID: {}", event.id);
    println!("  Topic: {}", event.topic);
    println!("  Source: {}", event.source);
    println!("  Timestamp: {}", event.timestamp);
    println!("  Payload: {} bytes", event.payload.len());

    // Event ID 唯一性
    println!("\n【Event ID 唯一性】");
    let e1 = Event::new("test", "src", vec![])?;
    let e2 = Event::new("test", "src", vec![])?;
    let e3 = Event::new("test", "src", vec![])?;
    
    println!("  Event 1: {}", e1.id);
    println!("  Event 2: {}", e2.id);
    println!("  Event 3: {}", e3.id);
    println!("  所有 ID 唯一: {}", e1.id != e2.id && e2.id != e3.id);

    // 不同大小的 Payload
    println!("\n【不同大小 Payload】");
    let sizes = [64, 1024, 4096, 65536];
    for size in sizes {
        let payload = vec![0u8; size];
        let event = Event::new("bench.topic", "bench", payload)?;
        println!("  {} bytes: ID={}", size, event.id);
    }

    // Topic 命名規範
    println!("\n【Topic 命名規範】");
    let valid_topics = [
        "system.status",
        "llm.chat.response",
        "auth.token.issued",
        "bus.subscriber.connected",
    ];
    
    for topic in valid_topics {
        let event = Event::new(topic, "demo", vec![])?;
        println!("  ✓ {}", event.topic);
    }

    // Publisher 配置
    println!("\n【Publisher 配置】");
    let pub_config = PublisherConfig {
        bind_addr: "127.0.0.1:9000".to_string(),
        channel_capacity: 1024,
        max_connections: 100,
    };
    
    println!("  Bind Address: {}", pub_config.bind_addr);
    println!("  Channel Capacity: {}", pub_config.channel_capacity);
    println!("  Max Connections: {}", pub_config.max_connections);

    // Subscriber 配置
    println!("\n【Subscriber 配置】");
    let sub_config = SubscriberConfig {
        publisher_addr: "tcp://127.0.0.1:9000".to_string(),
        topics: vec![
            "system.*".to_string(),
            "llm.*".to_string(),
        ],
        reconnect_delay_ms: 5000,
        auto_reconnect: true,
        buffer_capacity: 256,
    };
    
    println!("  Publisher Address: {}", sub_config.publisher_addr);
    println!("  Topics:");
    for topic in &sub_config.topics {
        println!("    - {}", topic);
    }
    println!("  Auto Reconnect: {}", sub_config.auto_reconnect);

    // 預設配置
    println!("\n【預設配置】");
    let default_pub = PublisherConfig::default();
    println!("  Publisher Default:");
    println!("    bind_addr: {}", default_pub.bind_addr);

    // Event 序列化
    println!("\n【Event 序列化】");
    let event = Event::new("demo.serialize", "example", b"Hello".to_vec())?;
    let json = serde_json::to_string_pretty(&event)?;
    println!("  JSON Preview:");
    for line in json.lines().take(5) {
        println!("    {}", line);
    }

    println!("\n=== 範例完成 ===");
    Ok(())
}
