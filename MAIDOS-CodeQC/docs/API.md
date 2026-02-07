# MAIDOS-CodeQC -- API Reference

| Field     | Value             |
|-----------|-------------------|
| Product   | MAIDOS-CodeQC     |
| Version   | v3.0              |
| Type      | API Reference     |

## CLI Interface

| Command                              | Description                              |
|--------------------------------------|------------------------------------------|
| `codeqc run <path>`               | Run full QC pipeline on target project   |
| `codeqc run --gate g1 <path>`     | Run only specified gate                  |
| `codeqc status`                   | Show current QC status                   |
| `codeqc report <path>`            | Generate QC report for project           |
| `codeqc dashboard`                | Launch web UI dashboard                  |
| `codeqc init <path>`              | Initialize QC structure in a project     |

## CLI Options

| Option             | Type    | Default | Description                        |
|--------------------|---------|---------|------------------------------------|
| `--gate <id>`     | string  | all     | Run specific gate (g1/g2/g3/g4)   |
| `--config <file>` | string  | auto    | Path to QC configuration file      |
| `--output <dir>`  | string  | proof/  | Output directory for proof pack    |
| `--verbose`       | boolean | false   | Enable verbose logging             |
| `--json`          | boolean | false   | Output results as JSON             |
| `--no-color`      | boolean | false   | Disable colored output             |

## Programmatic API

### Pipeline

```typescript
import { Pipeline } from 'maidos-codeqc';

const pipeline = new Pipeline({
  projectPath: './my-project',
  config: { /* gate configs */ }
});

const result = await pipeline.run();
// result.passed: boolean
// result.gates: GateResult[]
// result.evidence: EvidenceManifest
```

### Gate Interface

```typescript
interface Gate {
  id: string;           // 'g1' | 'g2' | 'g3' | 'g4'
  name: string;         // Human-readable name
  run(ctx: GateContext): Promise<GateResult>;
}

interface GateResult {
  gate: string;
  passed: boolean;
  duration: number;     // milliseconds
  evidence: string[];   // file paths
  errors: string[];
}
```

### Plugin Interface

```typescript
interface CodeQCPlugin {
  name: string;
  languages: string[];
  detectProject(path: string): Promise<boolean>;
  getBuildCommand(): string;
  getTestCommand(): string;
  getArtifactPaths(): string[];
}
```

### Evidence Collector

```typescript
interface EvidenceCollector {
  addLog(gate: string, content: string): void;
  addArtifact(gate: string, filePath: string): void;
  generateManifest(): EvidenceManifest;
}

interface EvidenceManifest {
  product: string;
  version: string;
  timestamp: string;
  gates: Record<string, GateEvidence>;
  hashes: Record<string, string>;  // SHA-256
}
```

## Web UI API

| Endpoint            | Method | Description                     |
|---------------------|--------|---------------------------------|
| `/api/status`      | GET    | Current pipeline status         |
| `/api/history`     | GET    | QC run history                  |
| `/api/report/:id`  | GET    | Specific run report             |
| `/api/run`         | POST   | Trigger a new QC run            |

*MAIDOS-CodeQC API v3.0 -- CodeQC Gate C Compliant*
