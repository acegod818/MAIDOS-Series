# MAIDOS-CodeQC -- Performance Considerations

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Performance        |

## Performance Targets

| Metric                        | Target          | Notes                      |
|-------------------------------|-----------------|----------------------------|
| G1 Spec Gate execution        | < 2 seconds     | File existence checks only |
| G2 Build Gate execution       | < project build | Depends on target project  |
| G3 Test Gate execution        | < project tests | Depends on target project  |
| G4 Proof Gate execution       | < 10 seconds    | Evidence collection + hash |
| CLI startup time              | < 500 ms        | Cold start, no cache       |
| Web UI initial load           | < 1 second      | Static assets, localhost   |
| Memory usage (pipeline)       | < 256 MB        | Pipeline engine overhead   |

## Optimization Strategies

| Strategy                      | Description                                    |
|-------------------------------|------------------------------------------------|
| Lazy plugin loading           | Plugins loaded only when needed                |
| Parallel file checks (G1)     | Check multiple docs concurrently               |
| Stream processing             | Log files streamed, not buffered entirely      |
| Incremental evidence          | Only new/changed evidence files are hashed     |
| Bundle tree-shaking           | tsup removes unused code from dist             |

## Benchmarks

| Operation                     | Duration (typical) | Notes                 |
|-------------------------------|--------------------|-----------------------|
| Full pipeline (small project) | 15-30 seconds      | Including build+test  |
| Full pipeline (large project) | 2-5 minutes        | Depends on test suite |
| Proof pack generation         | 3-8 seconds        | SHA-256 hashing       |
| Web UI startup                | < 1 second         | Express + static      |
| Plugin detection              | < 200 ms           | File system checks    |

## Timeout Configuration

| Gate | Default Timeout | Configurable |
|------|-----------------|--------------|
| G1   | 30 seconds      | Yes          |
| G2   | 120 seconds     | Yes          |
| G3   | 300 seconds     | Yes          |
| G4   | 60 seconds      | Yes          |

*MAIDOS-CodeQC PERF v3.0 -- CodeQC Gate C Compliant*
