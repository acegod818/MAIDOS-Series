//! 回滾處理器
//!
//! 處理驅動安裝失敗時的回滾操作

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 回滾處理器
pub struct RollbackHandler {
    /// 安裝操作記錄
    installation_records: HashMap<String, InstallationRecord>,
}

/// 安裝記錄
#[derive(Debug, Clone)]
pub struct InstallationRecord {
    pub device_id: String,
    pub driver_path: String,
    pub backup_paths: Vec<String>,
    pub installation_time: DateTime<Utc>,
    pub status: InstallationStatus,
}

/// 安裝狀態
#[derive(Debug, Clone)]
pub enum InstallationStatus {
    Installing,
    Installed,
    Failed,
    RollingBack,
    RolledBack,
}

impl Default for RollbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackHandler {
    /// 創建新的回滾處理器
    pub fn new() -> Self {
        Self {
            installation_records: HashMap::new(),
        }
    }

    /// 記錄安裝開始
    pub fn record_installation_start(&mut self, device_id: &str, driver_path: &str) {
        let record = InstallationRecord {
            device_id: device_id.to_string(),
            driver_path: driver_path.to_string(),
            backup_paths: Vec::new(),
            installation_time: Utc::now(),
            status: InstallationStatus::Installing,
        };

        self.installation_records
            .insert(device_id.to_string(), record);
    }

    /// 更新安裝記錄狀態
    pub fn update_installation_status(&mut self, device_id: &str, status: InstallationStatus) {
        if let Some(record) = self.installation_records.get_mut(device_id) {
            record.status = status;
        }
    }

    /// 添加備份路徑
    pub fn add_backup_path(&mut self, device_id: &str, backup_path: &str) {
        if let Some(record) = self.installation_records.get_mut(device_id) {
            record.backup_paths.push(backup_path.to_string());
        }
    }

    /// 執行回滾操作
    pub fn perform_rollback(&mut self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("開始回滾操作，設備: {}", device_id);

        // 更新狀態為回滾中
        self.update_installation_status(device_id, InstallationStatus::RollingBack);

        // 獲取安裝記錄
        let record = match self.installation_records.get(device_id) {
            Some(r) => r.clone(),
            None => return Err("找不到安裝記錄".into()),
        };

        // 執行實際的回滾邏輯
        self.execute_rollback(&record)?;

        // 更新狀態為已回滾
        self.update_installation_status(device_id, InstallationStatus::RolledBack);

        log::info!("回滾操作完成，設備: {}", device_id);
        Ok(())
    }

    /// Execute the actual rollback by reinstalling backed-up driver packages.
    fn execute_rollback(
        &self,
        record: &InstallationRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Executing rollback for device: {}", record.device_id);

        for backup_path in &record.backup_paths {
            let path = std::path::Path::new(backup_path);
            if path.exists() && path.extension().and_then(|e| e.to_str()) == Some("inf") {
                log::info!("Reinstalling backup driver: {}", backup_path);
                let output = std::process::Command::new("pnputil")
                    .args(["/add-driver", backup_path, "/install"])
                    .output()?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("pnputil rollback failed for {}: {}", backup_path, stderr);
                }
            }
        }

        Ok(())
    }

    /// 自動回滾（當安裝失敗時）
    pub fn auto_rollback_on_failure(
        &mut self,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::warn!("檢測到安裝失敗，自動執行回滾，設備: {}", device_id);

        // 更新狀態為失敗
        self.update_installation_status(device_id, InstallationStatus::Failed);

        // 執行回滾
        self.perform_rollback(device_id)
    }

    /// 清理舊記錄
    pub fn cleanup_old_records(&mut self, max_age_days: i64) {
        let cutoff_time = Utc::now() - chrono::Duration::days(max_age_days);

        self.installation_records
            .retain(|_, record| record.installation_time > cutoff_time);
    }

    /// 獲取設備的安裝記錄
    pub fn get_installation_record(&self, device_id: &str) -> Option<&InstallationRecord> {
        self.installation_records.get(device_id)
    }
}

/// 初始化回滾處理器模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("回滾處理器模組初始化完成");
    Ok(())
}
