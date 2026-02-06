@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  maidos-shared Build Verification
echo ============================================

echo [1/2] cargo build...
cd /d "%ROOT%"
cargo build --release 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] cargo build & exit /b 1)
echo [PASS] cargo build

echo [2/2] cargo clippy...
cargo clippy --all-targets -- -D warnings 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] cargo clippy & exit /b 1)
echo [PASS] cargo clippy

echo ============================================
echo  Build verification complete
echo ============================================
