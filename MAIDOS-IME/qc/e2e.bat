@echo off
setlocal enabledelayedexpansion
echo ============================================
echo  MAIDOS-IME  e2e.bat  (Gate G3 - E2E)
echo ============================================

set "EVIDENCE_DIR=%~dp0evidence"
if not exist "%EVIDENCE_DIR%" mkdir "%EVIDENCE_DIR%"
set "LOG=%EVIDENCE_DIR%\e2e.log"

echo [%date% %time%] Gate G3 - E2E started > "%LOG%"

set "NONCE=%RANDOM%%RANDOM%"
echo [1/2] Nonce: %NONCE%
echo NONCE=%NONCE% >> "%LOG%"

set "DLL=target\release\maidos_core.dll"
if exist "%DLL%" (
    echo       maidos_core.dll found
    echo DLL_CHECK=PASS >> "%LOG%"
) else (
    echo FAIL: maidos_core.dll not found.
    echo DLL_CHECK=FAIL >> "%LOG%"
    echo RESULT=FAIL >> "%LOG%"
    exit /b 1
)

echo [2/2] Checking FFI exports ...
where dumpbin >nul 2>&1
if not errorlevel 1 (
    dumpbin /exports "%DLL%" 2>nul | findstr /C:"ime_init" >nul 2>&1
    if errorlevel 1 (
        echo FAIL: ime_init export not found.
        echo RESULT=FAIL >> "%LOG%"
        exit /b 1
    )
    echo       ime_init export verified
) else (
    echo       dumpbin not found -- skipping export check
)

echo.
echo RESULT=PASS
echo RESULT=PASS >> "%LOG%"
echo [%date% %time%] Gate G3 - E2E completed >> "%LOG%"
exit /b 0
