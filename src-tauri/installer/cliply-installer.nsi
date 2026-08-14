Unicode true
ManifestDPIAware true
ManifestDPIAwareness PerMonitorV2
SetCompressor /SOLID lzma

!include MUI2.nsh
!include LogicLib.nsh
!include nsDialogs.nsh
!include x64.nsh
!include FileFunc.nsh
!include WinMessages.nsh

!define PRODUCT_NAME "Cliply"
!define PRODUCT_VERSION "0.4.1-beta.1"
!define PRODUCT_PUBLISHER "Cliply"
!define PRODUCT_EXE "cliply.exe"
!define PRODUCT_ICON "cliply-${PRODUCT_VERSION}.ico"
!define PRODUCT_UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Cliply"
!define PRODUCT_REG_KEY "Software\cliply\Cliply"
!define STARTMENU_FOLDER "Cliply"
!define SOURCE_EXE "E:\Cliply\src-tauri\target\release\cliply.exe"
!define OUT_EXE "E:\Cliply\src-tauri\target\release\bundle\custom\Cliply_0.4.1-beta.1_x64-modern-setup.exe"
!define ICON_FILE "E:\Cliply\src-tauri\icons\icon.ico"
!define LOGO_IMAGE "E:\Cliply\src-tauri\installer\cliply-logo-96.bmp"
!define CLIPLY_PRIMARY "FF6257"
!define CLIPLY_PRIMARY_SOFT "FFF0EF"
!define CLIPLY_BORDER "E5EAEE"
!define CLIPLY_TEXT "21282D"
!define CLIPLY_TEXT_SECONDARY "66737C"
!define CLIPLY_WHITE "FFFFFF"

Name "${PRODUCT_NAME}"
OutFile "${OUT_EXE}"
InstallDir "$PROGRAMFILES64\Cliply"
InstallDirRegKey HKLM "${PRODUCT_REG_KEY}" "InstallDir"
RequestExecutionLevel admin
BrandingText "Cliply"

!define MUI_ICON "${ICON_FILE}"
!define MUI_UNICON "${ICON_FILE}"
!define MUI_ABORTWARNING
!define MUI_BGCOLOR "FFFFFF"
!define MUI_TEXTCOLOR "1F2937"
!define MUI_INSTFILESPAGE_COLORS "/windows"
!define MUI_INSTFILESPAGE_PROGRESSBAR "smooth"
!define MUI_CUSTOMFUNCTION_GUIINIT InstallerGuiInit

Var ExistingInstallDir
Var IsUpdateMode
Var DirectoryPagePathInput
Var DirectoryPageDialog
Var DirectoryPageLogoHandle
Var FinishPageDialog
Var FinishRunCheckbox
Var FinishStartupCheckbox
Var FinishPageLogoHandle
Var FontHero
Var FontTitle
Var FontSubtitle
Var FontBody
Var StopCliplyExitCode
Var StopCliplyOutput

Page custom DirectoryPageCreate DirectoryPageLeave
!define MUI_PAGE_CUSTOMFUNCTION_SHOW InstFilesPageShow
!insertmacro MUI_PAGE_INSTFILES
Page custom FinishPageCreate FinishPageLeave

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

Function .onInit
  SetRegView 64
  SetShellVarContext all

  StrCpy $IsUpdateMode "0"

  ReadRegStr $ExistingInstallDir HKLM "${PRODUCT_REG_KEY}" "InstallDir"
  ${If} $ExistingInstallDir == ""
    ReadRegStr $ExistingInstallDir HKLM "${PRODUCT_REG_KEY}" ""
  ${EndIf}
  ${If} $ExistingInstallDir == ""
    ReadRegStr $ExistingInstallDir HKLM "${PRODUCT_UNINSTALL_KEY}" "InstallLocation"
  ${EndIf}
  ${If} $ExistingInstallDir == ""
    ReadRegStr $ExistingInstallDir HKCU "${PRODUCT_REG_KEY}" "InstallDir"
  ${EndIf}
  ${If} $ExistingInstallDir == ""
    ReadRegStr $ExistingInstallDir HKCU "${PRODUCT_REG_KEY}" ""
  ${EndIf}
  ${If} $ExistingInstallDir == ""
    ReadRegStr $ExistingInstallDir HKCU "${PRODUCT_UNINSTALL_KEY}" "InstallLocation"
  ${EndIf}

  ${If} $ExistingInstallDir != ""
    StrCpy $INSTDIR $ExistingInstallDir
    StrCpy $IsUpdateMode "1"
  ${EndIf}
FunctionEnd

Function InstallerGuiInit
  CreateFont $FontHero "Segoe UI" 20 700
  CreateFont $FontTitle "Segoe UI" 12 700
  CreateFont $FontSubtitle "Segoe UI" 9 400
  CreateFont $FontBody "Segoe UI" 9 400
FunctionEnd

Function StopCliply
  DetailPrint "正在关闭 Cliply…"
  nsExec::ExecToStack `"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -Command "Get-Process -Name cliply -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 1200; if (Get-Process -Name cliply -ErrorAction SilentlyContinue) { Write-Output 'Cliply is still running'; exit 1 } else { exit 0 }"`
  Pop $StopCliplyExitCode
  Pop $StopCliplyOutput
  ${If} $StopCliplyExitCode != 0
    MessageBox MB_ICONSTOP "无法关闭正在运行的 Cliply。$\r$\n$\r$\n请先退出 Cliply 后重试；如问题仍然存在，请重新启动 Windows。"
    Abort
  ${EndIf}
FunctionEnd

Function DirectoryPageCreate
  ShowWindow $mui.Button.Back ${SW_HIDE}
  EnableWindow $mui.Button.Back 0
  ShowWindow $mui.Button.Cancel ${SW_SHOW}
  SendMessage $mui.Button.Cancel ${WM_SETTEXT} 0 "STR:取消"

  ${If} $IsUpdateMode == "1"
    !insertmacro MUI_HEADER_TEXT "更新 Cliply" "将更新程序文件，并保留剪贴板历史记录和应用设置。"
    SendMessage $mui.Button.Next ${WM_SETTEXT} 0 "STR:更新 Cliply"
  ${Else}
    !insertmacro MUI_HEADER_TEXT "安装 Cliply" "剪贴板历史管理工具"
    SendMessage $mui.Button.Next ${WM_SETTEXT} 0 "STR:安装 Cliply"
  ${EndIf}

  nsDialogs::Create 1018
  Pop $DirectoryPageDialog
  ${If} $DirectoryPageDialog == error
    Abort
  ${EndIf}
  SetCtlColors $DirectoryPageDialog "" "${CLIPLY_WHITE}"

  ${NSD_CreateBitmap} 0u 0u 42u 42u ""
  Pop $0
  ${NSD_SetStretchedImage} $0 "${LOGO_IMAGE}" $DirectoryPageLogoHandle

  ${NSD_CreateLabel} 54u 0u 230u 22u "Cliply"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontHero 1
  SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"

  ${NSD_CreateLabel} 55u 27u 230u 12u "剪贴板历史管理工具"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
  SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"

  ${If} $IsUpdateMode == "1"
    ${NSD_CreateLabel} 0u 58u 100% 26u "检测到已安装的 Cliply。本次更新将替换程序文件。"
    Pop $0
    SendMessage $0 ${WM_SETFONT} $FontBody 1
    SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"
    ${NSD_CreateLabel} 0u 86u 100% 18u "剪贴板历史记录和应用设置不会更改。"
    Pop $0
    SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
    SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"
  ${Else}
    ${NSD_CreateLabel} 0u 58u 100% 42u "Cliply 用于保存和检索文本、链接、图片和代码等剪贴板记录。数据默认存储在本机。"
    Pop $0
    SendMessage $0 ${WM_SETFONT} $FontBody 1
    SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"
  ${EndIf}

  ${NSD_CreateLabel} 0u 112u 80u 12u "安装位置"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontTitle 1
  SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"

  ${NSD_CreateLabel} 0u 128u 100% 18u "可保留当前安装位置，也可选择其他目录。"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
  SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"

  ${NSD_CreateText} 0u 151u -68u 16u "$INSTDIR"
  Pop $DirectoryPagePathInput
  SendMessage $DirectoryPagePathInput ${WM_SETFONT} $FontBody 1

  ${NSD_CreateButton} -58u 149u 58u 20u "浏览"
  Pop $0
  ${NSD_OnClick} $0 DirectoryPageBrowse

  ${NSD_CreateLabel} 0u 178u 100% 14u "程序文件约需 13 MB。历史记录和日志存储在用户数据目录中。"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
  SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"

  nsDialogs::Show
FunctionEnd

Function DirectoryPageBrowse
  nsDialogs::SelectFolderDialog "选择 Cliply 安装目录" "$INSTDIR"
  Pop $0
  ${If} $0 != error
    ${NSD_SetText} $DirectoryPagePathInput "$0"
  ${EndIf}
FunctionEnd

Function DirectoryPageLeave
  ${NSD_GetText} $DirectoryPagePathInput $INSTDIR
  ${If} $INSTDIR == ""
    MessageBox MB_ICONEXCLAMATION "请选择安装目录。"
    Abort
  ${EndIf}
FunctionEnd

Function InstFilesPageShow
  SendMessage $mui.Button.Cancel ${WM_SETTEXT} 0 "STR:取消"
  ${If} $IsUpdateMode == "1"
    !insertmacro MUI_HEADER_TEXT "正在更新 Cliply" "正在替换程序文件。剪贴板历史记录和应用设置不会更改。"
  ${Else}
    !insertmacro MUI_HEADER_TEXT "正在安装 Cliply" "正在复制程序文件并创建快捷方式。"
  ${EndIf}
FunctionEnd

Function FinishPageCreate
  ShowWindow $mui.Button.Back ${SW_HIDE}
  EnableWindow $mui.Button.Back 0
  ShowWindow $mui.Button.Cancel ${SW_HIDE}
  EnableWindow $mui.Button.Cancel 0
  SendMessage $mui.Button.Next ${WM_SETTEXT} 0 "STR:完成"
  ${If} $IsUpdateMode == "1"
    !insertmacro MUI_HEADER_TEXT "Cliply 更新完成" "使用 Ctrl + Shift + V 打开剪贴板历史记录。"
  ${Else}
    !insertmacro MUI_HEADER_TEXT "Cliply 安装完成" "使用 Ctrl + Shift + V 打开剪贴板历史记录。"
  ${EndIf}

  nsDialogs::Create 1018
  Pop $FinishPageDialog
  ${If} $FinishPageDialog == error
    Abort
  ${EndIf}
  SetCtlColors $FinishPageDialog "" "${CLIPLY_WHITE}"

  ${NSD_CreateBitmap} 0u 0u 48u 48u ""
  Pop $0
  ${NSD_SetStretchedImage} $0 "${LOGO_IMAGE}" $FinishPageLogoHandle

  ${If} $IsUpdateMode == "1"
    ${NSD_CreateLabel} 60u 2u 230u 24u "Cliply 更新完成"
  ${Else}
    ${NSD_CreateLabel} 60u 2u 230u 24u "Cliply 安装完成"
  ${EndIf}
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontHero 1
  SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"

  ${NSD_CreateLabel} 61u 33u 230u 14u "使用 Ctrl + Shift + V 打开剪贴板历史记录。"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
  SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"

  ${NSD_CreateLabel} 0u 72u 100% 30u "安装位置：$INSTDIR"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontBody 1
  SetCtlColors $0 "${CLIPLY_TEXT}" "${CLIPLY_WHITE}"

  ${NSD_CreateCheckbox} 0u 118u 100% 18u "启动 Cliply"
  Pop $FinishRunCheckbox
  SendMessage $FinishRunCheckbox ${WM_SETFONT} $FontBody 1
  ${NSD_Check} $FinishRunCheckbox

  ${NSD_CreateCheckbox} 0u 144u 100% 18u "开机自动启动"
  Pop $FinishStartupCheckbox
  SendMessage $FinishStartupCheckbox ${WM_SETFONT} $FontBody 1
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}"
  ${If} $0 != ""
    ${NSD_Check} $FinishStartupCheckbox
  ${EndIf}

  ${NSD_CreateLabel} 0u 178u 100% 14u "剪贴板历史记录和应用设置默认存储在本机。"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FontSubtitle 1
  SetCtlColors $0 "${CLIPLY_TEXT_SECONDARY}" "${CLIPLY_WHITE}"

  nsDialogs::Show
FunctionEnd

Function FinishPageLeave
  ${NSD_GetState} $FinishStartupCheckbox $0
  ${If} $0 == ${BST_CHECKED}
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}" '"$INSTDIR\${PRODUCT_EXE}" --minimized'
  ${Else}
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}"
  ${EndIf}

  ${NSD_GetState} $FinishRunCheckbox $0
  ${If} $0 == ${BST_CHECKED}
    Exec '"$INSTDIR\${PRODUCT_EXE}"'
  ${EndIf}
FunctionEnd

Function EnsureWritableInstallDir
  ClearErrors
  CreateDirectory "$INSTDIR"
  FileOpen $0 "$INSTDIR\.cliply_write_test" w
  ${If} ${Errors}
    MessageBox MB_ICONSTOP "无法写入安装目录：$INSTDIR$\r$\n$\r$\n请以管理员身份运行安装程序，或选择具有写入权限的其他目录。"
    Abort
  ${EndIf}
  FileClose $0
  Delete "$INSTDIR\.cliply_write_test"
FunctionEnd

Section "Cliply" SEC_MAIN
  SetShellVarContext all
  SetRegView 64

  Call StopCliply
  Call EnsureWritableInstallDir

  SetOutPath "$INSTDIR"
  RMDir /r "$INSTDIR\cliply.exe.WebView2"
  Delete /REBOOTOK "$INSTDIR\${PRODUCT_EXE}"
  Delete /REBOOTOK "$INSTDIR\cliply*.ico"
  Delete /REBOOTOK "$INSTDIR\*.rbf"

  File "/oname=${PRODUCT_EXE}" "${SOURCE_EXE}"
  File "/oname=${PRODUCT_ICON}" "${ICON_FILE}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  Delete "$DESKTOP\Cliply.lnk"
  Delete "$PROFILE\Desktop\Cliply.lnk"
  Delete "$SMPROGRAMS\${STARTMENU_FOLDER}\Cliply.lnk"
  SetShellVarContext current
  Delete "$DESKTOP\Cliply.lnk"
  Delete "$SMPROGRAMS\Cliply.lnk"
  Delete "$SMPROGRAMS\${STARTMENU_FOLDER}\Cliply.lnk"
  SetShellVarContext all
  CreateDirectory "$SMPROGRAMS\${STARTMENU_FOLDER}"
  CreateShortcut "$SMPROGRAMS\${STARTMENU_FOLDER}\Cliply.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\${PRODUCT_ICON}" 0
  CreateShortcut "$SMPROGRAMS\${STARTMENU_FOLDER}\卸载 Cliply.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
  CreateShortcut "$DESKTOP\Cliply.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\${PRODUCT_ICON}" 0

  WriteRegStr HKLM "${PRODUCT_REG_KEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${PRODUCT_REG_KEY}" "" "$INSTDIR"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${PRODUCT_ICON}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegDWORD HKLM "${PRODUCT_UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${PRODUCT_UNINSTALL_KEY}" "NoRepair" 1

  DeleteRegKey HKCU "${PRODUCT_UNINSTALL_KEY}"
  DeleteRegKey HKCU "${PRODUCT_REG_KEY}"

  ; Ask Explorer to refresh shortcut and taskbar icon caches after an update.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
  IfFileExists "$SYSDIR\ie4uinit.exe" 0 +2
  ExecWait '"$SYSDIR\ie4uinit.exe" -show'
SectionEnd

Section "Uninstall"
  SetShellVarContext all
  SetRegView 64

  Call un.StopCliply

  Delete "$DESKTOP\Cliply.lnk"
  Delete "$PROFILE\Desktop\Cliply.lnk"
  Delete "$SMPROGRAMS\${STARTMENU_FOLDER}\Cliply.lnk"
  Delete "$SMPROGRAMS\${STARTMENU_FOLDER}\卸载 Cliply.lnk"
  RMDir "$SMPROGRAMS\${STARTMENU_FOLDER}"

  Delete "$INSTDIR\${PRODUCT_EXE}"
  Delete "$INSTDIR\${PRODUCT_ICON}"
  Delete "$INSTDIR\cliply-*.ico"
  RMDir /r "$INSTDIR\cliply.exe.WebView2"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKLM "${PRODUCT_UNINSTALL_KEY}"
  DeleteRegKey HKLM "${PRODUCT_REG_KEY}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT_NAME}"
SectionEnd

Function un.onInit
  SetRegView 64
  SetShellVarContext all
FunctionEnd

Function un.StopCliply
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -Command "Stop-Process -Name cliply -Force -ErrorAction SilentlyContinue"'
  Sleep 800
FunctionEnd
