# MAIDOS-CodeQC -- Configuration Reference

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Configuration      |

## Configuration Files

| File                | Purpose                              |
|---------------------|--------------------------------------|
| `package.json`      | npm dependencies and scripts         |
| `tsconfig.json`     | TypeScript compiler configuration    |
| `tsup.config.ts`    | Bundle configuration (tsup)          |
| `vitest.config.ts`  | Test runner configuration (vitest)   |
| `.codeqc.json`      | QC pipeline configuration (per-project) |

## QC Pipeline Configuration (`.codeqc.json`)

```json
{
  "product": "MyProduct",
  "version": "1.0.0",
  "gates": {
    "g1": { "enabled": true, "docs_path": "docs/" },
    "g2": { "enabled": true, "build_cmd": "npm run build" },
    "g3": { "enabled": true, "test_cmd": "npm test" },
    "g4": { "enabled": true, "proof_dir": "proof/" }
  },
  "plugins": ["systems", "web"]
}
```

## tsup.config.ts Settings

```typescript
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  sourcemap: true,
});
```

## vitest.config.ts Settings

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
    },
  },
});
```

## Environment Variables

| Variable            | Default  | Description                            |
|---------------------|----------|----------------------------------------|
| `CODEQC_CONFIG`    | auto     | Override path to .codeqc.json          |
| `CODEQC_OUTPUT`    | proof/   | Override proof pack output directory   |
| `CODEQC_VERBOSE`   | false    | Enable verbose logging                 |
| `CODEQC_NO_COLOR`  | false    | Disable colored terminal output        |
| `NODE_ENV`         | prod     | Node.js environment mode               |

*MAIDOS-CodeQC CONFIG v3.0 -- CodeQC Gate C Compliant*
