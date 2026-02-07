# Backup and Disaster Recovery - MAIDOS-Driver

## 1. Overview

MAIDOS-Driver provides driver backup and restore capabilities to protect against driver-related system failures. This document details the backup architecture, storage format, recovery procedures, and disaster recovery strategies.

## 2. Backup Architecture

### 2.1 What Is Backed Up

| Component | Included | Source |
|-----------|----------|--------|
| Third-party driver files (.sys, .dll) | Yes | Windows Driver Store |
| Driver INF files (.inf) | Yes | Windows Driver Store |
| Catalog files (.cat) | Yes | Windows Driver Store |
| Co-installer DLLs | Yes | Windows Driver Store |
| Microsoft inbox drivers | No | Restored by Windows automatically |
| Driver settings/registry | No | Device-specific, not portable |

### 2.2 Backup Scope

The backup engine targets third-party (OEM) drivers only. Microsoft inbox drivers are excluded because they are always available through Windows itself.

### 2.3 Driver Store Location

The Windows Driver Store is located at C:\Windows\System32\DriverStore\FileRepository\. Each driver package occupies a subdirectory named with the INF file name and a hash suffix.

## 3. Backup Format

### 3.1 ZIP Archive Structure

```
MAIDOS-Driver-Backup-20260206-143022.zip
  manifest.json
  checksums.sha256
  drivers/
    realtek_audio/
      hdaudio.inf
      rtkapo.sys
      rtkapo.cat
    nvidia_gpu/
      nv_dispi.inf
      nvlddmkm.sys
      nvlddmkm.cat
```

### 3.2 Manifest File

The manifest.json contains metadata about the backup: version, creation timestamp, machine name, OS version, driver count, total size, and a list of each driver with its name, INF file, version, vendor/device IDs, and file list.

### 3.3 Checksum File

The checksums.sha256 file contains one SHA-256 hash per file for integrity verification.

## 4. Backup Procedures

### 4.1 Full Backup via UI

1. Open MAIDOS-Driver with administrator privileges
2. Navigate to the Backup tab
3. Click "Create Full Backup"
4. Select the destination folder
5. Wait for the backup to complete (progress bar shown)
6. Verify the summary: number of drivers, total size, file path

### 4.2 Full Backup via FFI

Call backup_drivers(output_path) which returns 0 on success.

### 4.3 Backup Storage Recommendations

| Location | Suitability | Notes |
|----------|-------------|-------|
| External USB drive | Excellent | Survives OS reinstall |
| Network share | Good | Accessible from other machines |
| Local non-system drive | Acceptable | Lost if drive fails |
| Cloud sync folder | Good | Offsite protection via sync |
| System drive (C:) | Poor | Lost during OS reinstall |

### 4.4 Backup Retention

- Keep at least the 3 most recent backups
- Recommended backup frequency: before any major driver update
- Backups before OS upgrades are strongly recommended

## 5. Restore Procedures

### 5.1 Full Restore via UI

1. Open MAIDOS-Driver with administrator privileges
2. Navigate to the Restore tab
3. Click "Browse" and select the backup ZIP file
4. The manifest is read and displayed for review
5. Select "Restore All" or choose individual drivers
6. Confirm the restore operation
7. Wait for all drivers to be installed
8. Review the results summary and reboot if prompted

### 5.2 Selective Restore

Users may restore individual drivers from a backup by expanding the driver list and selecting specific entries.

### 5.3 Restore Validation

Before extracting, the engine validates ZIP structure integrity, manifest version compatibility, SHA-256 checksums, and INF file presence. If validation fails, the restore is aborted.

## 6. Disaster Recovery Scenarios

### 6.1 Blue Screen After Driver Update

1. Boot into Safe Mode
2. Open Device Manager, roll back the offending driver
3. If rollback unavailable, use MAIDOS-Driver to restore from backup
4. Restart normally

### 6.2 No Display After GPU Driver Update

1. Boot into Safe Mode with Networking
2. Uninstall the GPU driver in Device Manager
3. Restart (Windows uses basic display driver)
4. Use MAIDOS-Driver to restore the previous GPU driver

### 6.3 No Network After NIC Driver Update

1. If backup is on local storage: restore the NIC driver via MAIDOS-Driver
2. If backup is on network only: use USB tethering or another NIC
3. Alternatively: Device Manager > Network Adapters > Roll Back Driver

### 6.4 Complete OS Reinstall

1. Install Windows fresh
2. Install MAIDOS-Driver from the MSI installer
3. Navigate to Restore tab, browse to backup ZIP on external drive
4. Restore all drivers and reboot

## 7. Backup Verification

To verify a backup without restoring: open the ZIP, check manifest.json and checksums.sha256 are present, verify file counts match the manifest, and optionally compute checksums.

## 8. Limitations

- Registry-based driver settings are not backed up
- Drivers requiring custom installers (not INF-based) are not supported
- Cross-architecture restore (x86 backup to ARM64) is not supported
- Backup does not include firmware or UEFI drivers
