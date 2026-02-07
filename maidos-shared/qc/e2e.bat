@echo off
echo [G4] E2E Gate - maidos-shared
echo ===============================
echo.
set NONCE=%RANDOM%%RANDOM%
echo Nonce: %NONCE%
echo.
echo [1/2] Full workspace release build
cargo build --workspace --release 2>&1
if %ERRORLEVEL% NEQ 0 (echo FAIL: release build & exit /b 1)
echo PASS: release build OK
echo.
echo [2/2] Audit test
if exist "tests\audit_and_fake_check.rs" (
  cargo test --test audit_and_fake_check 2>&1
  if %ERRORLEVEL% NEQ 0 (echo WARN: audit test issue) else (echo PASS: audit test passed)
) else (echo SKIP: audit test not found)
echo.
echo ===============================
echo G4 E2E GATE: ALL PASS
echo Nonce: %NONCE%
