/**
 * Zig Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const ZIG_CONFIG = {
  id: 'zig',
  name: 'Zig',
  extensions: ['.zig'],
  lineComment: '//',
  blockComment: undefined,
};

/** R01: 硬編碼憑證 */
const ZIG_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /const\s+(?:password|secret|api_key)\s*=\s*"[^"]+"/gi,
];

/** R05: 忽略錯誤 */
const ZIG_ERROR_PATTERNS = [
  /catch\s*\|_\|\s*\{\s*\}/g,  // catch |_| {}
  /catch\s*\{\s*\}/g,
  /\btry\b[^;]+\s+catch\s*\|_\|\s*undefined/g,
];

/** Zig 特有：unreachable 使用 */
const ZIG_UNREACHABLE = /\bunreachable\b/g;

/** P05: 超長函數 */
const ZIG_FUNCTION_PATTERN = /(?:pub\s+)?fn\s+(\w+)\s*\([^)]*\)/g;

export function checkZigRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*\/\//.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of ZIG_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Zig] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 @embedFile 讀取配置或環境變數',
        });
        break;
      }
    }

    // Zig 特有：unreachable
    ZIG_UNREACHABLE.lastIndex = 0;
    if (ZIG_UNREACHABLE.test(line)) {
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'warning', file, line: i + 1, column: 1,
        message: `[Zig] unreachable 如果被執行會導致 panic`,
        snippet: line.trim(),
        suggestion: '確保邏輯正確或使用 @panic 提供訊息',
      });
    }
  }

  // R05: 忽略錯誤
  const normalized = source.replace(/\n\s*/g, ' ');
  for (const pattern of ZIG_ERROR_PATTERNS) {
    pattern.lastIndex = 0;
    let match;
    while ((match = pattern.exec(normalized)) !== null) {
      const before = source.substring(0, source.indexOf(match[0]));
      const lineNum = (before.match(/\n/g) || []).length + 1;
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: lineNum, column: 1,
        message: `[Zig] 忽略錯誤`,
        suggestion: '正確處理錯誤或使用 catch unreachable',
      });
    }
  }

  return violations;
}

export function checkZigProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長函數
  let inFunc = false, funcName = '', funcStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inFunc) {
      ZIG_FUNCTION_PATTERN.lastIndex = 0;
      const match = ZIG_FUNCTION_PATTERN.exec(line);
      if (match && line.includes('{')) {
        inFunc = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      }
    } else {
      braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      if (braceCount <= 0) {
        const len = i - funcStart + 1;
        if (len > 50) {
          violations.push({
            ruleId: 'P05', ruleName: '超長函數', severity: 'warning', file, line: funcStart + 1, column: 1,
            message: `[Zig] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數',
          });
        }
        inFunc = false;
      }
    }
  }

  return violations;
}

export function checkZig(source: string, file: string): Violation[] {
  return [...checkZigRedlines(source, file), ...checkZigProhibitions(source, file)];
}
