@echo off
setlocal enabledelayedexpansion
echo ============================================
echo  MAIDOS-IME  unit.bat  (Gate G1 - Unit)
echo ============================================

set "EVIDENCE_DIR=%~dp0evidence"
if not exist "%EVIDENCE_DIR%" mkdir "%EVIDENCE_DIR%"
set "LOG=%EVIDENCE_DIR%\unit.log"

echo [%date% %time%] Unit tests started > "%LOG%"

echo [1/1] cargo test ...
cargo test --workspace >> "%LOG%" 2>&1
if errorlevel 1 (
    echo FAIL: unit tests failed.
    echo RESULT=FAIL >> "%LOG%"
    exit /b 1
)

echo.
echo RESULT=PASS
echo RESULT=PASS >> "%LOG%"
echo [%date% %time%] Unit tests completed >> "%LOG%"
exit /b 0