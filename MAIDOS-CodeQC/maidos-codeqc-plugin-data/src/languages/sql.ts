/**
 * SQL Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：PostgreSQL, MySQL, SQLite, SQL Server, Oracle
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// SQL-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /PASSWORD\s*=\s*['"][^'"]+['"]/gi,
  /IDENTIFIED\s+BY\s+['"][^'"]+['"]/gi,
  /CREATE\s+USER\s+\w+\s+PASSWORD\s+['"][^'"]+['"]/gi,
];

/** R02: SQL 注入風險 */
const INJECTION_PATTERNS = [
  /EXECUTE\s+IMMEDIATE\s+['"].*\|\|/gi,
  /EXEC\s*\(\s*@\w+\s*\+/gi,
  /sp_executesql\s+@\w+\s*\+/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /DROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?(?:audit|log|history)/gi,
  /TRUNCATE\s+TABLE\s+(?:audit|log|history)/gi,
  /DELETE\s+FROM\s+(?:audit|log|history)/gi,
  /ALTER\s+TABLE\s+(?:audit|log|history)\s+DROP/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /ALTER\s+SYSTEM\s+SET\s+audit_trail\s*=\s*(?:NONE|FALSE)/gi,
  /SET\s+sql_safe_updates\s*=\s*0/gi,
  /REVOKE\s+ALL/gi,
  /GRANT\s+ALL\s+PRIVILEGES/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /DROP\s+DATABASE/gi,
  /DROP\s+SCHEMA/gi,
  /xp_cmdshell/gi,
  /LOAD_FILE\s*\(/gi,
  /INTO\s+(?:OUTFILE|DUMPFILE)/gi,
  /UTL_FILE/gi,
];

/** R09: 無限制查詢 */
const UNLIMITED_PATTERNS = [
  /SELECT\s+\*\s+FROM\s+(?!.*(?:LIMIT|TOP|FETCH|ROWNUM|WHERE))/gi,
  /DELETE\s+FROM\s+\w+\s*(?:;|$)/gim,
  /UPDATE\s+\w+\s+SET\s+.*(?:;|$)(?!.*WHERE)/gim,
];

/** P04: 魔法數字 */
const MAGIC_NUMBER_PATTERNS = [
  /WHERE\s+\w+\s*(?:=|>|<|>=|<=)\s*\d{3,}/gi,
  /LIMIT\s+\d{4,}/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*--/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'SQL: 硬編碼密碼', suggestion: '使用參數化或環境變數' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'SQL: 動態 SQL 拼接', suggestion: '使用預處理語句' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'SQL: 刪除審計表', suggestion: '保留審計追蹤' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'SQL: 禁用安全功能', suggestion: '保持安全設定啟用' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'SQL: 危險操作', suggestion: '使用安全替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'SQL: 無限制查詢', suggestion: '添加 LIMIT/WHERE 條件' },
    { patterns: MAGIC_NUMBER_PATTERNS, ruleId: 'P04', ruleName: '魔法數字', severity: 'warning', message: 'SQL: 魔法數字', suggestion: '使用變數或常量' },
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

export const sqlPlugin: LanguagePlugin = {
  name: 'sql',
  extensions: ['.sql', '.psql', '.plsql', '.pgsql'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default sqlPlugin;
