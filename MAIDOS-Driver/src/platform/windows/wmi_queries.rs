//! WMI query module — real COM/WMI implementation
//!
//! Uses Windows COM API to execute WQL queries against the local WMI service.
//! Connects to `\\.\root\cimv2` namespace for system information queries.
//!
//! Each query runs on a dedicated STA (Single-Threaded Apartment) thread.
//! This is the MSDN-recommended pattern: STA + synchronous WMI avoids the
//! MTA enumerator proxy security deadlock that plagues semi-async mode.

use std::collections::HashMap;
use windows::core::{BSTR, VARIANT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoSetProxyBlanket, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Wmi::{
    IWbemLocator, WbemLocator, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
};

/// RPC_C_AUTHN_WINNT (NTLMSSP authentication service)
const RPC_C_AUTHN_WINNT: u32 = 10;
/// RPC_C_AUTHZ_NONE (no authorization)
const RPC_C_AUTHZ_NONE: u32 = 0;

/// A single WMI query result row.
#[derive(Debug, Clone)]
pub struct WmiResult {
    pub class_name: String,
    pub properties: HashMap<String, String>,
}

/// WMI query engine backed by real COM/WMI.
///
/// Each query spawns a dedicated STA thread with its own COM lifecycle.
/// This avoids MTA enumerator proxy deadlocks while keeping the API simple.
#[derive(Default)]
pub struct WmiQuery {
    _private: (),
}


impl WmiQuery {
    /// Create a new WMI query engine.
    ///
    /// Validates connectivity by performing a test connection on a background
    /// STA thread. The connection is not cached — each query creates its own.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Validate WMI is accessible by doing a quick connection test
        run_wmi_query("SELECT __CLASS FROM Win32_OperatingSystem".to_string())
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        log::info!("WMI: connection validated (STA thread pool mode)");
        Ok(Self { _private: () })
    }

    /// Execute an arbitrary WQL query and return all result rows.
    pub fn execute_query(&self, wql: &str) -> Result<Vec<WmiResult>, Box<dyn std::error::Error>> {
        log::debug!("WMI query: {}", wql);
        let results = run_wmi_query(wql.to_string())
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        log::debug!("WMI query returned {} rows", results.len());
        Ok(results)
    }

    /// Query Win32_ComputerSystem for machine info.
    pub fn get_computer_system_info(&self) -> Result<WmiResult, Box<dyn std::error::Error>> {
        let rows = self.execute_query("SELECT * FROM Win32_ComputerSystem")?;
        rows.into_iter()
            .next()
            .ok_or_else(|| "No Win32_ComputerSystem result".into())
    }

    /// Query Win32_OperatingSystem.
    pub fn get_operating_system_info(&self) -> Result<WmiResult, Box<dyn std::error::Error>> {
        let rows = self.execute_query("SELECT * FROM Win32_OperatingSystem")?;
        rows.into_iter()
            .next()
            .ok_or_else(|| "No Win32_OperatingSystem result".into())
    }

    /// Query Win32_BIOS.
    pub fn get_bios_info(&self) -> Result<WmiResult, Box<dyn std::error::Error>> {
        let rows = self.execute_query("SELECT * FROM Win32_BIOS")?;
        rows.into_iter()
            .next()
            .ok_or_else(|| "No Win32_BIOS result".into())
    }

    /// Query Win32_Processor (may return multiple CPUs).
    pub fn get_cpu_info(&self) -> Result<Vec<WmiResult>, Box<dyn std::error::Error>> {
        self.execute_query(
            "SELECT Name, NumberOfCores, MaxClockSpeed, Manufacturer FROM Win32_Processor",
        )
    }

    /// Query Win32_PhysicalMemory.
    pub fn get_memory_info(&self) -> Result<Vec<WmiResult>, Box<dyn std::error::Error>> {
        self.execute_query(
            "SELECT Capacity, Speed, Manufacturer, PartNumber FROM Win32_PhysicalMemory",
        )
    }

    /// Query Win32_DiskDrive.
    pub fn get_disk_drive_info(&self) -> Result<Vec<WmiResult>, Box<dyn std::error::Error>> {
        self.execute_query("SELECT Model, Size, InterfaceType, MediaType FROM Win32_DiskDrive")
    }

    /// Query Win32_NetworkAdapter (physical adapters only).
    pub fn get_network_adapter_info(&self) -> Result<Vec<WmiResult>, Box<dyn std::error::Error>> {
        self.execute_query(
            "SELECT Name, MACAddress, Speed, AdapterType FROM Win32_NetworkAdapter WHERE PhysicalAdapter = TRUE",
        )
    }
}

/// Execute a WQL query on a dedicated STA thread with full COM lifecycle.
///
/// This is the core function that avoids all MTA/COM threading issues:
/// 1. Spawns a new thread
/// 2. Initializes COM as STA (Single-Threaded Apartment)
/// 3. Creates WMI locator and connects to ROOT\CIMV2
/// 4. Sets proxy security blanket for impersonation
/// 5. Runs synchronous query (safe in STA, deadlocks in MTA)
/// 6. Enumerates results and extracts properties
/// 7. Cleans up COM before returning
fn run_wmi_query(wql: String) -> Result<Vec<WmiResult>, String> {
    let handle = std::thread::spawn(move || -> Result<Vec<WmiResult>, String> {
        unsafe {
            // Step 1: Initialize COM as STA — required for synchronous WMI.
            // STA guarantees message-based dispatch, which WMI synchronous
            // operations depend on. MTA + synchronous = deadlock.
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(format!("COM STA init failed: {hr:?}"));
            }

            let result = wmi_query_inner(&wql);

            // Always uninitialize COM on this thread, even if query failed
            CoUninitialize();

            result
        }
    });

    handle
        .join()
        .map_err(|_| "WMI query thread panicked".to_string())?
}

/// Inner WMI query logic — runs inside a COM STA-initialized thread.
unsafe fn wmi_query_inner(wql: &str) -> Result<Vec<WmiResult>, String> {
    // Step 2: Create WMI locator via CoCreateInstance
    let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
        .map_err(|e| format!("WMI locator creation failed: {e}"))?;

    // Step 3: Connect to ROOT\CIMV2 namespace
    let services = locator
        .ConnectServer(
            &BSTR::from("ROOT\\CIMV2"),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            0,
            &BSTR::new(),
            None,
        )
        .map_err(|e| format!("WMI ConnectServer failed: {e}"))?;

    // Step 4: Set security blanket on the services proxy.
    // Without this, WMI denies access and returns empty/error results.
    CoSetProxyBlanket(
        &services,
        RPC_C_AUTHN_WINNT,
        RPC_C_AUTHZ_NONE,
        None,
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
    )
    .map_err(|e| format!("CoSetProxyBlanket failed: {e}"))?;

    // Step 5: Execute semi-synchronous query (FORWARD_ONLY + RETURN_IMMEDIATELY).
    // Semi-sync ExecQuery returns immediately; results are fetched via Next().
    // On STA threads, COM automatically pumps messages during Next() waits.
    let enumerator = services
        .ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from(wql),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )
        .map_err(|e| format!("WMI ExecQuery failed: {e}"))?;

    // Step 6: Enumerate results
    let mut results = Vec::new();

    loop {
        let mut objects = [None; 1];
        let mut returned: u32 = 0;

        let hr = enumerator.Next(10_000i32, &mut objects, &mut returned);
        if hr.is_err() || returned == 0 {
            break;
        }

        if let Some(obj) = &objects[0] {
            let mut props = HashMap::new();

            // Enumerate all properties of this WMI object
            let _ = obj.BeginEnumeration(0);

            loop {
                let mut name = BSTR::new();
                let mut value = VARIANT::default();

                let hr = obj.Next(
                    0,
                    &mut name,
                    &mut value,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                if hr.is_err() {
                    break;
                }

                // IWbemClassObject::Next returns WBEM_S_NO_MORE_DATA (0x00040005)
                // when enumeration is complete. The windows crate wraps this as Ok(())
                // since the severity bit is 0. Detect end-of-enumeration by checking
                // if the output name BSTR is empty (WMI property names are never empty).
                if name.is_empty() {
                    break;
                }

                let prop_name = name.to_string();
                let prop_value = variant_to_string(&value);
                props.insert(prop_name, prop_value);
            }

            let _ = obj.EndEnumeration();

            // Extract __CLASS if available
            let class_name = props.remove("__CLASS").unwrap_or_default();
            results.push(WmiResult {
                class_name,
                properties: props,
            });
        }
    }

    Ok(results)
}

/// Convert a COM VARIANT to a display string.
fn variant_to_string(v: &VARIANT) -> String {
    format!("{}", v)
}

/// Initialize WMI module.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("WMI query module initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmi_cpu_query() {
        let wmi = WmiQuery::new().expect("WMI connection should succeed");
        let cpus = wmi.get_cpu_info().expect("CPU query should succeed");
        assert!(!cpus.is_empty(), "Should find at least one CPU");

        // Verify we got real CPU data
        let cpu = &cpus[0];
        assert!(
            cpu.properties.contains_key("Name"),
            "CPU should have a Name property"
        );
    }

    #[test]
    fn test_wmi_os_query() {
        let wmi = WmiQuery::new().expect("WMI connection should succeed");
        let os = wmi
            .get_operating_system_info()
            .expect("OS query should succeed");
        assert!(!os.properties.is_empty(), "OS should have properties");

        // Verify we got real OS data
        assert!(
            os.properties.contains_key("Caption") || os.properties.contains_key("Name"),
            "OS should have Caption or Name"
        );
    }
}
