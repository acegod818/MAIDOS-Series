@echo off
REM ============================================================
REM MAIDOS-CodeQC -- QC Build Gate Script
REM Gate: G2 (Build Verification)
REM ============================================================
setlocal enabledelayedexpansion

echo ============================================================
echo  MAIDOS-CodeQC QC Build Gate
echo  %DATE% %TIME%
echo ============================================================

set "EXITCODE=0"

echo.
echo [G2-1] Installing dependencies...
call npm install
if errorlevel 1 (
    echo [FAIL] npm install failed
    set "EXITCODE=1"
    goto :done
)
echo [PASS] npm install succeeded

echo.
echo [G2-2] Building project with tsup...
call npm run build
if errorlevel 1 (
    echo [FAIL] npm run build failed
    set "EXITCODE=1"
    goto :done
)
echo [PASS] npm run build succeeded

echo.
echo [G2-3] Verifying build artifacts...
if exist "dist\index.js" (
    echo [PASS] dist\index.js exists
) else (
    echo [FAIL] dist\index.js not found
    set "EXITCODE=1"
)

:done
echo.
echo ============================================================
if %EXITCODE%==0 (
    echo  BUILD GATE: PASSED
) else (
    echo  BUILD GATE: FAILED
)
echo ============================================================

exit /b %EXITCODE%
