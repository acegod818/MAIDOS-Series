# Contributing to MAIDOS Shared Core

感謝您對 MAIDOS Shared Core 的興趣！本文檔說明如何參與貢獻。

## Code-QC v2.1B3 標準

本專案遵循 **Code-QC v2.1B3** 品質標準。所有貢獻必須符合以下要求：

### 雙軸驗證

#### X-Axis: 合規 (Compliance)
- ❌ 零 `TODO` / `FIXME` 標記
- ❌ 零 `unimplemented!()` / `todo!()` 巨集
- ❌ 零編譯警告
- ✅ 通過 `cargo fmt`
- ✅ 通過 `cargo clippy`

#### Y-Axis: 成果 (Deliverables)
- ✅ 新程式碼有對應測試
- ✅ 測試有有效斷言（不是空測試）
- ✅ 功能端到端可執行
- ✅ 下游可直接依賴

## 開發流程

### 1. Fork 與 Clone

```bash
git clone https://github.com/YOUR_USERNAME/maidos-shared.git
cd maidos-shared
```

### 2. 建立分支

```bash
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/your-bug-fix
```

### 3. 開發

```bash
# 編譯
cargo build --workspace

# 測試
cargo test --workspace

# 格式化
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# 基準測試（可選）
cargo bench --workspace
```

### 4. 提交

遵循 [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(auth): add token refresh functionality
fix(config): handle empty environment variables
docs: update API documentation
perf(bus): optimize event serialization
test(llm): add provider integration tests
```

### 5. 提交 PR

- 填寫 PR 模板
- 確保 CI 通過
- 等待 Code Review

## 程式碼風格

### Rust

```rust
//! 模組級文檔
//!
//! <impl>
//! WHAT: 描述這個模組做什麼
//! WHY: 為什麼需要這個模組
//! HOW: 如何實現
//! TEST: 測試策略
//! </impl>

/// 函數級文檔
///
/// # Arguments
/// * `arg` - 參數說明
///
/// # Returns
/// 返回值說明
///
/// # Errors
/// 錯誤情況說明
///
/// # Example
/// ```
/// let result = my_function(arg);
/// ```
pub fn my_function(arg: Type) -> Result<Output> {
    // 實現
}
```

### FFI 函數

```rust
/// FFI 函數必須有完整的安全文檔
///
/// # Safety
/// - 指標必須有效或為 null
/// - 呼叫者負責釋放記憶體
#[no_mangle]
pub unsafe extern "C" fn maidos_xxx_function(...) -> ... {
    // 實現
}
```

### 測試

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meaningful_name() {
        // Arrange
        let input = ...;

        // Act
        let result = function(input);

        // Assert - 必須有有效斷言
        assert_eq!(result, expected);
    }
}
```

## 模組開發指南

### 新增 FFI 函數

1. 在 Rust 中實現 `ffi.rs`
2. 更新 C# 綁定 `bindings/csharp/MaidosShared/*.cs`
3. 確保 FFI 計數匹配
4. 添加測試

### 新增 Provider (maidos-llm)

1. 在 `providers/` 下創建新檔案
2. 實現 `LlmProvider` trait
3. 在 `providers/mod.rs` 中註冊
4. 添加測試和文檔

## 測試

### 運行所有測試

```bash
cargo test --workspace
```

### 運行特定模組測試

```bash
cargo test -p maidos-auth
```

### 運行基準測試

```bash
cargo bench --workspace
cargo bench --bench ffi_bench  # FFI 效能測試
```

## 文檔

### 生成文檔

```bash
cargo doc --workspace --no-deps --open
```

### 文檔標準

- 每個公開 API 必須有文檔
- 包含範例程式碼
- 說明錯誤情況

## 發布流程

1. 更新 `CHANGELOG.md`
2. 更新版本號（所有 `Cargo.toml`）
3. 創建 Git 標籤 `v0.x.x`
4. Push 標籤觸發 Release workflow

## 問題回報

- Bug: 使用 Bug Report 模板
- 功能請求: 使用 Feature Request 模板
- 問題討論: 使用 Discussions

## 行為準則

請保持專業、尊重的態度。我們歡迎所有背景的貢獻者。

---

*Code-QC v2.1B3: Zero Defects, Zero Fakes.*
