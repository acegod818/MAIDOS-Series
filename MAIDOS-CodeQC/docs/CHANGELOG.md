# MAIDOS-CodeQC -- Changelog

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.5               |
| Type      | Changelog          |

## v3.5 (2026-02-13)

### Added
- R19-R28 redline checkers (audit + deep defense layers)
- R19-R28 dedicated unit tests (25+ test cases)
- Hardwarization engine: 5-pillar architecture + LV1-LV9 protection levels
- Product grades E (Commercial) / F (Deep-Tech)
- Pipeline 10-step wiring flow with waveform oscilloscope

### Changed
- **Threshold alignment to Code-QC v3.5 spec:**
  - P05 超長函數: 50 → 100 lines (relaxed)
  - P06 深層嵌套: 3 → 5 levels (relaxed)
  - P10 過長參數: 5 → 6 params (relaxed)
  - P13 TODO 堆積: 10 → 5 items (tightened)
- Version bump: 0.3.3 → 0.3.5 across all source files
- ABCD document references updated to v3.5

### Fixed
- Test thresholds aligned with new prohibition constants
- Server engine version string corrected to v3.5

## v3.0 (2026-02-07)

### Added
- Full CodeQC Gate C compliance documentation (13 docs)
- QC gate scripts (build, unit, integration, e2e, proof)
- Proof pack generation with SHA-256 manifest
- Evidence collection system for all gates

### Changed
- Upgraded pipeline engine for improved reliability
- Standardized plugin interface across all language adapters

### Fixed
- Gate timeout handling for long-running builds
- Web UI dashboard refresh on completed runs

## v2.6.1 (2026-01-15)

### Fixed
- Stabilized gate logic for edge cases
- Fixed plugin detection for monorepo projects

## v2.0 (2025-10-01)

### Added
- Web UI dashboard for QC status visualization
- Evidence collection system
- HTML report generation
- Plugin system for multi-language support

### Changed
- Restructured gate modules for better isolation

## v1.0 (2025-06-01)

### Added
- Initial 4-gate QC pipeline (G1-G4)
- CLI interface (`codeqc.cmd` / `codeqc.sh`)
- Basic spec compliance checking
- Build and test verification
- JSON output format

*MAIDOS-CodeQC CHANGELOG v3.5 -- CodeQC Gate C Compliant*
