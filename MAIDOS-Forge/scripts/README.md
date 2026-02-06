# 構建和部署腳本

這個目錄包含各種自動化腳本，用於構建、測試和部署MAIDOS-Forge。

## 腳本列表

### 構建腳本
- `build.ps1` - Windows PowerShell構建腳本
- `build.sh` - Unix/Linux/macOS構建腳本
- `cross-build.ps1` - 交叉編譯構建腳本

### 測試腳本
- `test.ps1` - Windows PowerShell測試腳本
- `test.sh` - Unix/Linux/macOS測試腳本
- `integration-test.ps1` - 集成測試腳本

### 部署腳本
- `deploy.ps1` - Windows PowerShell部署腳本
- `deploy.sh` - Unix/Linux/macOS部署腳本
- `publish-nuget.ps1` - NuGet包發布腳本

### 開發工具
- `setup-dev.ps1` - 開發環境設置腳本
- `update-dependencies.ps1` - 依賴更新腳本
- `generate-docs.ps1` - 文檔生成腳本

## 使用方法

```powershell
# 在Windows上構建
.\scripts\build.ps1

# 在Linux/macOS上構建
./scripts/build.sh

# 運行測試
.\scripts\test.ps1