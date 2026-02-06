# 示例項目

這個目錄包含各種示例項目，展示如何使用MAIDOS-Forge的不同功能。

## 示例列表

### 基礎示例
- `hello-world` - 最簡單的MAIDOS-Forge項目
- `multi-module` - 多模塊項目示例
- `cross-language` - 跨語言編譯示例

### 語言示例
- `rust-c-interoperability` - Rust與C語言互操作示例
- `csharp-rust-ffi` - C#與Rust的FFI示例
- `mixed-language-app` - 混合語言應用示例

### 高級示例
- `web-api` - Web API服務示例
- `desktop-app` - 桌面應用程序示例
- `cross-platform` - 跨平台編譯示例
- `wasm-target` - WebAssembly目標示例

## 使用方法

```bash
# 克隆特定示例
git clone https://github.com/maidos/forge-samples.git
cd forge-samples/hello-world

# 構建示例
forge build

# 運行示例
forge run

# 交叉編譯示例
forge cross --target linux-x64 --target windows-x64
```

## 貢獻

如果您想貢獻新的示例，請：

1. Fork示例倉庫
2. 創建新的示例目錄
3. 添加完整的README說明
4. 提交拉取請求