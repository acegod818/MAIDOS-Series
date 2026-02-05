/**
 * PHP Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Laravel, WordPress, Drupal, Symfony, CodeIgniter
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// PHP-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /\$(?:password|passwd|pwd|secret|api_?key|token|auth)\s*=\s*['"][^'"]+['"]/gi,
  /define\s*\(\s*['"](?:DB_PASSWORD|API_KEY|SECRET_KEY|AUTH_TOKEN)['"]\s*,\s*['"][^'"]+['"]\s*\)/gi,
  /\$_ENV\s*\[\s*['"](?:password|secret|key|token)['"]\s*\]\s*=\s*['"][^'"]+['"]/gi,
];

/** R02: SQL 注入 / 命令注入 */
const INJECTION_PATTERNS = [
  /\$\w+\s*=\s*['"]SELECT\s+.*\$(?!pdo)/gi,
  /mysql_query\s*\(\s*['"].*\$/gi,
  /mysqli_query\s*\([^,]+,\s*['"].*\$/gi,
  /\bexec\s*\(\s*\$/gi,
  /\bshell_exec\s*\(\s*\$/gi,
  /\bsystem\s*\(\s*\$/gi,
  /\bpassthru\s*\(\s*\$/gi,
  /\beval\s*\(\s*\$/gi,
  /\bpreg_replace\s*\(\s*['"]\/.*\/e['"]/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /catch\s*\(\s*(?:Exception|\$e)\s*\)\s*\{\s*\}/gi,
  /catch\s*\(\s*\w+\s+\$\w+\s*\)\s*\{\s*\/\/\s*\}/gi,
  /@\s*(?:file_get_contents|fopen|include|require)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /error_reporting\s*\(\s*0\s*\)/gi,
  /display_errors\s*=\s*(?:Off|0|false)/gi,
  /CURLOPT_SSL_VERIFYPEER\s*(?:=>|,)\s*false/gi,
  /CURLOPT_SSL_VERIFYHOST\s*(?:=>|,)\s*0/gi,
  /verify_peer\s*(?:=>|=)\s*false/gi,
];

/** R08: 已知漏洞函數 */
const VULNERABLE_PATTERNS = [
  /\bmysql_(?:query|connect|escape_string)\s*\(/gi,
  /\bereg(?:i)?\s*\(/gi,
  /\bsplit\s*\(/gi,
  /\bcreate_function\s*\(/gi,
  /\bextract\s*\(\s*\$_(?:GET|POST|REQUEST)/gi,
  /\bparse_str\s*\(\s*\$_(?:GET|POST|REQUEST)/gi,
  /\bmd5\s*\(\s*\$(?:password|passwd|pwd)/gi,
  /\bsha1\s*\(\s*\$(?:password|passwd|pwd)/gi,
  /\bunserialize\s*\(\s*\$_(?:GET|POST|REQUEST|COOKIE)/gi,
];

/** R10: 明文傳輸 */
const PLAINTEXT_PATTERNS = [
  /['"]http:\/\/[^'"]*(?:login|auth|password|token|api|admin)[^'"]*['"]/gi,
  /\bmail\s*\([^)]*password[^)]*\)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /\bglobal\s+\$/gi,
  /\$GLOBALS\s*\[/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*(?:\/\/|#|\/\*|\*)/.test(line)) return violations;
  
  for (const pattern of CREDENTIAL_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R01',
        ruleName: '硬編碼憑證',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 檢測到硬編碼憑證',
        snippet: line.trim(),
        suggestion: '使用環境變數或設定檔',
      });
    }
  }
  
  for (const pattern of INJECTION_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R02',
        ruleName: '跳過安全檢查',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 潛在注入漏洞',
        snippet: line.trim(),
        suggestion: '使用 PDO 預處理語句或 escapeshellarg()',
      });
    }
  }
  
  for (const pattern of ERROR_HANDLING_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 空 catch 或錯誤抑制',
        snippet: line.trim(),
        suggestion: '正確處理異常，移除 @ 錯誤抑制',
      });
    }
  }
  
  for (const pattern of SECURITY_DISABLE_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R07',
        ruleName: '關閉安全功能',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 安全功能被禁用',
        snippet: line.trim(),
        suggestion: '啟用 SSL 驗證和錯誤報告',
      });
    }
  }
  
  for (const pattern of VULNERABLE_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R08',
        ruleName: '使用已知漏洞',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 使用已棄用或不安全的函數',
        snippet: line.trim(),
        suggestion: '使用 PDO、password_hash()、preg_match()',
      });
    }
  }
  
  for (const pattern of PLAINTEXT_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R10',
        ruleName: '明文傳輸敏感',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 明文傳輸敏感資料',
        snippet: line.trim(),
        suggestion: '使用 HTTPS',
      });
    }
  }
  
  for (const pattern of GLOBAL_STATE_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'P07',
        ruleName: '全局狀態',
        severity: 'warning',
        file,
        line: lineNum,
        column: 1,
        message: 'PHP: 使用全局變量',
        snippet: line.trim(),
        suggestion: '使用依賴注入或類屬性',
      });
    }
  }
  
  return violations;
}

// =============================================================================
// Export
// =============================================================================

export const phpPlugin: LanguagePlugin = {
  name: 'php',
  extensions: ['.php', '.phtml', '.php3', '.php4', '.php5', '.php7', '.phps'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default phpPlugin;
