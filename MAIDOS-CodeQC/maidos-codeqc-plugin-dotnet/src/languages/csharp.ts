/**
 * C# Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const CSHARP_CONFIG = {
  id: 'csharp',
  name: 'C#',
  extensions: ['.cs'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

// =============================================================================
// C#-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CSHARP_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:ApiKey|api_key|API_KEY)\s*=\s*"[^"]{8,}"/gi,
  /(?:connectionString|ConnectionString)\s*=\s*"[^"]*(?:Password|pwd)=[^"]+"/gi,
  /(?:secret|token)\s*=\s*"[^"]{8,}"/gi,
];

/** R05: 忽略錯誤處理 */
const CSHARP_ERROR_PATTERNS = [
  /catch\s*(?:\([^)]*\))?\s*\{\s*\}/g,
  /catch\s*\{\s*\}/g,
];

/** C# 特有：不安全的反序列化 */
const CSHARP_UNSAFE_DESERIALIZATION = [
  /BinaryFormatter\s*\(\)/g,
  /new\s+JavaScriptSerializer\s*\(\)/g,
  /JsonConvert\.DeserializeObject\s*<\s*object\s*>/g,
];

/** C# 特有：SQL 注入風險 */
const CSHARP_SQL_INJECTION = [
  /(?:ExecuteSqlCommand|FromSqlRaw|ExecuteSqlRaw)\s*\(\s*\$"/g,
  /new\s+SqlCommand\s*\(\s*\$"/g,
  /\.CommandText\s*=\s*\$"/g,
];

/** C# 特有：不安全的 LINQ 注入 */
const CSHARP_DYNAMIC_LINQ = [
  /\.Where\s*\(\s*\$"/g,
  /\.OrderBy\s*\(\s*\$"/g,
];

/** P05: 超長方法 */
const CSHARP_METHOD_PATTERN = /(?:public|private|protected|internal|static|async|\s)+\s+(?:\w+(?:<[^>]+>)?)\s+(\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)/g;

/** P07: 公開可變靜態欄位 */
const CSHARP_STATIC_MUTABLE = /public\s+static\s+(?!readonly|const)[^=;]+=[^;]+;/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkCSharpRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of CSHARP_CREDENTIAL_PATTERNS) {
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
          message: `[C#] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 appsettings.json、User Secrets 或環境變數',
        });
        break;
      }
    }

    // R02: 不安全的反序列化
    for (const pattern of CSHARP_UNSAFE_DESERIALIZATION) {
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
          message: `[C#] 不安全的反序列化`,
          snippet: line.trim(),
          suggestion: '使用 System.Text.Json 或設定 TypeNameHandling.None',
        });
        break;
      }
    }

    // R02: SQL 注入
    for (const pattern of CSHARP_SQL_INJECTION) {
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
          message: `[C#] SQL 字串插值可能導致注入`,
          snippet: line.trim(),
          suggestion: '使用參數化查詢或 FromSqlInterpolated',
        });
        break;
      }
    }
  }

  // R05: 空 catch
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of CSHARP_ERROR_PATTERNS) {
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
        message: `[C#] 空的 catch 區塊`,
        snippet: match[0].substring(0, 50),
        suggestion: '記錄異常或重新拋出',
      });
    }
  }

  return violations;
}

export function checkCSharpProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: 公開可變靜態欄位
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    CSHARP_STATIC_MUTABLE.lastIndex = 0;
    const match = CSHARP_STATIC_MUTABLE.exec(line);
    if (match) {
      violations.push({
        ruleId: 'P07',
        ruleName: '全局狀態',
        severity: 'warning',
        file,
        line: i + 1,
        column: 1,
        message: `[C#] 公開的可變靜態欄位`,
        snippet: line.trim(),
        suggestion: '使用 private static readonly 或依賴注入',
      });
    }
  }

  // P05: 超長方法
  let inMethod = false;
  let methodName = '';
  let methodStart = 0;
  let braceCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inMethod) {
      CSHARP_METHOD_PATTERN.lastIndex = 0;
      const match = CSHARP_METHOD_PATTERN.exec(line);
      if (match && line.includes('{')) {
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
            message: `[C#] 方法 "${methodName}" 長度 ${methodLength} 行 > 50`,
            suggestion: '拆分為多個小方法或使用 local functions',
          });
        }
        inMethod = false;
      }
    }
  }

  return violations;
}

export function checkCSharp(source: string, file: string): Violation[] {
  return [
    ...checkCSharpRedlines(source, file),
    ...checkCSharpProhibitions(source, file),
  ];
}
