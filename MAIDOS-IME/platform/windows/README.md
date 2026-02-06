# Windows 平台支援

## 結構

```
windows/
├── maidos_ime.sln          # Visual Studio 解決方案
├── maidos_ime/             # IME 主體
│   ├── dllmain.cpp         # DLL 入口點
│   ├── ime_module.cpp      # IME 模組實作
│   ├── ime_module.h
│   ├── resource.h
│   └── maidos_ime.def      # DLL 導出函數定義
├── installer/              # 安裝程式
│   ├── installer.nsi       # NSIS 腳本
│   └── resources/          # 安裝資源
└── build/                  # 編譯產出
```

## 開發步驟

1. **編譯核心組件**
   ```bash
   cd ../../src/core
   cargo build --release
   ```

2. **建置 IME DLL**
   - 使用 Visual Studio 打開 `maidos_ime.sln`
   - 編譯生成 `maidos_ime.dll`

3. **打包安裝程式**
   - 安裝 NSIS 工具
   - 執行 `makensis installer.nsi` 生成安裝包

## 注意事項

- 需要管理员权限注册 IME
- 確保 Rust 編譯的 .dll 在系統 PATH 中可見
- 測試時建議使用虛擬機避免影響主系統