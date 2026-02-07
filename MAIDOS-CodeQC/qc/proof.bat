@echo off
REM ============================================================
REM MAIDOS-CodeQC -- QC Proof Pack Orchestrator
REM Gate: G4 (Proof Pack Generation)
REM ============================================================
setlocal enabledelayedexpansion

echo ============================================================
echo  MAIDOS-CodeQC QC Proof Pack Orchestrator
echo  %DATE% %TIME%
echo ============================================================

set "EXITCODE=0"
set "PROOF_DIR=proof"

if not exist "%PROOF_DIR%" mkdir "%PROOF_DIR%"

echo.
echo [PROOF-1] Running Build Gate (G2)...
call "%~dp0build.bat" > "%PROOF_DIR%\g2-build.log" 2>&1
if errorlevel 1 (
    echo [FAIL] Build gate failed -- see g2-build.log
    set "EXITCODE=1"
) else (
    echo [PASS] Build gate passed
)

echo.
echo [PROOF-2] Running Unit Test Gate (G3-unit)...
call "%~dp0unit.bat" > "%PROOF_DIR%\g3-unit.log" 2>&1
if errorlevel 1 (
    echo [FAIL] Unit test gate failed -- see g3-unit.log
    set "EXITCODE=1"
) else (
    echo [PASS] Unit test gate passed
)

echo.
echo [PROOF-3] Running Integration Test Gate (G3-integration)...
call "%~dp0integration.bat" > "%PROOF_DIR%\g3-integration.log" 2>&1
if errorlevel 1 (
    echo [FAIL] Integration test gate failed -- see g3-integration.log
    set "EXITCODE=1"
) else (
    echo [PASS] Integration test gate passed
)

echo.
echo [PROOF-4] Running E2E Test Gate (G3-e2e)...
call "%~dp0e2e.bat" > "%PROOF_DIR%\g3-e2e.log" 2>&1
if errorlevel 1 (
    echo [FAIL] E2E test gate failed -- see g3-e2e.log
    set "EXITCODE=1"
) else (
    echo [PASS] E2E test gate passed
)

echo.
echo [PROOF-5] Checking docs compliance (G1)...
set "DOC_COUNT=0"
for %%D in (
    SPEC.md ARCHITECTURE.md DESIGN.md API.md BUILD.md
    TEST.md USAGE.md CHANGELOG.md DEPS.md SECURITY.md
    CONFIG.md PERF.md CI.md
) do (
    if exist "docs\%%D" (
        set /a DOC_COUNT+=1
    ) else (
        echo [FAIL] Missing docs\%%D
        set "EXITCODE=1"
    )
)
echo [INFO] Found !DOC_COUNT! / 13 required docs

echo.
echo [PROOF-6] Generating manifest...
echo { > "%PROOF_DIR%\manifest.json"
echo   "product": "MAIDOS-CodeQC", >> "%PROOF_DIR%\manifest.json"
echo   "version": "v3.0", >> "%PROOF_DIR%\manifest.json"
echo   "timestamp": "%DATE% %TIME%", >> "%PROOF_DIR%\manifest.json"
echo   "gates": { >> "%PROOF_DIR%\manifest.json"
echo     "g1_spec": "checked", >> "%PROOF_DIR%\manifest.json"
echo     "g2_build": "logged", >> "%PROOF_DIR%\manifest.json"
echo     "g3_unit": "logged", >> "%PROOF_DIR%\manifest.json"
echo     "g3_integration": "logged", >> "%PROOF_DIR%\manifest.json"
echo     "g3_e2e": "logged" >> "%PROOF_DIR%\manifest.json"
echo   }, >> "%PROOF_DIR%\manifest.json"
echo   "evidence_files": [ >> "%PROOF_DIR%\manifest.json"
echo     "g2-build.log", >> "%PROOF_DIR%\manifest.json"
echo     "g3-unit.log", >> "%PROOF_DIR%\manifest.json"
echo     "g3-integration.log", >> "%PROOF_DIR%\manifest.json"
echo     "g3-e2e.log" >> "%PROOF_DIR%\manifest.json"
echo   ] >> "%PROOF_DIR%\manifest.json"
echo } >> "%PROOF_DIR%\manifest.json"
echo [PASS] Manifest generated at %PROOF_DIR%\manifest.json

echo.
echo ============================================================
if %EXITCODE%==0 (
    echo  PROOF PACK: ALL GATES PASSED
) else (
    echo  PROOF PACK: ONE OR MORE GATES FAILED
)
echo  Evidence directory: %PROOF_DIR%echo ============================================================

exit /b %EXITCODE%
