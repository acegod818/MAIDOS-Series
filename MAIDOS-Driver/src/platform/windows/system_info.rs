//! Windows 系統信息
//!
//! 獲取 Windows 系統的詳細信息

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};
use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION};
use windows::Win32::System::Registry::REG_VALUE_TYPE;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ,
};
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, GetSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
};
use windows::Win32::System::Threading::GetCurrentProcess;

/// 系統信息
#[derive(Debug, Clone)]
pub struct WindowsSystemInfo {
    pub os_version: String,
    pub build_number: String,
    pub architecture: String,
    pub total_memory: u64,
    pub available_memory: u64,
    pub processor_count: u32,
    pub system_language: String,
}

/// 獲取 Windows 系統信息
pub fn get_system_info() -> Result<WindowsSystemInfo, Box<dyn std::error::Error>> {
    log::info!("獲取 Windows 系統信息");

    // 獲取 OS 版本
    let os_version = get_os_version()?;

    // 獲取系統架構
    let architecture = get_system_architecture();

    // 獲取內存信息
    let (total_memory, available_memory) = get_memory_info()?;

    // 獲取處理器信息
    let processor_count = get_processor_count();

    // 獲取系統語言
    let system_language = get_system_language();

    let system_info = WindowsSystemInfo {
        os_version,
        build_number: "Unknown".to_string(), // 需要進一步實現
        architecture,
        total_memory,
        available_memory,
        processor_count,
        system_language,
    };

    log::info!("Windows 系統信息獲取完成");
    Ok(system_info)
}

/// 獲取 OS 版本
fn get_os_version() -> Result<String, Box<dyn std::error::Error>> {
    // 打開註冊表鍵
    let mut hkey: HKEY = HKEY(ptr::null_mut());
    let result = unsafe {
        RegOpenKeyExW(
            HKEY::default(),
            windows::core::w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
            0,
            KEY_READ,
            &mut hkey as *mut HKEY,
        )
    };

    if result != WIN32_ERROR(0) {
        return Err("無法打開註冊表鍵".into());
    }

    // 讀取 CurrentVersion 值
    let mut buffer = [0u16; 256];
    let mut buffer_size = (buffer.len() * 2) as u32;
    let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            windows::core::w!("CurrentVersion"),
            None,
            Some(&mut value_type as *mut REG_VALUE_TYPE),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut buffer_size),
        )
    };

    let version = if result == WIN32_ERROR(0) {
        OsString::from_wide(&buffer).to_string_lossy().into_owned()
    } else {
        "Unknown".to_string()
    };

    // 關閉註冊表鍵
    unsafe {
        let _ = RegCloseKey(hkey);
    }

    Ok(version)
}

/// 獲取系統架構
fn get_system_architecture() -> String {
    let mut system_info: SYSTEM_INFO = unsafe { mem::zeroed() };
    unsafe {
        GetNativeSystemInfo(&mut system_info);
    }

    let arch = unsafe { system_info.Anonymous.Anonymous }.wProcessorArchitecture;
    match arch.0 {
        9 => "x64".to_string(),    // PROCESSOR_ARCHITECTURE_AMD64
        0 => "x86".to_string(),    // PROCESSOR_ARCHITECTURE_INTEL
        5 => "ARM".to_string(),    // PROCESSOR_ARCHITECTURE_ARM
        12 => "ARM64".to_string(), // PROCESSOR_ARCHITECTURE_ARM64
        _ => "Unknown".to_string(),
    }
}

/// 獲取內存信息
fn get_memory_info() -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let mut memory_status: MEMORYSTATUSEX = unsafe { mem::zeroed() };
    memory_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

    let result = unsafe { GlobalMemoryStatusEx(&mut memory_status) };

    if result.is_err() {
        return Err("無法獲取內存信息".into());
    }

    let total_memory = memory_status.ullTotalPhys;
    let available_memory = memory_status.ullAvailPhys;

    Ok((total_memory, available_memory))
}

/// 獲取處理器數量
fn get_processor_count() -> u32 {
    let mut system_info: SYSTEM_INFO = unsafe { mem::zeroed() };
    unsafe {
        GetSystemInfo(&mut system_info);
    }

    system_info.dwNumberOfProcessors
}

/// 獲取系統語言
fn get_system_language() -> String {
    let language_id = unsafe { GetUserDefaultUILanguage() };

    // 將語言 ID 轉換為字符串表示
    match language_id {
        0x0409 => "English (United States)".to_string(),
        0x0804 => "Chinese (Simplified)".to_string(),
        0x0404 => "Chinese (Traditional)".to_string(),
        _ => format!("Unknown (0x{:04x})", language_id),
    }
}

/// 檢查管理員權限
pub fn is_admin() -> bool {
    let token: HANDLE = HANDLE(ptr::null_mut());
    let mut elevation: TOKEN_ELEVATION = unsafe { mem::zeroed() };
    let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;

    let result = unsafe {
        GetTokenInformation(
            GetCurrentProcess(),
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            size,
            &mut size,
        )
    };

    let is_elevated = result.is_ok() && elevation.TokenIsElevated != 0;

    unsafe {
        let _ = CloseHandle(token);
    }

    is_elevated
}

/// 初始化系統信息模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Windows 系統信息模組初始化完成");
    Ok(())
}
