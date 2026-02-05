/**
 * Erlang Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Erlang/OTP, Cowboy, RabbitMQ
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Erlang-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:Password|Secret|ApiKey|Token)\s*=\s*<<"[^"]+">>/gi,
  /\{(?:password|secret|api_key|token),\s*<<"[^"]+">>\}/gi,
  /application:get_env\s*\([^)]+,\s*(?:password|secret)[^)]*,\s*<<"[^"]+">>\)/gi,
];

/** R02: 代碼注入 */
const INJECTION_PATTERNS = [
  /erlang:binary_to_term\s*\(/gi,
  /os:cmd\s*\(/gi,
  /open_port\s*\(\s*\{spawn,/gi,
  /apply\s*\(\s*\w+,\s*\w+,/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /catch\s+_:_\s*->/gi,
  /try\s+[\s\S]*?catch\s+_\s*->\s*ok/gi,
  /process_flag\s*\(\s*trap_exit\s*,\s*false\s*\)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /\{verify,\s*verify_none\}/gi,
  /\{verify_fun,\s*\{fun\s*\(_,\s*_,\s*S\)\s*->\s*\{valid,\s*S\}/gi,
  /-kernel\s+inet_dist_listen_min\s+0/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /erlang:halt\s*\(/gi,
  /init:stop\s*\(/gi,
  /c:l\s*\(/gi,
  /code:purge\s*\(/gi,
  /crypto:block_encrypt.*des/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /receive\s+after\s+infinity/gi,
  /timer:sleep\s*\(\s*infinity\s*\)/gi,
  /gen_server:call\s*\([^,]+,\s*[^,]+,\s*infinity\s*\)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /ets:new\s*\([^)]*,\s*\[.*named_table/gi,
  /persistent_term:put\s*\(/gi,
  /application:set_env\s*\(/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*%/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Erlang: 硬編碼憑證', suggestion: '使用 application env 或 sys.config' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Erlang: 危險的動態操作', suggestion: '避免 binary_to_term 和 os:cmd' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Erlang: 忽略異常', suggestion: '正確處理 catch 模式' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Erlang: 禁用 SSL 驗證', suggestion: '啟用 verify_peer' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Erlang: 危險操作', suggestion: '使用安全的替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Erlang: 無限等待', suggestion: '設置合理的超時時間' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Erlang: 全局狀態', suggestion: '使用進程狀態' },
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

export const erlangPlugin: LanguagePlugin = {
  name: 'erlang',
  extensions: ['.erl', '.hrl', '.escript'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default erlangPlugin;
