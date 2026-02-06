# MAIDOS-IME v2.0 - Backup and Disaster Recovery

## 1. Purpose

This document defines backup procedures and disaster recovery strategies for MAIDOS-IME
user data, configuration, and dictionaries.

## 2. Data Classification

| Data | Location | Criticality | Backup Frequency |
|------|----------|-------------|------------------|
| User dictionary | `%AppData%\MAIDOS\IME\user_dict.json` | High | On every modification |
| User settings | `%AppData%\MAIDOS\IME\settings.json` | Medium | On every modification |
| Dictionary cache | `%LocalAppData%\MAIDOS\IME\cache\` | Low | Not backed up (regenerated) |
| Engine logs | `%LocalAppData%\MAIDOS\IME\logs\` | Low | Not backed up |
| System dictionaries | `C:\Program Files\MAIDOS\IME\data\` | Low | Recoverable via reinstall |

## 3. Backup Procedures

### 3.1 Automatic Backup (User Dictionary)

The Rust engine creates an automatic backup of `user_dict.json` before every write
operation:

- Backup file: `user_dict.json.bak` (single rolling backup in same directory).
- If the write fails, the `.bak` file is used to restore automatically on next launch.

### 3.2 Manual Export

Users can export their data via the Manager application:

1. Open MAIDOS-IME Manager.
2. Navigate to Settings > Data > Export.
3. Choose export location. The export produces a single ZIP file containing:
   - `user_dict.json` - User-added words and phrases.
   - `settings.json` - All user preferences.
   - `export_meta.json` - Export timestamp and version info.

### 3.3 FFI Export Function

Programmatic export is available via the Rust engine FFI:

```c
int32_t ime_user_dict_export(const char* file_path);  // Returns 0 on success
```

## 4. Restore Procedures

### 4.1 Restore from Export

1. Open MAIDOS-IME Manager.
2. Navigate to Settings > Data > Import.
3. Select the previously exported ZIP file.
4. Manager extracts and places files in the correct directories.
5. Engine is restarted to load the restored data.

### 4.2 Restore from Automatic Backup

If `user_dict.json` is corrupted:
1. The engine detects JSON parse failure during `ime_init`.
2. Engine automatically copies `user_dict.json.bak` to `user_dict.json`.
3. If the backup is also corrupted, engine starts with an empty user dictionary.
4. An ALT-002 alert is raised (see ALERTS.md).

### 4.3 FFI Import Function

```c
int32_t ime_user_dict_import(const char* file_path);  // Returns 0 on success
```

## 5. Disaster Recovery Scenarios

### 5.1 Complete Reinstallation

If the MAIDOS-IME installation is damaged beyond repair:

1. Uninstall via Windows Settings > Apps > MAIDOS-IME > Uninstall.
2. Verify COM entries are removed: `reg query "HKLM\SOFTWARE\Microsoft\CTF\TIP" /s`.
3. Reinstall from MSI.
4. Import user data from the most recent export ZIP.

### 5.2 OS Migration

When migrating to a new Windows installation:

1. On old system: Export data via Manager (Section 3.2).
2. Copy the export ZIP to the new system.
3. Install MAIDOS-IME on the new system.
4. Import data via Manager (Section 4.1).

### 5.3 Dictionary Corruption

If system dictionaries are corrupted:

1. Delete the cache directory: `%LocalAppData%\MAIDOS\IME\cache\`.
2. Run Manager > Dictionaries > Repair to re-download JSON dictionaries.
3. Cache will be regenerated on next engine startup.

## 6. Recovery Time Objectives

| Scenario | RTO |
|----------|-----|
| User dictionary restore from backup | < 5 seconds |
| Full reinstall with data import | < 10 minutes |
| Dictionary cache rebuild | < 5 seconds |
| OS migration with export/import | < 15 minutes |

## 7. References

- DEPLOY.md - Installation and file locations
- RUNBOOK.md - Operational troubleshooting
- CONTRACT.md - FFI export/import functions
