/**
 * Julia Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Julia, Flux, Pluto, JuMP
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Julia-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token)\s*=\s*["'][^"']+["']/gi,
  /ENV\[["'](?:PASSWORD|SECRET|API_KEY|TOKEN)["']\]\s*=\s*["'][^"']+["']/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /\beval\s*\(\s*Meta\.parse/gi,
  /\binclude\s*\(\s*\$/gi,
  /\brun\s*\(\s*`.*\$/gi,
  /\bread\s*\(\s*pipeline\s*\(`.*\$/gi,
  /@eval\s+/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /try\s*[\s\S]*?catch\s*[\s\S]*?end\s*$/gim,
  /catch\s+\w+\s*$/gim,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /ssl_verify\s*=\s*false/gi,
  /verify_ssl\s*=\s*false/gi,
  /HTTP\.request.*verify\s*=\s*false/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\bccall\s*\(/gi,
  /\b@ccall\s+/gi,
  /\bunsafe_load\s*\(/gi,
  /\bpointer_from_objref\s*\(/gi,
  /\bunsafe_pointer_to_objref\s*\(/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /while\s+true/gi,
  /for\s+\w+\s+in\s+1:Inf/gi,
  /Iterators\.repeated/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /^(?:const\s+)?[A-Z_][A-Z0-9_]*\s*=/gim,
  /\bglobal\s+\w+\s*=/gi,
  /Base\.\w+\s*=/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Julia: 硬編碼憑證', suggestion: '使用 ENV 環境變數' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Julia: 潛在代碼注入', suggestion: '避免動態 eval' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Julia: 空的 catch 塊', suggestion: '正確處理異常' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Julia: 禁用 SSL 驗證', suggestion: '啟用 SSL 驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Julia: 危險的低階操作', suggestion: '使用安全的高階 API' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Julia: 無限循環', suggestion: '添加退出條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Julia: 全局變量', suggestion: '使用函數參數或模組封裝' },
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

export const juliaPlugin: LanguagePlugin = {
  name: 'julia',
  extensions: ['.jl'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default juliaPlugin;
