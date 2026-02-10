# MAIDOS Series

> MAIDOS 全家桶 — 開源軟體工程工具集

## Projects

| Project | Description | Tech | Version |
|---------|-------------|------|---------|
| [MAIDOS-Driver](./MAIDOS-Driver/) | Windows 硬體驅動管理工具 — 設備列舉、驅動更新/備份/回滾、數位簽章驗證 | Rust + C# WPF | v2.0 |
| [MAIDOS-IME](./MAIDOS-IME/) | 多語言輸入法引擎 — 注音/拼音/倉頡/速成/五筆/英文/日文，AI 候選字 | Rust + C++ | v2.0 |
| [MAIDOS-Forge](./MAIDOS-Forge/) | 跨語言編譯框架 — 97 種語言 Plugin、多平台建置支援 | Rust | v0.3.0 |
| [MAIDOS-CodeQC](./MAIDOS-CodeQC/) | 程式碼品質控制工具 — Code-QC v3.3 規範引擎，4 Gate 自動審查 | TypeScript / Node.js | v0.3.3 |
| [maidos-shared](./maidos-shared/) | 跨產品共享核心庫 — Config、Auth、EventBus、LLM 抽象層 | Rust | v0.2.0 |

### Coming Soon

| Project | Description | Tech | Status |
|---------|-------------|------|--------|
| MAIDOS-PDF | PDF 閱讀 / WYSIWYG 編輯 / 批次處理 / 表單設計 / AI OCR | C# WPF (.NET 9) | v2.0 開源準備中 |
| MAIDOS-Office | Word / Excel / PowerPoint / VBA 所見即所得編輯器 | C# WPF (.NET 9) | v2.0 開源準備中 |

## Quick Start

### MAIDOS-Driver (Windows)

```bash
cd MAIDOS-Driver
cargo build --release          # Build Rust DLL
dotnet publish installer/ -c Release  # Build WPF + MSI
```

### MAIDOS-Forge

```bash
cd MAIDOS-Forge/maidos-forge-cli
cargo build --release
./target/release/maidos-forge build ./my-project
```

### MAIDOS-IME

```bash
cd MAIDOS-IME
cargo build --release    # Rust core
cmake -B build && cmake --build build  # C++ TSF module
```

### MAIDOS-CodeQC

```bash
cd MAIDOS-CodeQC
npm install
npm run build
./codeqc.cmd ./src       # Windows
./codeqc.sh ./src        # Linux/Mac
```

### maidos-shared (Rust)

```bash
cd maidos-shared
cargo build --release
cargo test
```

## Architecture

```
MAIDOS-Series/
├── MAIDOS-Driver/     # Windows driver management (Rust cdylib + C# WPF)
├── MAIDOS-IME/        # Input method engine (Rust + C++ TSF)
├── MAIDOS-Forge/      # Multi-language compiler framework (Rust)
├── MAIDOS-CodeQC/     # Code quality control (TypeScript)
└── maidos-shared/     # Shared Rust core library
```

## Requirements

| Component | Requirement |
|-----------|-------------|
| Driver | Rust 1.75+, .NET 9, Windows 10+ |
| IME | Rust 1.75+, CMake 3.20+, MSVC 2022 |
| Forge | Rust 1.75+ |
| CodeQC | Node.js 18+ |
| Shared | Rust 1.75+ |

## License

MIT