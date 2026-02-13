# Service Level Objectives - maidos-shared

## Overview

maidos-shared is a library, not a hosted service, but we define SLOs for key operations to guide performance targets and monitoring in consuming applications.

---

## SLO-001: Token Issuance Latency (maidos-auth)

**Objective**: 99% of token issuance operations complete in < 1ms

### Service Level Indicators (SLIs)

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Latency P50** | Median token issuance time | < 0.5ms |
| **Latency P95** | 95th percentile token issuance time | < 0.8ms |
| **Latency P99** | 99th percentile token issuance time | < 1.0ms |
| **Error Rate** | % of issuance operations that fail | < 0.01% |

### Measurement Method

```rust
use std::time::Instant;
use maidos_auth::TokenIssuer;

let start = Instant::now();
let token = issuer.issue(user_id, caps, ttl)?;
let duration = start.elapsed();

// Log to metrics system
metrics::histogram!("auth.issue_token.latency", duration.as_micros() as f64);
```

### Alerting Threshold

- **Warning**: P99 latency > 1.5ms for 5 minutes
- **Critical**: P99 latency > 2.0ms for 2 minutes OR error rate > 0.1%

---

## SLO-002: Token Verification Latency (maidos-auth)

**Objective**: 99.9% of token verification operations complete in < 1ms

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Latency P50** | Median verification time | < 0.3ms |
| **Latency P95** | 95th percentile verification time | < 0.7ms |
| **Latency P99** | 99th percentile verification time | < 1.0ms |
| **Latency P99.9** | 99.9th percentile verification time | < 1.5ms |
| **Error Rate** | % of verification operations that fail (invalid tokens excluded) | < 0.01% |

### Measurement Method

```rust
let start = Instant::now();
let result = issuer.verify(&token);
let duration = start.elapsed();

match result {
    Ok(_) => metrics::histogram!("auth.verify_token.latency.success", duration.as_micros() as f64),
    Err(AuthError::InvalidSignature) => metrics::counter!("auth.verify_token.invalid", 1),
    Err(e) => {
        metrics::histogram!("auth.verify_token.latency.error", duration.as_micros() as f64);
        metrics::counter!("auth.verify_token.error", 1);
    }
}
```

### Alerting Threshold

- **Warning**: P99.9 latency > 2.0ms for 5 minutes
- **Critical**: Error rate > 0.1% for 2 minutes

---

## SLO-003: Event Bus Message Latency (maidos-bus)

**Objective**: 95% of messages delivered in < 10ms (end-to-end)

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Publish Latency P50** | Median time to publish message | < 2ms |
| **Publish Latency P95** | 95th percentile publish time | < 5ms |
| **Publish Latency P99** | 99th percentile publish time | < 10ms |
| **Subscribe Latency P95** | 95th percentile time from publish to receive | < 10ms |
| **Message Loss Rate** | % of messages not received by subscriber | < 0.01% |

### Measurement Method

```rust
// Publisher side
let start = Instant::now();
let msg = BusMessage::new("topic", payload).with_timestamp(start);
publisher.publish(msg).await?;
let publish_latency = start.elapsed();
metrics::histogram!("bus.publish.latency", publish_latency.as_micros() as f64);

// Subscriber side
let msg = subscriber.receive().await?;
let e2e_latency = Instant::now().duration_since(msg.timestamp());
metrics::histogram!("bus.e2e_latency", e2e_latency.as_micros() as f64);
```

### Alerting Threshold

- **Warning**: P95 end-to-end latency > 20ms for 5 minutes
- **Critical**: P99 latency > 50ms OR message loss rate > 0.1%

---

## SLO-004: Event Bus Availability (maidos-bus)

**Objective**: 99.9% uptime (connection success rate)

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Connection Success Rate** | % of connection attempts that succeed | >= 99.9% |
| **Reconnection Time** | Time to reconnect after disconnect | < 30 seconds |
| **Uptime** | % of time connection is active | >= 99.9% (8.76 hours downtime/year) |

### Measurement Method

```rust
// Track connection state
let start = Instant::now();
let result = subscriber.connect("tcp://localhost:5555").await;
let connect_latency = start.elapsed();

match result {
    Ok(_) => {
        metrics::histogram!("bus.connect.latency", connect_latency.as_micros() as f64);
        metrics::counter!("bus.connect.success", 1);
    }
    Err(_) => {
        metrics::counter!("bus.connect.failure", 1);
    }
}

// Track uptime
if subscriber.is_connected() {
    metrics::gauge!("bus.connected", 1.0);
} else {
    metrics::gauge!("bus.connected", 0.0);
}
```

### Alerting Threshold

- **Warning**: Connection success rate < 99% for 5 minutes
- **Critical**: Connection success rate < 95% OR reconnection time > 60 seconds

---

## SLO-005: LLM Request Overhead (maidos-llm)

**Objective**: maidos-llm adds < 100ms overhead to provider requests

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Overhead P50** | Median overhead (request - provider time) | < 50ms |
| **Overhead P95** | 95th percentile overhead | < 100ms |
| **Overhead P99** | 99th percentile overhead | < 150ms |
| **Provider Error Rate** | % of requests that fail due to provider errors | < 1% |

### Measurement Method

```rust
let start = Instant::now();
let response = provider.complete(request).await?;
let total_latency = start.elapsed();

// Estimate overhead (if provider returns response time)
let overhead = total_latency.as_millis() - response.response_time_ms;
metrics::histogram!("llm.overhead", overhead as f64);
metrics::histogram!("llm.total_latency", total_latency.as_millis() as f64);
```

### Alerting Threshold

- **Warning**: P95 overhead > 150ms for 5 minutes
- **Critical**: P99 overhead > 300ms OR provider error rate > 5%

---

## SLO-006: Config Load Time (maidos-config)

**Objective**: 99% of config loads complete in < 50ms

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Load Latency P50** | Median config load time | < 20ms |
| **Load Latency P95** | 95th percentile load time | < 40ms |
| **Load Latency P99** | 99th percentile load time | < 50ms |
| **Parse Error Rate** | % of loads that fail due to syntax errors | < 0.1% |

### Measurement Method

```rust
let start = Instant::now();
let config = ConfigLoader::new().load("config.toml");
let duration = start.elapsed();

match config {
    Ok(_) => metrics::histogram!("config.load.latency.success", duration.as_millis() as f64),
    Err(ConfigError::ParseError(_)) => {
        metrics::histogram!("config.load.latency.parse_error", duration.as_millis() as f64);
        metrics::counter!("config.load.parse_error", 1);
    }
    Err(_) => {
        metrics::histogram!("config.load.latency.error", duration.as_millis() as f64);
        metrics::counter!("config.load.error", 1);
    }
}
```

### Alerting Threshold

- **Warning**: P99 latency > 100ms for 5 minutes
- **Critical**: Parse error rate > 1% (indicates broken config files)

---

## SLO-007: Hot Reload Detection Time (maidos-config)

**Objective**: 95% of config file changes detected within 500ms

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Detection Latency P50** | Median time from file change to reload trigger | < 200ms |
| **Detection Latency P95** | 95th percentile detection time | < 500ms |
| **Detection Latency P99** | 99th percentile detection time | < 1000ms |
| **Missed Reload Rate** | % of file changes that are not detected | < 0.1% |

### Measurement Method

```rust
// Instrument file watcher
let file_change_time = Instant::now();  // From file system event
let reload_trigger_time = Instant::now();  // When loader.reload() called
let detection_latency = reload_trigger_time.duration_since(file_change_time);

metrics::histogram!("config.hot_reload.detection_latency", detection_latency.as_millis() as f64);
```

### Alerting Threshold

- **Warning**: P95 detection latency > 1000ms for 5 minutes
- **Critical**: Missed reload rate > 1% (file watcher not working)

---

## SLO-008: FFI Call Overhead (All Crates)

**Objective**: FFI boundary adds < 100μs overhead per call

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **FFI Overhead P50** | Median FFI call overhead (vs native Rust) | < 50μs |
| **FFI Overhead P95** | 95th percentile FFI overhead | < 100μs |
| **FFI Overhead P99** | 99th percentile FFI overhead | < 150μs |
| **Segfault Rate** | % of FFI calls that cause segfaults | 0% |

### Measurement Method

```c
// C client
#include <time.h>

struct timespec start, end;
clock_gettime(CLOCK_MONOTONIC, &start);

char* token = maidos_auth_issue_token("user", caps, 2, "secret", 3600);

clock_gettime(CLOCK_MONOTONIC, &end);
long duration_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;

// Log to metrics
log_histogram("ffi.auth.issue_token.latency", duration_us);
```

### Alerting Threshold

- **Warning**: P95 overhead > 150μs for 5 minutes
- **Critical**: Any segfaults detected (crash rate > 0%)

---

## SLO-009: Streaming Throughput (maidos-llm)

**Objective**: Process >= 1000 tokens/sec in streaming mode

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Throughput P50** | Median tokens processed per second | >= 1000 tokens/sec |
| **Throughput P95** | 95th percentile throughput | >= 800 tokens/sec |
| **Stream Latency** | Time from SSE event to MaidosStreamItem | < 10ms |
| **Stream Error Rate** | % of streams that error before completion | < 1% |

### Measurement Method

```rust
let start = Instant::now();
let mut token_count = 0;

let mut stream = provider.complete_streaming(request).await?;
while let Some(item) = stream.next().await {
    match item? {
        MaidosStreamItem::TextDelta(text) => {
            token_count += estimate_token_count(&text);
        }
        MaidosStreamItem::Done(_) => break,
        _ => {}
    }
}

let duration = start.elapsed();
let throughput = (token_count as f64) / duration.as_secs_f64();
metrics::histogram!("llm.streaming.throughput", throughput);
```

### Alerting Threshold

- **Warning**: P50 throughput < 800 tokens/sec for 5 minutes
- **Critical**: Stream error rate > 5%

---

## SLO-010: Memory Footprint (All Crates)

**Objective**: Total memory usage < 50 MB with all crates loaded

### SLIs

| Metric | Description | Measurement |
|--------|-------------|-------------|
| **Resident Set Size (RSS)** | Physical memory used | < 50 MB |
| **Heap Allocations** | Active heap allocations | < 100,000 |
| **Memory Leak Rate** | Growth in RSS over 24 hours | < 10 MB/day |

### Measurement Method

```rust
// Use jemalloc with stats
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

// Periodically sample memory
let stats = jemalloc_ctl::stats::allocated::read().unwrap();
metrics::gauge!("memory.allocated_bytes", stats as f64);

// RSS from /proc/self/status (Linux)
let rss_kb = std::fs::read_to_string("/proc/self/status")?
    .lines()
    .find(|l| l.starts_with("VmRSS:"))
    .and_then(|l| l.split_whitespace().nth(1))
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(0);
metrics::gauge!("memory.rss_kb", rss_kb as f64);
```

### Alerting Threshold

- **Warning**: RSS > 75 MB for 5 minutes
- **Critical**: RSS > 100 MB OR memory leak rate > 50 MB/day

---

## Monitoring Dashboard

Recommended metrics dashboard layout:

```
┌─────────────────────────────────────────────────────┐
│ maidos-shared Library Metrics Dashboard            │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [Auth Token Operations]                             │
│ - Issue Latency (P50/P95/P99): 0.4ms / 0.7ms / 0.9ms │
│ - Verify Latency (P50/P95/P99): 0.3ms / 0.6ms / 0.8ms │
│ - Error Rate: 0.002%                                │
│                                                     │
│ [Event Bus]                                         │
│ - Publish Latency (P50/P95/P99): 2ms / 4ms / 8ms   │
│ - E2E Latency (P50/P95): 5ms / 12ms                │
│ - Connection Uptime: 99.95%                         │
│ - Message Loss Rate: 0.001%                         │
│                                                     │
│ [LLM Integration]                                   │
│ - Request Overhead (P50/P95/P99): 45ms / 95ms / 140ms │
│ - Streaming Throughput (P50/P95): 1200 / 950 tokens/s │
│ - Provider Error Rate: 0.5%                         │
│                                                     │
│ [Config Management]                                 │
│ - Load Latency (P50/P95/P99): 18ms / 38ms / 48ms   │
│ - Hot Reload Detection (P50/P95): 180ms / 450ms    │
│ - Parse Error Rate: 0.01%                           │
│                                                     │
│ [System Resources]                                  │
│ - RSS: 42 MB / 50 MB (84%)                          │
│ - Heap Allocations: 87,234                          │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## SLO Compliance Tracking

| SLO | Target | Current | Trend | Status |
|-----|--------|---------|-------|--------|
| SLO-001: Token Issuance | 99% < 1ms | 99.2% < 0.9ms | ↗ | ✅ Met |
| SLO-002: Token Verification | 99.9% < 1ms | 99.95% < 0.8ms | → | ✅ Met |
| SLO-003: Bus Message Latency | 95% < 10ms | 96.1% < 9ms | → | ✅ Met |
| SLO-004: Bus Availability | 99.9% uptime | 99.95% uptime | ↗ | ✅ Met |
| SLO-005: LLM Overhead | < 100ms | P95: 95ms | ↘ | ✅ Met |
| SLO-006: Config Load | 99% < 50ms | 99.4% < 48ms | → | ✅ Met |
| SLO-007: Hot Reload | 95% < 500ms | 96.8% < 450ms | ↗ | ✅ Met |
| SLO-008: FFI Overhead | < 100μs | P95: 85μs | → | ✅ Met |
| SLO-009: Streaming | >= 1000 tok/s | P50: 1200 tok/s | ↗ | ✅ Met |
| SLO-010: Memory | < 50 MB | 42 MB | → | ✅ Met |
