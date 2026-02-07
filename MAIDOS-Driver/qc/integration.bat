@echo off
echo [G3] Integration Gate - MAIDOS-Driver
echo =======================================
echo.
echo [1/4] Check WMI module
findstr /s /m "wmi" src\*.rs >nul
if %ERRORLEVEL% NEQ 0 (echo FAIL: WMI module missing & exit /b 1)
echo PASS: WMI module present
echo.
echo [2/4] Check SetupDI module
findstr /s /m "SetupDi" src\*.rs >nul
if %ERRORLEVEL% NEQ 0 (echo FAIL: SetupDI module missing & exit /b 1)
echo PASS: SetupDI module present
echo.
echo [3/4] Check FFI exports
findstr /s /m "extern \"C\"" src\*.rs >nul
if %ERRORLEVEL% NEQ 0 (echo FAIL: FFI exports missing & exit /b 1)
echo PASS: FFI exports present
echo.
echo [4/4] Check driver database
findstr /s /m "drivers.tsv" src\*.rs >nul
if %ERRORLEVEL% NEQ 0 (echo FAIL: TSV database reference missing & exit /b 1)
echo PASS: TSV database referenced
echo.
echo =======================================
echo G3 INTEGRATION GATE: ALL PASS
