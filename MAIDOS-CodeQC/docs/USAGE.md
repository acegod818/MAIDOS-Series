# CodeQC v3.2 MaxHard - 使用路徑（不改你的 A/A1/A2/B/C/D/E/F）

## 1) 你在做什麼（選擇 E 或 F）
- 商用產品：用 **E = A1 + D**
- 深科技產品：用 **F = A2 + D**
- 只要讓模型施工不偷懶：用 **D（B+C）**，並要求 A（A0）與 Proof Pack 必須 PASS

## 2) 最小必備（A0）
A0 的目標是把驗收變成「用戶旅程 + AC + 契約/狀態 + Ops 最低集合」，避免“看起來能跑”的假完工。
必備文件與門檻：見 `CodeQC_v3.0_A.md`

## 3) 驗收只看結果（C）
C 的 Proof Pack v3（Nonce + Hashes + Merkle + 可選簽章）是「結果證明論」的載體。
驗收與反造假條款：見 `CodeQC_v3.0_C.md`


## 3.1) Verifier 模式（更硬：strict-level 6/7/8）
> 目的：把「信任」從 Runner（施工端/LLM）移到 Verifier（裁判端），降低自簽自演與舊證據回放。

- **strict=6（外部 challenge）**：要求 `proof/challenge.json`（由 verifier 發）且簽章有效，並強制 `manifest.nonce == challenge.nonce`。
- **strict=7（verifier 重跑 + verdict）**：要求 `proof/replay/manifest.json`（verifier 重跑產生）與 `proof/verdict.json`（verifier 簽章裁決）。
- **strict=8（attestation 介面）**：要求 `proof/attestation/attest.json`（可由 CI/TEE/TPM 出具）與 `proof/attestation/policy.json`（白名單政策），probe 會做簽章與欄位匹配驗證。
  - 注意：這裡提供的是「介面與政策驗證」；真正的硬體 attestation 需要你在 CI/Runner 端接上對應供應商的 attestor。

最短流程（推薦）：
- 先準備 `proof/trust.json`（信任錨，見 templates/trust/TRUST_TEMPLATE.json）

1) Verifier 端發 challenge：`python3 codeqc_probe.py challenge --out proof/challenge.json`
2) Runner 端跑 pipeline（把 challenge.json 帶入 proof/，並用其 nonce 產生 manifest）
3) Verifier 端重跑（由你的 CI/獨立環境產生 `proof/replay/manifest.json`）
4) Verifier 端出 verdict：`python3 codeqc_probe.py verdict`
5) 最後驗收：`python3 codeqc_probe.py proof --strict-level 7`

## 4) 一鍵流程（建議）
- WSL/Linux：`./maidos_pipeline.sh`
- Windows：`maidos_pipeline.bat`

Pipeline 會依序執行：
1. A（spec）→ 產生 `A.log`
2. B（redlines/build/unit/integration）→ 通用門檻
3. C（e2e/proof）→ 產生 Proof Pack
4. proof 驗證（strict level 預設依 probe 參數/腳本設定）


## 5) LV9（只在 F=A2+D）
- 設定 `CODEQC_PROFILE=F`（或 `CODEQC_LV9=1`）會在 Proof Pack 之後追加 LV9 Gate。
- LV9 規格：見 `CodeQC_v3.1_LV9.md`。
