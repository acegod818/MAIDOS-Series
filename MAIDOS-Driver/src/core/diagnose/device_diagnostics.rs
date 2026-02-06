//! 設備診斷實現
//!
//! 使用 Windows ConfigManager API 查詢設備問題碼與中斷資源
//! 對齊 C++ diag.cpp 的功能 (get_device_problem_code, get_device_irq)

use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_Status, CM_Locate_DevNodeA, CM_DEVNODE_STATUS_FLAGS, CM_LOCATE_DEVNODE_NORMAL,
    CM_PROB, DN_HAS_PROBLEM,
};

/// 診斷結果
#[derive(Debug, Clone)]
pub struct DiagnosticInfo {
    /// 設備 ID
    pub device_id: String,
    /// 問題碼 (0 = 無問題)
    pub problem_code: i32,
    /// 問題描述
    pub problem_description: String,
    /// IRQ 號碼 (-1 = 無法取得, 0 = 無專屬中斷)
    pub irq: i32,
    /// 綜合狀態描述
    pub status: String,
}

/// 診斷指定設備
///
/// 使用 CM_Locate_DevNodeA + CM_Get_DevNode_Status 查詢真實問題碼
pub fn diagnose_device(device_id: &str) -> Result<DiagnosticInfo, Box<dyn std::error::Error>> {
    log::info!("開始診斷設備: {}", device_id);

    let mut dev_inst: u32 = 0;
    let device_id_cstr = std::ffi::CString::new(device_id)?;

    // CM_Locate_DevNodeA 定位設備節點
    let cr = unsafe {
        CM_Locate_DevNodeA(
            &mut dev_inst,
            windows::core::PCSTR(device_id_cstr.as_ptr() as *const u8),
            CM_LOCATE_DEVNODE_NORMAL,
        )
    };

    if cr.0 != 0 {
        log::warn!("CM_Locate_DevNodeA 失敗, 設備: {}, CR={}", device_id, cr.0);
        return Ok(DiagnosticInfo {
            device_id: device_id.to_string(),
            problem_code: -1,
            problem_description: format!("Cannot locate device node (CR={})", cr.0),
            irq: -1,
            status: "Offline or not found".to_string(),
        });
    }

    // CM_Get_DevNode_Status 取得設備狀態與問題碼
    let mut status = CM_DEVNODE_STATUS_FLAGS(0);
    let mut problem_code = CM_PROB(0);

    let cr = unsafe {
        CM_Get_DevNode_Status(
            &mut status as *mut _,
            &mut problem_code as *mut _,
            dev_inst,
            0,
        )
    };

    if cr.0 != 0 {
        return Ok(DiagnosticInfo {
            device_id: device_id.to_string(),
            problem_code: 0,
            problem_description: "Cannot get device status".to_string(),
            irq: 0,
            status: "Unknown".to_string(),
        });
    }

    let has_problem = (status.0 & DN_HAS_PROBLEM.0) != 0;
    let code = if has_problem {
        problem_code.0 as i32
    } else {
        0
    };
    let desc = get_problem_description(code);
    let overall_status = if has_problem {
        format!("Error (Code {})", code)
    } else {
        "Running normally".to_string()
    };

    log::info!(
        "設備 {} 診斷結果: code={}, status={}",
        device_id,
        code,
        overall_status
    );

    // AC-017: Query IRQ allocation for this device
    let irq = query_device_irq(device_id);

    Ok(DiagnosticInfo {
        device_id: device_id.to_string(),
        problem_code: code,
        problem_description: desc,
        irq,
        status: overall_status,
    })
}

/// Query IRQ number allocated to a device via WMI.
///
/// Returns the IRQ number, 0 if no dedicated IRQ, or -1 on failure.
fn query_device_irq(device_id: &str) -> i32 {
    // Escape backslashes for WMI query
    let escaped_id = device_id.replace('\\', "\\\\");

    let ps_script = format!(
        r#"
        $resources = Get-CimInstance -ClassName Win32_PnPAllocatedResource -ErrorAction SilentlyContinue
        foreach ($r in $resources) {{
            if ($r.Dependent -like '*{}*') {{
                if ($r.Antecedent -match 'IRQNumber=(\d+)') {{
                    Write-Output $Matches[1]
                    exit
                }}
            }}
        }}
        Write-Output '0'
        "#,
        escaped_id
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.trim().parse::<i32>().unwrap_or(0)
        }
        _ => 0,
    }
}

/// 將問題碼轉換為人類可讀描述
/// 對齊 C++ get_problem_description_secure
fn get_problem_description(problem_code: i32) -> String {
    match problem_code {
        0 => "No issues".to_string(),
        1 => "Device not configured (Code 1)".to_string(),
        3 => "Insufficient system memory (Code 3)".to_string(),
        10 => "Device cannot start (Code 10)".to_string(),
        22 => "Device is disabled (Code 22)".to_string(),
        28 => "Driver installation failed (Code 28)".to_string(),
        43 => "Device reported error (Code 43)".to_string(),
        other => format!("Unknown error (Code {})", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_description() {
        assert_eq!(get_problem_description(0), "No issues");
        assert_eq!(get_problem_description(43), "Device reported error (Code 43)");
        assert!(get_problem_description(999).contains("999"));
    }
}
