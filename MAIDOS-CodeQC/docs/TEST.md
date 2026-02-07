# MAIDOS-CodeQC -- Test Documentation

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Test Guide         |

## Test Framework

| Component      | Technology       |
|----------------|------------------|
| Test Runner    | vitest           |
| Config         | vitest.config.ts |
| Assertion      | vitest built-in (expect) |
| Mocking        | vitest built-in (vi) |

## Test Categories

| Category      | Location                   | Description                          |
|---------------|----------------------------|--------------------------------------|
| Unit          | `src/**/*.test.ts`         | Individual module tests              |
| Integration   | `tests/integration/`       | Cross-module interaction tests       |
| E2E           | `tests/e2e/`               | Full pipeline execution tests        |

## Running Tests

### All Tests

```bash
npm test
```

### Unit Tests Only

```bash
npx vitest run src/
```

### Integration Tests Only

```bash
npx vitest run tests/integration/
```

### E2E Tests Only

```bash
npx vitest run tests/e2e/
```

### With Coverage

```bash
npx vitest run --coverage
```

## Key Test Suites

| Suite                         | Tests | Description                         |
|-------------------------------|-------|-------------------------------------|
| Pipeline Engine               | 12    | Gate orchestration logic            |
| G1 Spec Gate                  | 8     | Document validation checks          |
| G2 Build Gate                 | 6     | Build execution verification        |
| G3 Test Gate                  | 7     | Test suite execution verification   |
| G4 Proof Gate                 | 9     | Evidence pack generation            |
| Plugin System                 | 10    | Plugin loading and detection        |
| Evidence Collector            | 6     | Artifact collection and hashing     |
| Config Loader                 | 5     | Configuration parsing               |
| Reporter                      | 4     | Output formatting                   |
| Web UI API                    | 8     | Dashboard API endpoints             |

## Test Conventions

- Test files are co-located with source: `module.ts` -> `module.test.ts`
- Integration tests use fixture projects in `tests/fixtures/`
- E2E tests run the full CLI pipeline against sample projects
- All tests must be deterministic and not depend on external services

*MAIDOS-CodeQC TEST v3.0 -- CodeQC Gate C Compliant*
