//! MAIDOS Cross-process Event Bus
//!
//! A lightweight, zero-dependency pub/sub message bus for inter-process
//! communication. Uses TCP sockets with MessagePack serialization.
//!
//! <impl>
//! WHAT: Event-driven message bus with topic-based routing
//! WHY: Decouple MAIDOS components across processes/languages
//! HOW: TCP server/client with broadcast channel, async runtime
//! TEST: Unit tests per module + integration tests
//! </impl>
//!
//! # Example
//!
//! ```rust,no_run
//! use maidos_bus::{Publisher, Subscriber, Event};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Start publisher
//!     let mut publisher = Publisher::with_defaults();
//!     publisher.start().await.unwrap();
//!     let addr = publisher.bound_addr().await.unwrap();
//!
//!     // Connect subscriber
//!     let mut subscriber = Subscriber::connect_to(addr.to_string());
//!     subscriber.start().await.unwrap();
//!
//!     // Publish event
//!     let event = Event::new("my.topic", "source", vec![1, 2, 3]).unwrap();
//!     publisher.publish(event).await.unwrap();
//!
//!     // Receive event
//!     let received = subscriber.recv().await;
//! }
//! ```

pub mod error;
pub mod event;
pub mod ffi;
pub mod publisher;
pub mod subscriber;

// Re-exports for convenience
pub use error::{BusError, Result};
pub use event::Event;
pub use publisher::{Publisher, PublisherConfig};
pub use subscriber::{Subscriber, SubscriberConfig, SubscriberState};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_full_roundtrip() {
        // Start publisher
        let mut publisher = Publisher::with_defaults();
        publisher.start().await.unwrap();
        let addr = publisher.bound_addr().await.unwrap();

        // Start subscriber
        let mut subscriber = Subscriber::connect_to(addr.to_string());
        subscriber.start().await.unwrap();

        // Wait for connection
        sleep(Duration::from_millis(100)).await;

        // Publish multiple events
        for i in 0..5 {
            let event = Event::new(
                format!("test.event.{}", i),
                "integration-test",
                vec![i as u8],
            )
            .unwrap();
            publisher.publish(event).await.unwrap();
        }

        // Receive all events
        for i in 0..5 {
            let received = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
                .await
                .unwrap();

            assert!(received.is_some());
            let event = received.unwrap();
            assert_eq!(event.topic, format!("test.event.{}", i));
            assert_eq!(event.payload, vec![i as u8]);
        }

        assert_eq!(publisher.events_published().await, 5);
        assert_eq!(subscriber.events_received().await, 5);

        // Cleanup
        subscriber.stop().await.unwrap();
        publisher.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let mut publisher = Publisher::with_defaults();
        publisher.start().await.unwrap();
        let addr = publisher.bound_addr().await.unwrap();

        // Create two subscribers
        let mut sub1 = Subscriber::connect_to(addr.to_string());
        let mut sub2 = Subscriber::connect_to(addr.to_string());

        sub1.start().await.unwrap();
        sub2.start().await.unwrap();

        sleep(Duration::from_millis(100)).await;

        // Both should be connected
        assert_eq!(publisher.connection_count().await, 2);

        // Publish event
        let event = Event::new("broadcast.test", "src", vec![99]).unwrap();
        publisher.publish(event).await.unwrap();

        // Both should receive it
        let r1 = tokio::time::timeout(Duration::from_secs(1), sub1.recv())
            .await
            .unwrap();
        let r2 = tokio::time::timeout(Duration::from_secs(1), sub2.recv())
            .await
            .unwrap();

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert_eq!(r1.unwrap().topic, "broadcast.test");
        assert_eq!(r2.unwrap().topic, "broadcast.test");

        sub1.stop().await.unwrap();
        sub2.stop().await.unwrap();
        publisher.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_typed_event_data() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct ConfigChange {
            key: String,
            old_value: Option<String>,
            new_value: String,
        }

        let mut publisher = Publisher::with_defaults();
        publisher.start().await.unwrap();
        let addr = publisher.bound_addr().await.unwrap();

        let mut subscriber = Subscriber::connect_to(addr.to_string());
        subscriber.start().await.unwrap();

        sleep(Duration::from_millis(100)).await;

        // Publish typed event
        let change = ConfigChange {
            key: "api.endpoint".to_string(),
            old_value: Some("http://old".to_string()),
            new_value: "http://new".to_string(),
        };

        let event = Event::with_data("config.changed", "config-service", &change).unwrap();
        publisher.publish(event).await.unwrap();

        // Receive and deserialize
        let received = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
            .await
            .unwrap()
            .unwrap();

        let decoded: ConfigChange = received.data().unwrap();
        assert_eq!(decoded, change);

        subscriber.stop().await.unwrap();
        publisher.stop().await.unwrap();
    }
}
