/**
 * Reporter Tests
 */

import { describe, it, expect } from 'vitest';
import { consoleReporter, jsonReporter, htmlReporter, getReporter } from '../src/reporter/index.js';
import type { AnalysisResult } from '../src/types.js';

const mockResult: AnalysisResult = {
  timestamp: '2026-01-26T12:00:00.000Z',
  targetPath: './src',
  level: 'D',
  files: [
    {
      file: 'src/config.ts',
      language: 'typescript',
      violations: [
        {
          ruleId: 'R01',
          ruleName: '硬編碼憑證',
          severity: 'error',
          file: 'src/config.ts',
          line: 5,
          column: 7,
          message: '檢測到可能的硬編碼憑證',
          snippet: 'const password = "secret";',
          suggestion: '使用環境變數',
        },
      ],
      duration: 10,
      stats: {
        totalLines: 20,
        codeLines: 15,
        commentLines: 3,
        blankLines: 2,
      },
    },
  ],
  summary: {
    totalFiles: 1,
    totalViolations: 1,
    errorCount: 1,
    warningCount: 0,
    infoCount: 0,
    byRule: { R01: 1 } as any,
  },
  duration: 50,
};

describe('Console Reporter', () => {
  it('should generate console output', () => {
    const output = consoleReporter.report(mockResult);
    
    expect(output).toContain('MAIDOS CodeQC');
    expect(output).toContain('R01');
    expect(output).toContain('src/config.ts');
    expect(output).toContain('FAIL');
  });

  it('should show PASS when no errors', () => {
    const cleanResult: AnalysisResult = {
      ...mockResult,
      files: [],
      summary: {
        totalFiles: 1,
        totalViolations: 0,
        errorCount: 0,
        warningCount: 0,
        infoCount: 0,
        byRule: {} as any,
      },
    };
    
    const output = consoleReporter.report(cleanResult);
    expect(output).toContain('PASS');
  });
});

describe('JSON Reporter', () => {
  it('should generate valid JSON', () => {
    const output = jsonReporter.report(mockResult);
    const parsed = JSON.parse(output);
    
    expect(parsed.targetPath).toBe('./src');
    expect(parsed.level).toBe('D');
    expect(parsed.files).toHaveLength(1);
    expect(parsed.summary.errorCount).toBe(1);
  });

  it('should include all fields', () => {
    const output = jsonReporter.report(mockResult);
    const parsed = JSON.parse(output);
    
    expect(parsed).toHaveProperty('timestamp');
    expect(parsed).toHaveProperty('files');
    expect(parsed).toHaveProperty('summary');
    expect(parsed).toHaveProperty('duration');
  });
});

describe('HTML Reporter', () => {
  it('should generate valid HTML', () => {
    const output = htmlReporter.report(mockResult);
    
    expect(output).toContain('<!DOCTYPE html>');
    expect(output).toContain('<html');
    expect(output).toContain('</html>');
  });

  it('should include violations', () => {
    const output = htmlReporter.report(mockResult);
    
    expect(output).toContain('R01');
    expect(output).toContain('硬編碼憑證');
    expect(output).toContain('src/config.ts');
  });

  it('should escape HTML entities', () => {
    const resultWithHtml: AnalysisResult = {
      ...mockResult,
      targetPath: '<script>alert("xss")</script>',
    };
    
    const output = htmlReporter.report(resultWithHtml);
    
    expect(output).not.toContain('<script>alert');
    expect(output).toContain('&lt;script&gt;');
  });

  it('should show FAIL badge when errors exist', () => {
    const output = htmlReporter.report(mockResult);
    expect(output).toContain('FAIL');
  });

  it('should show PASS badge when no errors', () => {
    const cleanResult: AnalysisResult = {
      ...mockResult,
      files: [],
      summary: {
        totalFiles: 1,
        totalViolations: 0,
        errorCount: 0,
        warningCount: 0,
        infoCount: 0,
        byRule: {} as any,
      },
    };
    
    const output = htmlReporter.report(cleanResult);
    expect(output).toContain('PASS');
  });
});

describe('getReporter', () => {
  it('should return console reporter', () => {
    const reporter = getReporter('console');
    expect(reporter.name).toBe('console');
  });

  it('should return json reporter', () => {
    const reporter = getReporter('json');
    expect(reporter.name).toBe('json');
  });

  it('should return html reporter', () => {
    const reporter = getReporter('html');
    expect(reporter.name).toBe('html');
  });

  it('should throw for unknown reporter', () => {
    expect(() => getReporter('unknown')).toThrow('Unknown reporter');
  });
});
