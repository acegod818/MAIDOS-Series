using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;
using System.Text;
using System.Diagnostics;

namespace MAIDOS.Driver.Service
{
    /// <summary>
    /// [MAIDOS-AUDIT] 硬體與驅動服務
    /// FFI 已對齊 Rust temp_test/src/ffi.rs 的 13 個導出函式
    /// 原有 4 個 + 新增 6 個功能 + 3 個記憶體釋放 = 13 個
    /// </summary>
    public class HardwareDetectionService
    {
        // DLL 名稱對齊 Rust cdylib 產出 (crate name = maidOS-driver -> maidOS_driver.dll)
        private const string NativeDll = "maidOS_driver.dll";

        // =====================================================================
        // FFI Structs - 對齊 Rust #[repr(C)] 結構體
        // =====================================================================

        /// <summary>
        /// 對齊 Rust CDeviceInfo: 5 個 *mut c_char 指標 (各 8 bytes on x64)
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct CDeviceInfo
        {
            public IntPtr id;       // *mut c_char
            public IntPtr name;     // *mut c_char
            public IntPtr vendor;   // *mut c_char
            public IntPtr version;  // *mut c_char
            public IntPtr status;   // *mut c_char
        }

        /// <summary>
        /// 對齊 Rust CUpdateInfo: 5 個 *mut c_char 指標 + 1 個 i32
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct CUpdateInfo
        {
            public IntPtr device_id;         // *mut c_char
            public IntPtr current_version;   // *mut c_char
            public IntPtr latest_version;    // *mut c_char
            public int update_available;     // 0 = false, 1 = true
            public IntPtr status;            // *mut c_char
            public IntPtr download_url;      // *mut c_char — official download URL
        }

        /// <summary>
        /// 對齊 Rust CBackupEntry: 3 個 *mut c_char 指標 + 1 個 u64
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct CBackupEntry
        {
            public IntPtr id;        // *mut c_char
            public IntPtr timestamp; // *mut c_char
            public IntPtr path;      // *mut c_char
            public ulong size;       // u64
        }

        /// <summary>
        /// 對齊 Rust CDiagnosticInfo: 3 個 *mut c_char 指標 + 2 個 i32
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        internal struct CDiagnosticInfo
        {
            public IntPtr device_id;            // *mut c_char
            public int problem_code;            // i32
            public IntPtr problem_description;  // *mut c_char
            public int irq;                     // i32
            public IntPtr status;               // *mut c_char
        }

        // =====================================================================
        // Native API Imports — 原有 4 個
        // =====================================================================

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "scan_all_devices_c")]
        private static extern int scan_all_devices_c(out IntPtr devices_ptr);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_device_info")]
        private static extern void free_device_info(IntPtr devices, int count);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_last_error")]
        private static extern IntPtr get_last_error();

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_string")]
        private static extern void free_string(IntPtr s);

        // =====================================================================
        // Native API Imports — 新增 6 個功能 + 3 個記憶體釋放
        // =====================================================================

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "install_driver_c")]
        private static extern int install_driver_c(IntPtr inf_path);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "backup_drivers_c")]
        private static extern int backup_drivers_c(IntPtr backup_path, out IntPtr entries_ptr);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_backup_entries")]
        private static extern void free_backup_entries(IntPtr entries, int count);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "check_driver_update_c")]
        private static extern int check_driver_update_c(IntPtr device_id, IntPtr update_server, IntPtr info_ptr);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_update_info")]
        private static extern void free_update_info(IntPtr info);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "download_update_c")]
        private static extern long download_update_c(IntPtr download_url, IntPtr save_path);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "apply_update_c")]
        private static extern int apply_update_c(IntPtr inf_path, IntPtr device_id);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "check_all_updates_c")]
        private static extern int check_all_updates_c(out IntPtr updates_ptr);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_update_info_array")]
        private static extern void free_update_info_array(IntPtr updates, int count);

        // =====================================================================
        // Native API Imports — BUG-004/005 新增 3 個 (Diagnose + Rollback)
        // =====================================================================

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "diagnose_device_c")]
        private static extern int diagnose_device_c(IntPtr device_id, IntPtr info_ptr);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "free_diagnostic_info")]
        private static extern void free_diagnostic_info(IntPtr info);

        [DllImport(NativeDll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rollback_driver_c")]
        private static extern int rollback_driver_c(IntPtr device_id, IntPtr backup_path);

        // =====================================================================
        // Managed Types
        // =====================================================================

        public class DeviceInfo
        {
            public string Id { get; set; } = string.Empty;
            public string Name { get; set; } = string.Empty;
            public string Vendor { get; set; } = string.Empty;
            public string Version { get; set; } = string.Empty;
            public string Status { get; set; } = string.Empty;
            public string ProblemDescription { get; set; } = string.Empty;
            public string ResourceInfo { get; set; } = string.Empty;
        }

        public class UpdateInfo
        {
            public string DeviceId { get; set; } = string.Empty;
            public string CurrentVersion { get; set; } = string.Empty;
            public string LatestVersion { get; set; } = string.Empty;
            public bool UpdateAvailable { get; set; }
            public string Status { get; set; } = string.Empty;
            public string DownloadUrl { get; set; } = string.Empty;
        }

        public class BackupInfo
        {
            public string Id { get; set; } = string.Empty;
            public string Timestamp { get; set; } = string.Empty;
            public string Path { get; set; } = string.Empty;
            public ulong Size { get; set; }
        }

        public class DiagnosticInfo
        {
            public string DeviceId { get; set; } = string.Empty;
            public int ProblemCode { get; set; }
            public string ProblemDescription { get; set; } = string.Empty;
            public int Irq { get; set; }
            public string Status { get; set; } = string.Empty;
        }

        // =====================================================================
        // Helper: 指標 → 字串 / 結構讀取
        // =====================================================================

        private static string PtrToStringUtf8(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero) return "Unknown";
            return Marshal.PtrToStringUTF8(ptr) ?? "Unknown";
        }

        private static string GetLastError()
        {
            IntPtr errPtr = get_last_error();
            string errMsg = PtrToStringUtf8(errPtr);
            free_string(errPtr);
            return errMsg;
        }

        private static IntPtr StringToPtr(string? s)
        {
            if (string.IsNullOrEmpty(s)) return IntPtr.Zero;
            return Marshal.StringToCoTaskMemUTF8(s);
        }

        private static void FreePtr(IntPtr ptr)
        {
            if (ptr != IntPtr.Zero) Marshal.FreeCoTaskMem(ptr);
        }

        private static DeviceInfo ReadDeviceFromPtr(IntPtr basePtr, int index)
        {
            int structSize = Marshal.SizeOf<CDeviceInfo>();
            IntPtr itemPtr = basePtr + index * structSize;
            CDeviceInfo c = Marshal.PtrToStructure<CDeviceInfo>(itemPtr);

            return new DeviceInfo
            {
                Id = PtrToStringUtf8(c.id),
                Name = PtrToStringUtf8(c.name),
                Vendor = PtrToStringUtf8(c.vendor),
                Version = PtrToStringUtf8(c.version),
                Status = PtrToStringUtf8(c.status),
                ProblemDescription = "OK",
                ResourceInfo = "No dedicated IRQ"
            };
        }

        private static UpdateInfo ReadUpdateFromPtr(IntPtr basePtr, int index)
        {
            int structSize = Marshal.SizeOf<CUpdateInfo>();
            IntPtr itemPtr = basePtr + index * structSize;
            CUpdateInfo c = Marshal.PtrToStructure<CUpdateInfo>(itemPtr);

            string dlUrl = PtrToStringUtf8(c.download_url);
            return new UpdateInfo
            {
                DeviceId = PtrToStringUtf8(c.device_id),
                CurrentVersion = PtrToStringUtf8(c.current_version),
                LatestVersion = PtrToStringUtf8(c.latest_version),
                UpdateAvailable = c.update_available != 0,
                Status = PtrToStringUtf8(c.status),
                DownloadUrl = dlUrl == "Unknown" ? string.Empty : dlUrl
            };
        }

        // =====================================================================
        // Service Methods — 原有功能
        // =====================================================================

        /// <summary>
        /// [MAIDOS-AUDIT] 掃描所有硬體設備 (透過 Rust FFI)
        /// </summary>
        public DeviceInfo[] ScanAllDevices()
        {
            int count = scan_all_devices_c(out IntPtr devicesPtr);

            if (count < 0)
            {
                throw new Exception($"[MAIDOS-AUDIT] Native Scan Failed: {GetLastError()}");
            }

            if (count == 0 || devicesPtr == IntPtr.Zero)
            {
                return Array.Empty<DeviceInfo>();
            }

            try
            {
                var results = new DeviceInfo[count];
                for (int i = 0; i < count; i++)
                {
                    results[i] = ReadDeviceFromPtr(devicesPtr, i);
                }
                return results;
            }
            finally
            {
                free_device_info(devicesPtr, count);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 診斷指定設備 (BUG-004 修復: 透過 Rust FFI diagnose_device_c)
        /// 使用 CM_Locate_DevNode + CM_Get_DevNode_Status 查詢真實問題碼
        /// </summary>
        public DiagnosticInfo DiagnoseDevice(string deviceId)
        {
            IntPtr devIdPtr = StringToPtr(deviceId);
            IntPtr infoMem = Marshal.AllocHGlobal(Marshal.SizeOf<CDiagnosticInfo>());
            try
            {
                int result = diagnose_device_c(devIdPtr, infoMem);
                if (result < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Diagnose Failed: {GetLastError()}");
                }

                CDiagnosticInfo c = Marshal.PtrToStructure<CDiagnosticInfo>(infoMem);
                var info = new DiagnosticInfo
                {
                    DeviceId = PtrToStringUtf8(c.device_id),
                    ProblemCode = c.problem_code,
                    ProblemDescription = PtrToStringUtf8(c.problem_description),
                    Irq = c.irq,
                    Status = PtrToStringUtf8(c.status)
                };

                free_diagnostic_info(infoMem);
                return info;
            }
            finally
            {
                FreePtr(devIdPtr);
                Marshal.FreeHGlobal(infoMem);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 回滾驅動程式 (BUG-005 修復: 透過 Rust FFI rollback_driver_c)
        /// 取代原本的 Process.Start("rstrui.exe") 空殼
        /// </summary>
        public bool RollbackDriver(string deviceId, string? backupPath = null)
        {
            IntPtr devIdPtr = StringToPtr(deviceId);
            IntPtr pathPtr = StringToPtr(backupPath);
            try
            {
                int result = rollback_driver_c(devIdPtr, pathPtr);
                if (result < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Rollback Failed: {GetLastError()}");
                }
                return true;
            }
            finally
            {
                FreePtr(devIdPtr);
                FreePtr(pathPtr);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 系統還原精靈 (保留為備用，主要使用 RollbackDriver)
        /// </summary>
        public void LaunchSystemRestore()
        {
            Process.Start("rstrui.exe");
        }

        // =====================================================================
        // Service Methods — 新增 6 個 (Rust FFI 已實作)
        // =====================================================================

        /// <summary>
        /// [MAIDOS-AUDIT] 安裝驅動 (透過 Rust FFI install_driver_c)
        /// </summary>
        public bool InstallDriver(string infPath)
        {
            IntPtr pathPtr = StringToPtr(infPath);
            try
            {
                int result = install_driver_c(pathPtr);
                if (result < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Install Driver Failed: {GetLastError()}");
                }
                return true;
            }
            finally
            {
                FreePtr(pathPtr);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 備份驅動 (透過 Rust FFI backup_drivers_c)
        /// </summary>
        public BackupInfo[] BackupDrivers(string path)
        {
            IntPtr pathPtr = StringToPtr(path);
            try
            {
                int count = backup_drivers_c(pathPtr, out IntPtr entriesPtr);
                if (count < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Backup Drivers Failed: {GetLastError()}");
                }

                if (count == 0 || entriesPtr == IntPtr.Zero)
                {
                    return Array.Empty<BackupInfo>();
                }

                try
                {
                    var results = new BackupInfo[count];
                    int structSize = Marshal.SizeOf<CBackupEntry>();
                    for (int i = 0; i < count; i++)
                    {
                        IntPtr itemPtr = entriesPtr + i * structSize;
                        CBackupEntry c = Marshal.PtrToStructure<CBackupEntry>(itemPtr);
                        results[i] = new BackupInfo
                        {
                            Id = PtrToStringUtf8(c.id),
                            Timestamp = PtrToStringUtf8(c.timestamp),
                            Path = PtrToStringUtf8(c.path),
                            Size = c.size
                        };
                    }
                    return results;
                }
                finally
                {
                    free_backup_entries(entriesPtr, count);
                }
            }
            finally
            {
                FreePtr(pathPtr);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 檢查單一設備驅動更新 (透過 Rust FFI check_driver_update_c)
        /// </summary>
        public UpdateInfo CheckUpdate(string deviceId, string? updateServer = null)
        {
            IntPtr devIdPtr = StringToPtr(deviceId);
            IntPtr serverPtr = StringToPtr(updateServer);
            IntPtr infoMem = Marshal.AllocHGlobal(Marshal.SizeOf<CUpdateInfo>());
            try
            {
                int result = check_driver_update_c(devIdPtr, serverPtr, infoMem);
                if (result < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Check Update Failed: {GetLastError()}");
                }

                CUpdateInfo c = Marshal.PtrToStructure<CUpdateInfo>(infoMem);
                string dlUrl = PtrToStringUtf8(c.download_url);
                var info = new UpdateInfo
                {
                    DeviceId = PtrToStringUtf8(c.device_id),
                    CurrentVersion = PtrToStringUtf8(c.current_version),
                    LatestVersion = PtrToStringUtf8(c.latest_version),
                    UpdateAvailable = c.update_available != 0,
                    Status = PtrToStringUtf8(c.status),
                    DownloadUrl = dlUrl == "Unknown" ? string.Empty : dlUrl
                };

                // 釋放 Rust 分配的字串指標
                free_update_info(infoMem);
                return info;
            }
            finally
            {
                FreePtr(devIdPtr);
                FreePtr(serverPtr);
                // infoMem 已由 free_update_info 處理字串，但結構記憶體由我們分配
                Marshal.FreeHGlobal(infoMem);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 下載驅動更新 (透過 Rust FFI download_update_c)
        /// </summary>
        public bool DownloadUpdate(string downloadUrl, string savePath)
        {
            IntPtr urlPtr = StringToPtr(downloadUrl);
            IntPtr pathPtr = StringToPtr(savePath);
            try
            {
                long bytes = download_update_c(urlPtr, pathPtr);
                if (bytes < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Download Failed: {GetLastError()}");
                }
                return true;
            }
            finally
            {
                FreePtr(urlPtr);
                FreePtr(pathPtr);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 套用驅動更新 (透過 Rust FFI apply_update_c)
        /// </summary>
        public bool ApplyUpdate(string infPath, string? deviceId = null)
        {
            IntPtr pathPtr = StringToPtr(infPath);
            IntPtr devIdPtr = StringToPtr(deviceId);
            try
            {
                int result = apply_update_c(pathPtr, devIdPtr);
                if (result < 0)
                {
                    throw new Exception($"[MAIDOS-AUDIT] Apply Update Failed: {GetLastError()}");
                }
                return true;
            }
            finally
            {
                FreePtr(pathPtr);
                FreePtr(devIdPtr);
            }
        }

        /// <summary>
        /// [MAIDOS-AUDIT] 批次檢查所有設備更新 (透過 Rust FFI check_all_updates_c)
        /// </summary>
        public UpdateInfo[] CheckAllUpdates()
        {
            int count = check_all_updates_c(out IntPtr updatesPtr);

            if (count < 0)
            {
                throw new Exception($"[MAIDOS-AUDIT] Check All Updates Failed: {GetLastError()}");
            }

            if (count == 0 || updatesPtr == IntPtr.Zero)
            {
                return Array.Empty<UpdateInfo>();
            }

            try
            {
                var results = new UpdateInfo[count];
                for (int i = 0; i < count; i++)
                {
                    results[i] = ReadUpdateFromPtr(updatesPtr, i);
                }
                return results;
            }
            finally
            {
                free_update_info_array(updatesPtr, count);
            }
        }
    }
}
