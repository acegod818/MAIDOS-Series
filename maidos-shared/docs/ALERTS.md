# Alert Definitions - maidos-shared

## Overview

This document defines alert conditions for monitoring maidos-shared library health in production applications. Alerts are categorized by severity: **Critical** (immediate action), **Warning** (investigation needed), **Info** (awareness only).

---

## Auth Alerts (maidos-auth)

### ALERT-AUTH-001: High Token Verification Latency

**Severity**: Warning
**Condition**: P99 token verification latency > 1.5ms for 5 minutes
**Impact**: User authentication delays, potential timeout errors
**Possible Causes**:
- CPU contention on host
- Large token store (>100k active tokens)
- Memory pressure causing swapping

**Remediation**:
1. Check system load: `top`, `htop`
2. Review token store size: `metrics.gauge("auth.token_store.size")`
3. Consider token TTL reduction to limit active tokens
4. Scale horizontally if load is sustained

**Escalation**: Platform Team if latency > 2.0ms for 10 minutes

---

### ALERT-AUTH-002: Token Verification Error Rate

**Severity**: Critical
**Condition**: Token verification error rate > 0.1% for 2 minutes
**Impact**: Users unable to authenticate, service outages
**Possible Causes**:
- Corrupted secret key in environment
- Clock skew between issuer and verifier (>5 minutes)
- Memory corruption in token store

**Remediation**:
1. Check secret key consistency: `echo $MAIDOS_AUTH_SECRET | md5sum`
2. Verify system clock sync: `ntpdate -q pool.ntp.org`
3. Restart affected services with correct secret
4. Review logs for segfault or panic messages

**Escalation**: Immediate page to on-call engineer

---

### ALERT-AUTH-003: Token Store Capacity

**Severity**: Warning
**Condition**: Active token count > 80,000 (80% of 100k limit)
**Impact**: Potential token issuance failures, memory pressure
**Possible Causes**:
- Token TTL too long (>24 hours)
- High user concurrency
- Token cleanup not running

**Remediation**:
1. Review token TTL configuration: `config.get("auth.token_ttl")`
2. Force token store cleanup: `issuer.cleanup_expired()`
3. Consider reducing default TTL to 1-6 hours
4. Monitor revocation list size

**Escalation**: N/A (informational)

---

## Bus Alerts (maidos-bus)

### ALERT-BUS-001: High Message Latency

**Severity**: Warning
**Condition**: P95 end-to-end message latency > 20ms for 5 minutes
**Impact**: Delayed event processing, stale data in subscribers
**Possible Causes**:
- Network congestion between publisher and subscriber
- Subscriber processing backlog
- High message volume

**Remediation**:
1. Check network latency: `ping <publisher_host>`
2. Review subscriber message processing time
3. Check ZeroMQ queue depth: `metrics.gauge("bus.queue_depth")`
4. Consider adding more subscriber instances

**Escalation**: Platform Team if latency > 50ms for 10 minutes

---

### ALERT-BUS-002: Message Loss Detected

**Severity**: Critical
**Condition**: Message loss rate > 0.1% over 5 minutes
**Impact**: Missing events, data inconsistency
**Possible Causes**:
- Publisher restart without subscriber reconnect
- Network partition
- ZeroMQ buffer overflow

**Remediation**:
1. Check publisher/subscriber connectivity: `netstat -an | grep 5555`
2. Review logs for disconnect events
3. Verify subscribers reconnected: `metrics.gauge("bus.connected")`
4. Restart subscribers if necessary

**Escalation**: Immediate page to on-call engineer

---

### ALERT-BUS-003: Low Connection Availability

**Severity**: Critical
**Condition**: Connection success rate < 95% for 5 minutes
**Impact**: Event bus unavailable, service disruption
**Possible Causes**:
- Publisher service down
- Port conflict (another process using 5555)
- Network firewall blocking connections

**Remediation**:
1. Check publisher status: `systemctl status maidos-publisher`
2. Test port availability: `telnet localhost 5555`
3. Check firewall: `sudo iptables -L -n | grep 5555`
4. Restart publisher if down

**Escalation**: Immediate page to on-call engineer

---

### ALERT-BUS-004: Slow Reconnection

**Severity**: Warning
**Condition**: Average reconnection time > 60 seconds over 10 minutes
**Impact**: Extended event processing delays
**Possible Causes**:
- Publisher restarting frequently
- Network instability
- High connection retry backoff

**Remediation**:
1. Check publisher uptime: `uptime`
2. Review network stability: `mtr <publisher_host>`
3. Adjust retry backoff configuration if needed
4. Consider adding connection health checks

**Escalation**: Platform Team if reconnection time > 120 seconds

---

## LLM Alerts (maidos-llm)

### ALERT-LLM-001: High Request Overhead

**Severity**: Warning
**Condition**: P95 maidos-llm overhead > 150ms for 5 minutes
**Impact**: Slow LLM responses, poor user experience
**Possible Causes**:
- Router evaluating too many providers
- Budget tracking overhead
- Request serialization bottleneck

**Remediation**:
1. Review router strategy: prefer Priority over Weighted
2. Check budget tracking frequency: `metrics.gauge("llm.budget_checks")`
3. Profile request path with `cargo flamegraph`
4. Consider disabling budget checks for low-cost requests

**Escalation**: Platform Team if overhead > 300ms

---

### ALERT-LLM-002: High Provider Error Rate

**Severity**: Critical
**Condition**: Provider error rate > 5% for 5 minutes
**Impact**: LLM requests failing, feature degradation
**Possible Causes**:
- Provider API outage (OpenAI, Anthropic)
- Invalid API key (expired or revoked)
- Rate limit exceeded

**Remediation**:
1. Check provider status page:
   - OpenAI: https://status.openai.com
   - Anthropic: https://status.anthropic.com
2. Verify API key validity: `curl -H "Authorization: Bearer $API_KEY" https://api.openai.com/v1/models`
3. Review rate limit headers in logs
4. Switch to fallback provider if available

**Escalation**: Immediate page to on-call engineer if error rate > 20%

---

### ALERT-LLM-003: Budget Limit Exceeded

**Severity**: Warning
**Condition**: Daily LLM spending > 90% of budget
**Impact**: LLM requests blocked, feature unavailable
**Possible Causes**:
- Unexpected usage spike
- Budget misconfiguration
- Expensive model (GPT-4) used by default

**Remediation**:
1. Review usage dashboard: `metrics.counter("llm.requests_total")`
2. Check model distribution: prefer GPT-4o-mini over GPT-4o
3. Increase daily budget if justified
4. Implement request throttling for non-critical features

**Escalation**: N/A (financial decision by product team)

---

### ALERT-LLM-004: Streaming Throughput Degradation

**Severity**: Warning
**Condition**: P50 streaming throughput < 800 tokens/sec for 10 minutes
**Impact**: Slow streaming responses, poor UX
**Possible Causes**:
- Provider throttling
- Network bandwidth saturation
- SSE parsing overhead

**Remediation**:
1. Check provider response time in logs
2. Test network bandwidth: `iperf3`
3. Profile SSE parsing with `cargo bench`
4. Consider switching to non-streaming for small responses

**Escalation**: Platform Team if throughput < 500 tokens/sec

---

## Config Alerts (maidos-config)

### ALERT-CFG-001: Config Parse Errors

**Severity**: Critical
**Condition**: Config parse error rate > 1% for 5 minutes
**Impact**: Application using stale config, potential misbehavior
**Possible Causes**:
- Broken TOML syntax in config file
- Invalid schema changes
- File corruption

**Remediation**:
1. Validate config file: `maidos-config-validate config.toml`
2. Review recent config changes: `git diff config.toml`
3. Restore last known good config from backup
4. Check file permissions: `ls -la config.toml`

**Escalation**: Immediate page to on-call engineer

---

### ALERT-CFG-002: Hot Reload Not Triggering

**Severity**: Warning
**Condition**: No hot reload events for 24 hours (with known file changes)
**Impact**: Config changes require manual restart
**Possible Causes**:
- File watcher stopped (inotify limit exceeded)
- File system events dropped
- File edited with `mv` instead of in-place save

**Remediation**:
1. Check inotify limits: `cat /proc/sys/fs/inotify/max_user_watches`
2. Increase limits: `echo 524288 | sudo tee /proc/sys/fs/inotify/max_user_watches`
3. Test file watcher: `inotifywait -m config.toml`
4. Restart application to reinitialize watcher

**Escalation**: Platform Team if issue persists after restart

---

### ALERT-CFG-003: Slow Config Load

**Severity**: Warning
**Condition**: P99 config load latency > 100ms for 5 minutes
**Impact**: Slow application startup, delayed hot reloads
**Possible Causes**:
- Large config file (>1 MB)
- Slow disk I/O
- Excessive environment variable expansion

**Remediation**:
1. Check config file size: `ls -lh config.toml`
2. Test disk I/O: `dd if=config.toml of=/dev/null bs=1M`
3. Reduce environment variable expansions
4. Consider splitting into multiple config files

**Escalation**: N/A (performance optimization)

---

## FFI Alerts (All Crates)

### ALERT-FFI-001: Segfault Detected

**Severity**: Critical
**Condition**: Any segfault in FFI boundary (crash rate > 0%)
**Impact**: Application crash, data loss
**Possible Causes**:
- Null pointer dereference
- Use-after-free (forgot to free or double-free)
- Buffer overflow in string marshaling

**Remediation**:
1. Review crash logs for stack trace
2. Check FFI call site for null pointer checks
3. Verify `_free` functions called exactly once per allocation
4. Run with AddressSanitizer: `cargo build --target x86_64-unknown-linux-gnu -Zbuild-std`
5. File GitHub issue with reproduction case

**Escalation**: Immediate escalation to MAIDOS Core Team

---

### ALERT-FFI-002: Memory Leak Detected

**Severity**: Warning
**Condition**: RSS growth > 10 MB/hour for 6 hours
**Impact**: OOM kill, application instability
**Possible Causes**:
- Missing `_free` calls in C#/C++ consumer
- Circular references in provider/subscriber
- String leaks in error paths

**Remediation**:
1. Profile with Valgrind: `valgrind --leak-check=full ./app`
2. Review FFI wrapper code for missing `_free` calls
3. Ensure IDisposable implemented in C# wrappers
4. Check finalizers are registered: `GC.SuppressFinalize(this)`

**Escalation**: Platform Team if leak rate > 50 MB/hour

---

### ALERT-FFI-003: High FFI Call Latency

**Severity**: Warning
**Condition**: P95 FFI call overhead > 150μs for 5 minutes
**Impact**: Performance degradation in tight loops
**Possible Causes**:
- Excessive string marshaling (UTF-8 conversion)
- Large struct copies
- Contention in FFI thread pool

**Remediation**:
1. Profile FFI calls: `cargo bench --bench ffi_bench`
2. Reduce string allocations (pass pointers where possible)
3. Use buffer reuse for repeated calls
4. Consider batching multiple FFI calls

**Escalation**: N/A (performance optimization)

---

## System Resource Alerts

### ALERT-SYS-001: High Memory Usage

**Severity**: Warning
**Condition**: RSS > 75 MB for 10 minutes
**Impact**: Memory pressure, potential OOM kill
**Possible Causes**:
- Token store too large
- LLM response caching not evicting
- Memory leak in consumer application

**Remediation**:
1. Review heap allocations: `jemalloc_ctl::stats::allocated()`
2. Check token store size: `metrics.gauge("auth.token_store.size")`
3. Clear LLM response cache: `provider.clear_cache()`
4. Restart application if memory usage unsustainable

**Escalation**: Platform Team if RSS > 100 MB

---

### ALERT-SYS-002: Thread Pool Exhaustion

**Severity**: Critical
**Condition**: Tokio thread pool utilization > 95% for 5 minutes
**Impact**: Request timeouts, application unresponsive
**Possible Causes**:
- Blocking I/O in async context
- Infinite loop in async task
- Thread pool too small for workload

**Remediation**:
1. Review blocking operations in async code
2. Move blocking I/O to `tokio::task::spawn_blocking`
3. Check for deadlocks: `ps -eLf | grep <pid>`
4. Increase thread pool size: `TOKIO_WORKER_THREADS=16`

**Escalation**: Immediate page to on-call engineer

---

## Alert Summary Matrix

| Alert ID | Severity | Condition | Auto-Remediation | Page On-Call |
|----------|----------|-----------|------------------|--------------|
| AUTH-001 | Warning | P99 verify > 1.5ms | No | No |
| AUTH-002 | Critical | Error rate > 0.1% | No | Yes |
| AUTH-003 | Warning | Token count > 80k | Auto-cleanup | No |
| BUS-001 | Warning | P95 latency > 20ms | No | No |
| BUS-002 | Critical | Loss rate > 0.1% | No | Yes |
| BUS-003 | Critical | Connect rate < 95% | Restart publisher | Yes |
| BUS-004 | Warning | Reconnect > 60s | No | No |
| LLM-001 | Warning | Overhead > 150ms | No | No |
| LLM-002 | Critical | Error rate > 5% | Switch provider | Yes (>20%) |
| LLM-003 | Warning | Budget > 90% | Block requests | No |
| LLM-004 | Warning | Throughput < 800 tok/s | No | No |
| CFG-001 | Critical | Parse error > 1% | Rollback config | Yes |
| CFG-002 | Warning | No reload 24h | Restart watcher | No |
| CFG-003 | Warning | P99 load > 100ms | No | No |
| FFI-001 | Critical | Segfault | Restart service | Yes |
| FFI-002 | Warning | Leak > 10 MB/h | No | No |
| FFI-003 | Warning | FFI > 150μs | No | No |
| SYS-001 | Warning | RSS > 75 MB | Clear caches | No |
| SYS-002 | Critical | Threads > 95% | Restart service | Yes |

---

## Alert Configuration Examples

### Prometheus AlertManager

```yaml
groups:
  - name: maidos-shared
    interval: 30s
    rules:
      - alert: HighTokenVerificationLatency
        expr: histogram_quantile(0.99, auth_verify_token_latency_bucket) > 0.0015
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Token verification P99 latency > 1.5ms"

      - alert: TokenVerificationErrors
        expr: rate(auth_verify_token_error_total[2m]) > 0.001
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Token verification error rate > 0.1%"

      - alert: MessageLossDetected
        expr: rate(bus_message_loss_total[5m]) > 0.001
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Event bus message loss rate > 0.1%"
```

### Grafana Dashboard Alert

```json
{
  "alert": {
    "name": "High LLM Provider Error Rate",
    "conditions": [
      {
        "evaluator": { "type": "gt", "params": [0.05] },
        "query": { "params": ["A", "5m", "now"] },
        "reducer": { "type": "avg" }
      }
    ],
    "executionErrorState": "alerting",
    "frequency": "1m",
    "handler": 1,
    "message": "LLM provider error rate exceeded 5%",
    "name": "LLM Error Rate",
    "noDataState": "no_data",
    "notifications": [{ "uid": "oncall-slack" }]
  }
}
```

---

## Notification Channels

- **Critical Alerts**: PagerDuty → On-Call Engineer + Slack #maidos-oncall
- **Warning Alerts**: Slack #maidos-alerts + Email to platform-team@maidos.dev
- **Info Alerts**: Grafana dashboard only (no notification)

---

## On-Call Playbooks

Each alert links to a detailed playbook in RUNBOOK.md. On-call engineers should:

1. Acknowledge alert in PagerDuty within 5 minutes
2. Follow diagnostic steps in RUNBOOK.md
3. Escalate to MAIDOS Core Team if root cause unclear
4. Document resolution in incident post-mortem
5. Update alert thresholds if false positive rate > 10%
