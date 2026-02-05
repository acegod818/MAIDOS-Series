import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';
import { maskJsStringsAndComments } from './b-redlines-utils.js';

// =============================================================================
// R13: 假實現 (v2.6+)
// =============================================================================

const FAKE_IMPL_PATTERNS = [
  /\btodo!\b/g,
  /\bunimplemented!\b/g,
  /throw\s+new\s+NotImplementedException/gi,
  /raise\s+NotImplementedError/gi,
];

export const R13_CHECKER: RuleChecker = {
  rule: getRedline('R13')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of FAKE_IMPL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R13', ruleName: '假實現', severity: 'error', file, line: i + 1, column: match.index + 1, message: `假實現: ${match[0]}`, snippet: line.trim(), suggestion: '替換為真實業務邏輯' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R14: 靜默失敗 (v2.6+)
// =============================================================================

const SILENT_FAIL_PATTERNS = [
  /catch\s*\([^)]*\)\s*\{\s*\}/g,
  /\.catch\s*\(\s*\(\s*\w*\s*\)\s*=>\s*\{\s*\}\s*\)/g,
  /except\s*:\s*pass/g,
  /except\s+\w+\s*:\s*pass/g,
  /if\s+err\s*!=\s*nil\s*\{\s*\}/g,
];

export const R14_CHECKER: RuleChecker = {
  rule: getRedline('R14')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const normalized = source.replace(/\r\n/g, '\n');
    for (const pattern of SILENT_FAIL_PATTERNS) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(normalized)) !== null) {
        const before = normalized.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({ ruleId: 'R14', ruleName: '靜默失敗', severity: 'error', file, line: lineNum, column: 1, message: '靜默失敗: catch 不 log 不 rethrow', snippet: match[0].substring(0, 50), suggestion: '添加 log 或 re-throw' });
      }
    }
    return violations;
  },
};

// =============================================================================
// R15: TODO殘留 (v2.6+)
// =============================================================================

const TODO_PATTERNS = [
  /\/\/\s*TODO\b/gi,
  /\/\/\s*FIXME\b/gi,
  /#\s*TODO\b/gi,
  /#\s*FIXME\b/gi,
  /\/\*\s*TODO\b/gi,
];

export const R15_CHECKER: RuleChecker = {
  rule: getRedline('R15')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of TODO_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R15', ruleName: 'TODO殘留', severity: 'error', file, line: i + 1, column: match.index + 1, message: `TODO殘留: ${match[0]}`, snippet: line.trim(), suggestion: '實作或移除 TODO' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R16: 空方法 — v3.2 反詐欺
// =============================================================================

const EMPTY_METHOD_PATTERNS: Record<string, RegExp[]> = {
  typescript: [
    /\b(?:async\s+)?(?:function\s+\w+|(?:get|set)\s+\w+|(?!(?:if|for|while|switch|catch|with)\b)\w+)\s*\([^)]*\)\s*(?::\s*\w+[<[\]>|]*\s*)?\{\s*\}/g,
    /\b(?:async\s+)?(?!(?:if|for|while|switch|catch|with)\b)\w+\s*\([^)]*\)\s*(?::\s*\w+[<[\]>|]*\s*)?\{\s*return\s+(?:null|undefined|false|true|0|''|""|``|\[\]|\{\})\s*;?\s*\}/g,
  ],
  python: [
    /def\s+\w+\s*\([^)]*\)\s*(?:->\s*\w+\s*)?:\s*(?:\n\s+)?pass\b/g,
    /def\s+\w+\s*\([^)]*\)\s*(?:->\s*\w+\s*)?:\s*(?:\n\s+)?return\s+(?:None|False|True|0|''|""|\[\]|\{\})\s*$/gm,
  ],
  rust: [
    /fn\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->\s*\w+[<[\]>]*\s*)?\{\s*\}/g,
    /fn\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->\s*\w+[<[\]>]*\s*)?\{\s*(?:Ok\(\(\)\)|None|Default::default\(\)|0|true|false|String::new\(\)|Vec::new\(\))\s*\}/g,
  ],
  csharp: [
    /(?:public|private|protected|internal)\s+(?:static\s+)?(?:async\s+)?\w+[<[\]>]*\s+\w+\s*\([^)]*\)\s*\{\s*\}/g,
    /(?:public|private|protected|internal)\s+(?:static\s+)?(?:async\s+)?\w+[<[\]>]*\s+\w+\s*\([^)]*\)\s*\{\s*return\s+(?:null|false|true|0|"")\s*;\s*\}/g,
  ],
  cpp: [
    /\w+\s+\w+::\w+\s*\([^)]*\)\s*\{\s*\}/g,
    /\w+\s+\w+::\w+\s*\([^)]*\)\s*\{\s*return\s*;\s*\}/g,
  ],
};

export const R16_CHECKER: RuleChecker = {
  rule: getRedline('R16')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const ext = file.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = { ts: 'typescript', tsx: 'typescript', js: 'typescript', jsx: 'typescript', py: 'python', rs: 'rust', cs: 'csharp', cpp: 'cpp', h: 'cpp' };
    const lang = langMap[ext];
    if (!lang || !EMPTY_METHOD_PATTERNS[lang]) return violations;
    const normalized = source.replace(/\r\n/g, '\n');
    const scanSource = lang === 'typescript' ? maskJsStringsAndComments(normalized) : normalized;
    for (const pattern of EMPTY_METHOD_PATTERNS[lang]!) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(scanSource)) !== null) {
        const before = scanSource.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        // 排除測試檔案中的 mock
        if (/(?:test|spec|mock|fixture)/i.test(file)) continue;
        violations.push({ ruleId: 'R16', ruleName: '空方法', severity: 'error', file, line: lineNum, column: 1, message: '空方法 (R16): 方法體為空或僅return默認值', snippet: match[0].substring(0, 60), suggestion: '實作真實業務邏輯，回答 IAV 五問' });
      }
    }
    return violations;
  },
};

// =============================================================================
// R17: 詐欺物件 — v3.2 反詐欺
// =============================================================================

const FRAUD_OBJECT_PATTERNS = [
  /(?:const|let|var)\s+\w+\s*=\s*\{[^}]*(?:name|title|label)\s*:\s*['"`](?:Test|test|Placeholder|placeholder|Dummy|dummy|Fake|fake|Sample|sample|Mock|mock|Default|default|Example|example|Lorem|TODO)[^}]*\}/g,
  /stub\.\w+\s*=/gi,
  /mock\.\w+\s*=/gi,
  /dummy\.\w+\s*=/gi,
  /fake\.\w+\s*=/gi,
];

export const R17_CHECKER: RuleChecker = {
  rule: getRedline('R17')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture|__test__|\.test\.|\.spec\.)/i.test(file)) return violations;
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of FRAUD_OBJECT_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R17', ruleName: '詐欺物件', severity: 'error', file, line: i + 1, column: match.index + 1, message: '詐欺物件 (R17): 硬編碼假數據', snippet: match[0].substring(0, 60), suggestion: '使用真實數據源，標注 @DATASOURCE' });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R18: 繞道實作 — v3.2 反詐欺
// =============================================================================

const BYPASS_IMPL_PATTERNS = [
  /\/\/\s*(?:TODO|FIXME|HACK|TEMP)\s*.*(?:從|from|fetch|load|get|query|API|api|service|db|database)/gi,
  /\/\/\s*(?:HARDCODED|BYPASSED|SHORTCUT|STUBBED)/gi,
  /#\s*(?:HARDCODED|BYPASSED|SHORTCUT|STUBBED)/gi,
  /\/\/\s*(?:應該|should)\s*(?:從|from|call|use)/gi,
];

export const R18_CHECKER: RuleChecker = {
  rule: getRedline('R18')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of BYPASS_IMPL_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({ ruleId: 'R18', ruleName: '繞道實作', severity: 'error', file, line: i + 1, column: match.index + 1, message: `繞道實作 (R18): ${match[0].substring(0, 40)}`, snippet: line.trim(), suggestion: '使用設計上的數據源/API/Service' });
          break;
        }
      }
    }
    return violations;
  },
};
