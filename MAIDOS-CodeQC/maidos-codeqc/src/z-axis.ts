/**
 * Code-QC Z-axis (v3.2/v3.3): Authenticity Protocol Helpers
 *
 * Keep this file <500 lines (self-proof gate).
 */

// =============================================================================
// IAV: Implementation Authenticity Verification (v3.2/v3.3)
// =============================================================================

/** IAV 五問回答 */
export interface IAVAnswer {
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
export const IAV_DISQUALIFIERS: Record<keyof IAVAnswer, string[]> = {
  q1_dataSource: ['硬編碼', '默認值', '不知道', 'hardcoded', 'default', 'unknown'],
  q2_callChain: ['沒有', '直接return', 'none', 'direct return'],
  q3_inputOutput: ['不依賴', '固定值', 'not dependent', 'fixed'],
  q4_errorHandling: ['忽略', 'return默認', 'ignore', 'return default'],
  q5_proof: ['編譯通過', '沒報錯', 'compiles', 'no error'],
};

/** IAV 修復記錄 */
export interface IAVRecord {
  /** 修復點: 檔案:行號 */
  location: string;
  /** 五問回答 */
  answers: IAVAnswer;
  /** BLDS 評分 */
  bldsScore: number;
  /** 判定 */
  verdict: 'PASS' | 'FAIL';
}

// =============================================================================
// BLDS: Business Logic Depth Score (v3.2/v3.3)
// =============================================================================

/** BLDS 等級 */
export type BLDSLevel = 0 | 1 | 2 | 3 | 4 | 5;

/** BLDS 等級名稱 */
export const BLDS_LEVELS: Record<BLDSLevel, string> = {
  0: '詐欺 — 空方法/return默認/硬編碼',
  1: '假貨 — 有代碼但不訪問真實數據源',
  2: '半成品 — 訪問數據源但缺錯誤處理',
  3: '合格 — 完整調用鏈+錯誤處理+正確轉換',
  4: '優秀 — 合格+邊界處理+性能考量',
  5: '武器級 — 優秀+可測+可重播+審計日誌',
};

/** BLDS 最低門禁要求 (v3.3 baseline) */
export const BLDS_GATE_MINIMUM: BLDSLevel = 3;

