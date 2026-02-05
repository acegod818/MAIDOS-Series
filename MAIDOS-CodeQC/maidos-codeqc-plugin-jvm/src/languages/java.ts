/**
 * Java Language Support
 */

import type { Violation, RuleChecker } from '@maidos/codeqc';

export const JAVA_CONFIG = {
  id: 'java',
  name: 'Java',
  extensions: ['.java'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

// =============================================================================
// Java-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 - Java 特化 */
const JAVA_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:apiKey|api_key|API_KEY)\s*=\s*"[^"]{8,}"/gi,
  /(?:secret|token|auth)\s*=\s*"[^"]{8,}"/gi,
  /new\s+String\s*\(\s*"(?:password|secret|key)[^"]*"\s*\)/gi,
];

/** R05: 忽略錯誤處理 - Java 特化 */
const JAVA_EMPTY_CATCH_PATTERNS = [
  /catch\s*\([^)]+\)\s*\{\s*\}/g,
  /catch\s*\([^)]+\)\s*\{\s*\/\/\s*(?:ignore|ignored|nothing|empty)?\s*\}/gi,
  /catch\s*\([^)]+\)\s*\{\s*\/\*[^*]*\*\/\s*\}/g,
];

/** P05: 超長方法 */
const JAVA_METHOD_PATTERN = /(?:public|private|protected|static|\s)+\s+\w+\s+(\w+)\s*\([^)]*\)\s*(?:throws\s+[^{]+)?\s*\{/g;

/** P07: 全局狀態 - Java static 變數 */
const JAVA_STATIC_MUTABLE_PATTERN = /(?:public|protected)\s+static\s+(?!final)[^=;]+=[^;]+;/g;

/** Java 特有：資源未關閉 */
const JAVA_UNCLOSED_RESOURCE_PATTERNS = [
  /new\s+(?:FileInputStream|FileOutputStream|BufferedReader|BufferedWriter|Connection|Statement|ResultSet)\s*\([^)]*\)(?![^;]*try)/g,
  /\.getConnection\s*\([^)]*\)(?![^;]*try)/g,
];

/** Java 特有：不安全的反序列化 */
const JAVA_UNSAFE_DESERIALIZATION = [
  /new\s+ObjectInputStream\s*\(/g,
  /readObject\s*\(\s*\)/g,
  /XMLDecoder\s*\(/g,
];

// =============================================================================
// Checkers
// =============================================================================

export function checkJavaRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    // 跳過註解
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of JAVA_CREDENTIAL_PATTERNS) {
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
          message: `[Java] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用環境變數或配置檔案（如 application.properties）',
        });
        break;
      }
    }

    // Java 特有：資源未關閉
    for (const pattern of JAVA_UNCLOSED_RESOURCE_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R05',
          ruleName: '忽略錯誤處理',
          severity: 'error',
          file,
          line: i + 1,
          column: match.index + 1,
          message: `[Java] 資源未使用 try-with-resources，可能導致資源洩漏`,
          snippet: line.trim(),
          suggestion: '使用 try-with-resources 自動關閉資源',
        });
        break;
      }
    }

    // Java 特有：不安全的反序列化
    for (const pattern of JAVA_UNSAFE_DESERIALIZATION) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R02',
          ruleName: '跳過安全檢查',
          severity: 'error',
          file,
          line: i + 1,
          column: match.index + 1,
          message: `[Java] 不安全的反序列化操作`,
          snippet: line.trim(),
          suggestion: '避免反序列化不受信任的資料，使用 JSON 等安全格式',
        });
        break;
      }
    }
  }

  // R05: 空 catch
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of JAVA_EMPTY_CATCH_PATTERNS) {
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
        message: `[Java] 空的 catch 區塊`,
        snippet: match[0].substring(0, 50),
        suggestion: '至少記錄異常或重新拋出',
      });
    }
  }

  return violations;
}

export function checkJavaProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: 公開的可變靜態變數
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    JAVA_STATIC_MUTABLE_PATTERN.lastIndex = 0;
    const match = JAVA_STATIC_MUTABLE_PATTERN.exec(line);
    if (match) {
      violations.push({
        ruleId: 'P07',
        ruleName: '全局狀態',
        severity: 'warning',
        file,
        line: i + 1,
        column: 1,
        message: `[Java] 公開的可變靜態變數`,
        snippet: line.trim(),
        suggestion: '使用 private static final 或依賴注入',
      });
    }
  }

  // P05: 超長方法（簡化版）
  let inMethod = false;
  let methodName = '';
  let methodStart = 0;
  let braceCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inMethod) {
      JAVA_METHOD_PATTERN.lastIndex = 0;
      const match = JAVA_METHOD_PATTERN.exec(line);
      if (match) {
        inMethod = true;
        methodName = match[1] || 'unknown';
        methodStart = i;
        braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      }
    } else {
      braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      if (braceCount <= 0) {
        const methodLength = i - methodStart + 1;
        if (methodLength > 50) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: methodStart + 1,
            column: 1,
            message: `[Java] 方法 "${methodName}" 長度 ${methodLength} 行 > 50`,
            suggestion: '拆分為多個小方法',
          });
        }
        inMethod = false;
      }
    }
  }

  return violations;
}

export function checkJava(source: string, file: string): Violation[] {
  return [
    ...checkJavaRedlines(source, file),
    ...checkJavaProhibitions(source, file),
  ];
}
