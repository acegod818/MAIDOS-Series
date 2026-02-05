/**
 * Objective-C Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const OBJC_CONFIG = {
  id: 'objc',
  name: 'Objective-C',
  extensions: ['.m', '.mm', '.h'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

/** R01: 硬編碼憑證 */
const OBJC_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*@"(?!")[^"]{3,}"/gi,
  /(?:apiKey|api_key)\s*=\s*@"[^"]{8,}"/gi,
  /(?:secret|token)\s*=\s*@"[^"]{8,}"/gi,
  /#define\s+(?:PASSWORD|API_KEY|SECRET)\s+@"[^"]+"/gi,
];

/** R02: 不安全的 API */
const OBJC_UNSAFE_PATTERNS = [
  /\bstrcpy\s*\(/g,
  /\bstrcat\s*\(/g,
  /\bsprintf\s*\(/g,
  /\bgets\s*\(/g,
];

/** R05: 忽略錯誤處理 */
const OBJC_ERROR_PATTERNS = [
  /@catch\s*\([^)]*\)\s*\{\s*\}/g,
  /error:\s*nil\b/g,  // 忽略 error 參數
  /error:\s*NULL\b/g,
];

/** Obj-C 特有：retain cycle 風險 */
const OBJC_RETAIN_CYCLE = /\^\s*\{[^}]*\bself\b/g;

/** P05: 超長方法 */
const OBJC_METHOD_PATTERN = /^[-+]\s*\([^)]+\)\s*(\w+)/gm;

export function checkObjCRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of OBJC_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Obj-C] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 Keychain 或配置檔案',
        });
        break;
      }
    }

    // R02: 不安全的 C 函數
    for (const pattern of OBJC_UNSAFE_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R02', ruleName: '跳過安全檢查', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Obj-C] 不安全的 C 函數 ${match[0]}`,
          snippet: line.trim(),
          suggestion: '使用安全版本如 strlcpy、snprintf',
        });
        break;
      }
    }

    // R05: 忽略 error 參數
    if (/error:\s*(?:nil|NULL)\b/.test(line)) {
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: 1,
        message: `[Obj-C] 忽略 NSError 參數`,
        snippet: line.trim(),
        suggestion: '傳入 NSError** 並檢查錯誤',
      });
    }

    // Obj-C 特有：block 中的 retain cycle
    OBJC_RETAIN_CYCLE.lastIndex = 0;
    if (OBJC_RETAIN_CYCLE.test(line)) {
      violations.push({
        ruleId: 'R09', ruleName: '無限制資源', severity: 'warning', file, line: i + 1, column: 1,
        message: `[Obj-C] Block 中直接使用 self 可能導致 retain cycle`,
        snippet: line.trim(),
        suggestion: '使用 __weak typeof(self) weakSelf = self',
      });
    }
  }

  return violations;
}

export function checkObjCProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P05: 超長方法
  let inMethod = false, methodName = '', methodStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inMethod) {
      OBJC_METHOD_PATTERN.lastIndex = 0;
      const match = OBJC_METHOD_PATTERN.exec(line);
      if (match && line.includes('{')) {
        inMethod = true;
        methodName = match[1] || 'unknown';
        methodStart = i;
        braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      }
    } else {
      braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      if (braceCount <= 0) {
        const len = i - methodStart + 1;
        if (len > 50) {
          violations.push({
            ruleId: 'P05', ruleName: '超長函數', severity: 'warning', file, line: methodStart + 1, column: 1,
            message: `[Obj-C] 方法 "${methodName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小方法',
          });
        }
        inMethod = false;
      }
    }
  }

  return violations;
}

export function checkObjC(source: string, file: string): Violation[] {
  return [...checkObjCRedlines(source, file), ...checkObjCProhibitions(source, file)];
}
