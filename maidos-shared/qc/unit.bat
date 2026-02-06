@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  maidos-shared Unit Tests
echo ============================================

cd /d "%ROOT%"
cargo test -- --nocapture 2>&1 | tee "%ROOT%\evidence\unit_rust.log"
echo Exit code: %ERRORLEVEL%

echo ============================================
echo  Unit tests complete
echo ============================================
