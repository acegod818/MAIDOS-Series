@echo off
REM ============================================================
REM MAIDOS-CodeQC -- QC Integration Test Gate Script
REM Gate: G3 (Test Verification - Integration)
REM ============================================================
setlocal enabledelayedexpansion

echo ============================================================
echo  MAIDOS-CodeQC QC Integration Test Gate
echo  %DATE% %TIME%
echo ============================================================

set "EXITCODE=0"

echo.
echo [INT-1] Checking key modules exist...

for %%M in (
    "src\pipeline"
    "src\gates"
    "src\plugins"
    "src\evidence"
    "src\config"
    "srceporter"
    "web-ui"
) do (
    if exist %%M (
        echo [PASS] Module %%M exists
    ) else (
        echo [FAIL] Module %%M not found
        set "EXITCODE=1"
    )
)

echo.
echo [INT-2] Checking entry points...
if exist "codeqc.cmd" (
    echo [PASS] codeqc.cmd exists
) else (
    echo [FAIL] codeqc.cmd not found
    set "EXITCODE=1"
)

if exist "codeqc.sh" (
    echo [PASS] codeqc.sh exists
) else (
    echo [FAIL] codeqc.sh not found
    set "EXITCODE=1"
)

echo.
echo [INT-3] Checking configuration files...
for %%F in (
    "package.json"
    "tsconfig.json"
    "tsup.config.ts"
    "vitest.config.ts"
) do (
    if exist %%F (
        echo [PASS] %%F exists
    ) else (
        echo [FAIL] %%F not found
        set "EXITCODE=1"
    )
)

echo.
echo [INT-4] Checking plugin directories...
for %%P in (
    "maidos-codeqc-plugin-config"
    "maidos-codeqc-plugin-systems"
    "maidos-codeqc-plugin-web"
    "maidos-codeqc-plugin-dotnet"
) do (
    if exist %%P (
        echo [PASS] Plugin %%P exists
    ) else (
        echo [WARN] Plugin %%P not found (optional)
    )
)

:done
echo.
echo ============================================================
if %EXITCODE%==0 (
    echo  INTEGRATION TEST GATE: PASSED
) else (
    echo  INTEGRATION TEST GATE: FAILED
)
echo ============================================================

exit /b %EXITCODE%
