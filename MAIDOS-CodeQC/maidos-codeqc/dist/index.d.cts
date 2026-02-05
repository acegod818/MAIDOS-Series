/**
 * CodeQC v3.3 Engine — Waveform (三通道示波器)
 *
 * CH1 (Y軸) 功能波形: 規格映射+測試+覆蓋+補完
 * CH2 (X軸) 品質波形: 編譯+Lint+紅線+安全
 * CH3 (Z軸) 真實波形: 詐欺掃描+IAV+BLDS+追溯
 *
 * 每個通道輸出: amplitude (0-100) + status (PASS/FAIL/WARN)
 */
type WaveformStatus = 'PASS' | 'FAIL' | 'WARN' | 'SKIP';
interface ChannelReading {
    id: string;
    name: string;
    amplitude: number;
    status: WaveformStatus;
    evidence?: string;
    detail?: string;
}
interface WaveformChannel {
    channel: 'CH1_Y' | 'CH2_X' | 'CH3_Z';
    name: string;
    readings: ChannelReading[];
    overall: WaveformStatus;
    score: number;
}
interface WaveformReport {
    timestamp: string;
    channels: [WaveformChannel, WaveformChannel, WaveformChannel];
    allPass: boolean;
    compositeScore: number;
}

/**
 * CodeQC language ids (single source of truth for the union type).
 *
 * Keep this file <500 lines (self-proof gate).
 */
/** 支援的語言（43）。避免「CLI 收進來但 analyzer 靜默跳過」的規格詐欺。 */
type SupportedLanguage = 'typescript' | 'javascript' | 'python' | 'rust' | 'go' | 'java' | 'kotlin' | 'scala' | 'groovy' | 'clojure' | 'csharp' | 'fsharp' | 'vbnet' | 'swift' | 'objc' | 'dart' | 'c' | 'cpp' | 'zig' | 'nim' | 'php' | 'ruby' | 'shell' | 'powershell' | 'perl' | 'lua' | 'sql' | 'r' | 'julia' | 'yaml' | 'json' | 'toml' | 'xml' | 'elixir' | 'haskell' | 'ocaml' | 'erlang' | 'cobol' | 'abap' | 'plsql' | 'fortran' | 'vba' | 'rpg';

/**
 * Code-QC Z-axis (v3.2/v3.3): Authenticity Protocol Helpers
 *
 * Keep this file <500 lines (self-proof gate).
 */
/** IAV 五問回答 */
interface IAVAnswer {
    /** Q1: 數據從哪來？ */
    q1_dataSource: string;
    /** Q2: 調用了什麼 API/Service/DB？ */
    q2_callChain: string;
    /** Q3: 輸出如何依賴輸入？ */
    q3_inputOutput: string;
    /** Q4: 錯誤時怎麼處理？ */
    q4_errorHandling: string;
    /** Q5: 如何證明是真的？ */
    q5_proof: string;
}
/** IAV 不合格回答模式 */
declare const IAV_DISQUALIFIERS: Record<keyof IAVAnswer, string[]>;
/** IAV 修復記錄 */
interface IAVRecord {
    /** 修復點: 檔案:行號 */
    location: string;
    /** 五問回答 */
    answers: IAVAnswer;
    /** BLDS 評分 */
    bldsScore: number;
    /** 判定 */
    verdict: 'PASS' | 'FAIL';
}
/** BLDS 等級 */
type BLDSLevel = 0 | 1 | 2 | 3 | 4 | 5;
/** BLDS 等級名稱 */
declare const BLDS_LEVELS: Record<BLDSLevel, string>;
/** BLDS 最低門禁要求 (v3.3 baseline) */
declare const BLDS_GATE_MINIMUM: BLDSLevel;

/**
 * Code-QC v3.3 — DoD (Definition of Done) definitions
 *
 * Keep this file <500 lines (self-proof gate).
 */
/** DoD 檢查項 */
interface DoDItem {
    /** 項目編號 1-8 */
    id: number;
    /** 項目名稱 */
    name: string;
    /** 驗證方式 */
    verification: string;
    /** 是否通過 */
    passed: boolean;
    /** 證據路徑 */
    evidencePath?: string;
}
/** DoD 狀態 */
interface DoDStatus {
    /** 8 個檢查項 */
    items: DoDItem[];
    /** 是否全部通過 */
    missionComplete: boolean;
}
/** DoD 8 點定義 (v3.3) */
declare const DOD_DEFINITIONS: Omit<DoDItem, 'passed' | 'evidencePath'>[];

/**
 * Code-QC v3.3 — Hardwarization constants (ABCD worldview)
 *
 * Keep this file <500 lines (self-proof gate).
 */
/** 五段式硬體化架構 */
declare const HARDWARIZATION_PILLARS: {
    readonly PINOUT: {
        readonly id: 1;
        readonly name: "腳位化";
        readonly en: "Pinout";
        readonly maps: "A";
        readonly question: "腳位定義完整嗎？";
        readonly faultIfMissing: "短路";
    };
    readonly WIRING: {
        readonly id: 2;
        readonly name: "走線化";
        readonly en: "Wiring";
        readonly maps: "D+A";
        readonly question: "Pipeline固定且不可跳步嗎？";
        readonly faultIfMissing: "斷路";
    };
    readonly LOGIC_GATE: {
        readonly id: 3;
        readonly name: "閘門化";
        readonly en: "Logic Gate";
        readonly maps: "C:G1-G4";
        readonly question: "閘門全HIGH嗎？";
        readonly faultIfMissing: "閘門不開";
    };
    readonly INSTRUMENT: {
        readonly id: 4;
        readonly name: "量測化";
        readonly en: "Instrumentation";
        readonly maps: "C:YXZ";
        readonly question: "波形出了嗎？";
        readonly faultIfMissing: "無波形";
    };
    readonly PROTECTION: {
        readonly id: 5;
        readonly name: "保護化";
        readonly en: "Protection";
        readonly maps: "B+Z";
        readonly question: "保護電路完好嗎？";
        readonly faultIfMissing: "熔斷";
    };
};
/** ABCD 文件體系 — 電路觀映射 */
declare const CIRCUIT_WORLDVIEW: {
    readonly A: {
        readonly name: "規格標準";
        readonly nameEn: "Schematic";
        readonly circuit: "電路圖";
        readonly question: "電路圖有沒有畫好？";
        readonly doc: "CodeQC_v3.3_A.md";
        readonly pillar: "PINOUT";
    };
    readonly B: {
        readonly name: "工作紀律";
        readonly nameEn: "Protection Circuit";
        readonly circuit: "保護電路";
        readonly question: "保護電路有沒有裝？";
        readonly doc: "CodeQC_v3.3_B.md";
        readonly pillar: "PROTECTION";
    };
    readonly C: {
        readonly name: "證明標準";
        readonly nameEn: "Power-On Waveform";
        readonly circuit: "上電波形";
        readonly question: "上電波形有沒有出？";
        readonly doc: "CodeQC_v3.3_C.md";
        readonly pillar: "INSTRUMENT+LOGIC_GATE";
    };
    readonly D: {
        readonly name: "測試台";
        readonly nameEn: "Test Bench";
        readonly circuit: "上電測試台";
        readonly question: "測試台能不能重跑？";
        readonly doc: "CodeQC_v3.3_D.md";
        readonly pillar: "WIRING+ALL";
    };
};
/** 門禁電路標註 — AND Gate 邏輯 */
declare const GATE_CIRCUIT_LABELS: {
    readonly G1: {
        readonly circuit: "腳位接觸測試";
        readonly en: "Pin Contact Test";
        readonly tool: "萬用表";
        readonly phase: "接得上嗎？";
        readonly logic: "AND Gate";
    };
    readonly G2: {
        readonly circuit: "走線連通測試";
        readonly en: "Trace Continuity Test";
        readonly tool: "蜂鳴檔";
        readonly phase: "通得了嗎？";
        readonly logic: "AND Gate";
    };
    readonly G3: {
        readonly circuit: "保護電路測試";
        readonly en: "Protection Circuit Test";
        readonly tool: "故障注入器";
        readonly phase: "撐得住嗎？";
        readonly logic: "AND Gate";
    };
    readonly G4: {
        readonly circuit: "上電量測";
        readonly en: "Power-On Measurement";
        readonly tool: "示波器";
        readonly phase: "跑得動嗎？";
        readonly logic: "AND Gate";
    };
};
/** 五層保護體系 (L1-L5) */
declare const PROTECTION_LAYERS: {
    readonly L1_FUSE: {
        readonly name: "保險絲";
        readonly en: "Fuse";
        readonly behavior: "過流即斷";
        readonly maps: "R01-R18";
    };
    readonly L2_REGULATOR: {
        readonly name: "穩壓器";
        readonly en: "Regulator";
        readonly behavior: "限制範圍";
        readonly maps: "P01-P14";
    };
    readonly L3_GROUND: {
        readonly name: "接地";
        readonly en: "Ground";
        readonly behavior: "基準參考";
        readonly maps: "A1-A8";
    };
    readonly L4_ESD: {
        readonly name: "ESD防護";
        readonly en: "ESD Protection";
        readonly behavior: "防靜電擊穿";
        readonly maps: "Z1-Z4";
    };
    readonly L5_ANTI_REPLAY: {
        readonly name: "防回放";
        readonly en: "Anti-Replay";
        readonly behavior: "封死假跑";
        readonly maps: "Nonce+Hash+Verifier";
    };
};
/** 電路故障模式 → 代碼缺陷分類 */
declare const FAULT_MODES: {
    readonly SHORT_CIRCUIT: {
        readonly severity: "critical";
        readonly label: "短路—假實現/Mock冒充";
        readonly rules: readonly ["R13", "R16", "R17", "R18"];
    };
    readonly OPEN_CIRCUIT: {
        readonly severity: "critical";
        readonly label: "斷路—TODO/未實現";
        readonly rules: readonly ["R15"];
    };
    readonly FAKE_SIGNAL: {
        readonly severity: "critical";
        readonly label: "偽信號—拼貼舊證據/假跑";
        readonly protection: "L5";
    };
    readonly COLD_SOLDER_JOINT: {
        readonly severity: "major";
        readonly label: "虛焊—空catch/靜默失敗";
        readonly rules: readonly ["R05", "R14"];
    };
    readonly LEAKAGE: {
        readonly severity: "major";
        readonly label: "漏電—無限制資源/洩漏";
        readonly rules: readonly ["R09"];
    };
    readonly CROSSTALK: {
        readonly severity: "moderate";
        readonly label: "串擾—全局狀態/緊耦合";
        readonly rules: readonly ["P07", "P08"];
    };
    readonly OVERVOLTAGE: {
        readonly severity: "moderate";
        readonly label: "過壓—超長函數/深嵌套";
        readonly rules: readonly ["P05", "P06", "P10"];
    };
    readonly NOISE: {
        readonly severity: "minor";
        readonly label: "噪聲—魔法數字/命名不清";
        readonly rules: readonly ["P04", "P09"];
    };
};
/** 電路速查卡（嵌入 LLM system prompt 用） */
declare const CIRCUIT_QUICK_CARD: string;
/** 防偽等級 LV1-LV9 (v3.3 新增) */
declare const PROTECTION_LEVELS: {
    readonly LV1: {
        readonly name: "保險絲保護";
        readonly nameEn: "Redline Fuse";
        readonly scope: "All projects";
        readonly tier: "basic";
    };
    readonly LV2: {
        readonly name: "穩壓器限制";
        readonly nameEn: "Prohibition Regulator";
        readonly scope: "All projects";
        readonly tier: "basic";
    };
    readonly LV3: {
        readonly name: "防詐欺掃描";
        readonly nameEn: "Anti-Fraud ESD";
        readonly scope: "All projects";
        readonly tier: "basic";
    };
    readonly LV4: {
        readonly name: "防回放鎖";
        readonly nameEn: "Nonce/Challenge";
        readonly scope: "Multi-model collaboration";
        readonly tier: "enhanced";
    };
    readonly LV5: {
        readonly name: "防篡改封印";
        readonly nameEn: "Hash/Merkle";
        readonly scope: "Outsourcing acceptance";
        readonly tier: "enhanced";
    };
    readonly LV6: {
        readonly name: "獨立檢測站";
        readonly nameEn: "Verifier Replay";
        readonly scope: "High-reliability projects";
        readonly tier: "independent";
    };
    readonly LV7: {
        readonly name: "可信模組";
        readonly nameEn: "Attestation/TEE";
        readonly scope: "Finance/Medical";
        readonly tier: "independent";
    };
    readonly LV8: {
        readonly name: "交叉對抗";
        readonly nameEn: "Cross-Model Adversarial";
        readonly scope: "Deep-tech";
        readonly tier: "formal";
    };
    readonly LV9: {
        readonly name: "形式化證明";
        readonly nameEn: "Formal Verification";
        readonly scope: "Military/Aerospace";
        readonly tier: "formal";
    };
};
/** 產品等級 (v3.3 新增) */
declare const PRODUCT_GRADES: {
    readonly E: {
        readonly name: "商用級";
        readonly nameEn: "Commercial Grade";
        readonly protection: "LV1-5";
        readonly gates: "G1-G4";
        readonly description: "一般商用產品";
    };
    readonly F: {
        readonly name: "深科技級";
        readonly nameEn: "Deep-Tech Grade";
        readonly protection: "LV1-9";
        readonly gates: "G1-G4+Formal";
        readonly description: "金融/醫療/軍規/航太";
    };
};
/** 保護元件完整對照 (v3.3 統一) */
declare const PROTECTION_COMPONENTS: {
    readonly FUSE: {
        readonly name: "保險絲";
        readonly nameEn: "Fuse";
        readonly behavior: "過流即斷";
        readonly maps: "Redlines R01-R18";
        readonly tier: "LV1";
    };
    readonly REGULATOR: {
        readonly name: "穩壓器";
        readonly nameEn: "Regulator";
        readonly behavior: "限制範圍";
        readonly maps: "Prohibitions P01-P14";
        readonly tier: "LV2";
    };
    readonly GROUND: {
        readonly name: "接地";
        readonly nameEn: "Ground";
        readonly behavior: "基準參考";
        readonly maps: "Axioms A1-A8";
        readonly tier: "LV1";
    };
    readonly ESD: {
        readonly name: "ESD防護";
        readonly nameEn: "ESD Protection";
        readonly behavior: "防靜電擊穿";
        readonly maps: "Anti-Fraud Z1-Z5";
        readonly tier: "LV3";
    };
    readonly THERMAL: {
        readonly name: "溫度保護";
        readonly nameEn: "Thermal Shutdown";
        readonly behavior: "過熱關斷";
        readonly maps: "Handover Tags";
        readonly tier: "LV1";
    };
};
/** 硬體化速查卡（完整版，嵌入 LLM system prompt） */
declare const HARDWARE_QUICK_CARD: "\n\u2554\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2557\n\u2551         Code-QC v3.3  \u8EDF\u9AD4\u5DE5\u7A0B\u786C\u9AD4\u5316  \u901F\u67E5\u5361               \u2551\n\u2551         \u4F60\u662F\u65BD\u5DE5\u968A\uFF0C\u4E0D\u662F\u5BEB\u624B\u3002\u63A5\u96FB\u8DEF\uFF0C\u51FA\u6CE2\u5F62\u3002              \u2551\n\u2560\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2563\n\u2551  \u2776 \u8173\u4F4D\u5316 (A): \u63A5\u53E3\u5B9A\u7FA9+\u985E\u578B\u7C3D\u540D+\u932F\u8AA4\u8173\u4F4D+\u72C0\u614B\u6A5F         \u2551\n\u2551  \u2777 \u8D70\u7DDA\u5316 (D): build\u2192lint\u2192test\u2192proof\u2192gate\u2192ship            \u2551\n\u2551  \u2778 \u9598\u9580\u5316 (B): R01-18=0 + P01-14\u5728\u9650 + A1-8\u7121\u9055\u53CD        \u2551\n\u2551  \u2779 \u91CF\u6E2C\u5316 (C): G1\u842C\u7528\u8868+G2\u8702\u9CF4+G3\u6545\u969C\u6CE8\u5165+G4\u793A\u6CE2\u5668       \u2551\n\u2551  \u277A \u904E\u8F09\u4FDD\u8B77: LV1-3\u57FA\u790E + LV4-5\u9632\u507D + LV6-9\u7368\u7ACB\u9A57\u8B49       \u2551\n\u2551  DoD: (1)\u5BE6\u73FE(2)\u88DC\u5B8C(3)\u898F\u683C(4)\u540C\u6B65(5)\u7DE8\u8B6F(6)\u4EA4\u4ED8(7)\u771F\u5BE6(8)\u9632\u8A50 \u2551\n\u2551  \u5168\u904E=MISSION COMPLETE  \u4EFB\u4E00\u4E0D\u904E=REJECTED                  \u2551\n\u255A\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u255D\n";

/** 違規嚴重程度 */
type Severity = 'error' | 'warning' | 'info';
/** 規則類別 */
type RuleCategory = 'axiom' | 'redline' | 'prohibition' | 'gate';
/** 規則 ID 類型 */
type AxiomId = 'A1' | 'A2' | 'A3' | 'A4' | 'A5' | 'A6' | 'A7' | 'A8';
type RedlineId = 'R01' | 'R02' | 'R03' | 'R04' | 'R05' | 'R06' | 'R07' | 'R08' | 'R09' | 'R10' | 'R11' | 'R12' | 'R13' | 'R14' | 'R15' | 'R16' | 'R17' | 'R18';
type ProhibitionId = 'P01' | 'P02' | 'P03' | 'P04' | 'P05' | 'P06' | 'P07' | 'P08' | 'P09' | 'P10' | 'P11' | 'P12' | 'P13' | 'P14';
type GateId = 'Gate-In' | 'Gate-Mid' | 'Gate-Out' | 'Gate-Accept';
/** v3.3 四門禁 ID (硬體化) */
type GateV33Id = 'G1' | 'G2' | 'G3' | 'G4';
/** v3.3 Pipeline 步驟 ID */
type PipelineStepId = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;
/** v3.3 Pipeline 步驟結果 */
interface PipelineStepResult {
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
interface PipelineResult {
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
    waveform?: WaveformReport;
    passed: boolean;
    duration: number;
    evidenceDir: string;
}
/** v3.3 門禁結果 */
interface GateV33Result {
    G1: {
        passed: boolean;
        tool: string;
        details: string;
    };
    G2: {
        passed: boolean;
        tool: string;
        details: string;
    };
    G3: {
        passed: boolean;
        tool: string;
        details: string;
    };
    G4: {
        passed: boolean;
        tool: string;
        details: string;
    };
    allPassed: boolean;
}
type RuleId = AxiomId | RedlineId | ProhibitionId | GateId;
/** 規則定義 */
interface Rule {
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
/** 違規記錄 */
interface Violation {
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
/** 單檔分析結果 */
interface FileAnalysisResult {
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
type AnalysisCategory = 'security' | 'structure' | 'quality';
/** 整體分析結果 */
interface AnalysisResult {
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
/** X 軸：合規性 */
interface ComplianceScore {
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
interface OutcomeScore {
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
interface DualAxisScore {
    x: ComplianceScore;
    y: OutcomeScore;
    /** 評級: A/B/C/D */
    grade: 'A' | 'B' | 'C' | 'D';
}
/** Z 軸：真實性評分 */
interface AuthenticityScore {
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
interface TriAxisScore {
    x: ComplianceScore;
    y: OutcomeScore;
    z: AuthenticityScore;
    /** 評級: S/A/B/C/D */
    grade: 'S' | 'A' | 'B' | 'C' | 'D';
}
/** 關卡檢查項 */
interface GateCheckItem {
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
interface GateResult {
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
interface GateStatus {
    gateIn: GateResult;
    gateMid: GateResult;
    gateOut: GateResult;
    gateAccept: GateResult;
}
/** 語言配置 */
interface LanguageConfig {
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
    blockComment?: {
        start: string;
        end: string;
    };
}
/** 檢查等級 */
type CheckLevel = 'B' | 'C' | 'D';
/** v3.3 版本標識 — 軟體工程硬體化 */
declare const CODEQC_VERSION = "3.3";
/** CodeQC 配置 */
interface CodeQCConfig {
    /** 檢查等級 */
    level: CheckLevel;
    /** 目標路徑 */
    include: string[];
    /** 排除路徑 */
    exclude: string[];
    /** 啟用的規則 */
    rules?: Partial<Record<RuleId, boolean | {
        enabled: boolean;
        threshold?: number;
    }>>;
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
declare const DEFAULT_CONFIG: CodeQCConfig;
/** 語言支援定義（給插件用） */
interface LanguageSupport {
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
interface Plugin {
    /** 插件名稱 */
    name: string;
    /** 版本 */
    version: string;
    /** 支援的語言 */
    languages: LanguageSupport[];
}
/** 報告器接口 */
interface Reporter {
    /** 報告器名稱 */
    name: string;
    /** 生成報告 */
    report(result: AnalysisResult): string | Promise<string>;
}
/** AST 節點位置 */
interface Position {
    row: number;
    column: number;
}
/** AST 節點（Tree-sitter 簡化） */
interface ASTNode {
    type: string;
    text: string;
    startPosition: Position;
    endPosition: Position;
    children: ASTNode[];
    parent?: ASTNode;
    namedChildren: ASTNode[];
}
/** 規則檢查器 */
interface RuleChecker {
    /** 規則定義 */
    rule: Rule;
    /** 檢查函數（基於原始碼） */
    checkSource?: (source: string, file: string) => Violation[];
    /** 檢查函數（基於 AST） */
    checkAST?: (ast: ASTNode, source: string, file: string) => Violation[];
}

/**
 * Code-QC v2.4 - C 驗收標準
 * §1 四關卡 (Gate-In, Gate-Mid, Gate-Out, Gate-Accept)
 */

interface GateDefinition {
    id: GateId;
    name: string;
    nameEn: string;
    description: string;
    items: Omit<GateCheckItem, 'passed'>[];
}
declare const GATES: GateDefinition[];
declare function evaluateGate(gateId: GateId, results: Record<string, boolean>): GateResult;
declare function createGateStatus(gateIn: Record<string, boolean>, gateMid: Record<string, boolean>, gateOut: Record<string, boolean>, gateAccept: Record<string, boolean>): GateStatus;
declare function calculateComplianceScore(scores: Omit<ComplianceScore, 'total'>): ComplianceScore;
declare function calculateOutcomeScore(scores: Omit<OutcomeScore, 'total'>): OutcomeScore;
declare function calculateDualAxisScore(xScores: Omit<ComplianceScore, 'total'>, yScores: Omit<OutcomeScore, 'total'>): DualAxisScore;
declare function generateGateChecklist(gateId: GateId): string;
declare function generateAllGatesChecklist(): string;
declare const HANDOVER_TEMPLATE = "# \u4EA4\u63A5\u6587\u6A94\n\n## \u6458\u8981\n- **\u9805\u76EE**: <\u9805\u76EE\u540D>\n- **\u65E5\u671F**: <\u65E5\u671F>\n- **\u72C0\u614B**: \uD83D\uDFE2/\uD83D\uDFE1/\uD83D\uDD34\n\n## \u9032\u5EA6\n- [x] \u5DF2\u5B8C\u6210\u9805\u76EE\n- [ ] \u9032\u884C\u4E2D - XX%\n\n## \u963B\u585E\u9805\n| \u963B\u585E | \u9700\u8981 |\n|:-----|:-----|\n| <\u554F\u984C> | <\u8CC7\u6E90> |\n\n## \u4E0B\u4E00\u6B65\n1. **P0**: <\u6700\u9AD8\u512A\u5148>\n2. **P1**: <\u6B21\u512A\u5148>\n\n## \u6CE8\u610F\u4E8B\u9805\n\u26A0\uFE0F <\u91CD\u8981\u63D0\u9192>\n";
declare const HANDOVER_TAGS: {
    tag: string;
    description: string;
    format: string;
}[];
declare function extractHandoverTags(source: string): {
    tag: string;
    line: number;
    content: string;
}[];

/**
 * Code-QC v2.4 - B 工作紀律
 * §3 十四禁止 (P01-P14)
 *
 * 實作狀態：
 * ⚠️ P01 過度工程 (需 LLM)
 * ⚠️ P02 過早優化 (需 LLM)
 * ✅ P03 複製粘貼 (基礎檢測)
 * ✅ P04 魔法數字 (regex)
 * ✅ P05 超長函數 (AST)
 * ✅ P06 深層嵌套 (AST)
 * ✅ P07 全局狀態 (regex)
 * ⚠️ P08 緊耦合 (需 LLM)
 * ✅ P09 無意義命名 (regex)
 * ✅ P10 過長參數 (regex)
 * ⚠️ P11 混合抽象 (需 LLM)
 * ✅ P12 註釋代碼 (regex)
 * ✅ P13 TODO 堆積 (regex)
 * ✅ P14 依賴膨脹 (基礎檢測)
 *
 * 總計：10/14 已實作
 */

interface Prohibition extends Rule {
    id: ProhibitionId;
    category: 'prohibition';
    implemented: boolean;
    requiresIntegration?: string;
}
declare const PROHIBITIONS: Prohibition[];
declare function getProhibition(id: ProhibitionId): Prohibition | undefined;
declare const PROHIBITION_CHECKERS: RuleChecker[];
declare function checkProhibitions(source: string, file: string): Violation[];

/**
 * Code-QC v3.3 - B 工作紀律
 * §2 十八紅線 (R01-R18)
 */

interface Redline extends Rule {
    id: RedlineId;
    category: 'redline';
    implemented: boolean;
    requiresIntegration?: string;
}
declare const REDLINES: Redline[];
declare function getRedline(id: RedlineId): Redline | undefined;

/**
 * Code-QC v2.4 - B 工作紀律
 * §1 八公理 (A1-A8)
 *
 * 公理為開發原則指導，非自動檢測項目
 * 用於 AI Executor 提示和報告參考
 */

/** 公理定義 */
interface Axiom extends Rule {
    id: AxiomId;
    category: 'axiom';
    /** 優先級（1 = 最高） */
    priority: number;
}
/**
 * 八公理列表
 * 優先級：A6 > A1 > A2 > A4 > A5 > A3 > A7 > A8
 */
declare const AXIOMS: Axiom[];
/** 按優先級排序的公理 */
declare const AXIOMS_BY_PRIORITY: Axiom[];
/** 取得公理 */
declare function getAxiom(id: AxiomId): Axiom | undefined;
/** 格式化公理提示（給 AI Executor 用） */
declare function formatAxiomsPrompt(): string;

declare const REDLINE_CHECKERS: RuleChecker[];
/** 反詐欺專用檢查器 (R16+R17+R18) */
declare const ANTI_FRAUD_CHECKERS: RuleChecker[];
/** 執行反詐欺掃描 — 生成 fraud.log 的資料 */
declare function checkAntifraud(source: string, file: string): Violation[];
declare function checkRedlines(source: string, file: string): Violation[];

/**
 * 執行規則檢查 (v3.2)
 * @param source 原始碼
 * @param file 檔案路徑
 * @param level 檢查等級 (B/C/D)
 */
declare function checkRules(source: string, file: string, level?: CheckLevel): Violation[];
/**
 * 執行反詐欺專用掃描 (v3.2 Z軸)
 * 檢測 R16空方法 + R17詐欺物件 + R18繞道實作
 */
declare function checkFraud(source: string, file: string): Violation[];

/**
 * MAIDOS CodeQC - Core Analyzer
 */

declare function detectLanguage(file: string): SupportedLanguage | null;
declare function isSupported(file: string): boolean;
declare function analyzeFile(source: string, file: string, level: CheckLevel): FileAnalysisResult;
interface AnalyzeOptions {
    files: Array<{
        path: string;
        content: string;
    }>;
    level: CheckLevel;
    targetPath: string;
}
declare function analyze(options: AnalyzeOptions): AnalysisResult;
declare function quickCheck(source: string, file: string): Violation[];

/**
 * Console Reporter
 * 輸出彩色終端報告
 */

declare const consoleReporter: Reporter;

/**
 * JSON Reporter
 * 輸出結構化 JSON 報告
 */

declare const jsonReporter: Reporter;

/**
 * HTML Reporter
 * 輸出可視化 HTML 報告
 */

declare const htmlReporter: Reporter;

/**
 * Reporters Module
 */

declare const reporters: Record<string, Reporter>;
declare function getReporter(name: string): Reporter;

/**
 * Code-QC v3.3 — Pipeline 引擎 (❷走線化)
 *
 * 十步走線：build → lint → test → proof → gate → ship
 * 走線固定不可跳步，斷一條即熔斷。
 *
 * 對照 D.md §8 Pipeline v3.3
 */

interface PipelineInput {
    /** 掃描目標路徑 */
    targetPath: string;
    /** 檔案列表 [{path, content}] */
    files: Array<{
        path: string;
        content: string;
    }>;
    /** 產品等級 E=商用 / F=深科技 */
    grade: 'E' | 'F';
    /** evidence 輸出目錄 */
    evidenceDir: string;
    /** 外部命令結果 (build/lint/test/coverage) — 由 CLI 層注入 */
    externalResults?: {
        build?: {
            exitCode: number;
            log: string;
        };
        lint?: {
            exitCode: number;
            log: string;
        };
        test?: {
            exitCode: number;
            log: string;
            passed: number;
            failed: number;
        };
        coverage?: {
            percentage: number;
            log: string;
        };
        audit?: {
            exitCode: number;
            log: string;
            critical: number;
            high: number;
        };
        package?: {
            exitCode: number;
            log: string;
        };
        run?: {
            exitCode: number;
            log: string;
        };
    };
    /** SPEC 文件路徑 (G2 走線連通用) */
    specPath?: string;
    /** SPEC 函數列表 (從 SPEC.md 提取的期望函數) */
    specFunctions?: string[];
    /** SPEC 完成度 (從 SPEC.md checkbox 提取) */
    specChecklist?: {
        total: number;
        done: number;
    };
    /** Z 軸真實性/追溯證據 (由 CLI 解析 evidence/*.log 注入) */
    proof?: {
        iav?: {
            passed: boolean;
            passedCount: number;
            failedCount: number;
            details: string;
        };
        blds?: {
            minScore: number;
            threshold: number;
            passed: boolean;
            details: string;
        };
        datasource?: {
            untraced: number;
            passed: boolean;
            details: string;
        };
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
declare function runPipeline(input: PipelineInput): PipelineResult;
/**
 * 格式化 Pipeline 報告 (Console 用)
 */
declare function formatPipelineReport(result: PipelineResult): string;

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

interface GateV33Input {
    /** Step 8 結果 (G1 腳位接觸) */
    step8Result: PipelineStepResult;
    /** Step 9 結果 (G2 走線連通) */
    step9Result: PipelineStepResult;
    /** Step 10 結果 (G4 驗收/交付證據) */
    step10Result: PipelineStepResult;
    /** 全部十步結果 (G3 保護電路用) */
    allSteps: PipelineStepResult[];
    /** 原始檔案 (G3 故障注入二次驗證用) */
    files: Array<{
        path: string;
        content: string;
    }>;
}
/**
 * 執行 v3.3 四門禁 (AND Gate 串聯)
 * G1 → G2 → G3 → G4，全 HIGH 才 PASS
 */
declare function runGatesV33(input: GateV33Input): GateV33Result;

/**
 * Code-QC v3.3 — Evidence 收集器 + DoD 8點判定器 (❹量測化)
 *
 * 對照 C.md §7 + D.md §7
 *
 * Proof Pack = evidence/ 目錄下的完整 LOG 集合
 * DoD 8 點 = 每點對應一個 evidence 文件
 * 8/8 = MISSION COMPLETE · <8 = REJECTED
 */

interface EvidenceCollection {
    /** 各 evidence 文件路徑 → 內容/狀態 */
    logs: Record<string, EvidenceLog>;
    /** Pipeline 步驟結果 (用於 DoD 判定) */
    steps: PipelineStepResult[];
    /** 門禁結果 */
    gates: GateV33Result;
    /** evidence 目錄 */
    dir: string;
}
interface EvidenceLog {
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
declare function collectEvidence(steps: PipelineStepResult[], gates: GateV33Result, input: {
    evidenceDir: string;
    files: Array<{
        path: string;
        content: string;
    }>;
    externalResults?: {
        build?: {
            exitCode: number;
            log: string;
        };
        lint?: {
            exitCode: number;
            log: string;
        };
        test?: {
            exitCode: number;
            log: string;
            passed: number;
            failed: number;
        };
        coverage?: {
            percentage: number;
            log: string;
        };
        audit?: {
            exitCode: number;
            log: string;
            critical: number;
            high: number;
        };
        package?: {
            exitCode: number;
            log: string;
        };
        run?: {
            exitCode: number;
            log: string;
        };
    };
    specChecklist?: {
        total: number;
        done: number;
    };
    proof?: {
        iav?: {
            passed: boolean;
            passedCount: number;
            failedCount: number;
            details: string;
        };
        blds?: {
            minScore: number;
            threshold: number;
            passed: boolean;
            details: string;
        };
        datasource?: {
            untraced: number;
            passed: boolean;
            details: string;
        };
    };
}): EvidenceCollection;
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
declare function judgeDod(evidence: EvidenceCollection): DoDStatus;
/**
 * 產出 Proof Pack 目錄結構 (給 CLI 用，實際寫入由 CLI 層執行)
 */
declare function generateProofPackManifest(evidence: EvidenceCollection): string;

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

interface ProtectionContext {
    /** LV4 anti-replay nonce */
    nonce?: string;
    /** LV5 anti-tamper hash (sha256 hex) */
    evidenceHash?: string;
}
interface ProtectionCheckResult {
    level: number;
    name: string;
    passed: boolean;
    details: string;
}
interface ProtectionReport {
    grade: 'E' | 'F';
    targetLevel: number;
    achievedLevel: number;
    checks: ProtectionCheckResult[];
    allPassed: boolean;
}
/**
 * 執行防偽等級檢查
 * E 等級 = LV1-5 全過
 * F 等級 = LV1-9 全過
 */
declare function checkProtection(grade: 'E' | 'F', steps: PipelineStepResult[], ctx?: ProtectionContext): ProtectionReport;
/**
 * 解析產品等級 → 實際防偽等級數字
 * 用於 PipelineResult.protectionLevel
 */
declare function resolveProtectionLevel(grade: 'E' | 'F', steps: PipelineStepResult[], ctx?: ProtectionContext): number;

/**
 * MAIDOS CodeQC
 * Code Quality Control implementing Code-QC v3.3 — 軟體工程硬體化
 *
 * v3.3 升級: 五段硬體化架構 + LV1-LV9防偽 + E/F產品等級
 * v3.2 基礎: 反詐欺紅線(R16-R18) + 三軸證明(Z軸) + IAV + BLDS + DoD 8點
 *
 * @packageDocumentation
 */

declare const VERSION = "0.3.3";

export { ANTI_FRAUD_CHECKERS, AXIOMS, AXIOMS_BY_PRIORITY, type AnalysisResult, type AnalyzeOptions, type AuthenticityScore, type AxiomId, type BLDSLevel, BLDS_GATE_MINIMUM, BLDS_LEVELS, CIRCUIT_QUICK_CARD, CIRCUIT_WORLDVIEW, CODEQC_VERSION, type CheckLevel, type CodeQCConfig, type ComplianceScore, DEFAULT_CONFIG, DOD_DEFINITIONS, type DoDItem, type DoDStatus, type DualAxisScore, type EvidenceCollection, type EvidenceLog, FAULT_MODES, type FileAnalysisResult, GATES, GATE_CIRCUIT_LABELS, type GateCheckItem, type GateId, type GateResult, type GateStatus, type GateV33Id, type GateV33Input, type GateV33Result, HANDOVER_TAGS, HANDOVER_TEMPLATE, HARDWARE_QUICK_CARD, HARDWARIZATION_PILLARS, type IAVAnswer, type IAVRecord, IAV_DISQUALIFIERS, type LanguageConfig, type LanguageSupport, type OutcomeScore, PRODUCT_GRADES, PROHIBITIONS, PROHIBITION_CHECKERS, PROTECTION_COMPONENTS, PROTECTION_LAYERS, PROTECTION_LEVELS, type PipelineInput, type PipelineResult, type PipelineStepId, type PipelineStepResult, type Plugin, type ProhibitionId, type ProtectionCheckResult, type ProtectionReport, REDLINES, REDLINE_CHECKERS, type RedlineId, type Reporter, type Rule, type RuleCategory, type RuleChecker, type RuleId, type Severity, type SupportedLanguage, type TriAxisScore, VERSION, type Violation, analyze, analyzeFile, calculateComplianceScore, calculateDualAxisScore, calculateOutcomeScore, checkAntifraud, checkFraud, checkProhibitions, checkProtection, checkRedlines, checkRules, collectEvidence, consoleReporter, createGateStatus, detectLanguage, evaluateGate, extractHandoverTags, formatAxiomsPrompt, formatPipelineReport, generateAllGatesChecklist, generateGateChecklist, generateProofPackManifest, getAxiom, getProhibition, getRedline, getReporter, htmlReporter, isSupported, jsonReporter, judgeDod, quickCheck, reporters, resolveProtectionLevel, runGatesV33, runPipeline };
