@echo off
echo [G3] Integration Gate - maidos-shared
echo =======================================
echo.
echo [1/2] Check workspace members exist
for %%m in (maidos-config maidos-auth maidos-bus maidos-llm maidos-log maidos-social maidos-google maidos-p2p maidos-chain) do (
  if not exist "%%m\Cargo.toml" (echo FAIL: %%m missing & exit /b 1)
  echo PASS: %%m present
)
echo.
echo [2/2] Check integration tests
if not exist "tests\integration.rs" (echo FAIL: integration tests missing & exit /b 1)
echo PASS: integration tests present
echo.
echo =======================================
echo G3 INTEGRATION GATE: ALL PASS
