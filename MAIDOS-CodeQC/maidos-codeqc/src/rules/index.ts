/**
 * Code-QC v3.2 Rules Module
 * 
 * B = 工作紀律 (49 rules: 8公理+18紅線+14禁止+9標記)
 * C = 驗收標準 (~75 rules: 三軸+4門禁+DoD8)
 * D = B + C (~124 rules)
 */

// B - 工作紀律
export { AXIOMS, AXIOMS_BY_PRIORITY, getAxiom, formatAxiomsPrompt } from './b-axioms.js';
export type { Axiom } from './b-axioms.js';

export { REDLINES, getRedline, checkRedlines, REDLINE_CHECKERS, ANTI_FRAUD_CHECKERS, checkAntifraud } from './b-redlines.js';
export type { Redline } from './b-redlines.js';

export { PROHIBITIONS, getProhibition, checkProhibitions, PROHIBITION_CHECKERS } from './b-prohibitions.js';
export type { Prohibition } from './b-prohibitions.js';

// C - 驗收標準
export {
  GATES,
  evaluateGate,
  createGateStatus,
  calculateComplianceScore,
  calculateOutcomeScore,
  calculateDualAxisScore,
  generateGateChecklist,
  generateAllGatesChecklist,
  HANDOVER_TEMPLATE,
  HANDOVER_TAGS,
  extractHandoverTags,
} from './c-gates.js';
export type { GateDefinition } from './c-gates.js';

// Combined check function
import { AXIOMS } from './b-axioms.js';
import { REDLINES, checkRedlines, checkAntifraud } from './b-redlines.js';
import { PROHIBITIONS, checkProhibitions } from './b-prohibitions.js';
import { GATES } from './c-gates.js';
import type { Violation, CheckLevel } from '../types.js';

/**
 * 執行規則檢查 (v3.2)
 * @param source 原始碼
 * @param file 檔案路徑
 * @param level 檢查等級 (B/C/D)
 */
export function checkRules(source: string, file: string, level: CheckLevel = 'D'): Violation[] {
  const violations: Violation[] = [];
  
  // B 層規則（紅線 R01-R18 + 禁止 P01-P14）
  if (level === 'B' || level === 'D') {
    violations.push(...checkRedlines(source, file));
    violations.push(...checkProhibitions(source, file));
  }
  
  // C 層規則（關卡檢查由外部執行，此處只做代碼檢查）
  // Gate 檢查是流程檢查，不是代碼檢查
  
  return violations;
}

/**
 * 執行反詐欺專用掃描 (v3.2 Z軸)
 * 檢測 R16空方法 + R17詐欺物件 + R18繞道實作
 */
export function checkFraud(source: string, file: string): Violation[] {
  return checkAntifraud(source, file);
}

/**
 * 取得所有規則定義
 */
export function getAllRules() {
  return {
    axioms: AXIOMS,
    redlines: REDLINES,
    prohibitions: PROHIBITIONS,
    gates: GATES,
  };
}
