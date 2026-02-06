@echo off
REM MAIDOS-Forge QC Unit Test Script
REM CodeQC v3.0 C Unified Entry Point
REM Runs: cargo test (Rust core unit tests)

setlocal enabledelayedexpansion
set "ROOT=%~dp0.."
set "EVIDENCE=%ROOT%\evidence"
set "ERRORS=0"

echo ============================================
echo  MAIDOS-Forge QC Unit Tests
echo  %DATE% %TIME%
echo ============================================

if not exist "%EVIDENCE%" mkdir "%EVIDENCE%"

echo.
echo [1/1] Rust Unit Tests: cargo test
echo -------------------------------------------
pushd "%ROOT%"
cargo test 2>&1 | tee "%EVIDENCE%\unit_rust.log"
if %ERRORLEVEL% neq 0 (
    echo [FAIL] Rust unit tests failed
    set "ERRORS=1"
) else (
    echo [PASS] Rust unit tests passed
)
popd

echo.
echo ============================================
if %ERRORS% equ 0 (
    echo  UNIT TESTS: ALL PASS
) else (
    echo  UNIT TESTS: FAILED
)
echo ============================================
exit /b %ERRORS%
