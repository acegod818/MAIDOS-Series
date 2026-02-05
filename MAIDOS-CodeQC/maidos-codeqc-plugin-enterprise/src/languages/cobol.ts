/**
 * COBOL Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：COBOL-85, COBOL 2002, Enterprise COBOL, GnuCOBOL
 * 領域：金融、保險、政府、銀行核心系統
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// COBOL-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /MOVE\s+["'][^"']+["']\s+TO\s+(?:WS-PASSWORD|WS-SECRET|WS-API-KEY|PASSWORD|SECRET)/gi,
  /01\s+(?:PASSWORD|SECRET|API-KEY)\s+PIC\s+X\([^)]+\)\s+VALUE\s+["'][^"']+["']/gi,
  /WORKING-STORAGE\s+SECTION[\s\S]*?(?:PASSWORD|SECRET)\s+VALUE\s+["'][^"']+["']/gi,
];

/** R02: SQL 注入 */
const INJECTION_PATTERNS = [
  /EXEC\s+SQL[\s\S]*?STRING[\s\S]*?END-EXEC/gi,
  /EXEC\s+SQL\s+EXECUTE\s+IMMEDIATE\s+:WS-/gi,
  /CALL\s+["']SYSTEM["']\s+USING/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /DELETE\s+FROM\s+(?:AUDIT|LOG|HISTORY)/gi,
  /EXEC\s+SQL\s+DELETE\s+FROM\s+(?:AUDIT|LOG)/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /NOT\s+ON\s+EXCEPTION\s+CONTINUE/gi,
  /ON\s+EXCEPTION\s+CONTINUE/gi,
  /INVALID\s+KEY\s+CONTINUE/gi,
  /AT\s+END\s+CONTINUE/gi,
  /FILE\s+STATUS\s+IS[\s\S]*?(?!IF\s+WS-FILE-STATUS)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /SECURITY\s+IS\s+NONE/gi,
  /WITH\s+NO\s+LOCK/gi,
  /COMMIT\s+RELEASE/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /ALTER\s+\w+\s+TO\s+PROCEED\s+TO/gi,
  /GO\s+TO\s+DEPENDING\s+ON/gi,
  /STOP\s+RUN/gi,
  /INSPECT\s+[\s\S]*?REPLACING\s+ALL/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /PERFORM\s+\w+\s+UNTIL\s+1\s*=\s*0/gi,
  /PERFORM\s+\w+\s+FOREVER/gi,
  /EXEC\s+SQL\s+SELECT\s+\*\s+FROM(?!\s+[\s\S]*?WHERE)/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /01\s+(?:WS-TEMP|WS-DATA|WS-VAR|WS-X|WS-Y|FILLER)\s+PIC/gi,
  /MOVE\s+\w+\s+TO\s+(?:TEMP|DATA|VAR|X|Y)\b/gi,
];

/** P13: TODO */
const TODO_PATTERNS = [
  /\*.*(?:TODO|FIXME|HACK|XXX)/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*\*/.test(line) && !TODO_PATTERNS.some(p => p.test(line))) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning' | 'info'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'COBOL: 硬編碼憑證', suggestion: '使用 CICS 安全設施或外部配置' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'COBOL: 動態 SQL 拼接', suggestion: '使用參數化查詢' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'COBOL: 刪除審計記錄', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'COBOL: 忽略錯誤狀態', suggestion: '檢查 FILE STATUS 和 SQLCODE' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'COBOL: 禁用安全功能', suggestion: '啟用記錄鎖定和安全' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'COBOL: 危險語句', suggestion: '避免 ALTER 和 GO TO DEPENDING' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'COBOL: 無限循環或無限制查詢', suggestion: '添加退出條件和 WHERE 子句' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'COBOL: 不良變量名', suggestion: '使用有意義的數據名' },
    { patterns: TODO_PATTERNS, ruleId: 'P13', ruleName: 'TODO 堆積', severity: 'info', message: 'COBOL: TODO 標記', suggestion: '處理或移除 TODO' },
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

export const cobolPlugin: LanguagePlugin = {
  name: 'cobol',
  extensions: ['.cob', '.cbl', '.cpy', '.CBL', '.COB'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default cobolPlugin;
