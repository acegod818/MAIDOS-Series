@echo off
REM MAIDOS-Forge QC Build Script
REM CodeQC v3.0 C Unified Entry Point
REM Runs: cargo build + dotnet build (Core + CLI + 15 Tier A/B Plugins)

setlocal enabledelayedexpansion
set "ROOT=%~dp0.."
set "EVIDENCE=%ROOT%\evidence"
set "ERRORS=0"

echo ============================================
echo  MAIDOS-Forge QC Build
echo  %DATE% %TIME%
echo ============================================

if not exist "%EVIDENCE%" mkdir "%EVIDENCE%"

echo.
echo [1/3] Rust Core: cargo build --release
echo -------------------------------------------
pushd "%ROOT%"
cargo build --release 2>&1 | tee "%EVIDENCE%\build_rust.log"
if %ERRORLEVEL% neq 0 (
    echo [FAIL] Rust build failed
    set "ERRORS=1"
) else (
    echo [PASS] Rust build succeeded
)
popd

echo.
echo [2/3] Rust Clippy: cargo clippy
echo -------------------------------------------
pushd "%ROOT%"
cargo clippy -- -D warnings 2>&1 | tee "%EVIDENCE%\clippy.log"
if %ERRORLEVEL% neq 0 (
    echo [FAIL] Clippy warnings found
    set "ERRORS=1"
) else (
    echo [PASS] Clippy clean
)
popd

echo.
echo [3/3] C# Build: dotnet build (Core + CLI + Plugins)
echo -------------------------------------------
pushd "%ROOT%"
dotnet build src\Forge.Core.New\Forge.Core.New.csproj --nologo -v q 2>&1 | tee "%EVIDENCE%\build_csharp_core.log"
if %ERRORLEVEL% neq 0 (
    echo [FAIL] C# Core build failed
    set "ERRORS=1"
) else (
    echo [PASS] C# Core build succeeded
)

dotnet build src\Forge.Cli\Forge.Cli.csproj --nologo -v q 2>&1 | tee "%EVIDENCE%\build_csharp_cli.log"
if %ERRORLEVEL% neq 0 (
    echo [FAIL] C# CLI build failed
    set "ERRORS=1"
) else (
    echo [PASS] C# CLI build succeeded
)

REM Build Tier A/B plugins
for %%P in (C cpp CSharp Rust Go Python JavaScript TypeScript Java Kotlin Swift Ruby Dart Haskell Elixir) do (
    dotnet build "src\Forge.Plugins\Forge.Plugin.%%P\Forge.Plugin.%%P.csproj" --nologo -v q 2>&1 >> "%EVIDENCE%\build_plugins.log"
    if !ERRORLEVEL! neq 0 (
        echo [FAIL] Plugin %%P build failed
        set "ERRORS=1"
    )
)
echo [INFO] Plugin build log: evidence\build_plugins.log
popd

echo.
echo ============================================
if %ERRORS% equ 0 (
    echo  BUILD: ALL PASS
) else (
    echo  BUILD: FAILED
)
echo ============================================
exit /b %ERRORS%
