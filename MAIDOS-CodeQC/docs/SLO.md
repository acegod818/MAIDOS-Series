# MAIDOS-CodeQC -- Service Level Objectives

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## 1. Scan Performance SLOs

| SLO ID | Metric | Target | Measurement Window |
|:-------|:-------|:-------|:-------------------|
| SLO-PERF-001 | p95 scan time for 10K LOC | < 5 seconds | Rolling 30 days of CI runs |
| SLO-PERF-002 | p95 scan time for 100K LOC | < 30 seconds | Rolling 30 days of CI runs |
| SLO-PERF-003 | p99 plugin load time | < 500 ms per plugin | Per release benchmark suite |
| SLO-PERF-004 | p95 report generation (JSON) | < 1 second | Rolling 30 days of CI runs |
| SLO-PERF-005 | p95 report generation (HTML) | < 3 seconds | Rolling 30 days of CI runs |

## 2. Accuracy SLOs

| SLO ID | Metric | Target | Measurement |
|:-------|:-------|:-------|:------------|
| SLO-ACC-001 | False positive rate | < 2% of total violations | Quarterly audit on MAIDOS codebase |
| SLO-ACC-002 | False negative rate (G1 rules) | < 1% | Mutation testing with known-bad samples |
| SLO-ACC-003 | Fake implementation detection rate | > 98% | Test suite with 200+ stub patterns |
| SLO-ACC-004 | Rule consistency (determinism) | 100% | Same input always produces same output |

## 3. Availability SLOs

| SLO ID | Metric | Target | Context |
|:-------|:-------|:-------|:--------|
| SLO-AVAIL-001 | CLI invocation success rate | > 99.9% | Excludes user config errors |
| SLO-AVAIL-002 | Plugin load success rate | > 99% | Across all supported platforms |
| SLO-AVAIL-003 | Graceful degradation on plugin failure | 100% | Core must never crash from plugin error |

## 4. Resource SLOs

| SLO ID | Metric | Target |
|:-------|:-------|:-------|
| SLO-RES-001 | Peak memory for 100K LOC scan | < 200 MB |
| SLO-RES-002 | Binary size (core, no plugins) | < 15 MB |
| SLO-RES-003 | Disk usage for full plugin suite | < 50 MB |

## 5. Error Budget Policy

- If an SLO is breached for two consecutive measurement windows, a fix is prioritized
  in the next sprint.
- Performance regressions caught in CI benchmarks block the release until resolved.
- Accuracy SLO breaches trigger a review of the affected rule set and test coverage.

## 6. Monitoring

| What | How |
|:-----|:----|
| Scan duration | CI pipeline timestamps; `--timing` flag output |
| Memory usage | `CODEQC_LOG=debug` includes peak RSS in summary |
| False positive reports | Issue tracker label `false-positive` |
| Plugin load failures | Warning log aggregation from CI runs |

---

*SLOs are reviewed quarterly and updated in sync with major releases.*
