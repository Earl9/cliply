$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$tauriDir = Join-Path $repoRoot "src-tauri"
$bundleDir = Join-Path $tauriDir "target\release\bundle\custom"
$scriptPath = Join-Path $tauriDir "installer\cliply-installer.nsi"
$compiledScriptPath = Join-Path $tauriDir "target\release\custom-installer.nsi"
$package = Get-Content -Path (Join-Path $repoRoot "package.json") -Raw | ConvertFrom-Json
$version = $package.version

$makensisCandidates = @(
  (Join-Path $env:LOCALAPPDATA "tauri\NSIS\makensis.exe"),
  (Join-Path $env:LOCALAPPDATA "tauri\NSIS\Bin\makensis.exe"),
  "C:\Program Files (x86)\NSIS\makensis.exe",
  "C:\Program Files\NSIS\makensis.exe"
)

$makensis = $makensisCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $makensis) {
  throw "makensis.exe not found. Build once with Tauri NSIS or install NSIS."
}

New-Item -ItemType Directory -Path $bundleDir -Force | Out-Null

Push-Location $repoRoot
try {
  node (Join-Path $repoRoot "scripts\generate-installer-art.mjs")
  if ($LASTEXITCODE -ne 0) {
    throw "Installer artwork generation failed with exit code $LASTEXITCODE"
  }
  $localBuildConfig = Join-Path $tauriDir "tauri.local-build.conf.json"
  npm run tauri -- build --bundles nsis --config $localBuildConfig
  if ($LASTEXITCODE -ne 0) {
    throw "Tauri build failed with exit code $LASTEXITCODE"
  }

  $releaseResourcesDir = Join-Path $tauriDir "target\release\resources"
  New-Item -ItemType Directory -Path $releaseResourcesDir -Force | Out-Null
  Copy-Item -Path (Join-Path $tauriDir "icons\icon.ico") `
    -Destination (Join-Path $releaseResourcesDir "icon.ico") `
    -Force

  $scriptContent = Get-Content -Path $scriptPath -Raw -Encoding UTF8
  # Keep the custom installer's displayed and output versions in sync with package.json.
  $scriptContent = $scriptContent -replace '(?m)^!define PRODUCT_VERSION "[^"]*"', "!define PRODUCT_VERSION `"$version`""
  $scriptContent = $scriptContent -replace '(?m)^!define OUT_EXE "[^"]*"', "!define OUT_EXE `"$(Join-Path $bundleDir "Cliply_${version}_x64-modern-setup.exe")`""
  $utf8Bom = New-Object System.Text.UTF8Encoding $true
  [System.IO.File]::WriteAllText($compiledScriptPath, $scriptContent, $utf8Bom)

  & $makensis $compiledScriptPath
  if ($LASTEXITCODE -ne 0) {
    throw "makensis failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

$outFile = Join-Path $bundleDir "Cliply_${version}_x64-modern-setup.exe"
Get-Item $outFile | Select-Object FullName, LastWriteTime, Length
