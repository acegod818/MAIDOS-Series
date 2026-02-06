//! 還原管理器
//!
//! [MAIDOS-AUDIT] M3 重寫: 從空殼升級為真實備份還原實現
//! 搜尋備份路徑下的 .inf 檔案並重新安裝

use std::path::Path;
use std::process::Command;

/// 從備份路徑還原驅動
///
/// 掃描 backup_path 下所有 .inf 檔案，用 pnputil /add-driver 重新安裝
pub fn restore_drivers(backup_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("從 {} 還原驅動", backup_path);

    let path = Path::new(backup_path);
    if !path.exists() {
        return Err(format!("Backup path not found: {}", backup_path).into());
    }

    // 搜尋所有 .inf 檔案
    let mut inf_count = 0;
    let mut fail_count = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) == Some("inf") {
            let inf_path = file_path.to_string_lossy().to_string();
            log::info!("還原驅動: {}", inf_path);

            let output = Command::new("pnputil")
                .args(["/add-driver", &inf_path, "/install"])
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    inf_count += 1;
                    log::info!("還原成功: {}", inf_path);
                }
                Ok(o) => {
                    fail_count += 1;
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    log::warn!("還原失敗: {} — {}", inf_path, stderr);
                }
                Err(e) => {
                    fail_count += 1;
                    log::error!("無法執行 pnputil: {}", e);
                }
            }
        }
    }

    if inf_count == 0 && fail_count == 0 {
        log::warn!("備份路徑下未找到 .inf 檔案: {}", backup_path);
    }

    log::info!("還原完成: 成功 {} 個, 失敗 {} 個", inf_count, fail_count);

    if fail_count > 0 && inf_count == 0 {
        Err(format!("All driver restorations failed ({} total)", fail_count).into())
    } else {
        Ok(())
    }
}

/// 初始化還原模組
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("還原模組初始化完成");
    Ok(())
}
