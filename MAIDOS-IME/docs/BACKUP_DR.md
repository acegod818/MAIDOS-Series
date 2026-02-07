# MAIDOS-IME -- Backup & Disaster Recovery

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Critical Data

| Asset           | Location                            | RPO    |
|-----------------|-------------------------------------|--------|
| User dictionary | %APPDATA%/MAIDOS/IME/user_dict.db  | 1 day  |
| Config          | %APPDATA%/MAIDOS/IME/config.toml   | 7 days |
| AI model cache  | %LOCALAPPDATA%/MAIDOS/IME/models/  | N/A    |

## Backup Strategy

- **Dictionary**: Auto-backup on every write to `user_dict.db.bak`
- **Config**: Copy on startup to `config.toml.bak`
- **Models**: Re-downloadable; no backup required

## Manual Backup

```bat
xcopy "%APPDATA%/MAIDOS/IME/user_dict.db" "D:/backup/" /Y
xcopy "%APPDATA%/MAIDOS/IME/config.toml"   "D:/backup/" /Y
```

## Restore Procedure

1. Unregister current IME: `regsvr32 /u /s maidos_ime.dll`
2. Copy backup files to `%APPDATA%/MAIDOS/IME/`
3. Re-register: `regsvr32 /s maidos_ime.dll`
4. Verify dictionary loads (type test input)

## Disaster Scenarios

| Scenario              | Recovery                             | RTO    |
|-----------------------|--------------------------------------|--------|
| Dictionary corrupted  | Restore from `.bak`; re-index        | 5 min  |
| DLL missing / corrupt | Reinstall MSI                        | 10 min |
| Full OS reinstall     | Install MSI + restore dictionary bak | 15 min |

*MAIDOS-IME BACKUP_DR v0.2.0 -- CodeQC Gate C Compliant*
