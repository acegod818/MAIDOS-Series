# MAIDOS CodeQC — 路線圖 v2.3

> **版本**: v2.3 (Session 5 最終驗證)
> **日期**: 2026-02-02
> **引擎**: Code-QC v3.3 (軟體工程硬體化)
> **形態**: 單品 (wocao 武器配件) + SaaS (對外營業)

---

## 1. 產品定位

```
┌─────────────────────────────┐
│    CodeQC Engine Core v3.3  │ ← 共用引擎
│  Pipeline + Gates + LV1-9   │
│  Waveform + DoD + ProofPack │
└──────────┬──────────┬───────┘
           │          │
  ┌────────┘          └────────┐
  ▼                            ▼
┌──────────────┐    ┌──────────────┐
│  單品 (wocao) │    │  SaaS (雲端)  │
│  CLI on USB   │    │  REST API    │
│  離線可攜     │    │  Dashboard   │
│  接案秀報告   │    │  多租戶+計費  │
└──────────────┘    └──────────────┘
```

---

## 2. 現況盤點 (2026-02-02 全驗證通過)

### 2.1 自證結果

```
TypeScript 編譯: 0 errors, 0 warnings       ✅
Vitest 測試:    167/167 passed               ✅
CLI Pipeline:   9/10 + G4/4 + DoD 8/8       ✅ MISSION COMPLETE
dist/ build:    ESM + CJS + DTS              ✅
Dashboard UI:   6 Tab 全功能 (API 連通驗證)  ✅
```

### 2.2 SDK 模組清單 (真實行數)

| 模組 | 路徑 | 行數 | 狀態 |
|:-----|:-----|:----:|:----:|
| 類型+常量 | src/types.ts | 670 | ✅ |
| 紅線規則 | src/rules/b-redlines.ts | 639 | ✅ 15/18 |
| 禁止規則 | src/rules/b-prohibitions.ts | 394 | ✅ 10/14 |
| 公理定義 | src/rules/b-axioms.ts | 148 | ✅ |
| 舊版門禁 | src/rules/c-gates.ts | 283 | ⚠️ 保留相容 |
| 規則入口 | src/rules/index.ts | 82 | ✅ |
| **Pipeline引擎** | src/engine/pipeline.ts | 543 | ✅ 十步走線+外部注入 |
| G1-G4門禁 | src/engine/gates-v33.ts | 167 | ✅ |
| Evidence收集 | src/engine/evidence.ts | 252 | ✅ |
| 防偽引擎 | src/engine/protection.ts | 180 | ✅ LV1-5 |
| 三通道示波器 | src/engine/waveform.ts | 206 | ✅ |
| DoD評估器 | src/engine/dod.ts | 142 | ✅ |
| 引擎入口 | src/engine/index.ts | 27 | ✅ |
| REST API | src/server/app.ts | 213 | ✅ 6端點 |
| 分析器 | src/analyzer.ts | 201 | ✅ |
| CLI主入口 | src/cli.ts | 493 | ✅ scan+路由 |
| CLI子命令 | src/commands.ts | 245 | ✅ pipeline+serve+外部注入 |
| Console報告 | src/reporter/console.ts | 168 | ✅ |
| HTML報告 | src/reporter/html.ts | 433 | ✅ |
| JSON報告 | src/reporter/json.ts | 17 | ✅ |
| 報告入口 | src/reporter/index.ts | 27 | ✅ |
| 主入口 | src/index.ts | 163 | ✅ 全導出 |
| **SDK src/ 合計** | | **~5,700** | |
| Dashboard UI | web-ui/dashboard.html | 720 | ✅ 6 Tab |
| 測試 | tests/ (8檔) | ~1,500 | ✅ 167/167 |
| **全專案合計** | | **~7,900** | |

### 2.3 Dashboard 功能驗證 (6 Tab)

| Tab | 功能 | 狀態 |
|:----|:-----|:----:|
| 🔧 Pipeline | 十步走線表 + 走線連通圖 (dot/wire SVG) | ✅ 已驗證 |
| 🚪 Gates | G1-G4 卡片 + AND Gate 判定 | ✅ 已驗證 |
| 📋 DoD | 8 點 DoD + MISSION COMPLETE 判定 | ✅ 已驗證 |
| 📡 Waveform | 三通道 SVG 示波器 + 12 Reading 明細 + Composite Score | ✅ 已驗證 |
| ⚡ Scan Results | 統計卡 + Rule 分佈表 + File 明細 | ✅ 已驗證 |
| 📝 Violations | Sev/Rule/File/Line/Snippet/Suggestion 完整表格 | ✅ 已驗證 |

連線方式: Dashboard → `http://localhost:3333` → REST API (6 端點)

### 2.4 外部注入能力 (CLI 限定，已驗證)

```
Auto-detect from package.json:
  build    → npm run typecheck (exit 0) ✅
  lint     → npm run typecheck (exit 0) ✅
  test     → npx vitest run (167 passed) ✅
  coverage → npm run test:coverage (42.43%) ✅

手動覆寫: --build-cmd / --test-cmd / --lint-cmd / --coverage-cmd
禁用: --no-auto
```

### 2.5 規則實作率

| 類別 | 實作/總數 | 覆蓋率 | 缺口 |
|:-----|:--------:|:------:|:-----|
| 紅線 R01-R18 | 15/18 | 83% | R04/R06/R11 需 LLM/CI/Git |
| 禁止 P01-P14 | 10/14 | 71% | P01/P02/P08/P11 需 LLM |
| **合計** | **25/32** | **78%** | |

---

## 3. 清理紀錄

- [x] ~~src/cli-commands.ts~~ — 已刪除 (被 commands.ts 取代)
- [x] ~~web-ui/index.html~~ — 已刪除 (假引擎，被 dashboard.html 取代)

---

## 4. CLI 使用方式

```bash
# 啟動 API + Dashboard
npx tsx src/cli.ts serve --port 3333
# 開啟 http://localhost:8765/dashboard.html (需另起 http-server)

# CLI 掃描
npx tsx src/cli.ts scan ./src
npx tsx src/cli.ts pipeline . --grade E          # 十步走線 (auto-detect)
npx tsx src/cli.ts pipeline . --grade E --no-auto # 純靜態掃描

# npm publish 後
npx @maidos/codeqc scan ./src
npx @maidos/codeqc pipeline . --grade E
npx @maidos/codeqc serve --port 3333
```

### SaaS API (6端點)
```
POST /api/v1/scan          → 快速掃描
POST /api/v1/pipeline      → 十步走線 (含 waveform)
POST /api/v1/fraud         → 反詐欺掃描
GET  /api/v1/rules         → 規則查詢
GET  /api/v1/health        → 健康檢查
GET  /api/v1/version       → 版本資訊
```

---

## 5. 引擎能力矩陣

```
功能                          狀態     單品CLI  SaaS API  Dashboard
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CLI scan (v3.2)               ✅驗證   ✅       ✅        ✅
CLI pipeline (v3.3 十步)      ✅驗證   ✅       ✅        ✅
CLI serve (SaaS API)          ✅驗證   —        ✅        —
十步走線引擎                   ✅       ✅       ✅        ✅ Tab
外部結果注入 (auto-detect)    ✅驗證   ✅       —         —
G1-G4 AND Gate 門禁           ✅       ✅       ✅        ✅ Tab
三通道示波器 (SVG)            ✅       ✅       ✅        ✅ Tab
DoD 8點判定                   ✅       ✅       ✅        ✅ Tab
LV1-5 防偽 (E商用)            ✅       ✅       ✅        —
LV6-9 防偽 (F深科技)          ⚠️接口   ✅       ✅        —
Proof Pack 生成               ✅       ✅       —         —
REST API (6 endpoints)        ✅       —        ✅        連接
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 6. 後續路線

### Sprint B — SaaS 擴展 (後續)
| # | 任務 | 預估 |
|:-:|:-----|:----:|
| B1 | Dashboard 內建到 serve (靜態檔案服務) | 1h |
| B2 | SQLite 持久化 (掃描歷史) | 2h |
| B3 | GitHub Webhook | 3h |
| B4 | 多租戶+API Key 管理 | 2h |
| B5 | 計費/配額 | 2h |

### Sprint C — 語言深度 (長線)
| # | 任務 | 優先 |
|:-:|:-----|:----:|
| C1 | tree-sitter AST | P1 |
| C2 | 缺失紅線 R04/R06/R11 | P2 |
| C3 | 缺失禁止 P01/P02/P08/P11 | P2 |
| C4 | 語言插件架構 | P3 |

---

## 7. 架構圖

```
src/                          行數
├── types.ts                  670   全量類型+常量
├── index.ts                  163   主導出
├── analyzer.ts               201   語言偵測+批量分析
├── cli.ts                    493   CLI入口 (scan/路由)
├── commands.ts               245   pipeline+serve+外部注入
│
├── rules/                          B規則層
│   ├── b-axioms.ts           148   A1-A8
│   ├── b-redlines.ts         639   R01-R18 (15/18)
│   ├── b-prohibitions.ts     394   P01-P14 (10/14)
│   ├── c-gates.ts            283   v2.4 舊版門禁
│   └── index.ts               82
│
├── engine/                         v3.3 硬體化引擎
│   ├── pipeline.ts           543   十步走線+外部注入
│   ├── gates-v33.ts          167   G1-G4 AND Gate
│   ├── evidence.ts           252   Proof Pack + DoD
│   ├── protection.ts         180   LV1-9 防偽
│   ├── waveform.ts           206   三通道示波器
│   ├── dod.ts                142   DoD 8點評估器
│   └── index.ts               27
│
├── server/app.ts             213   REST API (6端點)
└── reporter/                       Console/JSON/HTML
    ├── console.ts            168
    ├── html.ts               433
    ├── json.ts                17
    └── index.ts               27

web-ui/
└── dashboard.html            720   PCB風格 6-Tab Dashboard

tests/                        ~1,500  167/167 pass
```

---

*Code-QC v3.3 · MAIDOS · 全驗證通過 · 2026-02-02*
