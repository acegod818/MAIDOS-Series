//! Driver database — file-backed driver catalog
//!
//! Stores driver metadata in a simple TSV file and provides query methods
//! by device ID, manufacturer, and class.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Write};

/// A record describing a known driver package.
#[derive(Debug, Clone)]
pub struct DriverRecord {
    pub driver_id: String,
    pub name: String,
    pub version: String,
    pub release_date: String,
    pub manufacturer: String,
    pub device_ids: Vec<String>,
    pub os_versions: Vec<String>,
    pub architectures: Vec<String>,
    pub download_url: String,
    pub file_size: u64,
    pub checksum: String,
    pub compatibility_score: u8,
}

/// Database metadata.
#[derive(Debug, Clone)]
pub struct DatabaseMetadata {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub source: String,
    pub total_drivers: u32,
}

/// In-memory driver database backed by a TSV file.
pub struct DriverDatabase {
    drivers: HashMap<String, Vec<DriverRecord>>,
    metadata: DatabaseMetadata,
    file_path: Option<String>,
}

impl Default for DriverDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverDatabase {
    /// Create a new empty database.
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            metadata: DatabaseMetadata {
                version: "1.0.0".to_string(),
                last_updated: Utc::now(),
                source: "local".to_string(),
                total_drivers: 0,
            },
            file_path: None,
        }
    }

    /// Load the database from a TSV file.
    ///
    /// Format: driver_id \t name \t version \t manufacturer \t device_ids(;sep) \t download_url \t checksum \t score
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Loading driver database from {}", path);

        let file = fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        self.drivers.clear();
        let mut count: u32 = 0;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 8 {
                continue;
            }

            let record = DriverRecord {
                driver_id: cols[0].to_string(),
                name: cols[1].to_string(),
                version: cols[2].to_string(),
                release_date: String::new(),
                manufacturer: cols[3].to_string(),
                device_ids: cols[4].split(';').map(|s| s.to_string()).collect(),
                os_versions: vec!["Windows 10".to_string(), "Windows 11".to_string()],
                architectures: vec!["x64".to_string()],
                download_url: cols[5].to_string(),
                file_size: 0,
                checksum: cols[6].to_string(),
                compatibility_score: cols[7].parse().unwrap_or(50),
            };

            // Index by each device ID
            for dev_id in &record.device_ids {
                self.drivers
                    .entry(dev_id.to_uppercase())
                    .or_default()
                    .push(record.clone());
            }
            count += 1;
        }

        self.metadata.total_drivers = count;
        self.metadata.last_updated = Utc::now();
        self.file_path = Some(path.to_string());

        log::info!("Loaded {} driver records", count);
        Ok(())
    }

    /// Save the database to a TSV file.
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Saving driver database to {}", path);

        let mut file = fs::File::create(path)?;
        writeln!(
            file,
            "# MAIDOS Driver Database v{} — {}",
            self.metadata.version, self.metadata.last_updated
        )?;
        writeln!(
            file,
            "# driver_id\tname\tversion\tmanufacturer\tdevice_ids\tdownload_url\tchecksum\tscore"
        )?;

        let mut seen = std::collections::HashSet::new();
        for records in self.drivers.values() {
            for rec in records {
                if seen.insert(&rec.driver_id) {
                    writeln!(
                        file,
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        rec.driver_id,
                        rec.name,
                        rec.version,
                        rec.manufacturer,
                        rec.device_ids.join(";"),
                        rec.download_url,
                        rec.checksum,
                        rec.compatibility_score,
                    )?;
                }
            }
        }

        log::info!("Database saved");
        Ok(())
    }

    /// Insert a driver record into the database.
    pub fn insert(&mut self, record: DriverRecord) {
        for dev_id in &record.device_ids {
            self.drivers
                .entry(dev_id.to_uppercase())
                .or_default()
                .push(record.clone());
        }
        self.metadata.total_drivers += 1;
        self.metadata.last_updated = Utc::now();
    }

    /// Query drivers by hardware device ID.
    pub fn query_by_device_id(&self, device_id: &str) -> Vec<&DriverRecord> {
        self.drivers
            .get(&device_id.to_uppercase())
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Query drivers by manufacturer name (case-insensitive substring match).
    pub fn query_by_manufacturer(&self, manufacturer: &str) -> Vec<&DriverRecord> {
        let needle = manufacturer.to_lowercase();
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for records in self.drivers.values() {
            for rec in records {
                if rec.manufacturer.to_lowercase().contains(&needle) && seen.insert(&rec.driver_id)
                {
                    results.push(rec);
                }
            }
        }
        results
    }

    /// Query drivers by device class (case-insensitive substring).
    pub fn query_by_class(&self, class: &str) -> Vec<&DriverRecord> {
        let needle = class.to_lowercase();
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for records in self.drivers.values() {
            for rec in records {
                if rec.name.to_lowercase().contains(&needle) && seen.insert(&rec.driver_id) {
                    results.push(rec);
                }
            }
        }
        results
    }

    /// Return database metadata.
    pub fn metadata(&self) -> &DatabaseMetadata {
        &self.metadata
    }

    /// Import drivers from the live system scan into the database.
    pub fn import_from_system(&mut self) -> Result<u32, Box<dyn std::error::Error>> {
        use crate::core::detect::hardware::scan_all_devices;

        let devices = scan_all_devices()?;
        let mut count = 0u32;

        for dev in &devices {
            let record = DriverRecord {
                driver_id: format!("SYS_{:08X}", fxhash(&dev.id)),
                name: dev.name.clone(),
                version: dev.version.clone(),
                release_date: Utc::now().format("%Y-%m-%d").to_string(),
                manufacturer: dev.vendor.clone(),
                device_ids: vec![dev.id.clone()],
                os_versions: vec!["Windows 10".to_string(), "Windows 11".to_string()],
                architectures: vec!["x64".to_string()],
                download_url: String::new(),
                file_size: 0,
                checksum: String::new(),
                compatibility_score: 100,
            };
            self.insert(record);
            count += 1;
        }

        log::info!("Imported {} devices from system scan", count);
        Ok(count)
    }
}

/// Simple FNV-like hash for generating IDs without external deps.
fn fxhash(s: &str) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for b in s.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut db = DriverDatabase::new();
        db.insert(DriverRecord {
            driver_id: "DRV001".into(),
            name: "Test GPU Driver".into(),
            version: "1.0.0".into(),
            release_date: "2026-01-01".into(),
            manufacturer: "NVIDIA".into(),
            device_ids: vec!["PCI\\VEN_10DE&DEV_1234".into()],
            os_versions: vec!["Windows 11".into()],
            architectures: vec!["x64".into()],
            download_url: "https://example.com/driver.exe".into(),
            file_size: 50_000_000,
            checksum: "abc123".into(),
            compatibility_score: 95,
        });

        let results = db.query_by_device_id("PCI\\VEN_10DE&DEV_1234");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test GPU Driver");

        let by_mfg = db.query_by_manufacturer("nvidia");
        assert_eq!(by_mfg.len(), 1);
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("maidos_db_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.tsv");
        let path_str = path.to_string_lossy().to_string();

        let mut db = DriverDatabase::new();
        db.insert(DriverRecord {
            driver_id: "DRV002".into(),
            name: "Audio".into(),
            version: "2.0.0".into(),
            release_date: String::new(),
            manufacturer: "Realtek".into(),
            device_ids: vec!["HDAUDIO\\VEN_1234".into()],
            os_versions: vec![],
            architectures: vec![],
            download_url: String::new(),
            file_size: 0,
            checksum: String::new(),
            compatibility_score: 80,
        });

        db.save_to_file(&path_str).expect("save should work");

        let mut db2 = DriverDatabase::new();
        db2.load_from_file(&path_str).expect("load should work");

        let results = db2.query_by_device_id("HDAUDIO\\VEN_1234");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manufacturer, "Realtek");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
