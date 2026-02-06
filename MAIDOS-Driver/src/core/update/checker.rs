//! Driver update checker
//!
//! Checks for driver updates via DriverDatabase (official download URLs)
//! and Windows Update API, then downloads via BITS.

use crate::core::detect::hardware::scan_all_devices;
use crate::core::download::downloader;
use crate::database::driver_database::DriverDatabase;

/// Driver update information
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Device ID
    pub device_id: String,
    /// Currently installed version
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Whether an update is available
    pub update_available: bool,
    /// Status description
    pub status: String,
    /// Direct download URL from official source (empty if WU-only)
    pub download_url: String,
}

/// Check a single device for driver updates.
///
/// Queries DriverDatabase for official download URLs first,
/// then falls back to Windows Update API.
pub fn check_driver_update(
    device_id: &str,
    _update_server: Option<&str>,
) -> Result<UpdateInfo, Box<dyn std::error::Error>> {
    log::info!("Checking driver update for: {}", device_id);

    let devices = scan_all_devices()?;
    let device = devices.iter().find(|d| d.id == device_id);

    match device {
        Some(dev) => {
            let current = &dev.version;
            let db = load_driver_database();

            // 1. Check DriverDatabase for official download
            if let Some(ref database) = db {
                if let Some(rec) = find_best_db_match(database, &dev.id, current) {
                    return Ok(UpdateInfo {
                        device_id: device_id.to_string(),
                        current_version: current.clone(),
                        latest_version: rec.version.clone(),
                        update_available: true,
                        status: format!("Update {} from {}", rec.version, rec.manufacturer),
                        download_url: rec.download_url.clone(),
                    });
                }
            }

            // 2. Fallback to Windows Update
            let wu_updates = check_windows_update_available();
            let dev_name_lower = dev.name.to_lowercase();
            let matched_wu = wu_updates.iter().find(|(title, model)| {
                let t = title.to_lowercase();
                let m = model.to_lowercase();
                t.contains(&dev_name_lower) || m.contains(&dev_name_lower)
            });

            match matched_wu {
                Some((title, _)) => Ok(UpdateInfo {
                    device_id: device_id.to_string(),
                    current_version: current.clone(),
                    latest_version: title.clone(),
                    update_available: true,
                    status: format!("Windows Update: {}", title),
                    download_url: String::new(),
                }),
                None => Ok(UpdateInfo {
                    device_id: device_id.to_string(),
                    current_version: current.clone(),
                    latest_version: current.clone(),
                    update_available: false,
                    status: "Up to date".to_string(),
                    download_url: String::new(),
                }),
            }
        }
        None => Err(format!("Device not found: {}", device_id).into()),
    }
}

/// Batch check all devices for driver updates.
///
/// Queries DriverDatabase for official download URLs first,
/// then falls back to Windows Update API for remaining devices.
pub fn check_all_updates(
    _update_server: Option<&str>,
) -> Result<Vec<UpdateInfo>, Box<dyn std::error::Error>> {
    log::info!("Batch checking all device driver updates");

    let devices = scan_all_devices()?;
    let wu_updates = check_windows_update_available();
    let db = load_driver_database();

    log::info!(
        "Windows Update: {} pending updates, DriverDB loaded: {}",
        wu_updates.len(),
        db.is_some()
    );

    let mut updates = Vec::new();

    for device in &devices {
        let current = &device.version;
        let dev_name_lower = device.name.to_lowercase();

        // 1. Check DriverDatabase for official download URL
        let db_match = db
            .as_ref()
            .and_then(|database| find_best_db_match(database, &device.id, current));

        if let Some(rec) = db_match {
            updates.push(UpdateInfo {
                device_id: device.id.clone(),
                current_version: current.clone(),
                latest_version: rec.version.clone(),
                update_available: true,
                status: format!("Update {} from {}", rec.version, rec.manufacturer),
                download_url: rec.download_url.clone(),
            });
            continue;
        }

        // 2. Fallback: Check Windows Update
        let matched_wu = wu_updates.iter().find(|(title, model)| {
            let t = title.to_lowercase();
            let m = model.to_lowercase();
            t.contains(&dev_name_lower) || m.contains(&dev_name_lower)
        });

        let (update_available, latest, status, download_url) = match matched_wu {
            Some((title, _)) => (
                true,
                title.clone(),
                format!("Windows Update: {}", title),
                String::new(),
            ),
            None => (
                false,
                current.clone(),
                "Up to date".to_string(),
                String::new(),
            ),
        };

        updates.push(UpdateInfo {
            device_id: device.id.clone(),
            current_version: current.clone(),
            latest_version: latest,
            update_available,
            status,
            download_url,
        });
    }

    let available_count = updates.iter().filter(|u| u.update_available).count();
    let direct_dl = updates
        .iter()
        .filter(|u| !u.download_url.is_empty())
        .count();
    log::info!(
        "Check complete: {} devices, {} updates ({} with direct download)",
        updates.len(),
        available_count,
        direct_dl
    );

    Ok(updates)
}

// =====================================================================
// Internal helpers
// =====================================================================

/// Try to load the DriverDatabase from well-known paths.
fn load_driver_database() -> Option<DriverDatabase> {
    let mut db = DriverDatabase::new();

    let candidates: Vec<String> = [
        // Next to executable: data/drivers.tsv
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("data").join("drivers.tsv")))
            .map(|p| p.to_string_lossy().to_string()),
        // ProgramData
        Some(r"C:\ProgramData\MAIDOS\DriverManager\drivers.tsv".to_string()),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            if db.load_from_file(path).is_ok() {
                log::info!("Loaded driver database from {}", path);
                return Some(db);
            }
        }
    }

    log::info!("No driver database found, using Windows Update only");
    None
}

/// Find the best matching driver record in the database for a device.
///
/// Tries exact device ID match first, then VEN&DEV prefix match.
/// Returns the first record that has a non-empty download_url and
/// a version newer than the currently installed one.
fn find_best_db_match<'a>(
    db: &'a DriverDatabase,
    device_id: &str,
    current_version: &str,
) -> Option<&'a crate::database::driver_database::DriverRecord> {
    // Try exact match
    let mut records = db.query_by_device_id(device_id);

    // Try VEN&DEV prefix match if exact match fails
    if records.is_empty() {
        if let Some(prefix) = extract_ven_dev(device_id) {
            records = db.query_by_device_id(&prefix);
        }
    }

    records
        .into_iter()
        .find(|rec| !rec.download_url.is_empty() && version_is_newer(&rec.version, current_version))
}

/// Extract "PCI\VEN_XXXX&DEV_XXXX" prefix from a full device ID.
fn extract_ven_dev(device_id: &str) -> Option<String> {
    let upper = device_id.to_uppercase();
    let _ven_pos = upper.find("VEN_")?;
    let dev_pos = upper.find("DEV_")?;
    let dev_end = upper[dev_pos + 4..]
        .find('&')
        .map(|i| dev_pos + 4 + i)
        .unwrap_or(upper.len());
    Some(upper[..dev_end].to_string())
}

/// Simple semver-like comparison: is `a` newer than `b`?
fn version_is_newer(a: &str, b: &str) -> bool {
    let parse =
        |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };
    let va = parse(a);
    let vb = parse(b);
    for i in 0..va.len().max(vb.len()) {
        let pa = va.get(i).copied().unwrap_or(0);
        let pb = vb.get(i).copied().unwrap_or(0);
        if pa > pb {
            return true;
        }
        if pa < pb {
            return false;
        }
    }
    false
}

/// Query Windows Update API for pending driver updates.
fn check_windows_update_available() -> Vec<(String, String)> {
    // 使用 Get-WindowsDriver 列出第三方驅動版本日期
    // 與已安裝版本比較，檢測是否有 Windows Update 提供的更新
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
            try {
                $session = New-Object -ComObject Microsoft.Update.Session
                $searcher = $session.CreateUpdateSearcher()
                $result = $searcher.Search("IsInstalled=0 AND Type='Driver'")
                foreach ($update in $result.Updates) {
                    Write-Output "$($update.Title)|$($update.DriverModel)"
                }
            } catch {
                # Windows Update COM unavailable
            }
            "#,
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        Some((line.to_string(), String::new()))
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Download a driver update package.
///
/// Uses BITS transfer (with Invoke-WebRequest fallback) via the
/// download module. Validates HTTPS-only and trusted source whitelist.
pub fn download_update(
    download_url: &str,
    save_path: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    log::info!("Downloading driver update: {} -> {}", download_url, save_path);

    let save_dir = std::path::Path::new(save_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let filename = std::path::Path::new(save_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "driver_update.exe".to_string());

    let result = downloader::download_driver(download_url, &save_dir, &filename)?;

    crate::core::audit::audit_success(
        "DOWNLOAD",
        download_url,
        &format!("{} bytes, trusted={}", result.file_size, result.is_trusted_source),
    );
    Ok(result.file_size)
}

/// Apply a driver update using pnputil.
pub fn apply_update(
    inf_path: &str,
    device_id: Option<&str>,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!(
        "Applying driver update: {}, device: {}",
        inf_path,
        device_id.unwrap_or("all")
    );

    // 檢查 INF 文件是否存在
    if !std::path::Path::new(inf_path).exists() {
        return Err(format!("INF file not found: {}", inf_path).into());
    }

    // 使用 pnputil 安裝驅動 (Windows 內建工具)
    let output = std::process::Command::new("pnputil")
        .args(["/add-driver", inf_path, "/install"])
        .output()?;

    if output.status.success() {
        crate::core::audit::audit_success("APPLY_UPDATE", device_id.unwrap_or("all"), inf_path);
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let msg = format!("pnputil install failed: {} {}", stdout_str, stderr);
        crate::core::audit::audit_failure("APPLY_UPDATE", device_id.unwrap_or("all"), &msg);
        Err(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_newer() {
        assert!(version_is_newer("2.0.0", "1.0.0"));
        assert!(version_is_newer("1.1.0", "1.0.0"));
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_extract_ven_dev() {
        assert_eq!(
            extract_ven_dev("PCI\\VEN_10DE&DEV_1234&SUBSYS_5678&REV_01"),
            Some("PCI\\VEN_10DE&DEV_1234".to_string())
        );
        assert_eq!(
            extract_ven_dev("PCI\\VEN_8086&DEV_15B8"),
            Some("PCI\\VEN_8086&DEV_15B8".to_string())
        );
        assert_eq!(extract_ven_dev("USB\\SOMETHING_ELSE"), None);
    }

    #[test]
    fn test_check_driver_update_known_device() {
        let devices = scan_all_devices().expect("scan should succeed");
        assert!(!devices.is_empty(), "should have at least one device");
        let first_id = &devices[0].id;

        let result = check_driver_update(first_id, None);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.device_id, *first_id);
        assert!(!info.current_version.is_empty());
        assert!(!info.status.is_empty());
        // download_url is always a valid String (may be empty)
        let _ = &info.download_url;
    }

    #[test]
    fn test_check_driver_update_unknown_device() {
        let result = check_driver_update("NONEXISTENT999", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_all_updates() {
        let result = check_all_updates(None);
        assert!(result.is_ok());
        let updates = result.unwrap();
        assert!(!updates.is_empty());
        for u in &updates {
            assert!(!u.status.is_empty());
            assert!(!u.device_id.is_empty());
            // download_url is always a valid String (may be empty)
            let _ = &u.download_url;
        }
    }

    #[test]
    fn test_check_windows_update_api() {
        let updates = check_windows_update_available();
        eprintln!("Windows Update pending drivers: {}", updates.len());
        for (title, model) in &updates {
            eprintln!("  {} | {}", title, model);
        }
    }

    #[test]
    fn test_apply_update_missing_file() {
        let result = apply_update("nonexistent.inf", None);
        assert!(result.is_err());
    }
}
