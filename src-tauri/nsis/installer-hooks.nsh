!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Closing running Cliply process before installing..."
  ExecWait '"$SYSDIR\taskkill.exe" /F /T /IM cliply.exe' $0
  Sleep 1000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Closing running Cliply process before uninstalling..."
  ExecWait '"$SYSDIR\taskkill.exe" /F /T /IM cliply.exe' $0
  Sleep 1000
!macroend
