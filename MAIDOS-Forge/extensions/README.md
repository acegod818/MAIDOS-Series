# MAIDOS Forge 擴充包系統

## 概述

MAIDOS Forge 支援通過擴充包來增加對更多語言的支持。擴充包分為兩個層級：
- **Tier 2**: 常見語言擴充包
- **Tier 3**: 特殊語言擴充包

## 擴充包結構

每個擴充包應遵循以下目錄結構：

```
language-name-plugin/
├── src/
│   ├── lib.rs          # 擴充包主要實現
│   └── Cargo.toml      # 依賴配置
├── manifest.json       # 擴充包元數據
├── README.md           # 使用說明
└── tests/              # 測試文件
    └── integration_tests.rs
```

## 擴充包接口

所有擴充包必須實現 `LanguageAdapter` trait：

```rust
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    fn language_id(&self) -> &str;
    fn language_name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    
    async fn compile(
        &self,
        source_files: &[std::path::PathBuf],
        output_dir: &Path,
        options: &CompileOptions,
    ) -> Result<CompileResult>;

    async fn check_toolchain(&self) -> Result<bool>;
    async fn toolchain_info(&self) -> Result<ToolchainInfo>;
    async fn parse(&self, source_file: &Path) -> Result<ParseResult>;
    async fn check(&self, source_file: &Path) -> Result<CheckResult>;
    async fn extract_interface(&self, artifact_path: &Path) -> Result<InterfaceDescription>;
    async fn generate_glue(&self, interface: &InterfaceDescription, target_language: &str) -> Result<GlueCodeResult>;
}
```

## 擴充包元數據 (manifest.json)

```json
{
  "id": "language-name",
  "name": "Language Name",
  "version": "1.0.0",
  "description": "Description of the language support",
  "author": "Author Name",
  "license": "MIT",
  "language": {
    "id": "language-id",
    "name": "Language Name",
    "extensions": [".ext1", ".ext2"],
    "category": "System"
  },
  "dependencies": {
    "required_tools": ["tool1", "tool2"],
    "minimum_version": "1.0.0"
  }
}
```

## 開發指南

1. 複製模板擴充包作為起點
2. 實現 `LanguageAdapter` trait
3. 添加語言特定的編譯和解析邏輯
4. 編寫測試確保功能正確
5. 打包並提交到擴充包倉庫

## 安裝擴充包

用戶可以通過以下命令安裝擴充包：

```bash
forge plugin install language-name
```

或者手動將擴充包放置在 `extensions/tier2/` 或 `extensions/tier3/` 目錄下。