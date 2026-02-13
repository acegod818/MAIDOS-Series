# State Models - maidos-shared

## SM-001: Token Lifecycle (maidos-auth)

### States

| State | Description | Allowed Transitions |
|-------|-------------|---------------------|
| **Created** | Token issued but not yet used | → Active (on first verification) |
| **Active** | Token verified and in use | → Expired (TTL elapsed), → Revoked (explicit revocation) |
| **Expired** | Token TTL exceeded | Terminal state |
| **Revoked** | Token explicitly revoked by issuer | Terminal state |

### State Diagram

```
┌─────────┐
│ Created │
└────┬────┘
     │ verify()
     v
┌────────┐    TTL elapsed    ┌─────────┐
│ Active ├──────────────────>│ Expired │
└───┬────┘                   └─────────┘
    │
    │ revoke()
    v
┌─────────┐
│ Revoked │
└─────────┘
```

### Transitions

| From | To | Trigger | Guards | Actions |
|------|-----|---------|--------|---------|
| Created | Active | `verify()` | Token signature valid, not expired | Add to active token store |
| Active | Expired | System clock | Current time > `exp` claim | Remove from active store |
| Active | Revoked | `revoke()` | Token exists in store | Add to revocation list, remove from active store |

### Invariants

- A token can only exist in one state at a time
- Terminal states (Expired, Revoked) cannot transition to other states
- Token expiration is monotonic (exp claim never increases)

### Example

```rust
use maidos_auth::{TokenIssuer, TokenState};

let issuer = TokenIssuer::new(secret);
let token = issuer.issue("user_123", caps, Duration::from_secs(3600))?;
// State: Created

let claims = issuer.verify(&token)?;
// State: Active

// Wait 3600 seconds
let result = issuer.verify(&token);
// State: Expired
assert!(matches!(result, Err(AuthError::TokenExpired)));

// OR manually revoke
issuer.revoke(&token)?;
// State: Revoked
let result = issuer.verify(&token);
assert!(matches!(result, Err(AuthError::TokenRevoked)));
```

---

## SM-002: Bus Connection Lifecycle (maidos-bus)

### States

| State | Description | Allowed Transitions |
|-------|-------------|---------------------|
| **Disconnected** | Initial state, no connection | → Connecting (on connect()) |
| **Connecting** | Establishing connection to bus | → Connected (success), → Disconnected (timeout) |
| **Connected** | Active connection, can send/receive | → Disconnected (socket closed), → Reconnecting (error) |
| **Reconnecting** | Attempting to restore connection | → Connected (success), → Disconnected (max retries) |

### State Diagram

```
┌──────────────┐
│ Disconnected │<───────────────────┐
└──────┬───────┘                    │
       │ connect()         max retries exceeded
       v                            │
┌────────────┐    timeout   ┌──────┴────────┐
│ Connecting ├─────────────>│ Reconnecting  │
└─────┬──────┘               └───────┬───────┘
      │ success                      │ success
      v                              │
┌───────────┐    socket closed       │
│ Connected │<───────────────────────┘
└─────┬─────┘
      │ error
      └──────────────────────┐
                             v
                       ┌──────────────┐
                       │ Reconnecting │
                       └──────────────┘
```

### Transitions

| From | To | Trigger | Guards | Actions |
|------|-----|---------|--------|---------|
| Disconnected | Connecting | `connect()` | Valid address | Open socket, send handshake |
| Connecting | Connected | Handshake response | Response valid | Start message loop |
| Connecting | Disconnected | Timeout | 30s elapsed | Close socket, emit error |
| Connected | Disconnected | Socket closed | Remote disconnect or shutdown | Close socket, emit disconnect event |
| Connected | Reconnecting | Send/receive error | Error is recoverable | Close socket, start retry timer |
| Reconnecting | Connected | Retry attempt | Connection successful | Reset retry count |
| Reconnecting | Disconnected | Max retries | Retry count >= 5 | Close socket, emit fatal error |

### Reconnection Strategy

Exponential backoff with jitter:
- Retry 1: 1 second
- Retry 2: 2 seconds
- Retry 3: 4 seconds
- Retry 4: 8 seconds
- Retry 5: 16 seconds
- Max retries: 5 (total 31 seconds)

### Invariants

- Only Connected state allows publish/subscribe operations
- Reconnecting increments retry count monotonically
- Disconnected state resets retry count to 0

### Example

```rust
use maidos_bus::{Publisher, ConnectionState};

let mut publisher = Publisher::new("tcp://localhost:5555").await?;
// State: Connecting → Connected

publisher.publish(msg).await?;  // OK in Connected state

// Simulate network failure
// State: Connected → Reconnecting (retry 1)
// State: Reconnecting → Reconnecting (retry 2)
// State: Reconnecting → Connected (success)

publisher.publish(msg).await?;  // OK again
```

---

## SM-003: LLM Request Lifecycle (maidos-llm)

### States

| State | Description | Allowed Transitions |
|-------|-------------|---------------------|
| **Pending** | Request created, not yet sent | → Streaming (start request), → Error (validation failure) |
| **Streaming** | Receiving incremental responses | → Complete (done event), → Error (network failure) |
| **Complete** | Request finished successfully | Terminal state |
| **Error** | Request failed with error | Terminal state |

### State Diagram

```
┌─────────┐   validation error   ┌───────┐
│ Pending ├─────────────────────>│ Error │
└────┬────┘                      └───────┘
     │ start request                 ^
     v                               │ network/API error
┌────────────┐    done event        │
│ Streaming  ├──────────────────────┤
└─────┬──────┘                      │
      │                             │
      └─────────────────────────────┘
                  ↓
            ┌──────────┐
            │ Complete │
            └──────────┘
```

### Transitions

| From | To | Trigger | Guards | Actions |
|------|-----|---------|--------|---------|
| Pending | Streaming | `complete_streaming()` | Request valid, API key set | Send HTTP request, open SSE stream |
| Pending | Error | Validation | Model name invalid or API key missing | Emit validation error |
| Streaming | Complete | Done event | SSE stream sends `[DONE]` | Close stream, return usage metadata |
| Streaming | Error | Network failure | Timeout, 4xx/5xx status, or connection drop | Close stream, emit error |

### Streaming Substates

While in Streaming state, the request emits a sequence of MaidosStreamItem events:

1. **TextDelta**: Incremental text chunks
2. **FunctionCall / ToolCall**: Function calling metadata
3. **Done**: Final usage statistics
4. **Error**: Error during streaming

### Invariants

- Pending must transition to either Streaming or Error (no indefinite pending)
- Streaming must eventually reach Complete or Error (no infinite stream)
- Terminal states (Complete, Error) cannot transition further

### Example

```rust
use maidos_llm::{Provider, MaidosStreamItem};

let provider = create_provider(ProviderType::OpenAI, Some(api_key), None)?;
let request = CompletionRequest::quick("Hello");
// State: Pending

let mut stream = provider.complete_streaming(request).await?;
// State: Streaming

while let Some(item) = stream.next().await {
    match item? {
        MaidosStreamItem::TextDelta(text) => print!("{}", text),
        MaidosStreamItem::Done(usage) => {
            // State: Complete
            println!("\nTokens: {}", usage.total_tokens);
        }
        MaidosStreamItem::Error(err) => {
            // State: Error
            eprintln!("Error: {}", err);
            break;
        }
        _ => {}
    }
}
```

---

## SM-004: Config Hot Reload Lifecycle (maidos-config)

### States

| State | Description | Allowed Transitions |
|-------|-------------|---------------------|
| **Loaded** | Config loaded from file, active | → Reloading (file change detected), → Invalid (validation failure) |
| **Reloading** | Loading updated config from file | → Loaded (validation success), → Invalid (validation failure) |
| **Invalid** | Current config failed validation | → Reloading (manual reload attempt) |

### State Diagram

```
┌────────┐    file change detected    ┌───────────┐
│ Loaded │───────────────────────────>│ Reloading │
└───┬────┘<───────────────────────────└─────┬─────┘
    │                     validation success │
    │                                        │
    │ validation failure                     │ validation failure
    v                                        v
┌─────────┐    manual reload()         ┌─────────┐
│ Invalid │───────────────────────────>│ Invalid │
└─────────┘                             └─────────┘
```

### Transitions

| From | To | Trigger | Guards | Actions |
|------|-----|---------|--------|---------|
| Loaded | Reloading | File change | File watcher event | Read file, parse TOML |
| Reloading | Loaded | Validation success | Schema validates | Swap active config, notify listeners |
| Reloading | Invalid | Validation failure | Required field missing or type mismatch | Keep old config, log error |
| Invalid | Reloading | `reload()` | Manual call | Retry file read and parse |

### Debouncing

File watcher events are debounced to 500ms to avoid thrashing on rapid edits:

```
Edit #1 ──┬── 100ms ──┬── Edit #2 ──┬── 600ms ──┬── Reload
          └───────────┘              └───────────┘
          (ignored, within 500ms)    (triggers reload)
```

### Invariants

- Invalid state preserves last valid config
- Reloading does not block reads (old config remains accessible)
- Validation errors do not terminate the process

### Example

```rust
use maidos_config::{ConfigLoader, ConfigState};

let mut loader = ConfigLoader::new();
let config = loader.load("app.toml")?;
// State: Loaded

// Edit app.toml in background (add new field)
std::thread::sleep(Duration::from_millis(600));
// State: Loaded → Reloading → Loaded

let updated = loader.get_current()?;
assert_ne!(config, updated);  // Config changed

// Edit app.toml with invalid syntax
std::thread::sleep(Duration::from_millis(600));
// State: Loaded → Reloading → Invalid

let current = loader.get_current()?;
assert_eq!(current, updated);  // Old config preserved
```

---

## SM-005: P2P Peer Discovery (maidos-p2p)

### States

| State | Description | Allowed Transitions |
|-------|-------------|---------------------|
| **Idle** | No discovery active | → Discovering (start discovery) |
| **Discovering** | Broadcasting peer announcements | → PeerFound (peer response), → Idle (stop discovery) |
| **PeerFound** | At least one peer discovered | → Discovering (continue), → Connected (dial peer) |
| **Connected** | Established connection to peer | → Disconnected (peer disconnect), → PeerFound (connection lost, other peers available) |
| **Disconnected** | No active connections | → Discovering (restart discovery) |

### State Diagram

```
┌──────┐   start_discovery()   ┌──────────────┐
│ Idle │──────────────────────>│ Discovering  │<───┐
└──────┘                       └──────┬───────┘    │
                                      │            │
                                      │ peer found │
                                      v            │
                               ┌───────────┐       │
                               │ PeerFound │───────┘
                               └─────┬─────┘
                                     │ dial()
                                     v
                               ┌───────────┐   peer disconnect   ┌──────────────┐
                               │ Connected │───────────────────>│ Disconnected │
                               └───────────┘                    └──────────────┘
```

### Transitions

| From | To | Trigger | Guards | Actions |
|------|-----|---------|--------|---------|
| Idle | Discovering | `start_discovery()` | None | Broadcast mDNS/DHT announcement |
| Discovering | PeerFound | Peer response | Valid peer ID | Add peer to discovered list |
| PeerFound | Connected | `dial(peer_id)` | Peer reachable | Establish encrypted connection |
| Connected | Disconnected | Peer disconnect | Socket closed | Remove from active peers |
| Disconnected | Discovering | Auto-retry | No active peers | Restart discovery |

### Invariants

- PeerFound maintains list of discovered peer IDs
- Connected state may have multiple simultaneous connections
- Discovery continues in background while Connected

### Example

```rust
use maidos_p2p::{P2PNode, PeerState};

let mut node = P2PNode::new()?;
node.start_discovery()?;
// State: Idle → Discovering

// Wait for peers
let peers = node.wait_for_peers(Duration::from_secs(10))?;
// State: Discovering → PeerFound

if let Some(peer_id) = peers.first() {
    node.dial(peer_id)?;
    // State: PeerFound → Connected

    node.send_message(peer_id, b"Hello")?;
}
```
