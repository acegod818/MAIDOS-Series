//! Driver matching module
//!
//! Matches hardware device IDs to available driver packages
//! using PnP compatible ID comparison and version ranking.

pub mod driver_matching;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Driver matching module initialized");
    Ok(())
}
