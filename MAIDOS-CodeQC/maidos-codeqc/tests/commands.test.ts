/**
 * CodeQC v3.3 — CLI command helpers coverage tests
 *
 * These tests focus on:
 * - parsers (test/coverage/audit/Proof Pack)
 * - package.json script auto-detection
 * - pipelineCommand end-to-end (writes evidence/*)
 */

import { describe, it, expect } from 'vitest';
import { mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';

import {
  parseTestOutput,
  parseCoverage,
  parseAudit,
  parseIavLog,
  parseBldsLog,
  parseDatasourceLog,
  detectPackageScripts,
  pipelineCommand,
} from '../src/commands.js';

function makeTempDir(prefix = 'codeqc-test-') {
  const dir = join(tmpdir(), `${prefix}${randomUUID()}`);
  mkdirSync(dir, { recursive: true });
  return dir;
}

describe('commands parsers', () => {
  it('parseTestOutput supports vitest and jest', () => {
    expect(parseTestOutput('Tests  123 passed (123)\\n')).toEqual({ passed: 123, failed: 0 });
    expect(parseTestOutput('Tests: 2 passed, 1 failed\\n')).toEqual({ passed: 2, failed: 1 });
  });

  it('parseCoverage supports vitest + cargo llvm-cov TOTAL', () => {
    const vitestTable = [
      'File               | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s',
      'All files          |   80.00 |    81.00 |   90.00 |   80.00 |',
    ].join('\n');
    expect(parseCoverage(vitestTable)).toBe(80);

    const llvmCov = [
      'Filename Regions Missed Regions Cover',
      'TOTAL    10      2              80.00%',
    ].join('\n');
    expect(parseCoverage(llvmCov)).toBe(80);
  });

  it('parseAudit prefers npm audit --json', () => {
    const out = JSON.stringify({ metadata: { vulnerabilities: { critical: 0, high: 1 } } });
    expect(parseAudit(out)).toEqual({ critical: 0, high: 1 });
  });

  it('parseIav/BLDS/DATASOURCE parsers are strict enough', () => {
    const iav = parseIavLog('Q1 判定: PASS\nQ2 判定: PASS\nQ3 判定: PASS\nQ4 判定: PASS\nQ5 判定: PASS\n');
    expect(iav.passed).toBe(true);

    const blds = parseBldsLog('總分 4/5\n', 3);
    expect(blds.passed).toBe(true);
    expect(blds.minScore).toBe(4);

    const ds = parseDatasourceLog('OK: trace 1\nOK: trace 2\n');
    expect(ds.passed).toBe(true);
    expect(ds.untraced).toBe(0);
  });
});

describe('detectPackageScripts', () => {
  it('detects scripts from package.json', async () => {
    const dir = makeTempDir('codeqc-pkg-');
    writeFileSync(join(dir, 'package.json'), JSON.stringify({
      name: 'x',
      scripts: {
        build: 'tsup',
        lint: 'eslint .',
        test: 'vitest',
        'test:coverage': 'vitest --coverage',
      },
    }, null, 2), 'utf-8');

    const scripts = await detectPackageScripts(dir);
    expect(scripts.build).toContain('npm run');
    expect(scripts.lint).toContain('npm run');
    expect(scripts.test).toContain('vitest');
    expect(scripts.coverage).toContain('coverage');
  });
});

describe('pipelineCommand', () => {
  it('runs end-to-end and writes a Proof Report', async () => {
    const dir = makeTempDir('codeqc-pipeline-');

    // Minimal SPEC + source that satisfies G2 continuity.
    const spec = [
      '# SPEC',
      '- [x] Example function → `foo`',
      '',
    ].join('\n');
    writeFileSync(join(dir, 'SPEC.md'), spec, 'utf-8');

    // Z-axis proof logs (parsed by pipelineCommand).
    mkdirSync(join(dir, 'evidence'), { recursive: true });
    writeFileSync(join(dir, 'evidence', 'iav.log'), 'Q1 判定: PASS\nQ2 判定: PASS\nQ3 判定: PASS\nQ4 判定: PASS\nQ5 判定: PASS\n', 'utf-8');
    writeFileSync(join(dir, 'evidence', 'blds.log'), '總分 5/5\n', 'utf-8');
    writeFileSync(join(dir, 'evidence', 'datasource.log'), 'OK: src/app.ts -> git:deadbeef\n', 'utf-8');

    const loadFiles = (_target: string) => [
      { path: 'src/app.ts', content: 'export function foo(){ return 1; }\\n' },
    ];

    // Use tiny, safe external commands (avoid long toolchains in CI).
    const buildCmd = `node -e "console.log('build ok')"`;
    const lintCmd = `node -e "console.log('lint ok')"`;
    const testCmd = `node -e "console.log('Tests  1 passed (1)')"`;
    const coverageCmd = `node -e "console.log('TOTAL 85.00%')"`;
    const packageCmd = `node -e "console.log('package ok')"`;
    const runCmd = `node -e "console.log('v0.0.0')"`;

    await pipelineCommand(
      [
        dir,
        '--no-auto',
        '--spec', 'SPEC.md',
        '--build-cmd', buildCmd,
        '--lint-cmd', lintCmd,
        '--test-cmd', testCmd,
        '--coverage-cmd', coverageCmd,
        '--package-cmd', packageCmd,
        '--run-cmd', runCmd,
      ],
      loadFiles
    );

    const proofPath = join(dir, 'evidence', 'PROOF-REPORT.md');
    expect(existsSync(proofPath)).toBe(true);
    const proof = readFileSync(proofPath, 'utf-8');
    expect(proof).toContain('# Proof Report v3.3');
    expect(proof).toContain('MISSION COMPLETE');

    // Spot-check that internal evidence logs were emitted.
    expect(existsSync(join(dir, 'evidence', 'g4.log'))).toBe(true);
    expect(existsSync(join(dir, 'evidence', 'mapping.log'))).toBe(true);
    expect(existsSync(join(dir, 'evidence', 'impl.log'))).toBe(true);

    // The pipeline should have written some external logs too.
    expect(existsSync(join(dir, 'evidence', 'build.log'))).toBe(true);
    expect(existsSync(join(dir, 'evidence', 'test.log'))).toBe(true);
    expect(existsSync(join(dir, 'evidence', 'coverage.log'))).toBe(true);
  });
});
