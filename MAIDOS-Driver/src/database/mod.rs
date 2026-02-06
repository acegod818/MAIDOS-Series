//! Driver database module
//!
//! Persistent driver catalog stored as tab-separated text files.

pub mod driver_database;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Database module initialized");
    Ok(())
}
