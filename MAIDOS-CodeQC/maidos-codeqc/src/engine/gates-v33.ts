/**
 * Code-QC v3.3 — G1-G4 硬體化門禁 (❸閘門化)
 * 
 * 四門禁 = AND Gate 邏輯，一 LOW 即斷電
 * G1 萬用表 → 腳位接觸
 * G2 蜂鳴檔 → 走線連通
 * G3 故障注入 → 保護電路
 * G4 示波器 → 上電量測
 * 
 * 對照 C.md §6 + D.md §6
 */

import type {
  GateV33Result,
  PipelineStepResult,
} from '../types.js';
import { GATE_CIRCUIT_LABELS } from '../types.js';
import { checkProhibitions } from '../rules/b-prohibitions.js';

// =============================================================================
// Gate Input (由 Pipeline 注入)
// =============================================================================

export interface GateV33Input {
  /** Step 8 結果 (G1 腳位接觸) */
  step8Result: PipelineStepResult;
  /** Step 9 結果 (G2 走線連通) */
  step9Result: PipelineStepResult;
  /** Step 10 結果 (G4 驗收/交付證據) */
  step10Result: PipelineStepResult;
  /** 全部十步結果 (G3 保護電路用) */
  allSteps: PipelineStepResult[];
  /** 原始檔案 (G3 故障注入二次驗證用) */
  files: Array<{ path: string; content: string }>;
}

// =============================================================================
// Individual Gate Runners
// =============================================================================

/**
 * G1 — 萬用表: 腳位接觸測試
 * 核心問題: 接得上嗎？
 * 
 * 來源: Pipeline Step 8 (sync.log)
 * 判定: sync 斷開 = 0
 */
function runG1(input: GateV33Input): GateV33Result['G1'] {
  const { step8Result } = input;
  return {
    passed: step8Result.passed,
    tool: GATE_CIRCUIT_LABELS.G1.tool,
    details: step8Result.passed
      ? 'G1 PASS: 接口正常，沒有斷開的功能'
      : `G1 FAIL: ${step8Result.details}`,
  };
}

/**
 * G2 — 蜂鳴檔: 走線連通測試
 * 核心問題: 通得了嗎？
 * 
 * 來源: Pipeline Step 9 (mapping.log)
 * 判定: SPEC 函數 MISSING = 0
 */
function runG2(input: GateV33Input): GateV33Result['G2'] {
  const { step9Result } = input;
  return {
    passed: step9Result.passed,
    tool: GATE_CIRCUIT_LABELS.G2.tool,
    details: step9Result.passed
      ? 'G2 PASS: 規格全覆蓋，SPEC 100% 已實作'
      : `G2 FAIL: ${step9Result.details}`,
  };
}

/**
 * G3 — 故障注入器: 保護電路測試
 * 核心問題: 撐得住嗎？
 * 
 * 來源: Pipeline Steps 1,2,7 (保險絲+ESD+紅線全檢)
 * 判定: 紅線=0 AND 防詐=0 AND 禁止error=0
 */
function runG3(input: GateV33Input): GateV33Result['G3'] {
  const { allSteps, files } = input;

  // 從 Pipeline 結果收集保護相關步驟
  const step1 = allSteps[0]; // 保險絲 R13-18
  const step2 = allSteps[1]; // ESD 防詐
  const step7 = allSteps[6]; // 紅線全檢

  const protectionPassed = (step1?.passed ?? false) && (step2?.passed ?? false) && (step7?.passed ?? false);

  // 額外: 禁止規則 error 級檢查 (穩壓器)
  let prohibitionErrors = 0;
  for (const f of files) {
    const pViolations = checkProhibitions(f.content, f.path);
    prohibitionErrors += pViolations.filter(v => v.severity === 'error').length;
  }

  const passed = protectionPassed && prohibitionErrors === 0;
  const failReasons: string[] = [];
  if (!step1?.passed) failReasons.push('防假(R13-18)');
  if (!step2?.passed) failReasons.push('防詐');
  if (!step7?.passed) failReasons.push('紅線');
  if (prohibitionErrors > 0) failReasons.push(`禁止規則(${prohibitionErrors}e)`);

  return {
    passed,
    tool: GATE_CIRCUIT_LABELS.G3.tool,
    details: passed
      ? 'G3 PASS: 防護完整，所有防線都守住'
      : `G3 FAIL: ${failReasons.join(' + ')} 沒過`,
  };
}

/**
 * G4 — 示波器: 上電量測
 * 核心問題: 跑得動嗎？
 * 
 * 來源: Pipeline Step 10 + G1-G3 綜合
 * 判定: G1 AND G2 AND G3 AND Step3(build) AND Step5(test)
 */
function runG4(input: GateV33Input, g1: GateV33Result['G1'], g2: GateV33Result['G2'], g3: GateV33Result['G3']): GateV33Result['G4'] {
  const { allSteps, step10Result } = input;
  const buildPass = allSteps[2]?.passed ?? false;
  const testPass = allSteps[4]?.passed ?? false;

  // v3.3: G4 = G1 AND G2 AND G3 AND (build+test) AND (delivery evidence)
  const passed = g1.passed && g2.passed && g3.passed && buildPass && testPass && step10Result.passed;

  const failReasons: string[] = [];
  if (!g1.passed) failReasons.push('G1(接口)');
  if (!g2.passed) failReasons.push('G2(規格)');
  if (!g3.passed) failReasons.push('G3(防護)');
  if (!buildPass) failReasons.push('編譯');
  if (!testPass) failReasons.push('測試');
  if (!step10Result.passed) failReasons.push('交付證據');

  return {
    passed,
    tool: GATE_CIRCUIT_LABELS.G4.tool,
    details: passed
      ? 'G4 PASS: 驗收通過，全部正常，可以出貨'
      : `G4 FAIL: ${failReasons.join(' + ')} 有問題，驗收不通過`,
  };
}

// =============================================================================
// Gate Orchestrator
// =============================================================================

/**
 * 執行 v3.3 四門禁 (AND Gate 串聯)
 * G1 → G2 → G3 → G4，全 HIGH 才 PASS
 */
export function runGatesV33(input: GateV33Input): GateV33Result {
  const g1 = runG1(input);
  const g2 = runG2(input);
  const g3 = runG3(input);
  const g4 = runG4(input, g1, g2, g3);

  return {
    G1: g1,
    G2: g2,
    G3: g3,
    G4: g4,
    allPassed: g1.passed && g2.passed && g3.passed && g4.passed,
  };
}
