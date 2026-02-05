/**
 * Analyzer Tests
 */

import { describe, it, expect } from 'vitest';
import { analyze, analyzeFile, quickCheck, detectLanguage, isSupported } from '../src/analyzer.js';

describe('Language Detection', () => {
  it('should detect TypeScript', () => {
    expect(detectLanguage('file.ts')).toBe('typescript');
    expect(detectLanguage('file.tsx')).toBe('typescript');
  });

  it('should detect JavaScript', () => {
    expect(detectLanguage('file.js')).toBe('javascript');
    expect(detectLanguage('file.jsx')).toBe('javascript');
    expect(detectLanguage('file.mjs')).toBe('javascript');
    expect(detectLanguage('file.cjs')).toBe('javascript');
  });

  it('should detect Python', () => {
    expect(detectLanguage('file.py')).toBe('python');
  });

  it('should detect Rust', () => {
    expect(detectLanguage('file.rs')).toBe('rust');
  });

  it('should detect Go', () => {
    expect(detectLanguage('file.go')).toBe('go');
  });

  it('should detect non-core languages (43-language support)', () => {
    expect(detectLanguage('file.java')).toBe('java');
    expect(detectLanguage('file.kt')).toBe('kotlin');
    expect(detectLanguage('file.cs')).toBe('csharp');
    expect(detectLanguage('file.fsx')).toBe('fsharp');
    expect(detectLanguage('file.vb')).toBe('vbnet');
    expect(detectLanguage('file.swift')).toBe('swift');
    expect(detectLanguage('file.mm')).toBe('objc');
    expect(detectLanguage('file.cpp')).toBe('cpp');
    expect(detectLanguage('file.zig')).toBe('zig');
    expect(detectLanguage('file.nim')).toBe('nim');
    expect(detectLanguage('file.php')).toBe('php');
    expect(detectLanguage('file.rb')).toBe('ruby');
    expect(detectLanguage('file.ps1')).toBe('powershell');
    expect(detectLanguage('file.sql')).toBe('sql');
    expect(detectLanguage('file.r')).toBe('r');
    expect(detectLanguage('file.jl')).toBe('julia');
    expect(detectLanguage('file.yml')).toBe('yaml');
    expect(detectLanguage('file.toml')).toBe('toml');
    expect(detectLanguage('file.xml')).toBe('xml');
    expect(detectLanguage('file.ex')).toBe('elixir');
    expect(detectLanguage('file.hs')).toBe('haskell');
    expect(detectLanguage('file.ml')).toBe('ocaml');
    expect(detectLanguage('file.erl')).toBe('erlang');
    expect(detectLanguage('file.cob')).toBe('cobol');
    expect(detectLanguage('file.abap')).toBe('abap');
    expect(detectLanguage('file.pks')).toBe('plsql');
    expect(detectLanguage('file.f90')).toBe('fortran');
    expect(detectLanguage('file.bas')).toBe('vba');
    expect(detectLanguage('file.rpgle')).toBe('rpg');
  });

  it('should return null for unsupported', () => {
    expect(detectLanguage('file.txt')).toBe(null);
    expect(detectLanguage('file.unknownext')).toBe(null);
  });
});

describe('isSupported', () => {
  it('should return true for supported languages', () => {
    expect(isSupported('file.ts')).toBe(true);
    expect(isSupported('file.py')).toBe(true);
    expect(isSupported('file.rs')).toBe(true);
    expect(isSupported('file.java')).toBe(true);
    expect(isSupported('file.rb')).toBe(true);
  });

  it('should return false for unsupported languages', () => {
    expect(isSupported('file.txt')).toBe(false);
    expect(isSupported('file.md')).toBe(false);
  });
});

describe('analyzeFile', () => {
  it('should return violations for problematic code', () => {
    const code = `const password = "secret123";`;
    const result = analyzeFile(code, 'config.ts', 'D');
    
    expect(result.file).toBe('config.ts');
    expect(result.language).toBe('typescript');
    expect(result.violations.length).toBeGreaterThan(0);
    expect(result.violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should return empty violations for clean code', () => {
    const code = `
export function greet(name: string): string {
  return \`Hello, \${name}!\`;
}
`;
    const result = analyzeFile(code, 'greet.ts', 'D');
    expect(result.violations).toHaveLength(0);
  });

  it('should count lines correctly', () => {
    const code = `
// Comment
function foo() {
  return 1;
}

/* Block
   comment */
`;
    const result = analyzeFile(code, 'test.ts', 'D');
    
    expect(result.stats.totalLines).toBe(9);
    expect(result.stats.blankLines).toBeGreaterThan(0);
    expect(result.stats.commentLines).toBeGreaterThan(0);
    expect(result.stats.codeLines).toBeGreaterThan(0);
  });
});

describe('quickCheck', () => {
  it('should return violations array', () => {
    const code = `const api_key = "sk-12345678";`;
    const violations = quickCheck(code, 'config.ts');
    
    expect(Array.isArray(violations)).toBe(true);
    expect(violations.length).toBeGreaterThan(0);
  });

  it('should use level D by default', () => {
    // Level D includes both redlines and prohibitions
    const code = `
const temp = getValue();
const password = "secret";
`;
    const violations = quickCheck(code, 'index.ts');
    
    // Should catch both R01 (password) and P09 (temp)
    const ruleIds = violations.map(v => v.ruleId);
    expect(ruleIds).toContain('R01');
    expect(ruleIds).toContain('P09');
  });
});

describe('analyze (batch)', () => {
  it('should analyze multiple files', () => {
    const result = analyze({
      files: [
        { path: 'src/a.ts', content: 'const a = 1;' },
        { path: 'src/b.ts', content: 'const b = 2;' },
      ],
      level: 'D',
      targetPath: './src',
    });
    
    expect(result.files).toHaveLength(2);
    expect(result.summary.totalFiles).toBe(2);
    expect(result.targetPath).toBe('./src');
    expect(result.level).toBe('D');
  });

  it('should aggregate violations correctly', () => {
    const result = analyze({
      files: [
        { path: 'a.ts', content: `const password = "secret";` },
        { path: 'b.ts', content: `const api_key = "key123456";` },
      ],
      level: 'D',
      targetPath: '.',
    });
    
    expect(result.summary.errorCount).toBeGreaterThan(0);
    expect(result.summary.byRule['R01']).toBe(2);
  });

  it('should track duration', () => {
    const result = analyze({
      files: [{ path: 'a.ts', content: 'const x = 1;' }],
      level: 'D',
      targetPath: '.',
    });
    
    expect(result.duration).toBeGreaterThanOrEqual(0);
    expect(result.timestamp).toBeTruthy();
  });
});
