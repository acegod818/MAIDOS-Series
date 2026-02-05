/**
 * Elixir Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Elixir, Phoenix, Ecto, LiveView
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Elixir-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|secret|api_?key|token)\s*(?::|=)\s*["'][^"']+["']/gi,
  /Application\.(?:get_env|fetch_env!?)\s*\([^)]*(?:password|secret|key|token)[^)]*,\s*["'][^"']+["']/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /Code\.eval_string/gi,
  /Code\.eval_file/gi,
  /Code\.eval_quoted/gi,
  /:erlang\.binary_to_term/gi,
  /Kernel\.apply\s*\(\s*\w+,\s*\w+,/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /try\s+do[\s\S]*?rescue\s+_\s*->/gi,
  /catch\s+:exit,\s*_\s*->/gi,
  /\|>\s*case\s+do\s+_\s*->/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /verify:\s*:verify_none/gi,
  /ssl:\s*\[\s*verify:\s*:verify_none/gi,
  /Plug\.CSRFProtection.*false/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /:os\.cmd\s*\(/gi,
  /System\.cmd\s*\(\s*[^,]+,\s*\[.*#\{/gi,
  /Port\.open/gi,
  /:erlang\.open_port/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /Repo\.all\s*\(/gi,
  /Stream\.iterate/gi,
  /receive\s+do\s+after\s+:infinity/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /Agent\.start_link.*name:\s*:global/gi,
  /:persistent_term\.put/gi,
  /Application\.put_env/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Elixir: 硬編碼憑證', suggestion: '使用 runtime.exs 或環境變數' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Elixir: 動態代碼執行', suggestion: '避免 Code.eval_*' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Elixir: 忽略異常', suggestion: '正確處理 rescue/catch' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Elixir: 禁用安全功能', suggestion: '啟用 SSL 驗證和 CSRF' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Elixir: 危險系統調用', suggestion: '使用安全的替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Elixir: 無限制操作', suggestion: '使用分頁或 Stream' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Elixir: 全局狀態', suggestion: '使用進程狀態或 ETS' },
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

export const elixirPlugin: LanguagePlugin = {
  name: 'elixir',
  extensions: ['.ex', '.exs', '.eex', '.heex', '.leex'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default elixirPlugin;
