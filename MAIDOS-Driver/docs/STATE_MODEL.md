# MAIDOS-Driver -- State Model

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Device States

MAIDOS-Driver tracks each hardware device through a set of states derived from the Windows
Configuration Manager (CM) API. The `CM_Get_DevNode_Status` function provides the raw status
and problem code for each device node.

### 1.1 State Diagram

```
                    +------------------+
                    |     Unknown      |  (device not yet scanned)
                    +--------+---------+
                             |
                         scan_all_devices_c()
                             |
              +--------------+--------------+
              |                             |
    +---------v---------+        +----------v----------+
    |      Normal       |        |    Problem (Code N) |
    | status = Running  |        | status = Problem    |
    | problem_code = 0  |        | problem_code = N    |
    +---+-------+---+---+        +---+--------+--------+
        |       |   |                |        |
        |       |   +--- rollback ---+        |
        |       |                             |
        |  install/update             diagnose_device_c()
        |       |                             |
        |  +----v---------+          +--------v--------+
        |  |   Updating   |          |   Diagnosed     |
        |  | (transient)  |          | IRQ + code info |
        |  +----+---------+          +-----------------+
        |       |
        |  success / failure
        |       |
        +-------+
```

### 1.2 State Definitions

| State | CM Problem Code | Description |
|:------|:----------------|:------------|
| Normal | 0 | Device is functioning correctly. Driver loaded and running. |
| Problem (Code 1) | 1 | Device is not configured correctly. |
| Problem (Code 10) | 10 | Device cannot start. |
| Problem (Code 22) | 22 | Device is disabled by the user. |
| Problem (Code 28) | 28 | Drivers for this device are not installed. |
| Problem (Code 31) | 31 | Device is not working properly; Windows cannot load required drivers. |
| Problem (Code 43) | 43 | Windows has stopped this device because it has reported problems. |
| Missing Driver | 28 | Subset of Problem; no driver installed at all. |
| Updating | -- | Transient state during install_driver_c or apply_update_c. |
| Rolling Back | -- | Transient state during rollback_driver_c. |

## 2. Driver Lifecycle States

A driver package progresses through these stages within MAIDOS-Driver:

```
[Not Installed] --install_driver_c()--> [Installed]
[Installed]     --check_*_update_c()--> [Update Available]
[Update Available] --download_update_c()--> [Downloaded]
[Downloaded]    --apply_update_c()----> [Updated / Installed]
[Installed]     --rollback_driver_c()--> [Rolled Back]
[Installed]     --backup_drivers_c()--> [Backed Up]
[Backed Up]     --restore (pnputil)---> [Restored / Installed]
```

## 3. Audit State Transitions

Every state transition generates an audit log entry:

| Transition | Audit Operation | Example Entry |
|:-----------|:----------------|:--------------|
| Not Installed -> Installed | INSTALL | `[MAIDOS-AUDIT] 2026-02-07 INSTALL PCI\VEN_8086 SUCCESS` |
| Installed -> Updated | APPLY_UPDATE | `[MAIDOS-AUDIT] 2026-02-07 APPLY_UPDATE PCI\VEN_10DE SUCCESS` |
| Installed -> Rolled Back | ROLLBACK | `[MAIDOS-AUDIT] 2026-02-07 ROLLBACK PCI\VEN_1002 SUCCESS` |
| Any -> Backed Up | BACKUP | `[MAIDOS-AUDIT] 2026-02-07 BACKUP ALL SUCCESS` |
| Any -> Scanned | SCAN | `[MAIDOS-AUDIT] 2026-02-07 SCAN ALL 142_DEVICES` |

## 4. Error States

When an operation fails, the device returns to its previous state. The error is captured via
`get_last_error()` and written to the audit log with the `FAILURE` result code. No partial
state changes are persisted.
