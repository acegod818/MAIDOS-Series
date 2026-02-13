# Backup and Disaster Recovery - MAIDOS-CodeQC

## Overview

This document defines backup strategies and disaster recovery procedures for MAIDOS-CodeQC. Covers configuration files, evidence data, proof packs, and operational state.

**Scope**: All deployment modes (scan CLI, pipeline, serve)
**Version**: 0.3.5
**RPO (Recovery Point Objective)**: 24 hours
**RTO (Recovery Time Objective)**: 1 hour

---

## 1. Backup Strategy

### 1.1 Data Classification

| Data Type | Criticality | Backup Frequency | Retention |
|-----------|-------------|------------------|-----------|
| **Config Files** (`.codeqcrc.yml`) | High | On change | 90 days |
| **Evidence Logs** (`evidence/*.log`) | Medium | Daily | 30 days |
| **Proof Packs** (`proof/*.zip`) | High | Daily | 1 year |
| **CI/CD Configs** (`.github/workflows/*.yml`) | High | On change | Indefinite (Git) |
| **Serve Mode State** (in-memory) | Low | None (stateless) | N/A |

### 1.2 Backup Locations

| Environment | Primary Storage | Backup Location | Backup Method |
|-------------|-----------------|-----------------|---------------|
| **Local Dev** | `./evidence/`, `./proof/` | External HDD, cloud storage | Manual tar.gz |
| **CI/CD** | GitHub Actions artifacts | GitHub artifact storage | Auto (7 days) |
| **Production Serve** | `/var/lib/codeqc/` | S3, NAS, tape | Cron + rsync/rclone |

---

## 2. Backup Procedures

### 2.1 Manual Backup (Local Development)

```bash
# Create timestamped backup
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="/tmp/codeqc-backup-$TIMESTAMP"

mkdir -p "$BACKUP_DIR"

# Backup config files
cp .codeqcrc.yml "$BACKUP_DIR/" 2>/dev/null || echo "No config file"

# Backup evidence and proof directories
cp -r evidence/ "$BACKUP_DIR/" 2>/dev/null || echo "No evidence"
cp -r proof/ "$BACKUP_DIR/" 2>/dev/null || echo "No proof"

# Backup CI configs
cp -r .github/workflows/ "$BACKUP_DIR/" 2>/dev/null || echo "No CI configs"

# Create compressed archive
tar -czf "codeqc-backup-$TIMESTAMP.tar.gz" -C /tmp "codeqc-backup-$TIMESTAMP"

# Verify archive integrity
tar -tzf "codeqc-backup-$TIMESTAMP.tar.gz" | head -10

# Upload to cloud (example: AWS S3)
aws s3 cp "codeqc-backup-$TIMESTAMP.tar.gz" s3://my-backups/codeqc/

echo "Backup complete: codeqc-backup-$TIMESTAMP.tar.gz"
```

---

### 2.2 Automated Backup (Production Serve Mode)

**Cron Job** (`/etc/cron.d/codeqc-backup`):

```bash
# Daily backup at 2 AM
0 2 * * * root /opt/codeqc/scripts/backup.sh >> /var/log/codeqc-backup.log 2>&1
```

**Backup Script** (`/opt/codeqc/scripts/backup.sh`):

```bash
#!/bin/bash
set -euo pipefail

TIMESTAMP=$(date +%Y%m%d)
BACKUP_DIR="/var/backups/codeqc"
SOURCE_DIR="/var/lib/codeqc"
S3_BUCKET="s3://my-backups/codeqc"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup data
tar -czf "$BACKUP_DIR/codeqc-$TIMESTAMP.tar.gz" \
  -C "$SOURCE_DIR" \
  evidence/ proof/ config/ 2>/dev/null || true

# Calculate checksum
sha256sum "$BACKUP_DIR/codeqc-$TIMESTAMP.tar.gz" > "$BACKUP_DIR/codeqc-$TIMESTAMP.sha256"

# Upload to S3
aws s3 cp "$BACKUP_DIR/codeqc-$TIMESTAMP.tar.gz" "$S3_BUCKET/"
aws s3 cp "$BACKUP_DIR/codeqc-$TIMESTAMP.sha256" "$S3_BUCKET/"

# Clean up local backups older than 7 days
find "$BACKUP_DIR" -name "codeqc-*.tar.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "codeqc-*.sha256" -mtime +7 -delete

echo "Backup complete: $TIMESTAMP"
```

**Set Permissions**:
```bash
chmod +x /opt/codeqc/scripts/backup.sh
chown root:root /opt/codeqc/scripts/backup.sh
```

---

### 2.3 CI/CD Artifact Backup (GitHub Actions)

```yaml
# .github/workflows/codeqc.yml
name: Code Quality
on: [push, pull_request]

jobs:
  codeqc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install CodeQC
        run: npm install @maidos/codeqc

      - name: Run scan with report
        run: |
          npx maidos-codeqc scan -r json -o codeqc-report.json ./src
          npx maidos-codeqc scan -r html -o codeqc-report.html ./src

      - name: Upload reports as artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: codeqc-reports-${{ github.sha }}
          path: |
            codeqc-report.json
            codeqc-report.html
          retention-days: 90  # Retain for 90 days
```

**Download Artifacts**:
```bash
# Via GitHub CLI
gh run download 1234567890 --name codeqc-reports-abc123

# Via web UI
# Navigate to Actions > Workflow run > Artifacts > Download
```

---

## 3. Restore Procedures

### 3.1 Restore from Local Backup

```bash
# Download backup from cloud
aws s3 cp s3://my-backups/codeqc/codeqc-backup-20250213.tar.gz ./

# Verify integrity
sha256sum -c codeqc-backup-20250213.sha256

# Extract backup
tar -xzf codeqc-backup-20250213.tar.gz

# Restore config
cp codeqc-backup-20250213/.codeqcrc.yml ./

# Restore evidence and proof
cp -r codeqc-backup-20250213/evidence/ ./
cp -r codeqc-backup-20250213/proof/ ./

# Verify restored files
ls -lh evidence/ proof/

echo "Restore complete"
```

---

### 3.2 Restore Production Serve Mode

```bash
# Stop service
sudo systemctl stop maidos-codeqc
# OR
pm2 stop codeqc-api

# Download backup from S3
aws s3 cp s3://my-backups/codeqc/codeqc-20250213.tar.gz /tmp/
aws s3 cp s3://my-backups/codeqc/codeqc-20250213.sha256 /tmp/

# Verify integrity
cd /tmp
sha256sum -c codeqc-20250213.sha256

# Extract to staging directory
mkdir -p /tmp/restore
tar -xzf codeqc-20250213.tar.gz -C /tmp/restore

# Backup current data (precaution)
mv /var/lib/codeqc /var/lib/codeqc.old

# Restore data
cp -r /tmp/restore/evidence /var/lib/codeqc/
cp -r /tmp/restore/proof /var/lib/codeqc/
cp -r /tmp/restore/config /var/lib/codeqc/

# Fix permissions
chown -R codeqc:codeqc /var/lib/codeqc

# Restart service
sudo systemctl start maidos-codeqc
# OR
pm2 start codeqc-api

# Verify service health
curl http://localhost:3000/health

echo "Restore complete, service healthy"
```

---

### 3.3 Restore CI/CD Configs

```bash
# Extract CI configs from backup
tar -xzf codeqc-backup-20250213.tar.gz codeqc-backup-20250213/.github/

# Restore to repository
cp -r codeqc-backup-20250213/.github/workflows/ .github/

# Commit and push
git add .github/workflows/
git commit -m "Restore CI/CD configs from backup"
git push origin main
```

---

## 4. Disaster Recovery Scenarios

### 4.1 Scenario: Accidental Evidence Directory Deletion

**Impact**: Loss of audit trail, pipeline re-run required

**Recovery Steps**:
1. Check if backup exists: `ls /var/backups/codeqc/ | grep $(date +%Y%m%d)`
2. Restore from backup (see Section 3.1)
3. If no backup, re-run pipeline: `npx maidos-codeqc pipeline . --grade E`
4. Verify evidence regenerated: `ls -lh evidence/`

**Prevention**:
- Use `rm -i` alias for interactive deletion confirmation
- Implement backup verification script

---

### 4.2 Scenario: Production Server Failure

**Impact**: Serve mode API unavailable, dashboard down

**Recovery Steps**:
1. Provision new server (cloud VM or bare metal)
2. Install Node.js 18+: `curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt-get install -y nodejs`
3. Install CodeQC: `npm install -g @maidos/codeqc`
4. Restore data from backup (see Section 3.2)
5. Configure systemd or PM2 service
6. Update DNS/load balancer to point to new server
7. Verify service health: `curl http://new-server:3000/health`

**RTO Target**: 1 hour

---

### 4.3 Scenario: Corrupted Proof Pack

**Impact**: Audit failure, release blocked

**Recovery Steps**:
1. Identify corrupted proof pack: `unzip -t proof/codeqc-proof-*.zip`
2. Check for backup: `aws s3 ls s3://my-backups/codeqc/proof/`
3. Restore from backup: `aws s3 cp s3://my-backups/codeqc/proof/codeqc-proof-20250213.zip ./proof/`
4. Verify integrity: `unzip -t proof/codeqc-proof-20250213.zip`
5. If no backup, re-run pipeline: `npx maidos-codeqc pipeline . --grade E`
6. Regenerate proof pack

**Prevention**:
- Store proof packs in multiple locations (local + S3 + NAS)
- Verify proof pack integrity immediately after generation

---

### 4.4 Scenario: Config File Lost/Corrupted

**Impact**: Scan behavior reverts to defaults, false positives

**Recovery Steps**:
1. Check Git history: `git log --all --full-history -- .codeqcrc.yml`
2. Restore from Git: `git checkout HEAD~1 -- .codeqcrc.yml`
3. If not in Git, restore from backup: `tar -xzf codeqc-backup-20250213.tar.gz codeqc-backup-20250213/.codeqcrc.yml`
4. Verify config syntax: `npx tsx -e "import('yaml').then(y => y.parse(require('fs').readFileSync('.codeqcrc.yml', 'utf8')))"`
5. Test scan: `npx maidos-codeqc scan ./src`

**Prevention**:
- Always commit `.codeqcrc.yml` to Git
- Document config changes in commit messages

---

### 4.5 Scenario: Entire Repository Deleted

**Impact**: All source code, config, evidence lost

**Recovery Steps**:
1. Restore from Git remote: `git clone https://github.com/maidos/project.git`
2. Restore evidence/proof from backup: `aws s3 cp s3://my-backups/codeqc/codeqc-backup-20250213.tar.gz ./`
3. Extract backup: `tar -xzf codeqc-backup-20250213.tar.gz`
4. Copy evidence/proof to repository: `cp -r codeqc-backup-20250213/evidence/ ./` and `cp -r codeqc-backup-20250213/proof/ ./`
5. Re-run scan to verify: `npx maidos-codeqc scan ./src`

**Prevention**:
- Use Git with remote backup (GitHub, GitLab, Bitbucket)
- Regular backups to multiple locations

---

## 5. Backup Verification

### 5.1 Automated Verification Script

```bash
#!/bin/bash
# verify-backup.sh

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup.tar.gz>"
  exit 1
fi

# Check file exists
if [ ! -f "$BACKUP_FILE" ]; then
  echo "ERROR: Backup file not found"
  exit 1
fi

# Verify archive integrity
echo "Verifying archive integrity..."
tar -tzf "$BACKUP_FILE" > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "✓ Archive integrity OK"
else
  echo "✗ Archive corrupted"
  exit 1
fi

# List contents
echo "Backup contents:"
tar -tzf "$BACKUP_FILE" | head -20

# Check required files
REQUIRED_FILES=("evidence/" "proof/" ".codeqcrc.yml")
for FILE in "${REQUIRED_FILES[@]}"; do
  if tar -tzf "$BACKUP_FILE" | grep -q "$FILE"; then
    echo "✓ Found $FILE"
  else
    echo "⚠ Missing $FILE"
  fi
done

echo "Backup verification complete"
```

**Usage**:
```bash
chmod +x verify-backup.sh
./verify-backup.sh codeqc-backup-20250213.tar.gz
```

---

### 5.2 Test Restore Procedure (Quarterly)

```bash
# Test restore in isolated environment
mkdir -p /tmp/test-restore
cd /tmp/test-restore

# Download latest backup
aws s3 cp s3://my-backups/codeqc/codeqc-backup-20250213.tar.gz ./

# Extract
tar -xzf codeqc-backup-20250213.tar.gz

# Verify key files
ls -lh codeqc-backup-20250213/evidence/
ls -lh codeqc-backup-20250213/proof/
cat codeqc-backup-20250213/.codeqcrc.yml

# Simulate restore
cp -r codeqc-backup-20250213/* .
npx maidos-codeqc scan ./src 2>&1 | tee test-restore.log

# Check for errors
if grep -qi "error" test-restore.log; then
  echo "⚠ Restore test failed"
else
  echo "✓ Restore test passed"
fi

# Clean up
cd ~
rm -rf /tmp/test-restore

echo "Test restore complete"
```

**Schedule**: Run quarterly (every 3 months)

---

## 6. Data Retention Policy

### 6.1 Retention Periods

| Data Type | Retention Period | Reason |
|-----------|------------------|--------|
| **Evidence Logs** | 30 days | Short-term audit, debug |
| **Proof Packs** | 1 year | Compliance, long-term audit |
| **Config Files** | 90 days | Rollback capability |
| **CI Artifacts** | 90 days | Debug past builds |
| **Serve Mode Logs** | 7 days | Operational troubleshooting |

### 6.2 Automated Cleanup

```bash
# Daily cleanup script (cron)
# /etc/cron.daily/codeqc-cleanup

#!/bin/bash
set -euo pipefail

# Clean old evidence logs (> 30 days)
find /var/lib/codeqc/evidence -type f -mtime +30 -delete

# Clean old backups (> 90 days)
find /var/backups/codeqc -name "codeqc-*.tar.gz" -mtime +90 -delete

# Clean old proof packs (> 365 days)
find /var/lib/codeqc/proof -name "*.zip" -mtime +365 -delete

# Clean old logs (> 7 days)
find /var/log/codeqc -name "*.log" -mtime +7 -delete

echo "Cleanup complete: $(date)"
```

---

## 7. Compliance and Audit

### 7.1 Backup Audit Trail

All backup operations must be logged:

```bash
# Append to audit log
echo "$(date +%Y-%m-%d\ %H:%M:%S) - Backup created: codeqc-$TIMESTAMP.tar.gz - User: $USER" >> /var/log/codeqc-backup-audit.log
```

**Audit Log Format**:
```
2025-02-13 02:00:05 - Backup created: codeqc-20250213.tar.gz - User: root
2025-02-13 02:00:12 - Backup uploaded: s3://my-backups/codeqc/codeqc-20250213.tar.gz - User: root
2025-02-14 02:00:07 - Backup created: codeqc-20250214.tar.gz - User: root
```

### 7.2 Proof Pack Integrity Verification

```bash
# Verify proof pack has not been tampered with
unzip -t proof/codeqc-proof-20250213.zip

# Extract manifest
unzip -p proof/codeqc-proof-20250213.zip manifest.json | jq .

# Verify SHA-256 hash
EXPECTED_HASH=$(jq -r .hash manifest.json)
ACTUAL_HASH=$(sha256sum proof/codeqc-proof-20250213.zip | awk '{print $1}')

if [ "$EXPECTED_HASH" == "$ACTUAL_HASH" ]; then
  echo "✓ Proof pack integrity verified"
else
  echo "✗ Proof pack tampered"
  exit 1
fi
```

---

## 8. Disaster Recovery Testing

### 8.1 DR Test Plan (Annual)

**Objective**: Verify RTO (1 hour) and RPO (24 hours) targets

**Test Steps**:
1. Simulate production server failure (shut down VM)
2. Start timer
3. Provision new server
4. Restore from backup
5. Verify service health
6. Stop timer
7. Document results

**Success Criteria**:
- RTO < 1 hour
- Zero data loss (RPO = 0 if backup within 24 hours)
- All services operational
- No errors in logs

**Test Report Template**:
```
DR Test Report - MAIDOS-CodeQC
Date: 2025-02-13
Test Type: Full server recovery
RTO Target: 1 hour
RTO Actual: 47 minutes
RPO Target: 24 hours
RPO Actual: 12 hours (last backup)
Result: PASS
Notes: All services restored successfully, no data loss
```

---

## 9. Emergency Contacts

| Role | Name | Contact |
|------|------|---------|
| **Primary On-Call** | DevOps Engineer | +1-555-0100 |
| **Secondary On-Call** | Engineering Manager | +1-555-0101 |
| **Backup Admin** | SysAdmin | +1-555-0102 |
| **Cloud Provider Support** | AWS Support | 1-866-987-7323 |

---

## 10. Backup Checklist

**Daily**:
- [ ] Automated backup script runs successfully
- [ ] Backup uploaded to S3
- [ ] Backup log entry created

**Weekly**:
- [ ] Verify latest backup integrity (`verify-backup.sh`)
- [ ] Check backup size trends (should not grow unexpectedly)

**Monthly**:
- [ ] Test restore procedure in staging environment
- [ ] Review backup retention policy
- [ ] Audit backup logs for anomalies

**Quarterly**:
- [ ] Full DR test (simulate server failure, restore)
- [ ] Review and update DR documentation
- [ ] Test restore from oldest backup (1 year old proof pack)

**Annually**:
- [ ] Full-scale DR drill with stakeholders
- [ ] Review RTO/RPO targets
- [ ] Update emergency contact list

---

*MAIDOS-CodeQC Backup & DR v0.3.5 -- CodeQC Gate C Compliant*
