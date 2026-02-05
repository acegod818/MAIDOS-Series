/**
 * Code-QC v2.4 - B 工作紀律
 * §3 十四禁止 (P01-P14)
 * 
 * 實作狀態：
 * ⚠️ P01 過度工程 (需 LLM)
 * ⚠️ P02 過早優化 (需 LLM)
 * ✅ P03 複製粘貼 (基礎檢測)
 * ✅ P04 魔法數字 (regex)
 * ✅ P05 超長函數 (AST)
 * ✅ P06 深層嵌套 (AST)
 * ✅ P07 全局狀態 (regex)
 * ⚠️ P08 緊耦合 (需 LLM)
 * ✅ P09 無意義命名 (regex)
 * ✅ P10 過長參數 (regex)
 * ⚠️ P11 混合抽象 (需 LLM)
 * ✅ P12 註釋代碼 (regex)
 * ✅ P13 TODO 堆積 (regex)
 * ✅ P14 依賴膨脹 (基礎檢測)
 * 
 * 總計：10/14 已實作
 */

import type { Rule, ProhibitionId, Violation, RuleChecker } from '../types.js';

// =============================================================================
// Thresholds (Code-QC v2.4 規範)
// =============================================================================

export const THRESHOLDS = {
  /** P03: 重複代碼組數 */
  DUPLICATE_GROUPS: 3,
  /** P03: 最小重複次數 */
  DUPLICATE_MIN_COUNT: 3,
  /** P04: 魔法數字報告上限 */
  MAGIC_NUMBER_MAX_REPORT: 10,
  /** P05: 函數最大行數 */
  FUNCTION_MAX_LINES: 50,
  /** P06: 嵌套最大層數 */
  NESTING_MAX_DEPTH: 3,
  /** P09: 無意義命名報告上限 */
  MEANINGLESS_NAME_MAX_REPORT: 10,
  /** P10: 參數最大數量 */
  PARAM_MAX_COUNT: 5,
  /** P12: 註釋代碼行數閾值 */
  COMMENTED_CODE_THRESHOLD: 10,
  /** P13: TODO 最大數量 */
  TODO_MAX_COUNT: 10,
  /** P14: 生產依賴最大數量 */
  DEPS_MAX_COUNT: 50,
  /** P14: 開發依賴最大數量 */
  DEV_DEPS_MAX_COUNT: 80,
} as const;

// =============================================================================
// Types
// =============================================================================

export interface Prohibition extends Rule {
  id: ProhibitionId;
  category: 'prohibition';
  implemented: boolean;
  requiresIntegration?: string;
}

export const PROHIBITIONS: Prohibition[] = [
  { id: 'P01', category: 'prohibition', name: '過度工程', nameEn: 'Over-Engineering', description: '為不存在的需求做設計', severity: 'warning', action: '簡化', autoDetectable: false, detectMethod: 'llm', implemented: false, requiresIntegration: 'LLM 語義分析' },
  { id: 'P02', category: 'prohibition', name: '過早優化', nameEn: 'Premature Optimization', description: '在證明需要前優化', severity: 'warning', action: '移除', autoDetectable: false, detectMethod: 'llm', implemented: false, requiresIntegration: 'LLM 語義分析' },
  { id: 'P03', category: 'prohibition', name: '複製粘貼', nameEn: 'Copy-Paste', description: '大量重複代碼', severity: 'warning', action: 'DRY 重構', autoDetectable: true, detectMethod: 'heuristic', implemented: true },
  { id: 'P04', category: 'prohibition', name: '魔法數字', nameEn: 'Magic Numbers', description: '硬編碼數值無說明', severity: 'warning', action: '提取常量', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'P05', category: 'prohibition', name: '超長函數', nameEn: 'Long Function', description: '函數行數過多', severity: 'warning', action: '拆分', autoDetectable: true, detectMethod: 'ast', implemented: true, threshold: THRESHOLDS.FUNCTION_MAX_LINES },
  { id: 'P06', category: 'prohibition', name: '深層嵌套', nameEn: 'Deep Nesting', description: '嵌套層數過深', severity: 'warning', action: '提取/早返回', autoDetectable: true, detectMethod: 'ast', implemented: true, threshold: THRESHOLDS.NESTING_MAX_DEPTH },
  { id: 'P07', category: 'prohibition', name: '全局狀態', nameEn: 'Global State', description: '過度使用全局變量', severity: 'warning', action: '封裝', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'P08', category: 'prohibition', name: '緊耦合', nameEn: 'Tight Coupling', description: '模組間直接依賴', severity: 'warning', action: '依賴注入', autoDetectable: false, detectMethod: 'llm', implemented: false, requiresIntegration: 'LLM 語義分析' },
  { id: 'P09', category: 'prohibition', name: '無意義命名', nameEn: 'Meaningless Names', description: 'temp, data, info 等', severity: 'warning', action: '重命名', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'P10', category: 'prohibition', name: '過長參數', nameEn: 'Long Parameter List', description: '函數參數過多', severity: 'warning', action: '提取物件', autoDetectable: true, detectMethod: 'regex', implemented: true, threshold: THRESHOLDS.PARAM_MAX_COUNT },
  { id: 'P11', category: 'prohibition', name: '混合抽象', nameEn: 'Mixed Abstraction', description: '高低層邏輯混雜', severity: 'warning', action: '分層', autoDetectable: false, detectMethod: 'llm', implemented: false, requiresIntegration: 'LLM 語義分析' },
  { id: 'P12', category: 'prohibition', name: '註釋代碼', nameEn: 'Commented Code', description: '大量被註釋的代碼', severity: 'info', action: '刪除', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'P13', category: 'prohibition', name: 'TODO 堆積', nameEn: 'TODO Accumulation', description: '未處理的 TODO', severity: 'info', action: '處理或移除', autoDetectable: true, detectMethod: 'regex', implemented: true, threshold: THRESHOLDS.TODO_MAX_COUNT },
  { id: 'P14', category: 'prohibition', name: '依賴膨脹', nameEn: 'Dependency Bloat', description: '不必要的依賴', severity: 'info', action: '移除', autoDetectable: true, detectMethod: 'heuristic', implemented: true },
];

export function getProhibition(id: ProhibitionId): Prohibition | undefined {
  return PROHIBITIONS.find(p => p.id === id);
}

export function getImplementedProhibitions(): Prohibition[] {
  return PROHIBITIONS.filter(p => p.implemented);
}

// =============================================================================
// P03: 複製粘貼（基礎檢測 - 連續重複行）
// =============================================================================

export const P03_CHECKER: RuleChecker = {
  rule: getProhibition('P03')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n').map(l => l.trim()).filter(l => l.length > 10 && !/^\s*(?:\/\/|#|\*|\/\*)/.test(l));
    const counts: Map<string, number[]> = new Map();
    
    lines.forEach((line, i) => {
      const existing = counts.get(line) || [];
      existing.push(i + 1);
      counts.set(line, existing);
    });
    
    const duplicates = Array.from(counts.entries()).filter(([_, locs]) => locs.length >= THRESHOLDS.DUPLICATE_MIN_COUNT);
    if (duplicates.length > THRESHOLDS.DUPLICATE_GROUPS) {
      violations.push({ ruleId: 'P03', ruleName: '複製粘貼', severity: 'warning', file, line: 1, column: 1, message: `檢測到 ${duplicates.length} 組重複代碼`, suggestion: '提取為共用函數或常量' });
    }
    return violations;
  },
};

// =============================================================================
// P04: 魔法數字
// =============================================================================

const MAGIC_NUMBER_PATTERN = /(?<![\w.])\b(\d{2,})\b(?!\s*[,\]:}])/g;
const ALLOWED_NUMBERS = new Set(['0', '1', '2', '10', '100', '1000', '60', '24', '365', '1024', '4096']);

export const P04_CHECKER: RuleChecker = {
  rule: getProhibition('P04')!,
  checkSource(source: string, file: string): Violation[] {
    if (/(?:test|spec|config)\.[^/]+$/i.test(file)) return [];
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      // 跳過註解
      if (/^\s*(?:\/\/|#|\/\*|\*)/.test(line)) continue;
      // 跳過純常量定義（右邊只有數字）
      if (/^\s*(?:const|let|var|def|final|static)\s+\w+\s*=\s*\d+\s*;?\s*$/.test(line)) continue;
      
      MAGIC_NUMBER_PATTERN.lastIndex = 0;
      let match;
      while ((match = MAGIC_NUMBER_PATTERN.exec(line)) !== null) {
        if (!ALLOWED_NUMBERS.has(match[1]!)) {
          violations.push({ ruleId: 'P04', ruleName: '魔法數字', severity: 'warning', file, line: i + 1, column: match.index + 1, message: `魔法數字: ${match[1]}`, snippet: line.trim(), suggestion: '提取為有意義的常量' });
        }
      }
    }
    return violations.slice(0, THRESHOLDS.MAGIC_NUMBER_MAX_REPORT);
  },
};

// =============================================================================
// P05: 超長函數
// =============================================================================

export const P05_CHECKER: RuleChecker = {
  rule: getProhibition('P05')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    const threshold = THRESHOLDS.FUNCTION_MAX_LINES;
    const ext = file.split('.').pop()?.toLowerCase();
    
    let inFunction = false, funcName = '', funcStart = 0, braceCount = 0;
    const funcPattern = ext === 'py' ? /^\s*(?:async\s+)?def\s+(\w+)/ : /(?:function|fn|func)\s+(\w+)|(\w+)\s*[=:]\s*(?:async\s+)?(?:function|\([^)]*\)\s*=>)/;
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (!inFunction) {
        const match = line.match(funcPattern);
        if (match) {
          inFunction = true;
          funcName = match[1] || match[2] || 'anonymous';
          funcStart = i;
          braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
        }
      } else if (ext !== 'py') {
        braceCount += (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
        if (braceCount <= 0) {
          const len = i - funcStart + 1;
          if (len > threshold) {
            violations.push({ ruleId: 'P05', ruleName: '超長函數', severity: 'warning', file, line: funcStart + 1, column: 1, message: `函數 "${funcName}" 長度 ${len} 行 > ${threshold}`, suggestion: '拆分為多個小函數' });
          }
          inFunction = false;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// P06: 深層嵌套
// =============================================================================

export const P06_CHECKER: RuleChecker = {
  rule: getProhibition('P06')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    const threshold = THRESHOLDS.NESTING_MAX_DEPTH;
    let maxNesting = 0, maxLine = 0, current = 0;
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      current += (line.match(/\{/g) || []).length;
      if (current > maxNesting) { maxNesting = current; maxLine = i + 1; }
      current -= (line.match(/\}/g) || []).length;
      if (current < 0) current = 0;
    }
    
    if (maxNesting > threshold) {
      violations.push({ ruleId: 'P06', ruleName: '深層嵌套', severity: 'warning', file, line: maxLine, column: 1, message: `最大嵌套 ${maxNesting} 層 > ${threshold}`, suggestion: '使用早返回或提取子函數' });
    }
    return violations;
  },
};

// =============================================================================
// P07: 全局狀態
// =============================================================================

const GLOBAL_STATE_PATTERNS = [
  /^(?:var|let)\s+\w+\s*=/gm,  // 頂層 var/let
  /^window\.\w+\s*=/gm,
  /^global\.\w+\s*=/gm,
  /^globalThis\.\w+\s*=/gm,
];

export const P07_CHECKER: RuleChecker = {
  rule: getProhibition('P07')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of GLOBAL_STATE_PATTERNS) {
        pattern.lastIndex = 0;
        if (pattern.test(line)) {
          violations.push({ ruleId: 'P07', ruleName: '全局狀態', severity: 'warning', file, line: i + 1, column: 1, message: '檢測到全局狀態', snippet: line.trim(), suggestion: '封裝到模組或類別中' });
          break;
        }
      }
    }
    return violations.slice(0, 5);
  },
};

// =============================================================================
// P09: 無意義命名
// =============================================================================

const MEANINGLESS = ['temp', 'tmp', 'data', 'info', 'val', 'value', 'result', 'res', 'obj', 'item', 'foo', 'bar', 'baz', 'x', 'y', 'z', 'a', 'b', 'c'];

export const P09_CHECKER: RuleChecker = {
  rule: getProhibition('P09')!,
  checkSource(source: string, file: string): Violation[] {
    if (/(?:test|spec)\.[^/]+$/i.test(file)) return [];
    const violations: Violation[] = [];
    const pattern = new RegExp(`\\b(?:const|let|var|def|fn)\\s+(${MEANINGLESS.join('|')})\\b`, 'gi');
    
    source.split('\n').forEach((line, i) => {
      pattern.lastIndex = 0;
      const match = pattern.exec(line);
      if (match) {
        violations.push({ ruleId: 'P09', ruleName: '無意義命名', severity: 'warning', file, line: i + 1, column: match.index + 1, message: `無意義命名: "${match[1]}"`, suggestion: '使用描述性命名' });
      }
    });
    return violations.slice(0, THRESHOLDS.MAGIC_NUMBER_MAX_REPORT);
  },
};

// =============================================================================
// P10: 過長參數
// =============================================================================

export const P10_CHECKER: RuleChecker = {
  rule: getProhibition('P10')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const threshold = THRESHOLDS.PARAM_MAX_COUNT;
    const pattern = /(?:function|def|fn|func)\s+(\w+)\s*\(([^)]*)\)/g;
    
    source.split('\n').forEach((line, i) => {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(line)) !== null) {
        const params = match[2]?.trim();
        const count = params ? params.split(',').filter(p => p.trim()).length : 0;
        if (count > threshold) {
          violations.push({ ruleId: 'P10', ruleName: '過長參數', severity: 'warning', file, line: i + 1, column: 1, message: `函數 "${match[1]}" 有 ${count} 個參數 > ${threshold}`, suggestion: '使用參數物件' });
        }
      }
    });
    return violations;
  },
};

// =============================================================================
// P12: 註釋代碼
// =============================================================================

const COMMENTED_CODE_PATTERNS = [
  /^\s*\/\/\s*(?:const|let|var|function|if|for|while|return|import|export)\s/,
  /^\s*#\s*(?:def|class|if|for|while|return|import|from)\s/,
  /^\s*\/\*[\s\S]*(?:function|if|for|while)[\s\S]*\*\/$/,
];

export const P12_CHECKER: RuleChecker = {
  rule: getProhibition('P12')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    const lines = source.split('\n');
    let commentedCodeCount = 0;
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (COMMENTED_CODE_PATTERNS.some(p => p.test(line))) {
        commentedCodeCount++;
      }
    }
    
    if (commentedCodeCount > THRESHOLDS.COMMENTED_CODE_THRESHOLD) {
      violations.push({ ruleId: 'P12', ruleName: '註釋代碼', severity: 'info', file, line: 1, column: 1, message: `${commentedCodeCount} 行被註釋的代碼`, suggestion: '刪除無用註釋代碼，使用 VCS 保存歷史' });
    }
    return violations;
  },
};

// =============================================================================
// P13: TODO 堆積
// =============================================================================

export const P13_CHECKER: RuleChecker = {
  rule: getProhibition('P13')!,
  checkSource(source: string, file: string): Violation[] {
    const threshold = THRESHOLDS.TODO_MAX_COUNT;
    const count = (source.match(/\b(TODO|FIXME|HACK|XXX)\b/gi) || []).length;
    if (count > threshold) {
      return [{ ruleId: 'P13', ruleName: 'TODO 堆積', severity: 'info', file, line: 1, column: 1, message: `${count} 個 TODO/FIXME > ${threshold}`, suggestion: '處理或清理 TODO' }];
    }
    return [];
  },
};

// =============================================================================
// P14: 依賴膨脹（基礎檢測）
// =============================================================================

export const P14_CHECKER: RuleChecker = {
  rule: getProhibition('P14')!,
  checkSource(source: string, file: string): Violation[] {
    // 只檢查 package.json
    if (!file.endsWith('package.json')) return [];
    const violations: Violation[] = [];
    
    try {
      const pkg = JSON.parse(source);
      const deps = Object.keys(pkg.dependencies || {}).length;
      const devDeps = Object.keys(pkg.devDependencies || {}).length;
      
      if (deps > THRESHOLDS.DEPS_MAX_COUNT) {
        violations.push({ ruleId: 'P14', ruleName: '依賴膨脹', severity: 'info', file, line: 1, column: 1, message: `${deps} 個生產依賴可能過多`, suggestion: '審查並移除不必要依賴' });
      }
      if (devDeps > THRESHOLDS.DEV_DEPS_MAX_COUNT) {
        violations.push({ ruleId: 'P14', ruleName: '依賴膨脹', severity: 'info', file, line: 1, column: 1, message: `${devDeps} 個開發依賴可能過多`, suggestion: '整合或移除重複工具' });
      }
    } catch {
      // 解析失敗忽略
    }
    return violations;
  },
};

// =============================================================================
// 匯出
// =============================================================================

export const PROHIBITION_CHECKERS: RuleChecker[] = [
  P03_CHECKER, P04_CHECKER, P05_CHECKER, P06_CHECKER, P07_CHECKER,
  P09_CHECKER, P10_CHECKER, P12_CHECKER, P13_CHECKER, P14_CHECKER,
];

export function checkProhibitions(source: string, file: string): Violation[] {
  return PROHIBITION_CHECKERS.flatMap(c => c.checkSource?.(source, file) || []);
}

export function getProhibitionStats(): { total: number; implemented: number; unimplemented: string[] } {
  const unimpl = PROHIBITIONS.filter(p => !p.implemented);
  return {
    total: PROHIBITIONS.length,
    implemented: PROHIBITIONS.filter(p => p.implemented).length,
    unimplemented: unimpl.map(p => `${p.id}: ${p.name} (需要: ${p.requiresIntegration})`),
  };
}
