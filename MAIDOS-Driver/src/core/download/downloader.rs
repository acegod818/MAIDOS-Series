//! Driver downloader
//!
//! Uses PowerShell Start-BitsTransfer for reliable background downloading
//! with automatic retry, and validates against an official source whitelist.

use std::path::Path;
use std::process::Command;

/// Official driver source domains (HTTPS only).
const TRUSTED_DOMAINS: &[&str] = &[
    "download.microsoft.com",
    "downloadcenter.intel.com",
    "www.intel.com",
    "us.download.nvidia.com",
    "download.nvidia.com",
    "drivers.amd.com",
    "www.amd.com",
    "dlcdnets.asus.com",
    "www.realtek.com",
    "global.download.synology.com",
    "ftp.hp.com",
    "downloads.dell.com",
    "download.lenovo.com",
];

/// Download result.
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub file_path: String,
    pub file_size: u64,
    pub source_url: String,
    pub is_trusted_source: bool,
}

/// Download a driver package from a URL.
///
/// Uses BITS transfer for reliable background downloading.
/// Validates the source domain against the trusted whitelist.
pub fn download_driver(
    url: &str,
    save_dir: &str,
    filename: &str,
) -> Result<DownloadResult, Box<dyn std::error::Error>> {
    log::info!("Downloading driver: {} -> {}/{}", url, save_dir, filename);

    // Validate URL scheme
    if !url.starts_with("https://") {
        return Err("Only HTTPS URLs are allowed for driver downloads".into());
    }

    let is_trusted = is_trusted_source(url);
    if !is_trusted {
        log::warn!("URL is not from a trusted source: {}", url);
    }

    // Ensure save directory exists
    std::fs::create_dir_all(save_dir)?;

    let save_path = Path::new(save_dir).join(filename);
    let save_path_str = save_path.to_string_lossy().to_string();

    // Use BITS transfer via PowerShell
    let ps_script = format!(
        "Start-BitsTransfer -Source '{}' -Destination '{}' -ErrorAction Stop; (Get-Item '{}').Length",
        url, save_path_str, save_path_str
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Fallback to Invoke-WebRequest if BITS fails (e.g., BITS service not running)
        log::warn!(
            "BITS transfer failed, falling back to Invoke-WebRequest: {}",
            stderr
        );
        return download_fallback(url, &save_path_str, is_trusted);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let file_size: u64 = stdout.trim().parse().unwrap_or(0);

    if file_size == 0 {
        return Err("Downloaded file is empty".into());
    }

    log::info!(
        "Download complete: {} ({} bytes, trusted={})",
        save_path_str,
        file_size,
        is_trusted
    );

    Ok(DownloadResult {
        file_path: save_path_str,
        file_size,
        source_url: url.to_string(),
        is_trusted_source: is_trusted,
    })
}

/// Fallback download using Invoke-WebRequest.
fn download_fallback(
    url: &str,
    save_path: &str,
    is_trusted: bool,
) -> Result<DownloadResult, Box<dyn std::error::Error>> {
    let ps_script = format!(
        "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing; (Get-Item '{}').Length",
        url, save_path, save_path
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Download failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let file_size: u64 = stdout.trim().parse().unwrap_or(0);

    Ok(DownloadResult {
        file_path: save_path.to_string(),
        file_size,
        source_url: url.to_string(),
        is_trusted_source: is_trusted,
    })
}

/// Verify SHA256 checksum of a downloaded file.
pub fn verify_checksum(
    file_path: &str,
    expected_sha256: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("Verifying checksum for {}", file_path);

    let ps_script = format!("(Get-FileHash '{}' -Algorithm SHA256).Hash", file_path);

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()?;

    if !output.status.success() {
        return Err("Failed to compute file hash".into());
    }

    let actual = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_uppercase();
    let expected = expected_sha256.to_uppercase();
    let matches = actual == expected;

    if !matches {
        log::warn!(
            "Checksum mismatch for {}: expected={}, actual={}",
            file_path,
            expected,
            actual
        );
    }

    Ok(matches)
}

/// Check if a URL is from a trusted official driver source (HTTPS only).
pub fn is_trusted_source(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.starts_with("https://") && TRUSTED_DOMAINS.iter().any(|domain| lower.contains(domain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trusted_source_check() {
        assert!(is_trusted_source(
            "https://download.microsoft.com/driver.exe"
        ));
        assert!(is_trusted_source(
            "https://us.download.nvidia.com/gpu/driver.exe"
        ));
        assert!(!is_trusted_source("https://shadysite.com/driver.exe"));
        assert!(!is_trusted_source(
            "http://download.microsoft.com/driver.exe"
        ));
    }

    #[test]
    fn test_https_only() {
        let result = download_driver("http://example.com/driver.exe", "C:\\temp", "test.exe");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTPS"));
    }
}
