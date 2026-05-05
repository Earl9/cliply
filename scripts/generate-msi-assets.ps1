param(
  [string]$OutputDir = "src-tauri\wix"
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$outDir = Join-Path $root $OutputDir
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function ConvertFrom-CodePoints {
  param([int[]]$CodePoints)

  return -join ($CodePoints | ForEach-Object { [char]$_ })
}

function New-CliplyMark {
  param(
    [System.Drawing.Graphics]$Graphics,
    [int]$X,
    [int]$Y,
    [int]$Size
  )

  $radius = [Math]::Max(4, [int]($Size * 0.2))
  $rect = [System.Drawing.Rectangle]::new($X, $Y, $Size, $Size)
  $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $diameter = $radius * 2

  $path.AddArc($rect.X, $rect.Y, $diameter, $diameter, 180, 90)
  $path.AddArc($rect.Right - $diameter, $rect.Y, $diameter, $diameter, 270, 90)
  $path.AddArc($rect.Right - $diameter, $rect.Bottom - $diameter, $diameter, $diameter, 0, 90)
  $path.AddArc($rect.X, $rect.Bottom - $diameter, $diameter, $diameter, 90, 90)
  $path.CloseFigure()

  $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(126, 87, 255),
    [System.Drawing.Color]::FromArgb(20, 184, 166),
    45
  )
  $Graphics.FillPath($brush, $path)

  $clipPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(245, 248, 255), [Math]::Max(2, [int]($Size * 0.09)))
  $clipPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
  $clipPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
  $clipPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

  $innerX = $X + [int]($Size * 0.29)
  $innerY = $Y + [int]($Size * 0.24)
  $innerW = [int]($Size * 0.42)
  $innerH = [int]($Size * 0.52)
  $Graphics.DrawRectangle($clipPen, $innerX, $innerY + [int]($Size * 0.08), $innerW, $innerH)
  $Graphics.DrawLine($clipPen, $innerX + [int]($innerW * 0.25), $innerY, $innerX + [int]($innerW * 0.75), $innerY)
  $Graphics.DrawLine($clipPen, $innerX + [int]($innerW * 0.5), $innerY, $innerX + [int]($innerW * 0.5), $innerY + [int]($Size * 0.12))

  $clipPen.Dispose()
  $brush.Dispose()
  $path.Dispose()
}

function Save-Bitmap {
  param(
    [System.Drawing.Bitmap]$Bitmap,
    [string]$Path
  )

  if (Test-Path $Path) {
    Remove-Item -LiteralPath $Path -Force
  }

  $Bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Bmp)
}

function New-Banner {
  $width = 493
  $height = 58
  $bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit

  $background = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(250, 252, 255))
  $graphics.FillRectangle($background, 0, 0, $width, $height)

  $accentRect = [System.Drawing.Rectangle]::new(0, 0, 5, $height)
  $accent = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $accentRect,
    [System.Drawing.Color]::FromArgb(126, 87, 255),
    [System.Drawing.Color]::FromArgb(20, 184, 166),
    90
  )
  $graphics.FillRectangle($accent, $accentRect)

  $linePen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(226, 232, 240), 1)
  $graphics.DrawLine($linePen, 0, $height - 1, $width, $height - 1)

  Save-Bitmap $bitmap (Join-Path $outDir "banner.bmp")

  $linePen.Dispose()
  $accent.Dispose()
  $background.Dispose()
  $graphics.Dispose()
  $bitmap.Dispose()
}

function New-DialogImage {
  $width = 493
  $height = 312
  $bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit

  $background = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(0, 0, $width, $height),
    [System.Drawing.Color]::FromArgb(252, 253, 255),
    [System.Drawing.Color]::FromArgb(239, 246, 255),
    90
  )
  $graphics.FillRectangle($background, 0, 0, $width, $height)

  $accent = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(0, 0, 150, $height),
    [System.Drawing.Color]::FromArgb(123, 92, 255),
    [System.Drawing.Color]::FromArgb(15, 118, 110),
    90
  )
  $graphics.FillRectangle($accent, 0, 0, 150, $height)

  $soft = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(36, 255, 255, 255))
  $graphics.FillEllipse($soft, -48, -36, 150, 150)
  $graphics.FillEllipse($soft, 74, 198, 132, 132)

  New-CliplyMark -Graphics $graphics -X 42 -Y 42 -Size 66

  $leftTitleFont = [System.Drawing.Font]::new("Segoe UI Semibold", 22, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $leftBodyFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 10, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $whiteBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(250, 255, 255, 255))
  $mutedWhiteBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(210, 255, 255, 255))

  $dialogTagline = ConvertFrom-CodePoints @(0x526A, 0x8D34, 0x677F, 0xFF0C, 0x5E72, 0x51C0, 0x4E00, 0x70B9, 0x3002)
  $graphics.DrawString("Cliply", $leftTitleFont, $whiteBrush, 36, 126)
  $graphics.DrawString($dialogTagline, $leftBodyFont, $mutedWhiteBrush, 38, 158)

  $dividerPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(220, 228, 240), 1)
  $graphics.DrawLine($dividerPen, 150, 0, 150, $height)

  Save-Bitmap $bitmap (Join-Path $outDir "dialog.bmp")

  $dividerPen.Dispose()
  $whiteBrush.Dispose()
  $mutedWhiteBrush.Dispose()
  $leftTitleFont.Dispose()
  $leftBodyFont.Dispose()
  $soft.Dispose()
  $accent.Dispose()
  $background.Dispose()
  $graphics.Dispose()
  $bitmap.Dispose()
}

New-Banner
New-DialogImage

Write-Host "Generated MSI assets:"
Write-Host " - $(Join-Path $outDir "banner.bmp")"
Write-Host " - $(Join-Path $outDir "dialog.bmp")"
