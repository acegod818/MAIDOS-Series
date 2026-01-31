# MAIDOS P2P Network Module

Provides decentralized peer-to-peer networking using libp2p for the MAIDOS ecosystem.

## Features

- **Node Discovery**: Kademlia DHT-based peer discovery
- **Secure Communication**: Noise protocol encryption
- **Message Broadcasting**: Gossipsub pub/sub messaging
- **NAT Traversal**: Automatic NAT penetration
- **Relay Support**: Relay nodes for connectivity
- **Structured Logging**: Full MAIDOS-AUDIT trail

## Architecture

```
┌─────────────────────────────────────────────┐
│              P2P Node                       │
├─────────────────────────────────────────────┤
│  Network Behaviours:                        │
│  ├── Gossipsub (message broadcasting)       │
│  ├── Kademlia (peer discovery)              │
│  ├── Identify (peer identification)         │
│  └── Ping (connectivity checking)           │
├─────────────────────────────────────────────┤
│  Transport:                                 │
│  ├── TCP                                    │
│  ├── Noise (encryption)                     │
│  └── Yamux (stream multiplexing)            │
└─────────────────────────────────────────────┘
```

## Usage

### Basic Node Creation

```rust
use maidos_p2p::{Node, NodeConfig};

let config = NodeConfig::default();
let mut node = Node::new(config)?;
node.start().await?;
```

### Sending Messages

```rust
use maidos_p2p::{NetworkMessage, Node};

let message = NetworkMessage {
    id: "msg-1".to_string(),
    sender: node.peer_id(),
    topic: "chat".to_string(),
    content: b"Hello, World!".to_vec(),
    timestamp: std::time::SystemTime::now(),
};

node.broadcast(&message).await?;
```

### Receiving Events

```rust
use maidos_p2p::{Node, NodeEvent};

while let Some(event) = node.next_event().await {
    match event {
        NodeEvent::PeerConnected(peer_id) => {
            println!("Peer connected: {}", peer_id);
        }
        NodeEvent::MessageReceived(message) => {
            println!("Received: {:?}", String::from_utf8_lossy(&message.content));
        }
        _ => {}
    }
}
```

## Examples

Run the basic example:
```bash
cd source/maidos-shared
cargo run --example basic_p2p --features p2p
```

Run the two nodes example:
```bash
cd source/maidos-shared
cargo run --example two_nodes --features p2p
```

## Configuration

The `NodeConfig` struct allows customization of:

- Keypair (for node identity)
- Listen addresses
- Bootstrap peers
- Network topics
- Feature toggles (Kademlia, Gossipsub)

## Security

- All communications encrypted with Noise protocol
- Message signing with Ed25519 keys
- Capability-based access control integration
- Structured audit logging for all operations

## MAIDOS-AUDIT Compliance

All critical operations are logged with the `[MAIDOS-AUDIT]` prefix:

- Node startup/shutdown
- Peer connections/disconnections
- Message sending/receiving
- Error conditions

This ensures full traceability and compliance with MAIDOS quality standards.

## Dependencies

- `libp2p` - Core P2P networking library
- `tokio` - Async runtime
- `serde` - Serialization
- `tracing` - Structured logging