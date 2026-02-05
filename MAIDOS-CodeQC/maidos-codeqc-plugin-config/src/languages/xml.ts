/**
 * XML Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：pom.xml, web.xml, AndroidManifest.xml, .csproj, App.config
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// XML-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /<(?:password|passwd|secret|apiKey|token|auth)[^>]*>(?![\$\{])[^<]+<\//gi,
  /(?:password|secret|key|token)=["'][^"'$]+["']/gi,
  /<connectionString[^>]*>.*(?:Password|Pwd)=[^;]+/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /<debug>true<\/debug>/gi,
  /<customErrors\s+mode=["']Off["']/gi,
  /<compilation\s+debug=["']true["']/gi,
  /android:debuggable=["']true["']/gi,
  /android:allowBackup=["']true["']/gi,
  /android:usesCleartextTraffic=["']true["']/gi,
  /<httpCookies\s+requireSSL=["']false["']/gi,
];

/** R08: 危險配置 */
const DANGEROUS_PATTERNS = [
  /<permission[^>]*android:protectionLevel=["']normal["']/gi,
  /uses-permission[^>]*android\.permission\.(?:READ_CONTACTS|READ_SMS|CAMERA|RECORD_AUDIO)/gi,
  /<trace\s+enabled=["']true["']/gi,
  /<sessionState\s+mode=["']InProc["'][^>]*timeout=["']\d{3,}["']/gi,
];

/** R09: 無限制 */
const UNLIMITED_PATTERNS = [
  /<maxRequestLength=["']\d{6,}["']/gi,
  /<executionTimeout=["']0["']/gi,
  /connection-timeout=["']0["']/gi,
];

/** P14: 依賴問題 */
const DEPENDENCY_PATTERNS = [
  /<version>\s*(?:LATEST|RELEASE|\*)\s*<\/version>/gi,
  /<PackageReference[^>]*Version=["'](?:\*|latest)["']/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*<!--/.test(line)) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'XML: 硬編碼憑證', suggestion: '使用環境變數或加密配置' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'XML: 危險的安全設定', suggestion: '禁用 debug 模式，啟用安全功能' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'XML: 危險配置或權限', suggestion: '審查權限和配置' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'XML: 無限制的資源配置', suggestion: '設置合理的限制' },
    { patterns: DEPENDENCY_PATTERNS, ruleId: 'P14', ruleName: '依賴膨脹', severity: 'warning', message: 'XML: 不安全的版本範圍', suggestion: '使用固定版本號' },
  ];
  
  for (const { patterns: patternList, ruleId, ruleName, severity, message, suggestion } of patterns) {
    for (const pattern of patternList) {
      pattern.lastIndex = 0;
      if (pattern.test(line)) {
        violations.push({ ruleId, ruleName, severity, file, line: lineNum, column: 1, message, snippet: line.trim(), suggestion });
        break;
      }
    }
  }
  
  return violations;
}

// =============================================================================
// Export
// =============================================================================

export const xmlPlugin: LanguagePlugin = {
  name: 'xml',
  extensions: ['.xml', '.csproj', '.vbproj', '.fsproj', '.props', '.targets', '.config', '.plist', '.xsd', '.wsdl'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default xmlPlugin;
