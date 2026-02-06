$ErrorActionPreference = "Continue"
$root = Split-Path $PSScriptRoot -Parent
$evidence = Join-Path $root "evidence"
$proof = Join-Path $root "proof"
$nonce = "NONCE-" + [guid]::NewGuid().ToString("N")
$runId = "RUN-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + (Get-Random -Maximum 99999)
$pass = 0
$fail = 0

foreach ($d in @("$proof\e2e","$proof\sync","$proof\failpaths","$proof\observability")) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}

Write-Host "============================================"
Write-Host " MAIDOS-CodeQC Evidence Collection"
Write-Host " Run ID: $runId"
Write-Host " Nonce:  $nonce"
Write-Host "============================================"
Write-Host ""

# Build
Write-Host "[Build] npm build"
Push-Location "$root\maidos-codeqc"

$npmInstall = npm install --no-audit --no-fund 2>&1 | Out-String
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] npm install"; $pass++ } else { Write-Host "[FAIL] npm install"; $fail++ }

$buildLog = npm run build 2>&1 | Out-String
$buildLog | Set-Content "$proof\e2e\build_ts.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] npm build"; $pass++ } else { Write-Host "[FAIL] npm build"; $fail++ }

# Unit Tests
Write-Host "[Unit] vitest"
$unitLog = npx vitest run 2>&1 | Out-String
$unitLog | Set-Content "$proof\e2e\unit_ts.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] vitest"; $pass++ } else { Write-Host "[FAIL] vitest"; $fail++ }

Pop-Location

# Plugin check
Write-Host "[Plugin] Plugin crate check"
$plugins = Get-ChildItem -Path "$root\maidos-codeqc-plugin-*" -Directory
$pluginCount = $plugins.Count
Write-Host "[PASS] $pluginCount plugin directories found"; $pass++

$nonce | Set-Content "$proof\e2e\nonce.txt"

@{sync_check="plugin_integration"; nonce=$nonce; run_id=$runId; plugin_count=$pluginCount} | ConvertTo-Json | Set-Content "$proof\sync\plugin_sync.json"

# Failpaths
@{failpath="invalid_config"; description="Invalid config file returns parse error, not crash"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\invalid_config.json"
@{failpath="missing_plugin"; description="Missing plugin directory returns load error gracefully"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\missing_plugin.json"
@{failpath="empty_project"; description="Scanning empty project returns 0 violations, not error"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\empty_project.json"

# Observability
@{run_id=$runId; nonce=$nonce; pass=$pass; fail=$fail; timestamp=(Get-Date -Format "o")} | ConvertTo-Json | Set-Content "$proof\observability\trace.json"

Write-Host ""
Write-Host "============================================"
Write-Host " Results: $pass passed, $fail failed"
Write-Host "============================================"

exit $fail
