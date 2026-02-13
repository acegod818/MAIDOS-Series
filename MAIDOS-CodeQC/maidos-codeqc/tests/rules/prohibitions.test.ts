/**
 * Prohibitions (P01-P14) Tests
 */

import { describe, it, expect } from 'vitest';
import { checkProhibitions } from '../../src/rules/b-prohibitions.js';

describe('P05: 超長函數', () => {
  it('should detect function over 100 lines', () => {
    const lines = Array(105).fill('  console.log("line");').join('\n');
    const code = `function longFunction() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P05')).toBe(true);
  });

  it('should allow function under 100 lines', () => {
    const lines = Array(90).fill('  console.log("line");').join('\n');
    const code = `function shortFunction() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.filter(v => v.ruleId === 'P05')).toHaveLength(0);
  });

  it('should detect long async function', () => {
    const lines = Array(105).fill('  await doSomething();').join('\n');
    const code = `async function longAsync() {\n${lines}\n}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P05')).toBe(true);
  });
});

describe('P06: 深層嵌套', () => {
  it('should detect nesting over 5 levels', () => {
    const code = `
function test() {
  if (a) {
    if (b) {
      if (c) {
        if (d) {
          if (e) {
            if (f) {
              console.log("deep");
            }
          }
        }
      }
    }
  }
}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P06')).toBe(true);
  });

  it('should allow nesting at 5 levels', () => {
    const code = `
function test() {
  if (a) {
    if (b) {
      if (c) {
        if (d) {
          console.log("ok");
        }
      }
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
  it('should detect function with more than 6 parameters', () => {
    const code = `function test(a, b, c, d, e, f, g) {}`;
    const violations = checkProhibitions(code, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P10')).toBe(true);
  });

  it('should allow function with 6 parameters', () => {
    const code = `function test(a, b, c, d, e, f) {}`;
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
  it('should detect more than 5 TODOs', () => {
    const todos = Array(7).fill('// TODO: fix this').join('\n');
    const violations = checkProhibitions(todos, 'test.ts');
    expect(violations.some(v => v.ruleId === 'P13')).toBe(true);
  });

  it('should allow 5 or fewer TODOs', () => {
    const todos = Array(4).fill('// TODO: fix this').join('\n');
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

// =============================================================================
// P01: 過度工程
// =============================================================================

describe('P01: 過度工程', () => {
  it('should detect multiple Abstract class definitions', () => {
    const code = [
      'abstract class AbstractFactory {}',
      'abstract class AbstractBuilder {}',
      'abstract class AbstractStrategy {}',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/engine.ts');
    expect(violations.some(v => v.ruleId === 'P01')).toBe(true);
  });

  it('should detect excessive design pattern classes', () => {
    const code = [
      'class UserFactory { }',
      'class OrderFactory { }',
      'class PaymentBuilder { }',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/patterns.ts');
    expect(violations.some(v => v.ruleId === 'P01')).toBe(true);
  });

  it('should not flag single Abstract class', () => {
    const code = `abstract class AbstractService {}`;
    const violations = checkProhibitions(code, 'src/base.ts');
    expect(violations.filter(v => v.ruleId === 'P01')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = [
      'abstract class AbstractFactory {}',
      'abstract class AbstractBuilder {}',
      'abstract class AbstractStrategy {}',
    ].join('\n');
    const violations = checkProhibitions(code, 'tests/engine.test.ts');
    expect(violations.filter(v => v.ruleId === 'P01')).toHaveLength(0);
  });
});

// =============================================================================
// P02: 過早優化
// =============================================================================

describe('P02: 過早優化', () => {
  it('should detect excessive bit operations as arithmetic substitutes', () => {
    const code = [
      'const double = x << 1;',
      'const half = y >> 1;',
      'const isOdd = z & 1;',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/math.ts');
    expect(violations.some(v => v.ruleId === 'P02')).toBe(true);
  });

  it('should detect custom data structure implementation', () => {
    const code = `class LinkedList { constructor() { this.head = null; } }`;
    const violations = checkProhibitions(code, 'src/ds.ts');
    expect(violations.some(v => v.ruleId === 'P02')).toBe(true);
  });

  it('should detect memory pool implementation', () => {
    const code = `class ObjectPool { acquire() {} release() {} }`;
    const violations = checkProhibitions(code, 'src/pool.ts');
    expect(violations.some(v => v.ruleId === 'P02')).toBe(true);
  });

  it('should not flag normal arithmetic', () => {
    const code = `const result = x * 2 + y / 2;`;
    const violations = checkProhibitions(code, 'src/calc.ts');
    expect(violations.filter(v => v.ruleId === 'P02')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = `class LinkedList { }`;
    const violations = checkProhibitions(code, 'tests/ds.test.ts');
    expect(violations.filter(v => v.ruleId === 'P02')).toHaveLength(0);
  });
});

// =============================================================================
// P08: 緊耦合
// =============================================================================

describe('P08: 緊耦合', () => {
  it('should detect file with too many imports (>15)', () => {
    const imports = Array(16).fill(0).map((_, i) => `import { mod${i} } from './mod${i}.js';`).join('\n');
    const code = imports + '\nexport function main() {}';
    const violations = checkProhibitions(code, 'src/god-module.ts');
    expect(violations.some(v => v.ruleId === 'P08')).toBe(true);
  });

  it('should detect deep relative path imports', () => {
    const code = [
      `import { a } from '../../../../core/a.js';`,
      `import { b } from '../../../../core/b.js';`,
    ].join('\n');
    const violations = checkProhibitions(code, 'src/deep/nested/file.ts');
    expect(violations.some(v => v.ruleId === 'P08')).toBe(true);
  });

  it('should detect excessive direct service instantiation', () => {
    const code = [
      'const a = new UserService();',
      'const b = new OrderService();',
      'const c = new PaymentService();',
      'const d = new NotificationService();',
      'const e = new LoggingService();',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/controller.ts');
    expect(violations.some(v => v.ruleId === 'P08')).toBe(true);
  });

  it('should not flag normal number of imports', () => {
    const code = [
      `import { a } from './a.js';`,
      `import { b } from './b.js';`,
      `import { c } from './c.js';`,
    ].join('\n');
    const violations = checkProhibitions(code, 'src/main.ts');
    expect(violations.filter(v => v.ruleId === 'P08')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const imports = Array(20).fill(0).map((_, i) => `import { mod${i} } from './mod${i}.js';`).join('\n');
    const violations = checkProhibitions(imports, 'tests/all.test.ts');
    expect(violations.filter(v => v.ruleId === 'P08')).toHaveLength(0);
  });
});

// =============================================================================
// P11: 混合抽象
// =============================================================================

describe('P11: 混合抽象', () => {
  it('should detect SQL + UI mixing in same file', () => {
    const code = [
      'const users = db.query("SELECT * FROM users WHERE active = true");',
      'document.getElementById("user-list").innerHTML = renderList(users);',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/page.ts');
    expect(violations.some(v => v.ruleId === 'P11')).toBe(true);
  });

  it('should detect HTTP + low-level IO mixing', () => {
    const code = [
      'const resp = await fetch("https://api.example.com/data");',
      'fs.writeFileSync("/tmp/data.json", JSON.stringify(resp));',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/sync.ts');
    expect(violations.some(v => v.ruleId === 'P11')).toBe(true);
  });

  it('should not flag properly layered code', () => {
    const code = [
      'export function getUsers() {',
      '  return userRepository.findAll();',
      '}',
    ].join('\n');
    const violations = checkProhibitions(code, 'src/service.ts');
    expect(violations.filter(v => v.ruleId === 'P11')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = [
      'const users = db.query("SELECT * FROM users");',
      'document.getElementById("test").innerHTML = "test";',
    ].join('\n');
    const violations = checkProhibitions(code, 'tests/page.test.ts');
    expect(violations.filter(v => v.ruleId === 'P11')).toHaveLength(0);
  });
});
