# State Model - MAIDOS-Driver

## 1. Overview

This document defines the state models used in MAIDOS-Driver to track the lifecycle of detected devices, driver update operations, and backup/restore processes.

## 2. Device Lifecycle State Model

### 2.1 State Diagram

```
                    detect_hardware()
                         |
                         v
  +----------+    +-----------+    +----------+    +-----------+    +----------+
  | Unknown  |--->| Detected  |--->| Matched  |--->| Updated   |--->| Verified |
  +----------+    +-----------+    +----------+    +-----------+    +----------+
       |               |               |                |
       |               |               v                v
       |               |          +----------+    +-----------+
       |               +--------->| No Match |    |  Failed   |
       |                          +----------+    +-----------+
       v
  +-----------+
  |  Error    |
  +-----------+
```

### 2.2 State Definitions

| State | Description | Entry Condition |
|-------|-------------|-----------------|
| **Unknown** | Device exists on the bus but has not been enumerated | Initial state before scan |
| **Detected** | Device enumerated via WMI with VEN/DEV IDs extracted | detect_hardware() returns device info |
| **Matched** | Device matched to a driver entry in TSV database or WU | check_all_updates() finds a match |
| **No Match** | Device has no known driver in database or WU | check_all_updates() finds no match |
| **Updated** | New driver downloaded and installed via pnputil | apply_update() returns success |
| **Failed** | Driver update attempted but installation failed | apply_update() returns error |
| **Verified** | Installed driver passes Authenticode signature check | verify_driver_signature() returns 0 |
| **Error** | Device enumeration or identification failed | WMI/SetupDI error during detection |

### 2.3 Transition Rules

| From | To | Trigger | Guard Condition |
|------|----|---------|-----------------|
| Unknown | Detected | WMI enumeration | Valid VEN/DEV ID extracted |
| Unknown | Error | WMI failure | WMI query timeout or COM error |
| Detected | Matched | Database lookup | TSV entry found for VEN&DEV pair |
| Detected | No Match | Database lookup | No entry found, WU returns no match |
| Matched | Updated | Driver installation | pnputil returns success |
| Matched | Failed | Driver installation | pnputil returns error |
| Updated | Verified | Signature check | WinVerifyTrust returns 0 |
| Failed | Matched | Retry after rollback | Previous driver restored |

## 3. Download Operation State Model

```
  +--------+    +-------------+    +----------+    +----------+    +--------+
  | Queued |--->| Downloading |--->| Verifying|--->| Complete |--->| Ready  |
  +--------+    +-------------+    +----------+    +----------+    +--------+
                      |                 |
                      v                 v
                 +---------+      +-----------+
                 | Stalled |      | Corrupted |
                 +---------+      +-----------+
                      |                 |
                      v                 v
                 +---------+      +---------+
                 |  Retry  |----->| Failed  |
                 +---------+      +---------+
```

| State | Description |
|-------|-------------|
| **Queued** | Download request accepted, waiting for BITS slot |
| **Downloading** | BITS transfer in progress |
| **Stalled** | BITS transfer stalled due to network issues |
| **Retry** | Automatic retry after stall (up to 3 attempts) |
| **Verifying** | Download complete, SHA-256 checksum being computed |
| **Corrupted** | SHA-256 checksum does not match expected value |
| **Complete** | Download verified, file ready for installation |
| **Ready** | File moved to staging directory, ready for pnputil |
| **Failed** | Download permanently failed after all retries |

## 4. Backup/Restore State Model

### 4.1 Backup States

```
  +------+    +-------------+    +-------------+    +----------+    +------+
  | Init |--->| Enumerating |--->| Collecting  |--->| Zipping  |--->| Done |
  +------+    +-------------+    +-------------+    +----------+    +------+
```

### 4.2 Restore States

```
  +------+    +------------+    +------------+    +------------+    +------+
  | Init |--->| Validating |--->| Extracting |--->| Installing |--->| Done |
  +------+    +------------+    +------------+    +------------+    +------+
                    |                                    |
                    v                                    v
               +---------+                          +---------+
               | Invalid |                          | Partial |
               +---------+                          +---------+
```

## 5. Application Lifecycle

```
  +-----------+    +----------+    +--------+    +----------+    +----------+
  | Launching |--->| Loading  |--->| Ready  |--->| Scanning |--->|  Idle    |
  +-----------+    +----------+    +--------+    +----------+    +----------+
```

| State | Description | Timeout |
|-------|-------------|---------|
| **Launching** | Process started, .NET runtime initializing | 3s |
| **Loading** | DLL loaded, WMI COM initializing | 2s |
| **Ready** | UI rendered, awaiting user action | - |
| **Scanning** | Hardware detection in progress | 10s |
| **Idle** | Scan complete, results displayed | - |

## 6. State Persistence

Device states are held in memory during the application session and are not persisted to disk between sessions. Each application launch begins a fresh state cycle starting from Unknown for all devices.

Backup metadata (timestamps, file paths) is persisted to support restore operations across sessions.
