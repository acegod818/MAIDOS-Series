/**
 * Perl Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Perl 5, CGI, Mojolicious, Dancer
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Perl-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /\$(?:password|passwd|secret|api_?key|token)\s*=\s*["'][^"']+["']/gi,
  /my\s+\$(?:password|secret|key|token)\s*=\s*["'][^"']+["']/gi,
];

/** R02: 注入漏洞 */
const INJECTION_PATTERNS = [
  /\beval\s+["']\$/gi,
  /\beval\s+\$/gi,
  /`\$\w+`/gi,
  /\bsystem\s*\(\s*\$/gi,
  /\bexec\s*\(\s*\$/gi,
  /\bopen\s*\(\s*\w+\s*,\s*["']\|/gi,
  /\bopen\s*\(\s*\w+\s*,\s*\$/gi,
  /\bqx\s*[{(\/]\$/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /\bopen\s*\([^)]+\)\s*(?:;|\|\|)\s*$/gim,
  /\beval\s*\{[^}]*\}\s*;\s*$/gim,
  /\bor\s+die\s*;/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /no\s+strict/gi,
  /no\s+warnings/gi,
  /\$ENV\{PATH\}\s*=\s*["']["']/gi,
  /verify_hostname\s*=>\s*0/gi,
  /SSL_verify_mode\s*=>\s*0/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\bcrypt\s*\(\s*\$(?:password|passwd)/gi,
  /\bMD5\s*\(\s*\$(?:password|passwd)/gi,
  /\bSHA1\s*\(\s*\$(?:password|passwd)/gi,
  /\buntaint\s*\(/gi,
  /\$\w+\s*=~\s*\/\.\*\/\s*;\s*\$\w+\s*=\s*\$&/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /while\s*\(\s*1\s*\)/gi,
  /for\s*\(\s*;\s*;\s*\)/gi,
  /SELECT\s+\*\s+FROM/gi,
];

/** R10: 明文傳輸 */
const PLAINTEXT_PATTERNS = [
  /["']http:\/\/[^"']*(?:login|auth|password|api)[^"']*["']/gi,
  /LWP::UserAgent.*http:\/\//gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /\bour\s+\$/gi,
  /\buse\s+vars\s+qw/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Perl: 硬編碼憑證', suggestion: '使用環境變數' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Perl: 潛在注入漏洞', suggestion: '使用 taint mode 和參數化查詢' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Perl: 錯誤處理不足', suggestion: '使用 Try::Tiny 或 autodie' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Perl: 禁用安全檢查', suggestion: '使用 use strict; use warnings;' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Perl: 使用不安全函數', suggestion: '使用 Crypt::Argon2 或 bcrypt' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Perl: 無限制操作', suggestion: '添加限制條件' },
    { patterns: PLAINTEXT_PATTERNS, ruleId: 'R10', ruleName: '明文傳輸敏感', severity: 'error', message: 'Perl: 明文傳輸', suggestion: '使用 HTTPS' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Perl: 全局變量', suggestion: '使用 my 局部變量' },
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

export const perlPlugin: LanguagePlugin = {
  name: 'perl',
  extensions: ['.pl', '.pm', '.t', '.cgi'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default perlPlugin;
