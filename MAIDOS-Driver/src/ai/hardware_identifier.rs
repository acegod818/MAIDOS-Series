//! Hardware identifier — local heuristic-based device identification
//!
//! Parses PCI/USB vendor and device IDs from hardware ID strings
//! and maps them to known manufacturers and device types using
//! a built-in lookup table of common hardware vendors.

use std::collections::HashMap;

/// Identification result for an unknown device.
#[derive(Debug, Clone)]
pub struct IdentificationResult {
    pub device_name: String,
    pub manufacturer: String,
    pub device_type: String,
    pub confidence: f32,
    pub alternative_matches: Vec<AlternativeMatch>,
}

/// An alternative possible identification.
#[derive(Debug, Clone)]
pub struct AlternativeMatch {
    pub device_name: String,
    pub manufacturer: String,
    pub confidence: f32,
}

/// Information about an unknown device to be identified.
#[derive(Debug, Clone)]
pub struct UnknownDeviceInfo {
    pub hardware_id: String,
    pub description: String,
    pub device_class: String,
    pub vendor_string: String,
}

/// Hardware identifier engine using local PCI/USB vendor database.
pub struct HardwareIdentifier {
    pci_vendors: HashMap<String, VendorInfo>,
    usb_vendors: HashMap<String, VendorInfo>,
    cache: HashMap<String, IdentificationResult>,
}

#[derive(Debug, Clone)]
struct VendorInfo {
    name: String,
    known_devices: HashMap<String, String>,
}

impl Default for HardwareIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareIdentifier {
    /// Create a new identifier with built-in vendor database.
    pub fn new() -> Self {
        let mut id = Self {
            pci_vendors: HashMap::new(),
            usb_vendors: HashMap::new(),
            cache: HashMap::new(),
        };
        id.load_builtin_database();
        id
    }

    /// Identify an unknown device using heuristics.
    pub fn identify_device(&mut self, device: &UnknownDeviceInfo) -> IdentificationResult {
        // Check cache first
        if let Some(cached) = self.cache.get(&device.hardware_id) {
            return cached.clone();
        }

        let result = self.identify_internal(device);
        self.cache
            .insert(device.hardware_id.clone(), result.clone());
        result
    }

    fn identify_internal(&self, device: &UnknownDeviceInfo) -> IdentificationResult {
        let hw_id = device.hardware_id.to_uppercase();

        // Strategy 1: Parse PCI VEN/DEV IDs
        if let Some(result) = self.identify_from_pci_id(&hw_id) {
            return result;
        }

        // Strategy 2: Parse USB VID/PID
        if let Some(result) = self.identify_from_usb_id(&hw_id) {
            return result;
        }

        // Strategy 3: Heuristic from class and description
        self.identify_from_heuristics(device)
    }

    /// Parse PCI\VEN_xxxx&DEV_yyyy and look up in vendor database.
    fn identify_from_pci_id(&self, hw_id: &str) -> Option<IdentificationResult> {
        let ven_id = extract_field(hw_id, "VEN_")?;
        let dev_id = extract_field(hw_id, "DEV_");

        let vendor = self.pci_vendors.get(&ven_id)?;
        let device_name = dev_id
            .as_ref()
            .and_then(|d| vendor.known_devices.get(d))
            .cloned()
            .unwrap_or_else(|| format!("{} Device", vendor.name));

        let confidence =
            if dev_id.is_some() && vendor.known_devices.contains_key(dev_id.as_ref().unwrap()) {
                0.95
            } else {
                0.7
            };

        Some(IdentificationResult {
            device_name,
            manufacturer: vendor.name.clone(),
            device_type: classify_by_class_code(hw_id),
            confidence,
            alternative_matches: vec![],
        })
    }

    /// Parse USB\VID_xxxx&PID_yyyy and look up.
    fn identify_from_usb_id(&self, hw_id: &str) -> Option<IdentificationResult> {
        let vid = extract_field(hw_id, "VID_")?;
        let pid = extract_field(hw_id, "PID_");

        let vendor = self.usb_vendors.get(&vid)?;
        let device_name = pid
            .as_ref()
            .and_then(|p| vendor.known_devices.get(p))
            .cloned()
            .unwrap_or_else(|| format!("{} USB Device", vendor.name));

        Some(IdentificationResult {
            device_name,
            manufacturer: vendor.name.clone(),
            device_type: "USB Device".to_string(),
            confidence: 0.8,
            alternative_matches: vec![],
        })
    }

    /// Fallback: identify from class name and description keywords.
    fn identify_from_heuristics(&self, device: &UnknownDeviceInfo) -> IdentificationResult {
        let desc = device.description.to_lowercase();
        let class = device.device_class.to_lowercase();

        let (device_type, confidence) =
            if class.contains("display") || desc.contains("gpu") || desc.contains("graphics") {
                ("Graphics Adapter".to_string(), 0.6)
            } else if class.contains("net")
                || desc.contains("ethernet")
                || desc.contains("wi-fi")
                || desc.contains("wireless")
            {
                ("Network Adapter".to_string(), 0.6)
            } else if class.contains("media") || desc.contains("audio") || desc.contains("sound") {
                ("Audio Device".to_string(), 0.6)
            } else if class.contains("usb") || desc.contains("usb") {
                ("USB Controller".to_string(), 0.4)
            } else if class.contains("hid") || desc.contains("keyboard") || desc.contains("mouse") {
                ("Input Device".to_string(), 0.5)
            } else if class.contains("disk")
                || desc.contains("storage")
                || desc.contains("nvme")
                || desc.contains("sata")
            {
                ("Storage Controller".to_string(), 0.5)
            } else {
                ("Unknown Device".to_string(), 0.1)
            };

        let manufacturer = if !device.vendor_string.is_empty() {
            device.vendor_string.clone()
        } else {
            "(Unknown)".to_string()
        };

        IdentificationResult {
            device_name: if !device.description.is_empty() {
                device.description.clone()
            } else {
                device_type.clone()
            },
            manufacturer,
            device_type,
            confidence,
            alternative_matches: vec![],
        }
    }

    /// Load the built-in PCI/USB vendor database.
    #[allow(clippy::type_complexity)]
    fn load_builtin_database(&mut self) {
        // PCI vendors (top vendors by market share)
        let pci_data: &[(&str, &str, &[(&str, &str)])] = &[
            (
                "8086",
                "Intel",
                &[
                    ("0A16", "HD Graphics 4400"),
                    ("1901", "Xeon E3-1200 v5 PCI Express"),
                    ("6F08", "Xeon E5 v4 PCIe Root Port"),
                    ("A170", "100 Series Chipset SATA"),
                    ("15B8", "Ethernet Connection I219-LM"),
                    ("A2AF", "200 Series Chipset USB 3.0"),
                    ("3E92", "CoffeeLake-S GT2 UHD 630"),
                    ("A3B1", "Comet Lake PCH-V USB"),
                ],
            ),
            (
                "10DE",
                "NVIDIA",
                &[
                    ("2204", "GeForce RTX 3090"),
                    ("2206", "GeForce RTX 3080"),
                    ("2208", "GeForce RTX 3080 Ti"),
                    ("2484", "GeForce RTX 3070"),
                    ("2786", "GeForce RTX 4070"),
                    ("2684", "GeForce RTX 4090"),
                ],
            ),
            (
                "1002",
                "AMD/ATI",
                &[
                    ("73BF", "Radeon RX 6900 XT"),
                    ("73DF", "Radeon RX 6700 XT"),
                    ("744C", "Radeon RX 7900 XTX"),
                    ("7480", "Radeon RX 7600"),
                ],
            ),
            (
                "1022",
                "AMD",
                &[
                    ("1480", "Starship/Matisse Root Complex"),
                    ("1630", "Renoir Root Complex"),
                    ("15E8", "Raven USB 3.1"),
                ],
            ),
            (
                "14E4",
                "Broadcom",
                &[("4353", "BCM43224 Wireless"), ("43A0", "BCM4360 802.11ac")],
            ),
            (
                "10EC",
                "Realtek",
                &[
                    ("8168", "RTL8111/8168 Gigabit Ethernet"),
                    ("8821", "RTL8821CE 802.11ac"),
                    ("0887", "ALC887 HD Audio"),
                ],
            ),
            (
                "168C",
                "Qualcomm Atheros",
                &[("003E", "QCA6174 802.11ac"), ("0042", "QCA9377 802.11ac")],
            ),
            (
                "1B21",
                "ASMedia",
                &[("1142", "ASM1042A USB 3.0"), ("2142", "ASM2142 USB 3.1")],
            ),
            (
                "144D",
                "Samsung",
                &[
                    ("A808", "NVMe SSD 970 EVO Plus"),
                    ("A80A", "NVMe SSD 980 PRO"),
                ],
            ),
        ];

        for (ven_id, name, devices) in pci_data {
            let mut known = HashMap::new();
            for (dev_id, dev_name) in *devices {
                known.insert(dev_id.to_string(), dev_name.to_string());
            }
            self.pci_vendors.insert(
                ven_id.to_string(),
                VendorInfo {
                    name: name.to_string(),
                    known_devices: known,
                },
            );
        }

        // USB vendors (common peripherals)
        let usb_data: &[(&str, &str, &[(&str, &str)])] = &[
            (
                "046D",
                "Logitech",
                &[
                    ("C077", "M105 Mouse"),
                    ("C534", "Unifying Receiver"),
                    ("0825", "C920 Webcam"),
                ],
            ),
            (
                "045E",
                "Microsoft",
                &[("07A5", "Wireless Adapter"), ("0810", "LifeCam HD-3000")],
            ),
            ("0951", "Kingston", &[("1666", "DataTraveler USB")]),
            ("058F", "Alcor Micro", &[("6387", "USB Card Reader")]),
            (
                "8087",
                "Intel",
                &[("0A2A", "Bluetooth Adapter"), ("0029", "AX200 Bluetooth")],
            ),
        ];

        for (vid, name, devices) in usb_data {
            let mut known = HashMap::new();
            for (pid, dev_name) in *devices {
                known.insert(pid.to_string(), dev_name.to_string());
            }
            self.usb_vendors.insert(
                vid.to_string(),
                VendorInfo {
                    name: name.to_string(),
                    known_devices: known,
                },
            );
        }
    }
}

/// Extract a 4-character hex field from a hardware ID string.
fn extract_field(hw_id: &str, prefix: &str) -> Option<String> {
    let upper = hw_id.to_uppercase();
    let prefix_upper = prefix.to_uppercase();
    if let Some(pos) = upper.find(&prefix_upper) {
        let start = pos + prefix_upper.len();
        let end = upper[start..]
            .find(|c: char| !c.is_ascii_hexdigit())
            .map(|p| start + p)
            .unwrap_or(upper.len());
        if end > start {
            return Some(upper[start..end].to_string());
        }
    }
    None
}

/// Classify device type from PCI class code in hardware ID (if present).
fn classify_by_class_code(hw_id: &str) -> String {
    let upper = hw_id.to_uppercase();
    if let Some(pos) = upper.find("CC_") {
        let code = &upper[pos + 3..std::cmp::min(pos + 7, upper.len())];
        return match &code[..2.min(code.len())] {
            "01" => "Storage Controller",
            "02" => "Network Controller",
            "03" => "Display Controller",
            "04" => "Multimedia Controller",
            "05" => "Memory Controller",
            "06" => "Bridge Device",
            "07" => "Communication Controller",
            "08" => "System Peripheral",
            "09" => "Input Device",
            "0C" => "Serial Bus Controller",
            "0D" => "Wireless Controller",
            _ => "PCI Device",
        }
        .to_string();
    }
    "PCI Device".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_intel_pci() {
        let mut id = HardwareIdentifier::new();
        let result = id.identify_device(&UnknownDeviceInfo {
            hardware_id: "PCI\\VEN_8086&DEV_15B8&SUBSYS_00008086&REV_00".into(),
            description: String::new(),
            device_class: String::new(),
            vendor_string: String::new(),
        });
        assert_eq!(result.manufacturer, "Intel");
        assert!(result.confidence >= 0.9);
        assert!(result.device_name.contains("I219"));
    }

    #[test]
    fn test_identify_nvidia_pci() {
        let mut id = HardwareIdentifier::new();
        let result = id.identify_device(&UnknownDeviceInfo {
            hardware_id: "PCI\\VEN_10DE&DEV_2204".into(),
            description: String::new(),
            device_class: String::new(),
            vendor_string: String::new(),
        });
        assert_eq!(result.manufacturer, "NVIDIA");
        assert!(result.device_name.contains("RTX 3090"));
    }

    #[test]
    fn test_identify_usb_logitech() {
        let mut id = HardwareIdentifier::new();
        let result = id.identify_device(&UnknownDeviceInfo {
            hardware_id: "USB\\VID_046D&PID_C077".into(),
            description: String::new(),
            device_class: String::new(),
            vendor_string: String::new(),
        });
        assert_eq!(result.manufacturer, "Logitech");
        assert!(result.device_name.contains("M105"));
    }

    #[test]
    fn test_heuristic_fallback() {
        let mut id = HardwareIdentifier::new();
        let result = id.identify_device(&UnknownDeviceInfo {
            hardware_id: "ACPI\\UNKNOWN_DEVICE".into(),
            description: "High Definition Audio Controller".into(),
            device_class: "MEDIA".into(),
            vendor_string: String::new(),
        });
        assert_eq!(result.device_type, "Audio Device");
        assert!(result.confidence > 0.0);
    }
}
