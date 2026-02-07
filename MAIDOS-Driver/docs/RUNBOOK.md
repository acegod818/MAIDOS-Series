# Operations Runbook - MAIDOS-Driver

## 1. Overview

This runbook provides troubleshooting procedures, diagnostic steps, and operational guidance for MAIDOS-Driver.

## 2. Log Locations

| Log Type | Location |
|----------|----------|
| Engine logs | %ProgramData%\MAIDOS\DriverManager\logs\ |
| MSI install logs | Created with /l*v flag during installation |
| Windows Event Log | Event Viewer > Application > Source: MAIDOS-Driver |
| pnputil output | Captured in engine logs during driver installation |

### Enable Debug Logging

Set environment variable MAIDOS_LOG=debug before launching the application.

## 3. Common Issues and Resolutions

### 3.1 WMI Enumeration Fails

**Symptoms**: Hardware detection returns an empty list or times out.

**Diagnostic Steps**:
1. Verify WMI service is running: sc query winmgmt
2. Test WMI from PowerShell: Get-WmiObject -Class Win32_PnPEntity
3. Check for WMI repository corruption: winmgmt /verifyrepository

**Resolution**:
- If WMI is stopped: net start winmgmt
- If repository is corrupt: winmgmt /resetrepository (requires reboot)
- If WMI returns WBEM_S_NO_MORE_DATA prematurely: check Group Policy restrictions

### 3.2 SetupDI Returns Incomplete Data

**Symptoms**: Some devices show missing names or Unknown Device.

**Resolution**:
- Missing device names are expected for unrecognized hardware
- Ensure the TSV database is up to date for name resolution
- Unknown devices may need manual driver identification

### 3.3 BITS Download Fails

**Symptoms**: Driver downloads fail with error code 2.

**Diagnostic Steps**:
1. Verify BITS service: sc query bits
2. Check network connectivity to the download URL
3. Verify the URL is HTTPS
4. Check BITS job queue: bitsadmin /list /allusers

**Resolution**:
- Restart BITS: net stop bits && net start bits
- Clear stuck BITS jobs: bitsadmin /reset /allusers
- If behind a proxy: configure BITS proxy settings via Group Policy

### 3.4 Driver Installation Fails

**Symptoms**: apply_update() returns non-zero, driver not installed.

**Resolution**:
- Ensure the application is running as administrator
- Verify the .inf file is valid and the driver package is complete
- Check if the driver signature is valid
- For Secure Boot issues: the driver must be WHQL signed

### 3.5 SHA-256 Checksum Mismatch

**Symptoms**: Download completes but verification fails with error code 3.

**Resolution**:
- Retry the download (may be transient corruption)
- If persistent: the TSV database entry may have an incorrect hash
- Check for man-in-the-middle proxy rewriting content

### 3.6 Application Fails to Start

**Resolution**:
- Verify maidOS_driver.dll exists alongside the .exe
- Check for missing .NET runtime dependencies
- Reinstall via MSI to restore missing files
- If DLL load fails: ensure Visual C++ Redistributable is installed

## 4. Driver Rollback Procedures

### 4.1 Rollback via Device Manager
1. Open Device Manager (devmgmt.msc)
2. Right-click the affected device > Properties
3. Driver tab > Roll Back Driver

### 4.2 Rollback via MAIDOS-Driver Backup
1. Open MAIDOS-Driver, navigate to Restore tab
2. Select the backup ZIP created before the update
3. Select the specific driver to restore, click Restore

### 4.3 Safe Mode Recovery
If the system fails to boot after a driver update:
1. Boot into Safe Mode (hold Shift during restart)
2. Open Device Manager
3. Roll back or uninstall the problematic driver
4. Restart normally

## 5. Health Checks

Verify DLL: check maidOS_driver.dll exists in the install directory.
Verify WMI: wmic path Win32_PnPEntity get Name returns results.
Verify BITS: sc query bits shows RUNNING.
Verify database: drivers.tsv exists in the data directory.

## 6. Maintenance Tasks

| Task | Frequency | Command |
|------|-----------|---------|
| Update TSV database | Monthly | Replace drivers.tsv and restart |
| Clean old logs | Automatic | Log rotation at 10 MB, 5 files |
| Verify installation | After updates | Run health check script |
| Clear BITS queue | If downloads stall | bitsadmin /reset |
