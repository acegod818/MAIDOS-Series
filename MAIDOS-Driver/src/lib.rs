#![allow(non_snake_case)]
//! MAIDOS-Driver 核心函式庫

/// 核心模組
pub mod core;

/// 平台特定模組
pub mod platform;

/// FFI 接口模組
pub mod ffi;

/// AI 硬體辨識與驅動推薦模組
pub mod ai;

/// 驅動資料庫模組
pub mod database;

/// 一個簡單的測試函數
pub fn hello_world() -> &'static str {
    "Hello, MAIDOS-Driver!"
}
