# MAIDOS-Driver -- Service Level Objectives

**Product**: MAIDOS-Driver
**Version**: v2.0
**Date**: 2026-02-07
**Status**: Approved

---

## 1. Purpose

This document defines measurable Service Level Objectives (SLOs) for MAIDOS-Driver. As a
local desktop application (not a networked service), SLOs focus on responsiveness, resource
consumption, and operational reliability rather than traditional uptime metrics.

## 2. Scan Latency

| Metric | Target | Measurement |
|:-------|:-------|:------------|
| Full device scan (up to 500 devices) | < 5 seconds | Wall-clock time from `scan_all_devices_c()` call to return |
| Single device diagnosis | < 2 seconds | Wall-clock time for `diagnose_device_c()` |
| Update check (batch, all devices) | < 10 seconds | Wall-clock time for `check_all_updates_c()` including TSV lookup |

**Rationale**: Users perceive delays over 5 seconds as unresponsive. Scan latency is the
primary indicator of application health since it exercises the SetupDI + WMI pipeline.

## 3. Memory Usage

| Metric | Target | Measurement |
|:-------|:-------|:------------|
| Idle memory (after launch, no scan) | < 30 MB | Working set via Task Manager |
| Peak memory (during full scan) | < 50 MB | Peak working set via `tasklist` |
| Post-scan memory (after free) | < 35 MB | Working set after `free_device_info()` returns |

**Rationale**: NFR-002 mandates < 50 MB. The SLO adds idle and post-operation targets to
detect memory leaks over extended usage sessions.

## 4. Operation Completion Time

| Operation | Target | Notes |
|:----------|:-------|:------|
| Driver install (single INF) | < 30 seconds | Includes restore point creation |
| Driver download (100 MB package) | < 120 seconds | Depends on network; BITS provides resume |
| Batch backup (all OEM drivers) | < 60 seconds | pnputil /export-driver * |
| Driver rollback | < 30 seconds | pnputil /add-driver from backup |

## 5. Error Recovery

| Metric | Target |
|:-------|:-------|
| Time from error to user notification | < 1 second |
| Error message clarity | Every error includes a code and human-readable description |
| System state after failed operation | Unchanged (no partial driver installs) |
| Crash rate | 0% (all errors caught and returned as error codes) |

## 6. Availability

| Metric | Target | Notes |
|:-------|:-------|:------|
| Application startup time | < 3 seconds | From process start to UI visible |
| DLL load success rate | 100% | maidOS_driver.dll must load on every launch |
| TSV database load | 100% | drivers.tsv must parse without error |

## 7. Monitoring Approach

Since MAIDOS-Driver is a local desktop application, SLO monitoring is performed through:

1. **Audit log analysis**: Parse `[MAIDOS-AUDIT]` entries for operation timing and failure rates.
2. **Manual benchmarking**: Run scan on reference hardware and measure wall-clock time.
3. **Memory profiling**: Use `tasklist /FI "IMAGENAME eq MAIDOS.Driver.App.exe"` periodically.
4. **Test suite**: `cargo test` and `dotnet test` validate correctness as a proxy for reliability.
