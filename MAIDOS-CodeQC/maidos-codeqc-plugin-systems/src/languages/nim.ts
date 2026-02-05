/**
 * Nim Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const NIM_CONFIG = {
  id: 'nim',
  name: 'Nim',
  extensions: ['.nim', '.nims'],
  lineComment: '#',
  blockComment: { start: '#[', end: ']#' },
};

/** R01: 硬編碼憑證 */
const NIM_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /const\s+(?:Password|Secret|ApiKey)\s*=\s*"[^"]+"/gi,
  /let\s+(?:password|secret|apiKey)\s*=\s*"[^"]+"/gi,
];

/** R05: 忽略異常 */
const NIM_ERROR_PATTERNS = [
  /except\s*:\s*discard/g,
  /except\s+\w+\s*:\s*discard/g,
];

/** Nim 特有：不安全的字串格式化 */
const NIM_FORMAT_PATTERN = /\bfmt\s*"[^"]*\{[^}]*\}.*"/g;

/** P05: 超長函數 */
const NIM_FUNCTION_PATTERN = /proc\s+(\w+)\s*(?:\*\s*)?\([^)]*\)/g;

export function checkNimRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*#/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of NIM_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Nim] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用環境變數 getEnv()',
        });
        break;
      }
    }

    // R05: except discard
    if (/except\s*.*:\s*discard/.test(line)) {
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: 1,
        message: `[Nim] except 區塊使用 discard 忽略異常`,
        snippet: line.trim(),
        suggestion: '正確處理異常或記錄錯誤',
      });
    }
  }

  return violations;
}

export function checkNimProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長函數（基於縮排）
  let inFunc = false, funcName = '', funcStart = 0, baseIndent = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    const trimmed = line.trim();
    if (trimmed === '' || trimmed.startsWith('#')) continue;
    
    const currentIndent = line.search(/\S/);
    
    if (!inFunc) {
      NIM_FUNCTION_PATTERN.lastIndex = 0;
      const match = NIM_FUNCTION_PATTERN.exec(line);
      if (match && line.includes('=')) {
        inFunc = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        baseIndent = currentIndent;
      }
    } else {
      if (currentIndent <= baseIndent && !trimmed.startsWith('#')) {
        const len = i - funcStart;
        if (len > 50) {
          violations.push({
            ruleId: 'P05', ruleName: '超長函數', severity: 'warning', file, line: funcStart + 1, column: 1,
            message: `[Nim] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數',
          });
        }
        inFunc = false;
        i--;  // 重新檢查當前行
      }
    }
  }

  return violations;
}

export function checkNim(source: string, file: string): Violation[] {
  return [...checkNimRedlines(source, file), ...checkNimProhibitions(source, file)];
}
