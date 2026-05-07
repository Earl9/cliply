!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Preparing Cliply install directory..."

  !if "${INSTALLMODE}" == "perMachine"
    SetShellVarContext all
  !endif

  ExecWait '"$SYSDIR\taskkill.exe" /F /T /IM "${MAINBINARYNAME}.exe"'
  Sleep 800

  nsis_tauri_utils::KillProcess "${MAINBINARYNAME}.exe"
  Pop $R0
  Sleep 1200

  nsis_tauri_utils::FindProcess "${MAINBINARYNAME}.exe"
  Pop $R0
  ${If} $R0 = 0
    MessageBox MB_ICONSTOP "Cliply 仍在运行，无法覆盖安装。$\n$\n请从托盘退出 Cliply，或在任务管理器中结束 ${MAINBINARYNAME}.exe 后重试。"
    Abort
  ${EndIf}

  RMDir /r "$INSTDIR\cliply.exe.WebView2"
  Delete /REBOOTOK "$INSTDIR\${MAINBINARYNAME}.exe"
  Delete /REBOOTOK "$INSTDIR\*.rbf"

  ClearErrors
  CreateDirectory "$INSTDIR"
  FileOpen $R1 "$INSTDIR\.cliply_write_test" w
  ${If} ${Errors}
    MessageBox MB_ICONSTOP "无法写入安装目录：$INSTDIR$\n$\n请确认安装器已用管理员权限运行，或换到你有写入权限的目录。"
    Abort
  ${EndIf}
  FileClose $R1
  Delete "$INSTDIR\.cliply_write_test"
!macroend
