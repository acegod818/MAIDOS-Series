@echo off
REM ============================================================
REM MAIDOS-CodeQC -- QC Unit Test Gate Script
REM Gate: G3 (Test Verification - Unit)
REM ============================================================
setlocal enabledelayedexpansion

echo ============================================================
echo  MAIDOS-CodeQC QC Unit Test Gate
echo  %DATE% %TIME%
echo ============================================================

set "EXITCODE=0"

echo.
echo [G3-1] Running vitest unit tests...
call npm test
if errorlevel 1 (
    echo [FAIL] vitest unit tests failed
    set "EXITCODE=1"
    goto :done
)
echo [PASS] All unit tests passed

:done
echo.
echo ============================================================
if %EXITCODE%==0 (
    echo  UNIT TEST GATE: PASSED
) else (
    echo  UNIT TEST GATE: FAILED
)
echo ============================================================

exit /b %EXITCODE%
