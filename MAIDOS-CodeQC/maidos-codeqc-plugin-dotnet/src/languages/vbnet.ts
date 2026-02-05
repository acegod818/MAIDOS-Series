/**
 * VB.NET Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const VBNET_CONFIG = {
  id: 'vbnet',
  name: 'VB.NET',
  extensions: ['.vb'],
  lineComment: "'",
  blockComment: undefined,
};

// =============================================================================
// VB.NET-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const VBNET_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:ApiKey|api_key)\s*=\s*"[^"]{8,}"/gi,
  /(?:ConnectionString)\s*=\s*"[^"]*(?:Password|pwd)=[^"]+"/gi,
];

/** R05: 忽略錯誤處理 */
const VBNET_ERROR_PATTERNS = [
  /Catch\s*\n\s*End\s+Try/gi,
  /Catch\s+\w+\s+As\s+Exception\s*\n\s*End\s+Try/gi,
  /On\s+Error\s+Resume\s+Next/gi,  // 經典 VB 錯誤處理（危險）
];

/** VB.NET 特有：Option Strict Off */
const VBNET_OPTION_STRICT_OFF = /Option\s+Strict\s+Off/gi;

/** P05: 超長方法 */
const VBNET_METHOD_PATTERN = /(?:Public|Private|Protected|Friend|Shared|\s)+(?:Sub|Function)\s+(\w+)\s*\(/gi;

// =============================================================================
// Checkers
// =============================================================================

export function checkVBNetRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    // 跳過註解
    if (/^\s*'/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of VBNET_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01',
          ruleName: '硬編碼憑證',
          severity: 'error',
          file,
          line: i + 1,
          column: match.index + 1,
          message: `[VB.NET] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 ConfigurationManager 或環境變數',
        });
        break;
      }
    }

    // VB.NET 特有：On Error Resume Next
    if (/On\s+Error\s+Resume\s+Next/i.test(line)) {
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'error',
        file,
        line: i + 1,
        column: 1,
        message: `[VB.NET] 使用 On Error Resume Next 靜默忽略所有錯誤`,
        snippet: line.trim(),
        suggestion: '使用 Try...Catch 結構化異常處理',
      });
    }

    // VB.NET 特有：Option Strict Off
    VBNET_OPTION_STRICT_OFF.lastIndex = 0;
    if (VBNET_OPTION_STRICT_OFF.test(line)) {
      violations.push({
        ruleId: 'R02',
        ruleName: '跳過安全檢查',
        severity: 'warning',
        file,
        line: i + 1,
        column: 1,
        message: `[VB.NET] Option Strict Off 允許隱式類型轉換`,
        snippet: line.trim(),
        suggestion: '使用 Option Strict On 增加類型安全',
      });
    }
  }

  // R05: 空 Catch
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!.trim().toLowerCase();
    if (line.startsWith('catch') && i + 1 < lines.length) {
      const nextLine = lines[i + 1]!.trim().toLowerCase();
      if (nextLine === 'end try' || nextLine.startsWith("'")) {
        violations.push({
          ruleId: 'R05',
          ruleName: '忽略錯誤處理',
          severity: 'error',
          file,
          line: i + 1,
          column: 1,
          message: `[VB.NET] 空的 Catch 區塊`,
          suggestion: '記錄異常或重新拋出',
        });
      }
    }
  }

  return violations;
}

export function checkVBNetProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長方法
  let inMethod = false;
  let methodName = '';
  let methodStart = 0;
  let methodType = ''; // Sub or Function

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inMethod) {
      VBNET_METHOD_PATTERN.lastIndex = 0;
      const match = VBNET_METHOD_PATTERN.exec(line);
      if (match) {
        inMethod = true;
        methodName = match[1] || 'unknown';
        methodStart = i;
        methodType = /\bSub\b/i.test(line) ? 'Sub' : 'Function';
      }
    } else {
      // 檢查方法結束
      const endPattern = new RegExp(`End\\s+${methodType}`, 'i');
      if (endPattern.test(line)) {
        const methodLength = i - methodStart + 1;
        if (methodLength > 50) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: methodStart + 1,
            column: 1,
            message: `[VB.NET] ${methodType} "${methodName}" 長度 ${methodLength} 行 > 50`,
            suggestion: '拆分為多個小方法',
          });
        }
        inMethod = false;
      }
    }
  }

  return violations;
}

export function checkVBNet(source: string, file: string): Violation[] {
  return [
    ...checkVBNetRedlines(source, file),
    ...checkVBNetProhibitions(source, file),
  ];
}
