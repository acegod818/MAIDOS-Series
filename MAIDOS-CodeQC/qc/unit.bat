@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  MAIDOS-CodeQC Unit Tests
echo ============================================

cd /d "%ROOT%\maidos-codeqc"
call npx vitest run 2>&1 | tee "%ROOT%\evidence\unit_ts.log"
echo Exit code: %ERRORLEVEL%

echo ============================================
echo  Unit tests complete
echo ============================================
