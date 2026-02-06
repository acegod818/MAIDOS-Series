$installer = New-Object -ComObject WindowsInstaller.Installer
$db = $installer.OpenDatabase("$PSScriptRoot\MAIDOS-Driver-Setup-0.2.0.msi", 0)

# Count files
$view = $db.OpenView("SELECT COUNT(*) FROM File")
$view.Execute()
$record = $view.Fetch()
Write-Host "MSI contains $($record.IntegerData(1)) files"
$view.Close()

# Check key files
$view = $db.OpenView("SELECT FileName FROM File WHERE FileName LIKE '%MAIDOS%' OR FileName LIKE '%maidOS%'")
$view.Execute()
$record = $view.Fetch()
Write-Host "`nKey files:"
while ($null -ne $record) {
    Write-Host "  $($record.StringData(1))"
    $record = $view.Fetch()
}
$view.Close()

# Product info
$view = $db.OpenView("SELECT Value FROM Property WHERE Property='ProductName'")
$view.Execute()
$record = $view.Fetch()
Write-Host "`nProduct: $($record.StringData(1))"
$view.Close()

$view = $db.OpenView("SELECT Value FROM Property WHERE Property='Manufacturer'")
$view.Execute()
$record = $view.Fetch()
Write-Host "Manufacturer: $($record.StringData(1))"
$view.Close()
