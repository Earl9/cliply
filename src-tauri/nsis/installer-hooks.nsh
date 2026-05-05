!macro NSIS_HOOK_PREINSTALL
  DetailPrint "正在关闭后台运行的 Cliply..."
  ExecWait '"$SYSDIR\taskkill.exe" /F /T /IM cliply.exe' $0
  Sleep 1000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "正在关闭后台运行的 Cliply..."
  ExecWait '"$SYSDIR\taskkill.exe" /F /T /IM cliply.exe' $0
  Sleep 1000
!macroend
