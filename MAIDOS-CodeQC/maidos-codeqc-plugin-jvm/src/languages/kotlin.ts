/**
 * Kotlin Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const KOTLIN_CONFIG = {
  id: 'kotlin',
  name: 'Kotlin',
  extensions: ['.kt', '.kts'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

// =============================================================================
// Kotlin-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const KOTLIN_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:apiKey|api_key)\s*=\s*"[^"]{8,}"/gi,
  /(?:secret|token)\s*=\s*"[^"]{8,}"/gi,
];

/** R05: 忽略錯誤處理 */
const KOTLIN_ERROR_PATTERNS = [
  /catch\s*\([^)]+\)\s*\{\s*\}/g,
  /\.getOrNull\(\)/g,  // 靜默忽略錯誤
  /runCatching\s*\{[^}]+\}\.getOrNull\(\)/g,
];

/** Kotlin 特有：!! 強制解包 */
const KOTLIN_FORCE_UNWRAP = /\w+!!/g;

/** Kotlin 特有：lateinit 未初始化風險 */
const KOTLIN_LATEINIT_PATTERN = /lateinit\s+var\s+(\w+)/g;

/** P05: 超長函數 */
const KOTLIN_FUNCTION_PATTERN = /(?:fun|suspend\s+fun)\s+(\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkKotlinRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of KOTLIN_CREDENTIAL_PATTERNS) {
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
          message: `[Kotlin] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 BuildConfig 或環境變數',
        });
        break;
      }
    }

    // R05: !! 強制解包
    KOTLIN_FORCE_UNWRAP.lastIndex = 0;
    const unwrapMatch = KOTLIN_FORCE_UNWRAP.exec(line);
    if (unwrapMatch) {
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'error',
        file,
        line: i + 1,
        column: unwrapMatch.index + 1,
        message: `[Kotlin] 使用 !! 強制解包可能導致 NPE`,
        snippet: line.trim(),
        suggestion: '使用 ?. 安全調用或 ?: Elvis 運算符',
      });
    }
  }

  // 空 catch
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of KOTLIN_ERROR_PATTERNS) {
    pattern.lastIndex = 0;
    let match;
    while ((match = pattern.exec(normalizedSource)) !== null) {
      const beforeMatch = source.substring(0, source.indexOf(match[0]));
      const lineNum = (beforeMatch.match(/\n/g) || []).length + 1;
      
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'error',
        file,
        line: lineNum,
        column: 1,
        message: `[Kotlin] 靜默忽略錯誤`,
        snippet: match[0].substring(0, 50),
        suggestion: '正確處理錯誤或使用 getOrElse',
      });
    }
  }

  return violations;
}

export function checkKotlinProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // lateinit 使用警告
  let lateinitCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    KOTLIN_LATEINIT_PATTERN.lastIndex = 0;
    if (KOTLIN_LATEINIT_PATTERN.test(line)) {
      lateinitCount++;
    }
  }
  
  if (lateinitCount > 5) {
    violations.push({
      ruleId: 'P07',
      ruleName: '全局狀態',
      severity: 'warning',
      file,
      line: 1,
      column: 1,
      message: `[Kotlin] 過多 lateinit 變數 (${lateinitCount} 個)`,
      suggestion: '考慮使用 by lazy 或依賴注入',
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
      KOTLIN_FUNCTION_PATTERN.lastIndex = 0;
      const match = KOTLIN_FUNCTION_PATTERN.exec(line);
      if (match && line.includes('{')) {
        inFunction = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
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
            message: `[Kotlin] 函數 "${funcName}" 長度 ${funcLength} 行 > 50`,
            suggestion: '拆分為多個小函數或使用擴展函數',
          });
        }
        inFunction = false;
      }
    }
  }

  return violations;
}

export function checkKotlin(source: string, file: string): Violation[] {
  return [
    ...checkKotlinRedlines(source, file),
    ...checkKotlinProhibitions(source, file),
  ];
}
