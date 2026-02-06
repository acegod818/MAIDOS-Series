@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  MAIDOS-IME Proof Pack Generation
echo ============================================

echo [1/4] Running build verification...
call "%~dp0build.bat"

echo [2/4] Running unit tests...
call "%~dp0unit.bat"

echo [3/4] Collecting evidence...
powershell -ExecutionPolicy Bypass -File "%~dp0run_evidence.ps1"

echo [4/4] Generating manifest...
python "%~dp0gen_manifest.py"

echo ============================================
echo  Proof Pack generation complete
echo ============================================
