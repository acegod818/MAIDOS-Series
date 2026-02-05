/**
 * C Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const C_CONFIG = {
  id: 'c',
  name: 'C',
  extensions: ['.c', '.h'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

/** R01: 硬編碼憑證 */
const C_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /#define\s+(?:PASSWORD|API_KEY|SECRET)\s+"[^"]+"/gi,
  /char\s+(?:\*\s*)?(?:password|secret|key)\s*\[\s*\]\s*=\s*"[^"]+"/gi,
];

/** R02: 不安全函數 */
const C_UNSAFE_FUNCTIONS = [
  { pattern: /\bgets\s*\(/g, name: 'gets', suggestion: 'fgets' },
  { pattern: /\bstrcpy\s*\(/g, name: 'strcpy', suggestion: 'strncpy/strlcpy' },
  { pattern: /\bstrcat\s*\(/g, name: 'strcat', suggestion: 'strncat/strlcat' },
  { pattern: /\bsprintf\s*\(/g, name: 'sprintf', suggestion: 'snprintf' },
  { pattern: /\bscanf\s*\(\s*"[^"]*%s/g, name: 'scanf %s', suggestion: '限制長度 %Ns' },
  { pattern: /\batoi\s*\(/g, name: 'atoi', suggestion: 'strtol' },
  { pattern: /\batof\s*\(/g, name: 'atof', suggestion: 'strtod' },
];

/** R05: 未檢查返回值 */
const C_UNCHECKED_PATTERNS = [
  /\bmalloc\s*\([^)]+\)\s*;/g,  // malloc 未檢查
  /\bcalloc\s*\([^)]+\)\s*;/g,
  /\brealloc\s*\([^)]+\)\s*;/g,
];

/** R09: 記憶體洩漏風險 */
const C_MEMORY_PATTERNS = [
  /\bmalloc\s*\(/g,
  /\bcalloc\s*\(/g,
];

/** P05: 超長函數 */
const C_FUNCTION_PATTERN = /^(?:\w+\s+)+(\w+)\s*\([^)]*\)\s*\{/gm;

export function checkCRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of C_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[C] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用環境變數或配置檔案',
        });
        break;
      }
    }

    // R02: 不安全函數
    for (const { pattern, name, suggestion } of C_UNSAFE_FUNCTIONS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[C] 不安全函數 ${name}`,
          snippet: line.trim(),
          suggestion: `使用 ${suggestion}`,
        });
      }
    }

    // R05: 未檢查 malloc 返回值
    for (const pattern of C_UNCHECKED_PATTERNS) {
      pattern.lastIndex = 0;
      if (pattern.test(line) && !/if\s*\(/.test(lines[i - 1] || '') && !/=\s*NULL/.test(lines[i + 1] || '')) {
        violations.push({
          ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: 1,
          message: `[C] 未檢查記憶體分配返回值`,
          snippet: line.trim(),
          suggestion: '檢查是否為 NULL',
        });
      }
    }
  }

  return violations;
}

export function checkCProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長函數
  let inFunc = false, funcName = '', funcStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inFunc) {
      C_FUNCTION_PATTERN.lastIndex = 0;
      const match = C_FUNCTION_PATTERN.exec(line);
      if (match) {
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
            message: `[C] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數',
          });
        }
        inFunc = false;
      }
    }
  }

  // P06: 深層嵌套
  let maxNesting = 0, maxLine = 0, current = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    current += (line.match(/\{/g) || []).length;
    if (current > maxNesting) { maxNesting = current; maxLine = i + 1; }
    current -= (line.match(/\}/g) || []).length;
    if (current < 0) current = 0;
  }
  if (maxNesting > 4) {
    violations.push({
      ruleId: 'P06', ruleName: '深層嵌套', severity: 'warning', file, line: maxLine, column: 1,
      message: `[C] 最大嵌套 ${maxNesting} 層 > 4`,
      suggestion: '使用早返回或提取子函數',
    });
  }

  return violations;
}

export function checkC(source: string, file: string): Violation[] {
  return [...checkCRedlines(source, file), ...checkCProhibitions(source, file)];
}
