# MAIDOS-CodeQC -- Architecture Document

| Field     | Value                  |
|-----------|------------------------|
| Product   | MAIDOS-CodeQC          |
| Version   | v3.0                   |
| Type      | Architecture Overview  |

## High-Level Architecture

```
+---------------------+
|    codeqc.cmd/sh    |   Entry points (CLI)
+---------+-----------+
          |
+---------v-----------+
|   Pipeline Engine   |   Gate orchestration (G1-G4)
+---------+-----------+
          |
+---------v-----------+     +-----------------+
|    Gate Modules     |---->|   Plugin System  |
|  G1  G2  G3  G4    |     |  (per-language)  |
+---------+-----------+     +-----------------+
          |
+---------v-----------+
|  Evidence Collector |   Logs, artifacts, timestamps
+---------+-----------+
          |
+---------v-----------+
|   Web UI Dashboard  |   Status viewer (browser)
+---------------------+
```

## Module Breakdown

| Module              | Location               | Responsibility                          |
|---------------------|------------------------|-----------------------------------------|
| Pipeline Engine     | `src/pipeline/`        | Orchestrates G1-G4 in sequence          |
| Gate G1 - Spec      | `src/gates/spec/`      | Validates docs/ and manifest structure  |
| Gate G2 - Build     | `src/gates/build/`     | Executes build commands, checks output  |
| Gate G3 - Test      | `src/gates/test/`      | Runs unit/integration/e2e suites        |
| Gate G4 - Proof     | `src/gates/proof/`     | Generates proof pack with evidence      |
| Plugin System       | `src/plugins/`         | Language-specific adapters              |
| Evidence Collector  | `src/evidence/`        | Gathers logs and artifacts              |
| Web UI              | `web-ui/`              | Dashboard for QC status display         |
| Config Loader       | `src/config/`          | Reads project settings and gate configs |
| Reporter            | `src/reporter/`        | Formats output (console, JSON, HTML)    |

## Data Flow

1. User invokes `codeqc.cmd` or `codeqc.sh` with a target project path
2. Pipeline Engine loads project configuration
3. Gates execute sequentially: G1 -> G2 -> G3 -> G4
4. Each gate produces a result (pass/fail) and evidence artifacts
5. Evidence Collector aggregates all artifacts
6. Proof Pack is generated with manifest and timestamps
7. Results are displayed via CLI and optionally on Web UI

## Plugin Architecture

| Plugin                   | Target Languages / Frameworks      |
|--------------------------|------------------------------------|
| plugin-config            | Configuration validation           |
| plugin-data              | Data processing pipelines          |
| plugin-dotnet            | C# / .NET / WPF projects          |
| plugin-enterprise        | Enterprise Java / Spring           |
| plugin-functional        | Functional languages (F#, Elixir)  |
| plugin-jvm               | JVM languages (Java, Kotlin)       |
| plugin-mobile            | Mobile (Android, iOS, Flutter)     |
| plugin-scripting         | Scripting (Python, Ruby, Bash)     |
| plugin-systems           | Systems (Rust, C, C++)             |
| plugin-web               | Web (TypeScript, React, Vue)       |

## Build Pipeline

```
npm install          # Install dependencies
npm run build        # tsup bundles src/ -> dist/
npm test             # vitest runs test suites
```

*MAIDOS-CodeQC ARCHITECTURE v3.0 -- CodeQC Gate C Compliant*
