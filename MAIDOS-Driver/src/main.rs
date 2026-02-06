//! MAIDOS-Driver 主程式 - 真實硬體偵測

use log::{error, info};

fn main() {
    // 初始化日誌
    env_logger::init();

    info!("MAIDOS-Driver 啟動中...");
    println!("================================================================================");
    println!("🐉 MAIDOS-Driver - 真實硬體偵測工具");
    println!("================================================================================");

    // 初始化硬體偵測模組
    if let Err(e) = maidOS_driver::core::detect::hardware::init() {
        error!("硬體偵測初始化失敗: {}", e);
        return;
    }

    // 執行真實的硬體掃描
    info!("正在掃描硬體設備...");
    println!("\n🎯 正在掃描硬體設備...");

    match maidOS_driver::core::detect::hardware::scan_all_devices() {
        Ok(devices) => {
            println!("✅ 掃描完成，找到 {} 個設備", devices.len());
            println!(
                "================================================================================"
            );

            for (i, device) in devices.iter().enumerate() {
                println!("{} | 類別: {}", i + 1, device.class);
                println!("  ├── 名稱: {}", device.name);
                println!("  ├── 廠商: {}", device.vendor);
                println!("  ├── 版本: {}", device.version);
                println!("  ├── 狀態: {}", device.status);
                println!("  └── ID: {}", device.id);

                if i < devices.len() - 1 {
                    println!(
                        "------------------------------------------------------------------------"
                    );
                }
            }

            println!(
                "================================================================================"
            );
            info!("找到 {} 個硬體設備", devices.len());
        }
        Err(e) => {
            error!("硬體掃描失敗: {}", e);
            println!("❌ 硬體掃描失敗: {}", e);
        }
    }

    println!("🔚 MAIDOS-Driver 執行完畢。");
    info!("MAIDOS-Driver 執行完畢");
}
