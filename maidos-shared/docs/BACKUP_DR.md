# maidos-shared — Backup and Disaster Recovery

## 1. Purpose

This document defines backup strategies and disaster recovery procedures for state and
configuration managed by maidos-shared modules.

---

## 2. Data Classification

| Data Type | Module | Criticality | Location |
|-----------|--------|-------------|----------|
| Auth tokens | maidos-auth | High | `~/.maidos/tokens.json` |
| Configuration files | maidos-config | High | Application config directory |
| Audit chain | maidos-chain | High | Application data directory |
| Log files | maidos-log | Medium | Configured log output path |
| Peer roster | maidos-p2p | Low | Runtime memory / config |
| LLM model cache | maidos-llm | Low | Ollama data directory |

---

## 3. Backup Procedures

### 3.1 Configuration Backup

**Frequency:** On every successful config load and before config changes.

```
1. Copy current config file to <config_dir>/backups/<timestamp>.toml
2. Retain last 10 backups; delete older ones.
3. On config parse failure, log CFG-002 and attempt rollback to latest backup.
```

### 3.2 Auth Token Backup

**Frequency:** On every successful token refresh.

```
1. Write new token atomically (write to temp file, then rename).
2. Previous token file serves as implicit backup during rename window.
3. If token file is corrupted on read, delete and trigger re-authentication.
```

### 3.3 Audit Chain Backup

**Frequency:** After every 100 entries or daily, whichever comes first.

```
1. Export chain to <data_dir>/chain_backups/<timestamp>.json
2. Verify exported chain integrity (hash validation).
3. Retain last 30 days of backups.
4. Optionally replicate to secondary storage via consumer application.
```

### 3.4 Log File Management

**Frequency:** Continuous via rotation policy.

```
1. Rotate at 100 MB per file.
2. Retain 5 rotated files (500 MB total maximum).
3. Compressed rotated files with gzip to reduce storage.
```

---

## 4. Disaster Recovery Procedures

### 4.1 Corrupted Configuration

| Step | Action |
|------|--------|
| 1 | Detect: `ConfigError::ParseError` on load |
| 2 | Log alert CFG-002 |
| 3 | Attempt to load from `backups/` directory (most recent first) |
| 4 | If all backups fail, fall back to compiled-in default config |
| 5 | Notify consumer application via bus event or callback |

### 4.2 Lost Auth Tokens

| Step | Action |
|------|--------|
| 1 | Detect: token file missing or unreadable |
| 2 | Log alert AUTH-003 |
| 3 | Transition auth state to `Disconnected` |
| 4 | Prompt user to re-authenticate via consumer application UI |
| 5 | New tokens are persisted atomically on success |

### 4.3 Corrupted Audit Chain

| Step | Action |
|------|--------|
| 1 | Detect: hash verification failure during chain read |
| 2 | Log the corruption point (entry index and expected vs. actual hash) |
| 3 | Load most recent verified backup from `chain_backups/` |
| 4 | Entries after the backup point are lost; log the gap |
| 5 | Resume appending from the restored chain head |

### 4.4 P2P Network Partition

| Step | Action |
|------|--------|
| 1 | Detect: all peers enter `Disconnected` state |
| 2 | Log alert P2P-003 |
| 3 | Retry discovery with exponential backoff (max 5 minutes) |
| 4 | If manual bootstrap peers are configured, attempt direct connection |
| 5 | Consumer application continues in offline mode until peers recover |

---

## 5. Recovery Time Objectives

| Scenario | RTO | RPO |
|----------|-----|-----|
| Config corruption | < 5 seconds (auto-rollback) | Last successful load |
| Token loss | < 30 seconds (re-auth prompt) | Current session |
| Chain corruption | < 10 seconds (backup restore) | Last backup (max 1 day) |
| Network partition | Variable (depends on network) | No data loss (retry queue) |

## 6. Testing

Disaster recovery procedures are validated quarterly by:
1. Deliberately corrupting config files and verifying auto-rollback.
2. Deleting token cache and confirming re-authentication flow.
3. Injecting invalid entries into the audit chain and verifying detection.
4. Simulating network partitions and confirming reconnection behavior.
