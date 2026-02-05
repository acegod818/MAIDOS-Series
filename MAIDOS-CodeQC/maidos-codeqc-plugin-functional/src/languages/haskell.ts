/**
 * Haskell Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：Haskell, GHC, Cabal, Stack, Yesod, Servant
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// Haskell-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|secret|apiKey|token)\s*=\s*["'][^"']+["']/gi,
  /getEnv\s+["'](?:PASSWORD|SECRET|API_KEY|TOKEN)["']\s*`catch`\s*const\s+["'][^"']+["']/gi,
];

/** R02: 不安全操作 */
const INJECTION_PATTERNS = [
  /unsafePerformIO/gi,
  /unsafeCoerce/gi,
  /unsafeDupablePerformIO/gi,
  /readProcess\s+\$/gi,
  /callProcess\s+\$/gi,
];

/** R05: 錯誤處理 */
const ERROR_HANDLING_PATTERNS = [
  /catch\s+\(\s*_\s*::\s*SomeException\s*\)/gi,
  /`catch`\s*const\s+return\s+\(\)/gi,
  /fromJust\s+/gi,
  /head\s+\$/gi,
  /tail\s+\$/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /tlsClientConfig.*checkCertificate\s*=\s*False/gi,
  /managerSettings.*managerCheckCert\s*=\s*\\?\s*_\s*_\s*_\s*->\s*return\s+\(\)/gi,
];

/** R08: 危險函數 */
const DANGEROUS_PATTERNS = [
  /\bhead\b(?!\s*\$\s*filter)/gi,
  /\btail\b(?!\s*\$\s*filter)/gi,
  /\bread\s+::/gi,
  /\berror\s+["']/gi,
  /\bundefined\b/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /\brepeat\b/gi,
  /\bcycle\b/gi,
  /\biterate\b(?!\s+.*take)/gi,
];

/** P07: 全局狀態 */
const GLOBAL_STATE_PATTERNS = [
  /\{-#\s*NOINLINE\s+\w+\s*#-\}[\s\S]*unsafePerformIO/gi,
  /globalMVar/gi,
  /globalIORef/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*--/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'Haskell: 硬編碼憑證', suggestion: '使用環境變數或配置' },
    { patterns: INJECTION_PATTERNS, ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', message: 'Haskell: 不安全操作', suggestion: '避免 unsafe* 函數' },
    { patterns: ERROR_HANDLING_PATTERNS, ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', message: 'Haskell: 部分函數使用', suggestion: '使用 Maybe/Either 和安全版本' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'Haskell: 禁用 TLS 驗證', suggestion: '啟用證書驗證' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'Haskell: 危險的部分函數', suggestion: '使用 headMay, tailMay 等安全版本' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'Haskell: 無限列表操作', suggestion: '確保有 take 限制' },
    { patterns: GLOBAL_STATE_PATTERNS, ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', message: 'Haskell: 全局可變狀態', suggestion: '使用 Reader/State monad' },
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

export const haskellPlugin: LanguagePlugin = {
  name: 'haskell',
  extensions: ['.hs', '.lhs', '.hsc'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default haskellPlugin;
