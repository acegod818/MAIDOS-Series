# Code-QC v3.0.1 使用說明書

---

## 一、快速開始

> **v3.0.1 整合**：把「需求/設計/營運」收編回 **A=規格（A0）**，用單一 `A.log` 做前置證明。請先放齊 A0 文件（可從 `templates/` 複製）。

### 選擇版本

```
├─ AI Executor     → D.md + .clinerules
├─ 商用團隊        → E.md
└─ 完整存檔        → F.md
```

### 專案配置

將以下文件放入專案根目錄：
- `codeqc_probe.py` - 規格/證明檢查器（輸出 `A.log`、檢查 Proof Pack）
- `.clinerules` - AI 工具配置
- `maidos_pipeline.sh` - Linux/WSL Pipeline（只呼叫統一入口）
- `maidos_pipeline.bat` - Windows Pipeline（只呼叫統一入口）
- `qc/` - **統一入口腳本**（build/unit/integration/e2e/proof；與語言/框架無關）

---

## 二、Pipeline 執行 (重點)

> 完工只看兩個 Gate：`A.log=PASS` + `Proof Pack=PASS`（由 pipeline 自動產生與驗證）。
### Hardness（嚴格度）等級：你要的「硬」就是這個開關

`codeqc_probe.py proof --strict-level N`

| N | 名稱 | 你拿到的保證 | 仍可能被偷懶/造假的地方 |
|---:|---|---|---|
| 1 | 結構 | 有 proof 目錄與基本檔案 | 可以用空檔/假 log 混過 |
| 2 | 交叉驗證 | J-001+ 必須出現在 e2e；sync_assert 必須可解析且對應旅程 | 還可能用「舊證據拼湊」 |
| 3 | 來源證明 | 追加 git/env/ci 等 provenance 欄位 | 仍可能重用舊 run 的證據 |
| 4 | 反拼湊（v3 預設） | **Nonce + Hashes + Merkle Root**：run 具唯一性，證據不可任意拼貼 | 仍可能由執行者惡意產生“假但自洽”的證據 |
| 5 | 不可否認（簽章） | **Ed25519 簽章**：可驗證 proof 來源（CI/Runner key） | 私鑰若外流，來源保證失效 |
| 6 | 外部 Challenge | verifier 發 nonce（challenge.json 簽章），runner 不能自選 nonce | 需要 verifier 的信任錨（trust.json） |
| 7 | Verifier 重跑 + Verdict | verifier 重跑產生 replay + 簽章 verdict，可抓「假但自洽」 | verifier 需要能重播/重跑或重驗證核心證據 |
| 8 | Attestation 介面 | 導入 attestor 簽章與 policy 白名單，能綁定硬體/CI 身份 | attestor 本身要可信、policy 要維護 |

建議：你要「模型不能偷懶」→ **預設用 4**；你要「誰也不能否認證據來源」→ **用 5（CI 簽章）**。



### Step 1: 判斷環境

```bash
echo $SHELL
```

| 輸出 | 你在哪 | 用什麼 |
|:-----|:-------|:-------|
| `/bin/bash` | Linux/WSL | `./maidos_pipeline.sh` |
| 空白或報錯 | Windows CMD | `.\maidos_pipeline.bat` |
| PS 版本 | PowerShell | `.\maidos_pipeline.bat` |

### Step 2: 執行腳本

**Linux/WSL:**
```bash
# 1. 確認位置
pwd
ls -la

# 2. 給予執行權限 (首次)
chmod +x maidos_pipeline.sh

# 3. 執行
./maidos_pipeline.sh
```

**Windows CMD:**
```cmd
REM 1. 確認位置
cd
dir

REM 2. 執行
.\maidos_pipeline.bat
```

### Step 3: 處理失敗

```
腳本失敗 → 禁止提交
        → 讀 evidence/ 目錄下的日誌
        → 修復代碼
        → 重跑腳本
        → 直到通過
```

---

## 三、CMD vs WSL 語法對照

| 操作 | CMD | Bash |
|:-----|:----|:-----|
| 執行腳本 | `.\script.bat` | `./script.sh` |
| 查看目錄 | `dir` | `ls -la` |
| 當前路徑 | `cd` (無參數) | `pwd` |
| 環境變數 | `%VAR%` | `$VAR` |
| 路徑分隔 | `\` | `/` |
| 刪除文件 | `del file` | `rm file` |
| 建立目錄 | `mkdir dir` | `mkdir -p dir` |
| 複製文件 | `copy src dst` | `cp src dst` |
| 移動文件 | `move src dst` | `mv src dst` |

---

## 四、路徑格式

```
Windows: C:\Users\name\project
WSL:     /mnt/c/Users/name/project
Linux:   /home/name/project
```

**轉換規則:**
- `C:\` → `/mnt/c/`
- `D:\` → `/mnt/d/`
- `\` → `/`

---

## 五、常見錯誤

| 錯誤 | 原因 | 解法 |
|:-----|:-----|:-----|
| `'.' 不是內部或外部命令` | CMD 用了 `./` | 改用 `.\` |
| `command not found` | Bash 用了 `.\` | 改用 `./` |
| `Permission denied` | 腳本沒執行權限 | `chmod +x script.sh` |
| `\r: command not found` | Windows 換行符 | `sed -i 's/\r$//' script.sh` |
| `No such file or directory` | 路徑錯誤 | `pwd` + `ls` 確認 |

---

## 六、Definition of Done

| # | 證明項 | 驗證 |
|:-:|:-------|:-----|
| 1 | 實現 | redline.log = 0 |
| 2 | 規格 | mapping.log + SPEC 100% |
| 3 | 編譯 | build.log 0e/0w |
| 4 | 交付 | package.log + run.log |
| 5 | 日誌 | pipeline PASS |

**(1+2+3+4+5) == MISSION COMPLETE**

---

## 七、新增紅線 (R13/R14/R15)

| # | 紅線 | 說明 |
|:-:|:-----|:-----|
| R13 | 假實現 | return true/null/空物件 |
| R14 | 靜默失敗 | catch 不處理 |
| R15 | TODO 殘留 | todo!/unimplemented! |

---

## 八、FAQ

**Q: Pipeline 失敗怎麼辦？**
- 讀 `evidence/` 目錄下的日誌
- 找到錯誤原因
- 修復代碼
- 重跑腳本

**Q: .clinerules 放哪？**
- 專案根目錄

**Q: WSL 如何存取 Windows 文件？**
- `C:\project` → `/mnt/c/project`

**Q: 腳本報 `\r` 錯誤？**
- Windows 換行符問題
- 執行 `sed -i 's/\r$//' script.sh`
