/**
 * Code-QC v3.3 — Pipeline 引擎 (❷走線化)
 * 
 * 十步走線：build → lint → test → proof → gate → ship
 * 走線固定不可跳步，斷一條即熔斷。
 * 
 * 對照 D.md §8 Pipeline v3.3
 */

import type {
  PipelineStepResult,
  PipelineResult,
  Violation,
} from '../types.js';
import { checkRedlines, checkAntifraud } from '../rules/b-redlines.js';
import { checkProhibitions } from '../rules/b-prohibitions.js';
import { runGatesV33, type GateV33Input } from './gates-v33.js';
import { collectEvidence, judgeDod } from './evidence.js';
import { resolveProtectionLevel } from './protection.js';
import { buildYChannel, buildXChannel, buildZChannel, buildWaveformReport } from './waveform.js';
import { createHash, randomUUID } from 'node:crypto';

// =============================================================================
// Pipeline Step Definitions (D.md §8 十步走線)
// =============================================================================

interface StepDef {
  id: number;
  name: string;
  circuitTerm: string;
  pillar: string;
  fatalOnFail: boolean;
}

const STEP_DEFS: StepDef[] = [
  { id: 1,  name: '防偽檢查 (有沒有假程式碼)',       circuitTerm: '❸ 防偽掃描',     pillar: 'PROTECTION', fatalOnFail: true },
  { id: 2,  name: '詐欺掃描 (有沒有造假)',           circuitTerm: '❺ 詐欺掃描',     pillar: 'PROTECTION', fatalOnFail: true },
  { id: 3,  name: '編譯檢查 (能不能編譯)',            circuitTerm: '❸ 編譯',        pillar: 'LOGIC_GATE', fatalOnFail: true },
  { id: 4,  name: '風格檢查 (有沒有壞習慣)',          circuitTerm: '❸ 風格',        pillar: 'LOGIC_GATE', fatalOnFail: false },
  { id: 5,  name: '測試檢查 (測試有沒有全過)',         circuitTerm: '❹ 測試',        pillar: 'INSTRUMENT', fatalOnFail: true },
  { id: 6,  name: '覆蓋率 (測試涵蓋多少)',             circuitTerm: '❹ 覆蓋率',     pillar: 'INSTRUMENT', fatalOnFail: false },
  { id: 7,  name: '紅線全檢 (有沒有違反紅線)',         circuitTerm: '❸ 紅線全檢',    pillar: 'PROTECTION', fatalOnFail: true },
  { id: 8,  name: 'G1 接口同步 (有沒有對上)',          circuitTerm: '❹ G1同步',      pillar: 'LOGIC_GATE', fatalOnFail: true },
  { id: 9,  name: 'G2 規格覆蓋 (功能有沒有做完)',     circuitTerm: '❹ G2規格',      pillar: 'LOGIC_GATE', fatalOnFail: true },
  { id: 10, name: 'G4 最終驗收',                       circuitTerm: '❹ G4驗收',     pillar: 'INSTRUMENT', fatalOnFail: true },
];

// =============================================================================
// Pipeline Input
// =============================================================================

export interface PipelineInput {
  /** 掃描目標路徑 */
  targetPath: string;
  /** 檔案列表 [{path, content}] */
  files: Array<{ path: string; content: string }>;
  /** 產品等級 E=商用 / F=深科技 */
  grade: 'E' | 'F';
  /** evidence 輸出目錄 */
  evidenceDir: string;
  /** 外部命令結果 (build/lint/test/coverage) — 由 CLI 層注入 */
  externalResults?: {
    build?: { exitCode: number; log: string };
    lint?: { exitCode: number; log: string };
    test?: { exitCode: number; log: string; passed: number; failed: number };
    coverage?: { percentage: number; log: string };
    audit?: { exitCode: number; log: string; critical: number; high: number };
    package?: { exitCode: number; log: string };
    run?: { exitCode: number; log: string };
  };
  /** SPEC 文件路徑 (G2 走線連通用) */
  specPath?: string;
  /** SPEC 函數列表 (從 SPEC.md 提取的期望函數) */
  specFunctions?: string[];
  /** SPEC 完成度 (從 SPEC.md checkbox 提取) */
  specChecklist?: { total: number; done: number };

  /** Z 軸真實性/追溯證據 (由 CLI 解析 evidence/*.log 注入) */
  proof?: {
    iav?: { passed: boolean; passedCount: number; failedCount: number; details: string };
    blds?: { minScore: number; threshold: number; passed: boolean; details: string };
    datasource?: { untraced: number; passed: boolean; details: string };
  };

  /**
   * LV4 防回放 nonce (若不提供，pipeline 會自產)。
   * 建議由 pipeline 腳本生成並寫入 evidence/nonce.log。
   */
  nonce?: string;

  /**
   * Proof Pack 原始內容（用於 LV5 hash）
   * 由 CLI 層讀 evidence/*.log 注入，避免引擎直接碰檔案系統。
   */
  proofContent?: {
    iavLog?: string;
    bldsLog?: string;
    datasourceLog?: string;
  };
}

// =============================================================================
// Step Runners
// =============================================================================

type StepRunner = (input: PipelineInput, prev: PipelineStepResult[]) => PipelineStepResult;

function isTestPath(path: string): boolean {
  const norm = path.replace(/\\/g, '/');
  return /(^|\/)(tests?|specs?|__tests__|__test__|__specs__|__spec__|fixtures|__mocks__|mocks|mock|__snapshots__)(\/|$)/i.test(norm)
    || /\.(?:test|spec)\.[^/]+$/i.test(norm);
}

/**
 * [1/10] 保險絲檢查 — R13-R18 假實現
 * 只掃描 anti-fraud 紅線 (最致命的)
 */
function step01_fuseCheck(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const fraudViolations: Violation[] = [];

  for (const f of input.files) {
    // 排除測試 fixtures、web-ui demo 代碼
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    // v3.3 Step1: 只針對 R13-R18 (假實現/靜默失敗/TODO/反詐欺) 做保險絲快篩
    const subset = checkRedlines(f.content, f.path).filter(v => (
      v.ruleId === 'R13' || v.ruleId === 'R14' || v.ruleId === 'R15' ||
      v.ruleId === 'R16' || v.ruleId === 'R17' || v.ruleId === 'R18'
    ));
    fraudViolations.push(...subset);
  }

  const passed = fraudViolations.length === 0;
  const log = passed
    ? ''
    : fraudViolations
      .map(v => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`)
      .join('\n');
  return {
    step: 1,
    name: STEP_DEFS[0]!.name,
    circuitTerm: STEP_DEFS[0]!.circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed
      ? '防偽通過: 沒有假程式碼'
      : `🔴 防偽失敗! ${fraudViolations.length} 個假實現 — ${fraudViolations.map(v => `${v.ruleId}@${v.file}:${v.line}`).join(', ')}`,
    faultMode: passed ? undefined : 'SHORT_CIRCUIT',
    evidencePath: 'evidence/scan.log',
    log,
    violations: fraudViolations,
    stats: { violations: fraudViolations.length },
  };
}

/**
 * [2/10] 詐欺掃描 — 全面反造假檢查
 */
function step02_esdProtection(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const fraudViolations: Violation[] = [];

  for (const f of input.files) {
    // 排除測試檔案、fixtures、web-ui
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    fraudViolations.push(...checkAntifraud(f.content, f.path));
  }

  const passed = fraudViolations.length === 0;
  const log = passed
    ? ''
    : fraudViolations
      .map(v => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`)
      .join('\n');
  return {
    step: 2,
    name: STEP_DEFS[1]!.name,
    circuitTerm: STEP_DEFS[1]!.circuitTerm,
    passed,
    duration: Math.round(performance.now() - t0),
    details: passed
      ? '詐欺掃描通過: 沒有造假'
      : `🔴 發現造假! ${fraudViolations.length} 個詐欺問題 (R16/R17/R18)`,
    faultMode: passed ? undefined : 'SHORT_CIRCUIT',
    evidencePath: 'evidence/fraud.log',
    log,
    violations: fraudViolations,
    stats: { violations: fraudViolations.length },
  };
}

/**
 * [3/10] 焊接 — 編譯 0e+0w
 * 依賴外部 build 結果注入
 */
function step03_solder(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const ext = input.externalResults?.build;

  if (!ext) {
    return {
      step: 3, name: STEP_DEFS[2]!.name, circuitTerm: STEP_DEFS[2]!.circuitTerm,
      passed: false, duration: Math.round(performance.now() - t0),
      details: '⚠️ 沒有編譯結果 (用 --build 指定編譯指令，或讓 CLI 自動偵測)',
      faultMode: 'OPEN_CIRCUIT',
    };
  }

  const passed = ext.exitCode === 0;
  return {
    step: 3, name: STEP_DEFS[2]!.name, circuitTerm: STEP_DEFS[2]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed ? '編譯通過: 0 錯誤 0 警告' : `🔴 編譯失敗! (exit ${ext.exitCode})`,
    faultMode: passed ? undefined : 'COLD_SOLDER_JOINT',
    evidencePath: 'evidence/build.log',
  };
}

/**
 * [4/10] 清洗 — Lint
 */
function step04_wash(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const ext = input.externalResults?.lint;

  if (!ext) {
    // Lint 非致命，可降級為內部禁止規則檢查
    const violations: Violation[] = [];
    for (const f of input.files) {
      violations.push(...checkProhibitions(f.content, f.path));
    }
    const passed = violations.filter(v => v.severity === 'error').length === 0;
    return {
      step: 4, name: STEP_DEFS[3]!.name, circuitTerm: STEP_DEFS[3]!.circuitTerm,
      passed, duration: Math.round(performance.now() - t0),
      details: passed
        ? `風格檢查通過: 禁止規則掃描 ${violations.length} warnings`
        : `🟡 風格有問題: ${violations.length} 個違規 (禁止規則)`,
    };
  }

  const passed = ext.exitCode === 0;
  return {
    step: 4, name: STEP_DEFS[3]!.name, circuitTerm: STEP_DEFS[3]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed ? '風格檢查通過: Lint 0 錯誤 0 警告' : `🟡 風格有問題: Lint 有錯 (exit ${ext.exitCode})`,
    evidencePath: 'evidence/lint.log',
  };
}

/**
 * [5/10] 焊點測試 — 單元測試
 */
function step05_jointTest(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const ext = input.externalResults?.test;

  if (!ext) {
    return {
      step: 5, name: STEP_DEFS[4]!.name, circuitTerm: STEP_DEFS[4]!.circuitTerm,
      passed: false, duration: Math.round(performance.now() - t0),
      details: '⚠️ 沒有測試結果 (用 --test 指定測試指令，或讓 CLI 自動偵測)',
      faultMode: 'OPEN_CIRCUIT',
    };
  }

  const passed = (ext.exitCode === 0 && ext.failed === 0) || (ext.failed === 0 && ext.passed > 0);
  return {
    step: 5, name: STEP_DEFS[4]!.name, circuitTerm: STEP_DEFS[4]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? `測試通過: ${ext.passed} passed, 0 failed`
      : `🔴 測試失敗! ${ext.failed} tests failed`,
    faultMode: passed ? undefined : 'COLD_SOLDER_JOINT',
    evidencePath: 'evidence/test.log',
  };
}

/**
 * [6/10] 頻譜 — 覆蓋率
 */
function step06_spectrum(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const ext = input.externalResults?.coverage;
  const threshold = 80; // C.md §5: 覆蓋率 ≥80%

  if (!ext) {
    return {
      step: 6, name: STEP_DEFS[5]!.name, circuitTerm: STEP_DEFS[5]!.circuitTerm,
      passed: false, duration: Math.round(performance.now() - t0),
      details: '⚠️ 沒有覆蓋率結果',
    };
  }

  const passed = ext.percentage >= threshold;
  return {
    step: 6, name: STEP_DEFS[5]!.name, circuitTerm: STEP_DEFS[5]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? `覆蓋率達標: ${ext.percentage}% ≥ ${threshold}%`
      : `🟡 覆蓋率不足: ${ext.percentage}% < ${threshold}%`,
    evidencePath: 'evidence/coverage.log',
  };
}

/**
 * [7/10] 保險絲全檢 — 全紅線 R01-R18
 */
function step07_fuseFullCheck(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const violations: Violation[] = [];

  for (const f of input.files) {
    // 排除規則定義檔 (含敏感關鍵字作為 regex 模式)、測試檔、測試 fixtures、和 web-ui
    if (/(?:b-redlines|b-prohibitions|c-gates)[/\\.]/.test(f.path)) continue;
    if (isTestPath(f.path)) continue;
    if (/web-ui[/\\]/i.test(f.path)) continue;
    violations.push(...checkRedlines(f.content, f.path));
  }

  const errors = violations.filter(v => v.severity === 'error');
  const passed = errors.length === 0;
  const log = passed
    ? ''
    : errors
      .map(v => `${v.ruleId} ${v.file}:${v.line}:${v.column ?? 1} ${v.message}`)
      .join('\n');
  return {
    step: 7, name: STEP_DEFS[6]!.name, circuitTerm: STEP_DEFS[6]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? `紅線全檢通過: R01-R18 = 0 違規`
      : `🔴 紅線違規! ${errors.length} 條 — ${errors.slice(0, 5).map(v => v.ruleId).join(',')}`,
    faultMode: passed ? undefined : 'SHORT_CIRCUIT',
    evidencePath: 'evidence/redline.log',
    log,
    violations: errors,
    stats: { violations: errors.length },
  };
}

/**
 * [8/10] G1 腳位接觸 — 萬用表
 * 檢查 SPEC 同步 + TODO/DISCONNECT 標記
 */
function step08_g1Contact(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();
  const disconnects: string[] = [];

  for (const f of input.files) {
    // 排除引擎自身 + 規則定義檔 (含 TODO/FIXME 作為 regex pattern)
    if (/(?:engine|rules)[/\\]/i.test(f.path)) continue;
    // 排除測試檔
    if (isTestPath(f.path)) continue;
    const lines = f.content.split('\n');
    lines.forEach((line, i) => {
      // 只抓真正的 DISCONNECTED 標記和有意義的 TODO connect / FIXME wire
      if (/\bDISCONNECTED\b/.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      } else if (/\bTODO\b.*\bconnect\b/i.test(line) && !/regex|pattern|test|detect/i.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      } else if (/\bFIXME\b.*\bwire\b/i.test(line) && !/regex|pattern|test|detect/i.test(line)) {
        disconnects.push(`${f.path}:${i + 1}`);
      }
    });
  }

  const passed = disconnects.length === 0;
  const log = passed ? '' : disconnects.join('\n');
  return {
    step: 8, name: STEP_DEFS[7]!.name, circuitTerm: STEP_DEFS[7]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? 'G1 接口正常: 沒有斷開的功能'
      : `🔴 G1 有功能斷開! ${disconnects.length} 個 DISCONNECTED — ${disconnects.slice(0, 3).join(', ')}`,
    faultMode: passed ? undefined : 'OPEN_CIRCUIT',
    evidencePath: 'evidence/sync.log',
    log,
    stats: { disconnects: disconnects.length },
  };
}

/**
 * [9/10] G2 走線連通 — 蜂鳴檔
 * 檢查 SPEC 函數是否都已實現
 */
function step09_g2Continuity(input: PipelineInput): PipelineStepResult {
  const t0 = performance.now();

  const specTotal = input.specChecklist?.total ?? 0;
  const specDone = input.specChecklist?.done ?? 0;
  const specPct = specTotal > 0 ? Math.round((specDone / specTotal) * 100) : 0;
  const specChecklistOk = specTotal > 0 && specDone === specTotal;

  if (!input.specFunctions || input.specFunctions.length === 0) {
    return {
      step: 9, name: STEP_DEFS[8]!.name, circuitTerm: STEP_DEFS[8]!.circuitTerm,
      passed: false, duration: Math.round(performance.now() - t0),
      details: '⚠️ G2 規格核對: 找不到 SPEC 函數清單 (用 --spec 指向 SPEC.md，並用 `→ `標註函數)',
      faultMode: 'OPEN_CIRCUIT',
      evidencePath: 'evidence/mapping.log',
      log: 'MISSING: <no spec functions extracted>',
      stats: { specTotal, specDone, specPct, missing: 0 },
    };
  }

  const allSource = input.files.map(f => f.content).join('\n');
  const missing = input.specFunctions.filter(fn => !allSource.includes(fn));

  const passed = specChecklistOk && missing.length === 0;
  const log = missing.length === 0 ? '' : missing.map(fn => `MISSING: ${fn}`).join('\n');
  return {
    step: 9, name: STEP_DEFS[8]!.name, circuitTerm: STEP_DEFS[8]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? `G2 規格核對: SPEC=${specPct}% · ${input.specFunctions.length}/${input.specFunctions.length} 函數都有實作`
      : `🔴 G2 FAIL: SPEC=${specPct}% (${specDone}/${specTotal}) · 缺 ${missing.length} 個函數 — ${missing.slice(0, 5).join(', ')}`,
    faultMode: passed ? undefined : 'OPEN_CIRCUIT',
    evidencePath: 'evidence/mapping.log',
    log,
    stats: { specTotal, specDone, specPct, missing: missing.length },
  };
}

/**
 * [10/10] G4 打包+上電
 * 最終驗證: 所有前置步驟通過 + 可打包
 */
function step10_g4PowerOn(_input: PipelineInput, prevSteps: PipelineStepResult[]): PipelineStepResult {
  const t0 = performance.now();
  const fatalFails = prevSteps.filter((s, i) => STEP_DEFS[i]!.fatalOnFail && !s.passed);

  const pkg = _input.externalResults?.package;
  const run = _input.externalResults?.run;
  const deliveryProvided = Boolean(pkg) && Boolean(run);
  const deliveryPassed = deliveryProvided && pkg!.exitCode === 0 && run!.exitCode === 0;

  const passed = fatalFails.length === 0 && deliveryPassed;
  return {
    step: 10, name: STEP_DEFS[9]!.name, circuitTerm: STEP_DEFS[9]!.circuitTerm,
    passed, duration: Math.round(performance.now() - t0),
    details: passed
      ? '✅ G4 驗收通過: 全部合格，可以出貨'
      : (!deliveryProvided
        ? '⚠️ G4 缺交付證據 (需要 --package-cmd + --run-cmd)'
        : `🔴 G4 驗收不通過! 關鍵步驟失敗=${fatalFails.length} · package=${pkg!.exitCode} run=${run!.exitCode}`),
    faultMode: passed ? undefined : 'OPEN_CIRCUIT',
    evidencePath: 'evidence/g4_package.log',
    log: deliveryProvided
      ? `package.exit=${pkg!.exitCode}\nrun.exit=${run!.exitCode}`
      : 'MISSING: package/run',
    stats: {
      fatalFails: fatalFails.length,
      deliveryProvided: deliveryProvided ? 1 : 0,
      packageExit: pkg?.exitCode ?? 999,
      runExit: run?.exitCode ?? 999,
    },
  };
}

// =============================================================================
// Step Runner Registry
// =============================================================================

const STEP_RUNNERS: StepRunner[] = [
  (input) => step01_fuseCheck(input),
  (input) => step02_esdProtection(input),
  (input) => step03_solder(input),
  (input) => step04_wash(input),
  (input) => step05_jointTest(input),
  (input) => step06_spectrum(input),
  (input) => step07_fuseFullCheck(input),
  (input) => step08_g1Contact(input),
  (input) => step09_g2Continuity(input),
  (input, prev) => step10_g4PowerOn(input, prev),
];

// =============================================================================
// Pipeline Orchestrator — 主入口
// =============================================================================

/**
 * 執行 v3.3 十步走線 Pipeline
 * 
 * 走線規則:
 * - 順序執行，不可跳步
 * - fatalOnFail=true 的步驟失敗 → 記錄但繼續跑完全部 (收集完整 evidence)
 * - 最終由 step10 (G4) 判定總通過
 * 
 * @returns PipelineResult 完整結果 (含 evidence 路徑)
 */
export function runPipeline(input: PipelineInput): PipelineResult {
  const t0 = performance.now();
  const steps: PipelineStepResult[] = [];
  const timestamp = new Date().toISOString();
  const nonce = (input.nonce && input.nonce.trim().length > 0) ? input.nonce.trim() : randomUUID();

  // ── 十步走線 ──
  for (let i = 0; i < STEP_RUNNERS.length; i++) {
    const runner = STEP_RUNNERS[i]!;
    const result = runner(input, steps);
    steps.push(result);
  }

  // ── G1-G4 門禁彙整 ──
  const gateInput: GateV33Input = {
    step8Result: steps[7]!,  // G1
    step9Result: steps[8]!,  // G2
    step10Result: steps[9]!, // G4
    allSteps: steps,
    files: input.files,
  };
  const gates = runGatesV33(gateInput);

  // ── DoD 8 點判定 ──
  const evidence = collectEvidence(steps, gates, input);
  const dod = judgeDod(evidence);

  // ── LV5: evidence hash (sha256) ──
  // This is a minimal LV5 implementation: hash the proof inputs deterministically.
  const ext = input.externalResults;
  const hashPayload = JSON.stringify({
    version: '3.3',
    timestamp,
    targetPath: input.targetPath,
    grade: input.grade,
    nonce,
    external: {
      build: ext?.build?.log ?? '',
      lint: ext?.lint?.log ?? '',
      test: ext?.test?.log ?? '',
      coverage: ext?.coverage?.log ?? '',
      audit: ext?.audit?.log ?? '',
      package: ext?.package?.log ?? '',
      run: ext?.run?.log ?? '',
    },
    internal: steps.map(s => ({
      step: s.step,
      passed: s.passed,
      evidencePath: s.evidencePath ?? '',
      log: s.log ?? '',
    })),
    proof: {
      iav: input.proofContent?.iavLog ?? '',
      blds: input.proofContent?.bldsLog ?? '',
      datasource: input.proofContent?.datasourceLog ?? '',
    },
  });
  const evidenceHash = createHash('sha256').update(hashPayload, 'utf8').digest('hex');

  // ── 防偽等級 ──
  const protectionTarget = input.grade === 'E' ? 5 : 9;
  const protectionLevel = resolveProtectionLevel(input.grade, steps, { nonce, evidenceHash });

  // ── 最終判定: 十步走線 + 四門禁 + DoD 全過 ──
  const passed = steps[9]!.passed && gates.allPassed && dod.missionComplete && (protectionLevel >= protectionTarget);

  // ── 三通道示波器 ──
  const z = input.proof;
  const iavPass = z?.iav?.passed ?? false;
  const bldsScore = z?.blds?.minScore ?? 0;
  const bldsMin = z?.blds?.threshold ?? (input.grade === 'F' ? 4 : 3);
  const traceability = z?.datasource?.passed ?? false;

  const yChannel = buildYChannel({
    specMapped: steps[8]!.passed,       // G2 走線連通 = SPEC映射
    specMissingCount: steps[8]!.stats?.missing ?? 0,
    testsPass: steps[4]!.passed,        // Step5 焊點測試
    testsFailed: ext?.test?.failed ?? 0,
    testsTotal: (ext?.test?.passed ?? 0) + (ext?.test?.failed ?? 0),
    coveragePercent: ext?.coverage?.percentage ?? 0,
    coverageThreshold: input.grade === 'F' ? 90 : 80,
    implComplete: steps[8]!.passed,     // G2
  });
  const xChannel = buildXChannel({
    buildErrors: ext?.build?.exitCode === 0 ? 0 : 1,
    buildWarnings: 0,
    lintErrors: ext?.lint?.exitCode === 0 ? 0 : 1,
    lintWarnings: 0,
    redlineViolations: steps[6]!.stats?.violations ?? (steps[6]!.passed ? 0 : 1), // Step7 紅線全檢
    securityCritical: ext?.audit ? ext.audit.critical : 1,
    securityHigh: ext?.audit ? ext.audit.high : 0,
  });
  const zChannel = buildZChannel({
    fraudCount: steps[1]!.stats?.violations ?? (steps[1]!.passed ? 0 : 1),
    iavPass,
    bldsScore,
    bldsMinimum: bldsMin,
    traceability,
  });
  const waveform = buildWaveformReport(yChannel, xChannel, zChannel);

  return {
    version: '3.3',
    timestamp,
    targetPath: input.targetPath,
    productGrade: input.grade,
    protectionLevel,
    protectionTarget,
    nonce,
    evidenceHash,
    steps,
    gates,
    dod,
    waveform,
    passed,
    duration: Math.round(performance.now() - t0),
    evidenceDir: input.evidenceDir,
  };
}

/**
 * 格式化 Pipeline 報告 (Console 用)
 */
export function formatPipelineReport(result: PipelineResult): string {
  const lines: string[] = [];

  lines.push('');
  lines.push('╔══════════════════════════════════════════════════════════╗');
  lines.push('║  Code-QC v3.3 品質檢測台 (Test Bench)                   ║');
  lines.push('║  程式品質，用硬體標準來驗。                              ║');
  lines.push('╚══════════════════════════════════════════════════════════╝');
  lines.push('');
  lines.push(`📂 Target: ${result.targetPath}`);
  lines.push(`🏷️  Grade: ${result.productGrade} (${result.productGrade === 'E' ? '商用級' : '深科技級'})`);
  lines.push(`🔒 Protection: LV1-${result.protectionLevel} (target=LV${result.protectionTarget})`);
  lines.push(`🧷 Nonce (LV4): ${result.nonce}`);
  lines.push(`🔏 Evidence Hash (LV5): ${result.evidenceHash.substring(0, 16)}…`);
  lines.push('');

  // ── 十步走線 ──
  lines.push('─── Pipeline 十步檢查 ───');
  lines.push('');
  for (const step of result.steps) {
    const icon = step.passed ? '✅' : '❌';
    const duration = `${step.duration}ms`;
    lines.push(`  ${icon} [${String(step.step).padStart(2, ' ')}/10] ${step.circuitTerm} — ${step.name} (${duration})`);
    if (!step.passed) {
      lines.push(`         ${step.details}`);
    }
  }
  lines.push('');

  // ── 四門禁 ──
  lines.push('─── G1-G4 四道品質關卡 ───');
  lines.push('');
  const gateEntries = [
    ['G1', '接口完整', '逐點檢查', result.gates.G1],
    ['G2', '規格覆蓋', '連通測試', result.gates.G2],
    ['G3', '防護到位', '壓力測試', result.gates.G3],
    ['G4', '整體驗收', '綜合判定', result.gates.G4],
  ] as const;
  for (const [id, name, tool, gate] of gateEntries) {
    const icon = gate.passed ? '✅' : '❌';
    lines.push(`  ${icon} ${id} ${name} (${tool}): ${gate.details}`);
  }
  lines.push('');

  // ── DoD 8 點 ──
  lines.push('─── DoD 8 點交付確認 ───');
  lines.push('');
  for (const item of result.dod.items) {
    const icon = item.passed ? '✅' : '❌';
    lines.push(`  ${icon} [${item.id}] ${item.name}: ${item.verification}`);
  }
  lines.push('');

  // ── 最終判定 ──
  const passCount = result.steps.filter(s => s.passed).length;
  const dodCount = result.dod.items.filter(i => i.passed).length;
  lines.push('═══════════════════════════════════════════════════════════');
  if (result.passed) {
    lines.push(`  ✅ MISSION COMPLETE — Pipeline ${passCount}/10 · Gates ${result.gates.allPassed ? '4/4' : 'FAIL'} · DoD ${dodCount}/8`);
    lines.push('  全部合格，可以出貨。');
  } else {
    lines.push(`  ❌ REJECTED — Pipeline ${passCount}/10 · Gates ${result.gates.allPassed ? '4/4' : 'FAIL'} · DoD ${dodCount}/8`);
    lines.push('  有問題，需要修復再交。');
  }
  lines.push('═══════════════════════════════════════════════════════════');
  lines.push('');

  return lines.join('\n');
}
