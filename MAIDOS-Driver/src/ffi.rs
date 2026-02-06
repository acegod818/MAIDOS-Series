//! MAIDOS-Driver FFI 接口
//!
//! 提供 C 兼容的接口供外部調用
//!
//! [MAIDOS-AUDIT] 已對齊 C# HardwareDetectionService.cs 的 16 個 DllImport 聲明
//! 原有 13 個 + 新增 3 個 (diagnose_device_c, rollback_driver_c, free_diagnostic_info)

// FFI extern "C" functions must dereference raw pointers from the caller but cannot be
// marked `unsafe` because that would break the C ABI contract for P/Invoke consumers.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::core::backup::manager::{backup_drivers, BackupEntry};
use crate::core::detect::hardware::{scan_all_devices, DeviceInfo};
use crate::core::diagnose::device_diagnostics::{diagnose_device, DiagnosticInfo};
use crate::core::install::installer::DriverInstaller;
use crate::core::install::rollback_handler::RollbackHandler;
use crate::core::restore::manager::restore_drivers;
use crate::core::update::checker::{
    apply_update, check_all_updates, check_driver_update, download_update, UpdateInfo,
};

// =====================================================================
// 全域錯誤存儲 (thread-safe)
// =====================================================================

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_last_error(msg: &str) {
    if let Ok(mut err) = LAST_ERROR.lock() {
        *err = msg.to_string();
    }
}

// =====================================================================
// FFI 資料結構
// =====================================================================

/// C 兼容的設備信息結構 (對齊 C# CDeviceInfo)
#[repr(C)]
pub struct CDeviceInfo {
    pub id: *mut c_char,
    pub name: *mut c_char,
    pub vendor: *mut c_char,
    pub version: *mut c_char,
    pub status: *mut c_char,
}

impl From<DeviceInfo> for CDeviceInfo {
    fn from(device: DeviceInfo) -> Self {
        CDeviceInfo {
            id: CString::new(device.id).unwrap_or_default().into_raw(),
            name: CString::new(device.name).unwrap_or_default().into_raw(),
            vendor: CString::new(device.vendor).unwrap_or_default().into_raw(),
            version: CString::new(device.version).unwrap_or_default().into_raw(),
            status: CString::new(device.status).unwrap_or_default().into_raw(),
        }
    }
}

/// C 兼容的更新資訊結構 (對齊 C# CUpdateInfo)
#[repr(C)]
pub struct CUpdateInfo {
    pub device_id: *mut c_char,
    pub current_version: *mut c_char,
    pub latest_version: *mut c_char,
    pub update_available: i32, // 0 = false, 1 = true
    pub status: *mut c_char,
    pub download_url: *mut c_char,
}

impl From<UpdateInfo> for CUpdateInfo {
    fn from(info: UpdateInfo) -> Self {
        CUpdateInfo {
            device_id: CString::new(info.device_id).unwrap_or_default().into_raw(),
            current_version: CString::new(info.current_version)
                .unwrap_or_default()
                .into_raw(),
            latest_version: CString::new(info.latest_version)
                .unwrap_or_default()
                .into_raw(),
            update_available: if info.update_available { 1 } else { 0 },
            status: CString::new(info.status).unwrap_or_default().into_raw(),
            download_url: CString::new(info.download_url).unwrap_or_default().into_raw(),
        }
    }
}

/// C 兼容的備份條目結構
#[repr(C)]
pub struct CBackupEntry {
    pub id: *mut c_char,
    pub timestamp: *mut c_char,
    pub path: *mut c_char,
    pub size: u64,
}

impl From<BackupEntry> for CBackupEntry {
    fn from(entry: BackupEntry) -> Self {
        CBackupEntry {
            id: CString::new(entry.id).unwrap_or_default().into_raw(),
            timestamp: CString::new(entry.timestamp).unwrap_or_default().into_raw(),
            path: CString::new(entry.path).unwrap_or_default().into_raw(),
            size: entry.size,
        }
    }
}

// =====================================================================
// Helper: C 指標 → Rust &str
// =====================================================================

unsafe fn ptr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, Box<dyn std::error::Error>> {
    if ptr.is_null() {
        return Err("null pointer".into());
    }
    Ok(CStr::from_ptr(ptr).to_str()?)
}

// =====================================================================
// FFI 導出函數 — 原有 4 個
// =====================================================================

/// 掃描所有硬體設備
///
/// 返回設備數量，設備信息存儲在 devices_ptr 中
/// 調用者負責釋放返回的設備信息
#[no_mangle]
pub extern "C" fn scan_all_devices_c(devices_ptr: *mut *mut CDeviceInfo) -> i32 {
    if devices_ptr.is_null() {
        set_last_error("devices_ptr is null");
        return -1;
    }

    match scan_all_devices() {
        Ok(devices) => {
            let len = devices.len() as i32;
            crate::core::audit::audit_success("SCAN", "ALL", &format!("{} devices", len));
            if len > 0 {
                let c_devices: Vec<CDeviceInfo> = devices.into_iter().map(|d| d.into()).collect();
                unsafe {
                    *devices_ptr = c_devices.as_ptr() as *mut CDeviceInfo;
                    std::mem::forget(c_devices);
                }
            }
            len
        }
        Err(e) => {
            crate::core::audit::audit_failure("SCAN", "ALL", &e.to_string());
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 釋放設備信息內存
#[no_mangle]
pub extern "C" fn free_device_info(devices: *mut CDeviceInfo, count: i32) {
    if devices.is_null() || count <= 0 {
        return;
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(devices, count as usize);
        for device in slice {
            if !device.id.is_null() {
                let _ = CString::from_raw(device.id);
            }
            if !device.name.is_null() {
                let _ = CString::from_raw(device.name);
            }
            if !device.vendor.is_null() {
                let _ = CString::from_raw(device.vendor);
            }
            if !device.version.is_null() {
                let _ = CString::from_raw(device.version);
            }
            if !device.status.is_null() {
                let _ = CString::from_raw(device.status);
            }
        }
        let _ = Vec::from_raw_parts(devices, count as usize, count as usize);
    }
}

/// 獲取最後一次錯誤信息
#[no_mangle]
pub extern "C" fn get_last_error() -> *mut c_char {
    let msg = LAST_ERROR
        .lock()
        .map(|e| e.clone())
        .unwrap_or_else(|_| "lock error".to_string());
    CString::new(msg).unwrap_or_default().into_raw()
}

/// 釋放字符串內存
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// =====================================================================
// FFI 導出函數 — 新增 6 個 (對齊 C# HardwareDetectionService)
// =====================================================================

/// 安裝驅動程式
///
/// inf_path: INF 檔案路徑 (UTF-8)
/// 返回: 0 = 成功, -1 = 失敗 (錯誤信息透過 get_last_error 獲取)
#[no_mangle]
pub extern "C" fn install_driver_c(inf_path: *const c_char) -> i32 {
    let path = match unsafe { ptr_to_str(inf_path) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid inf_path: {}", e));
            return -1;
        }
    };

    let installer = DriverInstaller;
    match installer.install_driver(path, "auto") {
        Ok(result) => {
            if result.success {
                0
            } else {
                set_last_error(&result.message);
                -1
            }
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 備份驅動程式
///
/// backup_path: 備份目的地路徑
/// entries_ptr: 輸出參數，指向 CBackupEntry 陣列
/// 返回: 備份條目數 (>= 0) 或 -1 表示失敗
#[no_mangle]
pub extern "C" fn backup_drivers_c(
    backup_path: *const c_char,
    entries_ptr: *mut *mut CBackupEntry,
) -> i32 {
    let path = match unsafe { ptr_to_str(backup_path) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid backup_path: {}", e));
            return -1;
        }
    };

    if entries_ptr.is_null() {
        set_last_error("entries_ptr is null");
        return -1;
    }

    match backup_drivers(path) {
        Ok(entries) => {
            let len = entries.len() as i32;
            if len > 0 {
                let c_entries: Vec<CBackupEntry> = entries.into_iter().map(|e| e.into()).collect();
                unsafe {
                    *entries_ptr = c_entries.as_ptr() as *mut CBackupEntry;
                    std::mem::forget(c_entries);
                }
            }
            len
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 釋放備份條目記憶體
#[no_mangle]
pub extern "C" fn free_backup_entries(entries: *mut CBackupEntry, count: i32) {
    if entries.is_null() || count <= 0 {
        return;
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(entries, count as usize);
        for entry in slice {
            if !entry.id.is_null() {
                let _ = CString::from_raw(entry.id);
            }
            if !entry.timestamp.is_null() {
                let _ = CString::from_raw(entry.timestamp);
            }
            if !entry.path.is_null() {
                let _ = CString::from_raw(entry.path);
            }
        }
        let _ = Vec::from_raw_parts(entries, count as usize, count as usize);
    }
}

/// 檢查單一設備的驅動更新
///
/// device_id: 設備 ID
/// update_server: 更新服務器 URL (可為 null，使用默認)
/// info_ptr: 輸出參數，指向 CUpdateInfo
/// 返回: 0 = 成功, -1 = 失敗
#[no_mangle]
pub extern "C" fn check_driver_update_c(
    device_id: *const c_char,
    update_server: *const c_char,
    info_ptr: *mut CUpdateInfo,
) -> i32 {
    let dev_id = match unsafe { ptr_to_str(device_id) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid device_id: {}", e));
            return -1;
        }
    };

    let server = if update_server.is_null() {
        None
    } else {
        unsafe { ptr_to_str(update_server).ok() }
    };

    if info_ptr.is_null() {
        set_last_error("info_ptr is null");
        return -1;
    }

    match check_driver_update(dev_id, server) {
        Ok(info) => {
            let c_info: CUpdateInfo = info.into();
            unsafe {
                std::ptr::write(info_ptr, c_info);
            }
            0
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 釋放更新資訊記憶體
#[no_mangle]
pub extern "C" fn free_update_info(info: *mut CUpdateInfo) {
    if info.is_null() {
        return;
    }

    unsafe {
        let i = &mut *info;
        if !i.device_id.is_null() {
            let _ = CString::from_raw(i.device_id);
        }
        if !i.current_version.is_null() {
            let _ = CString::from_raw(i.current_version);
        }
        if !i.latest_version.is_null() {
            let _ = CString::from_raw(i.latest_version);
        }
        if !i.status.is_null() {
            let _ = CString::from_raw(i.status);
        }
        if !i.download_url.is_null() {
            let _ = CString::from_raw(i.download_url);
        }
    }
}

/// 下載驅動更新
///
/// download_url: 下載 URL
/// save_path: 本地儲存路徑
/// 返回: 下載的位元組數 (>= 0) 或 -1 表示失敗
#[no_mangle]
pub extern "C" fn download_update_c(download_url: *const c_char, save_path: *const c_char) -> i64 {
    let url = match unsafe { ptr_to_str(download_url) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid download_url: {}", e));
            return -1;
        }
    };

    let path = match unsafe { ptr_to_str(save_path) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid save_path: {}", e));
            return -1;
        }
    };

    match download_update(url, path) {
        Ok(bytes) => bytes as i64,
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 套用驅動更新
///
/// inf_path: INF 檔案路徑
/// device_id: 設備 ID (可為 null，表示自動)
/// 返回: 0 = 成功, -1 = 失敗
#[no_mangle]
pub extern "C" fn apply_update_c(inf_path: *const c_char, device_id: *const c_char) -> i32 {
    let path = match unsafe { ptr_to_str(inf_path) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid inf_path: {}", e));
            return -1;
        }
    };

    let dev_id = if device_id.is_null() {
        None
    } else {
        unsafe { ptr_to_str(device_id).ok() }
    };

    match apply_update(path, dev_id) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error("Apply update returned false");
            -1
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 批次檢查所有設備的驅動更新
///
/// updates_ptr: 輸出參數，指向 CUpdateInfo 陣列
/// 返回: 更新資訊數量 (>= 0) 或 -1 表示失敗
#[no_mangle]
pub extern "C" fn check_all_updates_c(updates_ptr: *mut *mut CUpdateInfo) -> i32 {
    if updates_ptr.is_null() {
        set_last_error("updates_ptr is null");
        return -1;
    }

    match check_all_updates(None) {
        Ok(updates) => {
            let len = updates.len() as i32;
            if len > 0 {
                let c_updates: Vec<CUpdateInfo> = updates.into_iter().map(|u| u.into()).collect();
                unsafe {
                    *updates_ptr = c_updates.as_ptr() as *mut CUpdateInfo;
                    std::mem::forget(c_updates);
                }
            }
            len
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 釋放更新資訊陣列記憶體
#[no_mangle]
pub extern "C" fn free_update_info_array(updates: *mut CUpdateInfo, count: i32) {
    if updates.is_null() || count <= 0 {
        return;
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(updates, count as usize);
        for info in slice {
            if !info.device_id.is_null() {
                let _ = CString::from_raw(info.device_id);
            }
            if !info.current_version.is_null() {
                let _ = CString::from_raw(info.current_version);
            }
            if !info.latest_version.is_null() {
                let _ = CString::from_raw(info.latest_version);
            }
            if !info.status.is_null() {
                let _ = CString::from_raw(info.status);
            }
            if !info.download_url.is_null() {
                let _ = CString::from_raw(info.download_url);
            }
        }
        let _ = Vec::from_raw_parts(updates, count as usize, count as usize);
    }
}

// =====================================================================
// FFI 資料結構 — 診斷 (BUG-004)
// =====================================================================

/// C 兼容的診斷結果結構 (對齊 C# CDiagnosticInfo)
#[repr(C)]
pub struct CDiagnosticInfo {
    pub device_id: *mut c_char,
    pub problem_code: i32,
    pub problem_description: *mut c_char,
    pub irq: i32,
    pub status: *mut c_char,
}

impl From<DiagnosticInfo> for CDiagnosticInfo {
    fn from(info: DiagnosticInfo) -> Self {
        CDiagnosticInfo {
            device_id: CString::new(info.device_id).unwrap_or_default().into_raw(),
            problem_code: info.problem_code,
            problem_description: CString::new(info.problem_description)
                .unwrap_or_default()
                .into_raw(),
            irq: info.irq,
            status: CString::new(info.status).unwrap_or_default().into_raw(),
        }
    }
}

// =====================================================================
// FFI 導出函數 — BUG-004: 設備診斷
// =====================================================================

/// 診斷指定設備
///
/// device_id: 設備實例 ID (UTF-8)
/// info_ptr: 輸出參數，指向 CDiagnosticInfo
/// 返回: 0 = 成功, -1 = 失敗
#[no_mangle]
pub extern "C" fn diagnose_device_c(
    device_id: *const c_char,
    info_ptr: *mut CDiagnosticInfo,
) -> i32 {
    let dev_id = match unsafe { ptr_to_str(device_id) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid device_id: {}", e));
            return -1;
        }
    };

    if info_ptr.is_null() {
        set_last_error("info_ptr is null");
        return -1;
    }

    match diagnose_device(dev_id) {
        Ok(info) => {
            let c_info: CDiagnosticInfo = info.into();
            unsafe {
                std::ptr::write(info_ptr, c_info);
            }
            0
        }
        Err(e) => {
            set_last_error(&e.to_string());
            -1
        }
    }
}

/// 釋放診斷結果記憶體
#[no_mangle]
pub extern "C" fn free_diagnostic_info(info: *mut CDiagnosticInfo) {
    if info.is_null() {
        return;
    }

    unsafe {
        let i = &mut *info;
        if !i.device_id.is_null() {
            let _ = CString::from_raw(i.device_id);
        }
        if !i.problem_description.is_null() {
            let _ = CString::from_raw(i.problem_description);
        }
        if !i.status.is_null() {
            let _ = CString::from_raw(i.status);
        }
    }
}

// =====================================================================
// FFI 導出函數 — BUG-005: 驅動回滾
// =====================================================================

/// 回滾驅動程式
///
/// device_id: 設備 ID (UTF-8)
/// backup_path: 備份路徑 (UTF-8，可為 null 表示使用系統還原)
/// 返回: 0 = 成功, -1 = 失敗
#[no_mangle]
pub extern "C" fn rollback_driver_c(device_id: *const c_char, backup_path: *const c_char) -> i32 {
    let dev_id = match unsafe { ptr_to_str(device_id) } {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("Invalid device_id: {}", e));
            return -1;
        }
    };

    let bk_path = if backup_path.is_null() {
        None
    } else {
        unsafe { ptr_to_str(backup_path).ok() }
    };

    // 策略: 若有 backup_path 則從備份還原，否則使用 RollbackHandler
    if let Some(path) = bk_path {
        match restore_drivers(path) {
            Ok(()) => {
                crate::core::audit::audit_success("ROLLBACK", dev_id, path);
                0
            }
            Err(e) => {
                crate::core::audit::audit_failure("ROLLBACK", dev_id, &e.to_string());
                set_last_error(&e.to_string());
                -1
            }
        }
    } else {
        let mut handler = RollbackHandler::new();
        match handler.perform_rollback(dev_id) {
            Ok(()) => {
                crate::core::audit::audit_success("ROLLBACK", dev_id, "via RollbackHandler");
                0
            }
            Err(e) => {
                crate::core::audit::audit_failure("ROLLBACK", dev_id, &e.to_string());
                set_last_error(&e.to_string());
                -1
            }
        }
    }
}
