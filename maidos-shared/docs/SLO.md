# maidos-shared — Service Level Objectives

## 1. Purpose

This document defines performance and reliability targets for maidos-shared sub-crates.
These SLOs guide development priorities and alert thresholds.

---

## 2. API Response Time Targets

| Sub-Crate | Operation | P50 | P95 | P99 |
|-----------|-----------|-----|-----|-----|
| maidos-auth | Token validation (cached) | 1 ms | 3 ms | 5 ms |
| maidos-auth | Token refresh (network) | 80 ms | 150 ms | 300 ms |
| maidos-auth | Full OAuth2 flow | 500 ms | 1.5 s | 3 s |
| maidos-bus | Publish (in-process) | 50 us | 200 us | 1 ms |
| maidos-bus | Publish (networked) | 2 ms | 10 ms | 50 ms |
| maidos-config | Config load (file) | 5 ms | 20 ms | 50 ms |
| maidos-config | Hot reload detection | 100 ms | 500 ms | 1 s |
| maidos-llm | Generate (short prompt) | 200 ms | 2 s | 5 s |
| maidos-llm | Generate (long prompt) | 1 s | 10 s | 30 s |
| maidos-log | Log write (single event) | 1 us | 5 us | 10 us |
| maidos-p2p | Peer discovery (LAN) | 500 ms | 2 s | 5 s |
| maidos-p2p | Message send (connected) | 5 ms | 20 ms | 100 ms |
| maidos-google | API call (simple) | 100 ms | 300 ms | 1 s |
| maidos-social | Post publish | 200 ms | 1 s | 3 s |

## 3. Reliability Targets

| Metric | Target | Measurement Window |
|--------|--------|--------------------|
| Build success rate | >= 99% | Rolling 30 days |
| Test pass rate | >= 99.5% | Rolling 30 days |
| Auth token refresh success | >= 99.9% | Rolling 7 days |
| Bus message delivery rate | >= 99.99% (in-process) | Per session |
| P2P reconnection success | >= 95% | Per disconnection event |

## 4. Availability Targets

| Sub-Crate | Dependency | Availability Target |
|-----------|-----------|---------------------|
| maidos-config | Local filesystem | 99.99% (OS uptime) |
| maidos-log | Local filesystem/stdout | 99.99% (OS uptime) |
| maidos-auth | External OAuth2 provider | 99.5% (provider SLA) |
| maidos-llm | Local Ollama instance | 99% (user-managed) |
| maidos-google | Google Cloud APIs | 99.5% (Google SLA) |
| maidos-p2p | Network connectivity | 95% (variable) |

## 5. Resource Consumption Limits

| Metric | Limit |
|--------|-------|
| Memory per sub-crate initialization | < 10 MB |
| Idle CPU usage (all modules loaded) | < 1% of single core |
| Log file rotation threshold | 100 MB per file, 5 files retained |
| Auth token cache size | < 1 MB |

## 6. SLO Review Cadence

SLOs are reviewed quarterly. Breaches exceeding 3 consecutive days trigger a postmortem
and action plan. Metrics are collected via `tracing` spans and exported to the observability
stack when deployed in production environments.
