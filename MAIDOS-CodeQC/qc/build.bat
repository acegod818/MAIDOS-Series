@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  MAIDOS-CodeQC Build Verification
echo ============================================

echo [1/2] npm install...
cd /d "%ROOT%\maidos-codeqc"
call npm install --no-audit --no-fund 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] npm install & exit /b 1)
echo [PASS] npm install

echo [2/2] npm run build (tsup)...
call npm run build 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] npm build & exit /b 1)
echo [PASS] npm build

echo ============================================
echo  Build verification complete
echo ============================================
