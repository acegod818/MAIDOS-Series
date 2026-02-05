/**
 * Code-QC v2.4 - B 工作紀律
 * §1 八公理 (A1-A8)
 * 
 * 公理為開發原則指導，非自動檢測項目
 * 用於 AI Executor 提示和報告參考
 */

import type { Rule, AxiomId } from '../types.js';

/** 公理定義 */
export interface Axiom extends Rule {
  id: AxiomId;
  category: 'axiom';
  /** 優先級（1 = 最高） */
  priority: number;
}

/**
 * 八公理列表
 * 優先級：A6 > A1 > A2 > A4 > A5 > A3 > A7 > A8
 */
export const AXIOMS: Axiom[] = [
  {
    id: 'A1',
    category: 'axiom',
    name: '完整交付',
    nameEn: 'Complete Delivery',
    description: '每個任務必須完整完成，不留半成品',
    severity: 'error',
    action: '回滾重做',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 2,
  },
  {
    id: 'A2',
    category: 'axiom',
    name: '零技術債',
    nameEn: 'Zero Tech Debt',
    description: '不引入已知的技術債務',
    severity: 'error',
    action: '立即修復',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 3,
  },
  {
    id: 'A3',
    category: 'axiom',
    name: '可追溯',
    nameEn: 'Traceability',
    description: '所有變更必須可追溯到需求',
    severity: 'warning',
    action: '補充記錄',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 6,
  },
  {
    id: 'A4',
    category: 'axiom',
    name: '可測試',
    nameEn: 'Testability',
    description: '所有代碼必須可被測試',
    severity: 'error',
    action: '重構',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 4,
  },
  {
    id: 'A5',
    category: 'axiom',
    name: '可維護',
    nameEn: 'Maintainability',
    description: '代碼必須易於理解和修改',
    severity: 'warning',
    action: '重構',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 5,
  },
  {
    id: 'A6',
    category: 'axiom',
    name: '安全優先',
    nameEn: 'Security First',
    description: '安全考量優先於功能和性能',
    severity: 'error',
    action: '立即修復',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 1, // 最高優先
  },
  {
    id: 'A7',
    category: 'axiom',
    name: '文檔同步',
    nameEn: 'Doc Sync',
    description: '文檔與代碼保持同步',
    severity: 'warning',
    action: '立即更新',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 7,
  },
  {
    id: 'A8',
    category: 'axiom',
    name: '持續改進',
    nameEn: 'Continuous Improvement',
    description: '每次迭代都要有改進',
    severity: 'info',
    action: '回顧分析',
    autoDetectable: false,
    detectMethod: 'manual',
    priority: 8,
  },
];

/** 按優先級排序的公理 */
export const AXIOMS_BY_PRIORITY = [...AXIOMS].sort((a, b) => a.priority - b.priority);

/** 取得公理 */
export function getAxiom(id: AxiomId): Axiom | undefined {
  return AXIOMS.find(a => a.id === id);
}

/** 格式化公理提示（給 AI Executor 用） */
export function formatAxiomsPrompt(): string {
  const lines = [
    '# Code-QC v2.4 八公理',
    '',
    '優先級：A6 > A1 > A2 > A4 > A5 > A3 > A7 > A8',
    '',
  ];

  for (const axiom of AXIOMS_BY_PRIORITY) {
    lines.push(`## ${axiom.id} ${axiom.name} (${axiom.nameEn})`);
    lines.push(`- 說明：${axiom.description}`);
    lines.push(`- 違反後果：${axiom.action}`);
    lines.push('');
  }

  return lines.join('\n');
}
