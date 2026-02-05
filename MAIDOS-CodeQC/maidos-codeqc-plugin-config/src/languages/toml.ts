/**
 * TOML Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Cargo.toml, pyproject.toml, Hugo config, Netlify
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// TOML-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token)\s*=\s*["'][^"'$]+["']/gi,
  /\[(?:credentials|secrets)\][^[]*\w+\s*=\s*["'][^"'$]+["']/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /ssl\s*=\s*false/gi,
  /verify\s*=\s*false/gi,
  /insecure\s*=\s*true/gi,
];

/** R08: 危險配置 */
const DANGEROUS_PATTERNS = [
  /build-override\s*=\s*true/gi,
  /\[profile\.release\][^[]*debug\s*=\s*true/gi,
];

/** P14: 依賴問題 */
const DEPENDENCY_PATTERNS = [
  /\[dependencies\][^[]*\w+\s*=\s*["']\*["']/gi,
  /version\s*=\s*["']\*["']/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'TOML: 硬編碼憑證', suggestion: '使用環境變數' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'TOML: 禁用安全功能', suggestion: '啟用安全驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'TOML: 危險配置', suggestion: '使用安全的配置' },
    { patterns: DEPENDENCY_PATTERNS, ruleId: 'P14', ruleName: '依賴膨脹', severity: 'warning', message: 'TOML: 不安全的版本', suggestion: '使用固定版本號' },
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

export const tomlPlugin: LanguagePlugin = {
  name: 'toml',
  extensions: ['.toml'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default tomlPlugin;
