//! Windows 服務控制器
//!
//! 管理 Windows 服務的啟動、停止和狀態查詢

use std::mem;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
    OpenServiceW, QueryServiceStatus, StartServiceW, SC_HANDLE, SERVICE_CONTROL_STOP,
    SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_QUERY_STATUS, SERVICE_START,
    SERVICE_STATUS, SERVICE_STATUS_CURRENT_STATE, SERVICE_STOP, SERVICE_WIN32_OWN_PROCESS,
};

/// 服務控制器
pub struct ServiceController;

/// 服務狀態
#[derive(Debug, Clone)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Paused,
    PausePending,
    ContinuePending,
}

impl ServiceController {
    /// 打開服務控制管理器
    fn open_sc_manager() -> Result<SC_HANDLE, Box<dyn std::error::Error>> {
        let manager_handle =
            unsafe { OpenSCManagerW(None, None, GENERIC_READ.0 | GENERIC_WRITE.0) }?;

        Ok(manager_handle)
    }

    /// 打開服務
    fn open_service(
        manager_handle: SC_HANDLE,
        service_name: &str,
        access: u32,
    ) -> Result<SC_HANDLE, Box<dyn std::error::Error>> {
        let wide_service_name: Vec<u16> = service_name.encode_utf16().chain(Some(0)).collect();

        let service_handle =
            unsafe { OpenServiceW(manager_handle, PCWSTR(wide_service_name.as_ptr()), access) }?;

        Ok(service_handle)
    }

    /// 啟動服務
    pub fn start_service(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("啟動服務: {}", service_name);

        let manager_handle = Self::open_sc_manager()?;
        let service_handle = Self::open_service(manager_handle, service_name, SERVICE_START)?;

        let result = unsafe { StartServiceW(service_handle, None) };

        unsafe {
            let _ = CloseServiceHandle(service_handle);
            let _ = CloseServiceHandle(manager_handle);
        }

        if let Err(e) = result {
            return Err(format!("Cannot start service: {} - {:?}", service_name, e).into());
        }

        Ok(())
    }

    /// 停止服務
    pub fn stop_service(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("停止服務: {}", service_name);

        let manager_handle = Self::open_sc_manager()?;
        let service_handle = Self::open_service(manager_handle, service_name, SERVICE_STOP)?;

        let mut service_status: SERVICE_STATUS = unsafe { mem::zeroed() };
        let result =
            unsafe { ControlService(service_handle, SERVICE_CONTROL_STOP, &mut service_status) };

        unsafe {
            let _ = CloseServiceHandle(service_handle);
            let _ = CloseServiceHandle(manager_handle);
        }

        if let Err(e) = result {
            return Err(format!("Cannot stop service: {} - {:?}", service_name, e).into());
        }

        Ok(())
    }

    /// 獲取服務狀態
    pub fn get_service_status(
        service_name: &str,
    ) -> Result<ServiceState, Box<dyn std::error::Error>> {
        log::debug!("獲取服務狀態: {}", service_name);

        let manager_handle = Self::open_sc_manager()?;
        let service_handle =
            Self::open_service(manager_handle, service_name, SERVICE_QUERY_STATUS)?;

        let mut service_status: SERVICE_STATUS = unsafe { mem::zeroed() };
        let result = unsafe { QueryServiceStatus(service_handle, &mut service_status) };

        unsafe {
            let _ = CloseServiceHandle(service_handle);
            let _ = CloseServiceHandle(manager_handle);
        }

        if let Err(e) = result {
            return Err(format!("Cannot query service status: {} - {:?}", service_name, e).into());
        }

        let state = match service_status.dwCurrentState {
            SERVICE_STATUS_CURRENT_STATE(1) => ServiceState::Stopped,
            SERVICE_STATUS_CURRENT_STATE(2) => ServiceState::Starting,
            SERVICE_STATUS_CURRENT_STATE(3) => ServiceState::Stopping,
            SERVICE_STATUS_CURRENT_STATE(4) => ServiceState::Running,
            SERVICE_STATUS_CURRENT_STATE(5) => ServiceState::ContinuePending,
            SERVICE_STATUS_CURRENT_STATE(6) => ServiceState::PausePending,
            SERVICE_STATUS_CURRENT_STATE(7) => ServiceState::Paused,
            _ => ServiceState::Stopped,
        };

        Ok(state)
    }

    /// 創建服務
    pub fn create_service(
        service_name: &str,
        display_name: &str,
        binary_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("創建服務: {} ({})", service_name, display_name);

        let manager_handle = Self::open_sc_manager()?;

        let wide_service_name: Vec<u16> = service_name.encode_utf16().chain(Some(0)).collect();
        let wide_display_name: Vec<u16> = display_name.encode_utf16().chain(Some(0)).collect();
        let wide_binary_path: Vec<u16> = binary_path.encode_utf16().chain(Some(0)).collect();

        let service_handle = unsafe {
            CreateServiceW(
                manager_handle,
                PCWSTR(wide_service_name.as_ptr()),
                PCWSTR(wide_display_name.as_ptr()),
                GENERIC_READ.0 | GENERIC_WRITE.0,
                SERVICE_WIN32_OWN_PROCESS,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                PCWSTR(wide_binary_path.as_ptr()),
                None,
                None,
                None,
                None,
                None,
            )
        }?;

        unsafe {
            let _ = CloseServiceHandle(manager_handle);
        }

        unsafe {
            let _ = CloseServiceHandle(service_handle);
        }

        Ok(())
    }

    /// 刪除服務
    pub fn delete_service(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("刪除服務: {}", service_name);

        let manager_handle = Self::open_sc_manager()?;
        let service_handle =
            Self::open_service(manager_handle, service_name, SERVICE_QUERY_STATUS)?;

        let result = unsafe { DeleteService(service_handle) };

        unsafe {
            let _ = CloseServiceHandle(service_handle);
            let _ = CloseServiceHandle(manager_handle);
        }

        if let Err(e) = result {
            return Err(format!("Cannot delete service: {} - {:?}", service_name, e).into());
        }

        Ok(())
    }
}

/// 初始化服務控制器模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Windows 服務控制器模組初始化完成");
    Ok(())
}
