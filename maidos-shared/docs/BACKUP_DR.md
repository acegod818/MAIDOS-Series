# Backup and Disaster Recovery - maidos-shared

## Overview

maidos-shared is a stateless library with minimal persistent state. However, consuming applications may need to back up token stores, configuration files, and provider credentials. This document outlines backup strategies and recovery procedures.

---

## Backup Scope

### In-Scope for Backup

| Component | Data Type | Criticality | Frequency |
|-----------|-----------|-------------|-----------|
| **Token Store** | Active tokens, revocation list | High | Hourly |
| **Configuration Files** | TOML config, secrets | High | On change |
| **LLM API Keys** | Provider credentials | Critical | On rotation |
| **Application Logs** | Structured logs (JSON) | Medium | Daily |
| **Metrics History** | Performance metrics | Low | Weekly |

### Out-of-Scope

- Compiled binaries (reproducible from source)
- LLM response cache (ephemeral, can be regenerated)
- Temporary files in `/tmp`

---

## Backup Strategy

### BS-001: Token Store Backup

**Objective**: Prevent token loss during service restarts or crashes

**Method**: Periodic snapshot to disk

```rust
use maidos_auth::TokenStore;
use std::fs::File;

// Serialize token store to JSON
let store = issuer.export_store()?;
let file = File::create("backup/tokens.json")?;
serde_json::to_writer_pretty(file, &store)?;
```

**Frequency**: Every hour (cron job)

**Retention**: 24 hours (24 snapshots), then rotate

**Storage**: Local disk (`/var/backups/maidos/tokens/`) + S3 (for redundancy)

**Cron Job**:
```bash
# /etc/cron.d/maidos-token-backup
0 * * * * root /usr/local/bin/backup-tokens.sh
```

**backup-tokens.sh**:
```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/maidos/tokens"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/tokens_$TIMESTAMP.json"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Export tokens via FFI or API
curl -s http://localhost:8080/api/admin/export-tokens > "$BACKUP_FILE"

# Compress
gzip "$BACKUP_FILE"

# Upload to S3 (optional)
aws s3 cp "$BACKUP_FILE.gz" s3://maidos-backups/tokens/

# Rotate old backups (keep last 24 hours)
find "$BACKUP_DIR" -name "tokens_*.json.gz" -mtime +1 -delete

echo "Token backup completed: $BACKUP_FILE.gz"
```

---

### BS-002: Configuration File Backup

**Objective**: Preserve config history for rollback

**Method**: Git versioning + file system backup

**Git Tracking**:
```bash
# Initialize git repo for config
cd /etc/maidos/
git init
git add config.toml secrets.toml
git commit -m "Initial config"

# On each change
git add config.toml
git commit -m "Update LLM provider settings"
```

**File System Backup**:
```bash
# /etc/cron.daily/maidos-config-backup
#!/bin/bash
rsync -av /etc/maidos/ /var/backups/maidos/config/$(date +%Y%m%d)/
aws s3 sync /var/backups/maidos/config/ s3://maidos-backups/config/
```

**Frequency**: On change (git) + daily (S3)

**Retention**: Git history (all commits) + 30 days of file backups

---

### BS-003: LLM API Key Backup

**Objective**: Securely store provider credentials for disaster recovery

**Method**: Encrypted backup to secret management system

**AWS Secrets Manager**:
```bash
# Store API keys in AWS Secrets Manager
aws secretsmanager create-secret \
  --name maidos/llm-api-keys \
  --secret-string '{"openai":"sk-...","anthropic":"sk-ant-..."}'

# Retrieve in application
aws secretsmanager get-secret-value \
  --secret-id maidos/llm-api-keys \
  --query SecretString \
  --output text | jq -r '.openai'
```

**Local Encrypted Backup** (fallback):
```bash
# Encrypt with GPG
gpg --encrypt --recipient admin@maidos.dev -o api-keys.gpg api-keys.json

# Decrypt for recovery
gpg --decrypt api-keys.gpg > api-keys.json
```

**Frequency**: On key rotation (manual)

**Retention**: All versions (Secrets Manager handles versioning)

---

### BS-004: Application Logs Backup

**Objective**: Retain logs for debugging and audit

**Method**: Log aggregation + archival

**Shipping Logs to S3**:
```bash
# /etc/cron.daily/maidos-logs-backup
#!/bin/bash
LOG_DIR="/var/log/maidos"
DATE=$(date -d yesterday +%Y-%m-%d)

# Compress yesterday's logs
tar -czf "$LOG_DIR/archive/logs_$DATE.tar.gz" "$LOG_DIR"/*.log

# Upload to S3
aws s3 cp "$LOG_DIR/archive/logs_$DATE.tar.gz" s3://maidos-backups/logs/

# Rotate local logs (keep last 7 days)
find "$LOG_DIR/archive" -name "logs_*.tar.gz" -mtime +7 -delete
```

**Frequency**: Daily

**Retention**: 7 days local + 90 days S3 + 1 year Glacier (cold storage)

---

### BS-005: Metrics History Backup

**Objective**: Preserve performance trends for capacity planning

**Method**: Prometheus/Grafana snapshot

**Prometheus Snapshot**:
```bash
# Take Prometheus snapshot
curl -X POST http://localhost:9090/api/v1/admin/tsdb/snapshot

# Backup snapshot to S3
SNAPSHOT_DIR=$(ls -td /prometheus/data/snapshots/* | head -1)
tar -czf prometheus-snapshot-$(date +%Y%m%d).tar.gz "$SNAPSHOT_DIR"
aws s3 cp prometheus-snapshot-$(date +%Y%m%d).tar.gz s3://maidos-backups/metrics/
```

**Frequency**: Weekly

**Retention**: 4 weeks local + 1 year S3

---

## Recovery Procedures

### RP-001: Token Store Recovery

**Scenario**: Service crashes, token store lost

**Steps**:

1. **Identify Latest Backup**
   ```bash
   aws s3 ls s3://maidos-backups/tokens/ | tail -1
   # Output: tokens_20260213_140000.json.gz
   ```

2. **Download and Decompress**
   ```bash
   aws s3 cp s3://maidos-backups/tokens/tokens_20260213_140000.json.gz .
   gunzip tokens_20260213_140000.json.gz
   ```

3. **Import to Token Store**
   ```rust
   use maidos_auth::TokenStore;

   let file = File::open("tokens_20260213_140000.json")?;
   let store: TokenStore = serde_json::from_reader(file)?;
   issuer.import_store(store)?;
   ```

4. **Verify Token Count**
   ```rust
   let count = issuer.token_count();
   println!("Recovered {} tokens", count);
   ```

5. **Resume Service**
   ```bash
   systemctl start maidos-auth-service
   systemctl status maidos-auth-service
   ```

**Recovery Time Objective (RTO)**: < 5 minutes
**Recovery Point Objective (RPO)**: < 1 hour (last hourly backup)

---

### RP-002: Configuration Rollback

**Scenario**: Bad config deployed, service misbehaving

**Steps**:

1. **Identify Last Known Good Commit**
   ```bash
   cd /etc/maidos/
   git log --oneline
   # Output:
   # abc123 Update LLM provider settings (BAD)
   # def456 Add hot reload config (GOOD)
   ```

2. **Revert to Previous Commit**
   ```bash
   git revert abc123
   # Or hard reset (if no one else pulled bad config)
   git reset --hard def456
   ```

3. **Verify Config**
   ```bash
   maidos-config-validate config.toml
   # Output: Config is valid
   ```

4. **Reload Config (Hot Reload)**
   ```bash
   # If hot reload enabled, just save file
   # Otherwise, restart service
   systemctl restart maidos-service
   ```

5. **Monitor for Stability**
   ```bash
   journalctl -u maidos-service -f
   # Watch for "Config reloaded successfully"
   ```

**RTO**: < 2 minutes (git revert)
**RPO**: 0 (git tracks all changes)

---

### RP-003: API Key Compromise Recovery

**Scenario**: LLM API key leaked, needs rotation

**Steps**:

1. **Revoke Compromised Key**
   - OpenAI: https://platform.openai.com/api-keys → Revoke
   - Anthropic: https://console.anthropic.com/settings/keys → Delete

2. **Generate New Key**
   - Create new key in provider dashboard
   - Copy new key to clipboard

3. **Update Secrets Manager**
   ```bash
   aws secretsmanager update-secret \
     --secret-id maidos/llm-api-keys \
     --secret-string '{"openai":"sk-NEW-KEY","anthropic":"sk-ant-..."}'
   ```

4. **Update Application Config**
   ```bash
   # If using env vars
   export OPENAI_API_KEY="sk-NEW-KEY"
   systemctl restart maidos-service

   # If using config file
   sed -i 's/sk-OLD-KEY/sk-NEW-KEY/' /etc/maidos/secrets.toml
   # Hot reload will pick up change
   ```

5. **Verify New Key Works**
   ```bash
   curl -H "Authorization: Bearer sk-NEW-KEY" \
        https://api.openai.com/v1/models
   # Should return model list
   ```

6. **Audit Usage Logs**
   - Check provider dashboard for suspicious activity
   - Review application logs for unauthorized requests

**RTO**: < 10 minutes
**RPO**: N/A (no data loss, key rotation)

---

### RP-004: Complete Service Rebuild

**Scenario**: Catastrophic failure (disk corruption, ransomware)

**Steps**:

1. **Provision New Server**
   ```bash
   # AWS EC2, Azure VM, or on-prem
   terraform apply
   ```

2. **Install Dependencies**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   apt-get update && apt-get install -y build-essential libssl-dev
   ```

3. **Clone Application Repository**
   ```bash
   git clone https://github.com/maidos/maidos-app.git
   cd maidos-app
   ```

4. **Restore Configuration**
   ```bash
   aws s3 cp s3://maidos-backups/config/$(date +%Y%m%d)/ /etc/maidos/ --recursive
   ```

5. **Restore Secrets**
   ```bash
   aws secretsmanager get-secret-value \
     --secret-id maidos/llm-api-keys \
     --query SecretString \
     --output text > /etc/maidos/secrets.toml
   ```

6. **Build Application**
   ```bash
   cargo build --release
   cp target/release/maidos-app /usr/local/bin/
   ```

7. **Restore Token Store**
   ```bash
   aws s3 cp s3://maidos-backups/tokens/latest.json.gz .
   gunzip latest.json.gz
   # Import via API or file system
   ```

8. **Start Service**
   ```bash
   systemctl enable maidos-app
   systemctl start maidos-app
   ```

9. **Verify Health**
   ```bash
   curl http://localhost:8080/health
   # Expected: {"status":"healthy","uptime":...}
   ```

10. **Restore Monitoring**
    ```bash
    # Restore Prometheus/Grafana dashboards
    aws s3 cp s3://maidos-backups/grafana-dashboards/ /etc/grafana/provisioning/dashboards/ --recursive
    systemctl restart grafana-server
    ```

**RTO**: < 1 hour (automated with Terraform)
**RPO**: < 1 hour (last token backup)

---

## Disaster Recovery Testing

### DR Drill Schedule

| Test | Frequency | Scope | Success Criteria |
|------|-----------|-------|------------------|
| **Token Store Recovery** | Monthly | Restore from backup | < 5 min RTO, zero token loss |
| **Config Rollback** | Quarterly | Git revert bad config | < 2 min RTO, service stable |
| **API Key Rotation** | Annually | Rotate all provider keys | < 10 min RTO, zero request failures |
| **Full Service Rebuild** | Semi-annually | Complete DR scenario | < 1 hour RTO, < 1 hour RPO |

### DR Drill Procedure

1. **Schedule Drill**: Announce 1 week in advance to team
2. **Backup Current State**: Take full snapshot before drill
3. **Simulate Failure**: Shut down service, delete data
4. **Execute Recovery**: Follow RP procedures
5. **Validate Recovery**: Run smoke tests
6. **Document Results**: Record RTO/RPO actuals vs targets
7. **Post-Mortem**: Identify gaps, update procedures

### Sample DR Drill Log

```
Date: 2026-02-13
Test: Token Store Recovery (RP-001)
Scenario: Simulated disk failure, token store lost

Timeline:
14:00 - Failure simulated (systemctl stop + rm tokens.db)
14:01 - Latest backup identified (tokens_20260213_130000.json.gz)
14:02 - Backup downloaded from S3
14:03 - Token store imported (12,345 tokens)
14:04 - Service restarted
14:05 - Health check passed

Results:
- RTO: 5 minutes (target: < 5 min) ✅
- RPO: 1 hour (target: < 1 hour) ✅
- Token Loss: 0 (target: 0) ✅

Issues:
- None

Action Items:
- None (test passed)
```

---

## Backup Verification

### BV-001: Automated Backup Testing

**Cron Job** (weekly):
```bash
# /etc/cron.weekly/maidos-backup-test
#!/bin/bash
set -euo pipefail

# Test token backup restore
LATEST_BACKUP=$(aws s3 ls s3://maidos-backups/tokens/ | tail -1 | awk '{print $4}')
aws s3 cp "s3://maidos-backups/tokens/$LATEST_BACKUP" /tmp/test-restore.json.gz
gunzip /tmp/test-restore.json.gz

# Verify JSON is valid
jq empty /tmp/test-restore.json || {
  echo "ERROR: Token backup is corrupted"
  exit 1
}

# Test config backup
aws s3 cp s3://maidos-backups/config/latest/config.toml /tmp/test-config.toml
maidos-config-validate /tmp/test-config.toml || {
  echo "ERROR: Config backup is invalid"
  exit 1
}

echo "Backup verification PASSED"
rm /tmp/test-restore.json /tmp/test-config.toml
```

---

## Backup Monitoring

### Metrics

- `backup.last_success_timestamp`: Unix timestamp of last successful backup
- `backup.size_bytes`: Size of latest backup file
- `backup.duration_seconds`: Time taken for backup operation
- `backup.failure_count`: Number of consecutive failures

### Alerts

**ALERT-BACKUP-001: Backup Failed**
- **Condition**: `backup.failure_count > 0` for 1 hour
- **Severity**: Critical
- **Action**: Check cron logs, verify S3 credentials

**ALERT-BACKUP-002: Backup Too Old**
- **Condition**: `time() - backup.last_success_timestamp > 7200` (2 hours)
- **Severity**: Warning
- **Action**: Verify cron job is running

**ALERT-BACKUP-003: Backup Size Anomaly**
- **Condition**: `backup.size_bytes < 0.5 * avg_over_time(backup.size_bytes[7d])`
- **Severity**: Warning
- **Action**: Backup may be incomplete, investigate

---

## Security Considerations

### Encryption at Rest

All backups stored in S3 use AES-256 encryption:
```bash
aws s3 cp file.json s3://maidos-backups/ \
  --server-side-encryption AES256
```

### Access Control

S3 bucket policy (least privilege):
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "BackupServiceWrite",
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::123456789012:role/MaidosBackupService"
      },
      "Action": ["s3:PutObject"],
      "Resource": "arn:aws:s3:::maidos-backups/*"
    },
    {
      "Sid": "AdminRead",
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::123456789012:role/MaidosAdmin"
      },
      "Action": ["s3:GetObject", "s3:ListBucket"],
      "Resource": [
        "arn:aws:s3:::maidos-backups",
        "arn:aws:s3:::maidos-backups/*"
      ]
    }
  ]
}
```

### Audit Logging

Enable CloudTrail for S3 bucket access:
```bash
aws cloudtrail create-trail \
  --name maidos-backup-trail \
  --s3-bucket-name maidos-audit-logs

aws cloudtrail start-logging --name maidos-backup-trail
```

---

## Contact Information

- **Backup Administrator**: backup-admin@maidos.dev
- **DR Coordinator**: dr-coordinator@maidos.dev
- **On-Call Engineer**: Slack #maidos-oncall
- **AWS Support**: +1-866-221-0634 (24/7)

---

## Appendix: Backup Checklist

**Pre-Production Checklist**:
- [ ] S3 bucket created with versioning enabled
- [ ] IAM roles configured with least privilege
- [ ] Cron jobs scheduled (hourly tokens, daily logs)
- [ ] Backup verification script tested
- [ ] DR procedures documented and reviewed
- [ ] DR drill scheduled (first month after launch)

**Monthly Verification**:
- [ ] Run backup verification script
- [ ] Check S3 storage costs (should be < $50/month)
- [ ] Review backup retention (delete old backups)
- [ ] Test token store recovery (< 5 min RTO)

**Quarterly Review**:
- [ ] Audit S3 access logs for anomalies
- [ ] Update DR procedures if architecture changed
- [ ] Run config rollback DR drill
- [ ] Review RTO/RPO targets with stakeholders
