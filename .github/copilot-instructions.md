# GitHub Copilot 指令 — MAIDOS-Series

**完整治理見根目錄 [`AGENTS.md`](../AGENTS.md)（必讀，Codex/Copilot/Claude 共同契約）。** 本檔為指標＋關鍵紅線摘要。

MAIDOS 全家桶 — 開源軟體工程工具集（**PUBLIC / MIT**，C# / Rust / TypeScript）。回覆一律**繁體中文**（程式碼、識別字除外）。

## 動手前必守的硬約束（完整版見 AGENTS.md）

1. **品質 > 速度；效率 = 砍浪費**：絕不為快跳驗證／作假／抄捷徑；效率是砍 busywork，不是把該做的事做爛。
2. **Code 紅線（CodeQC）**：禁空殼實作／裸 except 吞異常／assert True 假測試／sleep 假裝／硬編碼繞過／裸 TODO-TBD／fallback placeholder／幻覺 API-import。本 repo `MAIDOS-CodeQC` ＝ 此規範的 **canonical 工具**（`@maidos/codeqc` v0.3.5，Code-QC v3.5）；交付前用它掃描歸零。
3. **測試鑑別力**：關鍵測試要能 sentinel 注入翻紅，否則不構成回歸防線，誠實標明。
4. **接線五問**（測試綠 ≠ 交付完成）：誰建立／誰驅動／上游真來／下游真消費／整合測試證明。禁孤兒模組。
5. **收官 = 雙軸 100%**：功能完整軸＋驗證通過軸都 100% 才能稱完成；測試數 ≠ 系統完成。
6. **獨立驗證**：不信 build／agent 自評；PASS／0 錯一律獨立全新 process 重驗。
7. **安全 / fail-closed**：安全邏輯硬編碼不可 config 繞過；失效一律偏 fail-closed（告警型「一律丟棄」反而 fail-open）；AI 不繞關鍵驗證閘；不製造第二個驅動者。
8. **同步紀律**：改跨檔版本／規格值 grep 全樹一起改＋清舊值，禁留雙真相；搜尋帶 explicit root path。
9. **公開 repo 邊界**：本 repo 為 PUBLIC；治理與程式碼零機密、零跨專案內容；提交前確認無憑證／Token／內部路徑。
