#!/bin/bash
# MAIDOS-IME iOS 打包腳本
# 作者：諸葛臧草
# 組織：MAIDOS Org.

echo "MAIDOS-IME iOS 打包腳本"
echo "此腳本將建立 iOS 應用程式包"

# 建立打包目錄
mkdir -p dist/ios/Payload
mkdir -p dist/ios/Documents

# 複製文件
cp target/release/maidos-core-demo dist/ios/Payload/maidos-ime
cp README.md dist/ios/Documents/
cp LICENSE dist/ios/Documents/
cp docs/user_manual.md dist/ios/Documents/
cp docs/quick_start_guide.md dist/ios/Documents/

# 建立 Info.plist 範本
cat > dist/ios/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>MAIDOS-IME</string>
    <key>CFBundleIdentifier</key>
    <string>org.maidos.ime</string>
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
    <key>LSRequiresIPhoneOS</key>
    <true/>
    <key>MinimumOSVersion</key>
    <string>11.0</string>
</dict>
</plist>
EOF

# 建立 IPA 檔案
echo "建立 iOS IPA 檔案..."
echo "注意：這需要 Xcode 和 iOS SDK 來完成實際的 IPA 建立"

# 這裡只是一個示範，實際建立 IPA 需要更多步驟和工具
touch dist/maidos-ime-ios.ipa

echo "iOS IPA 範本已建立: dist/maidos-ime-ios.ipa"