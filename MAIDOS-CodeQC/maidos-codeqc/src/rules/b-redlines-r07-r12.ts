import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';

// =============================================================================
// R06: 直接操作生產
// 手法: 直連 production DB、SSH prod、DROP TABLE、kubectl delete prod
// =============================================================================

const DIRECT_PROD_PATTERNS = [
  // Production DB 連線字串
  /['"`](?:mysql|postgres|mongodb(?:\+srv)?|redis):\/\/[^'"`]*(?:prod|production)[^'"`]*['"`]/gi,
  // prod RDS/database host
  /['"`][^'"`]*\.(?:prod|production)\..*\.(?:rds|database|cluster)\.[^'"`]*['"`]/gi,
  // 危險 DDL (非 migration/seed/schema 檔)
  /\b(?:DROP\s+TABLE|TRUNCATE\s+TABLE)\b/gi,
  // kubectl 直接操作 prod
  /kubectl\s+(?:delete|exec|scale|rollout|apply).*(?:prod|production)/gi,
  // SSH/SCP 到 prod
  /\b(?:ssh|scp)\s+.*(?:prod|production)/gi,
];

const PROD_FILE_WHITELIST = /(?:migration|migrate|seed|schema|fixture|test|spec|mock)/i;

export const R06_CHECKER: RuleChecker = {
  rule: getRedline('R06')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture|__test__)/i.test(file)) return violations;
    // 排除 migration/seed 檔 (DDL 合法使用)
    const isDDLWhitelisted = PROD_FILE_WHITELIST.test(file);
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of DIRECT_PROD_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          // DDL 在 migration 檔中不報
          if (isDDLWhitelisted && /DROP\s+TABLE|TRUNCATE/i.test(match[0])) continue;
          violations.push({ ruleId: 'R06', ruleName: '直接操作生產', severity: 'error', file, line: i + 1, column: match.index + 1, message: `直接操作生產 (R06): ${match[0].substring(0, 40)}`, snippet: line.trim(), suggestion: '使用 CI/CD pipeline 部署，禁止直接操作生產環境' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R07: 關閉安全功能
// =============================================================================

const SECURITY_DISABLE_PATTERNS = [
  /(?:verify|ssl|tls)[_-]?(?:ssl|verify|cert)\s*[=:]\s*(?:false|False|FALSE|0)/gi,
  /NODE_TLS_REJECT_UNAUTHORIZED\s*=\s*['"]?0/gi,
  /rejectUnauthorized\s*:\s*false/gi,
  /Access-Control-Allow-Origin['"]\s*:\s*['"]\*/gi,
  /Content-Security-Policy['"]\s*:\s*['"]?none/gi,
  /DEBUG\s*=\s*(?:true|True|TRUE|1)/gi,
];

export const R07_CHECKER: RuleChecker = {
  rule: getRedline('R07')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of SECURITY_DISABLE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', file, line: i + 1, column: match.index + 1, message: `關閉安全功能: ${match[0]}`, snippet: line.trim(), suggestion: '確保生產啟用安全功能' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R08: 使用已知漏洞
// =============================================================================

const VULNERABLE_PATTERNS = [
  /\bgets\s*\(/g,
  /\bstrcpy\s*\(/g,
  /\bsprintf\s*\(/g,
  /\bstrcat\s*\(/g,
  /log4j.*2\.(?:0|1[0-4])\./gi,
  /(?:MD5|SHA1)\s*\(/gi,
  /\.createHash\s*\(\s*['"](?:md5|sha1)['"]\s*\)/gi,
  /\bDES\b|\bRC4\b|\b3DES\b/g,  // 完整單詞匹配
];

export const R08_CHECKER: RuleChecker = {
  rule: getRedline('R08')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of VULNERABLE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', file, line: i + 1, column: match.index + 1, message: `已知漏洞: ${match[0]}`, snippet: line.trim(), suggestion: '升級或使用安全替代' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R09: 無限制資源
// =============================================================================

const UNLIMITED_RESOURCE_PATTERNS = [
  /while\s*\(\s*true\s*\)/gi,
  /while\s*\(\s*1\s*\)/gi,
  /for\s*\(\s*;\s*;\s*\)/gi,
  /loop\s*\{/gi,
  /SELECT\s+\*\s+FROM(?!.*LIMIT)/gi,
  /\.find\s*\(\s*\{\s*\}\s*\)(?!.*limit)/gi,
  /rate[_-]?limit\s*[=:]\s*(?:0|false|none|null|nil)/gi,
];

export const R09_CHECKER: RuleChecker = {
  rule: getRedline('R09')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of UNLIMITED_RESOURCE_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R09', ruleName: '無限制資源', severity: 'error', file, line: i + 1, column: match.index + 1, message: `無限制資源: ${match[0]}`, snippet: line.trim(), suggestion: '添加限制/超時' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R10: 明文傳輸敏感
// =============================================================================

const PLAINTEXT_PATTERNS = [
  /['"`]http:\/\/[^'"`]+(?:login|auth|password|token|secret|api|user)[^'"`]*['"`]/gi,
  /['"`]ftp:\/\//gi,
  /['"`]telnet:\/\//gi,
];

export const R10_CHECKER: RuleChecker = {
  rule: getRedline('R10')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];

    // Avoid false positives in test projects/fixtures (e.g., InlineData("ftp://...") examples).
    // Note: keep it directory-based, so unit tests that pass file="test.ts" still validate behavior.
    if (/(?:^|[\\/])tests?(?:$|[\\/])/i.test(file) || /(?:^|[\\/])[^\\/]*\.tests(?:$|[\\/])/i.test(file)) {
      return violations;
    }

    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of PLAINTEXT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R10', ruleName: '明文傳輸敏感', severity: 'error', file, line: i + 1, column: match.index + 1, message: '明文傳輸', snippet: line.trim(), suggestion: '使用 HTTPS/TLS' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R11: 跳過代碼審查
// 手法: git push --force, 直接 push main, [skip ci], --no-verify
// =============================================================================

const SKIP_REVIEW_PATTERNS = [
  // Force push
  /git\s+push\s+.*(?:--force\b|-f\b)/gi,
  // 直接 push 到 main/master
  /git\s+push\s+(?:origin|upstream)\s+(?:main|master)\b/gi,
  // 跳過 CI
  /\[skip\s+ci\]|\[ci\s+skip\]/gi,
  // 跳過 git hooks
  /--no-verify\b/gi,
  /HUSKY\s*=\s*0/gi,
  // 繞過 review 配置
  /bypass[_-]?(?:review|approval|check)/gi,
  /required[_-]?reviews?\s*[:=]\s*0/gi,
];

export const R11_CHECKER: RuleChecker = {
  rule: getRedline('R11')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture|__test__)/i.test(file)) return violations;
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of SKIP_REVIEW_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R11', ruleName: '跳過代碼審查', severity: 'error', file, line: i + 1, column: match.index + 1, message: `跳過代碼審查 (R11): ${match[0].substring(0, 40)}`, snippet: line.trim(), suggestion: '遵循 code review 流程，不繞過審查機制' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R12: 偽造測試結果
// =============================================================================

const FAKE_TEST_PATTERNS = [
  /(?:assert|expect|should)\s*\(\s*true\s*\)/gi,
  /(?:assert|expect)\s*\.\s*(?:equal|toBe)\s*\(\s*true\s*,\s*true\s*\)/gi,
  /(?:it|test|describe)\s*\.\s*skip\s*\(/gi,
  /(?:@skip|@ignore|@disabled)/gi,
  /pytest\.skip\s*\(/gi,
  /@pytest\.mark\.skip/gi,
];

export const R12_CHECKER: RuleChecker = {
  rule: getRedline('R12')!,
  checkSource(source: string, file: string): Violation[] {
    if (!/(?:^test_|_test|\.test|\.spec|test\.|spec\.)[^/]+$/i.test(file)) return [];
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of FAKE_TEST_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R12', ruleName: '偽造測試結果', severity: 'error', file, line: i + 1, column: match.index + 1, message: `偽造測試: ${match[0]}`, snippet: line.trim(), suggestion: '移除跳過或修正斷言' });
          break;
        }
      }
    }
    return violations;
  },
};

