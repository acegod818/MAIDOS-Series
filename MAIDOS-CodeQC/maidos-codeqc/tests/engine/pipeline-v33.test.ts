/**
 * Code-QC v3.3 — Pipeline engine coverage tests
 *
 * Goal: exercise the v3.3 ten-step pipeline (pass + key fail branches),
 * so coverage reflects real execution paths (not just rule matchers).
 */

import { describe, it, expect } from 'vitest';
import { runPipeline } from '../../src/engine/pipeline.js';

describe('engine/pipeline v3.3', () => {
  it('passes when all required evidence is provided', () => {
    const files = [
      {
        path: 'src/app.ts',
        content: [
          'export function foo() { return 1; }',
          'export function bar() { return foo() + 1; }',
          'export const VALUE = bar();',
        ].join('\n'),
      },
    ];

    const result = runPipeline({
      targetPath: '.',
      files,
      grade: 'E',
      evidenceDir: 'evidence',
      externalResults: {
        build: { exitCode: 0, log: 'build ok' },
        lint: { exitCode: 0, log: 'lint ok' },
        test: { exitCode: 0, log: 'Tests  1 passed (1)', passed: 1, failed: 0 },
        coverage: { percentage: 85, log: 'All files | 85.00' },
        audit: { exitCode: 0, log: '{"metadata":{"vulnerabilities":{"critical":0,"high":0}}}', critical: 0, high: 0 },
        package: { exitCode: 0, log: 'package ok' },
        run: { exitCode: 0, log: 'v0.0.0' },
      },
      specFunctions: ['foo', 'bar'],
      specChecklist: { total: 2, done: 2 },
      proof: {
        iav: { passed: true, passedCount: 5, failedCount: 0, details: 'IAV PASS (5 points)' },
        blds: { minScore: 5, threshold: 3, passed: true, details: 'BLDS PASS (min=5/5 >= 3)' },
        datasource: { untraced: 0, passed: true, details: 'DATASOURCE PASS (lines=1)' },
      },
      proofContent: {
        iavLog: 'Q1 判定: PASS\nQ2 判定: PASS\nQ3 判定: PASS\nQ4 判定: PASS\nQ5 判定: PASS\n',
        bldsLog: '總分 5/5\n',
        datasourceLog: 'OK: src/app.ts -> git:deadbeef\n',
      },
    });

    expect(result.passed).toBe(true);
    expect(result.steps).toHaveLength(10);
    expect(result.gates.allPassed).toBe(true);
    expect(result.dod.missionComplete).toBe(true);
    expect(result.protectionLevel).toBeGreaterThanOrEqual(result.protectionTarget);
    expect(result.waveform?.allPass).toBe(true);
  });

  it('fails fast on missing SPEC functions list (G2 open circuit)', () => {
    const result = runPipeline({
      targetPath: '.',
      files: [{ path: 'src/app.ts', content: 'export const x = 1;\n' }],
      grade: 'E',
      evidenceDir: 'evidence',
      externalResults: {
        build: { exitCode: 0, log: 'ok' },
        lint: { exitCode: 0, log: 'ok' },
        test: { exitCode: 0, log: 'Tests  1 passed (1)', passed: 1, failed: 0 },
        package: { exitCode: 0, log: 'ok' },
        run: { exitCode: 0, log: 'ok' },
      },
      // specFunctions omitted on purpose
      specChecklist: { total: 1, done: 1 },
      proof: {
        iav: { passed: true, passedCount: 5, failedCount: 0, details: 'IAV PASS' },
        blds: { minScore: 5, threshold: 3, passed: true, details: 'BLDS PASS' },
        datasource: { untraced: 0, passed: true, details: 'DATASOURCE PASS' },
      },
    });

    expect(result.passed).toBe(false);
    expect(result.steps[8]?.passed).toBe(false);
    expect(result.gates.G2.passed).toBe(false);
    expect(result.dod.missionComplete).toBe(false);
    expect(result.waveform?.channels.find(c => c.channel === 'CH1_Y')?.overall).not.toBe('PASS');
  });
});

