//! Driver verification module
//!
//! Validates digital signatures on driver files using Windows Authenticode.

pub mod signature_verifier;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Signature verification module initialized");
    Ok(())
}
