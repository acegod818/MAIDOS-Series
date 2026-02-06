//! 未知設備識別模組
//!
//! 識別沒有正確驅動的設備

use super::hardware::DeviceInfo;

/// 識別未知設備
pub fn identify_unknown_devices(devices: &[DeviceInfo]) -> Vec<UnknownDevice> {
    let mut unknown_devices = Vec::new();

    for device in devices {
        // 如果設備狀態顯示為問題或未知，則標記為未知設備
        if device.status.contains("Problem") || device.status.contains("Unknown") {
            unknown_devices.push(UnknownDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                vendor: device.vendor.clone(),
                status: device.status.clone(),
            });
        }

        // 如果設備名稱包含未知或通用名稱，也標記為未知設備
        if device.name.to_lowercase().contains("unknown")
            || device.name.to_lowercase().contains("generic")
            || device.name.contains("Unknown")
        {
            unknown_devices.push(UnknownDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                vendor: device.vendor.clone(),
                status: device.status.clone(),
            });
        }
    }

    unknown_devices
}

/// 未知設備結構
#[derive(Debug, Clone)]
pub struct UnknownDevice {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub status: String,
}

/// 初始化未知設備識別模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("未知設備識別模組初始化完成");
    Ok(())
}
