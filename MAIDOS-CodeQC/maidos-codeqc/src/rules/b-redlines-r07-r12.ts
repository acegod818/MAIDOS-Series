import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';

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

