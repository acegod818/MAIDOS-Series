/**
 * Groovy Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const GROOVY_CONFIG = {
  id: 'groovy',
  name: 'Groovy',
  extensions: ['.groovy', '.gradle', '.gvy', '.gy', '.gsh'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

// =============================================================================
// Groovy-specific Rule Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const GROOVY_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*['"](?!['"])[^'"]{3,}['"]/gi,
  /(?:apiKey|api_key)\s*=\s*['"][^'"]{8,}['"]/gi,
  /(?:secret|token)\s*=\s*['"][^'"]{8,}['"]/gi,
  // Gradle 特有
  /(?:signingPassword|storePassword|keyPassword)\s*=\s*['"][^'"]+['"]/gi,
];

/** R05: 忽略錯誤處理 */
const GROOVY_ERROR_PATTERNS = [
  /catch\s*\([^)]+\)\s*\{\s*\}/g,
  /catch\s*\(\s*Exception\s+\w+\s*\)\s*\{\s*\}/g,
];

/** Groovy 特有：GString 注入風險 */
const GROOVY_GSTRING_SQL_PATTERN = /(?:execute|executeQuery|executeUpdate)\s*\(\s*["']\$\{/g;

/** Groovy 特有：eval 使用 */
const GROOVY_EVAL_PATTERN = /Eval\.(?:me|x|xy|xyz)\s*\(/g;

/** P05: 超長函數 */
const GROOVY_FUNCTION_PATTERN = /\bdef\s+(\w+)\s*\([^)]*\)/g;

// =============================================================================
// Checkers
// =============================================================================

export function checkGroovyRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of GROOVY_CREDENTIAL_PATTERNS) {
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
          message: `[Groovy] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: file.endsWith('.gradle') 
            ? '使用 gradle.properties 或環境變數'
            : '使用配置檔案或環境變數',
        });
        break;
      }
    }

    // Groovy 特有：GString SQL 注入
    GROOVY_GSTRING_SQL_PATTERN.lastIndex = 0;
    const sqlMatch = GROOVY_GSTRING_SQL_PATTERN.exec(line);
    if (sqlMatch) {
      violations.push({
        ruleId: 'R02',
        ruleName: '跳過安全檢查',
        severity: 'error',
        file,
        line: i + 1,
        column: sqlMatch.index + 1,
        message: `[Groovy] GString SQL 注入風險`,
        snippet: line.trim(),
        suggestion: '使用參數化查詢',
      });
    }

    // Groovy 特有：Eval 使用
    GROOVY_EVAL_PATTERN.lastIndex = 0;
    const evalMatch = GROOVY_EVAL_PATTERN.exec(line);
    if (evalMatch) {
      violations.push({
        ruleId: 'R02',
        ruleName: '跳過安全檢查',
        severity: 'error',
        file,
        line: i + 1,
        column: evalMatch.index + 1,
        message: `[Groovy] 危險的 Eval 使用`,
        snippet: line.trim(),
        suggestion: '避免使用 Eval，使用安全的替代方案',
      });
    }
  }

  // R05: 空 catch
  const normalizedSource = source.replace(/\n\s*/g, ' ');
  for (const pattern of GROOVY_ERROR_PATTERNS) {
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
        message: `[Groovy] 空的 catch 區塊`,
        snippet: match[0].substring(0, 50),
        suggestion: '正確處理異常',
      });
    }
  }

  return violations;
}

export function checkGroovyProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長函數
  let inFunction = false;
  let funcName = '';
  let funcStart = 0;
  let braceCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    
    if (!inFunction) {
      GROOVY_FUNCTION_PATTERN.lastIndex = 0;
      const match = GROOVY_FUNCTION_PATTERN.exec(line);
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
        // Gradle 檔案允許較長的 task 定義
        const threshold = file.endsWith('.gradle') ? 80 : 50;
        if (funcLength > threshold) {
          violations.push({
            ruleId: 'P05',
            ruleName: '超長函數',
            severity: 'warning',
            file,
            line: funcStart + 1,
            column: 1,
            message: `[Groovy] 函數 "${funcName}" 長度 ${funcLength} 行 > ${threshold}`,
            suggestion: '拆分為多個小函數或使用 closure',
          });
        }
        inFunction = false;
      }
    }
  }

  return violations;
}

export function checkGroovy(source: string, file: string): Violation[] {
  return [
    ...checkGroovyRedlines(source, file),
    ...checkGroovyProhibitions(source, file),
  ];
}
