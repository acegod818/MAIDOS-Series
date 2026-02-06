#!/bin/bash
# MAIDOS-IME Linux 打包腳本
# 作者：諸葛臧草
# 組織：MAIDOS Org.

echo "MAIDOS-IME Linux 打包腳本"
echo "此腳本將建立 Debian/Ubuntu 套件"

# 建立打包目錄
mkdir -p dist/linux/debian/DEBIAN
mkdir -p dist/linux/debian/usr/bin
mkdir -p dist/linux/debian/usr/share/doc/maidos-ime

# 複製文件
cp target/release/maidos-core-demo dist/linux/debian/usr/bin/maidos-ime
cp README.md dist/linux/debian/usr/share/doc/maidos-ime/
cp LICENSE dist/linux/debian/usr/share/doc/maidos-ime/
cp docs/user_manual.md dist/linux/debian/usr/share/doc/maidos-ime/
cp docs/quick_start_guide.md dist/linux/debian/usr/share/doc/maidos-ime/

# 建立控制文件
cat > dist/linux/debian/DEBIAN/control << EOF
Package: maidos-ime
Version: 1.0.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.24)
Maintainer: MAIDOS Org. <support@maidos.org>
Description: AI-driven intelligent input method
 MAIDOS-IME is an AI-driven intelligent input method with the following features:
 .
 - Clean: No ads, no pop-ups, no data collection, no bundling
 - Smart: AI word selection, context understanding, automatic correction
 - Cross-platform: Windows / macOS / Linux / Android / iOS
 - Local-first: AI models run locally to protect your privacy
EOF

# 建立安裝後腳本
cat > dist/linux/debian/DEBIAN/postinst << EOF
#!/bin/bash
echo "MAIDOS-IME 安裝完成"
echo "請重新啟動您的系統或重新登入以啟用輸入法"
EOF

chmod 755 dist/linux/debian/DEBIAN/postinst

# 建立移除腳本
cat > dist/linux/debian/DEBIAN/prerm << EOF
#!/bin/bash
echo "正在移除 MAIDOS-IME..."
EOF

chmod 755 dist/linux/debian/DEBIAN/prerm

# 設定權限
chmod 755 dist/linux/debian/usr/bin/maidos-ime

# 建立 DEB 套件
dpkg-deb --build dist/linux/debian dist/maidos-ime-linux-amd64.deb

echo "Linux DEB 套件已建立: dist/maidos-ime-linux-amd64.deb"