@echo off
setlocal EnableExtensions

rem ============================================
rem  MAIDOS CodeQC - Windows Installer
rem  One-click dependency installation
rem ============================================

echo.
echo  MAIDOS CodeQC Installer
echo  =======================
echo.

rem Check Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js not found!
    echo.
    echo Please install Node.js 18+ from:
    echo   https://nodejs.org/
    echo.
    pause
    exit /b 1
)

for /f "tokens=*" %%v in ('node -v') do set NODE_VER=%%v
echo [OK] Node.js %NODE_VER% detected

rem Install core package
echo.
echo Installing maidos-codeqc...
cd /d "%~dp0maidos-codeqc"
call npm install --silent
if %errorlevel% neq 0 (
    echo [ERROR] Failed to install maidos-codeqc
    pause
    exit /b 1
)
echo [OK] maidos-codeqc installed

rem Build if needed
if not exist "dist\cli.js" (
    echo Building...
    call npm run build --silent
)

echo.
echo ============================================
echo  Installation Complete!
echo.
echo  Usage:
echo    codeqc.cmd [target]
echo    codeqc.cmd .\src
echo    codeqc.cmd -h
echo ============================================
echo.

endlocal
pause
