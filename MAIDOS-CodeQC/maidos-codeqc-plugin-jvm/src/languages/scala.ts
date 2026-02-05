/**
 * Scala Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const SCALA_CONFIG = {
  id: 'scala',
  name: 'Scala',
  extensions: ['.scala', '.sc'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

// =============================================================================
// Scala-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const SCALA_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:apiKey|api_key)\s*=\s*"[^"]{8,}"/gi,
  /(?:secret|token)\s*=\s*"[^"]{8,}"/gi,
];

/** R05: 忽略錯誤處理 */
const SCALA_ERROR_PATTERNS = [
  /catch\s*\{[^}]*case\s+_\s*=>\s*\}/g,  // catch { case _ => }
  /\.getOrElse\s*\(\s*null\s*\)/g,
  /Try\s*\{[^}]+\}\.toOption\.getOrElse/g,
];

/** Scala 特有：var 可變變數 */
const SCALA_VAR_PATTERN = /\bvar\s+\w+/g;

/** Scala 特有：null 使用 */
const SCALA_NULL_PATTERN = /(?:=\s*null\b|\bnull\b(?!\s*\)))/g;

/** P05: 超長函數 */
const SCALA_FUNCTION_PATTERN = /\bdef\s+(\w+)\s*(?:\[[^\]]+\])?\s*\([^)]*\)/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkScalaRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of SCALA_CREDENTIAL_PATTERNS) {
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
          message: `[Scala] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 Typesafe Config 或環境變數',
        });
        break;
      }
    }

    // Scala 特有：null 使用
    SCALA_NULL_PATTERN.lastIndex = 0;
    const nullMatch = SCALA_NULL_PATTERN.exec(line);
    if (nullMatch) {
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'warning',
        file,
        line: i + 1,
        column: nullMatch.index + 1,
        message: `[Scala] 使用 null 不符合 Scala 慣例`,
        snippet: line.trim(),
        suggestion: '使用 Option[T] 代替 null',
      });
    }
  }

  return violations;
}

export function checkScalaProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: var 可變變數警告
  let varCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;
    
    SCALA_VAR_PATTERN.lastIndex = 0;
    if (SCALA_VAR_PATTERN.test(line)) {
      varCount++;
    }
  }
  
  if (varCount > 3) {
    violations.push({
      ruleId: 'P07',
      ruleName: '全局狀態',
      severity: 'warning',
      file,
      line: 1,
      column: 1,
      message: `[Scala] 過多 var 可變變數 (${varCount} 個)`,
      suggestion: '優先使用 val 不可變變數',
    });
  }

  // P05: 超長函數
  let inFunction = false;
  let funcName = '';
  let funcStart = 0;
  let braceCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inFunction) {
      SCALA_FUNCTION_PATTERN.lastIndex = 0;
      const match = SCALA_FUNCTION_PATTERN.exec(line);
      if (match && (line.includes('{') || line.includes('='))) {
        inFunction = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
        // Scala 單行函數
        if (braceCount === 0 && !line.includes('{')) {
          inFunction = false;
        }
      }
    } else {
      braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      if (braceCount <= 0) {
        const funcLength = i - funcStart + 1;
        if (funcLength > 50) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: funcStart + 1,
            column: 1,
            message: `[Scala] 函數 "${funcName}" 長度 ${funcLength} 行 > 50`,
            suggestion: '拆分為多個小函數，利用 for-comprehension',
          });
        }
        inFunction = false;
      }
    }
  }

  return violations;
}

export function checkScala(source: string, file: string): Violation[] {
  return [
    ...checkScalaRedlines(source, file),
    ...checkScalaProhibitions(source, file),
  ];
}
