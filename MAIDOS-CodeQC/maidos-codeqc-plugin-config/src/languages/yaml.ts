/**
 * YAML Language Support for MAIDOS CodeQC
 * 
 * 覆蓋：YAML, Docker Compose, Kubernetes, Ansible, CI/CD configs
 */

import type { LanguagePlugin, Violation } from '../types.js';

// =============================================================================
// YAML-Specific Patterns
// =============================================================================

/** R01: 硬編碼憑證 */
const CREDENTIAL_PATTERNS = [
  /(?:password|passwd|secret|api_?key|token|auth):\s*["']?(?![\$\{])[^\s"'#]+["']?/gi,
  /(?:DB_PASSWORD|API_SECRET|SECRET_KEY):\s*["']?(?![\$\{])[^\s"'#]+["']?/gi,
  /credentials:\s*\n\s+\w+:\s*["']?(?![\$\{])[^\s"'#]+["']?/gi,
];

/** R07: 安全功能禁用 */
const SECURITY_DISABLE_PATTERNS = [
  /privileged:\s*true/gi,
  /runAsRoot:\s*true/gi,
  /allowPrivilegeEscalation:\s*true/gi,
  /hostNetwork:\s*true/gi,
  /hostPID:\s*true/gi,
  /readOnlyRootFilesystem:\s*false/gi,
  /securityContext:\s*\{\}/gi,
  /ssl_verify:\s*(?:false|no|0)/gi,
  /verify_ssl:\s*(?:false|no|0)/gi,
  /insecure_skip_verify:\s*true/gi,
];

/** R08: 危險配置 */
const DANGEROUS_PATTERNS = [
  /image:\s*["']?(?:\w+\/)?[\w-]+(?::latest)?["']?\s*$/gim,
  /capabilities:\s*\n\s+add:\s*\n\s+-\s*(?:ALL|SYS_ADMIN|NET_ADMIN)/gi,
  /volumes:\s*\n\s+-\s*\/:/gi,
  /command:\s*\[?\s*["']?(?:rm|dd|mkfs|chmod\s+777)/gi,
];

/** R09: 無限制資源 */
const UNLIMITED_PATTERNS = [
  /resources:\s*\{\}/gi,
  /limits:\s*\n\s+(?:cpu|memory):\s*$/gim,
  /replicas:\s*(?:[5-9]\d|\d{3,})/gi,
];

/** P04: 魔法數字 */
const MAGIC_NUMBER_PATTERNS = [
  /(?:timeout|interval|delay|period):\s*\d{5,}/gi,
  /port:\s*(?!(?:80|443|8080|8443|3000|5000|5432|3306|27017|6379)\s*$)\d{4,5}/gi,
];

/** P13: TODO */
const TODO_PATTERNS = [
  /#.*(?:TODO|FIXME|HACK|XXX)/gi,
];

// =============================================================================
// Checker Implementation
// =============================================================================

function checkLine(line: string, lineNum: number, file: string): Violation[] {
  const violations: Violation[] = [];
  
  if (/^\s*#/.test(line) && !TODO_PATTERNS.some(p => p.test(line))) return violations;
  
  const patterns: Array<{ patterns: RegExp[]; ruleId: string; ruleName: string; severity: 'error' | 'warning' | 'info'; message: string; suggestion: string }> = [
    { patterns: CREDENTIAL_PATTERNS, ruleId: 'R01', ruleName: '硬編碼憑證', severity: 'error', message: 'YAML: 硬編碼憑證', suggestion: '使用環境變數或 secrets 管理' },
    { patterns: SECURITY_DISABLE_PATTERNS, ruleId: 'R07', ruleName: '關閉安全功能', severity: 'error', message: 'YAML: 危險的安全配置', suggestion: '使用最小權限原則' },
    { patterns: DANGEROUS_PATTERNS, ruleId: 'R08', ruleName: '使用已知漏洞', severity: 'error', message: 'YAML: 危險配置', suggestion: '指定版本標籤，限制權限' },
    { patterns: UNLIMITED_PATTERNS, ruleId: 'R09', ruleName: '無限制資源', severity: 'error', message: 'YAML: 未設置資源限制', suggestion: '設置 resources.limits' },
    { patterns: MAGIC_NUMBER_PATTERNS, ruleId: 'P04', ruleName: '魔法數字', severity: 'warning', message: 'YAML: 魔法數字', suggestion: '使用變數或錨點' },
    { patterns: TODO_PATTERNS, ruleId: 'P13', ruleName: 'TODO 堆積', severity: 'info', message: 'YAML: TODO 標記', suggestion: '處理或移除 TODO' },
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

export const yamlPlugin: LanguagePlugin = {
  name: 'yaml',
  extensions: ['.yaml', '.yml'],
  
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      violations.push(...checkLine(lines[i]!, i + 1, file));
    }
    
    return violations;
  },
};

export default yamlPlugin;
