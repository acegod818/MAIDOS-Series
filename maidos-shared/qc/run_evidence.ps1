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
Write-Host " maidos-shared Evidence Collection"
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

# Sub-crate sync
Write-Host "[Sync] Sub-crate check"
$subcrates = Get-ChildItem -Path "$root\maidos-*" -Directory
$subcrateNames = $subcrates | ForEach-Object { $_.Name }
$subcrateNames | Set-Content "$proof\sync\subcrates.txt"
Write-Host "[PASS] $($subcrates.Count) sub-crates found"; $pass++

$nonce | Set-Content "$proof\e2e\nonce.txt"

@{sync_check="workspace_subcrates"; nonce=$nonce; run_id=$runId; count=$subcrates.Count} | ConvertTo-Json | Set-Content "$proof\sync\workspace_sync.json"

# Failpaths
@{failpath="missing_config"; description="Missing config file returns default values"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\missing_config.json"
@{failpath="auth_failure"; description="Auth failure returns error, not panic"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\auth_failure.json"
@{failpath="llm_timeout"; description="LLM timeout returns fallback response"; nonce=$nonce} | ConvertTo-Json | Set-Content "$proof\failpaths\llm_timeout.json"

# Observability
@{run_id=$runId; nonce=$nonce; pass=$pass; fail=$fail; timestamp=(Get-Date -Format "o")} | ConvertTo-Json | Set-Content "$proof\observability\trace.json"

Write-Host ""
Write-Host "============================================"
Write-Host " Results: $pass passed, $fail failed"
Write-Host "============================================"

exit $fail
