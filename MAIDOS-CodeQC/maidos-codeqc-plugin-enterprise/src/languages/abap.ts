/**
 * ABAP Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：SAP ABAP, ABAP Objects, SAP Fiori
 * 領域：ERP, SAP S/4HANA, 企業資源規劃
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// ABAP-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:lv_password|lv_secret|lv_api_key)\s*=\s*['"][^'"]+['"]/gi,
  /DATA\s*:\s*(?:password|secret|api_key)\s+TYPE\s+\w+\s+VALUE\s+['"][^'"]+['"]/gi,
  /CONSTANTS\s*:\s*(?:c_password|c_secret)\s+TYPE\s+\w+\s+VALUE\s+['"][^'"]+['"]/gi,
];

/** R02: SQL 注入 / 命令注入 */
const INJECTION_PATTERNS = [
  /EXEC\s+SQL[\s\S]*?\([\s\S]*?\)/gi,
  /CALL\s+FUNCTION\s+['"]RFC_READ_TABLE['"]/gi,
  /GENERATE\s+SUBROUTINE\s+POOL/gi,
  /INSERT\s+REPORT\s+FROM/gi,
  /CALL\s+TRANSACTION\s+\w+\s+USING\s+\w+\s+MODE\s+['"]N['"]/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /DELETE\s+FROM\s+(?:CDHDR|CDPOS|BAL_LOG|SLG1)/gi,
  /CALL\s+FUNCTION\s+['"]CHANGEDOCUMENT_DELETE['"]/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /CATCH\s+cx_root\s+INTO\s+\w+\s*\.\s*(?:ENDTRY|")/gi,
  /CATCH\s+SYSTEM-EXCEPTIONS[\s\S]*?ENDCATCH\s*\./gi,
  /TRY\s*\.[\s\S]*?CATCH[\s\S]*?ENDTRY\s*\./gi,
  /sy-subrc\s*(?:NE|<>)\s*0[\s\S]*?(?:CONTINUE|EXIT)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /AUTHORITY-CHECK[\s\S]*?DUMMY/gi,
  /NO\s+AUTHORITY-CHECK/gi,
  /cl_http_client[\s\S]*?ssl_id\s*=\s*['"]ANONYM['"]/gi,
  /SET\s+UPDATE\s+TASK\s+LOCAL/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /CALL\s+FUNCTION\s+['"](?:SXPG_COMMAND_EXECUTE|RFC_REMOTE_EXEC)['"]/gi,
  /SUBMIT\s+\w+\s+VIA\s+JOB/gi,
  /DELETE\s+REPORT/gi,
  /GENERATE\s+REPORT/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /SELECT\s+\*\s+FROM\s+\w+(?!\s+(?:WHERE|UP\s+TO))/gi,
  /DO\s*\.[\s\S]*?ENDDO(?!\s+WHILE)/gi,
  /WHILE\s+abap_true/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /DATA\s+\w+\s+TYPE\s+\w+\s+COMMON\s+PART/gi,
  /STATICS\s*:/gi,
  /CLASS-DATA\s*:/gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /DATA\s*:\s*(?:lv_temp|lv_data|lv_var|lv_x|lv_y)\s+TYPE/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*[*"]/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'ABAP: 硬編碼憑證', suggestion: '使用 SSF 或 Secure Store' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'ABAP: 危險的動態代碼', suggestion: '使用 Open SQL 和標準 BAPI' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'ABAP: 刪除變更文檔', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'ABAP: 不當的異常處理', suggestion: '正確處理 sy-subrc 和異常' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'ABAP: 繞過權限檢查', suggestion: '始終執行 AUTHORITY-CHECK' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'ABAP: 危險函數調用', suggestion: '使用安全的替代方案' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'ABAP: 無限制查詢或循環', suggestion: '使用 UP TO n ROWS 和退出條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'ABAP: 全局/靜態數據', suggestion: '使用局部變量或實例屬性' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'ABAP: 不良變量名', suggestion: '遵循命名規範' },
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

export const abapPlugin: LanguagePlugin = {
  name: 'abap',
  extensions: ['.abap', '.abs', '.abl'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default abapPlugin;
