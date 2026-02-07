@echo off
echo [G1] Build Gate - maidos-shared
echo =================================
echo.
echo [1/2] cargo build --workspace
cargo build --workspace
if %ERRORLEVEL% NEQ 0 (echo FAIL: cargo build & exit /b 1)
echo PASS: cargo build
echo.
echo [2/2] cargo clippy --workspace -- -D warnings
cargo clippy --workspace -- -D warnings
if %ERRORLEVEL% NEQ 0 (echo FAIL: clippy & exit /b 1)
echo PASS: clippy 0 warnings
echo.
echo =================================
echo G1 BUILD GATE: ALL PASS
