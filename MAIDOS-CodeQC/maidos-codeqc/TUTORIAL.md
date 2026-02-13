# MAIDOS CodeQC v0.3.5 — 白話教程

> 你的程式碼品管引擎，42 條規則一鍵掃描，抓出偷工減料。

---

## 目錄

1. [安裝](#1-安裝)
2. [30 秒快速開始](#2-30-秒快速開始)
3. [三種用法](#3-三種用法)
4. [規則說明](#4-規則說明)
5. [軟配置：只掃你要的](#5-軟配置只掃你要的)
6. [設定檔](#6-設定檔)
7. [API 模式 + Dashboard](#7-api-模式--dashboard)
8. [當作 SDK 用](#8-當作-sdk-用)
9. [CI/CD 接入](#9-cicd-接入)
10. [常見問題](#10-常見問題)

---

## 1. 安裝

```bash
# 方式一：從 zip 解壓安裝
unzip maidos-codeqc-v0.3.5.zip -d maidos-codeqc
cd maidos-codeqc
npm install

# 方式二：npm 全域安裝（發布後）
npm install -g @maidos/codeqc
```

裝完就能用，沒有其他依賴。

---

## 2. 30 秒快速開始

```bash
# 掃描當前目錄
npx tsx src/cli.ts scan .

# 掃描指定資料夾
npx tsx src/cli.ts scan ./my-project/src
```

跑完會列出所有違規，紅色是紅線（必修），黃色是禁止（應修）。

---

## 3. 三種用法

### A. scan — 快速掃描

日常開發用，掃完看報告。

```bash
# 基本掃描
npx tsx src/cli.ts scan ./src

# 指定等級（B=工作紀律, C=驗收標準, D=全部）
npx tsx src/cli.ts scan -l D ./src

# 輸出 JSON 報告
npx tsx src/cli.ts scan -r json -o report.json ./src

# 輸出 HTML 報告
npx tsx src/cli.ts scan -r html -o report.html ./src

# CI 模式（有紅線就 exit 1）
npx tsx src/cli.ts scan --ci ./src
```

### B. pipeline — 十步走線（正式出貨用）

完整品管流程：編譯 → 測試 → 靜態分析 → 四道門禁 → 出報告。

```bash
# 自動偵測 build/test/lint 指令
npx tsx src/cli.ts pipeline . --grade E

# 手動指定指令
npx tsx src/cli.ts pipeline . --grade E \
  --build-cmd "npm run build" \
  --test-cmd "npm test" \
  --lint-cmd "npm run lint"

# 純靜態掃描（不跑外部指令）
npx tsx src/cli.ts pipeline . --grade E --no-auto
```

**等級說明：**

| 等級 | 用途 | 說明 |
|------|------|------|
| **E** (Commercial) | 一般商用專案 | 防偽等級 LV1-5 |
| **F** (Deep-Tech) | 深科技/安全敏感 | 防偽等級 LV6-9 |

Pipeline 跑完會產出：
- 終端彩色報告
- `evidence/PROOF_PACK.md`（含 SHA-256 簽名）
- 示波器波形圖（ASCII）

### C. serve — API + Dashboard

團隊協作用，開一個 REST API 讓大家打。

```bash
# 啟動 API 伺服器
npx tsx src/cli.ts serve --port 3333

# 另開終端，起 Dashboard
cd web-ui && npx http-server -p 8765
```

瀏覽器開 `http://localhost:8765/dashboard.html` 就能看到六個分頁的品管儀表板。

---

## 4. 規則說明

共 42 條規則 = 28 紅線 + 14 禁止，全部自動偵測。

### 28 條紅線（error 級 — 必須修）

| 分類 | 規則 | 白話翻譯 |
|------|------|----------|
| 安全 | R01 硬編碼憑證 | 密碼寫死在程式裡 |
| 安全 | R02 注入攻擊 | SQL injection、XSS 等 |
| 安全 | R03 審計日誌 | 改了東西不留紀錄 |
| 安全 | R04 未授權數據訪問 | 偷看不該看的資料表 |
| 安全 | R05 忽略錯誤處理 | catch 空白、錯誤吃掉 |
| 生產 | R06 直接操作生產 | DROP TABLE、直連 prod DB |
| 安全 | R07 關閉安全功能 | 把驗證/SSL/CSRF 關掉 |
| 安全 | R08 已知漏洞 | 用有洞的套件版本 |
| 穩定 | R09 無資源限制 | 不限記憶體/連線/timeout |
| 安全 | R10 明文傳輸敏感 | 密碼用 HTTP 傳 |
| 流程 | R11 跳過代碼審查 | --force push、[skip ci] |
| 測試 | R12 偽造測試結果 | expect(true).toBe(true) |
| 詐欺 | R13 假實現 | 函數有名字但沒內容 |
| 詐欺 | R14 靜默失敗 | 錯誤吃掉假裝沒事 |
| 詐欺 | R15 TODO/FIXME佔位 | 寫了 TODO 就交貨 |
| 詐欺 | R16 空方法體 | 方法裡面是空的 |
| 詐欺 | R17 詐欺物件 | stub/mock 混進正式碼 |
| 詐欺 | R18 繞道捷徑 | HACK、WORKAROUND |
| 審計 | R19-R28 | 深層防禦 + 審計補漏 |

### 14 條禁止（warning 級 — 應該修）

| 規則 | 白話翻譯 | 偵測方式 |
|------|----------|----------|
| P01 過度工程 | Abstract 套三層、設計模式堆疊 | 類別名稱計數 |
| P02 過早優化 | 位運算代替加減、手寫 LinkedList | 模式比對 |
| P03 複製粘貼 | 大段重複代碼 | 行比對 |
| P04 魔法數字 | 數字寫死不給名字 | regex |
| P05 超長函數 | 函數超過 50 行 | 行數計算 |
| P06 深層嵌套 | if 套 if 套 if 超過 3 層 | 縮排計算 |
| P07 全局狀態 | 全域變數到處改 | regex |
| P08 緊耦合 | import 超過 15 個、../../.. | import 計數 |
| P09 無意義命名 | 變數叫 a, b, tmp, data | 黑名單比對 |
| P10 過長參數 | 函數參數超過 5 個 | 參數計數 |
| P11 混合抽象 | SQL 寫在 UI 裡面 | 層級模式交叉比對 |
| P12 註釋代碼 | 大段被註解掉的程式碼 | 連續註解偵測 |
| P13 TODO 堆積 | TODO/FIXME 超過 10 個 | 關鍵字計數 |
| P14 依賴膨脹 | package.json 依賴太多 | 數量計算 |

---

## 5. 軟配置：只掃你要的

不想全掃？可以選分類：

```bash
# 只掃安全性
npx tsx src/cli.ts scan --only-security ./src

# 只掃結構
npx tsx src/cli.ts scan --only-structure ./src

# 只掃品質
npx tsx src/cli.ts scan --only-quality ./src

# 複選：安全 + 結構
npx tsx src/cli.ts scan -C security,structure ./src

# 簡寫也行
npx tsx src/cli.ts scan -C s,t ./src
```

**分類對照：**

| 分類 | 簡寫 | 涵蓋規則 |
|------|------|----------|
| security (安全) | s, sec | R01-R10 |
| structure (結構) | t, struct | P03, P05-P07, P10 |
| quality (品質) | q, qual | P04, P09, P12-P14 |

---

## 6. 設定檔

不想每次打一堆參數？建一個 `.codeqcrc.yml`：

```yaml
# .codeqcrc.yml
level: D
reporter: console
ci: false
categories:
  - security
  - structure
  - quality
```

用法：

```bash
# 自動讀取同目錄的 .codeqcrc.yml
npx tsx src/cli.ts scan ./src

# 指定設定檔
npx tsx src/cli.ts scan -c my-config.yml ./src
```

---

## 7. API 模式 + Dashboard

### API 端點

啟動後預設跑在 `http://localhost:3333`。

| 方法 | 端點 | 用途 |
|------|------|------|
| POST | `/api/v1/scan` | 快速掃描（傳 source code） |
| POST | `/api/v1/pipeline` | 完整十步走線 |
| POST | `/api/v1/fraud` | 反詐欺掃描 (R13-R28) |
| GET | `/api/v1/rules` | 列出所有規則 |
| GET | `/api/v1/health` | 健康檢查 |
| GET | `/api/v1/version` | 版本資訊 |

### API 用法範例

```bash
# 掃描
curl -X POST http://localhost:3333/api/v1/scan \
  -H "Content-Type: application/json" \
  -d '{"source": "const pw = \"admin123\";", "file": "app.ts"}'

# 查規則
curl http://localhost:3333/api/v1/rules

# 健康檢查
curl http://localhost:3333/api/v1/health
```

### Dashboard 六個分頁

1. **Pipeline** — 十步走線結果
2. **Gates** — G1-G4 四道門禁
3. **DoD** — 8 點交付定義
4. **Waveform** — 示波器波形圖
5. **Scan** — 掃描結果明細
6. **Violations** — 違規清單排序

---

## 8. 當作 SDK 用

在你自己的程式裡直接 import：

```typescript
import { checkRedlines, checkProhibitions } from '@maidos/codeqc';
import { readFileSync } from 'fs';

// 掃描單一檔案
const code = readFileSync('src/app.ts', 'utf-8');
const reds = checkRedlines(code, 'src/app.ts');
const proh = checkProhibitions(code, 'src/app.ts');

console.log(`紅線違規: ${reds.length}`);
console.log(`禁止違規: ${proh.length}`);

// 列出明細
for (const v of [...reds, ...proh]) {
  console.log(`  [${v.severity}] ${v.ruleId}: ${v.message} (line ${v.line})`);
}
```

```typescript
// 完整分析
import { analyze, getReporter } from '@maidos/codeqc';

const result = analyze({
  files: [{ path: 'src/app.ts', content: code }],
  level: 'D',
  targetPath: './src',
});

// 輸出報告
const reporter = getReporter('console');
console.log(reporter.report(result));
```

---

## 9. CI/CD 接入

### GitHub Actions

```yaml
name: Code Quality
on: [push, pull_request]

jobs:
  codeqc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm install
      - run: npx tsx src/cli.ts scan --ci ./src
```

### GitLab CI

```yaml
codeqc:
  image: node:20
  script:
    - npm install
    - npx tsx src/cli.ts scan --ci ./src
  allow_failure: false
```

`--ci` 模式：有紅線就 exit code 1，CI 自動紅燈。

---

## 10. 常見問題

### Q: 支援什麼語言？

TypeScript、JavaScript、Python、Rust、Go — 五大語言直接掃。
看副檔名自動判斷，不用設定。

### Q: 掃描速度如何？

純靜態分析（regex + heuristic），不跑 AST 解析。
萬行級專案幾秒內完成。

### Q: 規則太嚴？某條想關掉？

用 `.codeqcrc.yml` 設定 categories 來關整類，
或用 `--only-security` 只跑安全性掃描。

### Q: Pipeline 跑失敗？

1. 確認 `build-cmd`、`test-cmd` 指令是正確的
2. 用 `--no-auto` 跳過自動偵測，手動指定指令
3. 看 `evidence/` 資料夾下的 log 找原因

### Q: 怎麼看 Proof Pack？

Pipeline 跑完後開 `evidence/PROOF_PACK.md`，
裡面有每一步的結果 + SHA-256 雜湊，防篡改。

### Q: 可以掃第三方的程式碼嗎？

當然可以。`scan` 指令指向任何資料夾就行：

```bash
npx tsx src/cli.ts scan /path/to/their/code
```

紅線一亮就知道有沒有偷工減料。

---

## 規則覆蓋

| 類型 | 數量 | 覆蓋率 |
|------|------|--------|
| 紅線 (R01-R28) | 28/28 | 100% ✅ |
| 禁止 (P01-P14) | 14/14 | 100% ✅ |
| **合計** | **42/42** | **100%** ✅ |

---

MIT © MAIDOS
