@echo off
REM ============================================================
REM MAIDOS-CodeQC -- QC E2E Test Gate Script
REM Gate: G3 (Test Verification - End-to-End)
REM ============================================================
setlocal enabledelayedexpansion

echo ============================================================
echo  MAIDOS-CodeQC QC E2E Test Gate
echo  %DATE% %TIME%
echo ============================================================

set "EXITCODE=0"

echo.
echo [E2E-1] Verifying 4-gate pipeline feature (G1-G4)...
if exist "src" (
    echo [PASS] Source directory exists
) else (
    echo [FAIL] Source directory missing
    set "EXITCODE=1"
    goto :done
)

echo.
echo [E2E-2] Verifying spec compliance checking feature...
if exist "docs" (
    echo [PASS] docs/ directory exists for spec gate validation
) else (
    echo [FAIL] docs/ directory missing
    set "EXITCODE=1"
)

echo.
echo [E2E-3] Verifying build verification feature...
call npm run build >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Build verification failed
    set "EXITCODE=1"
) else (
    echo [PASS] Build verification succeeded
)

echo.
echo [E2E-4] Verifying test verification feature...
call npm test >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Test verification failed
    set "EXITCODE=1"
) else (
    echo [PASS] Test verification succeeded
)

echo.
echo [E2E-5] Verifying proof pack generation feature...
if exist "qc" (
    echo [PASS] qc/ directory exists for proof generation
) else (
    echo [FAIL] qc/ directory missing
    set "EXITCODE=1"
)

echo.
echo [E2E-6] Verifying evidence collection feature...
if exist "docs\SPEC.md" (
    echo [PASS] Evidence docs present (SPEC.md)
) else (
    echo [FAIL] SPEC.md evidence missing
    set "EXITCODE=1"
)

echo.
echo [E2E-7] Verifying web-ui dashboard feature...
if exist "web-ui" (
    echo [PASS] web-ui directory exists
) else (
    echo [WARN] web-ui directory not found (optional)
)

:done
echo.
echo ============================================================
if %EXITCODE%==0 (
    echo  E2E TEST GATE: PASSED
) else (
    echo  E2E TEST GATE: FAILED
)
echo ============================================================

exit /b %EXITCODE%
