# MAIDOS-CodeQC -- Build Instructions

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Build Guide        |

## Prerequisites

| Requirement   | Version    | Purpose                     |
|---------------|------------|-----------------------------|
| Node.js       | >= 20.x    | JavaScript runtime          |
| npm           | >= 10.x    | Package manager             |
| TypeScript    | >= 5.x     | Language (dev dependency)   |
| tsup          | latest     | Bundler (dev dependency)    |
| vitest        | latest     | Test runner (dev dependency) |

## Build Steps

### 1. Install Dependencies

```bash
npm install
```

### 2. Build the Project

```bash
npm run build
```

This runs tsup to bundle the TypeScript source into distributable JavaScript.

### 3. Run Tests

```bash
npm test
```

### 4. Type Check (Optional)

```bash
npx tsc --noEmit
```

## Build Outputs

| Artifact            | Path                  | Description                |
|---------------------|-----------------------|----------------------------|
| CLI bundle          | `dist/index.js`      | Main CLI entry point       |
| Type declarations   | `dist/index.d.ts`    | TypeScript declarations    |
| Plugin bundles      | `dist/plugins/`      | Per-plugin bundles         |
| Web UI assets       | `web-ui/dist/`       | Dashboard static files     |

## Environment Variables

| Variable            | Default | Description                         |
|---------------------|---------|-------------------------------------|
| `CODEQC_CONFIG`    | auto    | Path to configuration file          |
| `CODEQC_OUTPUT`    | proof/  | Output directory for proof packs    |
| `CODEQC_VERBOSE`   | false   | Enable verbose logging              |
| `NODE_ENV`         | prod    | Node environment mode               |

## Clean Build

```bash
rm -rf dist/ node_modules/
npm install
npm run build
```

*MAIDOS-CodeQC BUILD v3.0 -- CodeQC Gate C Compliant*
