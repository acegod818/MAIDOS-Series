# User Journeys - MAIDOS-Driver

## 1. Overview

This document describes the primary user journeys for MAIDOS-Driver, detailing the step-by-step interactions between the user and the application for each core workflow.

## 2. Journey 1: Scan Hardware

### Persona
A user who wants to see what hardware is installed and check driver status.

### Preconditions
- MAIDOS-Driver is installed and launched
- No administrator privileges required for scanning

### Steps

1. **Launch Application**: User opens MAIDOS-Driver from the Start Menu or desktop shortcut
2. **Automatic Detection**: The application begins hardware detection automatically on startup
3. **View Progress**: A progress indicator shows the scan is in progress (typically 5-10 seconds)
4. **Review Results**: The Device List view populates with all detected PCI and USB devices
5. **Inspect Device**: User clicks on a device row to see detailed properties including device name, class, vendor ID, device ID, current driver version, and driver signature status
6. **Filter Devices**: User can filter by device class (Display, Network, Audio, etc.)
7. **Export Report**: User can export the device list to a text file for reference

### Expected Outcome
All PCI and USB devices are listed with accurate vendor/device identification and current driver versions.

---

## 3. Journey 2: Update Drivers

### Persona
A user who wants to update outdated drivers to the latest available versions.

### Preconditions
- Hardware scan has been completed
- Internet connectivity is available for downloading updates
- Administrator privileges are required for installation

### Steps

1. **Check for Updates**: User clicks the "Check for Updates" button
2. **Database Lookup**: The engine queries the TSV database for each detected device
3. **Results Display**: Available updates are shown with version comparison: current version versus available version, source indicator (Database or Windows Update), and download size estimate
4. **Select Updates**: User selects which drivers to update (select all or individual)
5. **Download Phase**: For database-sourced drivers, BITS downloads the package automatically with progress bar showing download status and SHA-256 verification after each download
6. **Signature Verification**: Each downloaded package is verified via WinVerifyTrust
7. **Installation Phase**: If running as admin, drivers are installed via pnputil; if not admin, UAC elevation prompt appears
8. **Windows Update Fallback**: For devices only available via Windows Update, the application opens ms-settings:windowsupdate in the system Settings app
9. **Verification**: Updated devices show new version numbers after refresh

### Expected Outcome
Selected drivers are updated to the latest versions. The user sees confirmation of each successful installation.

---

## 4. Journey 3: Backup Drivers

### Persona
A user preparing for a system reinstall or wanting to preserve current drivers.

### Preconditions
- MAIDOS-Driver is running with administrator privileges
- Sufficient disk space for the backup archive

### Steps

1. **Navigate to Backup**: User clicks the "Backup" tab in the main navigation
2. **Select Destination**: User chooses a folder for the backup ZIP file
3. **Start Backup**: User clicks "Create Backup"
4. **Enumeration Phase**: The engine enumerates all third-party (non-Microsoft) drivers
5. **Collection Phase**: Driver files are collected from the driver store
6. **Compression Phase**: Files are compressed into a ZIP archive with progress indicator
7. **Verification Phase**: SHA-256 checksums are computed for each file in the archive
8. **Completion**: A summary shows number of drivers backed up, total size, and backup file location

### Expected Outcome
A self-contained ZIP archive containing all third-party drivers is created at the specified location.

---

## 5. Journey 4: Restore Drivers

### Persona
A user who has reinstalled Windows and needs to restore previously backed-up drivers.

### Preconditions
- A valid backup ZIP created by MAIDOS-Driver exists
- Administrator privileges are required
- The target system is a fresh or rebuilt Windows installation

### Steps

1. **Navigate to Restore**: User clicks the "Restore" tab in the main navigation
2. **Select Backup**: User browses to and selects the backup ZIP file
3. **Validation Phase**: The engine validates the ZIP structure and checksums
4. **Preview**: A list of drivers in the backup is displayed for review
5. **Select Drivers**: User selects all or specific drivers to restore
6. **Extraction Phase**: Selected drivers are extracted to a temporary directory
7. **Installation Phase**: Each driver is installed via pnputil /add-driver /install
8. **Progress Tracking**: Individual driver installation progress is displayed
9. **Results**: A summary shows successful and failed installations
10. **Cleanup**: Temporary extracted files are removed

### Expected Outcome
Previously backed-up drivers are reinstalled on the system. Any failures are reported with actionable information.

---

## 6. Journey 5: Verify Driver Signatures

### Persona
A security-conscious user who wants to verify the authenticity of installed drivers.

### Preconditions
- Hardware scan has been completed
- No special privileges required for verification alone

### Steps

1. **Navigate to Security**: User clicks the "Security" or "Verify" tab
2. **Start Verification**: User clicks "Verify All Signatures"
3. **Scanning Phase**: Each driver file is checked via WinVerifyTrust
4. **Results Display**: Drivers are categorized as Signed (Valid) with green indicator, Signed (Expired) with yellow indicator, Unsigned with red indicator, or Tampered with red indicator
5. **Detail View**: User clicks a driver to see certificate details including signer identity, certificate expiration date, and certificate chain
6. **Action Recommendations**: Unsigned or tampered drivers show recommended actions

### Expected Outcome
The user has a clear view of the signature status of all installed drivers and can take informed action on any security concerns.

---

## 7. Journey 6: First-Time Setup

### Persona
A new user installing MAIDOS-Driver for the first time.

### Steps

1. **Download Installer**: User obtains MAIDOS-Driver.msi (60 MB)
2. **Run Installer**: Double-click launches WiX MSI installer
3. **Accept License**: User reviews and accepts the license agreement
4. **Choose Location**: Default install path is offered; user may customize
5. **Installation**: Self-contained .NET 10 runtime and Rust DLL are deployed
6. **First Launch**: Application starts, automatically performs initial hardware scan
7. **Review Hardware**: User sees all detected devices immediately
8. **Check Updates**: Application offers to check for driver updates

### Expected Outcome
The application is installed and operational within 2 minutes, with an immediate hardware overview presented to the user.
