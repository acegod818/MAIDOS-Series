//! 硬體偵測實現 — 使用 Windows SetupDI API 進行真實設備枚舉
//!
//! [MAIDOS-AUDIT] M3 重寫: 從假資料升級為 SetupDiGetClassDevsW + SetupDiEnumDeviceInfo
//! 替代原本回傳單一硬編碼 "CPU0" 的空殼

use log::info;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_Status, CM_DEVNODE_STATUS_FLAGS, CM_PROB, DN_HAS_PROBLEM,
};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
    SetupDiGetDeviceRegistryPropertyW, SetupDiOpenDevRegKey, DICS_FLAG_GLOBAL, DIGCF_ALLCLASSES,
    DIGCF_PRESENT, DIREG_DRV, HDEVINFO, SETUP_DI_REGISTRY_PROPERTY, SPDRP_CLASS, SPDRP_DEVICEDESC,
    SPDRP_HARDWAREID, SPDRP_MFG, SP_DEVINFO_DATA,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegQueryValueExW, KEY_READ, REG_SZ, REG_VALUE_TYPE,
};

/// 硬體設備信息
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// 設備 ID (Hardware ID)
    pub id: String,
    /// 設備名稱 (DeviceDesc)
    pub name: String,
    /// 設備廠商 (Manufacturer)
    pub vendor: String,
    /// 驅動版本 (Driver)
    pub version: String,
    /// 設備狀態
    pub status: String,
    /// 設備類別 (Class)
    pub class: String,
}

impl DeviceInfo {
    pub fn new(
        id: String,
        name: String,
        vendor: String,
        version: String,
        status: String,
        class: String,
    ) -> Self {
        Self {
            id,
            name,
            vendor,
            version,
            status,
            class,
        }
    }
}

/// 從 SetupDI 讀取設備的 WSTR 屬性
unsafe fn get_device_property_string(
    dev_info: HDEVINFO,
    dev_info_data: &mut SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> String {
    let mut buffer = [0u8; 4096];
    let mut required_size: u32 = 0;

    let ok = SetupDiGetDeviceRegistryPropertyW(
        dev_info,
        dev_info_data,
        property,
        None,
        Some(&mut buffer),
        Some(&mut required_size),
    );

    if ok.is_err() || required_size == 0 {
        return String::new();
    }

    // buffer 是 UTF-16LE 編碼，每 2 bytes 一個 wchar
    let wchar_count = (required_size as usize) / 2;
    if wchar_count == 0 {
        return String::new();
    }

    let wide: Vec<u16> = buffer[..wchar_count * 2]
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // 去掉尾部的 null terminator
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    OsString::from_wide(&wide[..end])
        .to_string_lossy()
        .into_owned()
}

/// Read the DriverVersion string from the device's driver registry key.
///
/// Opens the driver key via SetupDiOpenDevRegKey and reads "DriverVersion".
unsafe fn get_driver_version(dev_info: HDEVINFO, dev_info_data: &mut SP_DEVINFO_DATA) -> String {
    let hkey = SetupDiOpenDevRegKey(
        dev_info,
        dev_info_data,
        DICS_FLAG_GLOBAL.0,
        0,
        DIREG_DRV,
        KEY_READ.0,
    );

    let hkey = match hkey {
        Ok(k) => k,
        Err(_) => return String::new(),
    };

    let value_name: Vec<u16> = "DriverVersion\0".encode_utf16().collect();
    let mut buffer = [0u8; 512];
    let mut buffer_size = buffer.len() as u32;
    let mut value_type = REG_VALUE_TYPE(0);

    let result = RegQueryValueExW(
        hkey,
        windows::core::PCWSTR(value_name.as_ptr()),
        None,
        Some(&mut value_type),
        Some(buffer.as_mut_ptr()),
        Some(&mut buffer_size),
    );

    let _ = RegCloseKey(hkey);

    if result.0 != 0 || (value_type != REG_SZ && value_type.0 != 2) {
        return String::new();
    }

    let wchar_count = (buffer_size as usize) / 2;
    if wchar_count == 0 {
        return String::new();
    }
    let wide: Vec<u16> = buffer[..wchar_count * 2]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    OsString::from_wide(&wide[..end])
        .to_string_lossy()
        .into_owned()
}

/// 查詢設備節點狀態
unsafe fn get_device_status_string(dev_inst: u32) -> String {
    let mut status = CM_DEVNODE_STATUS_FLAGS(0);
    let mut problem = CM_PROB(0);

    let cr = CM_Get_DevNode_Status(&mut status as *mut _, &mut problem as *mut _, dev_inst, 0);

    if cr.0 != 0 {
        return "Unknown".to_string();
    }

    if (status.0 & DN_HAS_PROBLEM.0) != 0 {
        format!("Problem (Code {})", problem.0)
    } else {
        "OK".to_string()
    }
}

/// 使用 SetupDI API 枚舉所有已安裝的硬體設備
///
/// 調用 SetupDiGetClassDevsW(DIGCF_ALLCLASSES | DIGCF_PRESENT) 掃描所有在線設備
pub fn scan_all_devices() -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    info!("開始掃描硬體設備 (SetupDI API)...");

    // 取得所有已安裝且在線的設備集合
    let dev_info = unsafe {
        SetupDiGetClassDevsW(
            None, // ClassGuid = null → 全部類別
            None, // Enumerator = null
            None, // hwndParent = null
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        )?
    };

    let mut devices = Vec::new();
    let mut index: u32 = 0;

    loop {
        let mut dev_info_data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..unsafe { std::mem::zeroed() }
        };

        let ok = unsafe { SetupDiEnumDeviceInfo(dev_info, index, &mut dev_info_data) };

        if ok.is_err() {
            break; // 枚舉結束
        }

        // 讀取各屬性
        let name =
            unsafe { get_device_property_string(dev_info, &mut dev_info_data, SPDRP_DEVICEDESC) };
        let vendor = unsafe { get_device_property_string(dev_info, &mut dev_info_data, SPDRP_MFG) };
        let hw_id =
            unsafe { get_device_property_string(dev_info, &mut dev_info_data, SPDRP_HARDWAREID) };
        let class =
            unsafe { get_device_property_string(dev_info, &mut dev_info_data, SPDRP_CLASS) };
        let driver = unsafe { get_driver_version(dev_info, &mut dev_info_data) };

        // 設備狀態 (透過 CM_Get_DevNode_Status)
        let status = unsafe { get_device_status_string(dev_info_data.DevInst) };

        // 過濾掉名稱為空的設備（通常是總線控制器等虛擬節點）
        if !name.is_empty() {
            devices.push(DeviceInfo {
                id: if hw_id.is_empty() {
                    format!("DEV_{:04}", index)
                } else {
                    hw_id
                },
                name,
                vendor: if vendor.is_empty() {
                    "(Unknown vendor)".to_string()
                } else {
                    vendor
                },
                version: if driver.is_empty() {
                    "(Unknown)".to_string()
                } else {
                    driver
                },
                status,
                class: if class.is_empty() {
                    "(Uncategorized)".to_string()
                } else {
                    class
                },
            });
        }

        index += 1;
    }

    // 釋放設備信息集合
    unsafe {
        let _ = SetupDiDestroyDeviceInfoList(dev_info);
    }

    info!(
        "硬體掃描完成，找到 {} 個設備 (共枚舉 {} 個節點)",
        devices.len(),
        index
    );
    Ok(devices)
}

/// 初始化硬體偵測模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    info!("硬體偵測模組初始化完成 (SetupDI backend)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection_basics() {
        let result = init();
        assert!(result.is_ok(), "硬體偵測初始化應該成功");

        let scan_result = scan_all_devices();
        assert!(scan_result.is_ok(), "硬體掃描應該能執行成功");
        let devices = scan_result.unwrap();
        assert!(!devices.is_empty(), "應該至少找到一個設備");

        // 驗證每個設備都有名稱
        for d in &devices {
            assert!(!d.name.is_empty(), "設備名稱不應為空");
        }
    }
}
