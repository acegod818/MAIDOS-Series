/**
 * Code-QC v3.5 - B 工作紀律
 * §2 二十八紅線 (R01-R28)
 */

import type { Rule, RedlineId } from '../types.js';

export interface Redline extends Rule {
  id: RedlineId;
  category: 'redline';
  implemented: boolean;
  requiresIntegration?: string;
}

export const REDLINES: Redline[] = [
  { id: 'R01', category: 'redline', name: '硬編碼憑證', nameEn: 'Hardcoded Credentials', description: '代碼中硬編碼密碼、密鑰、Token', severity: 'error', action: '🔴 立即刪除，輪換', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R02', category: 'redline', name: '跳過安全檢查', nameEn: 'Bypass Security', description: '繞過認證、授權、輸入驗證', severity: 'error', action: '🔴 回滾，審查', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R03', category: 'redline', name: '刪除審計日誌', nameEn: 'Delete Audit Logs', description: '刪除或篡改審計記錄', severity: 'error', action: '🔴 立即恢復', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R04', category: 'redline', name: '未授權數據訪問', nameEn: 'Unauthorized Data Access', description: '訪問超出權限的數據', severity: 'error', action: '🔴 撤銷，審查', autoDetectable: false, detectMethod: 'llm', implemented: false, requiresIntegration: 'LLM 語義分析' },
  { id: 'R05', category: 'redline', name: '忽略錯誤處理', nameEn: 'Ignore Error Handling', description: '空 catch、吞異常', severity: 'error', action: '🔴 立即修復', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R06', category: 'redline', name: '直接操作生產', nameEn: 'Direct Production Access', description: '未經審批修改生產環境', severity: 'error', action: '🔴 回滾', autoDetectable: false, detectMethod: 'integration', implemented: false, requiresIntegration: 'CI/CD 系統' },
  { id: 'R07', category: 'redline', name: '關閉安全功能', nameEn: 'Disable Security', description: '關閉防火牆、TLS、加密', severity: 'error', action: '🔴 立即恢復', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R08', category: 'redline', name: '使用已知漏洞', nameEn: 'Known Vulnerabilities', description: '使用有漏洞的依賴版本', severity: 'error', action: '🔴 立即升級', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R09', category: 'redline', name: '無限制資源', nameEn: 'Unlimited Resources', description: '無限制 API 調用、查詢、循環', severity: 'error', action: '🔴 添加限制', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R10', category: 'redline', name: '明文傳輸敏感', nameEn: 'Plaintext Sensitive Data', description: '明文傳輸密碼、PII、財務', severity: 'error', action: '🔴 加密傳輸', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R11', category: 'redline', name: '跳過代碼審查', nameEn: 'Skip Code Review', description: '未審查代碼進入主分支', severity: 'error', action: '🔴 回滾，補審查', autoDetectable: false, detectMethod: 'integration', implemented: false, requiresIntegration: 'Git/VCS 系統' },
  { id: 'R12', category: 'redline', name: '偽造測試結果', nameEn: 'Fake Test Results', description: '偽造、跳過、硬編碼測試', severity: 'error', action: '🔴 重新測試', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R13', category: 'redline', name: '假實現', nameEn: 'Fake Implementation', description: 'return true/null/空物件無實際邏輯', severity: 'error', action: '🔴 立即重寫', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R14', category: 'redline', name: '靜默失敗', nameEn: 'Silent Failure', description: 'catch 不 log 也不 re-throw', severity: 'error', action: '🔴 立即修復', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R15', category: 'redline', name: 'TODO殘留', nameEn: 'TODO Residue', description: 'todo!/unimplemented!/TODO 進入提交', severity: 'error', action: '🔴 立即清除', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R16', category: 'redline', name: '空方法', nameEn: 'Empty Method', description: '方法簽名正確但方法體空或僅return默認值', severity: 'error', action: '🔴 砍掉重寫', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R17', category: 'redline', name: '詐欺物件', nameEn: 'Fraud Object', description: '物件結構正確但數據硬編碼/不來自真實數據源', severity: 'error', action: '🔴 砍掉重寫', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R18', category: 'redline', name: '繞道實作', nameEn: 'Bypass Implementation', description: '跳過應使用的API/DB/Config用假資料替代', severity: 'error', action: '🔴 砍掉重寫', autoDetectable: true, detectMethod: 'regex', implemented: true },
  // === v3.4 新增: 審計補漏六規則 (R19-R24) ===
  { id: 'R19', category: 'redline', name: '固定字串回傳', nameEn: 'Hardcoded String Return', description: '函數回傳寫死字串/固定值而非真實計算結果', severity: 'error', action: '🔴 接入真實數據源', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R20', category: 'redline', name: '模板複製灌水', nameEn: 'Template Copy-Paste', description: '大量檔案結構相同僅改名字，功能全空', severity: 'error', action: '🔴 實作或移除', autoDetectable: true, detectMethod: 'regex+heuristic', implemented: true },
  { id: 'R21', category: 'redline', name: '假認證', nameEn: 'Fake Auth', description: '收了憑證參數但未用於簽名/驗證，或壓 unused warning', severity: 'error', action: '🔴 實作真實認證', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R22', category: 'redline', name: '資料量不足', nameEn: 'Insufficient Data', description: '字典/對照表/配置數據量遠低於實用門檻', severity: 'error', action: '🔴 補齊數據', autoDetectable: true, detectMethod: 'heuristic', implemented: true },
  { id: 'R23', category: 'redline', name: '永遠失敗', nameEn: 'Always-Fail Path', description: '函數永遠回 Err/Failure/false，等於功能不存在', severity: 'error', action: '🔴 實作或標記 unsupported', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R24', category: 'redline', name: '自白註解', nameEn: 'Self-Confessing Comment', description: '註解承認是假實作（In a real implementation/simplified/for now）', severity: 'error', action: '🔴 實作真實邏輯', autoDetectable: true, detectMethod: 'regex', implemented: true },
  // === v3.5 新增: 深掃補漏四規則 (R25-R28) ===
  { id: 'R25', category: 'redline', name: '日誌灌水', nameEn: 'Log Stuffing', description: '函數體只有 log 語句 + trivial return，無實際業務邏輯', severity: 'error', action: '🔴 實作真實邏輯', autoDetectable: true, detectMethod: 'regex+heuristic', implemented: true },
  { id: 'R26', category: 'redline', name: '幽靈異步', nameEn: 'Phantom Async', description: 'async fn 內部沒有任何 .await 調用，假裝是異步操作', severity: 'error', action: '🔴 加入真實 .await 或移除 async', autoDetectable: true, detectMethod: 'regex', implemented: true },
  { id: 'R27', category: 'redline', name: '錯誤洗白', nameEn: 'Error Laundering', description: '在安全相關操作中用 unwrap_or_default() 靜默吞掉錯誤', severity: 'error', action: '🔴 用 ? 傳播錯誤或明確處理', autoDetectable: true, detectMethod: 'regex+context', implemented: true },
  { id: 'R28', category: 'redline', name: '複製軍團', nameEn: 'Clone Army', description: '跨檔案完全相同的函數體，結構膨脹灌水', severity: 'error', action: '🔴 抽取共用函數', autoDetectable: true, detectMethod: 'heuristic', implemented: true },
];

export function getRedline(id: RedlineId): Redline | undefined {
  return REDLINES.find(r => r.id === id);
}

export function getImplementedRedlines(): Redline[] {
  return REDLINES.filter(r => r.implemented);
}

export function getUnimplementedRedlines(): Redline[] {
  return REDLINES.filter(r => !r.implemented);
}

export function getRedlineStats(): { total: number; implemented: number; unimplemented: string[] } {
  const unimpl = REDLINES.filter(r => !r.implemented);
  return {
    total: REDLINES.length,
    implemented: REDLINES.filter(r => r.implemented).length,
    unimplemented: unimpl.map(r => `${r.id}: ${r.name} (需要: ${r.requiresIntegration})`),
  };
}

