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

  $accentRect = [System.Drawing.Rectangle]::new(0, 0, 7, $height)
  $accent = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $accentRect,
    [System.Drawing.Color]::FromArgb(126, 87, 255),
    [System.Drawing.Color]::FromArgb(20, 184, 166),
    90
  )
  $graphics.FillRectangle($accent, $accentRect)

  New-CliplyMark -Graphics $graphics -X 24 -Y 13 -Size 32

  $titleFont = [System.Drawing.Font]::new("Segoe UI Semibold", 14, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $subtitleFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 10, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $titleBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(17, 24, 39))
  $subtitleBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(100, 116, 139))

  $bannerSubtitle = ConvertFrom-CodePoints @(0x672C, 0x5730, 0x526A, 0x8D34, 0x677F, 0x7BA1, 0x7406, 0x5668)
  $graphics.DrawString("Cliply", $titleFont, $titleBrush, 68, 12)
  $graphics.DrawString($bannerSubtitle, $subtitleFont, $subtitleBrush, 68, 31)

  $linePen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(226, 232, 240), 1)
  $graphics.DrawLine($linePen, 0, $height - 1, $width, $height - 1)

  Save-Bitmap $bitmap (Join-Path $outDir "banner.bmp")

  $linePen.Dispose()
  $titleBrush.Dispose()
  $subtitleBrush.Dispose()
  $titleFont.Dispose()
  $subtitleFont.Dispose()
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

  $cardPath = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $cardRect = [System.Drawing.Rectangle]::new(192, 42, 246, 188)
  $radius = 18
  $diameter = $radius * 2
  $cardPath.AddArc($cardRect.X, $cardRect.Y, $diameter, $diameter, 180, 90)
  $cardPath.AddArc($cardRect.Right - $diameter, $cardRect.Y, $diameter, $diameter, 270, 90)
  $cardPath.AddArc($cardRect.Right - $diameter, $cardRect.Bottom - $diameter, $diameter, $diameter, 0, 90)
  $cardPath.AddArc($cardRect.X, $cardRect.Bottom - $diameter, $diameter, $diameter, 90, 90)
  $cardPath.CloseFigure()

  $cardBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(248, 255, 255, 255))
  $graphics.FillPath($cardBrush, $cardPath)
  $cardPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(226, 232, 240), 1)
  $graphics.DrawPath($cardPen, $cardPath)

  $titleFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 21, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
  $bodyFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 12, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $smallFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 10, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
  $titleBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(15, 23, 42))
  $bodyBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(71, 85, 105))
  $tagBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(99, 102, 241))
  $tagBack = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(238, 242, 255))

  $installTitle = (ConvertFrom-CodePoints @(0x5B89, 0x88C5)) + " Cliply"
  $installBody = "Windows " + (ConvertFrom-CodePoints @(0x672C, 0x5730, 0x526A, 0x8D34, 0x677F, 0x5386, 0x53F2, 0x5DE5, 0x5177, 0x3002))
  $privacyLine = ConvertFrom-CodePoints @(0x65E0, 0x9700, 0x8D26, 0x53F7, 0x4E0E, 0x4E91, 0x540C, 0x6B65, 0xFF0C, 0x6570, 0x636E, 0x4FDD, 0x7559, 0x5728, 0x672C, 0x673A, 0x3002)

  $graphics.DrawString($installTitle, $titleFont, $titleBrush, 216, 70)
  $graphics.DrawString($installBody, $bodyFont, $bodyBrush, 216, 104)
  $graphics.DrawString($privacyLine, $smallFont, $bodyBrush, 216, 132)

  $pillPath = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $pillRect = [System.Drawing.Rectangle]::new(216, 172, 132, 26)
  $pillRadius = 13
  $pillPath.AddArc($pillRect.X, $pillRect.Y, $pillRadius * 2, $pillRadius * 2, 180, 90)
  $pillPath.AddArc($pillRect.Right - $pillRadius * 2, $pillRect.Y, $pillRadius * 2, $pillRadius * 2, 270, 90)
  $pillPath.AddArc($pillRect.Right - $pillRadius * 2, $pillRect.Bottom - $pillRadius * 2, $pillRadius * 2, $pillRadius * 2, 0, 90)
  $pillPath.AddArc($pillRect.X, $pillRect.Bottom - $pillRadius * 2, $pillRadius * 2, $pillRadius * 2, 90, 90)
  $pillPath.CloseFigure()
  $graphics.FillPath($tagBack, $pillPath)
  $graphics.DrawString("v0.1.0-alpha", $smallFont, $tagBrush, 232, 178)

  $dotBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(20, 184, 166))
  $graphics.FillEllipse($dotBrush, 216, 254, 7, 7)
  $readyLine = (ConvertFrom-CodePoints @(0x51C6, 0x5907, 0x5B89, 0x88C5)) + " Windows MVP"
  $graphics.DrawString($readyLine, $smallFont, $bodyBrush, 230, 250)

  Save-Bitmap $bitmap (Join-Path $outDir "dialog.bmp")

  $dotBrush.Dispose()
  $pillPath.Dispose()
  $tagBack.Dispose()
  $tagBrush.Dispose()
  $titleBrush.Dispose()
  $bodyBrush.Dispose()
  $titleFont.Dispose()
  $bodyFont.Dispose()
  $smallFont.Dispose()
  $cardPen.Dispose()
  $cardBrush.Dispose()
  $cardPath.Dispose()
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
