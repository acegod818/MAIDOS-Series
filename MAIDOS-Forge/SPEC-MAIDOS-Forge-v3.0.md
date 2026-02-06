# MAIDOS Forge 規格 v3.0

> **版本**：v3.0
> **日期**：2026-02-06
> **狀態**：Governor 重新拍板
> **前版**：v2.2（已作廢，範圍灌水）
> **場景**：`scene_sys_perf`
> **Code-QC**：v3.2

---

## 1. 北極星

```
定位：跨語言統一編譯框架

核心價值：
├── 一個 CLI 指令編譯任何支援語言
├── 統一錯誤報告格式（不論底層編譯器）
├── 跨平台交叉編譯（同一份 source → 多目標）
└── 插件架構（社群可擴展語言支援）

不是什麼：
├── 不是 IDE（沒有編輯器、不做 LSP）
├── 不是套件管理器（不取代 npm/cargo/pip）
└── 不是建置系統（不取代 CMake/Make/Gradle）
```

---

## 2. 範圍定義

### 2.1 語言支援（v3.0 誠實範圍）

| 級別 | 定義 | 語言 | 數量 | 驗收標準 |
|:-----|:-----|:-----|:----:|:---------|
| **Tier A（完整支援）** | 編譯+檢查+交叉編譯+介面擷取全通 | C, C++, C#, Rust, Go | 5 | 全 AC 通過 |
| **Tier B（基本支援）** | 編譯+檢查通過，交叉編譯可選 | Python, JavaScript, TypeScript, Java, Kotlin, Swift, Ruby, Dart, Haskell, Elixir | 10 | FR-001~003 通過 |
| **Tier C（社群擴展）** | Plugin 骨架+文件，標明「Community Welcome」 | 其餘（Lua, Perl, R, PHP 等） | 不限 | 有 plugin.json + README |
| ~~Tier 2/3 (v2.2)~~ | ~~已作廢~~ | ~~80+ 語言空殼~~ | ~~0~~ | ~~刪除~~ |

**v3.0 交付承諾：5 完整 + 10 基本 = 15 語言可用**

### 2.2 平台支援

| 平台 | 架構 | Tier A | Tier B |
|:-----|:-----|:------:|:------:|
| Windows | x64 | ✅ | ✅ |
| Linux | x64 | ✅ | ✅ |
| macOS | x64, ARM64 | ✅ | - |
| WebAssembly | WASM | - | ✅ (JS/TS) |

**v3.0 交付承諾：3 平台穩定，WASM 實驗性**

---

## 3. 技術選型

| 層次 | 技術 | 版本 | 用途 |
|:-----|:-----|:-----|:-----|
| 高性能核心 | Rust | 1.70+ | Parser, Checker, Builder — 熱路徑 <1ms |
| 協調層 | C# .NET 8.0 | 8.0 | Plugin 管理、BuildOrchestrator、CrossCompiler |
| 解析 | Tree-sitter | latest | C/C++/Rust 增量解析（不覆蓋全語言） |
| FFI | P/Invoke | - | C# ↔ Rust cdylib |
| 底層 | 各語言原生編譯器 | - | GCC/Clang/rustc/go/dotnet/javac/node 等 |

---

## 4. 系統架構

```
┌──────────────────────────────────────────────────────────┐
│                      CLI 層 (C#)                          │
│  forge build <lang> [--target <platform>] [--release]    │
│  forge check <lang>                                       │
│  forge clean | init | watch | graph | toolchain | plugin │
├──────────────────────────────────────────────────────────┤
│                   C# 協調層 (.NET 8.0)                    │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐            │
│  │PluginMgr   │ │BuildOrch.  │ │CrossCompile│            │
│  │(載入/註冊) │ │(排程/快取) │ │(目標平台)  │            │
│  └────────────┘ └────────────┘ └────────────┘            │
├──────────────────────────────────────────────────────────┤
│                   Rust 核心 (cdylib FFI)                  │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐            │
│  │Parser  │ │Checker │ │Builder │ │  FFI   │            │
│  │(tree-  │ │(lint/  │ │(invoke │ │(C#↔   │            │
│  │sitter) │ │analyze)│ │compile)│ │Rust)   │            │
│  └────────┘ └────────┘ └────────┘ └────────┘            │
├──────────────────────────────────────────────────────────┤
│              語言插件層 (ILanguagePlugin)                  │
│  ┌───┐ ┌───┐ ┌────┐ ┌────┐ ┌───┐ ┌───┐ ┌──┐           │
│  │ C │ │C++│ │Rust│ │ Go │ │C# │ │...│ │15│           │
│  └───┘ └───┘ └────┘ └────┘ └───┘ └───┘ └──┘           │
├──────────────────────────────────────────────────────────┤
│              原生編譯器 (系統 PATH)                        │
│  GCC/Clang | rustc/cargo | go | dotnet | javac | node   │
└──────────────────────────────────────────────────────────┘
```

---

## 5. 功能需求 (FR)

### FR-001 單語言編譯

| 項目 | 內容 |
|:-----|:-----|
| 描述 | `forge build <lang> <source>` 呼叫對應編譯器產出二進位 |
| 前置 | 系統已安裝該語言編譯器 |
| 輸出 | 編譯產物 + 標準化結果 JSON |
| 範圍 | Tier A + Tier B 全部 15 語言 |

**AC-001**: Given 系統有 GCC，When `forge build c hello.c`，Then `./hello` 可執行且輸出正確
**AC-002**: Given 系統有 rustc，When `forge build rust main.rs`，Then 產出二進位檔
**AC-003**: Given 系統無 Swift 編譯器，When `forge build swift`，Then 報錯「toolchain not found: swift」並列出安裝建議

### FR-002 工具鏈偵測

| 項目 | 內容 |
|:-----|:-----|
| 描述 | `forge check <lang>` 偵測系統已安裝的編譯器/runtime |
| 輸出 | JSON: `{found: bool, version: string, path: string}` |

**AC-004**: Given 系統有 Go 1.21，When `forge check go`，Then 回傳 `{found:true, version:"1.21.x", path:"/usr/local/go/bin/go"}`
**AC-005**: Given 系統有 Clang 和 GCC，When `forge check c`，Then 列出兩個可用 toolchain 供選擇

### FR-003 錯誤標準化

| 項目 | 內容 |
|:-----|:-----|
| 描述 | 不論底層編譯器，錯誤格式統一為 `ForgeError {file, line, col, severity, message, lang}` |
| 範圍 | 所有 Tier A/B 語言 |

**AC-006**: Given C 程式有語法錯誤，When `forge build c broken.c`，Then 輸出含 `file`, `line`, `severity:error`, `message` 欄位
**AC-007**: Given Rust 程式有型別錯誤，When `forge build rust bad.rs`，Then 同樣格式輸出

### FR-004 交叉編譯

| 項目 | 內容 |
|:-----|:-----|
| 描述 | `forge build <lang> --target <platform>` 產出指定平台產物 |
| 範圍 | 僅 Tier A（C, C++, Rust, Go, C#）|

**AC-008**: Given Windows 主機有交叉編譯工具鏈，When `forge build c hello.c --target linux-x64`，Then 產出 ELF 二進位
**AC-009**: Given Go 源碼，When `forge build go main.go --target linux-arm64`，Then 產出 ARM64 Linux 二進位

### FR-005 介面擷取

| 項目 | 內容 |
|:-----|:-----|
| 描述 | 從編譯產物擷取 public API/符號清單 |
| 範圍 | 僅 Tier A |

**AC-010**: Given C 標頭檔有 3 個 public 函數，When `forge interface hello.h`，Then 回傳 JSON 含 3 個函數簽名

### FR-006 插件系統

| 項目 | 內容 |
|:-----|:-----|
| 描述 | 第三方可透過 ILanguagePlugin 介面擴展語言支援 |
| 介面 | `GetCapabilities()`, `ValidateToolchainAsync()`, `CompileAsync()`, `ExtractInterfaceAsync()` |

**AC-011**: Given 自製 Plugin DLL 實作 ILanguagePlugin，When 放入 plugins/ 目錄，Then `forge build <custom-lang>` 可呼叫

### FR-007 CLI 指令集

| 指令 | 功能 | 優先級 |
|:-----|:-----|:------:|
| `forge build` | 編譯 | P0 |
| `forge check` | 工具鏈偵測 | P0 |
| `forge clean` | 清理產物 | P0 |
| `forge init` | 初始化專案 | P1 |
| `forge watch` | 檔案監聽自動編譯 | P1 |
| `forge graph` | 依賴圖 | P2 |
| `forge toolchain` | 工具鏈管理 | P1 |
| `forge plugin` | 插件管理 | P1 |

---

## 6. 非功能需求 (NFR)

| NFR | 目標 | 測量方式 |
|:----|:-----|:---------|
| **NFR-001 延遲** | Rust 核心路徑 <10ms (parser+checker) | benchmark test |
| **NFR-002 編譯延遲** | 單檔 <底層編譯器耗時 +500ms overhead | 計時測試 |
| **NFR-003 記憶體** | Forge 自身常駐 <100MB | memory profiling |
| **NFR-004 測試覆蓋率** | Rust ≥70%, C# ≥60% | coverage report |
| **NFR-005 編譯** | 0 error, 0 warning (`cargo build` + `dotnet build`) | CI pipeline |
| **NFR-006 安全** | 不執行不信任程式碼（只調用系統編譯器） | code audit |

---

## 7. 風險與依賴

| 風險 | 影響 | 緩解 |
|:-----|:-----|:-----|
| 系統無對應編譯器 | 該語言不可用 | FR-002 偵測 + 友善錯誤提示 |
| Tree-sitter 不支援某語言 | 無增量解析 | 退回正則解析或跳過 |
| 交叉編譯工具鏈安裝複雜 | 使用者門檻高 | 提供 `forge toolchain install` 輔助 |

| 依賴 | 版本 |
|:-----|:-----|
| .NET SDK | 8.0+ |
| Rust toolchain | 1.70+ |
| 各語言編譯器 | 使用者自行安裝 |

---

## 8. 驗收矩陣

| Req | AC | 證據類型 | 通過條件 |
|:----|:---|:---------|:---------|
| FR-001 | AC-001~003 | integration test | 15 語言各至少 1 個 hello-world 編譯通過 |
| FR-002 | AC-004~005 | unit test | 15 語言 toolchain 偵測正確 |
| FR-003 | AC-006~007 | unit test | 錯誤 JSON 格式一致 |
| FR-004 | AC-008~009 | integration test | Tier A 至少 2 個交叉目標通過 |
| FR-005 | AC-010 | unit test | C/C++/Rust 介面擷取正確 |
| FR-006 | AC-011 | integration test | 自製 Plugin 載入+編譯成功 |
| FR-007 | - | e2e test | 8 個指令 exit code 正確 |
| NFR-001~006 | - | benchmark + CI | 全部達標 |

---

## 9. 交付物結構

```
MAIDOS-Forge/
├── maidos-forge-core/        # Rust 核心 (parser/checker/builder/ffi)
├── maidos-forge-cli/         # Rust CLI 入口
├── src/
│   ├── Forge.Core.New/       # C# 協調層
│   ├── Forge.Cli/            # C# CLI 指令路由
│   ├── Forge.Plugins/        # 語言插件 (15 個 Tier A/B)
│   ├── Forge.Tests/          # C# 測試
│   └── Forge.Studio/         # (未來) GUI
├── evidence/                 # CodeQC 證據
├── specs/                    # 本文件
└── SPEC-MAIDOS-Forge-v3.0.md
```

---

## 10. 里程碑

| 里程碑 | 內容 | 預估 |
|:-------|:-----|:-----|
| M1 | 清理空殼 Plugin + 修復測試 | 3 天 |
| M2 | Tier B 10 語言 Plugin 實作 | 5 天 |
| M3 | NFR 達標 + 覆蓋率 ≥60% | 2 天 |
| M4 | G2 規格核對通過 | 1 天 |
| M5 | G3 壓力測試 | 2 天 |

---

*Governor 重新拍板。v2.2 作廢。v3.0 誠實範圍、真實驗收。*

**簽署**：MAIDOS 技術委員會
**日期**：2026-02-06
