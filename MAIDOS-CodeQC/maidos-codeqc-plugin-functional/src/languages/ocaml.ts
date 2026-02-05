/**
 * OCaml Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：OCaml, Reason, ReScript, Dune
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// OCaml-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /let\s+(?:password|secret|api_?key|token)\s*=\s*["'][^"']+["']/gi,
  /Sys\.getenv_opt\s*["'](?:PASSWORD|SECRET|API_KEY)["']\s*\|>\s*Option\.value\s*~default:["'][^"']+["']/gi,
];

/** R02: 不安全操作 */
const INJECTION_PATTERNS = [
  /Obj\.magic/gi,
  /Obj\.repr/gi,
  /Unix\.system\s+\(/gi,
  /Unix\.open_process_in\s+\(/gi,
  /Marshal\.from_string/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /try\s+[\s\S]*?with\s+_\s*->/gi,
  /\|\s*exception\s+_\s*->/gi,
  /Option\.get\s+/gi,
  /List\.hd\s+/gi,
  /List\.tl\s+/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /\?\s*verify\s*=\s*false/gi,
  /ssl_verify\s*=\s*false/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\bfailwith\b/gi,
  /\binvalid_arg\b/gi,
  /\braise\s+Not_found\b/gi,
  /\bArray\.unsafe_get\b/gi,
  /\bArray\.unsafe_set\b/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /Seq\.forever/gi,
  /let\s+rec\s+\w+\s+\(\)\s*=[\s\S]*?\w+\s+\(\)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /let\s+\w+\s*=\s*ref\s+/gi,
  /let\s+\w+\s*:\s*\w+\s+ref\s*=/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*\(\*/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'OCaml: 硬編碼憑證', suggestion: '使用環境變數' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'OCaml: 不安全操作', suggestion: '避免 Obj.magic 和外部命令' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'OCaml: 忽略異常或使用部分函數', suggestion: '使用 Option 或 Result 類型' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'OCaml: 禁用 SSL 驗證', suggestion: '啟用 SSL 驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'OCaml: 危險的部分函數', suggestion: '使用安全版本如 List.nth_opt' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'OCaml: 無限遞歸', suggestion: '確保有終止條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'OCaml: 全局 ref', suggestion: '使用局部狀態或函數參數' },
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

export const ocamlPlugin: LanguagePlugin = {
  name: 'ocaml',
  extensions: ['.ml', '.mli', '.re', '.rei', '.res', '.resi'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default ocamlPlugin;
