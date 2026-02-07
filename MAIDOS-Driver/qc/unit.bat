@echo off
echo [G2] Unit Test Gate - MAIDOS-Driver
echo ====================================
echo.
echo [1/1] cargo test
cargo test 2>&1
if %ERRORLEVEL% NEQ 0 (echo FAIL: unit tests & exit /b 1)
echo PASS: all unit tests passed
echo.
echo ====================================
echo G2 UNIT GATE: ALL PASS
