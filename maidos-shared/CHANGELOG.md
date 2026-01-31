# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-01-09

### Added

#### Tier 2 Cloud Providers (T01-T05)
- **Mistral Provider** (`mistral.rs`)
  - 9 models: mistral-large, medium, small, codestral, ministral-8b/3b, nemo, pixtral-large/12b
  - Vision support via Pixtral models
  - Function calling support
  - 13 unit tests

- **Azure OpenAI Provider** (`azure_openai.rs`)
  - 5 models: gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-4, gpt-35-turbo
  - Deployment-based routing
  - API version parameter
  - 8 unit tests

- **Cohere Provider** (`cohere.rs`)
  - 6 models: command-r-plus, command-r, command-light, command
  - RAG support (documents parameter)
  - Native Cohere API format
  - 8 unit tests

- **Together AI Provider** (`together.rs`)
  - 8 models: Llama 3.3/3.2, Mixtral, Qwen, DeepSeek Coder
  - Vision support for Llama Vision models
  - OpenAI-compatible API
  - 6 unit tests

- **Replicate Provider** (`replicate.rs`)
  - 5 models: Llama 2, Mixtral, LLaVA, Mistral
  - Async prediction model (POST create → GET poll)
  - Vision support via LLaVA
  - 6 unit tests

#### Streaming Enhancement (T06-T07)
- **Unified Streaming Interface** (`streaming.rs`)
  - `StreamChunk` - unified streaming response type
  - `StreamUsage` - token usage in streaming
  - `SseParser` - Server-Sent Events parser
  - `StreamingResponse` trait
  - OpenAI and Anthropic format converters
  - 17 unit tests

- **Provider Streaming Implementation**
  - Mistral: Real SSE streaming
  - Azure OpenAI: Real SSE streaming
  - Together AI: Real SSE streaming
  - Cohere: Fallback (single chunk)
  - Replicate: Fallback (single chunk)

#### Tool Format (T11)
- **MaidosTool Unified Format** (`tool.rs`)
  - `MaidosTool` - provider-agnostic tool definition
  - `ToolParameter` - typed parameter with constraints
  - `ToProviderFormat` trait - convert to 5 provider formats
  - `ToolCall` / `ToolResult` - execution handling
  - Provider hint support (cache control, strict mode)
  - 20 unit tests

#### Error Handling Enhancement (T10)
- `VisionNotSupported` error with helpful suggestions
- `ToolsNotSupported` error with helpful suggestions
- `is_capability_error()` helper method
- 4 unit tests

#### Integration Tests (T12)
- `test_provider_type_all_v020` - 13 provider validation
- `test_tool_format_conversion` - cross-provider format
- `test_error_capability_classification` - error types
- `test_streaming_types` - streaming API
- `test_budget_controller_v020` - budget enforcement

### Changed
- Total providers: 8 → 13 (10 cloud + 3 local)
- Unit tests: 218 → 304
- Integration tests: 15 → 20

### Code Quality (Code-QC v2.2B/C)
- TODO/FIXME: 0
- Compilation warnings: 0
- Clippy warnings: 0
- Test pass rate: 100%
- New code unwrap: 0

---

## [0.1.0] - 2026-01-04

### Added

#### Phase 1: maidos-config
- TOML 配置檔案載入與解析
- 環境變數展開 (`${VAR}` 語法)
- Schema 驗證
- 熱重載支援 (file watcher)
- C FFI 導出 (9 函數)

#### Phase 2: maidos-auth
- HMAC-SHA256 簽名的 Capability Token
- 18 種預定義權限 (Capability)
- CapabilitySet 位元遮罩操作
- 時間戳記過期驗證
- C FFI 導出 (7 函數)

#### Phase 3: maidos-bus
- TCP 基礎的事件匯流排
- Publisher/Subscriber 模式
- MessagePack 序列化
- 主題匹配 (支援萬用字元)
- C FFI 導出 (8 函數)

#### Phase 4: maidos-llm
- 多 Provider 支援 (OpenAI, Anthropic, Ollama)
- 統一的 CompletionRequest API
- 串流回應支援
- 使用量追蹤
- C FFI 導出 (7 函數)

#### Phase 5: C# Binding
- 完整的 P/Invoke 綁定 (31 函數)
- MaidosConfig.cs
- MaidosAuth.cs
- MaidosBus.cs
- MaidosLlm.cs

#### Phase 6: 整合測試
- 跨模組互動測試 (15 測試)
- Config → Auth 整合
- Auth → Bus 整合
- 完整工作流程測試

#### Phase 7: 效能基準測試
- auth_bench.rs (Token 操作)
- bus_bench.rs (Event 操作)
- config_bench.rs (配置載入)
- llm_bench.rs (Request 建構)

#### Phase 8: 文檔與範例
- ARCHITECTURE.md
- QUICKSTART.md
- 5 個可執行範例

#### Phase 9: FFI 效能測試
- ffi_bench.rs (38 測試)
- Native vs FFI 比較
- 記憶體分配模式測試
- 吞吐量測試

### Performance Baseline

| 操作 | 延遲 | 吞吐量 |
|------|-----:|-------:|
| Token 生成 | 700 ns | 1.4M ops/sec |
| Token 驗證 | 680 ns | 1.5M ops/sec |
| Capability 檢查 | 3.3 ns | 300M ops/sec |
| Config 解析 (完整) | 19 µs | 53K ops/sec |
| Event 創建 (小) | 120 ns | 8.3M ops/sec |
| FFI 調用開銷 | < 15% | - |

### Code Quality (Code-QC v2.1B3)

- TODO/FIXME: 0
- 編譯警告: 0
- 測試通過率: 100%
- FFI ↔ P/Invoke: 31:31

---

[Unreleased]: https://github.com/maidos/maidos-shared/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/maidos/maidos-shared/releases/tag/v0.1.0
