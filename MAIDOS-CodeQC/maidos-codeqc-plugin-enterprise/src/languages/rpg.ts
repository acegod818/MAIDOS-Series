/**
 * RPG Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：RPG III, RPG IV (ILE RPG), RPG Free Format
 * 領域：IBM i (AS/400), 銀行核心系統、製造業 ERP
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// RPG-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:PASSWORD|SECRET|APIKEY)\s*=\s*['"][^'"]+['"]/gi,
  /DCL-C\s+(?:PASSWORD|SECRET|APIKEY)\s+['"][^'"]+['"]/gi,
  /CONST\s*\(\s*['"][^'"]+['"]\s*\)[\s\S]*?(?:PASSWORD|SECRET)/gi,
];

/** R02: 命令注入 */
const INJECTION_PATTERNS = [
  /QCMDEXC\s*\(/gi,
  /QCAPCMD\s*\(/gi,
  /SYSTEM\s*\(/gi,
  /EXEC\s+SQL[\s\S]*?EXECUTE\s+IMMEDIATE/gi,
];

/** R03: 刪除審計 */
const DELETE_AUDIT_PATTERNS = [
  /DELETE\s+FROM\s+(?:AUDIT|LOG|HISTORY)/gi,
  /QCMDEXC[\s\S]*?DLTF[\s\S]*?(?:AUDIT|LOG)/gi,
  /CLEAR\s+\*ALL[\s\S]*?(?:AUDIT|LOG)/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /MONITOR[\s\S]*?ON-ERROR[\s\S]*?ENDMON(?!\s*;)/gi,
  /\(E\)\s*$/gim,
  /SQLCOD\s*(?:<>|NE)\s*0[\s\S]*?(?:ITER|LEAVE)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /USRPRF\s*\(\s*\*OWNER\s*\)/gi,
  /ADOPT\s*\(\s*\*YES\s*\)/gi,
  /AUT\s*\(\s*\*ALL\s*\)/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /QCMDEXC[\s\S]*?(?:DLTLIB|CLRPFM|RSTOBJ)/gi,
  /PSSR\s+BEGSR/gi,
  /\*INLR\s*=\s*\*OFF/gi,
  /GOTO\s+/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /DOW\s+\*ON/gi,
  /DOU\s+\*OFF/gi,
  /SELECT\s+\*\s+FROM(?!\s+[\s\S]*?(?:FETCH\s+FIRST|WHERE))/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /DCL-S[\s\S]*?STATIC/gi,
  /S\s+\d+\s+\d+\s*$/gim,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /DCL-S\s+(?:TEMP|DATA|VAR|X|Y|WRK|W\d+)\s+/gi,
  /D\s+(?:TEMP|DATA|VAR|X|Y|WRK)\s+S\s+/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*(?:\*|\/\/)/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'RPG: 硬編碼憑證', suggestion: '使用 Data Area 或環境配置' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'RPG: 危險的命令執行', suggestion: '驗證輸入，避免動態 SQL' },
    { patterns: DELETE_AUDIT_PATTERNS, ruleId: 'R03', ruleName: '刪除審計日誌', severity: 'error', message: 'RPG: 刪除審計記錄', suggestion: '保留審計追蹤' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'RPG: 不當的錯誤處理', suggestion: '正確處理 MONITOR/ON-ERROR' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'RPG: 危險的權限設定', suggestion: '使用 *USER 而非 *OWNER' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'RPG: 危險操作', suggestion: '避免 GOTO 和 PSSR' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'RPG: 無限循環或查詢', suggestion: '添加退出條件和 FETCH FIRST' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'RPG: 靜態變量', suggestion: '使用局部變量和參數' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'RPG: 不良變量名', suggestion: '使用描述性命名' },
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

export const rpgPlugin: LanguagePlugin = {
  name: 'rpg',
  extensions: ['.rpg', '.rpgle', '.sqlrpgle', '.RPGLE', '.RPG'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default rpgPlugin;
