//! Driver recommender — recommends drivers based on system context
//!
//! Combines hardware identification, database lookup, and scoring
//! to recommend the best driver for a device.

use crate::ai::hardware_identifier::{HardwareIdentifier, UnknownDeviceInfo};
use crate::core::detect::hardware::{scan_all_devices, DeviceInfo};
use crate::database::driver_database::DriverDatabase;

/// A driver recommendation.
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub device_id: String,
    pub device_name: String,
    pub recommended_action: RecommendedAction,
    pub confidence: f32,
    pub reasoning: String,
}

/// What action is recommended for the device.
#[derive(Debug, Clone)]
pub enum RecommendedAction {
    /// Driver is up to date, no action needed.
    UpToDate,
    /// A newer driver version is available.
    UpdateAvailable { version: String },
    /// Device is unidentified; manual investigation needed.
    ManualInvestigation,
    /// Device has a problem; reinstall recommended.
    Reinstall,
}

/// User preferences for driver selection.
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub prefer_stable: bool,
    pub prefer_latest: bool,
    pub avoid_beta: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            prefer_stable: true,
            prefer_latest: false,
            avoid_beta: true,
        }
    }
}

/// Driver recommender engine.
pub struct DriverRecommender {
    identifier: HardwareIdentifier,
}

impl Default for DriverRecommender {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverRecommender {
    pub fn new() -> Self {
        Self {
            identifier: HardwareIdentifier::new(),
        }
    }

    /// Analyze a single device and produce a recommendation.
    pub fn recommend_for_device(
        &mut self,
        device: &DeviceInfo,
        db: &DriverDatabase,
    ) -> Recommendation {
        // Step 1: Identify the device
        let id_result = self.identifier.identify_device(&UnknownDeviceInfo {
            hardware_id: device.id.clone(),
            description: device.name.clone(),
            device_class: device.class.clone(),
            vendor_string: device.vendor.clone(),
        });

        // Step 2: Check if device has a problem
        if device.status.contains("Problem") {
            return Recommendation {
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                recommended_action: RecommendedAction::Reinstall,
                confidence: 0.9,
                reasoning: format!(
                    "Device reports problem status: {}. Reinstalling the driver may fix it.",
                    device.status
                ),
            };
        }

        // Step 3: Look up in database for newer versions
        let db_matches = db.query_by_device_id(&device.id);
        if let Some(best) = db_matches.first() {
            if best.version != device.version && version_is_newer(&best.version, &device.version) {
                return Recommendation {
                    device_id: device.id.clone(),
                    device_name: device.name.clone(),
                    recommended_action: RecommendedAction::UpdateAvailable {
                        version: best.version.clone(),
                    },
                    confidence: id_result.confidence * 0.9,
                    reasoning: format!(
                        "Database has newer version {} (current: {}), from {}",
                        best.version, device.version, best.manufacturer
                    ),
                };
            }
        }

        // Step 4: If we couldn't identify the device well, flag for manual investigation
        if id_result.confidence < 0.3 {
            return Recommendation {
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                recommended_action: RecommendedAction::ManualInvestigation,
                confidence: id_result.confidence,
                reasoning: format!(
                    "Low identification confidence ({:.0}%). Device may need manual driver lookup.",
                    id_result.confidence * 100.0
                ),
            };
        }

        // Step 5: Device appears up to date
        Recommendation {
            device_id: device.id.clone(),
            device_name: device.name.clone(),
            recommended_action: RecommendedAction::UpToDate,
            confidence: id_result.confidence,
            reasoning: format!(
                "Identified as {} by {} (confidence {:.0}%). Current driver appears up to date.",
                id_result.device_name,
                id_result.manufacturer,
                id_result.confidence * 100.0
            ),
        }
    }

    /// Analyze all system devices and produce recommendations.
    pub fn recommend_all(
        &mut self,
        db: &DriverDatabase,
    ) -> Result<Vec<Recommendation>, Box<dyn std::error::Error>> {
        let devices = scan_all_devices()?;
        let mut recommendations = Vec::new();

        for device in &devices {
            recommendations.push(self.recommend_for_device(device, db));
        }

        Ok(recommendations)
    }
}

/// Simple version comparison: check if `a` is newer than `b` (semver-like).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(version_is_newer("2.0.0", "1.0.0"));
        assert!(version_is_newer("1.1.0", "1.0.0"));
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_recommend_for_device() {
        let mut recommender = DriverRecommender::new();
        let db = DriverDatabase::new();

        let device = DeviceInfo::new(
            "PCI\\VEN_8086&DEV_15B8".to_string(),
            "Intel Ethernet I219-LM".to_string(),
            "Intel".to_string(),
            "12.19.1.37".to_string(),
            "OK".to_string(),
            "Net".to_string(),
        );

        let rec = recommender.recommend_for_device(&device, &db);
        assert!(!rec.reasoning.is_empty());
    }
}
