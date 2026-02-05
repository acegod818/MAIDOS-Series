@echo off
setlocal EnableExtensions

rem ============================================
rem  MAIDOS CodeQC - Windows Runner
rem  Usage: codeqc.cmd [options] [target]
rem ============================================

set "ROOT=%~dp0"
set "CLI=%ROOT%maidos-codeqc\dist\cli.js"

rem Check if installed
if not exist "%CLI%" (
    echo [ERROR] CodeQC not installed. Run install.cmd first.
    exit /b 1
)

rem Run CodeQC
node "%CLI%" %*

endlocal
