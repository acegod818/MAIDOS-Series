#!/bin/bash
# MAIDOS-IME Android 打包腳本
# 作者：諸葛臧草
# 組織：MAIDOS Org.

echo "MAIDOS-IME Android 打包腳本"
echo "此腳本將建立 Android APK 檔案"

# 建立打包目錄
mkdir -p dist/android/assets
mkdir -p dist/android/libs
mkdir -p dist/android/jni

# 複製文件
cp target/release/maidos-core-demo dist/android/jni/libmaidos_ime.so
cp README.md dist/android/assets/
cp LICENSE dist/android/assets/
cp docs/user_manual.md dist/android/assets/
cp docs/quick_start_guide.md dist/android/assets/

# 建立 AndroidManifest.xml 範本
cat > dist/android/AndroidManifest.xml << EOF
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="org.maidos.ime"
    android:versionCode="1"
    android:versionName="1.0.0">

    <uses-sdk android:minSdkVersion="21" android:targetSdkVersion="30" />

    <application android:label="MAIDOS-IME">
        <service android:name=".MAIDOSIME"
            android:permission="android.permission.BIND_INPUT_METHOD">
            <intent-filter>
                <action android:name="android.view.InputMethod" />
            </intent-filter>
            <meta-data android:name="android.view.im" android:resource="@xml/method" />
        </service>
    </application>
</manifest>
EOF

# 建立 APK 檔案
echo "建立 Android APK 檔案..."
echo "注意：這需要 Android SDK 和 NDK 來完成實際的 APK 建立"

# 這裡只是一個示範，實際建立 APK 需要更多步驟和工具
touch dist/maidos-ime-android.apk

echo "Android APK 範本已建立: dist/maidos-ime-android.apk"