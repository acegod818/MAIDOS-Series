/**
 * Code-QC v3.3 — Hardwarization constants (ABCD worldview)
 *
 * Keep this file <500 lines (self-proof gate).
 */

/** 五段式硬體化架構 */
export const HARDWARIZATION_PILLARS = {
  PINOUT:     { id: 1, name: '腳位化', en: 'Pinout', maps: 'A', question: '腳位定義完整嗎？', faultIfMissing: '短路' },
  WIRING:     { id: 2, name: '走線化', en: 'Wiring', maps: 'D+A', question: 'Pipeline固定且不可跳步嗎？', faultIfMissing: '斷路' },
  LOGIC_GATE: { id: 3, name: '閘門化', en: 'Logic Gate', maps: 'C:G1-G4', question: '閘門全HIGH嗎？', faultIfMissing: '閘門不開' },
  INSTRUMENT: { id: 4, name: '量測化', en: 'Instrumentation', maps: 'C:YXZ', question: '波形出了嗎？', faultIfMissing: '無波形' },
  PROTECTION: { id: 5, name: '保護化', en: 'Protection', maps: 'B+Z', question: '保護電路完好嗎？', faultIfMissing: '熔斷' },
} as const;

/** ABCD 文件體系 — 電路觀映射 */
export const CIRCUIT_WORLDVIEW = {
  A: { name: '規格標準', nameEn: 'Schematic', circuit: '電路圖', question: '電路圖有沒有畫好？', doc: 'CodeQC_v3.3_A.md', pillar: 'PINOUT' },
  B: { name: '工作紀律', nameEn: 'Protection Circuit', circuit: '保護電路', question: '保護電路有沒有裝？', doc: 'CodeQC_v3.3_B.md', pillar: 'PROTECTION' },
  C: { name: '證明標準', nameEn: 'Power-On Waveform', circuit: '上電波形', question: '上電波形有沒有出？', doc: 'CodeQC_v3.3_C.md', pillar: 'INSTRUMENT+LOGIC_GATE' },
  D: { name: '測試台', nameEn: 'Test Bench', circuit: '上電測試台', question: '測試台能不能重跑？', doc: 'CodeQC_v3.3_D.md', pillar: 'WIRING+ALL' },
} as const;

/** 門禁電路標註 — AND Gate 邏輯 */
export const GATE_CIRCUIT_LABELS = {
  G1: { circuit: '腳位接觸測試', en: 'Pin Contact Test', tool: '萬用表', phase: '接得上嗎？', logic: 'AND Gate' },
  G2: { circuit: '走線連通測試', en: 'Trace Continuity Test', tool: '蜂鳴檔', phase: '通得了嗎？', logic: 'AND Gate' },
  G3: { circuit: '保護電路測試', en: 'Protection Circuit Test', tool: '故障注入器', phase: '撐得住嗎？', logic: 'AND Gate' },
  G4: { circuit: '上電量測', en: 'Power-On Measurement', tool: '示波器', phase: '跑得動嗎？', logic: 'AND Gate' },
} as const;

/** 五層保護體系 (L1-L5) */
export const PROTECTION_LAYERS = {
  L1_FUSE:        { name: '保險絲', en: 'Fuse', behavior: '過流即斷', maps: 'R01-R18' },
  L2_REGULATOR:   { name: '穩壓器', en: 'Regulator', behavior: '限制範圍', maps: 'P01-P14' },
  L3_GROUND:      { name: '接地', en: 'Ground', behavior: '基準參考', maps: 'A1-A8' },
  L4_ESD:         { name: 'ESD防護', en: 'ESD Protection', behavior: '防靜電擊穿', maps: 'Z1-Z4' },
  L5_ANTI_REPLAY: { name: '防回放', en: 'Anti-Replay', behavior: '封死假跑', maps: 'Nonce+Hash+Verifier' },
} as const;

/** 電路故障模式 → 代碼缺陷分類 */
export const FAULT_MODES = {
  SHORT_CIRCUIT:     { severity: 'critical' as const, label: '短路—假實現/Mock冒充', rules: ['R13','R16','R17','R18'] },
  OPEN_CIRCUIT:      { severity: 'critical' as const, label: '斷路—TODO/未實現', rules: ['R15'] },
  FAKE_SIGNAL:       { severity: 'critical' as const, label: '偽信號—拼貼舊證據/假跑', protection: 'L5' },
  COLD_SOLDER_JOINT: { severity: 'major' as const, label: '虛焊—空catch/靜默失敗', rules: ['R05','R14'] },
  LEAKAGE:           { severity: 'major' as const, label: '漏電—無限制資源/洩漏', rules: ['R09'] },
  CROSSTALK:         { severity: 'moderate' as const, label: '串擾—全局狀態/緊耦合', rules: ['P07','P08'] },
  OVERVOLTAGE:       { severity: 'moderate' as const, label: '過壓—超長函數/深嵌套', rules: ['P05','P06','P10'] },
  NOISE:             { severity: 'minor' as const, label: '噪聲—魔法數字/命名不清', rules: ['P04','P09'] },
} as const;

/** 電路速查卡（嵌入 LLM system prompt 用） */
export const CIRCUIT_QUICK_CARD = [
  'Code-QC v3.3 · 軟體工程硬體化',
  '❶腳位化: SPEC+type定義+信號流 (沒腳位=短路)',
  '❷走線化: Pipeline固定不可跳步 (不通=斷路)',
  '❸閘門化: G1-G4 AND全HIGH才開 (一LOW=斷電)',
  '❹量測化: YXZ三軸+evidence/LOG (沒波形=沒跑)',
  '❺保護化: L1保險絲+L2穩壓+L3接地+L4ESD+L5防回放 (偷懶=熔斷)',
  '五段全過=MISSION COMPLETE | 任一熔斷=REJECTED',
].join('\n') as string;

/** 防偽等級 LV1-LV9 (v3.3 新增) */
export const PROTECTION_LEVELS = {
  LV1: { name: '保險絲保護', nameEn: 'Redline Fuse', scope: 'All projects', tier: 'basic' as const },
  LV2: { name: '穩壓器限制', nameEn: 'Prohibition Regulator', scope: 'All projects', tier: 'basic' as const },
  LV3: { name: '防詐欺掃描', nameEn: 'Anti-Fraud ESD', scope: 'All projects', tier: 'basic' as const },
  LV4: { name: '防回放鎖', nameEn: 'Nonce/Challenge', scope: 'Multi-model collaboration', tier: 'enhanced' as const },
  LV5: { name: '防篡改封印', nameEn: 'Hash/Merkle', scope: 'Outsourcing acceptance', tier: 'enhanced' as const },
  LV6: { name: '獨立檢測站', nameEn: 'Verifier Replay', scope: 'High-reliability projects', tier: 'independent' as const },
  LV7: { name: '可信模組', nameEn: 'Attestation/TEE', scope: 'Finance/Medical', tier: 'independent' as const },
  LV8: { name: '交叉對抗', nameEn: 'Cross-Model Adversarial', scope: 'Deep-tech', tier: 'formal' as const },
  LV9: { name: '形式化證明', nameEn: 'Formal Verification', scope: 'Military/Aerospace', tier: 'formal' as const },
} as const;

/** 產品等級 (v3.3 新增) */
export const PRODUCT_GRADES = {
  E: { name: '商用級', nameEn: 'Commercial Grade', protection: 'LV1-5', gates: 'G1-G4', description: '一般商用產品' },
  F: { name: '深科技級', nameEn: 'Deep-Tech Grade', protection: 'LV1-9', gates: 'G1-G4+Formal', description: '金融/醫療/軍規/航太' },
} as const;

/** 保護元件完整對照 (v3.3 統一) */
export const PROTECTION_COMPONENTS = {
  FUSE:       { name: '保險絲', nameEn: 'Fuse', behavior: '過流即斷', maps: 'Redlines R01-R18', tier: 'LV1' },
  REGULATOR:  { name: '穩壓器', nameEn: 'Regulator', behavior: '限制範圍', maps: 'Prohibitions P01-P14', tier: 'LV2' },
  GROUND:     { name: '接地', nameEn: 'Ground', behavior: '基準參考', maps: 'Axioms A1-A8', tier: 'LV1' },
  ESD:        { name: 'ESD防護', nameEn: 'ESD Protection', behavior: '防靜電擊穿', maps: 'Anti-Fraud Z1-Z5', tier: 'LV3' },
  THERMAL:    { name: '溫度保護', nameEn: 'Thermal Shutdown', behavior: '過熱關斷', maps: 'Handover Tags', tier: 'LV1' },
} as const;

/** 硬體化速查卡（完整版，嵌入 LLM system prompt） */
export const HARDWARE_QUICK_CARD = `
╔════════════════════════════════════════════════════════════╗
║         Code-QC v3.3  軟體工程硬體化  速查卡               ║
║         你是施工隊，不是寫手。接電路，出波形。              ║
╠════════════════════════════════════════════════════════════╣
║  ❶ 腳位化 (A): 接口定義+類型簽名+錯誤腳位+狀態機         ║
║  ❷ 走線化 (D): build→lint→test→proof→gate→ship            ║
║  ❸ 閘門化 (B): R01-18=0 + P01-14在限 + A1-8無違反        ║
║  ❹ 量測化 (C): G1萬用表+G2蜂鳴+G3故障注入+G4示波器       ║
║  ❺ 過載保護: LV1-3基礎 + LV4-5防偽 + LV6-9獨立驗證       ║
║  DoD: (1)實現(2)補完(3)規格(4)同步(5)編譯(6)交付(7)真實(8)防詐 ║
║  全過=MISSION COMPLETE  任一不過=REJECTED                  ║
╚════════════════════════════════════════════════════════════╝
` as const;

