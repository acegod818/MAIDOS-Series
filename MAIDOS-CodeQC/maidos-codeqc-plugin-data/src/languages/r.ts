/**
 * R Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：R, RStudio, Shiny, tidyverse, data.table
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// R-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token)\s*(?:<-|=)\s*["'][^"']+["']/gi,
  /Sys\.setenv\s*\(\s*(?:PASSWORD|SECRET|API_KEY)\s*=\s*["'][^"']+["']/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /\beval\s*\(\s*parse\s*\(/gi,
  /\bsource\s*\(\s*paste/gi,
  /\bsystem\s*\(\s*paste/gi,
  /\bsystem2\s*\(\s*paste/gi,
  /\bshell\s*\(\s*paste/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /tryCatch\s*\([^)]+,\s*error\s*=\s*function\s*\([^)]*\)\s*\{\s*\}/gi,
  /suppressWarnings\s*\(/gi,
  /suppressMessages\s*\(/gi,
  /options\s*\(\s*warn\s*=\s*-1\s*\)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /httr::config\s*\(\s*ssl_verifypeer\s*=\s*(?:FALSE|0)/gi,
  /RCurl::curlOptions\s*\(\s*ssl\.verifypeer\s*=\s*FALSE/gi,
  /options\s*\(\s*download\.file\.method\s*=\s*["']wget["']/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\battach\s*\(/gi,
  /\beval\s*\(\s*parse\s*\(\s*text\s*=/gi,
  /\bget\s*\(\s*paste/gi,
  /\bassign\s*\(\s*paste/gi,
  /\.Internal\s*\(/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /while\s*\(\s*TRUE\s*\)/gi,
  /repeat\s*\{/gi,
  /read\.csv\s*\([^)]*\)\s*(?!.*nrows)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /<<-/g,
  /assign\s*\(\s*["'][^"']+["']\s*,.*envir\s*=\s*\.GlobalEnv/gi,
  /options\s*\(/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /(?:^|\s)(?:temp|tmp|foo|bar|x|y|z|df|data)\s*(?:<-|=)/gim,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'R: 硬編碼憑證', suggestion: '使用 Sys.getenv() 或 keyring 包' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'R: 潛在代碼注入', suggestion: '避免 eval(parse())' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'R: 忽略錯誤或警告', suggestion: '正確處理 tryCatch 錯誤' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'R: 禁用 SSL 驗證', suggestion: '啟用 SSL 驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'R: 危險函數', suggestion: '使用安全替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'R: 無限制操作', suggestion: '添加限制條件或使用 nrows' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'R: 全局賦值', suggestion: '使用局部變量或參數' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'R: 不良變量名', suggestion: '使用有意義的名稱' },
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

export const rPlugin: LanguagePlugin = {
  name: 'r',
  extensions: ['.r', '.R', '.rmd', '.Rmd'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default rPlugin;
