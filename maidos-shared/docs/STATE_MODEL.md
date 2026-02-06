# maidos-shared — State Models

## 1. Overview

Stateful modules in maidos-shared follow explicit state machines. Each state transition is
logged via maidos-log and emits a bus event when the bus feature is enabled.

---

## 2. maidos-auth — Authentication State

```
         ┌──────────────┐
         │ Disconnected  │ (initial)
         └──────┬───────┘
                │ authenticate()
                v
         ┌──────────────┐
         │ Authenticating│
         └──────┬───────┘
           ok / │ \ error
              v   v
  ┌────────────┐  ┌──────────┐
  │ Authenticated│  │  Failed  │
  └──────┬─────┘  └────┬─────┘
         │ token_expired  │ retry
         v               v
  ┌──────────────┐     (back to Authenticating)
  │  Refreshing  │
  └──────┬───────┘
    ok / │ \ error
       v   v
  Authenticated / Failed
```

| State | Description |
|-------|------------|
| Disconnected | No credentials loaded; initial state |
| Authenticating | OAuth2 flow or token exchange in progress |
| Authenticated | Valid token held; API calls permitted |
| Refreshing | Token expired; refresh in progress |
| Failed | Auth attempt failed; retryable |

---

## 3. maidos-bus — Message Bus State

```
  ┌──────────┐     start()     ┌──────────┐
  │  Stopped  │ ─────────────> │  Running  │
  └──────────┘                 └────┬─────┘
                                    │ close()
                                    v
                               ┌──────────┐
                               │ Draining  │
                               └────┬─────┘
                                    │ all delivered
                                    v
                               ┌──────────┐
                               │  Closed   │
                               └──────────┘
```

| State | Description |
|-------|------------|
| Stopped | Bus created but not yet accepting messages |
| Running | Actively routing messages between subscribers |
| Draining | Shutdown requested; delivering pending messages |
| Closed | All resources released; no further operations |

---

## 4. maidos-p2p — Peer Connection State

```
  ┌──────────────┐
  │  Discovering  │ (mDNS / bootstrap)
  └──────┬───────┘
         │ peer found
         v
  ┌──────────────┐
  │  Connecting   │ (QUIC handshake)
  └──────┬───────┘
    ok / │ \ error
       v   v
  ┌──────────┐  ┌──────────────┐
  │ Connected │  │ Unreachable  │
  └────┬─────┘  └──────┬───────┘
       │ lost           │ retry after backoff
       v               v
  ┌──────────────┐   (back to Connecting)
  │ Reconnecting │
  └──────┬───────┘
    ok / │ \ max retries
       v   v
  Connected / Disconnected
```

| State | Description |
|-------|------------|
| Discovering | Searching for peers via mDNS or bootstrap nodes |
| Connecting | QUIC handshake and TLS negotiation in progress |
| Connected | Active bidirectional communication |
| Unreachable | Initial connection failed; will retry |
| Reconnecting | Previously connected peer lost; attempting recovery |
| Disconnected | All retries exhausted; peer removed from roster |

---

## 5. State Observability

Each module exposes a `state()` method returning the current enum variant. State
transitions emit `tracing` span events and, when the bus is available, publish to
the `maidos.state.<module>` topic.
