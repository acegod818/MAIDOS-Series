//! Driver backup manager
//!
//! Uses `pnputil /export-driver` to export all third-party OEM drivers
//! to a specified backup directory.

use std::process::Command;

/// Minimum required free space for backup: 500 MB
const MIN_FREE_SPACE_BYTES: u64 = 500 * 1024 * 1024;

/// A single backup entry representing one exported OEM driver package.
#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub id: String,
    pub timestamp: String,
    pub path: String,
    pub size: u64,
}

/// Check available disk space on the drive containing `path`.
///
/// Returns free bytes, or an error if the check fails.
fn check_disk_space(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    // Extract drive letter or use the path directly
    let drive = if path.len() >= 2 && path.as_bytes()[1] == b':' {
        &path[..2]
    } else {
        "C:"
    };

    let ps_script = format!(
        "(Get-PSDrive -Name '{}').Free",
        &drive[..1] // Just the letter, e.g., "C"
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().parse::<u64>().unwrap_or(0))
    } else {
        Err("Cannot determine free disk space".into())
    }
}

/// Export all third-party (OEM) drivers to `backup_path`.
///
/// Checks disk space first (AC-015), then calls
/// `pnputil /export-driver * <backup_path>`.
pub fn backup_drivers(backup_path: &str) -> Result<Vec<BackupEntry>, Box<dyn std::error::Error>> {
    log::info!("Backing up OEM drivers to {}", backup_path);

    // AC-015: Check disk space before backup
    match check_disk_space(backup_path) {
        Ok(free) => {
            log::info!("Free disk space: {} MB", free / (1024 * 1024));
            if free < MIN_FREE_SPACE_BYTES {
                let msg = format!(
                    "Insufficient disk space: {} MB free, {} MB required",
                    free / (1024 * 1024),
                    MIN_FREE_SPACE_BYTES / (1024 * 1024)
                );
                crate::core::audit::audit_failure("BACKUP", "ALL", &msg);
                return Err(msg.into());
            }
        }
        Err(e) => {
            log::warn!("Disk space check failed (proceeding anyway): {}", e);
        }
    }

    // Ensure the backup directory exists
    std::fs::create_dir_all(backup_path)?;

    let output = Command::new("pnputil")
        .args(["/export-driver", "*", backup_path])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    log::info!("pnputil export stdout: {}", stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pnputil /export-driver failed: {}", stderr).into());
    }

    // Enumerate exported .inf files to build the entry list
    let mut entries = Vec::new();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for entry in std::fs::read_dir(backup_path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) == Some("inf") {
            let name = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            entries.push(BackupEntry {
                id: name,
                timestamp: timestamp.clone(),
                path: file_path.to_string_lossy().to_string(),
                size,
            });
        }
    }

    crate::core::audit::audit_success(
        "BACKUP",
        "ALL",
        &format!("{} drivers exported to {}", entries.len(), backup_path),
    );
    Ok(entries)
}

/// Initialize backup module.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Backup module initialized");
    Ok(())
}
