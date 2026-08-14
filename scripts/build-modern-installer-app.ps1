$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$installerRoot = Join-Path $repoRoot "apps\cliply-installer"

Push-Location $repoRoot
try {
  node (Join-Path $repoRoot "scripts\generate-app-icon.mjs")
  if ($LASTEXITCODE -ne 0) {
    throw "Application artwork generation failed with exit code $LASTEXITCODE"
  }
  node (Join-Path $repoRoot "scripts\generate-installer-art.mjs")
  if ($LASTEXITCODE -ne 0) {
    throw "Installer artwork generation failed with exit code $LASTEXITCODE"
  }
  $localBuildConfig = Join-Path $repoRoot "src-tauri\tauri.local-build.conf.json"
  npm run tauri -- build --bundles nsis --config $localBuildConfig
  if ($LASTEXITCODE -ne 0) {
    throw "Tauri build failed with exit code $LASTEXITCODE"
  }
  powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "prepare-modern-installer-payload.ps1")
  if ($LASTEXITCODE -ne 0) {
    throw "Installer payload preparation failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

Push-Location $installerRoot
try {
  npm run tauri -- build --no-bundle
  if ($LASTEXITCODE -ne 0) {
    throw "Installer app build failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

$outFile = Join-Path $installerRoot "src-tauri\target\release\cliply-modern-installer.exe"
Get-Item $outFile | Select-Object FullName, LastWriteTime, Length
