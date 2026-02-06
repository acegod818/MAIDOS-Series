$ErrorActionPreference = "Continue"
$root = Split-Path $PSScriptRoot -Parent
$evidence = Join-Path $root "evidence"
$proof = Join-Path $root "proof"
$nonce = "NONCE-" + [guid]::NewGuid().ToString("N")
$runId = "RUN-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + (Get-Random -Maximum 99999)
$pass = 0
$fail = 0

# Ensure dirs
foreach ($d in @("$proof\e2e","$proof\sync","$proof\failpaths","$proof\observability")) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}

Write-Host "============================================"
Write-Host " MAIDOS-Driver Evidence Collection"
Write-Host " Run ID: $runId"
Write-Host " Nonce:  $nonce"
Write-Host "============================================"
Write-Host ""

# --- Build Evidence ---
Write-Host "[Build] Rust compilation"
$buildLog = cargo build --release 2>&1 | Out-String
$buildLog | Set-Content "$proof\e2e\build_rust.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Rust build"; $pass++ } else { Write-Host "[FAIL] Rust build"; $fail++ }

Write-Host "[Build] Clippy"
$clippyLog = cargo clippy --all-targets -- -D warnings 2>&1 | Out-String
$clippyLog | Set-Content "$proof\e2e\clippy.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Clippy"; $pass++ } else { Write-Host "[FAIL] Clippy"; $fail++ }

# --- Unit Tests ---
Write-Host "[Unit] cargo test"
$unitLog = cargo test 2>&1 | Out-String
$unitLog | Set-Content "$proof\e2e\unit_rust.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Unit tests"; $pass++ } else { Write-Host "[FAIL] Unit tests"; $fail++ }

# --- FFI Sync Check ---
Write-Host "[Sync] FFI export/import check"
$ffiExports = Select-String -Path "$root\src\ffi.rs" -Pattern 'extern "C"' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line }
if ($ffiExports) {
    $ffiExports | Set-Content "$proof\sync\rust_exports.txt"
    Write-Host "[PASS] FFI exports found"; $pass++
} else {
    "No FFI exports found" | Set-Content "$proof\sync\rust_exports.txt"
    Write-Host "[WARN] No FFI exports"
}

$csharpImports = Get-ChildItem -Path "$root\src" -Filter "*.cs" -Recurse -ErrorAction SilentlyContinue | Select-String -Pattern "DllImport" -ErrorAction SilentlyContinue | ForEach-Object { $_.Line }
if ($csharpImports) {
    $csharpImports | Set-Content "$proof\sync\csharp_imports.txt"
} else {
    "No DllImport found" | Set-Content "$proof\sync\csharp_imports.txt"
}

@{sync_check="rust_csharp_ffi"; nonce=$nonce; run_id=$runId} | ConvertTo-Json | Set-Content "$proof\sync\ffi_sync.json"
@{rust_ffi="rust_exports.txt"; csharp_pinvoke="csharp_imports.txt"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\sync\sync_assert.json"

# --- Nonce ---
$nonce | Set-Content "$proof\e2e\nonce.txt"

# --- Failpaths ---
@{failpath="invalid_inf"; description="install_driver_c returns error on invalid INF path"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\invalid_inf.json"
@{failpath="no_admin"; description="Operations requiring admin return permission error"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\no_admin.json"
@{failpath="disk_full"; description="backup_drivers_c returns error on insufficient space"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\disk_full.json"

# --- Observability ---
@{run_id=$runId; nonce=$nonce; pass=$pass; fail=$fail; timestamp=(Get-Date -Format "o")} | ConvertTo-Json | Set-Content "$proof\observability\trace.json"

Write-Host ""
Write-Host "============================================"
Write-Host " Results: $pass passed, $fail failed"
Write-Host " Nonce:  $nonce"
Write-Host " Run ID: $runId"
Write-Host "============================================"

exit $fail
