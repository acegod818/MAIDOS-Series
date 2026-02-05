# MAIDOS CodeQC 規格

> **版本**：v1.0  
> **日期**：2026-01-24  
> **狀態**：Governor 拍板鎖定  
> **場景**：`scene_backend_startup`

---

## 1. 北極星

```
定位：代碼品質檢查工具

核心能力：
├── 靜態分析
├── 規則引擎（Code-QC v2.6）
├── 多語言支援
└── CI/CD 整合
```

---

## 2. 技術選型

| 層次 | 選定 |
|:-----|:-----|
| 語言 | TypeScript |
| 解析 | Tree-sitter |
| LLM | maidos-llm（智能檢查） |

---

## 3. 規則體系

| 規則集 | 內容 | 數量 |
|:-------|:-----|:-----|
| B（工作紀律） | 公理 + 紅線 + 禁令 | 41 條 |
| C（驗收標準） | 關卡 + 雙軸驗證 | ~50 條 |
| D = B + C | 完整規則 | ~90 條 |

---

## 4. 核心功能

| 功能 | 說明 |
|:-----|:-----|
| **規則檢查** | 靜態分析（AST） |
| **AI 檢查** | LLM 輔助語義分析 |
| **報告** | HTML / JSON / Console |
| **CI 整合** | GitHub Actions / GitLab CI |

---

## 5. 交付物結構

```
maidos-codeqc/
├── package.json
├── src/
│   ├── index.ts
│   ├── analyzer/
│   │   ├── static.ts
│   │   └── ai.ts
│   ├── rules/
│   │   ├── b-discipline.ts
│   │   └── c-acceptance.ts
│   └── reporter/
│       ├── html.ts
│       └── json.ts
└── tests/
```

---

*Governor 拍板，規格鎖定。*
