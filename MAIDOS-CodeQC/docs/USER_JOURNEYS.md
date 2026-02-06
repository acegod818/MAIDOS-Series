# MAIDOS-CodeQC -- User Journeys

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## J-001: Run Quality Scan

**Actor**: MAIDOS Developer
**Goal**: Scan a project and receive a quality report with gate results.

1. Developer opens a terminal in the project root.
2. Runs `maidos-codeqc scan .` to scan all source files.
3. Engine discovers files, loads applicable plugins based on file types.
4. Rules are applied per gate level (G1 through G4).
5. Console displays a summary: total files, violations by severity, gate pass/fail.
6. Developer reviews violations and decides whether to fix or suppress.

**Success**: Gate results are printed; exit code reflects pass/fail.

---

## J-002: Configure Rules

**Actor**: Tech Lead
**Goal**: Customize which rules are active and adjust severity thresholds.

1. Tech Lead creates or edits `.codeqc.toml` in the project root.
2. Disables rules that do not apply (e.g., `jvm.*` for a Rust-only project).
3. Adjusts gate thresholds: sets G2 max-warnings to 10.
4. Runs `maidos-codeqc config validate` to verify the configuration.
5. Commits the config file to version control.

**Success**: Subsequent scans respect the custom configuration.

---

## J-003: Install Plugin

**Actor**: Plugin Author / Developer
**Goal**: Add a new language plugin to extend scanning capabilities.

1. Downloads the plugin binary (e.g., `codeqc-plugin-mobile.dll`).
2. Places it in `~/.codeqc/plugins/` or the project-local `plugins/` directory.
3. Runs `maidos-codeqc plugins list` to confirm the plugin is detected.
4. Runs a scan; the new plugin processes its target file types.

**Success**: Plugin appears in the list and contributes rules to scans.

---

## J-004: View Report

**Actor**: Tech Lead
**Goal**: Generate and review a detailed HTML quality report.

1. Runs `maidos-codeqc scan . --format html --output report.html`.
2. Opens `report.html` in a browser.
3. Reviews the dashboard: file tree, violation heatmap, gate status.
4. Drills into individual files to see inline violation annotations.
5. Exports the report URL to share with the team.

**Success**: Report is generated and viewable with all violations detailed.

---

## J-005: Fix Violations

**Actor**: Developer
**Goal**: Resolve reported violations and achieve a passing gate.

1. Runs a scan and identifies G1 violations (e.g., fake implementations).
2. Opens the flagged file at the reported line number.
3. Replaces the stub (`return true`) with a real implementation.
4. Re-runs the scan to confirm the violation is resolved.
5. Repeats until the target gate passes.

**Success**: Gate status changes from FAIL to PASS.

---

## J-006: CI/CD Integration

**Actor**: DevOps Engineer
**Goal**: Add CodeQC as a mandatory check in the CI pipeline.

1. Adds a step to `github-actions.yml`:
   `maidos-codeqc scan . --gate G2 --format json --output qc-report.json`
2. Configures the step to fail the build if exit code is non-zero.
3. Archives `qc-report.json` as a build artifact.
4. On pull requests, the pipeline blocks merge if G2 fails.

**Success**: PRs cannot merge without passing the configured quality gate.

---

*Each journey maps to acceptance criteria in AC_MATRIX.md.*
