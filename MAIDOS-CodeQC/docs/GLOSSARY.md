# MAIDOS-CodeQC -- Glossary

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Glossary           |

## Terms

| Term                | Definition                                                     |
|---------------------|----------------------------------------------------------------|
| Gate                | A quality checkpoint in the QC pipeline (G1-G4)                |
| G1 (Spec Gate)      | Gate that validates documentation completeness                 |
| G2 (Build Gate)     | Gate that verifies project builds successfully                 |
| G3 (Test Gate)      | Gate that ensures all test suites pass                         |
| G4 (Proof Gate)     | Gate that generates evidence packs with manifests              |
| Pipeline            | The sequential execution of all gates                          |
| Proof Pack          | Collection of evidence files from a QC run                     |
| Manifest            | JSON file listing all evidence with SHA-256 hashes             |
| Evidence            | Logs, artifacts, and outputs collected during gate execution   |
| Plugin              | Language-specific adapter for build/test commands              |
| Gate C Compliant    | Product has passed CodeQC compliance requirements              |
| tsup                | TypeScript bundler used to build MAIDOS-CodeQC                 |
| vitest              | Test runner used for MAIDOS-CodeQC test suites                 |
| CodeQC              | Short name for MAIDOS-CodeQC                                   |
| QC Run              | A single execution of the full pipeline                        |
| Evidence Collector  | Module that gathers and hashes gate outputs                    |
| Web UI              | Browser-based dashboard for viewing QC results                 |
| Proof Directory     | Output folder for generated proof packs (default: `proof/`)   |

## Abbreviations

| Abbreviation | Expansion                               |
|--------------|-----------------------------------------|
| QC           | Quality Control                         |
| CI           | Continuous Integration                  |
| CD           | Continuous Deployment                   |
| CLI          | Command-Line Interface                  |
| UI           | User Interface                          |
| API          | Application Programming Interface       |
| E2E          | End-to-End                              |
| SHA          | Secure Hash Algorithm                   |
| JSON         | JavaScript Object Notation              |
| MAIDOS       | Maid Operating System                   |

*MAIDOS-CodeQC GLOSSARY v3.0 -- CodeQC Gate C Compliant*
