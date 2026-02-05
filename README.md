# MAIDOS Series

> MAIDOS 全家桶 - 軟體工程工具集

## Projects

| Project | Description | Version |
|---------|-------------|---------|
| [maidos-shared](./maidos-shared/) | Rust 跨語言共享核心庫 (Config, Auth, Bus, LLM) | v0.2.0 |
| [MAIDOS-CodeQC](./MAIDOS-CodeQC/) | 程式碼品質控制工具 (Code-QC v3.3) | v0.3.3 |

## Quick Start

### MAIDOS-CodeQC

```bash
cd MAIDOS-CodeQC
./install.cmd      # Windows
./install.sh       # Linux/Mac

./codeqc.cmd ./src # Windows
./codeqc.sh ./src  # Linux/Mac
```

### maidos-shared (Rust)

```bash
cd maidos-shared
cargo build --release
```

## Requirements

- **CodeQC**: Node.js 18+
- **Shared Core**: Rust 1.75+

## License

MIT
