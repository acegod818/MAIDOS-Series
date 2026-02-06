//! 設備診斷模組
//!
//! 提供設備問題碼查詢、IRQ 資源偵測等功能

pub mod device_diagnostics;

/// 初始化診斷模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("設備診斷模組初始化完成");
    Ok(())
}
