/**
 * PL/SQL Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Oracle PL/SQL, Oracle Database
 * 領域：企業資料庫、金融系統、ERP
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// PL/SQL-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:v_password|v_secret|v_api_key|l_password)\s*:?=\s*['"][^'"]+['"]/gi,
  /CONNECT\s+\w+\/[^\s@]+@/gi,
  /IDENTIFIED\s+BY\s+['"][^'"]+['"]/gi,
  /UTL_HTTP\.SET_AUTHENTICATION[\s\S]*?['"][^'"]+['"]/gi,
];

/** R02: SQL 注入 */
const INJECTION_PATTERNS = [
  /EXECUTE\s+IMMEDIATE\s+\w+\s*\|\|/gi,
  /DBMS_SQL\.PARSE[\s\S]*?\|\|/gi,
  /DBMS_UTILITY\.EXEC_DDL_STATEMENT/gi,
  /UTL_FILE\.FOPEN[\s\S]*?\|\|/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /DELETE\s+FROM\s+(?:AUD\$|FGA_LOG\$|AUDIT_TRAIL|SYS\.AUD)/gi,
  /TRUNCATE\s+TABLE\s+(?:AUD\$|FGA_LOG\$)/gi,
  /DBMS_AUDIT_MGMT\.CLEAN_AUDIT_TRAIL/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /EXCEPTION\s+WHEN\s+OTHERS\s+THEN\s+NULL\s*;/gi,
  /EXCEPTION\s+WHEN\s+OTHERS\s+THEN\s+RAISE\s*;/gi,
  /PRAGMA\s+EXCEPTION_INIT[\s\S]*?-20\d{3}/gi,
  /WHEN\s+NO_DATA_FOUND\s+THEN\s+NULL/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /GRANT\s+DBA\s+TO/gi,
  /GRANT\s+ALL\s+PRIVILEGES/gi,
  /ALTER\s+SYSTEM\s+SET\s+audit_trail\s*=\s*NONE/gi,
  /EXEMPT\s+ACCESS\s+POLICY/gi,
  /AUTHID\s+DEFINER/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /DBMS_SCHEDULER\.CREATE_JOB/gi,
  /UTL_HTTP\.REQUEST/gi,
  /UTL_TCP/gi,
  /UTL_SMTP/gi,
  /DBMS_JAVA\.LOADJAVA/gi,
  /DBMS_SYS_SQL/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /SELECT\s+\*\s+FROM\s+\w+(?!\s+WHERE)/gi,
  /LOOP\s*;?\s*(?!EXIT\s+WHEN)/gi,
  /FOR\s+\w+\s+IN\s+1\s*\.\.\s*999999999/gi,
  /FETCH[\s\S]*?(?!LIMIT)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /CREATE\s+(?:OR\s+REPLACE\s+)?PACKAGE\s+\w+[\s\S]*?g_\w+/gi,
  /DBMS_SESSION\.SET_CONTEXT/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /(?:v_temp|v_data|v_var|v_x|v_y|l_temp)\s+(?:VARCHAR2|NUMBER|DATE)/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*--/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'PL/SQL: 硬編碼憑證', suggestion: '使用 Oracle Wallet 或 DBMS_CREDENTIAL' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'PL/SQL: 動態 SQL 拼接', suggestion: '使用綁定變量' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'PL/SQL: 刪除審計記錄', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'PL/SQL: 不當的異常處理', suggestion: '記錄並重新拋出異常' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'PL/SQL: 危險的權限設定', suggestion: '使用最小權限原則，AUTHID CURRENT_USER' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'PL/SQL: 危險的系統包', suggestion: '限制 UTL_* 包的使用' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'PL/SQL: 無限制查詢或循環', suggestion: '使用 ROWNUM/FETCH FIRST 和 EXIT WHEN' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'PL/SQL: 包級全局變量', suggestion: '使用局部變量或參數' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'PL/SQL: 不良變量名', suggestion: '使用有意義的命名' },
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

export const plsqlPlugin: LanguagePlugin = {
  name: 'plsql',
  extensions: ['.pls', '.plb', '.pks', '.pkb', '.pck', '.fnc', '.prc', '.trg'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default plsqlPlugin;
