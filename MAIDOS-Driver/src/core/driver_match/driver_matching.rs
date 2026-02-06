//! Driver matching engine
//!
//! Given a hardware device ID, scores and ranks candidate driver packages
//! from the database by compatibility, version, and manufacturer trust.

use crate::database::driver_database::{DriverDatabase, DriverRecord};

/// A match result with relevance score.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub driver_id: String,
    pub driver_name: String,
    pub version: String,
    pub manufacturer: String,
    pub score: u32,
    pub match_type: MatchType,
}

/// How the match was determined.
#[derive(Debug, Clone)]
pub enum MatchType {
    /// Exact hardware ID match
    ExactId,
    /// Vendor+device match (ignoring subsystem/revision)
    VendorDevice,
    /// Vendor-only match
    VendorOnly,
    /// Compatible ID or class-based match
    ClassBased,
}

/// Driver matcher that queries the database and ranks results.
pub struct DriverMatcher<'a> {
    db: &'a DriverDatabase,
}

impl<'a> DriverMatcher<'a> {
    pub fn new(db: &'a DriverDatabase) -> Self {
        Self { db }
    }

    /// Find the best matching driver for a device.
    pub fn find_best_driver(&self, device_id: &str) -> Option<MatchResult> {
        let mut candidates = self.get_all_matches(device_id);
        candidates.sort_by(|a, b| b.score.cmp(&a.score));
        candidates.into_iter().next()
    }

    /// Get all matching drivers, scored and ranked.
    pub fn get_all_matches(&self, device_id: &str) -> Vec<MatchResult> {
        let mut results = Vec::new();

        // 1. Exact ID match
        let exact = self.db.query_by_device_id(device_id);
        for rec in &exact {
            results.push(score_match(rec, MatchType::ExactId));
        }

        // 2. Vendor+Device match (strip subsystem & revision)
        if let Some(short_id) = extract_vendor_device(device_id) {
            let candidates = self.db.query_by_device_id(&short_id);
            for rec in candidates {
                if !results.iter().any(|r| r.driver_id == rec.driver_id) {
                    results.push(score_match(rec, MatchType::VendorDevice));
                }
            }
        }

        // 3. Vendor-only match
        if let Some(vendor) = extract_vendor(device_id) {
            let candidates = self.db.query_by_manufacturer(&vendor);
            for rec in candidates {
                if !results.iter().any(|r| r.driver_id == rec.driver_id) {
                    results.push(score_match(rec, MatchType::VendorOnly));
                }
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }

    /// Get compatible driver versions for a device, sorted newest first.
    pub fn get_compatible_versions(&self, device_id: &str) -> Vec<MatchResult> {
        let mut matches = self.get_all_matches(device_id);
        // Sort by version descending (lexicographic, works for semver)
        matches.sort_by(|a, b| b.version.cmp(&a.version));
        matches
    }
}

/// Score a driver record based on match type and metadata.
fn score_match(rec: &DriverRecord, match_type: MatchType) -> MatchResult {
    let base_score: u32 = match &match_type {
        MatchType::ExactId => 1000,
        MatchType::VendorDevice => 800,
        MatchType::VendorOnly => 400,
        MatchType::ClassBased => 200,
    };

    // Bonus for compatibility score from database
    let compat_bonus = rec.compatibility_score as u32 * 2;

    // Bonus for having a version string (indicates real driver info)
    let version_bonus = if rec.version.contains('.') { 50 } else { 0 };

    MatchResult {
        driver_id: rec.driver_id.clone(),
        driver_name: rec.name.clone(),
        version: rec.version.clone(),
        manufacturer: rec.manufacturer.clone(),
        score: base_score + compat_bonus + version_bonus,
        match_type,
    }
}

/// Extract VEN_xxxx&DEV_xxxx portion from a PnP hardware ID.
///
/// Input:  `PCI\VEN_8086&DEV_6F08&SUBSYS_00008086&REV_01`
/// Output: `PCI\VEN_8086&DEV_6F08`
fn extract_vendor_device(hw_id: &str) -> Option<String> {
    let upper = hw_id.to_uppercase();
    if let Some(ven_pos) = upper.find("VEN_") {
        if let Some(dev_end) = upper[ven_pos..].find("&SUBSYS") {
            let prefix_end = hw_id.find('\\').map(|p| p + 1).unwrap_or(0);
            let prefix = &hw_id[..prefix_end];
            return Some(format!("{}{}", prefix, &hw_id[ven_pos..ven_pos + dev_end]));
        }
    }
    None
}

/// Extract vendor name from hardware ID.
///
/// Looks for VEN_xxxx and maps well-known vendor IDs.
fn extract_vendor(hw_id: &str) -> Option<String> {
    let upper = hw_id.to_uppercase();
    if let Some(pos) = upper.find("VEN_") {
        let ven_start = pos + 4;
        let ven_end = upper[ven_start..]
            .find('&')
            .map(|p| ven_start + p)
            .unwrap_or(upper.len());
        let ven_id = &upper[ven_start..ven_end];

        return Some(
            match ven_id {
                "8086" => "Intel",
                "10DE" => "NVIDIA",
                "1002" => "AMD",
                "1022" => "AMD",
                "14E4" => "Broadcom",
                "168C" => "Qualcomm",
                "10EC" => "Realtek",
                "1B21" => "ASMedia",
                "1D6B" => "Linux Foundation",
                _ => return Some(format!("VEN_{}", ven_id)),
            }
            .to_string(),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::driver_database::DriverRecord;

    fn make_test_db() -> DriverDatabase {
        let mut db = DriverDatabase::new();
        db.insert(DriverRecord {
            driver_id: "DRV_INTEL_NET".into(),
            name: "Intel I219-LM Network".into(),
            version: "12.19.1.37".into(),
            release_date: "2025-06-01".into(),
            manufacturer: "Intel".into(),
            device_ids: vec!["PCI\\VEN_8086&DEV_15B8".into()],
            os_versions: vec!["Windows 11".into()],
            architectures: vec!["x64".into()],
            download_url: "https://downloadcenter.intel.com/example".into(),
            file_size: 0,
            checksum: String::new(),
            compatibility_score: 95,
        });
        db
    }

    #[test]
    fn test_exact_match() {
        let db = make_test_db();
        let matcher = DriverMatcher::new(&db);
        let best = matcher.find_best_driver("PCI\\VEN_8086&DEV_15B8");
        assert!(best.is_some());
        assert_eq!(best.unwrap().driver_name, "Intel I219-LM Network");
    }

    #[test]
    fn test_vendor_extraction() {
        assert_eq!(
            extract_vendor("PCI\\VEN_8086&DEV_6F08&SUBSYS_00008086&REV_01"),
            Some("Intel".to_string())
        );
        assert_eq!(
            extract_vendor("PCI\\VEN_10DE&DEV_1234"),
            Some("NVIDIA".to_string())
        );
    }

    #[test]
    fn test_no_match() {
        let db = make_test_db();
        let matcher = DriverMatcher::new(&db);
        let best = matcher.find_best_driver("USB\\VID_DEAD&PID_BEEF");
        assert!(best.is_none());
    }
}
