# 項目模板

這個目錄包含可以用於快速啟動新項目的模板。

## 模板列表

### 基礎模板
- `basic` - 基本的MAIDOS-Forge項目模板
- `webapp` - Web應用程序模板
- `library` - 庫項目模板
- `cli` - 命令行工具模板

### 語言特定模板
- `rust-basic` - Rust基礎模板
- `csharp-basic` - C#基礎模板
- `c-basic` - C語言基礎模板
- `go-basic` - Go語言基礎模板

### 高級模板
- `microservice` - 微服務架構模板
- `desktop-app` - 桌面應用程序模板
- `mobile-app` - 移動應用程序模板
- `game-engine` - 遊戲引擎模板

## 使用方法

```bash
# 使用基本模板創建新項目
forge init --template basic my-new-project

# 使用Web應用模板創建新項目
forge init --template webapp my-web-app

# 列出所有可用模板
forge init --list-templates
```

## 自定義模板

您可以創建自己的模板並放置在此目錄中。模板應該包含：

1. 一個`template.yaml`配置文件
2. 模板源代碼文件
3. 必要的配置文件（如`forge.yaml`）
4. README.md文件說明模板用途