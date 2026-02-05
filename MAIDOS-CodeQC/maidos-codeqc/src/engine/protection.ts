/**
 * Code-QC v3.3 — LV1-LV9 防偽引擎 (❺過載保護)
 * 
 * 對照 D.md §9 + B.md §5
 * 
 * 基礎 (LV1-3): 自動掃描器可做
 * 增強 (LV4-5): 需 nonce/hash 機制 (多模型協作/外包驗收)
 * 獨立 (LV6-7): 需獨立執行環境 (裁判重跑/可信模組)
 * 形式 (LV8-9): 需交叉對抗/形式化工具 (深科技)
 * 
 * SDK 實作範圍: LV1-3 完整, LV4-5 基礎骨架, LV6-9 定義+接口
 */

import type { PipelineStepResult } from '../types.js';
import { PROTECTION_LEVELS } from '../types.js';

// =============================================================================
// Protection Level Result
// =============================================================================

export interface ProtectionContext {
  /** LV4 anti-replay nonce */
  nonce?: string;
  /** LV5 anti-tamper hash (sha256 hex) */
  evidenceHash?: string;
}

export interface ProtectionCheckResult {
  level: number;       // 1-9
  name: string;
  passed: boolean;
  details: string;
}

export interface ProtectionReport {
  grade: 'E' | 'F';
  targetLevel: number;  // E=5, F=9
  achievedLevel: number;
  checks: ProtectionCheckResult[];
  allPassed: boolean;
}

// =============================================================================
// LV1: 保險絲保護 (Redline Fuse)
// =============================================================================

function checkLV1(steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  // Pipeline Step 7 = 紅線全檢
  const step7 = steps[6];
  const passed = step7?.passed ?? false;
  return {
    level: 1,
    name: PROTECTION_LEVELS.LV1.name,
    passed,
    details: passed ? 'LV1 PASS: R01-R18 紅線 = 0' : 'LV1 FAIL: 紅線違規未清零',
  };
}

// =============================================================================
// LV2: 穩壓器限制 (Prohibition Regulator)
// =============================================================================

function checkLV2(steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  // Pipeline Step 4 = Lint/禁止規則
  const step4 = steps[3];
  const passed = step4?.passed ?? false;
  return {
    level: 2,
    name: PROTECTION_LEVELS.LV2.name,
    passed,
    details: passed ? 'LV2 PASS: P01-P14 禁止規則在限' : 'LV2 FAIL: 禁止規則超限',
  };
}

// =============================================================================
// LV3: 防詐欺掃描 (Anti-Fraud ESD)
// =============================================================================

function checkLV3(steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  // Pipeline Step 1 + Step 2 = 保險絲 + ESD
  const step1 = steps[0];
  const step2 = steps[1];
  const passed = (step1?.passed ?? false) && (step2?.passed ?? false);
  return {
    level: 3,
    name: PROTECTION_LEVELS.LV3.name,
    passed,
    details: passed ? 'LV3 PASS: Z軸反詐欺 = 0' : 'LV3 FAIL: 詐欺信號未清零',
  };
}

// =============================================================================
// LV4: 防回放鎖 (Nonce/Challenge)
// =============================================================================

function checkLV4(_steps: PipelineStepResult[], ctx?: ProtectionContext): ProtectionCheckResult {
  const nonce = ctx?.nonce;
  const passed = typeof nonce === 'string' && nonce.trim().length > 0;
  return {
    level: 4,
    name: PROTECTION_LEVELS.LV4.name,
    passed,
    details: passed
      ? 'LV4 PASS: nonce 已產出並記錄 (基礎模式)'
      : 'LV4 FAIL: 缺少 nonce (需 pipeline 產出 anti-replay token)',
  };
}

// =============================================================================
// LV5: 防篡改封印 (Hash/Merkle)
// =============================================================================

function checkLV5(_steps: PipelineStepResult[], ctx?: ProtectionContext): ProtectionCheckResult {
  const h = ctx?.evidenceHash;
  const passed = typeof h === 'string' && /^[a-f0-9]{64}$/i.test(h);
  return {
    level: 5,
    name: PROTECTION_LEVELS.LV5.name,
    passed,
    details: passed
      ? 'LV5 PASS: evidence hash 已產出並記錄 (sha256)'
      : 'LV5 FAIL: 缺少 evidence hash (需輸出 sha256，防拼貼/篡改)',
  };
}

// =============================================================================
// LV6-9: 高級防偽 (接口定義，外部實現)
// =============================================================================

function checkLV6(_steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  return { level: 6, name: PROTECTION_LEVELS.LV6.name, passed: false, details: 'LV6 SKIP: 獨立檢測站需外部 Verifier 整合' };
}

function checkLV7(_steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  return { level: 7, name: PROTECTION_LEVELS.LV7.name, passed: false, details: 'LV7 SKIP: 可信模組需 TEE/Attestation 整合' };
}

function checkLV8(_steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  return { level: 8, name: PROTECTION_LEVELS.LV8.name, passed: false, details: 'LV8 SKIP: 交叉對抗需多模型 Adversarial 整合' };
}

function checkLV9(_steps: PipelineStepResult[], _ctx?: ProtectionContext): ProtectionCheckResult {
  return { level: 9, name: PROTECTION_LEVELS.LV9.name, passed: false, details: 'LV9 SKIP: 形式化證明需外部工具 (TLA+/Coq)' };
}

// =============================================================================
// Protection Check Registry
// =============================================================================

type ProtectionChecker = (steps: PipelineStepResult[], ctx?: ProtectionContext) => ProtectionCheckResult;
const LV_CHECKERS: ProtectionChecker[] = [checkLV1, checkLV2, checkLV3, checkLV4, checkLV5, checkLV6, checkLV7, checkLV8, checkLV9];

/**
 * 執行防偽等級檢查
 * E 等級 = LV1-5 全過
 * F 等級 = LV1-9 全過
 */
export function checkProtection(grade: 'E' | 'F', steps: PipelineStepResult[], ctx?: ProtectionContext): ProtectionReport {
  const targetLevel = grade === 'E' ? 5 : 9;
  const checks = LV_CHECKERS.map(fn => fn(steps, ctx));

  // 計算達到的最高連續等級
  let achievedLevel = 0;
  for (const check of checks) {
    if (check.passed) {
      achievedLevel = check.level;
    } else {
      break; // 連續鏈斷掉
    }
  }

  const relevantChecks = checks.slice(0, targetLevel);
  const allPassed = relevantChecks.every(c => c.passed);

  return {
    grade,
    targetLevel,
    achievedLevel,
    checks: relevantChecks,
    allPassed,
  };
}

/**
 * 解析產品等級 → 實際防偽等級數字
 * 用於 PipelineResult.protectionLevel
 */
export function resolveProtectionLevel(grade: 'E' | 'F', steps: PipelineStepResult[], ctx?: ProtectionContext): number {
  const report = checkProtection(grade, steps, ctx);
  return report.achievedLevel;
}
