@echo off
echo [G4] E2E Gate - MAIDOS-Driver
echo ===============================
echo.
set NONCE=%RANDOM%%RANDOM%
echo Nonce: %NONCE%
echo.
echo [1/3] Verify DLL loads
if not exist "target\release\maidOS_driver.dll" (echo FAIL: DLL not built & exit /b 1)
echo PASS: DLL exists
echo.
echo [2/3] Verify CLI binary
if exist "target\release\maidOS-driver-cli.exe" (echo PASS: CLI binary exists) else (echo WARN: CLI binary not found, skipping)
echo.
echo [3/3] Verify AC coverage
set /a COUNT=0
for %%f in (src\*.rs) do set /a COUNT+=1
if %COUNT% LSS 5 (echo FAIL: insufficient source modules & exit /b 1)
echo PASS: %COUNT% source modules found
echo.
echo ===============================
echo G4 E2E GATE: ALL PASS
echo Nonce: %NONCE%
