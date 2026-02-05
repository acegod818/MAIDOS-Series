/**
 * Swift Language Support
 */

import type { Violation } from '@maidos/codeqc';

export const SWIFT_CONFIG = {
  id: 'swift',
  name: 'Swift',
  extensions: ['.swift'],
  lineComment: '//',
  blockComment: { start: '/*', end: '*/' },
};

/** R01: 硬編碼憑證 */
const SWIFT_CREDENTIAL_PATTERNS = [
  /(?:password|passwd|pwd)\s*=\s*"(?!")[^"]{3,}"/gi,
  /(?:apiKey|api_key|API_KEY)\s*=\s*"[^"]{8,}"/gi,
  /(?:secret|token)\s*=\s*"[^"]{8,}"/gi,
];

/** R05: 強制解包 */
const SWIFT_FORCE_UNWRAP = /\w+!/g;
const SWIFT_FORCE_TRY = /try!/g;
const SWIFT_FORCE_CAST = /as!/g;

/** R07: 不安全的 HTTP */
const SWIFT_INSECURE_HTTP = /NSAppTransportSecurity|NSAllowsArbitraryLoads/g;

/** Swift 特有：implicitly unwrapped optional 過度使用 */
const SWIFT_IUO_PATTERN = /(?:var|let)\s+\w+\s*:\s*\w+!/g;

/** P05: 超長函數 */
const SWIFT_FUNCTION_PATTERN = /(?:func|init)\s+(\w+)?[^{]*\{/g;

export function checkSwiftRedlines(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*(?:\/\/|\*)/.test(line)) continue;

    // R01: 硬編碼憑證
    for (const pattern of SWIFT_CREDENTIAL_PATTERNS) {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({
          ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', file, line: i + 1, column: match.index + 1,
          message: `[Swift] 檢測到硬編碼憑證`,
          snippet: line.trim(),
          suggestion: '使用 Keychain 或環境變數',
        });
        break;
      }
    }

    // R05: 強制解包
    SWIFT_FORCE_UNWRAP.lastIndex = 0;
    if (SWIFT_FORCE_UNWRAP.test(line) && !/\?\s*\?/.test(line)) { // 排除 ??
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: 1,
        message: `[Swift] 強制解包 (!) 可能導致崩潰`,
        snippet: line.trim(),
        suggestion: '使用 if let、guard let 或 ??',
      });
    }

    // R05: try!
    SWIFT_FORCE_TRY.lastIndex = 0;
    if (SWIFT_FORCE_TRY.test(line)) {
      violations.push({
        ruleId: 'R05', ruleName: '忽略錯誤處理', severity: 'error', file, line: i + 1, column: 1,
        message: `[Swift] try! 會在錯誤時崩潰`,
        snippet: line.trim(),
        suggestion: '使用 do-catch 或 try?',
      });
    }

    // R07: 不安全的 HTTP
    SWIFT_INSECURE_HTTP.lastIndex = 0;
    if (SWIFT_INSECURE_HTTP.test(line)) {
      violations.push({
        ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', file, line: i + 1, column: 1,
        message: `[Swift] App Transport Security 設定可能允許不安全連線`,
        snippet: line.trim(),
        suggestion: '僅在必要時允許特定域名的 HTTP',
      });
    }
  }

  return violations;
}

export function checkSwiftProhibitions(source: string, file: string): Violation[] {
  const violations: Violation[] = [];
  const lines = source.split('\n');

  // P07: IUO 過度使用
  let iuoCount = 0;
  for (const line of lines) {
    SWIFT_IUO_PATTERN.lastIndex = 0;
    if (SWIFT_IUO_PATTERN.test(line)) iuoCount++;
  }
  if (iuoCount > 5) {
    violations.push({
      ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', file, line: 1, column: 1,
      message: `[Swift] 過多 Implicitly Unwrapped Optional (${iuoCount} 個)`,
      suggestion: '使用普通 Optional 或 lazy 初始化',
    });
  }

  // P05: 超長函數
  let inFunc = false, funcName = '', funcStart = 0, braceCount = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (!inFunc) {
      SWIFT_FUNCTION_PATTERN.lastIndex = 0;
      const match = SWIFT_FUNCTION_PATTERN.exec(line);
      if (match) {
        inFunc = true;
        funcName = match[1] || 'init';
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
            message: `[Swift] 函數 "${funcName}" 長度 ${len} 行 > 50`,
            suggestion: '拆分為多個小函數或使用 extension',
          });
        }
        inFunc = false;
      }
    }
  }

  return violations;
}

export function checkSwift(source: string, file: string): Violation[] {
  return [...checkSwiftRedlines(source, file), ...checkSwiftProhibitions(source, file)];
}
