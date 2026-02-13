/**
 * Redlines R04/R06/R11 Tests
 *
 * R04 未授權數據訪問 | R06 直接操作生產 | R11 跳過代碼審查
 */

import { describe, it, expect } from 'vitest';
import { checkRedlines } from '../../src/rules/b-redlines.js';

// =============================================================================
// R04: 未授權數據訪問
// =============================================================================

describe('R04: 未授權數據訪問', () => {
  it('should detect SQL query on sensitive table (passwords)', () => {
    const code = `db.query("SELECT * FROM passwords WHERE user_id = ?")`;
    const violations = checkRedlines(code, 'src/repo.ts');
    expect(violations.some(v => v.ruleId === 'R04')).toBe(true);
  });

  it('should detect reading /etc/passwd', () => {
    const code = `const data = fs.readFileSync("/etc/passwd", "utf-8")`;
    const violations = checkRedlines(code, 'src/util.ts');
    expect(violations.some(v => v.ruleId === 'R04')).toBe(true);
  });

  it('should detect environment variable dump', () => {
    const code = `const allEnv = Object.keys(process.env)`;
    const violations = checkRedlines(code, 'src/config.ts');
    expect(violations.some(v => v.ruleId === 'R04')).toBe(true);
  });

  it('should detect chmod 777', () => {
    const code = `exec("chmod 777 /var/data")`;
    const violations = checkRedlines(code, 'src/deploy.sh');
    expect(violations.some(v => v.ruleId === 'R04')).toBe(true);
  });

  it('should not flag normal env access', () => {
    const code = `const port = process.env.PORT`;
    const violations = checkRedlines(code, 'src/config.ts');
    expect(violations.filter(v => v.ruleId === 'R04')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = `db.query("SELECT * FROM passwords WHERE user_id = 1")`;
    const violations = checkRedlines(code, 'tests/repo.test.ts');
    expect(violations.filter(v => v.ruleId === 'R04')).toHaveLength(0);
  });
});

// =============================================================================
// R06: 直接操作生產
// =============================================================================

describe('R06: 直接操作生產', () => {
  it('should detect DROP TABLE', () => {
    const code = `db.execute("DROP TABLE users")`;
    const violations = checkRedlines(code, 'src/cleanup.ts');
    expect(violations.some(v => v.ruleId === 'R06')).toBe(true);
  });

  it('should detect TRUNCATE TABLE', () => {
    const code = `TRUNCATE TABLE sessions`;
    const violations = checkRedlines(code, 'src/admin.sql');
    expect(violations.some(v => v.ruleId === 'R06')).toBe(true);
  });

  it('should detect production DB connection string', () => {
    const code = `const db = connect("postgres://admin:pass@db.production.myapp.com:5432/main")`;
    const violations = checkRedlines(code, 'src/db.ts');
    expect(violations.some(v => v.ruleId === 'R06')).toBe(true);
  });

  it('should detect kubectl delete in prod', () => {
    const code = `kubectl delete pod my-app --context production`;
    const violations = checkRedlines(code, 'scripts/deploy.sh');
    expect(violations.some(v => v.ruleId === 'R06')).toBe(true);
  });

  it('should not flag DDL in migration files', () => {
    const code = `DROP TABLE old_users`;
    const violations = checkRedlines(code, 'db/migration_001.sql');
    expect(violations.filter(v => v.ruleId === 'R06')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = `DROP TABLE test_data`;
    const violations = checkRedlines(code, 'tests/cleanup.test.ts');
    expect(violations.filter(v => v.ruleId === 'R06')).toHaveLength(0);
  });
});

// =============================================================================
// R11: 跳過代碼審查
// =============================================================================

describe('R11: 跳過代碼審查', () => {
  it('should detect git push --force', () => {
    const code = `exec("git push origin main --force")`;
    const violations = checkRedlines(code, 'scripts/deploy.sh');
    expect(violations.some(v => v.ruleId === 'R11')).toBe(true);
  });

  it('should detect [skip ci]', () => {
    const code = `commitMessage = "fix: quick patch [skip ci]"`;
    const violations = checkRedlines(code, 'src/release.ts');
    expect(violations.some(v => v.ruleId === 'R11')).toBe(true);
  });

  it('should detect --no-verify', () => {
    const code = `git commit --no-verify -m "hotfix"`;
    const violations = checkRedlines(code, 'scripts/hotfix.sh');
    expect(violations.some(v => v.ruleId === 'R11')).toBe(true);
  });

  it('should detect HUSKY=0', () => {
    const code = `HUSKY=0 git push`;
    const violations = checkRedlines(code, 'scripts/push.sh');
    expect(violations.some(v => v.ruleId === 'R11')).toBe(true);
  });

  it('should not flag normal git operations', () => {
    const code = `git push origin feature-branch`;
    const violations = checkRedlines(code, 'scripts/push.sh');
    expect(violations.filter(v => v.ruleId === 'R11')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = `git push origin main --force`;
    const violations = checkRedlines(code, 'tests/git.test.ts');
    expect(violations.filter(v => v.ruleId === 'R11')).toHaveLength(0);
  });
});
