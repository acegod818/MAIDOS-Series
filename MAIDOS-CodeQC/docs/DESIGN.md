# MAIDOS-CodeQC -- Design Document

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Design Decisions   |

## Design Principles

| Principle            | Description                                              |
|----------------------|----------------------------------------------------------|
| Gate Isolation       | Each gate is independent; failure stops the pipeline     |
| Plugin Extensibility | New languages supported via plugin interface             |
| Evidence First       | Every gate action produces verifiable evidence           |
| Fail Fast            | Pipeline halts at first gate failure                     |
| Reproducible         | Same inputs always produce same QC results               |

## Gate Design

### G1 -- Spec Gate

| Check                  | Criteria                                    |
|------------------------|---------------------------------------------|
| docs/ directory        | Must exist with required files              |
| SPEC.md                | Must contain product metadata table         |
| ARCHITECTURE.md        | Must describe module structure              |
| qc/ directory          | Must contain build, unit, proof scripts     |

### G2 -- Build Gate

| Check                  | Criteria                                    |
|------------------------|---------------------------------------------|
| Build command          | Must exit with code 0                       |
| Output artifacts       | Expected files must exist after build       |
| No warnings policy     | Configurable: treat warnings as errors      |

### G3 -- Test Gate

| Check                  | Criteria                                    |
|------------------------|---------------------------------------------|
| Unit tests             | All must pass                               |
| Integration tests      | All must pass                               |
| E2E tests              | All must pass                               |
| Coverage threshold     | Configurable minimum coverage percentage    |

### G4 -- Proof Gate

| Check                  | Criteria                                    |
|------------------------|---------------------------------------------|
| Manifest generation    | JSON manifest with timestamps               |
| Evidence collection    | All gate logs collected                     |
| Hash verification      | SHA-256 hashes for all evidence files       |
| Pack completeness      | All expected artifacts present              |

## Configuration Schema

```json
{
  "product": "string",
  "version": "string",
  "gates": {
    "g1": { "enabled": true, "docs_path": "docs/" },
    "g2": { "enabled": true, "build_cmd": "npm run build" },
    "g3": { "enabled": true, "test_cmd": "npm test" },
    "g4": { "enabled": true, "proof_dir": "proof/" }
  },
  "plugins": ["systems", "web", "dotnet"]
}
```

## Error Handling

| Scenario               | Behavior                                     |
|------------------------|----------------------------------------------|
| Gate failure           | Pipeline halts, evidence saved, exit code 1  |
| Missing docs           | G1 fails with detailed missing-file list     |
| Build failure          | G2 captures stderr, saves build log          |
| Test failure           | G3 captures test output, saves report        |
| Plugin not found       | Warning logged, gate continues if possible   |

*MAIDOS-CodeQC DESIGN v3.0 -- CodeQC Gate C Compliant*
