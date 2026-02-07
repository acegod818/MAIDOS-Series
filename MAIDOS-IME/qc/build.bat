@echo off

setlocal enabledelayedexpansion

echo ============================================

echo  MAIDOS-IME  build.bat  (Gate G1 - Build)

echo ============================================



set "EVIDENCE_DIR=%%~dp0evidence"

if not exist "%%EVIDENCE_DIR%%" mkdir "%%EVIDENCE_DIR%%"

set "LOG=%%EVIDENCE_DIR%%uild.log"



echo [%%date%% %%time%%] Build started > "%%LOG%%"



echo [1/2] cargo build --release ...

cargo build --release >> "%%LOG%%" 2>&1

if errorlevel 1 (

    echo FAIL: cargo build failed.

    echo RESULT=FAIL >> "%%LOG%%"

    exit /b 1

)

echo       cargo build OK



echo [2/2] cargo clippy ...

cargo clippy --all-targets -- -D warnings >> "%%LOG%%" 2>&1

if errorlevel 1 (

    echo FAIL: clippy warnings detected.

    echo RESULT=FAIL >> "%%LOG%%"

    exit /b 1

)

echo       clippy clean



echo.

echo RESULT=PASS

echo RESULT=PASS >> "%%LOG%%"

echo [%%date%% %%time%%] Build completed >> "%%LOG%%"

exit /b 0

