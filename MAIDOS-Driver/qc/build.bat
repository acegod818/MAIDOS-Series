@echo off
echo [G1] Build Gate - MAIDOS-Driver
echo ================================
echo.
echo [1/3] cargo build --release
cargo build --release
if %ERRORLEVEL% NEQ 0 (echo FAIL: cargo build & exit /b 1)
echo PASS: cargo build
echo.
echo [2/3] cargo clippy -- -D warnings
cargo clippy -- -D warnings
if %ERRORLEVEL% NEQ 0 (echo FAIL: clippy & exit /b 1)
echo PASS: clippy 0 warnings
echo.
echo [3/3] Verify DLL exists
if not exist "target\release\maidOS_driver.dll" (echo FAIL: DLL not found & exit /b 1)
echo PASS: maidOS_driver.dll exists
echo.
echo ================================
echo G1 BUILD GATE: ALL PASS
