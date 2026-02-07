# MAIDOS-IME -- Alert Definitions

| Field   | Value        |
|---------|--------------|
| Product | MAIDOS-IME   |
| Version | 0.2.0        |

## Alert-001: High Keystroke Latency

| Property   | Value                                      |
|------------|--------------------------------------------|
| Metric     | Keystroke round-trip p99                   |
| Threshold  | > 20 ms for 5 consecutive minutes         |
| Severity   | Warning                                    |
| Action     | Log slow-path trace; check dictionary index|
| Escalation | If sustained > 15 min, file bug P1         |

## Alert-002: Excessive Memory Usage

| Property   | Value                                      |
|------------|--------------------------------------------|
| Metric     | Working set (MB)                           |
| Threshold  | > 120 MB                                   |
| Severity   | Critical                                   |
| Action     | Dump heap snapshot; restart IME service     |
| Escalation | If recurring, hotfix within 48 h           |

## Alert-003: Crash Detected

| Property   | Value                                      |
|------------|--------------------------------------------|
| Metric     | Unhandled exception / access violation     |
| Threshold  | Any occurrence                             |
| Severity   | Critical                                   |
| Action     | Collect minidump; auto-restart via TSF     |
| Escalation | Immediate P0 investigation                 |

## Notification Channels

- Local: Windows Event Log (`MAIDOS-IME` source)
- File: `%APPDATA%/MAIDOS/IME/logs/alerts.log`

*MAIDOS-IME ALERTS v0.2.0 -- CodeQC Gate C Compliant*
