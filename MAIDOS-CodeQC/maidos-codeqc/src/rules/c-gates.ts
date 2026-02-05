/**
 * Code-QC v2.4 - C 驗收標準
 * §1 四關卡 (Gate-In, Gate-Mid, Gate-Out, Gate-Accept)
 */

import type { GateId, GateResult, GateCheckItem, GateStatus, DualAxisScore, ComplianceScore, OutcomeScore } from '../types.js';

// =============================================================================
// Gate Definitions
// =============================================================================

export interface GateDefinition {
  id: GateId;
  name: string;
  nameEn: string;
  description: string;
  items: Omit<GateCheckItem, 'passed'>[];
}

export const GATES: GateDefinition[] = [
  {
    id: 'Gate-In',
    name: '入口關卡',
    nameEn: 'Gate-In',
    description: '需求確認階段',
    items: [
      { name: '需求明確', required: true },
      { name: '驗收標準定義', required: true },
      { name: '技術選型確定', required: true },
      { name: '依賴確認可用', required: true },
      { name: '資源已評估', required: true },
    ],
  },
  {
    id: 'Gate-Mid',
    name: '中期關卡',
    nameEn: 'Gate-Mid',
    description: '50% 進度檢查',
    items: [
      { name: '進度在 ±20% 內', required: true },
      { name: '核心架構穩定', required: true },
      { name: '核心邏輯有測試', required: true },
      { name: '阻塞項有方案', required: true },
    ],
  },
  {
    id: 'Gate-Out',
    name: '出口關卡',
    nameEn: 'Gate-Out',
    description: '完成檢查',
    items: [
      { name: '所有功能完成', required: true },
      { name: '所有測試通過', required: true },
      { name: '覆蓋率 ≥ 80%', required: true },
      { name: '無紅線違規', required: true },
      { name: '文檔完成', required: true },
      { name: '代碼審查通過', required: true },
      { name: '安全掃描通過', required: true },
    ],
  },
  {
    id: 'Gate-Accept',
    name: '驗收關卡',
    nameEn: 'Gate-Accept',
    description: '驗收確認',
    items: [
      { name: '用戶驗收測試通過', required: true },
      { name: '性能指標達標', required: true },
      { name: '可部署到目標環境', required: true },
      { name: '回滾方案就緒', required: true },
      { name: '監控告警配置', required: true },
    ],
  },
];

// =============================================================================
// Gate Evaluation
// =============================================================================

export function evaluateGate(
  gateId: GateId,
  results: Record<string, boolean>
): GateResult {
  const definition = GATES.find(g => g.id === gateId);
  if (!definition) {
    throw new Error(`Unknown gate: ${gateId}`);
  }
  
  const items: GateCheckItem[] = definition.items.map(item => ({
    ...item,
    passed: results[item.name] ?? false,
  }));
  
  const passed = items
    .filter(item => item.required)
    .every(item => item.passed);
  
  return {
    id: gateId,
    name: definition.name,
    passed,
    items,
  };
}

export function createGateStatus(
  gateIn: Record<string, boolean>,
  gateMid: Record<string, boolean>,
  gateOut: Record<string, boolean>,
  gateAccept: Record<string, boolean>
): GateStatus {
  return {
    gateIn: evaluateGate('Gate-In', gateIn),
    gateMid: evaluateGate('Gate-Mid', gateMid),
    gateOut: evaluateGate('Gate-Out', gateOut),
    gateAccept: evaluateGate('Gate-Accept', gateAccept),
  };
}

// =============================================================================
// Dual-Axis Scoring
// =============================================================================

const X_WEIGHTS = {
  codeStandard: 0.15,
  architecture: 0.20,
  security: 0.25,
  testing: 0.20,
  documentation: 0.10,
  process: 0.10,
};

const Y_WEIGHTS = {
  functionality: 0.30,
  quality: 0.20,
  performance: 0.20,
  usability: 0.15,
  satisfaction: 0.15,
};

export function calculateComplianceScore(scores: Omit<ComplianceScore, 'total'>): ComplianceScore {
  const total = 
    scores.codeStandard * X_WEIGHTS.codeStandard +
    scores.architecture * X_WEIGHTS.architecture +
    scores.security * X_WEIGHTS.security +
    scores.testing * X_WEIGHTS.testing +
    scores.documentation * X_WEIGHTS.documentation +
    scores.process * X_WEIGHTS.process;
  
  return { ...scores, total: Math.round(total * 100) / 100 };
}

export function calculateOutcomeScore(scores: Omit<OutcomeScore, 'total'>): OutcomeScore {
  const total = 
    scores.functionality * Y_WEIGHTS.functionality +
    scores.quality * Y_WEIGHTS.quality +
    scores.performance * Y_WEIGHTS.performance +
    scores.usability * Y_WEIGHTS.usability +
    scores.satisfaction * Y_WEIGHTS.satisfaction;
  
  return { ...scores, total: Math.round(total * 100) / 100 };
}

export function calculateDualAxisScore(
  xScores: Omit<ComplianceScore, 'total'>,
  yScores: Omit<OutcomeScore, 'total'>
): DualAxisScore {
  const x = calculateComplianceScore(xScores);
  const y = calculateOutcomeScore(yScores);
  
  // 評級規則
  // A: X ≥ 80% AND Y ≥ 80%
  // B: X < 80% AND Y ≥ 80%
  // C: X ≥ 80% AND Y < 80%
  // D: X < 80% AND Y < 80%
  let grade: 'A' | 'B' | 'C' | 'D';
  if (x.total >= 80 && y.total >= 80) {
    grade = 'A';
  } else if (x.total < 80 && y.total >= 80) {
    grade = 'B';
  } else if (x.total >= 80 && y.total < 80) {
    grade = 'C';
  } else {
    grade = 'D';
  }
  
  return { x, y, grade };
}

// =============================================================================
// Gate Checklist Generator
// =============================================================================

export function generateGateChecklist(gateId: GateId): string {
  const definition = GATES.find(g => g.id === gateId);
  if (!definition) {
    throw new Error(`Unknown gate: ${gateId}`);
  }
  
  const lines = [
    `# ${definition.name} (${definition.nameEn})`,
    '',
    definition.description,
    '',
    '## 檢查項',
    '',
  ];
  
  for (const item of definition.items) {
    const marker = item.required ? '✅' : '⚪';
    lines.push(`- [ ] ${marker} ${item.name}`);
  }
  
  return lines.join('\n');
}

export function generateAllGatesChecklist(): string {
  return GATES.map(g => generateGateChecklist(g.id)).join('\n\n---\n\n');
}

// =============================================================================
// HANDOVER Template
// =============================================================================

export const HANDOVER_TEMPLATE = `# 交接文檔

## 摘要
- **項目**: <項目名>
- **日期**: <日期>
- **狀態**: 🟢/🟡/🔴

## 進度
- [x] 已完成項目
- [ ] 進行中 - XX%

## 阻塞項
| 阻塞 | 需要 |
|:-----|:-----|
| <問題> | <資源> |

## 下一步
1. **P0**: <最高優先>
2. **P1**: <次優先>

## 注意事項
⚠️ <重要提醒>
`;

// =============================================================================
// Handover Tags
// =============================================================================

export const HANDOVER_TAGS = [
  { tag: '@HANDOVER', description: '交接點', format: '// @HANDOVER: <說明>' },
  { tag: '@WIP', description: '進行中', format: '// @WIP: <剩餘工作>' },
  { tag: '@BLOCKED', description: '阻塞點', format: '// @BLOCKED: <原因>' },
  { tag: '@DECISION', description: '需決策', format: '// @DECISION: <選項>' },
  { tag: '@REVIEW', description: '需審查', format: '// @REVIEW: <關注點>' },
  { tag: '@FIXME', description: '需修復', format: '// @FIXME: <問題>' },
  { tag: '@HACK', description: '臨時方案', format: '// @HACK: <原因>' },
];

export function extractHandoverTags(source: string): { tag: string; line: number; content: string }[] {
  const results: { tag: string; line: number; content: string }[] = [];
  const lines = source.split('\n');
  const tagPattern = /@(HANDOVER|WIP|BLOCKED|DECISION|REVIEW|FIXME|HACK)\s*:\s*(.+)/gi;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    tagPattern.lastIndex = 0;
    const match = tagPattern.exec(line);
    if (match) {
      results.push({
        tag: match[1]!.toUpperCase(),
        line: i + 1,
        content: match[2]!.trim(),
      });
    }
  }
  
  return results;
}
