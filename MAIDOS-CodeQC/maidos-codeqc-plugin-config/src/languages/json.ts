/**
 * JSON Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：package.json, tsconfig.json, .eslintrc, manifest.json
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// JSON-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /"(?:password|passwd|secret|api_?key|token|auth)":\s*"(?![\$\{])[^"]+"/gi,
  /"(?:DB_PASSWORD|API_SECRET|SECRET_KEY|PRIVATE_KEY)":\s*"[^"]+"/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /"(?:ssl|tls|verify|secure)":\s*false/gi,
  /"rejectUnauthorized":\s*false/gi,
  /"strictSSL":\s*false/gi,
];

/** R08: 危險配置 */
const DANGEROUS_PATTERNS = [
  /"scripts":\s*\{[^}]*"(?:pre|post)?install":\s*"[^"]*(?:curl|wget|rm\s+-rf|chmod\s+777)[^"]*"/gi,
  /"permissions":\s*\[\s*"<all_urls>"/gi,
];

/** P14: 依賴問題 */
const DEPENDENCY_PATTERNS = [
  /"dependencies":\s*\{[^}]*"\*"/gi,
  /"(?:dependencies|devDependencies)":\s*\{[^}]*"latest"/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'JSON: 硬編碼憑證', suggestion: '使用環境變數' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'JSON: 禁用安全功能', suggestion: '啟用 SSL/TLS 驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'JSON: 危險的 scripts 或權限', suggestion: '審查安裝腳本和權限' },
    { patterns: DEPENDENCY_PATTERNS, ruleId: 'P14', ruleName: '依賴膨脹', severity: 'warning', message: 'JSON: 不安全的版本範圍', suggestion: '使用固定版本號' },
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

export const jsonPlugin: LanguagePlugin = {
  name: 'json',
  extensions: ['.json', '.jsonc', '.json5'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default jsonPlugin;
