@echo off
setlocal enabledelayedexpansion
echo ============================================
echo  MAIDOS-IME  integration.bat  (Gate G2)
echo ============================================

set "EVIDENCE_DIR=%~dp0evidence"
if not exist "%EVIDENCE_DIR%" mkdir "%EVIDENCE_DIR%"
set "LOG=%EVIDENCE_DIR%\integration.log"

echo [%date% %time%] Gate G2 started > "%LOG%"

echo [1/2] Checking doc artefacts ...
set "DOCS=%~dp0..\docs"
set FAIL=0
for %%F in (PRD.md ARCHITECTURE.md CONTRACT.md NFR.md AC_MATRIX.md ADR.md USER_JOURNEYS.md DEPLOY.md RUNBOOK.md SLO.md STATE_MODEL.md BACKUP_DR.md ALERTS.md) do (
    if not exist "%DOCS%\%%F" (
        echo   MISSING: %%F
        set FAIL=1
    )
)
if \!FAIL\! == 1 (
    echo FAIL: missing docs.
    echo RESULT=FAIL >> "%LOG%"
    exit /b 1
)
echo       all 13 docs present

echo [2/2] Checking footer compliance ...
set FAIL=0
for %%F in (PRD.md ARCHITECTURE.md CONTRACT.md NFR.md AC_MATRIX.md ADR.md USER_JOURNEYS.md DEPLOY.md RUNBOOK.md SLO.md STATE_MODEL.md BACKUP_DR.md ALERTS.md) do (
    findstr /C:"CodeQC Gate C Compliant" "%DOCS%\%%F" >nul 2>&1
    if errorlevel 1 (
        echo   NO FOOTER: %%F
        set FAIL=1
    )
)
if \!FAIL\! == 1 (
    echo FAIL: footer compliance.
    echo RESULT=FAIL >> "%LOG%"
    exit /b 1
)
echo       all footers valid

echo.
echo RESULT=PASS
echo RESULT=PASS >> "%LOG%"
echo [%date% %time%] Gate G2 completed >> "%LOG%"
exit /b 0
