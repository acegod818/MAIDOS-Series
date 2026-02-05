/**
 * MAIDOS CodeQC - Core Types
 * Implements Code-QC v3.3 specification — 軟體工程硬體化
 * 
 * v3.3 升級:
 * - 五段硬體化架構: 腳位化/走線化/閘門化/量測化/過載保護
 * - LV1-LV9 九級防偽等級
 * - E/F 產品等級 (商用/深科技)
 * - A6 時序圖 (狀態機+調用序列)
 * - 電路故障模式完整分類
 * - 保護元件五種完整對照
 * 
 * v3.2 基礎 (保留):
 * - R13-R18 反詐欺紅線
 * - Z 軸真實性證明
 * - IAV 實作真實性協議
 * - BLDS 業務邏輯深度評分
 * - S 級武器級評等
 */

import type { SupportedLanguage } from './language-types.js';
import type { IAVRecord } from './z-axis.js';
import type { DoDStatus } from './dod.js';
import { SUPPORTED_EXTENSIONS } from './languages.js';

// Re-export split modules to keep `./types.js` as the stable public surface.
export type { SupportedLanguage } from './language-types.js';
export * from './z-axis.js';
export * from './dod.js';
export * from './hardwarization.js';

// =============================================================================
// Severity & Rule Types
// =============================================================================

/** 違規嚴重程度 */
export type Severity = 'error' | 'warning' | 'info';

/** 規則類別 */
export type RuleCategory = 
  | 'axiom'       // 公理 (A1-A8)
  | 'redline'     // 紅線 (R01-R12)
  | 'prohibition' // 禁止 (P01-P14)
  | 'gate';       // 關卡

/** 規則 ID 類型 */
export type AxiomId = 'A1' | 'A2' | 'A3' | 'A4' | 'A5' | 'A6' | 'A7' | 'A8';
export type RedlineId = 'R01' | 'R02' | 'R03' | 'R04' | 'R05' | 'R06' | 'R07' | 'R08' | 'R09' | 'R10' | 'R11' | 'R12' | 'R13' | 'R14' | 'R15' | 'R16' | 'R17' | 'R18';
export type ProhibitionId = 'P01' | 'P02' | 'P03' | 'P04' | 'P05' | 'P06' | 'P07' | 'P08' | 'P09' | 'P10' | 'P11' | 'P12' | 'P13' | 'P14';
export type GateId = 'Gate-In' | 'Gate-Mid' | 'Gate-Out' | 'Gate-Accept';

/** v3.3 四門禁 ID (硬體化) */
export type GateV33Id = 'G1' | 'G2' | 'G3' | 'G4';

/** v3.3 Pipeline 步驟 ID */
export type PipelineStepId = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;

/** v3.3 Pipeline 步驟結果 */
export interface PipelineStepResult {
  step: PipelineStepId;
  name: string;
  circuitTerm: string;
  passed: boolean;
  duration: number;
  evidencePath?: string;
  details: string;
  faultMode?: string;
  /**
   * Optional: raw evidence content intended to be written to `evidencePath`
   * by the CLI layer. Keep this small (think grep-like lines).
   */
  log?: string;
  /** Optional: structured violations for programmatic reporting/debugging. */
  violations?: Violation[];
  /** Optional: lightweight counters (e.g. { missing: 3, violations: 12 }). */
  stats?: Record<string, number>;
}

/** v3.3 Pipeline 結果 */
export interface PipelineResult {
  version: '3.3';
  timestamp: string;
  targetPath: string;
  productGrade: 'E' | 'F';
  protectionLevel: number;
  /** LV target derived from product grade (E=5, F=9). */
  protectionTarget: number;
  /** LV4 nonce (anti-replay). */
  nonce: string;
  /** LV5 SHA-256 hash of the proof inputs (anti-tamper). */
  evidenceHash: string;
  steps: PipelineStepResult[];
  gates: GateV33Result;
  dod: DoDStatus;
  waveform?: import('./engine/waveform.js').WaveformReport;
  passed: boolean;
  duration: number;
  evidenceDir: string;
}

/** v3.3 門禁結果 */
export interface GateV33Result {
  G1: { passed: boolean; tool: string; details: string };
  G2: { passed: boolean; tool: string; details: string };
  G3: { passed: boolean; tool: string; details: string };
  G4: { passed: boolean; tool: string; details: string };
  allPassed: boolean;
}

export type RuleId = AxiomId | RedlineId | ProhibitionId | GateId;

// =============================================================================
// Rule Definition
// =============================================================================

/** 規則定義 */
export interface Rule {
  /** 規則 ID */
  id: RuleId;
  /** 規則類別 */
  category: RuleCategory;
  /** 規則名稱 */
  name: string;
  /** 規則名稱（英文） */
  nameEn: string;
  /** 規則說明 */
  description: string;
  /** 嚴重程度 */
  severity: Severity;
  /** 違反後的處理動作 */
  action: string;
  /** 是否可自動檢測 */
  autoDetectable: boolean;
  /** 閾值（如適用） */
  threshold?: number | string;
  /** 檢測方法 */
  detectMethod?: 'regex' | 'ast' | 'heuristic' | 'llm' | 'manual' | 'integration';
}

// =============================================================================
// Violation
// =============================================================================

/** 違規記錄 */
export interface Violation {
  /** 規則 ID */
  ruleId: RuleId;
  /** 規則名稱 */
  ruleName: string;
  /** 嚴重程度 */
  severity: Severity;
  /** 檔案路徑 */
  file: string;
  /** 起始行號（1-indexed） */
  line: number;
  /** 起始列號（1-indexed） */
  column: number;
  /** 結束行號 */
  endLine?: number;
  /** 結束列號 */
  endColumn?: number;
  /** 違規訊息 */
  message: string;
  /** 相關程式碼片段 */
  snippet?: string;
  /** 建議修復方式 */
  suggestion?: string;
}

// =============================================================================
// Analysis Result
// =============================================================================

/** 單檔分析結果 */
export interface FileAnalysisResult {
  /** 檔案路徑 */
  file: string;
  /** 語言 */
  language: SupportedLanguage;
  /** 違規列表 */
  violations: Violation[];
  /** 分析耗時（毫秒） */
  duration: number;
  /** 行數統計 */
  stats: {
    totalLines: number;
    codeLines: number;
    commentLines: number;
    blankLines: number;
  };
}

/** 分析類別（軟配置） */
export type AnalysisCategory = 'security' | 'structure' | 'quality';

/** 整體分析結果 */
export interface AnalysisResult {
  /** 分析時間戳 */
  timestamp: string;
  /** 分析目標路徑 */
  targetPath: string;
  /** 檢查等級 */
  level: CheckLevel;
  /** 啟用的分析類別（軟配置） */
  categories?: AnalysisCategory[];
  /** 各檔案結果 */
  files: FileAnalysisResult[];
  /** 違規統計 */
  summary: {
    totalFiles: number;
    totalViolations: number;
    errorCount: number;
    warningCount: number;
    infoCount: number;
    byRule: Record<RuleId, number>;
  };
  /** 雙軸評分 (v2.x 向後兼容) */
  score?: DualAxisScore;
  /** 三軸評分 (v3.2) */
  triAxisScore?: TriAxisScore;
  /** 關卡狀態 */
  gates?: GateStatus;
  /** IAV 記錄 (v3.2) */
  iavRecords?: IAVRecord[];
  /** DoD 8 點驗證 (v3.2) */
  dod?: DoDStatus;
  /** 總耗時（毫秒） */
  duration: number;
}

// =============================================================================
// Dual-Axis Scoring (雙軸驗證)
// =============================================================================

/** X 軸：合規性 */
export interface ComplianceScore {
  /** 代碼規範 15% */
  codeStandard: number;
  /** 架構規範 20% */
  architecture: number;
  /** 安全規範 25% */
  security: number;
  /** 測試規範 20% */
  testing: number;
  /** 文檔規範 10% */
  documentation: number;
  /** 流程規範 10% */
  process: number;
  /** 加權總分 */
  total: number;
}

/** Y 軸：成果性 */
export interface OutcomeScore {
  /** 功能完成 30% */
  functionality: number;
  /** 質量達標 20% */
  quality: number;
  /** 性能達標 20% */
  performance: number;
  /** 可用性 15% */
  usability: number;
  /** 用戶滿意 15% */
  satisfaction: number;
  /** 加權總分 */
  total: number;
}

/** 雙軸評分 */
export interface DualAxisScore {
  x: ComplianceScore;
  y: OutcomeScore;
  /** 評級: A/B/C/D */
  grade: 'A' | 'B' | 'C' | 'D';
}

// =============================================================================
// Z-Axis: Authenticity (v3.2 真實性軸)
// =============================================================================

/** Z 軸：真實性評分 */
export interface AuthenticityScore {
  /** Z1: 反詐欺掃描 — fraud.log 中 R16/R17/R18 違規數 */
  fraudScanCount: number;
  /** Z2: IAV 五問 — 所有修復點 Q1-Q5 是否合格 */
  iavPass: boolean;
  /** Z3: BLDS 評分 — 最低分 */
  bldsMin: number;
  /** Z4: 數據源追溯 — 未追溯的數據輸出數 */
  untracedOutputs: number;
  /** Z 軸是否通過 */
  passed: boolean;
}

/** 三軸評分 (v3.2) */
export interface TriAxisScore {
  x: ComplianceScore;
  y: OutcomeScore;
  z: AuthenticityScore;
  /** 評級: S/A/B/C/D */
  grade: 'S' | 'A' | 'B' | 'C' | 'D';
}

// =============================================================================
// Gate Status (四關卡)
// =============================================================================

/** 關卡檢查項 */
export interface GateCheckItem {
  /** 檢查項名稱 */
  name: string;
  /** 是否必須 */
  required: boolean;
  /** 是否通過 */
  passed: boolean;
  /** 備註 */
  note?: string;
}

/** 單關卡狀態 */
export interface GateResult {
  /** 關卡 ID */
  id: GateId;
  /** 關卡名稱 */
  name: string;
  /** 是否通過 */
  passed: boolean;
  /** 檢查項列表 */
  items: GateCheckItem[];
}

/** 四關卡狀態 */
export interface GateStatus {
  gateIn: GateResult;
  gateMid: GateResult;
  gateOut: GateResult;
  gateAccept: GateResult;
}

// =============================================================================
// Language Support
// =============================================================================

/** 語言配置 */
export interface LanguageConfig {
  /** 語言 ID */
  id: SupportedLanguage;
  /** 顯示名稱 */
  name: string;
  /** 副檔名 */
  extensions: string[];
  /** Tree-sitter parser 模組名 */
  parserModule: string;
  /** 單行註解符號 */
  lineComment: string;
  /** 區塊註解 */
  blockComment?: { start: string; end: string };
}

// =============================================================================
// Configuration
// =============================================================================

/** 檢查等級 */
export type CheckLevel = 'B' | 'C' | 'D';

/** v3.3 版本標識 — 軟體工程硬體化 */
export const CODEQC_VERSION = '3.3';

/** CodeQC 配置 */
export interface CodeQCConfig {
  /** 檢查等級 */
  level: CheckLevel;
  /** 目標路徑 */
  include: string[];
  /** 排除路徑 */
  exclude: string[];
  /** 啟用的規則 */
  rules?: Partial<Record<RuleId, boolean | { enabled: boolean; threshold?: number }>>;
  /** 報告格式 */
  reporter: 'console' | 'json' | 'html';
  /** 輸出路徑 */
  output?: string;
  /** 是否啟用 AI 檢查 */
  ai?: {
    enabled: boolean;
    provider?: 'maidos-llm' | 'openai' | 'anthropic';
    apiKey?: string;
  };
  /** CI 模式（違規時退出碼非零） */
  ci?: boolean;
}

/** 預設配置 */
export const DEFAULT_CONFIG: CodeQCConfig = {
  level: 'D',
  include: SUPPORTED_EXTENSIONS.map(ext => `**/*${ext}`),
  exclude: ['**/node_modules/**', '**/dist/**', '**/build/**', '**/.git/**', '**/vendor/**'],
  reporter: 'console',
  ci: false,
};

// =============================================================================
// Plugin System
// =============================================================================

/** 語言支援定義（給插件用） */
export interface LanguageSupport {
  /** 語言 ID */
  id: string;
  /** 副檔名 */
  extensions: string[];
  /** Parser 載入函數 */
  parser: () => Promise<unknown>;
  /** 語言特定規則 */
  rules?: Rule[];
}

/** 插件定義 */
export interface Plugin {
  /** 插件名稱 */
  name: string;
  /** 版本 */
  version: string;
  /** 支援的語言 */
  languages: LanguageSupport[];
}

// =============================================================================
// Reporter
// =============================================================================

/** 報告器接口 */
export interface Reporter {
  /** 報告器名稱 */
  name: string;
  /** 生成報告 */
  report(result: AnalysisResult): string | Promise<string>;
}

// =============================================================================
// AST Node (簡化)
// =============================================================================

/** AST 節點位置 */
export interface Position {
  row: number;
  column: number;
}

/** AST 節點（Tree-sitter 簡化） */
export interface ASTNode {
  type: string;
  text: string;
  startPosition: Position;
  endPosition: Position;
  children: ASTNode[];
  parent?: ASTNode;
  namedChildren: ASTNode[];
}

// =============================================================================
// Checker Interface
// =============================================================================

/** 規則檢查器 */
export interface RuleChecker {
  /** 規則定義 */
  rule: Rule;
  /** 檢查函數（基於原始碼） */
  checkSource?: (source: string, file: string) => Violation[];
  /** 檢查函數（基於 AST） */
  checkAST?: (ast: ASTNode, source: string, file: string) => Violation[];
}
