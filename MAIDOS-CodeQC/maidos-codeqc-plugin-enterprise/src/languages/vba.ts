/**
 * VBA Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：VBA (Excel, Access, Word), VBScript
 * 領域：Office 自動化、企業報表、遺留系統
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// VBA-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:strPassword|strSecret|strApiKey|sPassword)\s*=\s*["'][^"']+["']/gi,
  /Const\s+(?:PASSWORD|SECRET|API_KEY)\s*=\s*["'][^"']+["']/gi,
  /\.Open\s+["'][^"']*;Password=[^;]+;/gi,
];

/** R02: 代碼注入 / 命令執行 */
const INJECTION_PATTERNS = [
  /Shell\s*\(/gi,
  /CreateObject\s*\(\s*["']WScript\.Shell["']\s*\)/gi,
  /CreateObject\s*\(\s*["']Scripting\.FileSystemObject["']\s*\)/gi,
  /Eval\s*\(/gi,
  /Execute\s+/gi,
  /ExecuteGlobal\s+/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /Kill\s+["'][^"']*(?:log|audit|history)[^"']*["']/gi,
  /\.Delete[\s\S]*?(?:log|audit)/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /On\s+Error\s+Resume\s+Next/gi,
  /On\s+Error\s+GoTo\s+0\s*$/gim,
  /Err\.Clear/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /Application\.EnableEvents\s*=\s*False/gi,
  /Application\.DisplayAlerts\s*=\s*False/gi,
  /Application\.ScreenUpdating\s*=\s*False/gi,
  /Application\.AutomationSecurity\s*=\s*msoAutomationSecurityLow/gi,
  /TrustAccess.*True/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /GetObject\s*\(/gi,
  /CreateObject\s*\(\s*["']ADODB/gi,
  /\.SaveAs[\s\S]*?FileFormat\s*:?=\s*xlOpenXMLWorkbookMacroEnabled/gi,
  /SendKeys\s+/gi,
  /DoCmd\.RunSQL/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /Do\s+While\s+True/gi,
  /Do\s*$[\s\S]*?Loop\s+While\s+True/gim,
  /While\s+True[\s\S]*?Wend/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /Public\s+\w+\s+As\s+/gi,
  /Global\s+\w+\s+As\s+/gi,
  /Static\s+\w+\s+As\s+/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /Dim\s+(?:temp|data|var|x|y|z|i|j|k|n|m|str|obj|rng)\s+As\s+/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*'/.test(line) || /^\s*Rem\s+/i.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'VBA: 硬編碼憑證', suggestion: '使用環境變數或加密存儲' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'VBA: 危險的代碼執行', suggestion: '避免 Shell/Eval/Execute' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'VBA: 刪除日誌文件', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'VBA: On Error Resume Next', suggestion: '使用結構化錯誤處理' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'VBA: 禁用安全功能', suggestion: '保持安全設定啟用' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'VBA: 危險操作', suggestion: '使用更安全的替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'VBA: 無限循環', suggestion: '添加退出條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'VBA: 公開/全局變量', suggestion: '使用局部變量和參數' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'VBA: 不良變量名', suggestion: '使用描述性命名' },
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

export const vbaPlugin: LanguagePlugin = {
  name: 'vba',
  extensions: ['.bas', '.cls', '.frm', '.vbs', '.vba'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default vbaPlugin;
