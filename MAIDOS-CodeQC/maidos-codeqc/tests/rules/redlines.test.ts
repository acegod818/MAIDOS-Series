/**
 * Redlines (R01-R12) Tests
 */

import { describe, it, expect } from 'vitest';
import { checkRedlines } from '../../src/rules/b-redlines.js';

describe('R01: 硬編碼憑證', () => {
  it('should detect hardcoded password', () => {
    const code = `const password = "secret123";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect hardcoded api_key', () => {
    const code = `const api_key = "sk-1234567890abcdef";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect hardcoded token', () => {
    const code = `const token = "ghp_xxxxxxxxxxxxxxxxxxxx";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should allow environment variable reference', () => {
    const code = `const password = process.env.PASSWORD;`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'R01')).toHaveLength(0);
  });

  it('should allow Python env reference', () => {
    const code = `password = os.environ.get("PASSWORD")`;
    const violations = checkRedlines(code, 'test.py');
    expect(violations.filter(v => v.ruleId === 'R01')).toHaveLength(0);
  });

  it('should ignore comments', () => {
    const code = `// const password = "secret123";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'R01')).toHaveLength(0);
  });
});

describe('R05: 忽略錯誤處理', () => {
  it('should detect empty catch in TypeScript', () => {
    const code = `try { foo(); } catch (e) { }`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect empty catch in JavaScript', () => {
    const code = `.catch(() => {})`;
    const violations = checkRedlines(code, 'test.js');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect except pass in Python', () => {
    const code = `except: pass`;
    const violations = checkRedlines(code, 'test.py');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect unwrap in Rust', () => {
    const code = `let value = result.unwrap();`;
    const violations = checkRedlines(code, 'test.rs');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect ignored error in Go', () => {
    const code = `if err != nil { }`;
    const violations = checkRedlines(code, 'test.go');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });
});

describe('R07: 關閉安全功能', () => {
  it('should detect disabled SSL verification', () => {
    const code = `verify_ssl = false`;
    const violations = checkRedlines(code, 'test.py');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });

  it('should detect NODE_TLS_REJECT_UNAUTHORIZED', () => {
    const code = `process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0"`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });

  it('should detect rejectUnauthorized: false', () => {
    const code = `{ rejectUnauthorized: false }`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });

  it('should detect DEBUG=true', () => {
    const code = `DEBUG = True`;
    const violations = checkRedlines(code, 'test.py');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });
});

describe('R10: 明文傳輸敏感', () => {
  it('should detect http:// with sensitive path', () => {
    const code = `const url = "http://api.example.com/auth/login";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R10')).toBe(true);
  });

  it('should detect ftp://', () => {
    const code = `const url = "ftp://server.com/file";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'R10')).toBe(true);
  });

  it('should allow https://', () => {
    const code = `const url = "https://api.example.com/auth";`;
    const violations = checkRedlines(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'R10')).toHaveLength(0);
  });
});

describe('R12: 偽造測試結果', () => {
  it('should detect assert(true)', () => {
    const code = `assert(true)`;
    const violations = checkRedlines(code, 'test.spec.ts');
    expect(violations.some(v => v.ruleId === 'R12')).toBe(true);
  });

  it('should detect expect(true).toBe(true)', () => {
    const code = `expect.toBe(true, true)`;
    const violations = checkRedlines(code, 'test.spec.ts');
    expect(violations.some(v => v.ruleId === 'R12')).toBe(true);
  });

  it('should detect it.skip', () => {
    const code = `it.skip('should work', () => {})`;
    const violations = checkRedlines(code, 'test.spec.ts');
    expect(violations.some(v => v.ruleId === 'R12')).toBe(true);
  });

  it('should detect pytest.skip', () => {
    const code = `pytest.skip("reason")`;
    const violations = checkRedlines(code, 'test_example.py');
    expect(violations.some(v => v.ruleId === 'R12')).toBe(true);
  });

  it('should not flag non-test files', () => {
    const code = `assert(true)`;
    const violations = checkRedlines(code, 'index.ts');
    expect(violations.filter(v => v.ruleId === 'R12')).toHaveLength(0);
  });
});
