@echo off
setlocal
set ROOT=%~dp0..
echo ============================================
echo  MAIDOS-Driver Build Verification
echo ============================================

echo [1/3] cargo build...
cd /d "%ROOT%"
cargo build --release 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] cargo build & exit /b 1)
echo [PASS] cargo build

echo [2/3] cargo clippy...
cargo clippy --all-targets -- -D warnings 2>&1
if %ERRORLEVEL% NEQ 0 (echo [FAIL] cargo clippy & exit /b 1)
echo [PASS] cargo clippy

echo [3/3] dotnet build (if available)...
if exist "%ROOT%\src\MAIDOS.Driver.App\MAIDOS.Driver.App.csproj" (
    dotnet build "%ROOT%\src\MAIDOS.Driver.App\MAIDOS.Driver.App.csproj" --nologo -v q 2>&1
    if %ERRORLEVEL% NEQ 0 (echo [WARN] dotnet build failed) else (echo [PASS] dotnet build)
) else (
    echo [SKIP] No C# project found
)

echo ============================================
echo  Build verification complete
echo ============================================
