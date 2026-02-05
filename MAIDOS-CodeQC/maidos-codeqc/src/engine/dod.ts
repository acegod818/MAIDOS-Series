/**
 * CodeQC v3.3 Engine — DoD 8-Point Evaluator
 * 
 * 8/8 = MISSION COMPLETE | <8 = REJECTED
 * 
 * 1. 實現證明 (redline.log=0)
 * 2. 補完證明 (impl+mapping)
 * 3. 規格證明 (SPEC 100%)
 * 4. 同步證明 (sync.log=0)
 * 5. 編譯證明 (build 0e/0w)
 * 6. 交付證明 (package+run)
 * 7. 真實性證明 (iav PASS + BLDS≥3)
 * 8. 反詐欺證明 (fraud.log=0)
 */

// =============================================================================
// Types
// =============================================================================

export interface DoDPoint {
  id: number;
  name: string;
  verification: string;
  passed: boolean;
  evidencePath?: string;
  detail?: string;
}

export interface DoDVerdict {
  timestamp: string;
  points: DoDPoint[];
  passCount: number;
  total: 8;
  missionComplete: boolean;   // 8/8
  verdict: 'MISSION_COMPLETE' | 'REJECTED';
}

// =============================================================================
// Input
// =============================================================================

export interface DoDInput {
  // Point 1: 實現證明
  redlineViolations: number;
  // Point 2: 補完證明
  implComplete: boolean;
  mappingComplete: boolean;
  // Point 3: 規格證明
  specCoverage: number;         // 0-100%
  specMissingCount: number;
  // Point 4: 同步證明
  syncIssues: number;
  // Point 5: 編譯證明
  buildErrors: number;
  buildWarnings: number;
  // Point 6: 交付證明
  packageSuccess: boolean;
  canRun: boolean;
  // Point 7: 真實性證明
  iavPass: boolean;
  bldsScore: number;
  bldsMinimum?: number;         // default 3
  // Point 8: 反詐欺證明
  fraudDetections: number;
}

// =============================================================================
// Evaluator
// =============================================================================

export function evaluateDoD(input: DoDInput): DoDVerdict {
  const bldsMin = input.bldsMinimum ?? 3;

  const points: DoDPoint[] = [
    {
      id: 1, name: '紅線清零',
      verification: '紅線掃描 = 0 違規',
      passed: input.redlineViolations === 0,
      evidencePath: 'evidence/redline.log',
      detail: `${input.redlineViolations} violations`,
    },
    {
      id: 2, name: '功能補完',
      verification: '所有功能都已實作',
      passed: input.implComplete && input.mappingComplete,
      evidencePath: 'evidence/mapping.log',
    },
    {
      id: 3, name: '規格全覆蓋',
      verification: '需求 100% 對應，0 遺漏',
      passed: input.specCoverage >= 100 && input.specMissingCount === 0,
      evidencePath: 'evidence/mapping.log',
      detail: `${input.specCoverage}%, ${input.specMissingCount} missing`,
    },
    {
      id: 4, name: '接口同步',
      verification: '接口狀態正常，無斷開',
      passed: input.syncIssues === 0,
      evidencePath: 'evidence/sync.log',
      detail: `${input.syncIssues} issues`,
    },
    {
      id: 5, name: '編譯通過',
      verification: '編譯 0 錯誤 0 警告',
      passed: input.buildErrors === 0 && input.buildWarnings === 0,
      evidencePath: 'evidence/build.log',
      detail: `${input.buildErrors}e/${input.buildWarnings}w`,
    },
    {
      id: 6, name: '可以交付',
      verification: '能打包、能運行',
      passed: input.packageSuccess && input.canRun,
      evidencePath: 'evidence/g4_package.log',
    },
    {
      id: 7, name: '真實性驗證',
      verification: `程式碼是真的，不是假的 (BLDS ≥ ${bldsMin})`,
      passed: input.iavPass && input.bldsScore >= bldsMin,
      evidencePath: 'evidence/iav.log',
      detail: `IAV=${input.iavPass ? 'PASS' : 'FAIL'}, BLDS=${input.bldsScore}`,
    },
    {
      id: 8, name: '無詐欺',
      verification: '詐欺掃描 = 0',
      passed: input.fraudDetections === 0,
      evidencePath: 'evidence/fraud.log',
      detail: `${input.fraudDetections} detections`,
    },
  ];

  const passCount = points.filter(p => p.passed).length;

  return {
    timestamp: new Date().toISOString(),
    points,
    passCount,
    total: 8,
    missionComplete: passCount === 8,
    verdict: passCount === 8 ? 'MISSION_COMPLETE' : 'REJECTED',
  };
}
