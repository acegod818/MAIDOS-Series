//! Audit logging module
//!
//! Writes all driver operations to an audit log file:
//! `[MAIDOS-AUDIT] {timestamp} {operation} {device} {result}`
//!
//! Log location: %ProgramData%\MAIDOS\DriverManager\audit.log

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

/// Global audit log path
static AUDIT_LOG_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Initialize the audit log, ensuring the directory exists.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = r"C:\ProgramData\MAIDOS\DriverManager";
    fs::create_dir_all(log_dir)?;

    let path = format!(r"{}\audit.log", log_dir);
    if let Ok(mut guard) = AUDIT_LOG_PATH.lock() {
        *guard = Some(path.clone());
    }

    log::info!("Audit log initialized: {}", path);
    Ok(())
}

/// Get (or lazily initialize) the audit log path.
fn get_log_path() -> String {
    if let Ok(guard) = AUDIT_LOG_PATH.lock() {
        if let Some(ref p) = *guard {
            return p.clone();
        }
    }
    // Lazy init
    let log_dir = r"C:\ProgramData\MAIDOS\DriverManager";
    let _ = fs::create_dir_all(log_dir);
    let path = format!(r"{}\audit.log", log_dir);
    if let Ok(mut guard) = AUDIT_LOG_PATH.lock() {
        *guard = Some(path.clone());
    }
    path
}

/// Record an audit entry for a successful operation.
///
/// Format: `[MAIDOS-AUDIT] 2026-02-06 14:30:00 INSTALL PCI\VEN_10DE&DEV_2684 SUCCESS`
pub fn audit_success(operation: &str, device: &str, detail: &str) {
    let entry = format_entry(operation, device, "SUCCESS", detail);
    write_entry(&entry);
    log::info!("{}", entry);
}

/// Record an audit entry for a failed operation.
///
/// Format: `[MAIDOS-AUDIT] 2026-02-06 14:30:00 INSTALL PCI\VEN_10DE&DEV_2684 FAILED error_code: message`
pub fn audit_failure(operation: &str, device: &str, error: &str) {
    let entry = format_entry(operation, device, "FAILED", error);
    write_entry(&entry);
    log::error!("{}", entry);
}

/// Format an audit log entry.
fn format_entry(operation: &str, device: &str, result: &str, detail: &str) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    if detail.is_empty() {
        format!("[MAIDOS-AUDIT] {} {} {} {}", timestamp, operation, device, result)
    } else {
        format!(
            "[MAIDOS-AUDIT] {} {} {} {} {}",
            timestamp, operation, device, result, detail
        )
    }
}

/// Append an entry to the audit log file.
fn write_entry(entry: &str) {
    let path = get_log_path();
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| writeln!(file, "{}", entry));

    if let Err(e) = result {
        log::warn!("Failed to write audit log: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_entry() {
        let entry = format_entry("INSTALL", "PCI\\VEN_10DE", "SUCCESS", "driver.inf");
        assert!(entry.starts_with("[MAIDOS-AUDIT]"));
        assert!(entry.contains("INSTALL"));
        assert!(entry.contains("PCI\\VEN_10DE"));
        assert!(entry.contains("SUCCESS"));
        assert!(entry.contains("driver.inf"));
    }

    #[test]
    fn test_format_entry_no_detail() {
        let entry = format_entry("SCAN", "ALL", "SUCCESS", "");
        assert!(entry.contains("SCAN"));
        assert!(!entry.ends_with(' '));
    }

    #[test]
    fn test_audit_write() {
        // Should not panic even if directory doesn't exist
        audit_success("TEST", "UNIT_TEST", "test entry");
    }
}
