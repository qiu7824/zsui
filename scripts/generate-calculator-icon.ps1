$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ZsuiIconHandle {
    [DllImport("user32.dll")]
    public static extern bool DestroyIcon(IntPtr handle);
}
"@

function New-RoundedPath {
    param([float]$X, [float]$Y, [float]$Width, [float]$Height, [float]$Radius)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $diameter = $Radius * 2
    $path.AddArc($X, $Y, $diameter, $diameter, 180, 90)
    $path.AddArc($X + $Width - $diameter, $Y, $diameter, $diameter, 270, 90)
    $path.AddArc($X + $Width - $diameter, $Y + $Height - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($X, $Y + $Height - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    return $path
}

function New-CalculatorBitmap {
    param([int]$Size)
    $bitmap = New-Object System.Drawing.Bitmap $Size, $Size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.Clear([System.Drawing.Color]::Transparent)
    $scale = $Size / 256.0

    $blue = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 0, 103, 192))
    $body = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 250, 251, 252))
    $display = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 219, 236, 249))
    $key = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 55, 60, 68))
    $accent = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 18, 125, 82))

    $backgroundPath = New-RoundedPath (8*$scale) (8*$scale) (240*$scale) (240*$scale) (44*$scale)
    $bodyPath = New-RoundedPath (48*$scale) (24*$scale) (160*$scale) (208*$scale) (26*$scale)
    $displayPath = New-RoundedPath (68*$scale) (48*$scale) (120*$scale) (46*$scale) (10*$scale)
    $graphics.FillPath($blue, $backgroundPath)
    $graphics.FillPath($body, $bodyPath)
    $graphics.FillPath($display, $displayPath)

    foreach ($row in 0..2) {
        foreach ($column in 0..2) {
            $x = (70 + $column * 42) * $scale
            $y = (116 + $row * 38) * $scale
            $brush = if ($row -eq 2 -and $column -eq 2) { $accent } else { $key }
            $graphics.FillEllipse($brush, $x, $y, 24*$scale, 24*$scale)
        }
    }

    $graphics.Dispose()
    $backgroundPath.Dispose()
    $bodyPath.Dispose()
    $displayPath.Dispose()
    $blue.Dispose()
    $body.Dispose()
    $display.Dispose()
    $key.Dispose()
    $accent.Dispose()
    return $bitmap
}

$workspace = Split-Path -Parent $PSScriptRoot
$output = Join-Path $workspace "assets\calculator"
New-Item -ItemType Directory -Force $output | Out-Null

$png = New-CalculatorBitmap 256
try {
    $png.Save((Join-Path $output "calculator.png"), [System.Drawing.Imaging.ImageFormat]::Png)
}
finally {
    $png.Dispose()
}

$iconBitmap = New-CalculatorBitmap 64
$handle = $iconBitmap.GetHicon()
try {
    $icon = [System.Drawing.Icon]::FromHandle($handle)
    $stream = [System.IO.File]::Create((Join-Path $output "calculator.ico"))
    try {
        $icon.Save($stream)
    }
    finally {
        $stream.Dispose()
        $icon.Dispose()
    }
}
finally {
    [void][ZsuiIconHandle]::DestroyIcon($handle)
    $iconBitmap.Dispose()
}

Write-Host "calculator icon assets: $output"
