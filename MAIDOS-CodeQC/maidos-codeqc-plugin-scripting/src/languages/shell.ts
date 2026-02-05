/**
 * Shell/Bash Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Bash, Sh, Zsh, Fish
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Shell-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:PASSWORD|PASSWD|SECRET|API_?KEY|TOKEN|AUTH)=["']?[^$\s"']+["']?/gi,
  /export\s+(?:PASSWORD|SECRET|API_?KEY|TOKEN)=/gi,
  /curl\s+.*--user\s+\w+:[^\s]+/gi,
  /wget\s+.*--password=[^\s]+/gi,
];

/** R02: 命令注入 */
const INJECTION_PATTERNS = [
  /eval\s+["']?\$/gi,
  /\$\(\s*\$\w+\s*\)/gi,
  /`\s*\$\w+\s*`/gi,
  /\bsh\s+-c\s+["']\$/gi,
  /\bbash\s+-c\s+["']\$/gi,
];

/** R03: 刪除日誌 */
const DELETE_AUDIT_PATTERNS = [
  /rm\s+(?:-rf?\s+)?(?:\/var\/log|\/tmp|~\/\.\w+_history)/gi,
  />\s*\/var\/log\/\w+/gi,
  /truncate\s+.*\/var\/log/gi,
  /shred\s+.*(?:history|log)/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /2>\s*\/dev\/null/gi,
  /\|\|\s*true\s*$/gim,
  /\|\|\s*:\s*$/gim,
  /set\s+\+e/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /--insecure/gi,
  /-k\s+/gi,
  /curl\s+.*--no-check-certificate/gi,
  /wget\s+.*--no-check-certificate/gi,
  /StrictHostKeyChecking=no/gi,
  /chmod\s+777/gi,
  /chmod\s+666/gi,
];

/** R08: 危險命令 */
const DANGEROUS_PATTERNS = [
  /rm\s+-rf\s+\/(?!\w)/gi,
  /rm\s+-rf\s+\$\w*\/?\s*$/gim,
  /dd\s+.*of=\/dev\/sd/gi,
  /mkfs\s+/gi,
  /:\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:/g,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /while\s+true\s*;?\s*do/gi,
  /while\s+:\s*;?\s*do/gi,
  /for\s*\(\s*;\s*;\s*\)/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /^(?:temp|tmp|foo|bar|baz|test|data|var|x|y|z)=/gim,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Shell: 硬編碼憑證', suggestion: '使用環境變數或密鑰管理' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Shell: 潛在命令注入', suggestion: '使用引號和驗證輸入' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'Shell: 刪除日誌或歷史', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Shell: 忽略錯誤', suggestion: '使用 set -e 並正確處理錯誤' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Shell: 禁用安全檢查', suggestion: '啟用 SSL 驗證，避免 777 權限' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Shell: 危險命令', suggestion: '使用安全的替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Shell: 無限循環', suggestion: '添加退出條件' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'Shell: 不良變量名', suggestion: '使用有意義的名稱' },
  ];
  
  for (const { patterns: patternList, ruleId, ruleName, severity, message, suggestion } of patterns) {
    for (const pattern of patternList) {
      pattern.lastIndex = 0;
      if (pattern.test(line)) {
        violations.push({ ruleId, ruleName, severity, file, line: lineNum, column: 1, message, snippet: line.trim(), suggestion });
        break;
      }
    }
  }
  
  return violations;
}

// =============================================================================
// Export
// =============================================================================

export const shellPlugin: LanguagePlugin = {
  name: 'shell',
  extensions: ['.sh', '.bash', '.zsh', '.fish', '.ksh'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default shellPlugin;
