$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$installerRoot = Join-Path $repoRoot "apps\cliply-installer"
$payloadDir = Join-Path $installerRoot "src-tauri\payload"
$installerIconsDir = Join-Path $installerRoot "src-tauri\icons"
$installerPublicLogo = Join-Path $installerRoot "public\cliply-logo.png"

$sourceExe = Join-Path $repoRoot "src-tauri\target\release\cliply.exe"
$sourceIcon = Join-Path $repoRoot "src-tauri\icons\icon.ico"
$sourceLogo = Join-Path $repoRoot "src\assets\cliply-logo.png"

if (-not (Test-Path $sourceExe)) {
  throw "Cliply release exe not found: $sourceExe. Run `npm run tauri -- build --bundles nsis` first."
}

if (-not (Test-Path $sourceIcon)) {
  throw "Cliply icon not found: $sourceIcon"
}

if (-not (Test-Path $sourceLogo)) {
  throw "Cliply logo not found: $sourceLogo"
}

New-Item -ItemType Directory -Path $payloadDir -Force | Out-Null
New-Item -ItemType Directory -Path $installerIconsDir -Force | Out-Null
New-Item -ItemType Directory -Path (Split-Path $installerPublicLogo -Parent) -Force | Out-Null

$payloadExe = Join-Path $payloadDir "cliply.exe"
$payloadExeGz = Join-Path $payloadDir "cliply.exe.gz"
if (Test-Path $payloadExe) {
  Remove-Item -Path $payloadExe -Force
}

if (Test-Path $payloadExeGz) {
  Remove-Item -Path $payloadExeGz -Force
}

$inputStream = [System.IO.File]::OpenRead($sourceExe)
$outputStream = [System.IO.File]::Create($payloadExeGz)
$gzipStream = $null
try {
  $gzipStream = [System.IO.Compression.GzipStream]::new($outputStream, [System.IO.Compression.CompressionMode]::Compress)
  $inputStream.CopyTo($gzipStream)
} finally {
  if ($gzipStream) {
    $gzipStream.Dispose()
  }
  $outputStream.Dispose()
  $inputStream.Dispose()
}

Copy-Item -Path $sourceIcon -Destination (Join-Path $payloadDir "cliply.ico") -Force
Copy-Item -Path $sourceIcon -Destination (Join-Path $installerIconsDir "icon.ico") -Force
Copy-Item -Path $sourceLogo -Destination $installerPublicLogo -Force

$iconPngs = @("32x32.png", "128x128.png", "128x128@2x.png")
foreach ($iconPng in $iconPngs) {
  $source = Join-Path $repoRoot "src-tauri\icons\$iconPng"
  if (Test-Path $source) {
    Copy-Item -Path $source -Destination (Join-Path $installerIconsDir $iconPng) -Force
  }
}

Get-ChildItem -Path $payloadDir, $installerIconsDir, (Split-Path $installerPublicLogo -Parent) |
  Select-Object FullName, Length, LastWriteTime
