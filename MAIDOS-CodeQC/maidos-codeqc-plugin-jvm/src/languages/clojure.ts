/**
 * Clojure Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const CLOJURE_CONFIG = {
  id: 'clojure',
  name: 'Clojure',
  extensions: ['.clj', '.cljs', '.cljc', '.edn'],
  lineComment: ';',
  blockComment: undefined, // Clojure 沒有區塊註解
};

// =============================================================================
// Clojure-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CLOJURE_CREDENTIAL_PATTERNS = [
  /:(?:password|passwd|pwd)\s+"(?!")[^"]{3,}"/gi,
  /:(?:api-key|apikey|api_key)\s+"[^"]{8,}"/gi,
  /:(?:secret|token)\s+"[^"]{8,}"/gi,
  /\(def\s+(?:password|secret|api-key)\s+"[^"]+"\)/gi,
];

/** R05: 忽略錯誤處理 */
const CLOJURE_ERROR_PATTERNS = [
  /\(catch\s+Exception\s+_\s*\)/g,  // (catch Exception _ )
  /\(catch\s+\w+\s+\w+\s*nil\)/g,   // (catch ... nil)
];

/** Clojure 特有：def 在函數內部（應用 let） */
const CLOJURE_INNER_DEF_PATTERN = /\(defn[^(]+\([^)]*\)\s*\(def\s/g;

/** Clojure 特有：atom/ref 過度使用 */
const CLOJURE_MUTABLE_PATTERN = /\((?:atom|ref|agent)\s/g;

/** P05: 超長函數 */
const CLOJURE_FUNCTION_PATTERN = /\(defn-?\s+(\S+)/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkClojureRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    // 跳過註解
    if (/^\s*;/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of CLOJURE_CREDENTIAL_PATTERNS) {
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
          message: `[Clojure] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 environ 或環境變數',
        });
        break;
      }
    }
  }

  // R05: 空 catch
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of CLOJURE_ERROR_PATTERNS) {
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
        message: `[Clojure] 靜默忽略異常`,
        snippet: match[0].substring(0, 50),
        suggestion: '正確處理異常或記錄錯誤',
      });
    }
  }

  // Clojure 特有：函數內部 def
  CLOJURE_INNER_DEF_PATTERN.lastIndex = 0;
  let innerDefMatch;
  while ((innerDefMatch = CLOJURE_INNER_DEF_PATTERN.exec(source)) !== null) {
    const beforeMatch = source.substring(0, innerDefMatch.index);
    const lineNum = (beforeMatch.match(/\n/g) || []).length + 1;
    
    violations.push({
      ruleId: 'P07',
      ruleName: '全局狀態',
      severity: 'warning',
      file,
      line: lineNum,
      column: 1,
      message: `[Clojure] 函數內部使用 def 創建全局變數`,
      suggestion: '使用 let 創建本地綁定',
    });
  }

  return violations;
}

export function checkClojureProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];

  // P07: 過度使用可變狀態
  let mutableCount = 0;
  CLOJURE_MUTABLE_PATTERN.lastIndex = 0;
  while (CLOJURE_MUTABLE_PATTERN.exec(source) !== null) {
    mutableCount++;
  }
  
  if (mutableCount > 5) {
    violations.push({
      ruleId: 'P07',
      ruleName: '全局狀態',
      severity: 'warning',
      file,
      line: 1,
      column: 1,
      message: `[Clojure] 過多可變狀態 (${mutableCount} 個 atom/ref/agent)`,
      suggestion: '優先使用純函數和不可變資料',
    });
  }

  // P05: 超長函數（基於括號配對）
  const lines = source.split('\n');
  let inFunction = false;
  let funcName = '';
  let funcStart = 0;
  let parenCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inFunction) {
      CLOJURE_FUNCTION_PATTERN.lastIndex = 0;
      const match = CLOJURE_FUNCTION_PATTERN.exec(line);
      if (match) {
        inFunction = true;
        funcName = match[1] || 'unknown';
        funcStart = i;
        parenCount = (line.match(/\(/g) || []).length - (line.match(/\)/g) || []).length;
      }
    } else {
      parenCount += (line.match(/\(/g) || []).length - (line.match(/\)/g) || []).length;
      if (parenCount <= 0) {
        const funcLength = i - funcStart + 1;
        if (funcLength > 50) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: funcStart + 1,
            column: 1,
            message: `[Clojure] 函數 "${funcName}" 長度 ${funcLength} 行 > 50`,
            suggestion: '拆分為多個小函數，使用 threading macros',
          });
        }
        inFunction = false;
      }
    }
  }

  return violations;
}

export function checkClojure(source: string, file: string): Violation[] {
  return [
    ...checkClojureRedlines(source, file),
    ...checkClojureProhibitions(source, file),
  ];
}
