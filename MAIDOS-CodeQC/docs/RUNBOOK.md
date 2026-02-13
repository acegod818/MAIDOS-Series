# Runbook - MAIDOS-CodeQC

## Overview

This runbook provides troubleshooting procedures, common issues, and operational guidance for MAIDOS-CodeQC v0.3.5. Intended for developers, DevOps engineers, and support teams.

---

## 1. Common Issues and Solutions

### 1.1 Scan Hangs or Times Out

**Symptoms**:
- CLI command `npx maidos-codeqc scan ./src` hangs indefinitely
- No progress output after initial "Scanning..." message
- Process consumes 100% CPU

**Root Causes**:
1. ReDoS (Regular Expression Denial of Service) from pathological input
2. Large files (> 50k lines) exceeding memory limits
3. Infinite loop in custom rules (if plugins installed)

**Diagnosis**:
```bash
# Enable debug mode
CODEQC_DEBUG=1 npx maidos-codeqc scan ./src

# Check which file is being processed
# Debug output shows: [DEBUG] Scanning file: <path>

# Inspect problematic file
wc -l <path>  # Check line count
file <path>   # Check file type
```

**Resolution**:
```bash
# Option 1: Exclude problematic file
echo "excludePatterns:\n  - '**/large-file.js'" > .codeqcrc.yml
npx maidos-codeqc scan ./src

# Option 2: Increase timeout (if rule timeout is too strict)
# Edit src/cli.ts (not configurable in v0.3.5)

# Option 3: Skip AST parsing for large files
# Create issue for CodeQC team to add --skip-ast-large-files flag
```

**Prevention**:
- Add `.codeqcrc.yml` with `excludePatterns` for generated code
- Monitor scan performance with `time` command
- Report ReDoS patterns to CodeQC maintainers

---

### 1.2 False Positives in Test Files

**Symptoms**:
- R01 (hardcoded credentials) flagged in test fixtures
- P13 (TODO accumulation) flagged in mock data
- Console shows violations in `*.test.ts` or `*.spec.py` files

**Root Cause**:
- Test files intentionally contain "bad" code for testing purposes
- CodeQC's exclusion heuristic (`test|spec|mock` in path) may not match all patterns

**Diagnosis**:
```bash
# Check if test files are being scanned
CODEQC_DEBUG=1 npx maidos-codeqc scan ./src | grep -i test

# Expected: Should skip test files
# Actual: If test files appear, exclusion failed
```

**Resolution**:
```bash
# Add explicit exclusions to .codeqcrc.yml
cat > .codeqcrc.yml <<EOF
excludePatterns:
  - "**/*.test.ts"
  - "**/*.spec.ts"
  - "**/tests/**"
  - "**/fixtures/**"
  - "**/__mocks__/**"
EOF

# Re-run scan
npx maidos-codeqc scan ./src
```

**Prevention**:
- Always commit `.codeqcrc.yml` with project-specific exclusions
- Document test directory structure in README

---

### 1.3 CI/CD Pipeline Fails with Exit Code 1

**Symptoms**:
- GitHub Actions or GitLab CI job fails at CodeQC step
- Exit code 1 but no clear violation message in logs
- Logs show "Found N violations" but details are truncated

**Root Cause**:
1. Redline violations (R01-R28) in committed code
2. CI mode (`--ci` flag) exits with 1 on any violation
3. Console output truncated in CI logs

**Diagnosis**:
```bash
# Run locally with same CI command
npx maidos-codeqc scan --ci ./src

# Generate detailed report
npx maidos-codeqc scan -r json -o report.json ./src
cat report.json | jq '.violations[] | select(.severity == "error")'
```

**Resolution**:
```bash
# Fix violations before committing
npx maidos-codeqc scan ./src
# Address all redlines (red errors)

# Alternatively, generate HTML report for CI artifacts
npx maidos-codeqc scan -r html -o codeqc-report.html ./src

# Upload as artifact in GitHub Actions:
# - uses: actions/upload-artifact@v4
#   with:
#     name: codeqc-report
#     path: codeqc-report.html
```

**Prevention**:
- Add pre-commit hook to run `npx maidos-codeqc scan --ci ./src`
- Configure GitHub branch protection to require CodeQC passing

---

### 1.4 Tree-sitter Parser Not Found

**Symptoms**:
- Warning: "Tree-sitter grammar for <language> not found, skipping AST rules"
- P05 (long functions), P06 (deep nesting) not detected in affected language

**Root Cause**:
- Tree-sitter peer dependencies not installed
- CodeQC supports 5 core languages (TS/JS/Python/Rust/Go) but grammars are optional

**Diagnosis**:
```bash
# Check installed peer dependencies
npm list tree-sitter-typescript tree-sitter-python tree-sitter-rust tree-sitter-go

# Expected: All 4 packages listed
# Actual: Some missing
```

**Resolution**:
```bash
# Install all peer dependencies
npm install tree-sitter-typescript tree-sitter-javascript tree-sitter-python tree-sitter-rust tree-sitter-go

# Re-run scan
npx maidos-codeqc scan ./src
```

**Prevention**:
- Document peer dependencies in project README
- Add peer dependencies to `package.json` `devDependencies`

---

### 1.5 Serve Mode WebSocket Connection Fails

**Symptoms**:
- Dashboard loads but shows "Disconnected" status
- Browser console error: "WebSocket connection to 'ws://localhost:3000' failed"
- Real-time updates not working

**Root Cause**:
1. Firewall blocking WebSocket connections
2. Reverse proxy (Nginx) not configured for WebSocket upgrade
3. CORS policy blocking cross-origin WebSocket

**Diagnosis**:
```bash
# Test WebSocket connection manually
npm install -g wscat
wscat -c ws://localhost:3000

# Expected: Connection established
# Actual: Connection refused or timeout
```

**Resolution**:

**Option 1**: Configure Nginx for WebSocket
```nginx
location / {
    proxy_pass http://localhost:3000;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

**Option 2**: Disable firewall for local testing
```bash
sudo ufw allow 3000/tcp  # Ubuntu
sudo firewall-cmd --add-port=3000/tcp --permanent  # CentOS
```

**Option 3**: Use HTTP polling fallback (not implemented in v0.3.5, feature request)

**Prevention**:
- Document WebSocket requirements in deployment guide
- Provide Nginx config template in repo

---

### 1.6 Pipeline Mode Fails at G2 Gate

**Symptoms**:
- Pipeline execution stops at "STEP_09_G2_SPEC"
- Error: "G2 gate failed: Spec coverage below threshold"
- Evidence log shows "Spec coverage: 45% (36/80 items)"

**Root Cause**:
- SPEC.md checklist not fully completed
- G2 gate requires ≥ 80% checkbox completion

**Diagnosis**:
```bash
# Check SPEC.md checklist
grep -c '- \[ \]' docs/SPEC.md  # Incomplete items
grep -c '- \[x\]' docs/SPEC.md  # Completed items

# Calculate coverage
# Coverage = (completed / total) * 100
```

**Resolution**:
```bash
# Option 1: Complete missing spec items
# Edit docs/SPEC.md, mark items as done: - [x]

# Option 2: Lower threshold temporarily (not recommended)
# Edit .codeqcrc.yml
cat > .codeqcrc.yml <<EOF
gates:
  g2:
    threshold: 50  # Lower from 80% to 50%
EOF

# Re-run pipeline
npx maidos-codeqc pipeline . --grade E
```

**Prevention**:
- Maintain SPEC.md checklist in sync with development
- Run pipeline locally before CI/CD deployment

---

### 1.7 High Memory Usage (> 512 MB)

**Symptoms**:
- Node.js process consumes > 512 MB RAM
- System becomes unresponsive during scan
- Out-of-memory error in CI/CD with resource limits

**Root Cause**:
1. Large codebase (> 100k LOC) scanned in single pass
2. Too many concurrent scans in serve mode
3. Memory leak in custom reporter plugin

**Diagnosis**:
```bash
# Monitor memory usage during scan
node --expose-gc --max-old-space-size=512 node_modules/.bin/maidos-codeqc scan ./src

# If OOM occurs, increase heap size
node --max-old-space-size=1024 node_modules/.bin/maidos-codeqc scan ./src
```

**Resolution**:
```bash
# Option 1: Scan subdirectories separately
npx maidos-codeqc scan ./src/module1
npx maidos-codeqc scan ./src/module2

# Option 2: Increase Node.js heap size in CI
# GitHub Actions:
# - run: node --max-old-space-size=1024 node_modules/.bin/maidos-codeqc scan ./src

# Option 3: Exclude large generated files
echo "excludePatterns:\n  - '**/dist/**'\n  - '**/build/**'" > .codeqcrc.yml
```

**Prevention**:
- Profile memory usage with `--expose-gc` during development
- Report memory leaks to CodeQC maintainers

---

## 2. Operational Procedures

### 2.1 Daily Health Check (Serve Mode)

```bash
# Check API server status
curl http://localhost:3000/health

# Expected response:
# {"status":"ok","version":"0.3.5","uptime":86400}

# Check systemd service (Linux)
sudo systemctl status maidos-codeqc

# Check PM2 process
pm2 status codeqc-api
```

### 2.2 Log Rotation (Serve Mode)

```bash
# Logs location (systemd)
sudo journalctl -u maidos-codeqc --since "1 day ago" > /tmp/codeqc-logs.txt

# Logs location (PM2)
pm2 logs codeqc-api --lines 1000 > /tmp/codeqc-logs.txt

# Archive old logs
tar -czf codeqc-logs-$(date +%Y%m%d).tar.gz /tmp/codeqc-logs.txt
```

### 2.3 Database Cleanup (Evidence Files)

```bash
# Evidence files accumulate in pipeline mode
# Clean up old evidence directories

find ./evidence -type d -mtime +30 -exec rm -rf {} \;  # Delete > 30 days old
find ./proof -type f -mtime +30 -exec rm -f {} \;      # Delete old proof packs
```

### 2.4 Version Upgrade Procedure

```bash
# Step 1: Check current version
npx maidos-codeqc --version

# Step 2: Check for updates
npm outdated @maidos/codeqc

# Step 3: Upgrade to latest
npm install @maidos/codeqc@latest

# Step 4: Verify upgrade
npx maidos-codeqc --version

# Step 5: Test scan on sample project
npx maidos-codeqc scan ./test-project

# Step 6: Update CI/CD configs if needed
# Edit .github/workflows/*.yml, update version pin
```

---

## 3. Monitoring and Alerts

### 3.1 Key Metrics to Monitor

| Metric | Threshold | Alert Condition |
|--------|-----------|-----------------|
| Scan duration | < 10 seconds | > 30 seconds for 10k LOC |
| Memory usage | < 512 MB | > 768 MB |
| API response time | < 2 seconds | > 5 seconds |
| WebSocket latency | < 100 ms | > 500 ms |
| Error rate | < 1% | > 5% of scans fail |

### 3.2 Alerting Examples

**Prometheus + Grafana** (if serve mode exposes `/metrics`):
```yaml
# Alert if API response time > 5s
- alert: CodeQCSlowResponse
  expr: http_request_duration_seconds{endpoint="/api/scan"} > 5
  for: 5m
  annotations:
    summary: "CodeQC API slow response"
```

**Log-based alerts** (Elastic Stack, Splunk):
```
# Alert if ERROR appears in logs
search index=codeqc "ERROR" | stats count | where count > 10
```

---

## 4. Disaster Recovery

### 4.1 Backup Procedures

**Config Files**:
```bash
# Backup all config files
tar -czf codeqc-config-backup-$(date +%Y%m%d).tar.gz \
  .codeqcrc.yml \
  .github/workflows/codeqc.yml \
  /etc/systemd/system/maidos-codeqc.service
```

**Evidence and Proof Packs**:
```bash
# Backup all evidence/proof directories
tar -czf codeqc-evidence-backup-$(date +%Y%m%d).tar.gz \
  ./evidence/ \
  ./proof/
```

### 4.2 Restore Procedures

```bash
# Restore config files
tar -xzf codeqc-config-backup-20250213.tar.gz

# Restore evidence/proof
tar -xzf codeqc-evidence-backup-20250213.tar.gz

# Restart services
sudo systemctl restart maidos-codeqc
pm2 restart codeqc-api
```

### 4.3 Rollback to Previous Version

```bash
# Rollback npm package
npm install @maidos/codeqc@0.3.4

# Verify rollback
npx maidos-codeqc --version  # Should show 0.3.4

# Clear npm cache if needed
npm cache clean --force
```

---

## 5. Performance Tuning

### 5.1 Optimize Scan Performance

```bash
# Exclude unnecessary files
cat > .codeqcrc.yml <<EOF
excludePatterns:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/build/**"
  - "**/*.min.js"
  - "**/vendor/**"
EOF

# Disable AST parsing for specific languages (regex-only)
# (Not configurable in v0.3.5, feature request for v0.4.0)
```

### 5.2 Optimize Serve Mode

```bash
# Increase Node.js heap size
node --max-old-space-size=2048 dist/cli.js serve --port 3000

# Enable cluster mode (PM2)
pm2 start dist/cli.js -i 4 --name codeqc-api -- serve --port 3000
# 4 instances for load balancing
```

### 5.3 Reduce False Positives

```bash
# Fine-tune thresholds in .codeqcrc.yml
cat > .codeqcrc.yml <<EOF
thresholds:
  maxFunctionLines: 100   # Increase from default 50
  maxNestingDepth: 4      # Increase from default 3
  maxTodos: 20            # Increase from default 10
EOF
```

---

## 6. Debugging

### 6.1 Enable Debug Logging

```bash
# Set environment variable
export CODEQC_DEBUG=1

# Run command
npx maidos-codeqc scan ./src

# Debug output shows:
# [DEBUG] Loading config from .codeqcrc.yml
# [DEBUG] Discovered 42 files
# [DEBUG] Scanning file: src/index.ts
# [DEBUG] Running rule R01 on src/index.ts
# ...
```

### 6.2 Inspect Internal State

```bash
# Generate JSON report for inspection
npx maidos-codeqc scan -r json -o report.json ./src

# Pretty-print JSON
cat report.json | jq .

# Extract specific fields
cat report.json | jq '.violations[] | {rule, file, line}'
```

### 6.3 Test Specific Rules

```bash
# Create minimal test file
cat > test.ts <<EOF
const apiKey = "sk-1234567890abcdef";  // R01: Hardcoded credential
EOF

# Run scan
npx maidos-codeqc scan test.ts

# Expected: R01 violation reported
```

---

## 7. Contact and Escalation

### 7.1 Support Channels

- **GitHub Issues**: https://github.com/maidos/codeqc/issues
- **Email**: support@maidos.dev (not real, placeholder)
- **Internal Slack**: #maidos-codeqc (if applicable)

### 7.2 Escalation Path

1. **Level 1**: Self-service (this runbook, README.md, TUTORIAL.md)
2. **Level 2**: GitHub Issues (community support)
3. **Level 3**: Maintainer direct contact (critical production issues)

### 7.3 Incident Reporting

When reporting issues, include:
- CodeQC version: `npx maidos-codeqc --version`
- Node.js version: `node --version`
- Operating system: `uname -a` (Linux/Mac) or `ver` (Windows)
- Command executed: Full command with flags
- Error message: Copy-paste full error output
- Debug logs: `CODEQC_DEBUG=1` output

---

*MAIDOS-CodeQC Runbook v0.3.5 -- CodeQC Gate C Compliant*
