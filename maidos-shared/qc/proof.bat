@echo off
echo ==========================================
echo  maidos-shared  Proof Pack Generator
echo ==========================================
echo.
set NONCE=%RANDOM%%RANDOM%%RANDOM%
echo Nonce: %NONCE%
echo.
echo === G1: Build ===
call qc\build.bat
if %ERRORLEVEL% NEQ 0 (echo PROOF FAILED at G1 & exit /b 1)
echo.
echo === G2: Unit ===
call qc\unit.bat
if %ERRORLEVEL% NEQ 0 (echo PROOF FAILED at G2 & exit /b 1)
echo.
echo === G3: Integration ===
call qc\integration.bat
if %ERRORLEVEL% NEQ 0 (echo PROOF FAILED at G3 & exit /b 1)
echo.
echo === G4: E2E ===
call qc\e2e.bat
if %ERRORLEVEL% NEQ 0 (echo PROOF FAILED at G4 & exit /b 1)
echo.
echo ==========================================
echo  ALL GATES PASSED
echo  Nonce: %NONCE%
echo  Timestamp: %DATE% %TIME%
echo ==========================================
