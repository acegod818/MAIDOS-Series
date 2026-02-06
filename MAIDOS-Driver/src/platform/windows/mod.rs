//! Windows 平台特定功能
//!
//! 實現 Windows 操作系統的特定功能

pub mod registry_manager;
pub mod service_controller;
pub mod system_info;
pub mod wmi_queries;

/// 初始化 Windows 平台特定功能
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Windows 平台特定功能初始化完成");
    Ok(())
}
