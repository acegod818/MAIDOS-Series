# Service Level Objectives - MAIDOS-CodeQC

## Overview

This document defines Service Level Objectives (SLOs) for MAIDOS-CodeQC's serve mode API. SLOs establish measurable targets for availability, performance, and reliability.

**Scope**: Serve mode only (`npx maidos-codeqc serve`)
**Version**: 0.3.5
**Review Cycle**: Quarterly

---

## 1. Service Level Indicators (SLIs)

### 1.1 Availability

**Definition**: Percentage of time the API server responds to health checks with HTTP 200.

**Measurement**:
```bash
# Probe endpoint
GET /health

# Success criteria
HTTP 200 + {"status": "ok"}
```

**Target SLO**: 99.5% uptime per month

**Calculation**:
```
Uptime % = (Total minutes - Downtime minutes) / Total minutes * 100
```

**Example**:
- Month: 30 days = 43,200 minutes
- Allowed downtime: 216 minutes (3.6 hours)
- Target: > 43,200 - 216 = 42,984 minutes uptime

---

### 1.2 API Latency (Scan Endpoint)

**Definition**: Time from HTTP request to response for `POST /api/scan`.

**Measurement**:
```bash
# Request
POST /api/scan
{
  "path": "/var/repos/project",
  "level": "D"
}

# Measure response time
time curl -X POST http://localhost:3000/api/scan -d '{"path":"/tmp/test","level":"D"}'
```

**Target SLOs**:

| Percentile | Target Latency | Codebase Size |
|------------|----------------|---------------|
| p50 | < 5 seconds | 5,000 LOC |
| p90 | < 10 seconds | 10,000 LOC |
| p99 | < 20 seconds | 20,000 LOC |

**Breach**: If p90 > 10 seconds for 3 consecutive requests.

---

### 1.3 WebSocket Update Latency

**Definition**: Time from scan completion to dashboard WebSocket event delivery.

**Measurement**:
```javascript
// Client-side timing
const start = Date.now();
ws.on('scan_complete', () => {
  const latency = Date.now() - start;
  console.log('WebSocket latency:', latency);
});
```

**Target SLO**: p95 < 100 ms

**Breach**: If p95 > 500 ms for 10 minutes.

---

### 1.4 Error Rate

**Definition**: Percentage of API requests resulting in HTTP 5xx errors.

**Measurement**:
```bash
# Count 5xx responses in logs
grep "HTTP 5" /var/log/codeqc-api.log | wc -l
```

**Target SLO**: < 1% error rate per hour

**Calculation**:
```
Error Rate % = (5xx responses / Total requests) * 100
```

**Breach**: If error rate > 5% for 15 minutes.

---

### 1.5 Concurrent Request Capacity

**Definition**: Number of simultaneous scan requests the server can handle without degradation.

**Measurement**:
```bash
# Load test with Apache Bench
ab -n 100 -c 10 -p scan.json -T application/json http://localhost:3000/api/scan
```

**Target SLO**: Support 10 concurrent requests with p90 latency < 15 seconds

**Breach**: If concurrency > 10 causes > 50% increase in p90 latency.

---

## 2. Service Level Objectives

### 2.1 SLO Summary Table

| SLI | Metric | Target | Measurement Window |
|-----|--------|--------|-------------------|
| Availability | Uptime | 99.5% | 30 days |
| API Latency (p50) | Response time | < 5s | 1 hour |
| API Latency (p90) | Response time | < 10s | 1 hour |
| API Latency (p99) | Response time | < 20s | 1 hour |
| WebSocket Latency (p95) | Update delay | < 100ms | 1 hour |
| Error Rate | 5xx errors | < 1% | 1 hour |
| Concurrent Capacity | Max requests | 10 | Continuous |

---

### 2.2 Error Budget

**Error Budget**: Amount of allowed downtime/errors before SLO breach.

| SLO | Target | Error Budget (Monthly) |
|-----|--------|------------------------|
| Availability (99.5%) | 99.5% uptime | 3.6 hours downtime |
| Error Rate (< 1%) | < 1% errors | 7,200 failed requests (if 720k total) |

**Error Budget Policy**:
- If error budget exhausted: Freeze new features, focus on reliability
- If error budget > 50% remaining: Safe to deploy new features
- If error budget < 20% remaining: Code freeze, incident review

---

## 3. SLO Monitoring

### 3.1 Health Check Probes

**Liveness Probe** (Kubernetes/Docker):
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 30
```

**Readiness Probe**:
```yaml
readinessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```

### 3.2 Metrics Collection

**Prometheus Metrics** (future enhancement):
```yaml
# Example metrics to expose at /metrics
codeqc_http_requests_total{method="POST", endpoint="/api/scan", status="200"} 1543
codeqc_http_request_duration_seconds{endpoint="/api/scan", quantile="0.9"} 8.2
codeqc_websocket_latency_seconds{quantile="0.95"} 0.085
codeqc_concurrent_scans 7
```

**Log-based Monitoring** (current implementation):
```bash
# Parse logs for SLI data
cat /var/log/codeqc-api.log | grep "POST /api/scan" | awk '{print $NF}' | sort -n | tail -10
# Last 10 response times
```

---

## 4. Alerting Rules

### 4.1 Critical Alerts (Page On-Call)

| Alert | Condition | Action |
|-------|-----------|--------|
| API Down | Health check fails for 5 minutes | Page on-call engineer |
| High Error Rate | 5xx > 5% for 15 minutes | Page on-call engineer |
| High Latency | p90 > 20s for 10 minutes | Page on-call engineer |

### 4.2 Warning Alerts (Slack/Email)

| Alert | Condition | Action |
|-------|-----------|--------|
| Degraded Performance | p90 > 10s for 5 minutes | Notify team via Slack |
| WebSocket Latency | p95 > 200ms for 10 minutes | Notify team via Slack |
| Approaching Error Budget | < 20% budget remaining | Email weekly report |

---

## 5. SLO Violation Response

### 5.1 Incident Response Workflow

1. **Detection**: Monitoring system triggers alert
2. **Acknowledgment**: On-call engineer acks within 5 minutes
3. **Investigation**: Identify root cause (logs, metrics, code review)
4. **Mitigation**: Apply temporary fix (restart service, rollback deploy)
5. **Resolution**: Deploy permanent fix
6. **Postmortem**: Write incident report within 72 hours

### 5.2 SLO Breach Escalation

| Breach Severity | Definition | Response Time | Escalation |
|-----------------|------------|---------------|------------|
| **Critical** | Availability < 95% | 5 minutes | Page VP Engineering |
| **High** | Error rate > 10% | 15 minutes | Notify Engineering Manager |
| **Medium** | p90 latency > 15s | 1 hour | Team lead notification |
| **Low** | p99 latency > 30s | 24 hours | Track in backlog |

---

## 6. Capacity Planning

### 6.1 Traffic Projections

| Month | Expected Requests/Day | Required Capacity (concurrent) |
|-------|----------------------|-------------------------------|
| Current (Feb 2025) | 1,000 | 5 |
| 3 months | 5,000 | 10 |
| 6 months | 20,000 | 25 |
| 12 months | 100,000 | 50 |

### 6.2 Scaling Strategy

**Vertical Scaling** (current):
- Single Node.js process
- Max 10 concurrent scans
- 2 GB RAM, 2 CPU cores

**Horizontal Scaling** (future):
```bash
# PM2 cluster mode (4 processes)
pm2 start dist/cli.js -i 4 --name codeqc-api -- serve --port 3000

# Kubernetes deployment (3 replicas)
kubectl scale deployment codeqc-api --replicas=3
```

**Resource Limits**:
```yaml
# Kubernetes resource requests/limits
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "2Gi"
    cpu: "2000m"
```

---

## 7. SLO Review and Adjustment

### 7.1 Quarterly Review

**Review Questions**:
1. Are current SLOs achievable? (Check error budget consumption)
2. Are SLOs too lenient? (If error budget > 80% remaining, tighten SLOs)
3. Are SLOs too strict? (If error budget exhausted, loosen SLOs)
4. Do SLOs align with user expectations?

**Adjustment Process**:
1. Collect 90 days of historical data
2. Calculate actual p50/p90/p99 latencies
3. Propose new SLO targets
4. Get stakeholder approval
5. Update this document

### 7.2 SLO Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.3.5 | 2025-02-13 | Initial SLO definition |
| (future) | (TBD) | Adjust latency targets based on real data |

---

## 8. Dependency SLOs

### 8.1 Upstream Dependencies

MAIDOS-CodeQC serve mode has no external upstream dependencies (all rules are local, no API calls).

**Internal Dependencies**:
- **Node.js Runtime**: Assume 99.9% stability (crash = restart)
- **File System**: Assume 99.99% availability (local disk)

### 8.2 Downstream Consumers

**Clients**:
- Dashboard web UI (same SLOs as API)
- CI/CD webhooks (not implemented in v0.3.5)

**SLO Inheritance**:
- Dashboard SLO = API SLO (dependent service)
- If API breaches SLO, dashboard automatically breaches

---

## 9. Cost of SLO Violations

### 9.1 Business Impact

| Violation Type | Impact | Cost Estimate |
|----------------|--------|---------------|
| 1 hour downtime | Developers blocked | $500 (10 devs × $50/hr) |
| 10% error rate | CI/CD failures | $1,000 (re-runs, delays) |
| High latency (p90 > 30s) | Slow feedback loop | $200/day (productivity loss) |

### 9.2 Reputation Impact

- **Internal tool**: Low external reputation risk
- **Open source**: GitHub issues, poor reviews if unreliable
- **SaaS (future)**: Customer churn if SLOs breached frequently

---

## 10. Testing SLOs

### 10.1 Load Testing

```bash
# Simulate 10 concurrent users
ab -n 100 -c 10 -p scan.json -T application/json http://localhost:3000/api/scan

# Expected:
# - 100% success rate (no 5xx)
# - p90 < 10 seconds
# - p99 < 20 seconds
```

### 10.2 Chaos Testing

```bash
# Kill API server mid-scan
pm2 stop codeqc-api

# Expected:
# - Health checks fail within 30 seconds
# - Alert triggered within 5 minutes
# - Auto-restart (if configured)

# Test recovery
pm2 start codeqc-api

# Expected:
# - Health checks pass within 10 seconds
# - All endpoints operational
```

---

## 11. SLO Dashboard (Future Enhancement)

**Grafana Dashboard Panels**:
1. Availability % (gauge, 99.5% target)
2. API Latency (line chart, p50/p90/p99)
3. Error Rate % (bar chart, hourly)
4. Concurrent Scans (area chart)
5. Error Budget Remaining (gauge, % remaining)

**URL**: http://grafana.example.com/d/codeqc-slo (not yet implemented)

---

*MAIDOS-CodeQC SLO v0.3.5 -- CodeQC Gate C Compliant*
