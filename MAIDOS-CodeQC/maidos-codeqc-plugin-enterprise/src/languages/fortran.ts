/**
 * Fortran Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Fortran 77, Fortran 90/95, Fortran 2003/2008/2018
 * 領域：科學計算、數值模擬、HPC、氣象、航太
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Fortran-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|secret|api_key)\s*=\s*['"][^'"]+['"]/gi,
  /CHARACTER[\s\S]*?(?:password|secret)[\s\S]*?=\s*['"][^'"]+['"]/gi,
];

/** R02: 命令注入 */
const INJECTION_PATTERNS = [
  /CALL\s+SYSTEM\s*\(/gi,
  /CALL\s+EXECUTE_COMMAND_LINE\s*\(/gi,
  /EXECUTE_COMMAND_LINE\s*\(/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /OPEN\s*\([^)]*(?!ERR\s*=)/gi,
  /READ\s*\([^)]*(?!ERR\s*=|IOSTAT\s*=)/gi,
  /WRITE\s*\([^)]*(?!ERR\s*=|IOSTAT\s*=)/gi,
  /ALLOCATE\s*\([^)]*(?!STAT\s*=)/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /IMPLICIT\s+NONE/gi,
  /!\$OMP\s+PARALLEL[\s\S]*?(?!PRIVATE)/gi,
];

/** R08: 危險操作 */
const DANGEROUS_PATTERNS = [
  /EQUIVALENCE\s*\(/gi,
  /COMMON\s+\/\w+\//gi,
  /ENTRY\s+\w+/gi,
  /GOTO\s+\d+/gi,
  /COMPUTED\s+GOTO/gi,
  /ARITHMETIC\s+IF/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /DO\s+WHILE\s*\(\s*\.TRUE\.\s*\)/gi,
  /DO\s*$/gim,
  /DO\s+\w+\s*=\s*1\s*,\s*HUGE/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /COMMON\s+\/\w+\//gi,
  /SAVE\s+/gi,
  /DATA\s+\w+\s*\//gi,
];

/** P09: 不良命名 */
const BAD_NAMING_PATTERNS = [
  /(?:INTEGER|REAL|DOUBLE|CHARACTER)[\s\S]*?::\s*(?:temp|data|var|x|y|z|i|j|k|n|m)\s*(?:,|$|\()/gim,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*[!cC*]/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Fortran: 硬編碼憑證', suggestion: '使用環境變數或配置檔' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Fortran: 系統命令調用', suggestion: '驗證輸入參數' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Fortran: 缺少錯誤處理', suggestion: '使用 IOSTAT/STAT/ERR 參數' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Fortran: 過時或危險語法', suggestion: '使用現代 Fortran 結構' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Fortran: 無限循環', suggestion: '添加退出條件' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Fortran: COMMON 塊或 SAVE', suggestion: '使用 MODULE 和參數傳遞' },
    { patterns: BAD_NAMING_PATTERNS, ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', message: 'Fortran: 單字母變量名', suggestion: '使用描述性名稱' },
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

export const fortranPlugin: LanguagePlugin = {
  name: 'fortran',
  extensions: ['.f', '.f90', '.f95', '.f03', '.f08', '.for', '.fpp', '.F', '.F90'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default fortranPlugin;
