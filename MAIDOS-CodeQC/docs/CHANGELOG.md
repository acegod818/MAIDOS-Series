# MAIDOS-CodeQC -- Changelog

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Changelog          |

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

*MAIDOS-CodeQC CHANGELOG v3.0 -- CodeQC Gate C Compliant*
