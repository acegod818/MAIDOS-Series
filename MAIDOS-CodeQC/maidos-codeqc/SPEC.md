# SPEC — MAIDOS CodeQC (Code-QC v3.5)

> This file is the G2 continuity source of truth for the v3.5 pipeline.
> All items MUST be checked (SPEC 100%) to pass Gate G2.

## Library API

- [x] Analyze a project (multi-file) and return structured results → `analyze`
- [x] Quick check a single source file → `quickCheck`
- [x] Check rules at a given level (B/C/D) → `checkRules`

## CLI (maidos-codeqc)

- [x] Discover supported files and scan a target path (file/dir) → `discoverFiles`
- [x] Support console/json/html reporters → `htmlReporter`
- [x] Support config file (JSON/YAML) + CLI overrides → `loadCliConfig`

## v3.5 Test Bench (pipeline)

- [x] Pipeline command runs the v3.5 10-step wiring flow → `pipelineCommand`
- [x] Pipeline produces evidence logs for scan/fraud/redline/sync/mapping → `runPipeline`
- [x] Pipeline validates Z-axis proof inputs (IAV/BLDS/datasource) → `parseIavLog`
- [x] Pipeline emits DoD verdict + Proof Pack manifest/report → `generateProofPackManifest`
- [x] Pipeline enforces LV4 nonce + LV5 sha256 evidence hash → `resolveProtectionLevel`

## Server

- [x] Serve command starts v3.5 API server (dashboard backend) → `serveCommand`

