//! 備份管理器
//!
//! 管理驅動程序的備份和還原

use chrono::Utc;
use std::fs;
use std::path::Path;

/// 備份管理器
pub struct BackupManager {
    backup_directory: String,
}

/// 備份信息
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub backup_id: String,
    pub timestamp: String,
    pub device_id: String,
    pub backup_path: String,
    pub file_list: Vec<String>,
}

impl BackupManager {
    /// 創建新的備份管理器
    pub fn new(backup_directory: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 確保備份目錄存在
        fs::create_dir_all(backup_directory)?;

        Ok(Self {
            backup_directory: backup_directory.to_string(),
        })
    }

    /// 備份驅動文件
    pub fn backup_driver_files(
        &self,
        device_id: &str,
        driver_files: &[String],
    ) -> Result<BackupInfo, Box<dyn std::error::Error>> {
        log::info!("開始備份驅動文件，設備: {}", device_id);

        // 生成備份 ID 和時間戳
        let backup_id = self.generate_backup_id();
        let timestamp = Utc::now().to_rfc3339();
        let backup_path = format!("{}/{}", self.backup_directory, backup_id);

        // 創建備份目錄
        fs::create_dir_all(&backup_path)?;

        // 複製驅動文件到備份目錄
        let mut backed_up_files = Vec::new();

        for file_path in driver_files {
            let file_name = Path::new(file_path)
                .file_name()
                .ok_or("無效的文件路徑")?
                .to_str()
                .ok_or("無效的文件名")?;

            let backup_file_path = format!("{}/{}", backup_path, file_name);

            // 複製文件
            fs::copy(file_path, &backup_file_path)?;
            backed_up_files.push(backup_file_path);

            log::info!("已備份文件: {}", file_path);
        }

        let backup_info = BackupInfo {
            backup_id: backup_id.clone(),
            timestamp,
            device_id: device_id.to_string(),
            backup_path,
            file_list: backed_up_files,
        };

        log::info!("驅動文件備份完成，備份 ID: {}", backup_id);
        Ok(backup_info)
    }

    /// 生成唯一的備份 ID
    fn generate_backup_id(&self) -> String {
        let timestamp = Utc::now().timestamp();
        format!("backup_{}", timestamp)
    }

    /// 還原驅動文件
    pub fn restore_driver_files(&self, backup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("開始還原驅動文件，備份 ID: {}", backup_id);

        let backup_path = format!("{}/{}", self.backup_directory, backup_id);

        // 檢查備份目錄是否存在
        if !Path::new(&backup_path).exists() {
            return Err(format!("Backup directory not found: {}", backup_path).into());
        }

        // Reinstall all .inf files from the backup directory using pnputil
        for entry in std::fs::read_dir(&backup_path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().and_then(|e| e.to_str()) == Some("inf") {
                let inf_str = file_path.to_string_lossy().to_string();
                log::info!("Restoring driver from backup: {}", inf_str);
                let output = std::process::Command::new("pnputil")
                    .args(["/add-driver", &inf_str, "/install"])
                    .output()?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("pnputil restore failed for {}: {}", inf_str, stderr);
                }
            }
        }

        log::info!("Driver restore complete, backup ID: {}", backup_id);
        Ok(())
    }

    /// 列出所有備份
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>, Box<dyn std::error::Error>> {
        let mut backups = Vec::new();

        // 讀取備份目錄中的所有子目錄
        for entry in fs::read_dir(&self.backup_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let backup_id = path
                    .file_name()
                    .ok_or("無效的備份目錄名")?
                    .to_str()
                    .ok_or("無效的備份目錄名")?
                    .to_string();

                // 在實際實現中，這裡會從備份元數據文件中讀取備份信息
                // 目前使用簡化的實現
                let backup_info = BackupInfo {
                    backup_id: backup_id.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    device_id: "unknown".to_string(),
                    backup_path: path.to_string_lossy().to_string(),
                    file_list: vec![],
                };

                backups.push(backup_info);
            }
        }

        Ok(backups)
    }

    /// 刪除舊備份
    pub fn cleanup_old_backups(&self, max_age_days: i64) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("開始清理舊備份，最大保留天數: {}", max_age_days);

        let cutoff_time = Utc::now() - chrono::Duration::days(max_age_days);

        // 讀取備份目錄中的所有子目錄
        for entry in fs::read_dir(&self.backup_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let metadata = fs::metadata(&path)?;
                let modified_time = metadata.modified()?;

                // 檢查是否超過保留期限
                if modified_time < cutoff_time.into() {
                    fs::remove_dir_all(&path)?;
                    log::info!("已刪除舊備份: {:?}", path);
                }
            }
        }

        log::info!("舊備份清理完成");
        Ok(())
    }

    /// 匯出備份清單
    pub fn export_backup_list(&self, export_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("開始匯出備份清單到: {}", export_path);

        let backups = self.list_backups()?;

        // 將備份信息轉換為 CSV 格式
        let mut csv_content = "Backup ID,Timestamp,Device ID,Backup Path\n".to_string();

        for backup in backups {
            csv_content.push_str(&format!(
                "{},{},{},{}\n",
                backup.backup_id, backup.timestamp, backup.device_id, backup.backup_path
            ));
        }

        // 寫入文件
        fs::write(export_path, csv_content)?;

        log::info!("備份清單匯出完成: {}", export_path);
        Ok(())
    }
}

/// 初始化備份管理器模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("備份管理器模組初始化完成");
    Ok(())
}
