/**
 * F# Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const FSHARP_CONFIG = {
  id: 'fsharp',
  name: 'F#',
  extensions: ['.fs', '.fsi', '.fsx'],
  lineComment: '//',
  blockComment: { start: '(*', end: '*)' },
};

// =============================================================================
// F#-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const FSHARP_CREDENTIAL_PATTERNS = [
  /let\s+(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /let\s+(?:apiKey|api_key)\s*=\s*"[^"]{8,}"/gi,
  /let\s+(?:secret|token)\s*=\s*"[^"]{8,}"/gi,
];

/** R05: 忽略錯誤處理 */
const FSHARP_ERROR_PATTERNS = [
  /with\s*\|\s*_\s*->\s*\(\)/g,  // with | _ -> ()
  /\|\s*_\s*->\s*None/g,
];

/** F# 特有：mutable 使用 */
const FSHARP_MUTABLE_PATTERN = /let\s+mutable\s+\w+/g;

/** F# 特有：ignore 可能隱藏錯誤 */
const FSHARP_IGNORE_PATTERN = /\|>\s*ignore\b/g;

/** P05: 超長函數 */
const FSHARP_FUNCTION_PATTERN = /let\s+(?:rec\s+)?(\w+)\s+[^=]*=/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkFSharpRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (/^\s*(?:\/\/|\(\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of FSHARP_CREDENTIAL_PATTERNS) {
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
          message: `[F#] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用環境變數或配置提供者',
        });
        break;
      }
    }

    // R05: ignore 可能隱藏錯誤
    FSHARP_IGNORE_PATTERN.lastIndex = 0;
    const ignoreMatch = FSHARP_IGNORE_PATTERN.exec(line);
    if (ignoreMatch) {
      violations.push({
        ruleId: 'R05',
        ruleName: '忽略錯誤處理',
        severity: 'warning',
        file,
        line: i + 1,
        column: ignoreMatch.index + 1,
        message: `[F#] |> ignore 可能隱藏錯誤`,
        snippet: line.trim(),
        suggestion: '確認返回值是否需要處理',
      });
    }
  }

  // R05: 空的 exception handler
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of FSHARP_ERROR_PATTERNS) {
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
        message: `[F#] 靜默忽略異常`,
        snippet: match[0].substring(0, 50),
        suggestion: '正確處理異常或使用 Result 類型',
      });
    }
  }

  return violations;
}

export function checkFSharpProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: mutable 使用警告
  let mutableCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    FSHARP_MUTABLE_PATTERN.lastIndex = 0;
    if (FSHARP_MUTABLE_PATTERN.test(line)) {
      mutableCount++;
    }
  }
  
  if (mutableCount > 3) {
    violations.push({
      ruleId: 'P07',
      ruleName: '全局狀態',
      severity: 'warning',
      file,
      line: 1,
      column: 1,
      message: `[F#] 過多 mutable 綁定 (${mutableCount} 個)`,
      suggestion: '優先使用不可變綁定和函數式風格',
    });
  }

  // P05: 超長函數（基於縮排）
  let inFunction = false;
  let funcName = '';
  let funcStart = 0;
  let baseIndent = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    const trimmed = line.trim();
    if (trimmed === '') continue;
    
    const currentIndent = line.search(/\S/);
    
    if (!inFunction) {
      FSHARP_FUNCTION_PATTERN.lastIndex = 0;
      const match = FSHARP_FUNCTION_PATTERN.exec(line);
      if (match) {
        inFunction = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        baseIndent = currentIndent;
      }
    } else {
      // 當縮排回到或小於基準時，函數結束
      if (currentIndent <= baseIndent && !line.startsWith(' '.repeat(baseIndent + 1))) {
        const funcLength = i - funcStart;
        if (funcLength > 50) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: funcStart + 1,
            column: 1,
            message: `[F#] 函數 "${funcName}" 長度 ${funcLength} 行 > 50`,
            suggestion: '拆分為多個小函數，使用 composition',
          });
        }
        inFunction = false;
        // 重新檢查當前行是否是新函數
        i--;
      }
    }
  }

  return violations;
}

export function checkFSharp(source: string, file: string): Violation[] {
  return [
    ...checkFSharpRedlines(source, file),
    ...checkFSharpProhibitions(source, file),
  ];
}
