/**
 * Prohibitions (P01-P14) Tests
 */

import { describe, it, expect } from 'vitest';
import { checkProhibitions } from '../../src/rules/b-prohibitions.js';

describe('P05: 超長函數', () => {
  it('should detect function over 50 lines', () => {
    const lines = Array(60).fill('  console.log("line");').join('\n');
    const code = `function longFunction() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P05')).toBe(true);
  });

  it('should allow function under 50 lines', () => {
    const lines = Array(30).fill('  console.log("line");').join('\n');
    const code = `function shortFunction() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'P05')).toHaveLength(0);
  });

  it('should detect long async function', () => {
    const lines = Array(60).fill('  await doSomething();').join('\n');
    const code = `async function longAsync() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P05')).toBe(true);
  });
});

describe('P06: 深層嵌套', () => {
  it('should detect nesting over 3 levels', () => {
    const code = `
function test() {
  if (a) {
    if (b) {
      if (c) {
        if (d) {
          console.log("deep");
        }
      }
    }
  }
}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P06')).toBe(true);
  });

  it('should allow nesting at 3 levels', () => {
    const code = `
function test() {
  if (a) {
    if (b) {
      console.log("ok");
    }
  }
}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'P06')).toHaveLength(0);
  });
});

describe('P09: 無意義命名', () => {
  it('should detect meaningless variable names', () => {
    const code = `const temp = getValue();`;
    const violations = checkProhibitions(code, 'index.ts');
    expect(violations.some(v => v.ruleId === 'P09')).toBe(true);
  });

  it('should detect data as variable name', () => {
    const code = `let data = fetch();`;
    const violations = checkProhibitions(code, 'index.ts');
    expect(violations.some(v => v.ruleId === 'P09')).toBe(true);
  });

  it('should detect foo/bar names', () => {
    const code = `const foo = 1;`;
    const violations = checkProhibitions(code, 'index.ts');
    expect(violations.some(v => v.ruleId === 'P09')).toBe(true);
  });

  it('should allow descriptive names', () => {
    const code = `const userName = getUser();`;
    const violations = checkProhibitions(code, 'index.ts');
    expect(violations.filter(v => v.ruleId === 'P09')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = `const temp = getValue();`;
    const violations = checkProhibitions(code, 'index.test.ts');
    expect(violations.filter(v => v.ruleId === 'P09')).toHaveLength(0);
  });
});

describe('P10: 過長參數', () => {
  it('should detect function with more than 5 parameters', () => {
    const code = `function test(a, b, c, d, e, f) {}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P10')).toBe(true);
  });

  it('should allow function with 5 parameters', () => {
    const code = `function test(a, b, c, d, e) {}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'P10')).toHaveLength(0);
  });

  it('should detect Python function with many parameters', () => {
    const code = `def test(a, b, c, d, e, f, g):`;
    const violations = checkProhibitions(code, 'test.py');
    expect(violations.some(v => v.ruleId === 'P10')).toBe(true);
  });
});

describe('P13: TODO 堆積', () => {
  it('should detect more than 10 TODOs', () => {
    const todos = Array(12).fill('// TODO: fix this').join('\n');
    const violations = checkProhibitions(todos, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P13')).toBe(true);
  });

  it('should allow 10 or fewer TODOs', () => {
    const todos = Array(8).fill('// TODO: fix this').join('\n');
    const violations = checkProhibitions(todos, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'P13')).toHaveLength(0);
  });

  it('should count FIXME as well', () => {
    const mixed = [
      ...Array(6).fill('// TODO: task'),
      ...Array(6).fill('// FIXME: bug'),
    ].join('\n');
    const violations = checkProhibitions(mixed, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P13')).toBe(true);
  });
});
