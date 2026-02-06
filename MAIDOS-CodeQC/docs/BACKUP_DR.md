# MAIDOS-CodeQC -- Backup and Disaster Recovery

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. What Needs Protection

| Asset | Location | Criticality |
|:------|:---------|:------------|
| Project configuration | `.codeqc.toml` in project root | High |
| Custom rule definitions | `rules/` directory or plugin config | High |
| Plugin binaries | `~/.codeqc/plugins/` | Medium |
| Scan reports (historical) | `reports/` or CI artifact storage | Medium |
| Suppression list | Inline comments and config `[rules.disabled]` | Low |

## 2. Configuration Backup

### 2.1 Version Control (Primary)

All configuration files must be committed to the project's Git repository:

```
.codeqc.toml          # Project scan configuration
rules/                 # Custom rule definitions (if any)
.codeqcignore          # File exclusion patterns
```

**Policy**: Configuration changes require a pull request with review. No direct
commits to main/master for config changes.

### 2.2 Config Export

Export the effective (merged) configuration for archival:

```bash
maidos-codeqc config export > codeqc-config-backup-$(date +%Y%m%d).toml
```

Store exports in a shared drive or artifact repository for historical reference.

## 3. Rule Set Versioning

### 3.1 Built-in Rules

Built-in rules are versioned with the core binary. Each release includes a
`RULES_CHANGELOG.md` documenting rule additions, modifications, and deprecations.

### 3.2 Custom Rules

Custom rules in TOML format should be versioned alongside project source code.
Tag rule set changes with the format: `rules-v{major}.{minor}`.

```bash
git tag rules-v1.3 -m "Added WEB-015: detect unsafe innerHTML"
```

### 3.3 Plugin Rule Sets

Each plugin ships with its own rule definitions. Plugin versions pin their rule
sets. To freeze a known-good rule set:

1. Record plugin versions in `.codeqc.toml`:
   ```toml
   [plugins.pinned]
   web = "2.6.1"
   systems = "2.6.1"
   ```
2. Store plugin binaries in a versioned artifact repository.

## 4. Disaster Recovery Scenarios

### 4.1 Corrupted Configuration

**Symptom**: `config validate` fails; scan produces unexpected results.

**Recovery**:
1. Restore `.codeqc.toml` from Git: `git checkout HEAD -- .codeqc.toml`
2. If Git history is unavailable, use the exported backup.
3. Run `maidos-codeqc config validate` to confirm restoration.

### 4.2 Lost Plugin Binaries

**Symptom**: `plugins list` shows missing plugins; scan skips file types.

**Recovery**:
1. Re-download plugins from the MAIDOS release page matching the pinned versions.
2. Place binaries in `~/.codeqc/plugins/`.
3. Run `maidos-codeqc plugins check` to verify compatibility.

### 4.3 CI Pipeline Broken by Upgrade

**Symptom**: Scans fail after upgrading CodeQC or a plugin.

**Recovery**:
1. Revert to the previous binary version (keep old binaries in artifact storage).
2. Check `RULES_CHANGELOG.md` for breaking rule changes.
3. Update `.codeqc.toml` to accommodate new rules or disable them temporarily.
4. Re-run `maidos-codeqc plugins check` for API version mismatches.

## 5. Backup Schedule

| Asset | Method | Frequency |
|:------|:-------|:----------|
| Configuration files | Git commit | On every change |
| Config export | `config export` to shared storage | Monthly |
| Plugin binaries | Artifact repository snapshot | Per release |
| Historical reports | CI artifact retention policy | 90-day retention |

## 6. Recovery Time Objectives

| Scenario | RTO | RPO |
|:---------|:----|:----|
| Config corruption | < 5 minutes (Git restore) | Last commit |
| Plugin loss | < 15 minutes (re-download) | Pinned version |
| Full environment rebuild | < 30 minutes | Last release |

---

*Review this plan quarterly. Update after any DR scenario is exercised.*
