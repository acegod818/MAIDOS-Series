# MAIDOS CodeQC

> Code Quality Control implementing Code-QC v3.5 standards

[![npm version](https://badge.fury.io/js/@maidos%2Fcodeqc.svg)](https://badge.fury.io/js/@maidos%2Fcodeqc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🔍 **Static Analysis** - AST-based code analysis using Tree-sitter
- 📏 **Code-QC v3.5 Rules** - Complete implementation of B (工作紀律) + C (驗收標準)
- 🌐 **Multi-Language** - TypeScript, JavaScript, Python, Rust, Go (Phase 0)
- 📊 **Reports** - Console, JSON, HTML output formats
- 🚀 **CI/CD Ready** - GitHub Actions & GitLab CI integration
- 🔌 **Plugin System** - Extensible language support

## Installation

```bash
npm install @maidos/codeqc
# or
pnpm add @maidos/codeqc
```

## Quick Start

### CLI

```bash
# Check current directory
npx maidos-codeqc

# Check specific path with level D (full)
npx maidos-codeqc -l D ./src

# Generate HTML report
npx maidos-codeqc -r html -o report.html ./src

# CI mode (exit 1 on errors)
npx maidos-codeqc --ci ./src
```

### Programmatic API

```typescript
import { analyze, quickCheck, consoleReporter } from '@maidos/codeqc';

// Quick check single file
const violations = quickCheck(sourceCode, 'example.ts');
console.log(violations);

// Full analysis
const result = analyze({
  files: [
    { path: 'src/index.ts', content: sourceCode }
  ],
  level: 'D',
  targetPath: './src'
});

// Generate report
console.log(consoleReporter.report(result));
```

## Check Levels

| Level | Rules | Description |
|:------|:------|:------------|
| **B** | 41 | 工作紀律 (8 公理 + 12 紅線 + 14 禁止 + 7 標記) |
| **C** | ~50 | 驗收標準 (4 關卡 + 雙軸驗證 + A8 完整性) |
| **D** | ~90 | B + C (完整規則集) |

## Rules Overview

### 十二紅線 (R01-R12) - Auto-detected

| ID | Name | Detection |
|:---|:-----|:----------|
| R01 | 硬編碼憑證 | ✅ Regex |
| R05 | 忽略錯誤處理 | ✅ AST |
| R07 | 關閉安全功能 | ✅ Regex |
| R10 | 明文傳輸敏感 | ✅ Regex |
| R12 | 偽造測試結果 | ✅ AST |

### 十四禁止 (P01-P14) - Auto-detected

| ID | Name | Threshold |
|:---|:-----|:----------|
| P05 | 超長函數 | ≤50 行 |
| P06 | 深層嵌套 | ≤3 層 |
| P09 | 無意義命名 | 0 |
| P10 | 過長參數 | ≤5 個 |
| P13 | TODO 堆積 | ≤10 個 |

## Supported Languages

### Phase 0 (Core)

- TypeScript (.ts, .tsx)
- JavaScript (.js, .jsx, .mjs, .cjs)
- Python (.py)
- Rust (.rs)
- Go (.go)

### Phase 1+ (Plugins)

See [ROADMAP.md](./ROADMAP.md) for full language support plan (93 languages total).

## CI/CD Integration

### GitHub Actions

```yaml
name: Code Quality
on: [push, pull_request]

jobs:
  codeqc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm install @maidos/codeqc
      - run: npx maidos-codeqc --ci ./src
```

### GitLab CI

```yaml
codeqc:
  image: node:20
  script:
    - npm install @maidos/codeqc
    - npx maidos-codeqc --ci ./src
```

## API Reference

### analyze(options)

Main analysis function.

```typescript
interface AnalyzeOptions {
  files: Array<{ path: string; content: string }>;
  level: 'B' | 'C' | 'D';
  targetPath: string;
}

const result: AnalysisResult = analyze(options);
```

### quickCheck(source, file)

Quick single-file check with level D.

```typescript
const violations: Violation[] = quickCheck(sourceCode, 'file.ts');
```

### checkRules(source, file, level)

Low-level rule checking.

```typescript
const violations: Violation[] = checkRules(source, file, 'D');
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## License

MIT © MAIDOS

---

Built with ❤️ following Code-QC v2.6 standards
