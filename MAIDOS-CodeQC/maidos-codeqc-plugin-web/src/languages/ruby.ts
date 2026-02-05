/**
 * Ruby Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Rails, Sinatra, Jekyll, Hanami
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Ruby-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token|auth_token)\s*=\s*['"][^'"]+['"]/gi,
  /ENV\s*\[\s*['"](?:password|secret|key|token)['"]\s*\]\s*(?:\|\||=)\s*['"][^'"]+['"]/gi,
  /\.env\.fetch\s*\(\s*['"](?:password|secret|key|token)['"]\s*,\s*['"][^'"]+['"]\s*\)/gi,
];

/** R02: 注入漏洞 */
const INJECTION_PATTERNS = [
  /\.where\s*\(\s*['"].*#\{/gi,
  /\.find_by_sql\s*\(\s*['"].*#\{/gi,
  /ActiveRecord::Base\.connection\.execute\s*\(\s*['"].*#\{/gi,
  /\beval\s*\(\s*(?:params|request)/gi,
  /\bsystem\s*\(\s*(?:params|request|#\{)/gi,
  /\bexec\s*\(\s*(?:params|request|#\{)/gi,
  /`.*#\{.*params/gi,
  /%x\{.*#\{.*params/gi,
  /\.send\s*\(\s*params/gi,
  /\.constantize\s*$/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /rescue\s*=>\s*\w*\s*$/gim,
  /rescue\s+StandardError\s*$/gim,
  /rescue\s+Exception\s*$/gim,
  /rescue\s*;\s*end/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /protect_from_forgery\s*(?:with:|only:|except:)?\s*:null_session/gi,
  /skip_before_action\s*:verify_authenticity_token/gi,
  /ssl_options\s*=.*verify_mode.*VERIFY_NONE/gi,
  /OpenSSL::SSL::VERIFY_NONE/gi,
];

/** R08: 已知漏洞 */
const VULNERABLE_PATTERNS = [
  /Digest::MD5\.hexdigest\s*\(\s*password/gi,
  /Digest::SHA1\.hexdigest\s*\(\s*password/gi,
  /\.html_safe\s*$/gi,
  /raw\s*\(\s*(?:params|@)/gi,
  /render\s+(?:inline|text):\s*(?:params|request)/gi,
  /YAML\.load\s*\(\s*(?:params|request|File)/gi,
  /Marshal\.load\s*\(\s*(?:params|request)/gi,
];

/** R09: 無限制資源 */
const UNLIMITED_PATTERNS = [
  /\.all(?:\s*$|\.each)/gi,
  /loop\s+do\s*$/gim,
  /while\s+true/gi,
];

/** R10: 明文傳輸 */
const PLAINTEXT_PATTERNS = [
  /['"]http:\/\/[^'"]*(?:login|auth|password|api|admin)[^'"]*['"]/gi,
  /Net::HTTP\.(?:get|post)\s*\(\s*['"]http:/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /\$\w+\s*=/g,
  /@@\w+\s*=/g,
  /class\s+<<\s+self/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
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
        message: 'Ruby: 檢測到硬編碼憑證',
        snippet: line.trim(),
        suggestion: '使用 Rails credentials 或環境變數',
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
        message: 'Ruby: 潛在注入漏洞',
        snippet: line.trim(),
        suggestion: '使用 ActiveRecord 查詢介面或 sanitize',
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
        message: 'Ruby: 空的或過於寬泛的 rescue',
        snippet: line.trim(),
        suggestion: '捕獲特定異常並正確處理',
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
        message: 'Ruby: 安全功能被禁用',
        snippet: line.trim(),
        suggestion: '啟用 CSRF 保護和 SSL 驗證',
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
        message: 'Ruby: 使用不安全的方法',
        snippet: line.trim(),
        suggestion: '使用 BCrypt、YAML.safe_load、CGI.escapeHTML',
      });
    }
  }
  
  for (const pattern of UNLIMITED_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(line)) {
      violations.push({
        ruleId: 'R09',
        ruleName: '無限制資源',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: 'Ruby: 無限制查詢或循環',
        snippet: line.trim(),
        suggestion: '使用 .limit() 或 find_each',
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
        message: 'Ruby: 明文傳輸',
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
        message: 'Ruby: 全局或類變量',
        snippet: line.trim(),
        suggestion: '使用實例變量或依賴注入',
      });
    }
  }
  
  return violations;
}

// =============================================================================
// Export
// =============================================================================

export const rubyPlugin: LanguagePlugin = {
  name: 'ruby',
  extensions: ['.rb', '.rake', '.gemspec', '.ru', '.erb'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default rubyPlugin;
