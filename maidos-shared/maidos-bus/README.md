# maidos-bus

> ZeroMQ 事件總線 | Pub/Sub 模式 | 非同步

[![crates.io](https://img.shields.io/crates/v/maidos-bus.svg)](https://crates.io/crates/maidos-bus)
[![docs.rs](https://docs.rs/maidos-bus/badge.svg)](https://docs.rs/maidos-bus)

## 功能

- ✅ ZeroMQ Pub/Sub
- ✅ 主題過濾 (Topic Filtering)
- ✅ MessagePack 序列化
- ✅ 非同步 (Tokio)
- ✅ 萬用字元訂閱
- ✅ C FFI 支援

## 使用

```rust
use maidos_bus::{Publisher, Subscriber, Event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立發布者
    let mut publisher = Publisher::bind("tcp://127.0.0.1:5555")?;
    publisher.start().await?;

    // 建立訂閱者
    let mut subscriber = Subscriber::connect("tcp://127.0.0.1:5555")?;
    subscriber.subscribe("config.*")?;
    subscriber.start().await?;

    // 發布事件
    let event = Event::new("config.changed", "config-service", vec![1, 2, 3])?;
    publisher.publish(event).await?;

    // 接收事件
    if let Some(event) = subscriber.recv().await? {
        println!("Received: {}", event.topic);
    }

    Ok(())
}
```

## 主題格式

```
maidos.config.changed    # 精確匹配
maidos.config.*          # 萬用字元匹配
maidos.*                 # 前綴匹配
```

## 型別化事件

```rust
#[derive(Serialize, Deserialize)]
struct ConfigChange {
    key: String,
    old_value: String,
    new_value: String,
}

// 發布
let event = Event::with_data("config.changed", "source", &change)?;
publisher.publish(event).await?;

// 接收
let change: ConfigChange = event.data()?;
```

## FFI

```c
MaidosBusPublisher* pub = maidos_bus_publisher_create("tcp://127.0.0.1:5555");
maidos_bus_publish(pub, "topic", data, len);
maidos_bus_publisher_free(pub);
```

## License

MIT
