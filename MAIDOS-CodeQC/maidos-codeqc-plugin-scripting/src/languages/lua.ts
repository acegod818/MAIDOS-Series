/**
 * Lua Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Lua 5.x, LuaJIT, LÖVE, OpenResty, Neovim
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Lua-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token)\s*=\s*["'][^"']+["']/gi,
  /local\s+(?:password|secret|key|token)\s*=\s*["'][^"']+["']/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /\bloadstring\s*\(/gi,
  /\bload\s*\(\s*[^"']/gi,
  /\bdofile\s*\(\s*[^"']/gi,
  /\brequire\s*\(\s*[^"']/gi,
  /\bos\.execute\s*\(/gi,
  /\bio\.popen\s*\(/gi,
  /\bdebug\.debug\s*\(/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /pcall\s*\([^)]+\)\s*$/gim,
  /xpcall\s*\([^)]+\)\s*$/gim,
  /assert\s*\(\s*false\s*\)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /debug\.setmetatable/gi,
  /debug\.setfenv/gi,
  /rawset\s*\(\s*_G/gi,
  /package\.loadlib/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\brawequal\s*\(/gi,
  /\brawget\s*\(\s*_G/gi,
  /\bgetfenv\s*\(/gi,
  /\bsetfenv\s*\(/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /while\s+true\s+do/gi,
  /repeat\s+until\s+false/gi,
  /for\s+\w+\s*=\s*1\s*,\s*math\.huge/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /^[A-Z_][A-Z0-9_]*\s*=/gim,
  /\b_G\s*\[/gi,
  /\b_G\.\w+\s*=/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*--/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Lua: 硬編碼憑證', suggestion: '使用環境變數 os.getenv()' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Lua: 潛在代碼注入', suggestion: '避免動態代碼加載' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Lua: 錯誤處理不足', suggestion: '檢查 pcall 返回值' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Lua: 危險的元表操作', suggestion: '限制 debug 庫使用' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Lua: 危險函數', suggestion: '使用安全替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Lua: 無限循環', suggestion: '添加退出條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Lua: 全局變量', suggestion: '使用 local' },
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

export const luaPlugin: LanguagePlugin = {
  name: 'lua',
  extensions: ['.lua'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default luaPlugin;
