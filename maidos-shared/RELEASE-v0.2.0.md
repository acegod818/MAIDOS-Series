# MAIDOS Shared Core v0.2.0 發布說明書

> 發布日期：2026-01-09  
> 品質等級：SS（零缺陷）

---

## 1. 發布包一覽

| 檔案名稱 | 大小 | 適用對象 | 說明 |
|----------|-----:|----------|------|
| `maidos-shared-v0.2.0-full.zip` | 18 MB | 不確定選哪個 | **推薦**，包含以下全部 |
| `maidos-shared-v0.2.0-linux-x64.zip` | 3.3 MB | Linux C/C++/其他語言 | 預編譯 .so |
| `maidos-shared-v0.2.0-windows-x64.zip` | 5.6 MB | Windows C/C++/其他語言 | 預編譯 .dll |
| `maidos-shared-v0.2.0-source.zip` | 238 KB | Rust 開發者 | 純源碼 |
| `MaidosShared.0.2.0.nupkg` | 8.9 MB | C# .NET 開發者 | NuGet 套件 |

### macOS 版本

macOS 版本需透過以下方式取得：
- **方式 A**：GitHub Actions 自動編譯（推薦）
- **方式 B**：在 Mac 上手動編譯

---

## 2. 各平台安裝方式

### 2.1 Rust 開發者

```bash
# 解壓源碼
unzip maidos-shared-v0.2.0-source.zip
cd maidos-shared-v0.2.0-source

# 方式 A：直接使用
cargo build --release

# 方式 B：作為依賴（Cargo.toml）
```

```toml
[dependencies]
maidos-config = { path = "./maidos-shared-v0.2.0-source/maidos-config" }
maidos-auth = { path = "./maidos-shared-v0.2.0-source/maidos-auth" }
maidos-bus = { path = "./maidos-shared-v0.2.0-source/maidos-bus" }
maidos-llm = { path = "./maidos-shared-v0.2.0-source/maidos-llm" }
```

---

### 2.2 C/C++ 開發者（Linux）

```bash
# 解壓
unzip maidos-shared-v0.2.0-linux-x64.zip

# 目錄結構
maidos-shared-v0.2.0-linux-x64/
├── lib/
│   ├── libmaidos_config.so
│   ├── libmaidos_auth.so
│   ├── libmaidos_bus.so
│   └── libmaidos_llm.so
├── include/
│   └── maidos.h
└── docs/
    └── USAGE.md

# 編譯你的程式
gcc -I./include -L./lib -lmaidos_config -lmaidos_auth your_app.c -o your_app

# 運行（設置庫路徑）
export LD_LIBRARY_PATH=./lib:$LD_LIBRARY_PATH
./your_app
```

---

### 2.3 C/C++ 開發者（Windows）

```powershell
# 解壓
Expand-Archive maidos-shared-v0.2.0-windows-x64.zip

# 目錄結構
maidos-shared-v0.2.0-windows-x64\
├── lib\
│   ├── maidos_config.dll
│   ├── maidos_auth.dll
│   ├── maidos_bus.dll
│   └── maidos_llm.dll
├── include\
│   └── maidos.h
└── docs\
    └── USAGE.md

# 編譯（MSVC）
cl /I.\include your_app.c /link /LIBPATH:.\lib maidos_config.lib

# 運行（DLL 需在 PATH 或同目錄）
copy lib\*.dll .
your_app.exe
```

---

### 2.4 C# .NET 開發者

```bash
# 方式 A：本地安裝 NuGet
dotnet add package MaidosShared --source ./

# 方式 B：放到本地 NuGet 源
nuget add MaidosShared.0.2.0.nupkg -source ~/local-nuget
dotnet add package MaidosShared --source ~/local-nuget
```

**使用範例**：

```csharp
using MaidosShared;

// 配置
var config = MaidosConfig.Load("maidos.toml");

// 認證
var token = MaidosAuth.CreateToken(capabilities, 3600, secret);

// LLM
var response = await MaidosLlm.Complete("openai", "gpt-4", "Hello!");
```

---

### 2.5 macOS 開發者

#### 方式 A：GitHub Actions 自動編譯（推薦）

```bash
# 1. 解壓源碼並推送到 GitHub
unzip maidos-shared-v0.2.0-source.zip
cd maidos-shared-v0.2.0-source
git init
git remote add origin https://github.com/YOUR_ORG/maidos-shared.git
git add .
git commit -m "v0.2.0"
git push -u origin main

# 2. 打標籤觸發 CI
git tag v0.2.0
git push origin v0.2.0

# 3. 等待 CI 完成，到 Releases 頁面下載：
#    - maidos-shared-0.2.0-x86_64-apple-darwin.tar.gz
#    - maidos-shared-0.2.0-aarch64-apple-darwin.tar.gz
```

#### 方式 B：本地編譯

```bash
# 在 Mac 上執行
unzip maidos-shared-v0.2.0-source.zip
cd maidos-shared-v0.2.0-source

# 安裝 Rust（如果沒有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 編譯
cargo build --release

# 產出位置
ls -la target/release/*.dylib
# libmaidos_config.dylib
# libmaidos_auth.dylib
# libmaidos_bus.dylib
# libmaidos_llm.dylib
```

---

## 3. v0.2.0 新功能

### 3.1 新增 5 個 Tier 2 LLM Provider

| Provider | Vision | Tools | Streaming |
|----------|:------:|:-----:|:---------:|
| Mistral | ✅ Pixtral | ✅ | ✅ 即時 |
| Azure OpenAI | ✅ GPT-4o | ✅ | ✅ 即時 |
| Cohere | ❌ | ✅ + RAG | 🔄 模擬 |
| Together AI | ✅ Llama Vision | ✅ | ✅ 即時 |
| Replicate | ✅ LLaVA | ❌ | 🔄 模擬 |

### 3.2 統一串流介面

```rust
// 所有 Provider 統一用法
let stream = provider.complete_stream(&request).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk.delta);
}
```

### 3.3 統一工具格式 (MaidosTool)

```rust
// 定義一次，自動轉換成各家格式
let tool = MaidosTool::new("get_weather")
    .description("查詢天氣")
    .param("city", ToolParameter::string("城市名稱").required());

// 自動轉換
let openai_format = tool.to_openai();
let anthropic_format = tool.to_anthropic();
let google_format = tool.to_google();
```

### 3.4 友善錯誤提示

```rust
match provider.complete(&request).await {
    Err(LlmError::VisionNotSupported { provider, suggestion }) => {
        println!("{}不支援圖片，建議改用：{}", provider, suggestion);
    }
    Err(LlmError::ToolsNotSupported { provider, suggestion }) => {
        println!("{}不支援工具，建議改用：{}", provider, suggestion);
    }
    _ => {}
}
```

---

## 4. 目錄結構說明

### 4.1 源碼包結構

```
maidos-shared-v0.2.0-source/
├── Cargo.toml              # Workspace 根配置
├── README.md               # 專案說明
├── CHANGELOG.md            # 版本變更記錄
├── QUICKSTART.md           # 快速入門
├── LICENSE                 # 授權條款
│
├── maidos-config/          # 配置管理模組
│   ├── src/
│   └── Cargo.toml
│
├── maidos-auth/            # 認證授權模組
│   ├── src/
│   └── Cargo.toml
│
├── maidos-bus/             # 事件總線模組
│   ├── src/
│   └── Cargo.toml
│
├── maidos-llm/             # LLM 統一介面
│   ├── src/
│   │   ├── providers/      # 13 個 Provider 實作
│   │   ├── streaming.rs    # 串流支援
│   │   └── tool.rs         # 工具格式
│   └── Cargo.toml
│
├── bindings/
│   └── csharp/             # C# 綁定
│
├── include/
│   └── maidos.h            # C 頭文件
│
├── docs/
│   └── USAGE.md            # 詳細使用說明
│
├── tests/                  # 整合測試
├── benches/                # 效能測試
├── examples/               # 範例程式
│
├── scripts/
│   └── release.sh          # 發布腳本
│
└── .github/
    └── workflows/
        ├── ci.yml          # CI 測試
        └── release.yml     # 自動發布
```

---

## 5. 驗證安裝

### Rust

```bash
cd maidos-shared-v0.2.0-source
cargo test --workspace
# 預期：309 tests passed
```

### C（Linux）

```c
// test.c
#include "maidos.h"
#include <stdio.h>

int main() {
    MaidosConfigHandle cfg = maidos_config_load("maidos.toml");
    if (cfg) {
        printf("Config loaded!\n");
        maidos_config_free(cfg);
    }
    return 0;
}
```

```bash
gcc -I./include -L./lib test.c -lmaidos_config -o test
LD_LIBRARY_PATH=./lib ./test
# 預期：Config loaded!
```

### C#

```csharp
// Program.cs
using MaidosShared;

Console.WriteLine($"MaidosShared v0.2.0 loaded");
var config = MaidosConfig.Load("maidos.toml");
Console.WriteLine("Config loaded!");
```

---

## 6. 常見問題

### Q: Linux 找不到 .so 文件

```bash
# 設置庫路徑
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH

# 或複製到系統目錄
sudo cp lib/*.so /usr/local/lib/
sudo ldconfig
```

### Q: Windows DLL 找不到

```powershell
# 方式 A：複製 DLL 到執行檔同目錄
copy lib\*.dll .\bin\

# 方式 B：加入 PATH
$env:PATH = "C:\path\to\lib;$env:PATH"
```

### Q: macOS 安全性阻擋

```bash
# 移除隔離屬性
xattr -d com.apple.quarantine lib/*.dylib

# 或在系統偏好設定 > 安全性與隱私 > 允許
```

### Q: Rust 編譯錯誤 "OpenSSL not found"

```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)

# Windows
# 使用 vcpkg 或預編譯版本
```

---

## 7. SHA256 校驗碼

```
d95b71aaee516eed4ef3304228959277b0c8169c11bde276045fecbee7a1a940  maidos-shared-v0.2.0-full.zip
5ef597987c2c370d8db2f4e554dda95e17ed2350b6ba24bbf4087507ccc50d1c  maidos-shared-v0.2.0-linux-x64.zip
5616d01d809c58675791e05cda759a08acb251e812619c6b3cd8ab3a433f3d0b  maidos-shared-v0.2.0-windows-x64.zip
ec38474180e2999632c113bd50308b3d792bedaea00e913e3339dbdc5255b7a0  maidos-shared-v0.2.0-source.zip
d37554756594f52c9c86d1603da95414182bc434e8460965bf13b3affe2c1bb3  MaidosShared.0.2.0.nupkg
```

驗證方式：
```bash
sha256sum -c <<< "5ef597987c2c370d8db2f4e554dda95e17ed2350b6ba24bbf4087507ccc50d1c  maidos-shared-v0.2.0-linux-x64.zip"
```

---

## 8. 技術支援

- 問題回報：GitHub Issues
- 文檔：`docs/USAGE.md`
- 變更記錄：`CHANGELOG.md`

---

*MAIDOS Shared Core v0.2.0*  
*Code-QC v2.2C 驗收通過*  
*© 2026 MAIDOS Project*
