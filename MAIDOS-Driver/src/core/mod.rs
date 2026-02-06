//! MAIDOS-Driver 核心模組

/// 硬體偵測模組
pub mod detect;

/// 驅動安裝模組
pub mod install;

/// 驅動備份模組
pub mod backup;

/// 驅動更新模組
pub mod update;

/// 設備診斷模組
pub mod diagnose;

/// 驅動還原模組
pub mod restore;

/// 驅動匹配模組
pub mod driver_match;

/// 驅動下載模組
pub mod download;

/// 驅動簽章驗證模組
pub mod verify;

/// 審計日誌模組
pub mod audit;
