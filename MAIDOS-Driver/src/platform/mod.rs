//! 平台特定功能模組
//!
//! 實現不同操作系統平台的特定功能

pub mod windows;

/// 初始化平台特定功能模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("平台特定功能模組初始化完成");
    Ok(())
}
