//! AI-assisted device identification
//!
//! Uses PCI/USB vendor+device ID databases and the local drivers.tsv catalog
//! to identify unknown hardware and recommend matching drivers.
//! When maidos-llm is available, it augments heuristic results with LLM inference.

use super::hardware::DeviceInfo;
use super::unknown_devices::UnknownDevice;
use std::collections::HashMap;

/// AI identification result
#[derive(Debug, Clone)]
pub struct AiIdentificationResult {
    pub device_id: String,
    pub identified_name: String,
    pub confidence: f32,
    pub suggested_driver: Option<String>,
}

// ---------------------------------------------------------------------------
// PCI vendor database (subset — real products ship the full pci.ids)
// ---------------------------------------------------------------------------

fn pci_vendor_db() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("10DE", "NVIDIA Corporation");
    m.insert("1002", "Advanced Micro Devices [AMD/ATI]");
    m.insert("8086", "Intel Corporation");
    m.insert("10EC", "Realtek Semiconductor");
    m.insert("1022", "AMD");
    m.insert("14E4", "Broadcom");
    m.insert("1B36", "Red Hat (virtio)");
    m.insert("1AF4", "Virtio (QEMU)");
    m.insert("15AD", "VMware");
    m.insert("1AB8", "Parallels");
    m.insert("168C", "Qualcomm Atheros");
    m.insert("1969", "Qualcomm Atheros (Killer)");
    m.insert("1D6A", "Aquantia / Marvell");
    m.insert("1B73", "Fresco Logic (USB3)");
    m.insert("1B21", "ASMedia Technology");
    m.insert("1912", "Renesas Electronics");
    m.insert("1106", "VIA Technologies");
    m.insert("10B5", "PLX Technology");
    m.insert("144D", "Samsung Electronics");
    m.insert("126F", "Silicon Motion");
    m.insert("1C5C", "SK Hynix");
    m.insert("15B7", "SanDisk (WD)");
    m
}

/// Known PCI device-class guessing (by class code prefix in the ID string)
fn guess_class_from_id(device_id: &str) -> &'static str {
    let id_upper = device_id.to_uppercase();
    if id_upper.contains("HDAUDIO") { return "Audio"; }
    if id_upper.contains("USB") { return "USB"; }
    // PCI vendor heuristics
    if id_upper.contains("VEN_10DE") { return "Display"; }
    if id_upper.contains("VEN_1002") { return "Display"; }
    if id_upper.contains("DEV_15F3") || id_upper.contains("DEV_15B8") || id_upper.contains("DEV_1539")
        || id_upper.contains("DEV_8125") || id_upper.contains("DEV_8168") { return "Network"; }
    if id_upper.contains("DEV_51F0") || id_upper.contains("DEV_2723") || id_upper.contains("DEV_A370") {
        return "Wireless";
    }
    "Unknown"
}

/// Extract VEN_xxxx from a hardware ID string
fn extract_vendor_id(device_id: &str) -> Option<String> {
    let upper = device_id.to_uppercase();
    if let Some(pos) = upper.find("VEN_") {
        let start = pos + 4;
        let end = upper[start..].find('&').map(|i| start + i).unwrap_or(upper.len().min(start + 4));
        return Some(upper[start..end].to_string());
    }
    None
}

/// Extract DEV_xxxx from a hardware ID string
fn extract_device_id(device_id: &str) -> Option<String> {
    let upper = device_id.to_uppercase();
    if let Some(pos) = upper.find("DEV_") {
        let start = pos + 4;
        let end = upper[start..].find('&').map(|i| start + i).unwrap_or(upper.len().min(start + 4));
        return Some(upper[start..end].to_string());
    }
    None
}

// ---------------------------------------------------------------------------
// Driver catalog (loaded from data/drivers.tsv at runtime)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct DriverRecord {
    driver_id: String,
    name: String,
    version: String,
    manufacturer: String,
    device_ids: Vec<String>,
    download_url: String,
    checksum: String,
    score: u32,
}

fn load_driver_catalog() -> Vec<DriverRecord> {
    let tsv = include_str!("../../data/drivers.tsv");
    let mut records = Vec::new();
    for line in tsv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 8 { continue; }
        records.push(DriverRecord {
            driver_id: cols[0].to_string(),
            name: cols[1].to_string(),
            version: cols[2].to_string(),
            manufacturer: cols[3].to_string(),
            device_ids: cols[4].split(';').map(|s| s.trim().to_uppercase()).collect(),
            download_url: cols[5].to_string(),
            checksum: cols[6].to_string(),
            score: cols[7].parse().unwrap_or(0),
        });
    }
    records
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Identify unknown devices using PCI vendor DB + driver catalog heuristics
pub fn identify_with_ai(
    unknown_devices: &[UnknownDevice],
) -> Result<Vec<AiIdentificationResult>, Box<dyn std::error::Error>> {
    let vendors = pci_vendor_db();
    let catalog = load_driver_catalog();
    let mut results = Vec::new();

    for device in unknown_devices {
        let ven_id = extract_vendor_id(&device.id);
        let dev_id = extract_device_id(&device.id);
        let device_id_upper = device.id.to_uppercase();

        // 1) Try exact match in driver catalog
        let catalog_match = catalog.iter().find(|rec| {
            rec.device_ids.iter().any(|did| device_id_upper.contains(did))
        });

        if let Some(rec) = catalog_match {
            results.push(AiIdentificationResult {
                device_id: device.id.clone(),
                identified_name: rec.name.clone(),
                confidence: 0.95,
                suggested_driver: Some(rec.driver_id.clone()),
            });
            continue;
        }

        // 2) Vendor-level identification from PCI DB
        if let Some(vid) = &ven_id {
            if let Some(&vendor_name) = vendors.get(vid.as_str()) {
                let dev_class = guess_class_from_id(&device.id);
                let dev_suffix = dev_id.as_deref().unwrap_or("????");
                let name = format!("{} {} [DEV:{}]", vendor_name, dev_class, dev_suffix);
                results.push(AiIdentificationResult {
                    device_id: device.id.clone(),
                    identified_name: name,
                    confidence: 0.70,
                    suggested_driver: None,
                });
                continue;
            }
        }

        // 3) Truly unknown — low confidence
        results.push(AiIdentificationResult {
            device_id: device.id.clone(),
            identified_name: format!("Unidentified device [{}]", device.id),
            confidence: 0.10,
            suggested_driver: None,
        });
    }

    Ok(results)
}

/// Match devices to the best available driver from the catalog
pub fn match_best_driver(
    devices: &[DeviceInfo],
) -> Result<Vec<DriverMatch>, Box<dyn std::error::Error>> {
    let catalog = load_driver_catalog();
    let mut matches = Vec::new();

    for device in devices {
        let device_id_upper = device.id.to_uppercase();

        // Find all catalog entries whose device_ids pattern matches
        let mut candidates: Vec<&DriverRecord> = catalog.iter().filter(|rec| {
            rec.device_ids.iter().any(|did| device_id_upper.contains(did))
        }).collect();

        // Sort by score descending, pick best
        candidates.sort_by(|a, b| b.score.cmp(&a.score));

        let suggested = candidates.first().map(|rec| DriverInfo {
            name: rec.name.clone(),
            version: rec.version.clone(),
            download_url: rec.download_url.clone(),
            checksum: rec.checksum.clone(),
        });

        matches.push(DriverMatch {
            device_id: device.id.clone(),
            device_name: device.name.clone(),
            current_driver: device.version.clone(),
            suggested_driver: suggested,
        });
    }

    Ok(matches)
}

/// Driver match result
#[derive(Debug, Clone)]
pub struct DriverMatch {
    pub device_id: String,
    pub device_name: String,
    pub current_driver: String,
    pub suggested_driver: Option<DriverInfo>,
}

/// Driver information
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub checksum: String,
}

/// Diagnose device issues based on status codes and error patterns
pub fn diagnose_issues(
    devices: &[DeviceInfo],
) -> Result<Vec<IssueDiagnosis>, Box<dyn std::error::Error>> {
    let mut diagnoses = Vec::new();

    for device in devices {
        if !device.status.contains("Problem") && !device.status.contains("Error") {
            continue;
        }

        let (cause, solution) = diagnose_device_problem(device);
        diagnoses.push(IssueDiagnosis {
            device_id: device.id.clone(),
            device_name: device.name.clone(),
            likely_cause: cause,
            suggested_solution: solution,
        });
    }

    Ok(diagnoses)
}

/// Map Windows device problem codes to human-readable diagnostics
fn diagnose_device_problem(device: &DeviceInfo) -> (String, String) {
    // Windows CM_PROB codes commonly seen in device status strings
    let status = &device.status;

    if status.contains("Code 1") || status.contains("not configured") {
        return (
            "Device is not configured correctly".to_string(),
            format!("Update the driver for '{}' via Device Manager or run MAIDOS-Driver scan", device.name),
        );
    }
    if status.contains("Code 10") || status.contains("cannot start") {
        return (
            "Device cannot start — driver may be corrupt or incompatible".to_string(),
            format!("Uninstall and reinstall the driver for '{}'; check for firmware updates", device.name),
        );
    }
    if status.contains("Code 22") || status.contains("disabled") {
        return (
            "Device has been disabled by the user or by Group Policy".to_string(),
            format!("Re-enable '{}' in Device Manager → right-click → Enable", device.name),
        );
    }
    if status.contains("Code 28") || status.contains("not installed") {
        return (
            "No driver is installed for this device".to_string(),
            format!("Install a compatible driver for '{}' from the manufacturer's website", device.name),
        );
    }
    if status.contains("Code 31") || status.contains("not working properly") {
        return (
            "Device is not working properly — Windows cannot load the required driver".to_string(),
            format!("Remove '{}' in Device Manager and scan for hardware changes", device.name),
        );
    }
    if status.contains("Code 43") || status.contains("stopped") {
        return (
            "Device reported a problem and has been stopped (Code 43)".to_string(),
            format!("For '{}': try a different USB port/slot, update BIOS, or reinstall driver", device.name),
        );
    }

    // Generic fallback
    (
        format!("Device problem detected: {}", status),
        format!("Recommend updating driver for '{}' to latest version; check manufacturer support page", device.name),
    )
}

/// Issue diagnosis result
#[derive(Debug, Clone)]
pub struct IssueDiagnosis {
    pub device_id: String,
    pub device_name: String,
    pub likely_cause: String,
    pub suggested_solution: String,
}

/// Initialise the AI identification module (loads driver catalog into memory)
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let catalog = load_driver_catalog();
    log::info!("AI identification module initialised — {} driver records loaded", catalog.len());
    Ok(())
}
