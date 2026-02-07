# MAIDOS-CodeQC -- Usage Guide

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Usage Guide        |

## Quick Start

### 1. Initialize QC Structure

```bash
codeqc init ./my-project
```

Creates the required `docs/` and `qc/` directories with template files.

### 2. Run Full QC Pipeline

```bash
codeqc run ./my-project
```

### 3. View Results

```bash
codeqc report ./my-project
```

Or launch the web dashboard:

```bash
codeqc dashboard
```

## Common Workflows

### Run a Single Gate

```bash
codeqc run --gate g1 ./my-project    # Spec check only
codeqc run --gate g2 ./my-project    # Build only
codeqc run --gate g3 ./my-project    # Test only
codeqc run --gate g4 ./my-project    # Proof pack only
```

### JSON Output for CI/CD

```bash
codeqc run --json ./my-project > qc-result.json
```

## Gate Pass/Fail Criteria

| Gate | Pass Condition                                          |
|------|---------------------------------------------------------|
| G1   | All required docs exist and contain proper structure    |
| G2   | Build command exits with code 0, artifacts present      |
| G3   | All test suites pass with 0 failures                    |
| G4   | Proof pack generated with complete manifest             |

## Pipeline Output

```
proof/
  manifest.json        # QC manifest with timestamps and hashes
  g1-spec.log          # G1 gate evidence
  g2-build.log         # G2 gate evidence
  g3-test.log          # G3 gate evidence
  g4-proof.log         # G4 gate evidence
  artifacts/           # Collected build and test artifacts
```

## Web UI Dashboard

| Feature              | Description                               |
|----------------------|-------------------------------------------|
| Status Overview      | Current pass/fail status for all products |
| Run History          | Historical QC run results                 |
| Gate Details         | Per-gate evidence and logs                |
| Product Comparison   | Side-by-side QC status for all products   |

Access at `http://localhost:3000` after running `codeqc dashboard`.

*MAIDOS-CodeQC USAGE v3.0 -- CodeQC Gate C Compliant*
