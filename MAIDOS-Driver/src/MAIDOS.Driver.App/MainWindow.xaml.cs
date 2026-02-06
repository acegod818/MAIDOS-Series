using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Windows;
using Microsoft.Win32;
using MAIDOS.Driver.Service;

namespace MAIDOS.Driver.App
{
    public partial class MainWindow : Window
    {
        private HardwareDetectionService _hardwareDetectionService;

        public MainWindow()
        {
            InitializeComponent();
            _hardwareDetectionService = new HardwareDetectionService();
            UpdateStatus("Admin elevation OK. System restore monitor ready.");
        }

        private void ScanButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                UpdateStatus("Scanning all hardware devices...");
                var devices = _hardwareDetectionService.ScanAllDevices();
                DevicesDataGrid.ItemsSource = devices;
                UpdateStatus($"Scan complete: {devices.Length} devices found");
            }
            catch (Exception ex)
            {
                UpdateStatus($"Scan failed: {ex.Message}");
                MessageBox.Show(ex.Message, "Native Error", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void InstallButton_Click(object sender, RoutedEventArgs e)
        {
            OpenFileDialog openFileDialog = new OpenFileDialog();
            openFileDialog.Filter = "Driver INF Files (*.inf)|*.inf";
            if (openFileDialog.ShowDialog() == true)
            {
                try
                {
                    UpdateStatus($"Installing driver: {openFileDialog.FileName}...");
                    bool result = _hardwareDetectionService.InstallDriver(openFileDialog.FileName);
                    if (result)
                    {
                        UpdateStatus("Driver installed successfully!");
                        MessageBox.Show("Driver installed successfully. System restore point updated.",
                            "Success", MessageBoxButton.OK, MessageBoxImage.Information);
                    }
                }
                catch (Exception ex)
                {
                    UpdateStatus($"Install failed: {ex.Message}");
                    MessageBox.Show(ex.Message, "Install Error", MessageBoxButton.OK, MessageBoxImage.Error);
                }
            }
        }

        private void BackupButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                string backupPath = System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "MAIDOS", "DriverBackup");
                UpdateStatus($"Exporting drivers to {backupPath}...");
                var backups = _hardwareDetectionService.BackupDrivers(backupPath);
                if (backups != null && backups.Length > 0)
                {
                    UpdateStatus($"Backup complete: {backups.Length} drivers exported");
                    MessageBox.Show($"Exported {backups.Length} drivers to {backupPath}",
                        "Complete", MessageBoxButton.OK, MessageBoxImage.Information);
                }
                else
                {
                    UpdateStatus("Backup complete, no exportable drivers found");
                }
            }
            catch (Exception ex)
            {
                UpdateStatus($"Backup failed: {ex.Message}");
            }
        }

        private void RollbackButton_Click(object sender, RoutedEventArgs e)
        {
            var result = MessageBox.Show(
                "This will rollback the driver for the selected device.\n\nIf no device is selected, System Restore will be launched.",
                "Driver Rollback", MessageBoxButton.YesNo, MessageBoxImage.Warning);
            if (result == MessageBoxResult.Yes)
            {
                try
                {
                    var selected = DevicesDataGrid.SelectedItem as HardwareDetectionService.DeviceInfo;
                    if (selected != null && !string.IsNullOrEmpty(selected.Id))
                    {
                        UpdateStatus($"Rolling back driver: {selected.Name}...");
                        string backupPath = System.IO.Path.Combine(
                            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                            "MAIDOS", "DriverBackup");
                        bool rollbackResult = _hardwareDetectionService.RollbackDriver(selected.Id, backupPath);
                        if (rollbackResult)
                        {
                            UpdateStatus($"Rollback complete: {selected.Name}");
                            MessageBox.Show($"Driver for {selected.Name} rolled back successfully.",
                                "Rollback Complete", MessageBoxButton.OK, MessageBoxImage.Information);
                        }
                    }
                    else
                    {
                        UpdateStatus("Launching System Restore...");
                        _hardwareDetectionService.LaunchSystemRestore();
                    }
                }
                catch (Exception ex)
                {
                    UpdateStatus($"Rollback failed: {ex.Message}");
                    MessageBox.Show($"Driver rollback failed: {ex.Message}",
                        "Rollback Error", MessageBoxButton.OK, MessageBoxImage.Error);
                }
            }
        }

        private void DiagnoseButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                var devices = DevicesDataGrid.ItemsSource as HardwareDetectionService.DeviceInfo[];
                if (devices == null || devices.Length == 0)
                {
                    MessageBox.Show("Please run Scan first before diagnosing.",
                        "Info", MessageBoxButton.OK, MessageBoxImage.Warning);
                    return;
                }

                UpdateStatus("Running deep diagnostics on all devices...");
                int problemCount = 0;

                foreach (var device in devices)
                {
                    try
                    {
                        var diag = _hardwareDetectionService.DiagnoseDevice(device.Id);
                        device.ProblemDescription = diag.ProblemDescription;
                        device.ResourceInfo = diag.Irq > 0 ? $"IRQ {diag.Irq}" : "No dedicated IRQ";
                        if (diag.ProblemCode != 0) problemCount++;
                    }
                    catch
                    {
                        device.ProblemDescription = "Diagnosis failed";
                    }
                }

                DevicesDataGrid.ItemsSource = null;
                DevicesDataGrid.ItemsSource = devices;

                if (problemCount > 0)
                {
                    UpdateStatus($"Diagnosis complete: {problemCount} device(s) have issues");
                    MessageBox.Show($"Found {problemCount} device(s) with issues. Check the Diagnosis column.",
                        "Diagnosis Result", MessageBoxButton.OK, MessageBoxImage.Warning);
                }
                else
                {
                    UpdateStatus("Diagnosis complete: all devices OK");
                    MessageBox.Show("All devices are healthy.",
                        "Diagnosis Result", MessageBoxButton.OK, MessageBoxImage.Information);
                }
            }
            catch (Exception ex)
            {
                UpdateStatus($"Diagnosis failed: {ex.Message}");
                MessageBox.Show($"Diagnosis error: {ex.Message}",
                    "Error", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void UpdateButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                UpdateStatus("Checking driver updates (Database + Windows Update)...");
                var updates = _hardwareDetectionService.CheckAllUpdates();

                // Separate: direct-download vs WU-only
                var directUpdates = new List<HardwareDetectionService.UpdateInfo>();
                var wuOnlyUpdates = new List<HardwareDetectionService.UpdateInfo>();

                foreach (var u in updates)
                {
                    if (!u.UpdateAvailable) continue;
                    if (!string.IsNullOrEmpty(u.DownloadUrl))
                        directUpdates.Add(u);
                    else
                        wuOnlyUpdates.Add(u);
                }

                int totalAvailable = directUpdates.Count + wuOnlyUpdates.Count;

                if (totalAvailable == 0)
                {
                    UpdateStatus("All drivers are up to date");
                    MessageBox.Show("All drivers are up to date.",
                        "Update Check", MessageBoxButton.OK, MessageBoxImage.Information);
                    return;
                }

                // Build summary
                var summary = new System.Text.StringBuilder();
                if (directUpdates.Count > 0)
                {
                    summary.AppendLine($"[Auto-download] {directUpdates.Count} update(s):");
                    foreach (var u in directUpdates)
                        summary.AppendLine($"  {u.Status}");
                }
                if (wuOnlyUpdates.Count > 0)
                {
                    summary.AppendLine($"[Windows Update] {wuOnlyUpdates.Count} update(s):");
                    foreach (var u in wuOnlyUpdates)
                        summary.AppendLine($"  {u.Status}");
                }

                UpdateStatus($"Found {totalAvailable} update(s): {directUpdates.Count} direct, {wuOnlyUpdates.Count} via WU");

                var confirm = MessageBox.Show(
                    $"Found {totalAvailable} driver update(s):\n\n{summary}\nProceed with download and install?",
                    "Driver Updates", MessageBoxButton.YesNo, MessageBoxImage.Information);

                if (confirm != MessageBoxResult.Yes) return;

                // Auto-download + apply for direct-download updates
                int downloadOk = 0;
                int downloadFail = 0;
                string downloadDir = System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "MAIDOS", "DriverDownloads");
                System.IO.Directory.CreateDirectory(downloadDir);

                foreach (var u in directUpdates)
                {
                    try
                    {
                        string fileName = System.IO.Path.GetFileName(new Uri(u.DownloadUrl).LocalPath);
                        if (string.IsNullOrEmpty(fileName)) fileName = "driver_update.exe";
                        string savePath = System.IO.Path.Combine(downloadDir, fileName);

                        UpdateStatus($"Downloading {u.LatestVersion}...");
                        bool dlResult = _hardwareDetectionService.DownloadUpdate(u.DownloadUrl, savePath);

                        if (dlResult && savePath.EndsWith(".inf", StringComparison.OrdinalIgnoreCase))
                        {
                            UpdateStatus($"Applying {fileName}...");
                            _hardwareDetectionService.ApplyUpdate(savePath, u.DeviceId);
                        }

                        downloadOk++;
                    }
                    catch (Exception ex)
                    {
                        downloadFail++;
                        UpdateStatus($"Download failed: {ex.Message}");
                    }
                }

                // Open Windows Update for WU-only updates
                if (wuOnlyUpdates.Count > 0)
                {
                    Process.Start(new ProcessStartInfo("ms-settings:windowsupdate") { UseShellExecute = true });
                }

                // Final summary
                var finalMsg = new System.Text.StringBuilder();
                if (downloadOk > 0)
                    finalMsg.AppendLine($"Downloaded {downloadOk} driver(s) to:\n{downloadDir}");
                if (downloadFail > 0)
                    finalMsg.AppendLine($"{downloadFail} download(s) failed.");
                if (wuOnlyUpdates.Count > 0)
                    finalMsg.AppendLine($"Windows Update opened for {wuOnlyUpdates.Count} remaining update(s).");

                UpdateStatus($"Update complete: {downloadOk} downloaded, {downloadFail} failed, {wuOnlyUpdates.Count} via WU");
                MessageBox.Show(finalMsg.ToString(), "Update Result", MessageBoxButton.OK, MessageBoxImage.Information);
            }
            catch (Exception ex)
            {
                UpdateStatus($"Update check failed: {ex.Message}");
                MessageBox.Show($"Update check error: {ex.Message}",
                    "Error", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void UpdateStatus(string message)
        {
            StatusTextBlock.Text = $"[{DateTime.Now:HH:mm:ss}] {message}";
        }
    }
}
