# CLAUDE.md — MAIDOS-Series

**治理單一真相（git 內）＝根目錄 [`AGENTS.md`](AGENTS.md)（必讀）。** 本檔為指標＋快速索引，內容以 AGENTS.md 為準。

MAIDOS 全家桶 — 開源軟體工程工具集（Driver / IME / Forge / CodeQC / Shared，C# / Rust / TypeScript）。**PUBLIC / MIT。** 回覆一律**繁體中文**（程式碼、識別字除外）。

## 快速索引（完整規則見 AGENTS.md）

- **§A 品質總則**：品質>速度・效率=砍浪費／收官=雙軸100%／拍板前先盤舊文件
- **§B Code 紅線（CodeQC）**：空殼／裸except／假測試／sleep／硬編碼繞過／裸TODO-TBD／幻覺import ＋ sentinel 鑑別力。canonical 工具＝本 repo `MAIDOS-CodeQC`（`@maidos/codeqc` **v0.3.5**，Code-QC 規範 **v3.5**）
- **§C 接線五問**：測試綠 ≠ 交付完成；禁孤兒模組
- **§D 反作假**：不信自評、獨立全新 process 重驗、推理優先禁腳本代理
- **§E 系統思考/同步**：跨模組、版本/規格值全樹一次改齊、治理更新全樹重對齊、搜尋帶 root path、失敗3次升級
- **§F 安全/fail-closed**：零降級／硬編碼不可 config 繞過／失效偏 fail-closed（注意告警型反例）／AI 不繞驗證閘／反雙控制器
- **§G 溝通/交付**：白話優先、範圍詞紀律、DEFERRED 附解除條件
- **§H 機密邊界**：本 repo 為 PUBLIC；零機密、零跨專案內容
- **★本專案特定**：CodeQC canonical 工具版本／自動審查 4-Gate（G1–G4）；開源版本一致性／跨語言相容

## CodeQC 自檢（開發時）

```bash
cd MAIDOS-CodeQC
npm install && npm run build
./codeqc.cmd .\src      # Windows；Linux/Mac 用 ./codeqc.sh ./src
```

改 CodeQC 規則後先用它掃自身原始碼；關鍵規則要能 sentinel 注入翻紅，否則不構成回歸防線（見 §B）。
