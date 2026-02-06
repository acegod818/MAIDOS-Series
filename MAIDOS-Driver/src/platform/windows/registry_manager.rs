//! Windows 註冊表管理器
//!
//! 管理 Windows 註冊表的讀寫操作

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegDeleteValueW, RegEnumKeyExW, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW, HKEY, KEY_READ, KEY_WRITE, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
    REG_VALUE_TYPE,
};
/// 註冊表管理器
pub struct RegistryManager;

/// 註冊表值類型
#[derive(Debug, Clone)]
pub enum RegistryValue {
    String(String),
    DWORD(u32),
    QWORD(u64),
    Binary(Vec<u8>),
    MultiString(Vec<String>),
}

impl RegistryManager {
    /// 讀取註冊表字符串值
    pub fn read_string(
        hkey: HKEY,
        subkey: &str,
        value_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::debug!("讀取註冊表字符串值: {:?}\\{}\\{}", hkey, subkey, value_name);

        // 打開註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegOpenKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey_opened as *mut HKEY,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot open registry key: {:?}", result).into());
        }

        // 讀取值
        let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let mut buffer = [0u16; 1024];
        let mut buffer_size = (buffer.len() * 2) as u32;
        let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);

        let result = unsafe {
            RegQueryValueExW(
                hkey_opened,
                PCWSTR(wide_value_name.as_ptr()),
                None,
                Some(&mut value_type as *mut _),
                Some(buffer.as_mut_ptr() as *mut u8),
                Some(&mut buffer_size),
            )
        };

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot read registry value: {:?}", result).into());
        }

        // 確保是字符串類型
        if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
            return Err("註冊表值不是字符串類型".into());
        }

        // 轉換為字符串
        let value = OsString::from_wide(&buffer).to_string_lossy().into_owned();

        Ok(value)
    }

    /// 寫入註冊表字符串值
    pub fn write_string(
        hkey: HKEY,
        subkey: &str,
        value_name: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!(
            "寫入註冊表字符串值: {:?}\\{}\\{} = {}",
            hkey,
            subkey,
            value_name,
            value
        );

        // 打開或創建註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegCreateKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                None,
                windows::Win32::System::Registry::REG_OPEN_CREATE_OPTIONS(0),
                KEY_WRITE,
                None,
                &mut hkey_opened as *mut HKEY,
                None,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot create/open registry key: {:?}", result).into());
        }

        // 寫入值
        let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let wide_value: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegSetValueExW(
                hkey_opened,
                PCWSTR(wide_value_name.as_ptr()),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    wide_value.as_ptr() as *const u8,
                    wide_value.len() * 2,
                )),
            )
        };

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot write registry value: {:?}", result).into());
        }

        Ok(())
    }

    /// 讀取註冊表 DWORD 值
    pub fn read_dword(
        hkey: HKEY,
        subkey: &str,
        value_name: &str,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        log::debug!(
            "讀取註冊表 DWORD 值: {:?}\\{}\\{}",
            hkey,
            subkey,
            value_name
        );

        // 打開註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegOpenKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey_opened as *mut HKEY,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot open registry key: {:?}", result).into());
        }

        // 讀取值
        let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
        let mut value: u32 = 0;
        let mut buffer_size = mem::size_of::<u32>() as u32;
        let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);

        let result = unsafe {
            RegQueryValueExW(
                hkey_opened,
                PCWSTR(wide_value_name.as_ptr()),
                None,
                Some(&mut value_type as *mut _),
                Some(&mut value as *mut u32 as *mut u8),
                Some(&mut buffer_size),
            )
        };

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot read registry value: {:?}", result).into());
        }

        // 確保是 DWORD 類型
        if value_type != REG_DWORD {
            return Err("註冊表值不是 DWORD 類型".into());
        }

        Ok(value)
    }

    /// 寫入註冊表 DWORD 值
    pub fn write_dword(
        hkey: HKEY,
        subkey: &str,
        value_name: &str,
        value: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!(
            "寫入註冊表 DWORD 值: {:?}\\{}\\{} = {}",
            hkey,
            subkey,
            value_name,
            value
        );

        // 打開或創建註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegCreateKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                None,
                windows::Win32::System::Registry::REG_OPEN_CREATE_OPTIONS(0),
                KEY_WRITE,
                None,
                &mut hkey_opened as *mut HKEY,
                None,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot create/open registry key: {:?}", result).into());
        }

        // 寫入值
        let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegSetValueExW(
                hkey_opened,
                PCWSTR(wide_value_name.as_ptr()),
                0,
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &value as *const u32 as *const u8,
                    mem::size_of::<u32>(),
                )),
            )
        };

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot write registry value: {:?}", result).into());
        }

        Ok(())
    }

    /// 刪除註冊表鍵
    pub fn delete_key(hkey: HKEY, subkey: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("刪除註冊表鍵: {:?}\\{}", hkey, subkey);

        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe { RegDeleteKeyW(hkey, PCWSTR(wide_subkey.as_ptr())) };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot delete registry key: {:?}", result).into());
        }

        Ok(())
    }

    /// 刪除註冊表值
    pub fn delete_value(
        hkey: HKEY,
        subkey: &str,
        value_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("刪除註冊表值: {:?}\\{}\\{}", hkey, subkey, value_name);

        // 打開註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegOpenKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                KEY_WRITE,
                &mut hkey_opened as *mut HKEY,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot open registry key: {:?}", result).into());
        }

        // 刪除值
        let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();

        let result = unsafe { RegDeleteValueW(hkey_opened, PCWSTR(wide_value_name.as_ptr())) };

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot delete registry value: {:?}", result).into());
        }

        Ok(())
    }

    /// 枚舉註冊表子鍵
    pub fn enumerate_subkeys(
        hkey: HKEY,
        subkey: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        log::debug!("枚舉註冊表子鍵: {:?}\\{}", hkey, subkey);

        // 打開註冊表鍵
        let mut hkey_opened: HKEY = HKEY(ptr::null_mut());
        let wide_subkey: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            RegOpenKeyExW(
                hkey,
                PCWSTR(wide_subkey.as_ptr()),
                0,
                KEY_READ,
                &mut hkey_opened as *mut HKEY,
            )
        };

        if result != WIN32_ERROR(0) {
            return Err(format!("Cannot open registry key: {:?}", result).into());
        }

        // 枚舉子鍵
        let mut subkeys = Vec::new();
        let mut index = 0;

        loop {
            let mut buffer = [0u16; 256];
            let mut buffer_size = buffer.len() as u32;

            let mut class_name = [0u16; 256];
            let mut class_name_len = 0u32;
            let mut last_write_time = unsafe { std::mem::zeroed() };

            let result = unsafe {
                RegEnumKeyExW(
                    hkey_opened,
                    index,
                    windows::core::PWSTR(buffer.as_mut_ptr()),
                    &mut buffer_size,
                    Some(&class_name_len),
                    windows::core::PWSTR(class_name.as_mut_ptr()),
                    Some(&mut class_name_len),
                    Some(&mut last_write_time),
                )
            };

            if result == windows::Win32::Foundation::ERROR_NO_MORE_FILES {
                break;
            }

            if result != WIN32_ERROR(0) {
                // 關閉註冊表鍵
                unsafe {
                    let _ = RegCloseKey(hkey_opened);
                }
                return Err(format!("Cannot enumerate registry subkeys: {:?}", result).into());
            }

            let subkey_name = OsString::from_wide(&buffer[..buffer_size as usize])
                .to_string_lossy()
                .into_owned();

            subkeys.push(subkey_name);
            index += 1;
        }

        // 關閉註冊表鍵
        unsafe {
            let _ = RegCloseKey(hkey_opened);
        }

        Ok(subkeys)
    }
}

/// 初始化註冊表管理器模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Windows 註冊表管理器模組初始化完成");
    Ok(())
}
