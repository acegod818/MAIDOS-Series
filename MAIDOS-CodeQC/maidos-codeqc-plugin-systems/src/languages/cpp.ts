/**
 * C++ Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const CPP_CONFIG = {
  id: 'cpp',
  name: 'C++',
  extensions: ['.cpp', '.cc', '.cxx', '.hpp', '.hxx', '.hh'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

/** R01: 硬編碼憑證 */
const CPP_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /std::string\s+(?:password|secret|key)\s*=\s*"[^"]+"/gi,
  /constexpr.*(?:PASSWORD|API_KEY|SECRET)\s*=\s*"[^"]+"/gi,
];

/** R02: 不安全函數（繼承自 C） */
const CPP_UNSAFE_FUNCTIONS = [
  { pattern: /\bstrcpy\s*\(/g, name: 'strcpy', suggestion: 'std::string 或 strncpy' },
  { pattern: /\bsprintf\s*\(/g, name: 'sprintf', suggestion: 'snprintf 或 std::format' },
  { pattern: /\bgets\s*\(/g, name: 'gets', suggestion: 'std::getline' },
];

/** R05: 忽略異常 */
const CPP_EMPTY_CATCH = /catch\s*\([^)]*\)\s*\{\s*\}/g;

/** C++ 特有：raw pointer 過度使用 */
const CPP_RAW_POINTER = /\bnew\s+\w+(?:\s*\[[^\]]*\])?\s*(?:\([^)]*\))?\s*;/g;

/** C++ 特有：C-style cast */
const CPP_C_CAST = /\(\s*(?:int|float|double|char|long|short|unsigned|void)\s*\*?\s*\)\s*\w+/g;

/** P05: 超長函數 */
const CPP_FUNCTION_PATTERN = /(?:(?:virtual|static|inline|const|constexpr)\s+)*(?:\w+(?:<[^>]+>)?(?:::\w+)*\s+)+(\w+)\s*\([^)]*\)\s*(?:const|override|final|noexcept)?\s*\{/g;

export function checkCppRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*|\/\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of CPP_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[C++] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用環境變數或配置檔案',
        });
        break;
      }
    }

    // R02: 不安全函數
    for (const { pattern, name, suggestion } of CPP_UNSAFE_FUNCTIONS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[C++] 不安全函數 ${name}`,
          snippet: line.trim(),
          suggestion: `使用 ${suggestion}`,
        });
      }
    }

    // C++ 特有：raw new
    CPP_RAW_POINTER.lastIndex = 0;
    if (CPP_RAW_POINTER.test(line)) {
      violations.push({
        ruleId: 'R09', ruleName: '無限制資源', severity: 'warning', file, line: i + 1, column: 1,
        message: `[C++] 使用 raw new 可能導致記憶體洩漏`,
        snippet: line.trim(),
        suggestion: '使用 std::unique_ptr 或 std::make_unique',
      });
    }

    // C++ 特有：C-style cast
    CPP_C_CAST.lastIndex = 0;
    if (CPP_C_CAST.test(line)) {
      violations.push({
        ruleId: 'P01', ruleName: '過度工程', severity: 'info', file, line: i + 1, column: 1,
        message: `[C++] 使用 C-style cast`,
        snippet: line.trim(),
        suggestion: '使用 static_cast/dynamic_cast/reinterpret_cast',
      });
    }
  }

  // R05: 空 catch
  const normalized = source.replace(/\n\s*/g, ' ');
  CPP_EMPTY_CATCH.lastIndex = 0;
  let match;
  while ((match = CPP_EMPTY_CATCH.exec(normalized)) !== null) {
    const before = source.substring(0, source.indexOf(match[0]));
    const lineNum = (before.match(/\n/g) || []).length + 1;
    violations.push({
      ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: lineNum, column: 1,
      message: `[C++] 空的 catch 區塊`,
      suggestion: '記錄異常或重新拋出',
    });
  }

  return violations;
}

export function checkCppProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長函數
  let inFunc = false, funcName = '', funcStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inFunc) {
      CPP_FUNCTION_PATTERN.lastIndex = 0;
      const match = CPP_FUNCTION_PATTERN.exec(line);
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
            message: `[C++] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數或使用 lambda',
          });
        }
        inFunc = false;
      }
    }
  }

  return violations;
}

export function checkCpp(source: string, file: string): Violation[] {
  return [...checkCppRedlines(source, file), ...checkCppProhibitions(source, file)];
}
