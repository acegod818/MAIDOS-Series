/**
 * MAIDOS CodeQC
 * Code Quality Control implementing Code-QC v3.3 — 軟體工程硬體化
 * 
 * v3.3 升級: 五段硬體化架構 + LV1-LV9防偽 + E/F產品等級
 * v3.2 基礎: 反詐欺紅線(R16-R18) + 三軸證明(Z軸) + IAV + BLDS + DoD 8點
 * 
 * @packageDocumentation
 */

// Types
export type {
  Severity,
  RuleCategory,
  AxiomId,
  RedlineId,
  ProhibitionId,
  GateId,
  RuleId,
  Rule,
  Violation,
  FileAnalysisResult,
  AnalysisResult,
  ComplianceScore,
  OutcomeScore,
  DualAxisScore,
  // v3.2 new types
  AuthenticityScore,
  TriAxisScore,
  IAVAnswer,
  IAVRecord,
  BLDSLevel,
  DoDItem,
  DoDStatus,
  GateCheckItem,
  GateResult,
  GateStatus,
  SupportedLanguage,
  LanguageConfig,
  CheckLevel,
  CodeQCConfig,
  LanguageSupport,
  Plugin,
  Reporter,
  RuleChecker,
} from './types.js';

export {
  DEFAULT_CONFIG,
  IAV_DISQUALIFIERS,
  BLDS_LEVELS,
  BLDS_GATE_MINIMUM,
  DOD_DEFINITIONS,
  CODEQC_VERSION,
  // v3.3 Hardwarization (軟體工程硬體化)
  HARDWARIZATION_PILLARS,
  CIRCUIT_WORLDVIEW,
  GATE_CIRCUIT_LABELS,
  PROTECTION_LAYERS,
  FAULT_MODES,
  CIRCUIT_QUICK_CARD,
  PROTECTION_LEVELS,
  PRODUCT_GRADES,
  PROTECTION_COMPONENTS,
  HARDWARE_QUICK_CARD,
} from './types.js';

// Rules
export {
  // Axioms
  AXIOMS,
  AXIOMS_BY_PRIORITY,
  getAxiom,
  formatAxiomsPrompt,
  // Redlines
  REDLINES,
  getRedline,
  checkRedlines,
  REDLINE_CHECKERS,
  ANTI_FRAUD_CHECKERS,
  checkAntifraud,
  // Prohibitions
  PROHIBITIONS,
  getProhibition,
  checkProhibitions,
  PROHIBITION_CHECKERS,
  // Gates
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
  // Combined
  checkRules,
  checkFraud,
} from './rules/index.js';

// Analyzer
export {
  detectLanguage,
  isSupported,
  analyzeFile,
  analyze,
  quickCheck,
} from './analyzer.js';

export type { AnalyzeOptions } from './analyzer.js';

// Reporters
export {
  consoleReporter,
  jsonReporter,
  htmlReporter,
  getReporter,
  reporters,
} from './reporter/index.js';

// Version
export const VERSION = '0.3.3';

// =============================================================================
// v3.3 Engine (單品 + SaaS 共用核心引擎)
// =============================================================================

export {
  // Pipeline (❷走線化)
  runPipeline,
  formatPipelineReport,
  // Gates v3.3 (❸閘門化)
  runGatesV33,
  // Evidence + DoD (❹量測化)
  collectEvidence,
  judgeDod,
  generateProofPackManifest,
  // Protection (❺保護化)
  checkProtection,
  resolveProtectionLevel,
} from './engine/index.js';

export type {
  PipelineInput,
  GateV33Input,
  EvidenceCollection,
  EvidenceLog,
  ProtectionCheckResult,
  ProtectionReport,
} from './engine/index.js';

// v3.3 Types
export type {
  GateV33Id,
  GateV33Result,
  PipelineStepId,
  PipelineStepResult,
  PipelineResult,
} from './types.js';
