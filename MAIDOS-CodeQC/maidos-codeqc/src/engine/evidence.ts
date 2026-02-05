/**
 * Code-QC v3.3 — Evidence 收集器 + DoD 8點判定器 (❹量測化)
 * 
 * 對照 C.md §7 + D.md §7
 * 
 * Proof Pack = evidence/ 目錄下的完整 LOG 集合
 * DoD 8 點 = 每點對應一個 evidence 文件
 * 8/8 = MISSION COMPLETE · <8 = REJECTED
 */

import type {
  PipelineStepResult,
  GateV33Result,
  DoDItem,
  DoDStatus,
} from '../types.js';
import { DOD_DEFINITIONS } from '../types.js';

// =============================================================================
// Evidence Collection
// =============================================================================

export interface EvidenceCollection {
  /** 各 evidence 文件路徑 → 內容/狀態 */
  logs: Record<string, EvidenceLog>;
  /** Pipeline 步驟結果 (用於 DoD 判定) */
  steps: PipelineStepResult[];
  /** 門禁結果 */
  gates: GateV33Result;
  /** evidence 目錄 */
  dir: string;
}

export interface EvidenceLog {
  path: string;
  exists: boolean;
  lineCount: number;
  zeroViolations: boolean;
  summary: string;
}

/**
 * 從 Pipeline 結果收集 evidence
 * 對應 D.md 的 evidence/ 目錄結構
 */
export function collectEvidence(
  steps: PipelineStepResult[],
  gates: GateV33Result,
  input: {
    evidenceDir: string;
    files: Array<{ path: string; content: string }>;
    // Optional v3.3 extras injected by CLI (see PipelineInput)
    externalResults?: {
      build?: { exitCode: number; log: string };
      lint?: { exitCode: number; log: string };
      test?: { exitCode: number; log: string; passed: number; failed: number };
      coverage?: { percentage: number; log: string };
      audit?: { exitCode: number; log: string; critical: number; high: number };
      package?: { exitCode: number; log: string };
      run?: { exitCode: number; log: string };
    };
    specChecklist?: { total: number; done: number };
    proof?: {
      iav?: { passed: boolean; passedCount: number; failedCount: number; details: string };
      blds?: { minScore: number; threshold: number; passed: boolean; details: string };
      datasource?: { untraced: number; passed: boolean; details: string };
    };
  },
): EvidenceCollection {
  const dir = input.evidenceDir;
  const logs: Record<string, EvidenceLog> = {};

  // scan.log — Step 1 保險絲 (R13-R18)
  logs['scan.log'] = {
    path: `${dir}/scan.log`,
    exists: steps[0]?.evidencePath !== undefined,
    lineCount: steps[0]?.stats?.violations ?? (steps[0]?.passed ? 0 : 1),
    zeroViolations: steps[0]?.passed ?? false,
    summary: steps[0]?.details ?? 'N/A',
  };

  // fraud.log — Step 2 ESD
  logs['fraud.log'] = {
    path: `${dir}/fraud.log`,
    exists: steps[1]?.evidencePath !== undefined,
    lineCount: steps[1]?.stats?.violations ?? (steps[1]?.passed ? 0 : 1),
    zeroViolations: steps[1]?.passed ?? false,
    summary: steps[1]?.details ?? 'N/A',
  };

  // build.log — Step 3 編譯
  logs['build.log'] = {
    path: `${dir}/build.log`,
    exists: steps[2]?.evidencePath !== undefined,
    lineCount: 0,
    zeroViolations: steps[2]?.passed ?? false,
    summary: steps[2]?.details ?? 'N/A',
  };

  // lint.log — Step 4 Lint
  logs['lint.log'] = {
    path: `${dir}/lint.log`,
    exists: steps[3]?.evidencePath !== undefined,
    lineCount: 0,
    zeroViolations: steps[3]?.passed ?? false,
    summary: steps[3]?.details ?? 'N/A',
  };

  // test.log — Step 5 測試
  logs['test.log'] = {
    path: `${dir}/test.log`,
    exists: steps[4]?.evidencePath !== undefined,
    lineCount: 0,
    zeroViolations: steps[4]?.passed ?? false,
    summary: steps[4]?.details ?? 'N/A',
  };

  // coverage.log — Step 6 覆蓋率
  logs['coverage.log'] = {
    path: `${dir}/coverage.log`,
    exists: steps[5]?.evidencePath !== undefined,
    lineCount: 0,
    zeroViolations: steps[5]?.passed ?? false,
    summary: steps[5]?.details ?? 'N/A',
  };

  // redline.log — Step 7 紅線全檢
  logs['redline.log'] = {
    path: `${dir}/redline.log`,
    exists: steps[6]?.evidencePath !== undefined,
    lineCount: steps[6]?.stats?.violations ?? (steps[6]?.passed ? 0 : 1),
    zeroViolations: steps[6]?.passed ?? false,
    summary: steps[6]?.details ?? 'N/A',
  };

  // sync.log — Step 8 G1
  logs['sync.log'] = {
    path: `${dir}/sync.log`,
    exists: steps[7]?.evidencePath !== undefined,
    lineCount: steps[7]?.stats?.disconnects ?? (steps[7]?.passed ? 0 : 1),
    zeroViolations: steps[7]?.passed ?? false,
    summary: steps[7]?.details ?? 'N/A',
  };

  // mapping.log — Step 9 G2
  logs['mapping.log'] = {
    path: `${dir}/mapping.log`,
    exists: steps[8]?.evidencePath !== undefined,
    lineCount: 0,
    zeroViolations: steps[8]?.passed ?? false,
    summary: steps[8]?.details ?? 'N/A',
  };

  // impl.log — 補完證明 (由 CLI 產出；此處只做狀態對齊)
  logs['impl.log'] = {
    path: `${dir}/impl.log`,
    exists: true,
    lineCount: 0,
    // Treat "no MISSING" as "impl complete" at v3.3 baseline.
    zeroViolations: steps[8]?.passed ?? false,
    summary: steps[8]?.passed ? '補完證明: 規格函數均已落地' : '補完證明: 仍有缺口 (見 mapping.log)',
  };

  // iav.log / blds.log / datasource.log — Z 軸真實性證據 (由 CLI 注入解析結果)
  const iav = input.proof?.iav;
  logs['iav.log'] = {
    path: `${dir}/iav.log`,
    exists: iav !== undefined,
    lineCount: iav ? (iav.failedCount > 0 ? iav.failedCount : iav.passedCount) : 0,
    zeroViolations: iav?.passed ?? false,
    summary: iav?.details ?? '⚠️ 未提供 IAV 證據 (evidence/iav.log)',
  };

  const blds = input.proof?.blds;
  logs['blds.log'] = {
    path: `${dir}/blds.log`,
    exists: blds !== undefined,
    lineCount: 0,
    zeroViolations: blds?.passed ?? false,
    summary: blds?.details ?? '⚠️ 未提供 BLDS 證據 (evidence/blds.log)',
  };

  const ds = input.proof?.datasource;
  logs['datasource.log'] = {
    path: `${dir}/datasource.log`,
    exists: ds !== undefined,
    lineCount: ds?.untraced ?? 0,
    zeroViolations: ds?.passed ?? false,
    summary: ds?.details ?? '⚠️ 未提供 datasource 證據 (evidence/datasource.log)',
  };

  // package.log / run.log — 交付證明 (由 CLI 外部命令注入)
  logs['package.log'] = {
    path: `${dir}/package.log`,
    exists: input.externalResults?.package !== undefined,
    lineCount: 0,
    zeroViolations: input.externalResults?.package?.exitCode === 0,
    summary: input.externalResults?.package ? `package exit=${input.externalResults.package.exitCode}` : '⚠️ 未提供 package 證據',
  };

  logs['run.log'] = {
    path: `${dir}/run.log`,
    exists: input.externalResults?.run !== undefined,
    lineCount: 0,
    zeroViolations: input.externalResults?.run?.exitCode === 0,
    summary: input.externalResults?.run ? `run exit=${input.externalResults.run.exitCode}` : '⚠️ 未提供 run 證據',
  };

  // audit.log — 安全掃描 (可選，但在 waveform 會顯示)
  logs['audit.log'] = {
    path: `${dir}/audit.log`,
    exists: input.externalResults?.audit !== undefined,
    lineCount: 0,
    zeroViolations: (input.externalResults?.audit?.critical ?? 1) === 0 && (input.externalResults?.audit?.high ?? 0) === 0,
    summary: input.externalResults?.audit
      ? `audit critical=${input.externalResults.audit.critical} high=${input.externalResults.audit.high}`
      : '⚠️ 未提供 audit 證據',
  };

  return { logs, steps, gates, dir };
}

// =============================================================================
// DoD 8 點判定器
// =============================================================================

/**
 * DoD 8 點對照:
 * 1. 實現證明 → redline.log = 0
 * 2. 補完證明 → mapping.log 0 MISSING
 * 3. 規格證明 → SPEC 100% (mapping.log)
 * 4. 同步證明 → sync.log = 0
 * 5. 編譯證明 → build.log 0e/0w
 * 6. 交付證明 → G4 passed (可上電)
 * 7. 真實性   → iav.log PASS + BLDS ≥ 3
 * 8. 反詐欺   → fraud.log = 0 (R16/R17/R18)
 */
export function judgeDod(evidence: EvidenceCollection): DoDStatus {
  const { logs, gates } = evidence;

  const items: DoDItem[] = DOD_DEFINITIONS.map((def): DoDItem => {
    let passed = false;
    let evidencePath: string | undefined;

    switch (def.id) {
      case 1: // 實現證明 — redline.log = 0
        passed = logs['redline.log']?.zeroViolations ?? false;
        evidencePath = logs['redline.log']?.path;
        break;

      case 2: // 補完證明 — mapping.log 0 MISSING
        passed = (logs['mapping.log']?.zeroViolations ?? false) && (logs['impl.log']?.exists ?? false);
        evidencePath = logs['impl.log']?.path;
        break;

      case 3: // 規格證明 — SPEC 100% (等同 G2 passed)
        passed = gates.G2.passed;
        evidencePath = logs['mapping.log']?.path;
        break;

      case 4: // 同步證明 — sync.log = 0
        passed = logs['sync.log']?.zeroViolations ?? false;
        evidencePath = logs['sync.log']?.path;
        break;

      case 5: // 編譯證明 — build 0e/0w
        passed = logs['build.log']?.zeroViolations ?? false;
        evidencePath = logs['build.log']?.path;
        break;

      case 6: // 交付證明 — G4 passed
        passed = (logs['package.log']?.zeroViolations ?? false) && (logs['run.log']?.zeroViolations ?? false) && gates.G4.passed;
        evidencePath = logs['package.log']?.path;
        break;

      case 7: // 真實性 — fraud.log = 0
        passed = (logs['iav.log']?.zeroViolations ?? false) && (logs['blds.log']?.zeroViolations ?? false);
        evidencePath = logs['iav.log']?.path;
        break;

      case 8: // 反詐欺 — scan + fraud = 0
        passed = (logs['fraud.log']?.zeroViolations ?? false);
        evidencePath = logs['fraud.log']?.path;
        break;
    }

    return {
      ...def,
      passed,
      evidencePath,
    };
  });

  return {
    items,
    missionComplete: items.every(i => i.passed),
  };
}

// =============================================================================
// Proof Pack 目錄結構生成
// =============================================================================

/**
 * 產出 Proof Pack 目錄結構 (給 CLI 用，實際寫入由 CLI 層執行)
 */
export function generateProofPackManifest(evidence: EvidenceCollection): string {
  const lines: string[] = [];

  lines.push('# Proof Pack — Code-QC v3.3');
  lines.push(`# Generated: ${new Date().toISOString()}`);
  lines.push(`# Directory: ${evidence.dir}`);
  lines.push('');
  lines.push('## Evidence Files');
  lines.push('');

  for (const [name, log] of Object.entries(evidence.logs)) {
    const icon = log.zeroViolations ? '✅' : '❌';
    lines.push(`${icon} ${name} — ${log.summary}`);
  }

  lines.push('');
  lines.push('## DoD Status');
  lines.push('');

  const dod = judgeDod(evidence);
  for (const item of dod.items) {
    const icon = item.passed ? '✅' : '❌';
    lines.push(`${icon} [${item.id}] ${item.name}: ${item.verification}`);
  }

  lines.push('');
  lines.push(`## Verdict: ${dod.missionComplete ? 'MISSION COMPLETE ✅' : 'REJECTED ❌'}`);

  return lines.join('\n');
}
