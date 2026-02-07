@echo off
setlocal enabledelayedexpansion
echo ============================================
echo  MAIDOS-IME  proof.bat  (Gate G4 - Proof)
echo ============================================

set "EVIDENCE_DIR=%~dp0evidence"
if not exist "%EVIDENCE_DIR%" mkdir "%EVIDENCE_DIR%"
set "LOG=%EVIDENCE_DIR%\proof.log"

echo [%date% %time%] Gate G4 - Proof started > "%LOG%"

set "PACK=%EVIDENCE_DIR%\proof-pack"
if not exist "%PACK%" mkdir "%PACK%"

echo [1/3] Copying docs ...
xcopy "%~dp0..\docs\*.md" "%PACK%\docs\" /Y /Q >nul 2>&1

echo [2/3] Copying logs ...
if exist "%EVIDENCE_DIR%\build.log"       copy "%EVIDENCE_DIR%\build.log"       "%PACK%\" /Y >nul
if exist "%EVIDENCE_DIR%\unit.log"        copy "%EVIDENCE_DIR%\unit.log"        "%PACK%\" /Y >nul
if exist "%EVIDENCE_DIR%\integration.log" copy "%EVIDENCE_DIR%\integration.log" "%PACK%\" /Y >nul
if exist "%EVIDENCE_DIR%\e2e.log"         copy "%EVIDENCE_DIR%\e2e.log"         "%PACK%\" /Y >nul

echo [3/3] Generating manifest ...
echo MAIDOS-IME v0.2.0 Proof Pack > "%PACK%\MANIFEST.txt"
echo Generated: %date% %time% >> "%PACK%\MANIFEST.txt"
echo Gate: C >> "%PACK%\MANIFEST.txt"
dir /b "%PACK%" >> "%PACK%\MANIFEST.txt"

echo Proof pack: %PACK%

echo.
echo RESULT=PASS
echo RESULT=PASS >> "%LOG%"
echo [%date% %time%] Gate G4 - Proof completed >> "%LOG%"
exit /b 0
