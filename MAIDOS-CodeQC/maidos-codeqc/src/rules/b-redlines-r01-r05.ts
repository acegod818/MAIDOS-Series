import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';

// =============================================================================
// R01: 硬編碼憑證
// =============================================================================

const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{3,}/gi,
  /(?:api[_-]?key|apikey)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{8,}/gi,
  /(?:secret|token|auth)\s*[=:]\s*['"`](?![\s'"`])[^'"`]{8,}/gi,
  /(?:aws[_-]?(?:access[_-]?key|secret))\s*[=:]\s*['"`][A-Za-z0-9+/]{20,}/gi,
  /-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----/gi,
  /bearer\s+[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+/gi,
];

const CREDENTIAL_WHITELIST = [/process\.env\./, /os\.environ/, /env\s*\(/, /getenv\s*\(/, /\$\{[A-Z_]+\}/, /\$[A-Z_]+/];

export const R01_CHECKER: RuleChecker = {
  rule: getRedline('R01')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      if (CREDENTIAL_WHITELIST.some(p => p.test(line))) continue;
      for (const pattern of CREDENTIAL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1, message: '檢測到硬編碼憑證', snippet: line.trim(), suggestion: '使用環境變數或密鑰管理服務' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R02: 跳過安全檢查
// =============================================================================

const BYPASS_SECURITY_PATTERNS = [
  /(?:execute|query|raw)\s*\(\s*[`'"].*\$\{/gi,
  /(?:execute|query)\s*\(\s*['"`].*\+\s*\w+/gi,
  /f['"]SELECT.*\{/gi,
  /(?:skip|bypass|disable)[_-]?(?:auth|validation|verification|check)\s*[=:]\s*true/gi,
  /isAdmin\s*[=:]\s*true/gi,
  /\beval\s*\(/gi,
  /new\s+Function\s*\(/gi,
  /pickle\.loads?\s*\(/gi,
  /yaml\.(?:unsafe_)?load\s*\(/gi,
  /ObjectInputStream/gi,
  /BinaryFormatter/gi,
];

export const R02_CHECKER: RuleChecker = {
  rule: getRedline('R02')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      // Avoid self-false-positives when a project defines regex pattern lists
      // (e.g. CodeQC's own redline rules contain `/BinaryFormatter/`).
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of BYPASS_SECURITY_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', file, line: i + 1, column: match.index + 1, message: `檢測到安全繞過: ${match[0].substring(0, 30)}`, snippet: line.trim(), suggestion: '使用參數化查詢、正確驗證' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R03: 刪除審計日誌
// =============================================================================

const DELETE_AUDIT_PATTERNS = [
  // Use word boundaries to avoid false positives like "ViewModel ... Dialog..." (del + log substrings).
  /\b(?:rm|del|remove|unlink)\b\s+.*\b(?:log|audit|trace)\b/gi,
  /\.(?:delete|remove|unlink)\s*\([^)]*\b(?:log|audit)\b/gi,
  /os\.(?:remove|unlink)\s*\([^)]*\b(?:log|audit)\b/gi,
  /fs\.(?:unlink|rm)Sync?\s*\([^)]*\b(?:log|audit)\b/gi,
  /truncate.*\b(?:log|audit)\b/gi,
  />\s*\/(?:var\/)?log\//gi,
  /UPDATE\s+.*\b(?:audit|log)(?:[_-]\w+)?\b.*SET/gi,
  /DELETE\s+FROM\s+.*\b(?:audit|log)(?:[_-]\w+)?\b/gi,
];

export const R03_CHECKER: RuleChecker = {
  rule: getRedline('R03')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      // Avoid self-false-positives for regex pattern lists.
      if (/^\s*\/.*\/[gimsuy]*\s*,?\s*$/.test(line)) continue;
      for (const pattern of DELETE_AUDIT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', file, line: i + 1, column: match.index + 1, message: '檢測到審計日誌操作', snippet: line.trim(), suggestion: '審計日誌應只讀' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R05: 忽略錯誤處理
// =============================================================================

const EMPTY_CATCH_PATTERNS: Record<string, RegExp[]> = {
  typescript: [/catch\s*\([^)]*\)\s*\{\s*\}/g, /\.catch\s*\(\s*\(\s*\w*\s*\)\s*=>\s*\{\s*\}\s*\)/g],
  javascript: [/catch\s*\([^)]*\)\s*\{\s*\}/g, /\.catch\s*\(\s*\(\s*\)\s*=>\s*\{\s*\}\s*\)/g],
  python: [/except\s*:\s*pass/g, /except\s+\w+\s*:\s*pass/g, /except\s+\w+\s+as\s+\w+\s*:\s*pass/g],
  rust: [/\.unwrap\(\)/g],
  go: [/if\s+err\s*!=\s*nil\s*\{\s*\}/g, /_\s*,?\s*=\s*\w+\([^)]*\)/g],
};

export const R05_CHECKER: RuleChecker = {
  rule: getRedline('R05')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const ext = file.split('.').pop()?.toLowerCase();
    const langMap: Record<string, string> = { ts: 'typescript', tsx: 'typescript', js: 'javascript', jsx: 'javascript', py: 'python', rs: 'rust', go: 'go' };
    const lang = langMap[ext || ''];
    if (!lang) return violations;
    const patterns = EMPTY_CATCH_PATTERNS[lang] || [];
    const normalized = source.replace(/\r\n/g, '\n');
    for (const pattern of patterns) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(normalized)) !== null) {
        const before = normalized.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({ ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: lineNum, column: 1, message: '檢測到忽略錯誤處理', snippet: match[0].substring(0, 40), suggestion: '添加錯誤處理邏輯' });
      }
    }
    return violations;
  },
};
