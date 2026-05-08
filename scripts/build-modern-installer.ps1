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
  npm run tauri -- build --bundles nsis

  $releaseResourcesDir = Join-Path $tauriDir "target\release\resources"
  New-Item -ItemType Directory -Path $releaseResourcesDir -Force | Out-Null
  Copy-Item -Path (Join-Path $tauriDir "icons\icon.ico") `
    -Destination (Join-Path $releaseResourcesDir "icon.ico") `
    -Force

  $scriptContent = Get-Content -Path $scriptPath -Raw -Encoding UTF8
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
