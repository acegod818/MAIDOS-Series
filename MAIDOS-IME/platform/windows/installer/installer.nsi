; MAIDOS-IME 安裝程式腳本
!include "MUI2.nsh"

; 基本資訊
Name "MAIDOS-IME"
OutFile "maidos-ime-installer.exe"
InstallDir "$PROGRAMFILES\MAIDOS-IME"
RequestExecutionLevel admin

; 作者資訊
VIProductVersion "1.0.0.0"
VIAddVersionKey /LANG=1028 "ProductName" "MAIDOS-IME"
VIAddVersionKey /LANG=1028 "CompanyName" "MAIDOS Org."
VIAddVersionKey /LANG=1028 "FileVersion" "1.0.0.0"
VIAddVersionKey /LANG=1028 "FileDescription" "AI 驅動的智慧輸入法"
VIAddVersionKey /LANG=1028 "LegalCopyright" "Copyright (C) 2026 諸葛臥草 (MAIDOS Org.)"

; 界面設定
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; 語言
!insertmacro MUI_LANGUAGE "TradChinese"

; 安裝程序
Section "MAIDOS-IME" SecMain
  ; 設置輸出路徑
  SetOutPath "$INSTDIR"
  
  ; 複製文件
  File "..\..\..\target\release\maidos-core-demo.exe"
  File "..\..\..\README.md"
  File "..\..\..\SPEC-MAIDOS-IME.md"
  File "..\..\..\docs\user_manual.md"
  File "..\..\..\docs\quick_start_guide.md"
  
  ; 創建卸載信息
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MAIDOS-IME" \
                   "DisplayName" "MAIDOS-IME"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MAIDOS-IME" \
                   "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MAIDOS-IME" \
                   "Publisher" "MAIDOS Org."
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MAIDOS-IME" \
                   "HelpLink" "https://github.com/maidos-org/maidos-ime"
                   
  ; 創建卸載程式
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; 開始選單項目
Section "StartMenuShortcuts" SecShortcuts
  CreateDirectory "$SMPROGRAMS\MAIDOS-IME"
  CreateShortCut "$SMPROGRAMS\MAIDOS-IME\卸載.lnk" "$INSTDIR\uninstall.exe"
  CreateShortCut "$SMPROGRAMS\MAIDOS-IME\README.lnk" "notepad.exe" "$INSTDIR\README.md"
  CreateShortCut "$SMPROGRAMS\MAIDOS-IME\User Manual.lnk" "notepad.exe" "$INSTDIR\user_manual.md"
  CreateShortCut "$SMPROGRAMS\MAIDOS-IME\Quick Start Guide.lnk" "notepad.exe" "$INSTDIR\quick_start_guide.md"
  CreateShortCut "$SMPROGRAMS\MAIDOS-IME\MAIDOS-IME Demo.lnk" "$INSTDIR\maidos-core-demo.exe"
SectionEnd

; 桌面捷徑
Section /o "DesktopShortcut" SecDesktop
  CreateShortCut "$DESKTOP\MAIDOS-IME Demo.lnk" "$INSTDIR\maidos-core-demo.exe"
SectionEnd

; 卸載程序
Section "Uninstall"
  ; 刪除文件
  Delete "$INSTDIR\maidos-core-demo.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\SPEC-MAIDOS-IME.md"
  Delete "$INSTDIR\user_manual.md"
  Delete "$INSTDIR\quick_start_guide.md"
  Delete "$INSTDIR\uninstall.exe"
  
  ; 移除目錄
  RMDir "$INSTDIR"
  
  ; 移除開始選單項目
  Delete "$SMPROGRAMS\MAIDOS-IME\卸載.lnk"
  Delete "$SMPROGRAMS\MAIDOS-IME\README.lnk"
  Delete "$SMPROGRAMS\MAIDOS-IME\User Manual.lnk"
  Delete "$SMPROGRAMS\MAIDOS-IME\Quick Start Guide.lnk"
  Delete "$SMPROGRAMS\MAIDOS-IME\MAIDOS-IME Demo.lnk"
  RMDir "$SMPROGRAMS\MAIDOS-IME"
  
  ; 移除桌面捷徑
  Delete "$DESKTOP\MAIDOS-IME Demo.lnk"
  
  ; 移除卸載信息
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\MAIDOS-IME"
SectionEnd

; 描述
LangString DESC_SecMain ${LANG_TRADCHINESE} "MAIDOS-IME 主程序"
LangString DESC_SecShortcuts ${LANG_TRADCHINESE} "開始選單捷徑"
LangString DESC_SecDesktop ${LANG_TRADCHINESE} "桌面捷徑"

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
!insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_SecMain)
!insertmacro MUI_DESCRIPTION_TEXT ${SecShortcuts} $(DESC_SecShortcuts)
!insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END