//! 硬體偵測功能測試

use maidOS_driver::core::detect::hardware::scan_all_devices;

#[test]
fn test_hardware_detection_basic() {
    // 測試掃描功能是否能正常工作
    let devices = scan_all_devices().expect("硬體掃描應該成功");
    assert!(!devices.is_empty(), "應該至少找到1個設備");

    // 驗證設備基本信息
    let first_device = &devices[0];
    assert!(!first_device.name.is_empty(), "設備名稱不應為空");
    assert!(!first_device.id.is_empty(), "設備ID不應為空");

    println!("找到 {} 個設備:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!(
            "  {}. {} ({}) - {}",
            i + 1,
            device.name,
            device.vendor,
            device.status
        );
    }
}
