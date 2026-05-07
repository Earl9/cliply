$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$installerRoot = Join-Path $repoRoot "apps\cliply-installer"

Push-Location $repoRoot
try {
  npm run tauri -- build --bundles nsis
  powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "prepare-modern-installer-payload.ps1")
} finally {
  Pop-Location
}

Push-Location $installerRoot
try {
  npm run tauri -- build --no-bundle
} finally {
  Pop-Location
}

$outFile = Join-Path $installerRoot "src-tauri\target\release\cliply-modern-installer.exe"
Get-Item $outFile | Select-Object FullName, LastWriteTime, Length
