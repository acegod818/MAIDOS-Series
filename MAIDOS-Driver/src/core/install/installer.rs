//! 驅動安裝器
//!
//! 負責驅動程序的安全安裝

use std::path::Path;
use std::process::Command;

/// 驅動安裝器
pub struct DriverInstaller;

/// 安裝結果
#[derive(Debug)]
pub struct InstallationResult {
    pub success: bool,
    pub message: String,
    pub installed_files: Vec<String>,
    pub installation_time: std::time::Duration,
}

impl DriverInstaller {
    /// Install a driver with automatic system restore point creation (AC-005).
    pub fn install_driver(
        &self,
        driver_path: &str,
        device_id: &str,
    ) -> Result<InstallationResult, Box<dyn std::error::Error>> {
        log::info!("Installing driver: {}, device: {}", driver_path, device_id);

        let start_time = std::time::Instant::now();

        if !Path::new(driver_path).exists() {
            crate::core::audit::audit_failure("INSTALL", device_id, &format!("File not found: {}", driver_path));
            return Err(format!("Driver file not found: {}", driver_path).into());
        }

        // AC-005: Create system restore point before installation
        create_restore_point("MAIDOS Driver Install");

        let installation_success = self.perform_installation(driver_path, device_id)?;

        let installation_time = start_time.elapsed();

        let result = InstallationResult {
            success: installation_success,
            message: if installation_success {
                "Driver installed successfully".to_string()
            } else {
                "Driver installation failed".to_string()
            },
            installed_files: vec![driver_path.to_string()], // 簡化實現
            installation_time,
        };

        if result.success {
            crate::core::audit::audit_success("INSTALL", device_id, driver_path);
        } else {
            crate::core::audit::audit_failure("INSTALL", device_id, &result.message);
        }

        Ok(result)
    }

    /// 執行驅動安裝 (真實實現: 使用 pnputil /add-driver)
    fn perform_installation(
        &self,
        driver_path: &str,
        _device_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        log::info!("正在透過 pnputil 安裝驅動: {}", driver_path);

        let output = Command::new("pnputil")
            .args(["/add-driver", driver_path, "/install"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        log::info!("pnputil stdout: {}", stdout);
        if !stderr.is_empty() {
            log::warn!("pnputil stderr: {}", stderr);
        }

        if output.status.success() {
            log::info!("驅動安裝完成，可能需要重啟系統");
            Ok(true)
        } else {
            let msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            Err(format!("pnputil install failed (exit {}): {}", output.status, msg).into())
        }
    }

    /// 批量安裝多個驅動
    pub fn install_multiple_drivers(
        &self,
        drivers: &[DriverInstallationInfo],
    ) -> Result<Vec<InstallationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for driver_info in drivers {
            match self.install_driver(&driver_info.driver_path, &driver_info.device_id) {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    log::error!("驅動安裝失敗: {} - {}", driver_info.driver_path, e);
                    results.push(InstallationResult {
                        success: false,
                        message: format!("Installation failed: {}", e),
                        installed_files: vec![],
                        installation_time: std::time::Duration::from_secs(0),
                    });
                }
            }
        }

        Ok(results)
    }

    /// 卸載驅動程序
    pub fn uninstall_driver(
        &self,
        device_id: &str,
    ) -> Result<InstallationResult, Box<dyn std::error::Error>> {
        log::info!("開始卸載驅動，設備: {}", device_id);

        let start_time = std::time::Instant::now();

        // 在實際實現中，這裡會執行真正的驅動卸載過程
        // 可能包括：
        // 1. 停止相關服務或進程
        // 2. 刪除驅動文件
        // 3. 取消註冊驅動
        // 4. 清理註冊表項
        // 5. 重啟相關服務

        let uninstallation_success = self.perform_uninstallation(device_id)?;

        let installation_time = start_time.elapsed();

        let result = InstallationResult {
            success: uninstallation_success,
            message: if uninstallation_success {
                "Driver uninstalled successfully".to_string()
            } else {
                "Driver uninstall failed".to_string()
            },
            installed_files: vec![], // 卸載時不需要記錄安裝文件
            installation_time,
        };

        if result.success {
            log::info!("驅動卸載成功: {}, 耗時: {:?}", device_id, installation_time);
        } else {
            log::error!("驅動卸載失敗: {}", device_id);
        }

        Ok(result)
    }

    /// 執行驅動卸載 (真實實現: 使用 pnputil /remove-device)
    fn perform_uninstallation(&self, device_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        log::info!("正在透過 pnputil 卸載驅動: {}", device_id);

        let output = Command::new("pnputil")
            .args(["/remove-device", device_id])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        log::info!("pnputil 卸載結果: {}", stdout);

        Ok(output.status.success())
    }

    /// 重啟系統（如果需要）
    pub fn restart_system() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在重啟系統以完成驅動安裝");

        // 在實際實現中，這裡會執行系統重啟
        // 在 Windows 上可以使用 `shutdown /r /t 0` 命令

        #[cfg(windows)]
        {
            let output = Command::new("shutdown").args(["/r", "/t", "0"]).output()?;

            if !output.status.success() {
                return Err("系統重啟命令執行失敗".into());
            }
        }

        log::info!("系統重啟命令已發送");
        Ok(())
    }
}

/// 驅動安裝信息
#[derive(Debug, Clone)]
pub struct DriverInstallationInfo {
    pub driver_path: String,
    pub device_id: String,
    pub backup_required: bool,
}

/// Create a system restore point before driver installation (AC-005).
///
/// Uses PowerShell `Checkpoint-Computer` to invoke SRSetRestorePoint.
/// If System Restore is disabled, logs a warning and proceeds.
fn create_restore_point(description: &str) {
    log::info!("Creating system restore point: {}", description);

    let ps_script = format!(
        "try {{ Checkpoint-Computer -Description '{}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; Write-Output 'OK' }} catch {{ Write-Output \"WARN: $($_.Exception.Message)\" }}",
        description
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let msg = stdout.trim();
            if msg == "OK" {
                log::info!("System restore point created successfully");
                crate::core::audit::audit_success("RESTORE_POINT", "SYSTEM", description);
            } else {
                log::warn!("System restore point: {}", msg);
                crate::core::audit::audit_failure("RESTORE_POINT", "SYSTEM", msg);
            }
        }
        Err(e) => {
            log::warn!("Failed to create restore point: {}", e);
            crate::core::audit::audit_failure("RESTORE_POINT", "SYSTEM", &e.to_string());
        }
    }
}

/// Initialize installer module.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Installer module initialized");
    Ok(())
}
