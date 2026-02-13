# MAIDOS CodeQC - Phase 0 交接文檔

> **項目**：MAIDOS CodeQC  
> **Phase**：0（核心 + 標配）  
> **日期**：2026-01-26  
> **狀態**：🟢 Phase 0 完成

---

## 1. 摘要

Phase 0 目標：建立核心規則引擎 + 5 種標配語言支援

| 項目 | 目標 |
|:-----|:-----|
| 規則引擎 | Code-QC v3.5 B+C 全量（50 條） |
| 語言支援 | TypeScript, JavaScript, Python, Rust, Go |
| CLI | `npx maidos-codeqc` |
| CI/CD | GitHub Actions, GitLab CI |
| 報告 | Console, JSON, HTML |
| AI 檢查 | maidos-llm 接口預留 |

---

## 2. 進度追蹤

### 2.1 核心模組

| 模組 | 狀態 | 說明 |
|:-----|:----:|:-----|
| `src/types.ts` | ✅ | 類型定義 |
| `src/rules/b-axioms.ts` | ✅ | 8 公理 |
| `src/rules/b-redlines.ts` | ✅ | 12 紅線（5 自動檢測） |
| `src/rules/b-prohibitions.ts` | ✅ | 14 禁止（5 自動檢測） |
| `src/rules/c-gates.ts` | ✅ | 4 關卡 + 雙軸驗證 |
| `src/analyzer.ts` | ✅ | 靜態分析（Regex 版） |
| `src/analyzer/ai.ts` | ⚪ | LLM 接口（預留） |
| `src/reporter/console.ts` | ✅ | Console 報告 |
| `src/reporter/json.ts` | ✅ | JSON 報告 |
| `src/reporter/html.ts` | ✅ | HTML 報告 |
| `src/cli.ts` | ✅ | CLI 入口 |
| `src/index.ts` | ✅ | 主入口 |

### 2.2 語言解析器

| 語言 | Parser | 狀態 |
|:-----|:-------|:----:|
| TypeScript | tree-sitter-typescript | ⚪ |
| JavaScript | tree-sitter-javascript | ⚪ |
| Python | tree-sitter-python | ⚪ |
| Rust | tree-sitter-rust | ⚪ |
| Go | tree-sitter-go | ⚪ |

### 2.3 CI/CD 整合

| 平台 | 狀態 |
|:-----|:----:|
| GitHub Actions | ✅ |
| GitLab CI | ✅ |

### 2.4 測試

| 測試 | 狀態 |
|:-----|:----:|
| 規則單元測試 | ✅ |
| 分析器測試 | ✅ |
| 報告器測試 | ✅ |
| 測試 fixtures | ✅ |

---

## 3. 規則映射表

### 3.1 紅線（R01-R12）自動檢測

| 紅線 | 檢測方法 | 實現難度 |
|:-----|:---------|:--------:|
| R01 硬編碼憑證 | Regex + AST | ★☆☆ |
| R02 跳過安全檢查 | AST 啟發式 | ★★☆ |
| R03 刪除審計日誌 | AST（刪除調用） | ★★☆ |
| R04 未授權數據訪問 | LLM 輔助 | ★★★ |
| R05 忽略錯誤處理 | AST（空 catch） | ★☆☆ |
| R06 直接操作生產 | 配置檢測 | ★★☆ |
| R07 關閉安全功能 | AST + 配置 | ★★☆ |
| R08 使用已知漏洞 | 依賴掃描 | ★★☆ |
| R09 無限制資源 | AST 啟發式 | ★★☆ |
| R10 明文傳輸敏感 | AST + Regex | ★★☆ |
| R11 跳過代碼審查 | Git 整合 | ★★★ |
| R12 偽造測試結果 | AST（硬編碼斷言） | ★★☆ |

### 3.2 禁止（P01-P14）自動檢測

| 禁止 | 檢測方法 | 閾值 |
|:-----|:---------|:-----|
| P01 過度工程 | LLM 輔助 | - |
| P02 過早優化 | LLM 輔助 | - |
| P03 複製粘貼 | 重複檢測 | DRY |
| P04 魔法數字 | AST | 0 |
| P05 超長函數 | AST（行數） | ≤100 |
| P06 深層嵌套 | AST（層數） | ≤5 |
| P07 全局狀態 | AST | - |
| P08 緊耦合 | 依賴分析 | - |
| P09 無意義命名 | Regex + AST | 0 |
| P10 過長參數 | AST（參數數） | ≤6 |
| P11 混合抽象 | LLM 輔助 | - |
| P12 註釋代碼 | Regex | 刪除 |
| P13 TODO 堆積 | Regex | ≤5 |
| P14 依賴膨脹 | 依賴分析 | - |

---

## 4. 技術選型

| 層次 | 選定 | 原因 |
|:-----|:-----|:-----|
| 語言 | TypeScript | MAIDOS 標準 |
| 解析 | Tree-sitter (web-tree-sitter) | 跨語言 AST |
| 測試 | Vitest | 快速、ESM 友好 |
| 打包 | tsup | 簡潔、ESM/CJS |
| CLI | Commander.js | 業界標準 |

---

## 5. 目錄結構

```
maidos-codeqc/
├── package.json
├── tsconfig.json
├── tsup.config.ts
├── vitest.config.ts
├── README.md
├── ROADMAP.md
├── HANDOVER.md
│
├── src/
│   ├── index.ts              # 主入口
│   ├── cli.ts                # CLI 入口
│   ├── types.ts              # 類型定義
│   │
│   ├── rules/                # 規則定義
│   │   ├── index.ts
│   │   ├── b-axioms.ts       # 8 公理（提示用）
│   │   ├── b-redlines.ts     # 12 紅線（硬檢查）
│   │   ├── b-prohibitions.ts # 14 禁止（AST 檢查）
│   │   └── c-gates.ts        # 4 關卡（checklist）
│   │
│   ├── analyzer/             # 分析引擎
│   │   ├── index.ts
│   │   ├── static.ts         # AST 靜態分析
│   │   ├── ai.ts             # LLM 輔助分析
│   │   └── duplicate.ts      # 重複代碼檢測
│   │
│   ├── languages/            # 語言支援
│   │   ├── index.ts
│   │   ├── typescript.ts
│   │   ├── javascript.ts
│   │   ├── python.ts
│   │   ├── rust.ts
│   │   └── go.ts
│   │
│   ├── reporter/             # 報告生成
│   │   ├── index.ts
│   │   ├── console.ts
│   │   ├── json.ts
│   │   └── html.ts
│   │
│   └── ci/                   # CI/CD 整合
│       ├── github-action.ts
│       └── gitlab-ci.ts
│
├── templates/                # 報告模板
│   └── report.html
│
├── tests/
│   ├── rules/
│   ├── analyzer/
│   └── fixtures/             # 測試用例
│       ├── typescript/
│       ├── python/
│       └── rust/
│
└── docs/
    ├── API.md
    └── RULES.md
```

---

## 6. 下一步

1. **P0**: 建立 package.json + 基礎配置
2. **P0**: 實現類型定義 (types.ts)
3. **P0**: 實現 12 紅線規則
4. **P1**: 實現 14 禁止規則
5. **P1**: 整合 Tree-sitter 解析器
6. **P2**: 實現報告輸出
7. **P2**: CLI 整合
8. **P3**: CI/CD 模板

---

## 7. 阻塞項

| 阻塞 | 需要 | 狀態 |
|:-----|:-----|:----:|
| 無 | - | - |

---

## 8. 注意事項

⚠️ **Tree-sitter WASM**：web-tree-sitter 需要 WASM 初始化，注意異步加載

⚠️ **ESM/CJS 兼容**：確保打包同時支援 ESM 和 CJS

⚠️ **maidos-llm 接口**：AI 檢查模組預留接口，暫不實作具體邏輯

---

*Phase 0 開發中...*
