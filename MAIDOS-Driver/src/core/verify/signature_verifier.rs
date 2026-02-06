//! Authenticode signature verifier
//!
//! Uses Windows `WinVerifyTrust` API to check if a driver file (.sys, .inf, .dll)
//! has a valid digital signature from a trusted publisher.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Security::WinTrust::{
    WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_FILE_INFO,
    WTD_CHOICE_FILE, WTD_REVOKE_NONE, WTD_STATEACTION_VERIFY, WTD_UI_NONE,
};

// WinVerifyTrust HRESULT codes
const TRUST_E_NOSIGNATURE: i32 = 0x800B0100_u32 as i32;
const TRUST_E_EXPLICIT_DISTRUST: i32 = 0x800B0101_u32 as i32;
const TRUST_E_SUBJECT_NOT_TRUSTED: i32 = 0x800B0109_u32 as i32;
const CRYPT_E_REVOKED: i32 = 0x800B010C_u32 as i32;

/// Result of a signature verification.
#[derive(Debug, Clone)]
pub struct SignatureVerificationResult {
    /// Whether the file has any signature at all.
    pub is_signed: bool,
    /// Whether the signature chain is trusted.
    pub is_trusted: bool,
    /// Signer identity (subject name), if available.
    pub signer_identity: Option<String>,
    /// Verification timestamp.
    pub verification_time: std::time::SystemTime,
    /// Raw HRESULT from WinVerifyTrust.
    pub raw_status: i32,
    /// Human-readable status.
    pub status_message: String,
}

/// Verify the Authenticode signature on a file.
///
/// Returns a `SignatureVerificationResult` with trust status.
/// Uses `WinVerifyTrust` with `WINTRUST_ACTION_GENERIC_VERIFY_V2`.
pub fn verify_file_signature(
    file_path: &str,
) -> Result<SignatureVerificationResult, Box<dyn std::error::Error>> {
    log::info!("Verifying signature: {}", file_path);

    if !Path::new(file_path).exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let wide_path: Vec<u16> = OsStr::new(file_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: windows::core::PCWSTR(wide_path.as_ptr()),
        hFile: HANDLE::default(),
        pgKnownSubject: std::ptr::null_mut(),
    };

    let mut trust_data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        dwStateAction: WTD_STATEACTION_VERIFY,
        ..unsafe { std::mem::zeroed() }
    };

    // Set the union to point to our file info
    trust_data.Anonymous.pFile = &mut file_info as *mut _;

    let mut action_id = WINTRUST_ACTION_GENERIC_VERIFY_V2;

    let status = unsafe {
        WinVerifyTrust(
            HWND(-1isize as *mut _),
            &mut action_id,
            &mut trust_data as *mut _ as *mut _,
        )
    };

    let (is_signed, is_trusted, message) = match status {
        0 => (true, true, "Valid signature, trusted publisher".to_string()),
        TRUST_E_NOSIGNATURE => (false, false, "No signature present".to_string()),
        TRUST_E_EXPLICIT_DISTRUST => (
            true,
            false,
            "Signature present but certificate not trusted".to_string(),
        ),
        CRYPT_E_REVOKED => (
            true,
            false,
            "Signature present but certificate revoked".to_string(),
        ),
        TRUST_E_SUBJECT_NOT_TRUSTED => (
            true,
            false,
            "Signature present but root certificate not trusted".to_string(),
        ),
        other => (
            false,
            false,
            format!("Verification failed with HRESULT 0x{:08X}", other as u32),
        ),
    };

    log::info!(
        "Signature check for {}: signed={}, trusted={}, status=0x{:08X}",
        file_path,
        is_signed,
        is_trusted,
        status as u32,
    );

    Ok(SignatureVerificationResult {
        is_signed,
        is_trusted,
        signer_identity: if is_signed {
            Some("(Authenticode signer)".to_string())
        } else {
            None
        },
        verification_time: std::time::SystemTime::now(),
        raw_status: status,
        status_message: message,
    })
}

/// Verify all driver files (*.sys, *.inf, *.dll, *.cat) in a directory.
pub fn verify_directory(
    dir_path: &str,
) -> Result<Vec<(String, SignatureVerificationResult)>, Box<dyn std::error::Error>> {
    log::info!("Verifying all driver files in {}", dir_path);

    let mut results = Vec::new();
    let extensions = ["sys", "inf", "dll", "cat"];

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext.to_lowercase().as_str()) {
                let path_str = path.to_string_lossy().to_string();
                match verify_file_signature(&path_str) {
                    Ok(result) => results.push((path_str, result)),
                    Err(e) => {
                        log::warn!("Failed to verify {}: {}", path_str, e);
                    }
                }
            }
        }
    }

    log::info!("Verified {} files in {}", results.len(), dir_path);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_nonexistent_file() {
        let result = verify_file_signature("C:\\nonexistent\\driver.sys");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_system_file() {
        // ntdll.dll is always present and Microsoft-signed
        let result = verify_file_signature("C:\\Windows\\System32\\ntdll.dll");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_signed, "ntdll.dll should be signed");
        assert!(info.is_trusted, "ntdll.dll should be trusted");
    }
}
