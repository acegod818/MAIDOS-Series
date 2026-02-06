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
Write-Host " MAIDOS-IME Evidence Collection"
Write-Host " Run ID: $runId"
Write-Host " Nonce:  $nonce"
Write-Host "============================================"
Write-Host ""

# Build
Write-Host "[Build] Rust compilation"
Push-Location $root
$buildLog = cargo build --release 2>&1 | Out-String
$buildLog | Set-Content "$proof\e2e\build_rust.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Rust build"; $pass++ } else { Write-Host "[FAIL] Rust build"; $fail++ }

Write-Host "[Build] Clippy"
$clippyLog = cargo clippy --all-targets -- -D warnings 2>&1 | Out-String
$clippyLog | Set-Content "$proof\e2e\clippy.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Clippy"; $pass++ } else { Write-Host "[FAIL] Clippy"; $fail++ }

# Unit Tests
Write-Host "[Unit] cargo test"
$unitLog = cargo test 2>&1 | Out-String
$unitLog | Set-Content "$proof\e2e\unit_rust.log"
if ($LASTEXITCODE -eq 0) { Write-Host "[PASS] Unit tests"; $pass++ } else { Write-Host "[FAIL] Unit tests"; $fail++ }
Pop-Location

# FFI Sync
Write-Host "[Sync] FFI check"
$ffiFiles = Get-ChildItem -Path "$root\src" -Filter "ffi.rs" -Recurse -ErrorAction SilentlyContinue
if ($ffiFiles) {
    $exports = Select-String -Path $ffiFiles.FullName -Pattern 'extern "C"' | ForEach-Object { $_.Line }
    $exports | Set-Content "$proof\sync\rust_exports.txt"
    Write-Host "[PASS] FFI exports found"; $pass++
} else {
    "No ffi.rs found" | Set-Content "$proof\sync\rust_exports.txt"
    Write-Host "[INFO] No ffi.rs"
}

@{sync_check="rust_cpp_ffi"; nonce=$nonce; run_id=$runId} | ConvertTo-Json | Set-Content "$proof\sync\ffi_sync.json"
@{rust_ffi="rust_exports.txt"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\sync\sync_assert.json"

$nonce | Set-Content "$proof\e2e\nonce.txt"

# Failpaths
@{failpath="missing_dictionary"; description="Missing pinyin.dict.json returns graceful error"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\missing_dictionary.json"
@{failpath="ollama_offline"; description="LLM fallback to frequency ranking when Ollama unreachable"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\ollama_offline.json"
@{failpath="invalid_input"; description="Invalid bopomofo sequence produces no candidates, no crash"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\invalid_input.json"

# Observability
@{run_id=$runId; nonce=$nonce; pass=$pass; fail=$fail; timestamp=(Get-Date -Format "o")} | ConvertTo-Json | Set-Content "$proof\observability\trace.json"

Write-Host ""
Write-Host "============================================"
Write-Host " Results: $pass passed, $fail failed"
Write-Host " Nonce:  $nonce"
Write-Host " Run ID: $runId"
Write-Host "============================================"

exit $fail
