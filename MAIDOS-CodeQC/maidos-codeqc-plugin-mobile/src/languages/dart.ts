/**
 * Dart Language Support (Flutter)
 */

import type { Violation } from '@maidos/codeqc';

export const DART_CONFIG = {
  id: 'dart',
  name: 'Dart',
  extensions: ['.dart'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

/** R01: 硬編碼憑證 */
const DART_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*['"](?!['"])[^'"]{3,}['"]/gi,
  /(?:apiKey|api_key)\s*=\s*['"][^'"]{8,}['"]/gi,
  /(?:secret|token)\s*=\s*['"][^'"]{8,}['"]/gi,
];

/** R05: 忽略錯誤處理 */
const DART_ERROR_PATTERNS = [
  /catch\s*\([^)]*\)\s*\{\s*\}/g,
  /\.catchError\s*\(\s*\([^)]*\)\s*\{\s*\}\s*\)/g,
  /\.catchError\s*\(\s*\([^)]*\)\s*=>\s*null\s*\)/g,
];

/** Dart 特有：強制解包 */
const DART_FORCE_UNWRAP = /\w+!/g;

/** Dart 特有：dynamic 過度使用 */
const DART_DYNAMIC_PATTERN = /:\s*dynamic\b|dynamic\s+\w+/g;

/** Dart 特有：print 用於生產（應使用 logger） */
const DART_PRINT_PATTERN = /\bprint\s*\(/g;

/** P05: 超長函數 */
const DART_FUNCTION_PATTERN = /(?:void|Future|Stream|[A-Z]\w*)\s+(\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)\s*(?:async\s*)?\{/g;

export function checkDartRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of DART_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Dart] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 flutter_dotenv 或 --dart-define',
        });
        break;
      }
    }

    // R05: 強制解包
    DART_FORCE_UNWRAP.lastIndex = 0;
    const unwrapMatch = DART_FORCE_UNWRAP.exec(line);
    if (unwrapMatch && !/^\s*\/\//.test(line)) {
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: unwrapMatch.index + 1,
        message: `[Dart] 強制解包 (!) 可能導致運行時錯誤`,
        snippet: line.trim(),
        suggestion: '使用 ?. 或 ?? 安全存取',
      });
    }
  }

  // R05: 空 catch
  const normalized = source.replace(/\n\s*/g, ' ');
  for (const pattern of DART_ERROR_PATTERNS) {
    pattern.lastIndex = 0;
    let match;
    while ((match = pattern.exec(normalized)) !== null) {
      const before = source.substring(0, source.indexOf(match[0]));
      const lineNum = (before.match(/\n/g) || []).length + 1;
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: lineNum, column: 1,
        message: `[Dart] 空的 catch/catchError`,
        suggestion: '正確處理錯誤或記錄',
      });
    }
  }

  return violations;
}

export function checkDartProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: dynamic 過度使用
  let dynamicCount = 0;
  for (const line of lines) {
    DART_DYNAMIC_PATTERN.lastIndex = 0;
    while (DART_DYNAMIC_PATTERN.exec(line)) dynamicCount++;
  }
  if (dynamicCount > 5) {
    violations.push({
      ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', file, line: 1, column: 1,
      message: `[Dart] 過多 dynamic 類型 (${dynamicCount} 個)`,
      suggestion: '使用具體類型或泛型',
    });
  }

  // 警告：生產代碼中的 print
  if (!file.includes('_test.dart') && !file.includes('test/')) {
    let printCount = 0;
    for (const line of lines) {
      DART_PRINT_PATTERN.lastIndex = 0;
      if (DART_PRINT_PATTERN.test(line)) printCount++;
    }
    if (printCount > 3) {
      violations.push({
        ruleId: 'P14', ruleName: '依賴膨脹', severity: 'info', file, line: 1, column: 1,
        message: `[Dart] 生產代碼中有 ${printCount} 個 print()`,
        suggestion: '使用 logger 套件如 logging 或 logger',
      });
    }
  }

  // P05: 超長函數
  let inFunc = false, funcName = '', funcStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inFunc) {
      DART_FUNCTION_PATTERN.lastIndex = 0;
      const match = DART_FUNCTION_PATTERN.exec(line);
      if (match) {
        inFunc = true;
        funcName = match[1] || 'anonymous';
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
            message: `[Dart] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數或 Widget',
          });
        }
        inFunc = false;
      }
    }
  }

  return violations;
}

export function checkDart(source: string, file: string): Violation[] {
  return [...checkDartRedlines(source, file), ...checkDartProhibitions(source, file)];
}
