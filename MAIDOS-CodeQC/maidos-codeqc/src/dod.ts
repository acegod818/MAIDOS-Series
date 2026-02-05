/**
 * Code-QC v3.3 — DoD (Definition of Done) definitions
 *
 * Keep this file <500 lines (self-proof gate).
 */

/** DoD 檢查項 */
export interface DoDItem {
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
export interface DoDStatus {
  /** 8 個檢查項 */
  items: DoDItem[];
  /** 是否全部通過 */
  missionComplete: boolean;
}

/** DoD 8 點定義 (v3.3) */
export const DOD_DEFINITIONS: Omit<DoDItem, 'passed' | 'evidencePath'>[] = [
  { id: 1, name: '實現證明', verification: 'redline.log = 0 (無斷路/短路)' },
  { id: 2, name: '補完證明', verification: 'impl.log + mapping.log (走線連通)' },
  { id: 3, name: '規格證明', verification: 'SPEC 100% + 0 MISSING (電路圖完整)' },
  { id: 4, name: '同步證明', verification: 'sync.log = 0 (腳位接觸良好)' },
  { id: 5, name: '編譯證明', verification: 'build.log 0e/0w (焊接品質)' },
  { id: 6, name: '交付證明', verification: 'package.log + run.log (可上電)' },
  { id: 7, name: '真實性證明', verification: 'iav.log PASS + BLDS ≥ 3 (信號真實)' },
  { id: 8, name: '反詐欺證明', verification: 'fraud.log = 0 (ESD通過)' },
];

