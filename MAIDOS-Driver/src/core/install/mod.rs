//! 安裝引擎模組
//!
//! 負責驅動程序的安全安裝和管理

pub mod backup_manager;
pub mod installer;
pub mod rollback_handler;

/// 初始化安裝引擎模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("安裝引擎模組初始化完成");
    Ok(())
}
