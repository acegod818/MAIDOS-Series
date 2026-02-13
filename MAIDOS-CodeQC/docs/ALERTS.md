# Alert Conditions - MAIDOS-CodeQC

## Overview

This document defines alert conditions for MAIDOS-CodeQC's operational monitoring and CI/CD integration. Alerts are categorized by severity and include remediation procedures.

**Scope**: Serve mode monitoring + CI/CD integration
**Version**: 0.3.5
**Alert Channels**: Slack, Email, PagerDuty (configurable)

---

## 1. Alert Severity Levels

| Severity | Description | Response Time | Notification |
|----------|-------------|---------------|--------------|
| **P0 - Critical** | Service down, data loss risk | 5 minutes | Page on-call + Slack |
| **P1 - High** | Severe degradation, SLO breach | 15 minutes | Slack + Email |
| **P2 - Medium** | Performance issues, warnings | 1 hour | Slack only |
| **P3 - Low** | Informational, no action needed | 24 hours | Email digest |

---

## 2. Serve Mode Alerts

### 2.1 API Server Down (P0 - Critical)

**Condition**:
```yaml
# Health check fails for 3 consecutive attempts (5 minutes)
GET /health
Expected: HTTP 200 {"status": "ok"}
Actual: Connection refused OR HTTP 503/500 OR Timeout (> 10s)
```

**Trigger Logic**:
```bash
# Monitoring script (run every 1 minute via cron)
curl -f http://localhost:3000/health || echo "ALERT: API DOWN"
```

**Alert Message**:
```
🚨 CRITICAL: MAIDOS-CodeQC API Server Down
Service: codeqc-api
Host: prod-server-01
Status: Health check failed (3 consecutive failures)
Impact: All scans blocked, dashboard unavailable
Action: Restart service immediately
Runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md#api-down
```

**Remediation**:
1. Check service status: `sudo systemctl status maidos-codeqc` or `pm2 status codeqc-api`
2. Restart service: `sudo systemctl restart maidos-codeqc` or `pm2 restart codeqc-api`
3. Check logs: `sudo journalctl -u maidos-codeqc -n 50` or `pm2 logs codeqc-api`
4. If restart fails, escalate to P0 on-call engineer

**Auto-Remediation** (optional):
```bash
# Systemd auto-restart
Restart=on-failure
RestartSec=10s

# PM2 auto-restart
pm2 start dist/cli.js --name codeqc-api --max-restarts 3 --restart-delay 10000
```

---

### 2.2 High Error Rate (P1 - High)

**Condition**:
```yaml
# 5xx error rate > 5% over 15 minutes
(count(http_status=5xx) / count(http_status=*)) > 0.05
```

**Trigger Logic**:
```bash
# Parse logs (run every 5 minutes)
ERROR_COUNT=$(grep "HTTP 5" /var/log/codeqc-api.log | grep -c "$(date -d '15 minutes ago' +%H:%M)")
TOTAL_COUNT=$(grep "HTTP" /var/log/codeqc-api.log | grep -c "$(date -d '15 minutes ago' +%H:%M)")
ERROR_RATE=$(echo "scale=2; $ERROR_COUNT / $TOTAL_COUNT * 100" | bc)

if (( $(echo "$ERROR_RATE > 5" | bc -l) )); then
  echo "ALERT: High error rate $ERROR_RATE%"
fi
```

**Alert Message**:
```
⚠️ HIGH: MAIDOS-CodeQC High Error Rate
Service: codeqc-api
Host: prod-server-01
Error Rate: 8.3% (12 errors / 145 requests in 15 min)
Impact: Some scans failing, user experience degraded
Action: Investigate logs, check for recent deployments
Runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md#high-error-rate
```

**Remediation**:
1. Check recent deployments: `pm2 logs codeqc-api --lines 100 | grep ERROR`
2. Identify error patterns: `grep "HTTP 5" /var/log/codeqc-api.log | awk '{print $8}' | sort | uniq -c`
3. If specific rule causing errors, disable via config
4. If deployment-related, rollback: `npm install @maidos/codeqc@0.3.4 && pm2 restart codeqc-api`
5. Monitor error rate for 15 minutes post-fix

---

### 2.3 High API Latency (P1 - High)

**Condition**:
```yaml
# p90 response time > 20 seconds over 10 minutes
percentile(http_request_duration_seconds{endpoint="/api/scan"}, 0.9) > 20
```

**Trigger Logic**:
```bash
# Extract response times from logs (run every 5 minutes)
tail -1000 /var/log/codeqc-api.log | grep "POST /api/scan" | awk '{print $NF}' | sort -n > /tmp/latencies.txt
P90=$(awk 'BEGIN{c=0} {a[c++]=$1} END{print a[int(c*0.9)]}' /tmp/latencies.txt)

if (( $(echo "$P90 > 20" | bc -l) )); then
  echo "ALERT: High API latency p90=$P90s"
fi
```

**Alert Message**:
```
⚠️ HIGH: MAIDOS-CodeQC High API Latency
Service: codeqc-api
Host: prod-server-01
p90 Latency: 23.4 seconds (threshold: 20s)
Impact: Slow scan responses, poor user experience
Action: Check system load, optimize queries
Runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md#high-latency
```

**Remediation**:
1. Check system load: `top`, `htop`, `vmstat`
2. Check concurrent scans: `curl http://localhost:3000/health | jq .activeScans`
3. Check disk I/O: `iostat -x 1 10`
4. If CPU-bound, scale horizontally: `pm2 scale codeqc-api +2`
5. If memory-bound, increase heap size: `node --max-old-space-size=2048`

---

### 2.4 WebSocket Connection Failures (P2 - Medium)

**Condition**:
```yaml
# WebSocket connections fail > 10% over 10 minutes
(count(websocket_connection_status=failed) / count(websocket_connection_attempts)) > 0.1
```

**Trigger Logic**:
```bash
# Parse logs for WebSocket errors (run every 5 minutes)
WS_FAIL=$(grep "WebSocket connection failed" /var/log/codeqc-api.log | grep -c "$(date -d '10 minutes ago' +%H:%M)")
if (( $WS_FAIL > 5 )); then
  echo "ALERT: WebSocket connection failures"
fi
```

**Alert Message**:
```
🔶 MEDIUM: MAIDOS-CodeQC WebSocket Failures
Service: codeqc-api
Host: prod-server-01
Failure Rate: 12% (8 failures / 67 attempts in 10 min)
Impact: Dashboard real-time updates not working
Action: Check reverse proxy config, firewall rules
Runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md#websocket-failures
```

**Remediation**:
1. Check Nginx WebSocket config: `sudo nginx -t`
2. Check firewall: `sudo ufw status` or `sudo firewall-cmd --list-all`
3. Test WebSocket manually: `wscat -c ws://localhost:3000`
4. Check browser console for CORS errors
5. Restart Nginx if config changed: `sudo systemctl restart nginx`

---

### 2.5 High Memory Usage (P2 - Medium)

**Condition**:
```yaml
# Memory usage > 768 MB
process_resident_memory_bytes > 805306368  # 768 MB in bytes
```

**Trigger Logic**:
```bash
# Check memory usage (run every 5 minutes)
MEM_MB=$(ps aux | grep "node.*codeqc" | awk '{sum += $6} END {print sum/1024}')
if (( $(echo "$MEM_MB > 768" | bc -l) )); then
  echo "ALERT: High memory usage ${MEM_MB}MB"
fi
```

**Alert Message**:
```
🔶 MEDIUM: MAIDOS-CodeQC High Memory Usage
Service: codeqc-api
Host: prod-server-01
Memory: 823 MB / 768 MB threshold
Impact: Risk of OOM, service restart
Action: Check for memory leaks, restart if needed
Runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md#high-memory
```

**Remediation**:
1. Check heap usage: `node --expose-gc --max-old-space-size=512 dist/cli.js serve`
2. Restart service if memory keeps growing: `pm2 restart codeqc-api`
3. Investigate memory leaks: `node --inspect dist/cli.js serve` + Chrome DevTools
4. If persistent, report to maintainers with heap snapshot

---

## 3. CI/CD Integration Alerts

### 3.1 CI Pipeline Failure (P2 - Medium)

**Condition**:
```yaml
# GitHub Actions or GitLab CI job fails with exit code 1
npx maidos-codeqc scan --ci ./src
Exit Code: 1
```

**Trigger Logic**:
```yaml
# GitHub Actions workflow
- name: Run CodeQC
  run: npx maidos-codeqc scan --ci ./src
  continue-on-error: false  # Fail pipeline on violations
```

**Alert Message**:
```
🔶 MEDIUM: CodeQC CI Check Failed
Repository: maidos/example-project
Branch: feature/new-auth
Commit: a1b2c3d4
Violations: 3 redlines (R01, R05, R10)
Impact: PR blocked from merge
Action: Fix violations and re-push
Report: https://github.com/maidos/example-project/actions/runs/12345
```

**Remediation**:
1. Download CI artifact: `codeqc-report.json` or `codeqc-report.html`
2. Review violations locally: `npx maidos-codeqc scan ./src`
3. Fix all redlines (R01-R28)
4. Re-run locally: `npx maidos-codeqc scan --ci ./src` (should exit 0)
5. Push fix, CI re-runs automatically

---

### 3.2 High Violation Count (P3 - Low)

**Condition**:
```yaml
# CI scan finds > 50 violations (any severity)
Total Violations: 73
```

**Trigger Logic**:
```bash
# Parse JSON report in CI
VIOLATION_COUNT=$(cat codeqc-report.json | jq '.violations | length')
if (( $VIOLATION_COUNT > 50 )); then
  echo "WARNING: High violation count $VIOLATION_COUNT"
fi
```

**Alert Message**:
```
ℹ️ LOW: CodeQC High Violation Count
Repository: maidos/example-project
Branch: develop
Violations: 73 (12 redlines, 61 prohibitions)
Impact: Code quality degrading, tech debt accumulating
Action: Schedule refactoring sprint
Report: https://github.com/maidos/example-project/actions/runs/12345
```

**Remediation**:
1. Generate HTML report for team review
2. Prioritize redlines (security issues first)
3. Create tickets for top 10 offending files
4. Track violation trend over time (weekly)
5. Set team goal: reduce violations by 20% per month

---

### 3.3 CI Scan Timeout (P2 - Medium)

**Condition**:
```yaml
# CI job times out after 10 minutes
timeout: 10m
```

**Trigger Logic**:
```yaml
# GitHub Actions timeout
- name: Run CodeQC
  run: npx maidos-codeqc scan --ci ./src
  timeout-minutes: 10
```

**Alert Message**:
```
🔶 MEDIUM: CodeQC CI Scan Timeout
Repository: maidos/large-monorepo
Branch: main
Duration: 10 minutes (timeout)
Impact: CI pipeline blocked, no quality check
Action: Optimize scan or increase timeout
Report: https://github.com/maidos/large-monorepo/actions/runs/12345
```

**Remediation**:
1. Check project size: `find ./src -type f | wc -l`
2. Add exclusions to `.codeqcrc.yml` (node_modules, dist, build)
3. Split scan by directory: `npx maidos-codeqc scan ./src/module1 && npx maidos-codeqc scan ./src/module2`
4. Increase timeout: `timeout-minutes: 20`
5. Report performance issue to maintainers

---

## 4. Security Alerts

### 4.1 Critical Redline Detected (P1 - High)

**Condition**:
```yaml
# R01, R02, R07, or R10 violation detected in production code
Rules: R01 (Hardcoded Credentials), R02 (SQL Injection), R07 (Disable Security), R10 (Plaintext Transmission)
```

**Trigger Logic**:
```bash
# Parse scan report
CRITICAL=$(cat codeqc-report.json | jq -r '.violations[] | select(.rule | startswith("R01") or startswith("R02") or startswith("R07") or startswith("R10")) | .file')
if [ -n "$CRITICAL" ]; then
  echo "ALERT: Critical security violation detected"
fi
```

**Alert Message**:
```
⚠️ HIGH: CodeQC Critical Security Violation
Repository: maidos/payment-service
Branch: main
Violation: R01 - Hardcoded API Key
File: src/config.ts:15
Code: const apiKey = "sk-1234567890abcdef"
Impact: Security breach risk, credential exposure
Action: Rotate credentials immediately, fix code
Report: https://github.com/maidos/payment-service/actions/runs/12345
```

**Remediation**:
1. **URGENT**: Rotate exposed credentials (API keys, passwords, tokens)
2. Fix code: move credentials to environment variables or secret manager
3. Scan commit history for exposed secrets: `git log -p | grep -i "password\|api_key"`
4. If public repo, assume credential compromised
5. Add pre-commit hook to block future commits with secrets

---

### 4.2 Antifraud Violation (P1 - High)

**Condition**:
```yaml
# R13-R18 violations (test faking, ghosting, code whitewashing)
Rules: R13 (Dead Code), R14 (Ghost Async), R16 (Fake Test Pass), R17 (Fake Data Source), R18 (Mock Abuse)
```

**Alert Message**:
```
⚠️ HIGH: CodeQC Antifraud Violation Detected
Repository: maidos/billing-system
Branch: release/v2.0
Violation: R16 - Fake Test Pass (force pass via always-true assertion)
File: tests/payment.test.ts:42
Code: expect(true).toBe(true);  // R16: Always-pass test
Impact: Test coverage fraud, false confidence
Action: Rewrite test with real assertions
Report: https://github.com/maidos/billing-system/actions/runs/12345
```

**Remediation**:
1. Review test file, rewrite with meaningful assertions
2. Check test coverage report for anomalies
3. Run pipeline mode for full audit: `npx maidos-codeqc pipeline . --grade E`
4. If intentional fraud, escalate to engineering leadership

---

## 5. Alert Configuration

### 5.1 Slack Webhook Integration

```bash
# Send alert to Slack
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "🚨 CRITICAL: MAIDOS-CodeQC API Server Down",
    "attachments": [{
      "color": "danger",
      "fields": [
        {"title": "Service", "value": "codeqc-api", "short": true},
        {"title": "Host", "value": "prod-server-01", "short": true},
        {"title": "Impact", "value": "All scans blocked", "short": false}
      ]
    }]
  }'
```

### 5.2 PagerDuty Integration

```bash
# Trigger PagerDuty incident
curl -X POST https://events.pagerduty.com/v2/enqueue \
  -H 'Content-Type: application/json' \
  -d '{
    "routing_key": "YOUR_INTEGRATION_KEY",
    "event_action": "trigger",
    "payload": {
      "summary": "MAIDOS-CodeQC API Server Down",
      "severity": "critical",
      "source": "prod-server-01",
      "custom_details": {
        "service": "codeqc-api",
        "runbook": "https://github.com/maidos/codeqc/docs/RUNBOOK.md"
      }
    }
  }'
```

### 5.3 Email Alerts

```bash
# Send email via mailx
echo "MAIDOS-CodeQC API Server Down. Check runbook: https://github.com/maidos/codeqc/docs/RUNBOOK.md" | \
  mailx -s "CRITICAL: CodeQC API Down" ops@example.com
```

---

## 6. Alert Suppression

### 6.1 Maintenance Windows

During planned maintenance, suppress alerts:

```bash
# Create maintenance marker file
touch /var/lib/codeqc/maintenance_mode

# Monitoring script checks for marker
if [ -f /var/lib/codeqc/maintenance_mode ]; then
  echo "Maintenance mode active, skipping alerts"
  exit 0
fi

# Remove marker after maintenance
rm /var/lib/codeqc/maintenance_mode
```

### 6.2 Flapping Detection

Prevent alert spam from flapping services:

```bash
# Only alert if condition persists for 3 consecutive checks
FAIL_COUNT=$(cat /tmp/codeqc_fail_count 2>/dev/null || echo 0)
if curl -f http://localhost:3000/health; then
  echo 0 > /tmp/codeqc_fail_count  # Reset on success
else
  FAIL_COUNT=$((FAIL_COUNT + 1))
  echo $FAIL_COUNT > /tmp/codeqc_fail_count
  if (( $FAIL_COUNT >= 3 )); then
    echo "ALERT: API Down (3 consecutive failures)"
  fi
fi
```

---

## 7. Alert Testing

### 7.1 Test Alert Delivery

```bash
# Test Slack webhook
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -d '{"text": "TEST: CodeQC alert system check"}'

# Test PagerDuty
curl -X POST https://events.pagerduty.com/v2/enqueue \
  -H 'Content-Type: application/json' \
  -d '{"routing_key": "YOUR_KEY", "event_action": "trigger", "payload": {"summary": "TEST", "severity": "info"}}'
```

### 7.2 Simulate Alert Conditions

```bash
# Simulate API down
pm2 stop codeqc-api
sleep 300  # Wait 5 minutes for alert
pm2 start codeqc-api

# Simulate high error rate
# Inject bad requests to /api/scan
for i in {1..20}; do
  curl -X POST http://localhost:3000/api/scan -d '{"path": "/nonexistent"}' &
done
```

---

## 8. Alert Escalation Matrix

| Alert | P0 (Critical) | P1 (High) | P2 (Medium) | P3 (Low) |
|-------|---------------|-----------|-------------|----------|
| **Initial Response** | 5 min | 15 min | 1 hour | 24 hours |
| **Escalation 1** | 15 min | 1 hour | 4 hours | N/A |
| **Escalation 2** | 30 min | 2 hours | 8 hours | N/A |
| **Escalation 3** | 1 hour | 4 hours | Next day | N/A |

**Escalation Contacts**:
- P0: On-call engineer → VP Engineering → CTO
- P1: Team lead → Engineering manager → VP Engineering
- P2: Team lead → Engineering manager
- P3: Team lead (email digest)

---

*MAIDOS-CodeQC Alert Conditions v0.3.5 -- CodeQC Gate C Compliant*
