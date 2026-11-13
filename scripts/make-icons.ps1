# Generates Marsa's PNG and ICO icons from the same shapes as
# assets/icon/marsa-icon.svg. Windows PowerShell 5.1, no external tools.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/make-icons.ps1
#
# The drawn shapes and colours must stay in step with marsa-icon.svg.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$assetDir = Join-Path $root 'assets\icon'
$iconDir = Join-Path $root 'apps\desktop\src-tauri\icons'
New-Item -ItemType Directory -Force -Path $assetDir, $iconDir | Out-Null

$sea1 = [System.Drawing.Color]::FromArgb(255, 14, 116, 144)
$sea2 = [System.Drawing.Color]::FromArgb(255, 15, 23, 42)
$white = [System.Drawing.Color]::White
$clear = [System.Drawing.Color]::Transparent

function New-RoundedPath([single]$x, [single]$y, [single]$w, [single]$h, [single]$r) {
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $r * 2
    $p.AddArc($x, $y, $d, $d, 180, 90)
    $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $p.CloseFigure()
    return $p
}

function Draw-Marsa([System.Drawing.Graphics]$g, [switch]$Monochrome) {
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality

    if (-not $Monochrome) {
        $tile = New-RoundedPath 72 72 880 880 212
        $box = [System.Drawing.RectangleF]::new(72, 72, 880, 880)
        $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($box, $sea1, $sea2, 45.0)
        $g.FillPath($brush, $tile)
        $brush.Dispose()

        $sheenTop = [System.Drawing.Color]::FromArgb(46, 255, 255, 255)
        $sheenBottom = [System.Drawing.Color]::FromArgb(0, 255, 255, 255)
        $sheen = [System.Drawing.Drawing2D.LinearGradientBrush]::new($box, $sheenTop, $sheenBottom, 90.0)
        $g.FillPath($sheen, $tile)
        $sheen.Dispose()

        $edge = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(31, 255, 255, 255), 8)
        $g.DrawPath($edge, $tile)
        $edge.Dispose()
        $tile.Dispose()
    }

    $ink = if ($Monochrome) { [System.Drawing.Color]::Black } else { $white }
    $anchor = New-Object System.Drawing.Pen($ink, 54)
    $anchor.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $anchor.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $anchor.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    # Ring, shaft, crossbar.
    $g.DrawEllipse($anchor, 440, 236, 144, 144)
    $g.DrawLine($anchor, 512, 380, 512, 748)
    $g.DrawLine($anchor, 382, 468, 642, 468)

    # The two arms, one U-shaped curve.
    $arms = New-Object System.Drawing.Drawing2D.GraphicsPath
    $arms.AddBezier(322, 586, 322, 696, 405, 768, 512, 768)
    $arms.AddBezier(512, 768, 619, 768, 702, 696, 702, 586)
    $g.DrawPath($anchor, $arms)
    $arms.Dispose()
    $anchor.Dispose()

    $wave = New-Object System.Drawing.Pen($ink, 30)
    $wave.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $wave.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $wave.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    if (-not $Monochrome) { $wave.Color = [System.Drawing.Color]::FromArgb(158, 255, 255, 255) }
    $waves = New-Object System.Drawing.Drawing2D.GraphicsPath
    $waves.AddBezier(300, 856, 376, 820, 448, 892, 524, 856)
    $waves.AddBezier(524, 856, 600, 820, 672, 892, 748, 856)
    $g.DrawPath($wave, $waves)
    $waves.Dispose()
    $wave.Dispose()
}

function New-Png([int]$Size, [string]$Path, [switch]$Monochrome) {
    $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear($clear)
    $g.ScaleTransform(($Size / 1024.0), ($Size / 1024.0))
    Draw-Marsa $g -Monochrome:$Monochrome
    $g.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

function Get-PngBytes([int]$Size) {
    $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear($clear)
    $g.ScaleTransform(($Size / 1024.0), ($Size / 1024.0))
    Draw-Marsa $g
    $g.Dispose()
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    $bytes = $ms.ToArray()
    $ms.Dispose()
    return ,$bytes
}

# The source pictures.
New-Png 1024 (Join-Path $assetDir 'marsa-icon-1024.png')
New-Png 256 (Join-Path $iconDir '128x128@2x.png')
foreach ($pair in @(
    @(16, '16x16.png'), @(24, '24x24.png'), @(30, 'Square30x30Logo.png'),
    @(32, '32x32.png'), @(44, 'Square44x44Logo.png'), @(48, '48x48.png'),
    @(50, 'StoreLogo.png'), @(64, '64x64.png'), @(71, 'Square71x71Logo.png'),
    @(89, 'Square89x89Logo.png'), @(107, 'Square107x107Logo.png'),
    @(128, '128x128.png'), @(142, 'Square142x142Logo.png'),
    @(150, 'Square150x150Logo.png'), @(256, 'Square284x284Logo.png'),
    @(310, 'Square310x310Logo.png'), @(512, 'icon.png')
)) {
    New-Png $pair[0] (Join-Path $iconDir $pair[1])
}

# The macOS tray template, in black on transparency.
$tray = New-Object System.Drawing.Bitmap(44, 44, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$tg = [System.Drawing.Graphics]::FromImage($tray)
$tg.Clear($clear)
$tg.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$tg.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
$scale = 40.0 / 700.0
$left = (44.0 - 564.0 * $scale) / 2.0
$top = (44.0 - 40.0) / 2.0
$tg.Transform = [System.Drawing.Drawing2D.Matrix]::new($scale, 0, 0, $scale, ($left - 230.0 * $scale), ($top - 190.0 * $scale))
$blackPen = New-Object System.Drawing.Pen([System.Drawing.Color]::Black, 54)
$blackPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$blackPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$blackPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$tg.DrawEllipse($blackPen, 440, 236, 144, 144)
$tg.DrawLine($blackPen, 512, 380, 512, 748)
$tg.DrawLine($blackPen, 382, 468, 642, 468)
$arms = New-Object System.Drawing.Drawing2D.GraphicsPath
$arms.AddBezier(322, 586, 322, 696, 405, 768, 512, 768)
$arms.AddBezier(512, 768, 619, 768, 702, 696, 702, 586)
$tg.DrawPath($blackPen, $arms)
$arms.Dispose()
$blackPen.Dispose()
$blackWave = New-Object System.Drawing.Pen([System.Drawing.Color]::Black, 30)
$blackWave.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$blackWave.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$blackWave.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$waves = New-Object System.Drawing.Drawing2D.GraphicsPath
$waves.AddBezier(300, 856, 376, 820, 448, 892, 524, 856)
$waves.AddBezier(524, 856, 600, 820, 672, 892, 748, 856)
$tg.DrawPath($blackWave, $waves)
$waves.Dispose()
$blackWave.Dispose()
$tg.Dispose()
$tray.Save((Join-Path $iconDir 'tray-template.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$tray.Dispose()

# The ICO, with PNG-compressed entries (Windows Vista and later).
$icoSizes = @(16, 24, 32, 48, 64, 128, 256)
$images = @()
foreach ($size in $icoSizes) { $images += ,(Get-PngBytes $size) }
$icoPath = Join-Path $iconDir 'icon.ico'
$stream = [System.IO.File]::Create($icoPath)
$writer = New-Object System.IO.BinaryWriter($stream)
$writer.Write([uint16]0)                 # reserved
$writer.Write([uint16]1)                 # type: icon
$writer.Write([uint16]$icoSizes.Count)   # number of images
$offset = 6 + 16 * $icoSizes.Count
for ($i = 0; $i -lt $icoSizes.Count; $i++) {
    $size = $icoSizes[$i]
    $bytes = $images[$i]
    $dim = if ($size -ge 256) { 0 } else { $size }
    $writer.Write([byte]$dim)            # width
    $writer.Write([byte]$dim)            # height
    $writer.Write([byte]0)               # palette colours
    $writer.Write([byte]0)               # reserved
    $writer.Write([uint16]1)             # colour planes
    $writer.Write([uint16]32)            # bits per pixel
    $writer.Write([uint32]$bytes.Length) # bytes in resource
    $writer.Write([uint32]$offset)       # offset
    $offset += $bytes.Length
}
foreach ($bytes in $images) { $writer.Write($bytes) }
$writer.Dispose()
$stream.Dispose()

# macOS is not packaged in the Windows-first release; drop a stale icon if present.
$oldIcns = Join-Path $iconDir 'icon.icns'
if (Test-Path -LiteralPath $oldIcns) { Remove-Item -LiteralPath $oldIcns -Force }

Get-ChildItem -LiteralPath $iconDir -File | Where-Object { $_.Extension -in '.png', '.ico' } |
    Sort-Object Name | ForEach-Object { "{0}  {1} bytes" -f $_.Name, $_.Length }
