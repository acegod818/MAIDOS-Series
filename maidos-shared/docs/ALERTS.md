# maidos-shared — Alert Definitions

## 1. Purpose

This document defines alert conditions for maidos-shared modules. Alerts are emitted as
structured log events and optionally forwarded via the message bus.

---

## 2. Alert Severity Levels

| Level | Description | Response Time |
|-------|------------|---------------|
| Critical | Service-breaking failure; immediate action required | < 15 minutes |
| Warning | Degraded performance or approaching limits | < 4 hours |
| Info | Notable event for awareness; no action needed | Next business day |

---

## 3. maidos-auth Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| AUTH-001 | Token refresh failed 3 consecutive times | Critical | Check OAuth2 provider status; verify credentials |
| AUTH-002 | Token refresh latency > 5 seconds | Warning | Investigate network latency to auth provider |
| AUTH-003 | Cached token file corrupted or unreadable | Warning | Delete token cache and re-authenticate |
| AUTH-004 | Unknown auth provider in config | Critical | Fix `AuthConfig.provider` value |

## 4. maidos-bus Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| BUS-001 | Message queue depth exceeds 10,000 | Warning | Check for slow subscribers; consider backpressure |
| BUS-002 | Publish failure rate > 1% over 5 minutes | Critical | Inspect bus state; check for resource exhaustion |
| BUS-003 | Drain timeout on shutdown (> 30 seconds) | Warning | Force-close or investigate stuck subscribers |

## 5. maidos-llm Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| LLM-001 | Ollama connection refused | Critical | Verify Ollama is running; check endpoint config |
| LLM-002 | Generate request timeout (> 60 seconds) | Warning | Check model size vs. hardware; reduce prompt length |
| LLM-003 | Model not found on Ollama instance | Warning | Run `ollama pull <model>` to download required model |
| LLM-004 | Ollama response parse error | Warning | Check Ollama version compatibility |

## 6. maidos-p2p Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| P2P-001 | Zero peers discovered after 60 seconds | Warning | Check network; verify mDNS and firewall rules |
| P2P-002 | Peer disconnected unexpectedly | Info | Automatic reconnection will attempt recovery |
| P2P-003 | All reconnection attempts exhausted | Critical | Check network connectivity; restart peer node |
| P2P-004 | TLS handshake failure with peer | Warning | Verify TLS certificates; check clock synchronization |

## 7. maidos-config Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| CFG-001 | Config file not found at expected path | Critical | Verify file exists; check path in deployment config |
| CFG-002 | Config parse error (invalid TOML) | Critical | Fix syntax in config file |
| CFG-003 | Hot reload watcher stopped unexpectedly | Warning | Restart application or reinitialize config watcher |

## 8. maidos-google / maidos-social Alerts

| ID | Condition | Severity | Action |
|----|-----------|----------|--------|
| GOO-001 | Google API quota exceeded (429) | Warning | Back off; request quota increase if persistent |
| GOO-002 | Service account key expired | Critical | Rotate service account key |
| SOC-001 | Social API rate limit hit | Info | Automatic backoff in progress; no action needed |
| SOC-002 | Social API authentication failure | Critical | Verify API keys in config |

## 9. Alert Delivery

Alerts are emitted via:
1. `tracing::error!` / `tracing::warn!` with structured fields.
2. Message bus topic `maidos.alerts.<module>` when bus is active.
3. Consumer applications may forward to external monitoring (Prometheus, PagerDuty).
