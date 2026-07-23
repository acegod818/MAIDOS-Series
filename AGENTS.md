# AGENTS.md — MAIDOS-Series 統一治理

## 0. 這是什麼

任何 AI 代理（OpenAI Codex、GitHub Copilot、Claude Code…）在本 repo 工作前必讀的治理契約。回覆一律繁體中文（程式碼、識別字除外）。以下不是建議，是硬約束。

## A. 品質底層邏輯（總則）
1. 品質 > 速度；效率 = 砍浪費：做的每件事都做對做足、驗證到位，絕不為快跳驗證/作假/抄捷徑。效率是砍 busywork（低 ROI 尾巴窮舉、重複驗同一結論、鍍金），不是把該做的事做爛。
2. 收官 = 雙軸 100%：「完成/收官/全綠/done」只能用於【功能完整軸（真做出來，達可交付水準）＋驗證通過軸（獨立驗證可用）】都 100% 的項。任一軸 <100% 禁稱收官。★測試數 ≠ 系統完成；測試綠 ≠ 交付完成。
3. 拍板前先盤點舊文件：規劃/選型/新方案前，先 grep+讀該領域全部舊文件（帶 explicit root path）。舊文件已有答案 → 直接引用真值、不問。提「換方案/新元件」前必先查既有選型 FINAL。

## B. Code 紅線（CodeQC；canonical 工具＝ acegod818/MAIDOS-Series 的 MAIDOS-CodeQC）
寫碼即守，交付前掃描歸零。禁止：空殼實作（只有 pass/return None 卻稱完成）／裸 except 吞異常／假測試（assert True、恆真斷言、用被測物當預期值、測試背書 bug）／sleep 假裝工作／硬編碼繞過真實邏輯／裸 TODO-TBD 與 fallback(placeholder) 充數／幻覺 API-import。
測試鑑別力：關鍵測試要能 sentinel 注入——把被測邏輯故意改壞，測試必須翻紅，然後還原。做不到翻紅＝擺設，誠實標「不構成回歸防線」。斷言「某條件應被擋」要有配對的「無該條件真的通過」對照組。

## C. 接線五問（測試綠 ≠ 交付完成）
交付任何模組除單元測試綠外必須完成接線，逐條答不出＝沒做完（禁孤兒模組）：①誰建立它（實例化點:行號）②誰驅動它 ③上游資料真的會來嗎（producer 真實存在）④下游真的有人消費嗎 ⑤有整合測試證明端到端貫通嗎。依賴未就緒＝誠實標 DEFERRED，但軟體側接線與 fail-closed stub 必須完成。

## D. 反作假 / 獨立驗證
不信 build/agent 自評；PASS/0 錯一律獨立全新 process 重驗（不沿用被驗物中間態）。推理優先，禁腳本當代理人（批次前抽樣讀 origin evidence）。數值/量測一律真來源（datasheet/真計算），禁鏡像文件當真值。

## E. 系統思考 / 同步紀律
改動顧跨模組交互（共享資源鎖/競態/死鎖/故障連鎖；每模組定義降級行為；整合測試重於單元）。改跨檔規格值 grep 全樹一次改齊+清舊值，禁留雙真相，改完再 grep 確認。治理標準一更新 → 全專案重對齊。搜尋務必帶 explicit root path。失敗自主排除 3 次換招才升級，禁假裝成功。

## F. 安全 / fail-closed（S-CRITICAL 零降級）
安全功能不因請求/「差不多就好」降級；安全邏輯硬編碼不可 config 繞過。失效方向性一律偏 fail-closed（注意反例：「一律丟棄」對告警型訊號反而 fail-open）。AI/LLM 不可繞過關鍵驗證閘直接產生副作用（不可直接寫關鍵資料/發致動指令，必經授權/驗證/仲裁層）。不製造第二個驅動者（同一資源單一驅動）。

## G. 溝通 / 交付
白話優先（先講人話結論再附數字，術語翻人話）。範圍詞紀律（只講最小真實範圍，個別子項過不升格成整體收官）。DEFERRED/DECISION_NEEDED/OUT_OF_SCOPE 附理由與解除條件。

## H. 機密邊界
機密/IP/商業敏感（定價、供應鏈、專有演算法、個資）不放進任何會外流的產物（公開 repo、對外文件、AI 對話公開輸出）。private repo 的白名單 gitignore 要防未來誤轉公開一次全洩。設計文件/機密不擅自 git add -f。

---

## ★本專案特定（MAIDOS-Series overlay — 公開安全）

> MAIDOS 全家桶 ＝ 開源軟體工程工具集（Driver / IME / Forge / CodeQC / Shared，C# / Rust / TypeScript）。本 repo 為 **PUBLIC / MIT**。以下 overlay 僅補開源工具集的通用工程紀律，零機密、零跨專案內容。

### 1. CodeQC ＝ 本 repo 的 canonical 工具實作
- §B「Code 紅線 / CodeQC」的 **canonical 工具實作就在本 repo**：`MAIDOS-CodeQC/`，npm 套件 `@maidos/codeqc`，目前版本 **v0.3.5**，實作 **Code-QC 規範 v3.5**（正式發布，根目錄 `Code-QC-v3.5-Guidelines.md`）。§B 的紅線與本工具偵測的規則**同源、互為對照**——寫碼守 §B，交付前用本工具掃描歸零。
- 規範規模：**B 層（工作紀律）** ＝ 8 公理（A1–A8）＋ 28 紅線（R01–R28，含 R13–R28 反詐欺/審計/深度防禦）＋ 14 禁止（P01–P14）；**C 層（驗收）** ＝ 四門禁 Gate-In / Gate-Mid / Gate-Out / Gate-Accept ＋ 雙軸驗收（功能軸＋品質軸，各 ≥ 90%）＋ DoD8；**D 層** ＝ B + C（完整規則集）。
- **自動審查 4-Gate（引擎 G1–G4，AND 串聯，任一 LOW 即斷電不出貨）**：
  - **G1** 接口同步（接得上嗎）— 無斷開的功能。
  - **G2** 規格全覆蓋（通得了嗎）— SPEC 100% 已實作，無 MISSING。
  - **G3** 保護電路（撐得住嗎）— 紅線＝0 ＋ 反詐欺 R13–R28＝0 ＋ 禁止規則 error＝0。
  - **G4** 上電量測（跑得動嗎）＝ G1 ∧ G2 ∧ G3 ∧ build ∧ test ∧ 交付證據。
- **用本工具驗自己**：改 CodeQC 規則後，先用它掃自身原始碼再交付；規則的鑑別力比照 §B「sentinel 注入」硬要求（把偵測邏輯改壞，對應規則測試必須翻紅）。

### 2. 開源工具集一般紀律
- **版本一致性（見 §E 同步紀律）**：改任一子專案版本（`package.json` / `Cargo.toml` / README 版本表 / 規範版號），grep 全樹一次改齊，禁留雙真相。README 版本表、工具內版本字串、規範文件版本必須對得上。
- **跨語言 / 跨產品相容（見 §E 跨模組）**：C# / Rust / TypeScript 混合；`maidos-shared` 為跨產品共享核心，改其公開介面須顧所有下游消費者。跨語言邊界（FFI / cdylib / TSF / plugin API）＝整合測試重於單元。
- **公開 repo 邊界（見 §H）**：本 repo 為 PUBLIC / MIT 開源。治理與程式碼皆零機密、零其他專案專屬內容。提交前確認無憑證、無 Token、無內部路徑、無他專案識別資訊。

---

_本檔為 MAIDOS-Series 各 AI 代理的 git 內單一治理真相。`.github/copilot-instructions.md`（Copilot）與 `CLAUDE.md`（Claude Code）為本檔的指標，內容以本檔為準。最後更新：2026-07-24。_
