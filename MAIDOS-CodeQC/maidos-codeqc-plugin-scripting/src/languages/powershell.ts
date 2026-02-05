/**
 * PowerShell Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Windows PowerShell, PowerShell Core
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// PowerShell-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /\$(?:password|passwd|secret|apikey|token)\s*=\s*["'][^"']+["']/gi,
  /ConvertTo-SecureString\s+["'][^"']+["']\s+-AsPlainText/gi,
  /-Credential\s+\$?\w+:\w+/gi,
  /\[PSCredential\]::new\s*\(\s*["'][^"']+["']/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /Invoke-Expression\s+\$/gi,
  /iex\s+\$/gi,
  /\.\s+\$/gi,
  /&\s+\$/gi,
  /Start-Process\s+.*-ArgumentList\s+\$/gi,
];

/** R03: 刪除日誌 */
const DELETE_AUDIT_PATTERNS = [
  /Clear-EventLog/gi,
  /Remove-Item\s+.*\\Windows\\System32\\winevt/gi,
  /wevtutil\s+cl/gi,
  /Remove-Item\s+.*\.log/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /\$ErrorActionPreference\s*=\s*["']SilentlyContinue["']/gi,
  /-ErrorAction\s+SilentlyContinue/gi,
  /catch\s*\{\s*\}/gi,
  /trap\s*\{\s*continue\s*\}/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /Set-ExecutionPolicy\s+(?:Bypass|Unrestricted)/gi,
  /-ExecutionPolicy\s+Bypass/gi,
  /\[System\.Net\.ServicePointManager\]::ServerCertificateValidationCallback\s*=\s*\{\s*\$true\s*\}/gi,
  /SkipCertificateCheck/gi,
  /Disable-WindowsOptionalFeature.*Defender/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /Remove-Item\s+-Recurse\s+-Force\s+C:\\/gi,
  /Format-Volume/gi,
  /Clear-Disk/gi,
  /Stop-Service\s+.*-Force/gi,
];

/** R10: 明文傳輸 */
const PLAINTEXT_PATTERNS = [
  /Invoke-WebRequest\s+.*http:\/\/[^'"]*(?:login|auth|password|api)/gi,
  /Invoke-RestMethod\s+.*http:\/\/[^'"]*(?:login|auth|password|api)/gi,
  /\[Net\.WebClient\].*http:\/\//gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /\$global:/gi,
  /\$script:\w+\s*=/gi,
  /Set-Variable\s+-Scope\s+Global/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'PowerShell: 硬編碼憑證', suggestion: '使用 Get-Credential 或 Azure Key Vault' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'PowerShell: 潛在代碼注入', suggestion: '避免使用 Invoke-Expression' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'PowerShell: 刪除事件日誌', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'PowerShell: 忽略錯誤', suggestion: '使用 try/catch 正確處理' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'PowerShell: 禁用安全功能', suggestion: '保持安全策略啟用' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'PowerShell: 危險操作', suggestion: '使用安全的替代方案' },
    { patterns: PLAINTEXT_PATTERNS, ruleId: 'R10', ruleName: '明文傳輸敏感', severity: 'error', message: 'PowerShell: 明文傳輸', suggestion: '使用 HTTPS' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'PowerShell: 全局變量', suggestion: '使用參數傳遞' },
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

export const powershellPlugin: LanguagePlugin = {
  name: 'powershell',
  extensions: ['.ps1', '.psm1', '.psd1'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default powershellPlugin;
