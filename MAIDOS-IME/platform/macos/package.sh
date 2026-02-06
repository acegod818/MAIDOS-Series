#!/bin/bash
# MAIDOS-IME macOS 打包腳本
# 作者：諸葛臧草
# 組織：MAIDOS Org.

echo "MAIDOS-IME macOS 打包腳本"
echo "此腳本將建立 macOS 應用程式包"

# 建立打包目錄
mkdir -p dist/macos/MAIDOS-IME.app/Contents/MacOS
mkdir -p dist/macos/MAIDOS-IME.app/Contents/Resources
mkdir -p dist/macos/MAIDOS-IME.app/Contents/Docs

# 複製文件
cp target/release/maidos-core-demo dist/macos/MAIDOS-IME.app/Contents/MacOS/maidos-ime
cp README.md dist/macos/MAIDOS-IME.app/Contents/Docs/
cp LICENSE dist/macos/MAIDOS-IME.app/Contents/Docs/
cp docs/user_manual.md dist/macos/MAIDOS-IME.app/Contents/Docs/
cp docs/quick_start_guide.md dist/macos/MAIDOS-IME.app/Contents/Docs/

# 建立 Info.plist
cat > dist/macos/MAIDOS-IME.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>MAIDOS-IME</string>
    <key>CFBundleIdentifier</key>
    <string>org.maidos.inputmethod</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleExecutable</key>
    <string>maidos-ime</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
</dict>
</plist>
EOF

# 建立 DMG 映像檔
hdiutil create -volname "MAIDOS-IME" -srcfolder dist/macos/MAIDOS-IME.app -ov -format UDZO dist/maidos-ime-macos.dmg

echo "macOS DMG 映像檔已建立: dist/maidos-ime-macos.dmg"