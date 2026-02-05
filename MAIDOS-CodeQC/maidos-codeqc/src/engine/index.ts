/**
 * Code-QC v3.3 Engine Module
 * 
 * 五段硬體化引擎:
 * ❶ 腳位化 → types.ts (SPEC/接口定義)
 * ❷ 走線化 → pipeline.ts (十步走線)
 * ❸ 閘門化 → gates-v33.ts (G1-G4 AND Gate)
 * ❹ 量測化 → evidence.ts (Proof Pack + DoD 8點)
 * ❺ 過載保護 → protection.ts (LV1-9 防偽)
 */

// Pipeline
export { runPipeline, formatPipelineReport } from './pipeline.js';
export type { PipelineInput } from './pipeline.js';

// Gates v3.3
export { runGatesV33 } from './gates-v33.js';
export type { GateV33Input } from './gates-v33.js';

// Evidence + DoD
export { collectEvidence, judgeDod, generateProofPackManifest } from './evidence.js';
export type { EvidenceCollection, EvidenceLog } from './evidence.js';

// Protection LV1-9
export { checkProtection, resolveProtectionLevel } from './protection.js';
export type { ProtectionCheckResult, ProtectionReport } from './protection.js';
