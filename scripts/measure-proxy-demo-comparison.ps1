param(
    [Parameter(Mandatory = $true)]
    [string]$OldExe,
    [Parameter(Mandatory = $true)]
    [string]$NewExe,
    [string]$Output = "target/proxy-demo-comparison",
    [int]$Runs = 5
)

# Compares two builds of examples/zsui_proxy_client.rs on a real Windows
# desktop: process memory at two window sizes, pixel identity of the final
# surface, and on-screen flicker while the demo repaints every 30 ms. A
# deliberately unbuffered Windows Forms window is probed as a negative control.

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;

public static class ZsuiProxyProbe
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X, Y; }

    [StructLayout(LayoutKind.Sequential)]
    struct BITMAPINFOHEADER
    {
        public uint biSize; public int biWidth; public int biHeight;
        public ushort biPlanes; public ushort biBitCount; public uint biCompression;
        public uint biSizeImage; public int biXPelsPerMeter; public int biYPelsPerMeter;
        public uint biClrUsed; public uint biClrImportant;
    }

    delegate bool EnumWindowsProc(IntPtr window, IntPtr parameter);

    [DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc callback, IntPtr parameter);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr window, out RECT rect);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr window, out RECT rect);
    [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr window, ref POINT point);
    [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr window, IntPtr target, uint flags);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr window);
    [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr window, IntPtr dc);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
    [DllImport("gdi32.dll")] static extern IntPtr CreateCompatibleDC(IntPtr dc);
    [DllImport("gdi32.dll")] static extern bool DeleteDC(IntPtr dc);
    [DllImport("gdi32.dll")] static extern IntPtr CreateDIBSection(IntPtr dc, ref BITMAPINFOHEADER header, uint usage, out IntPtr bits, IntPtr section, uint offset);
    [DllImport("gdi32.dll")] static extern IntPtr SelectObject(IntPtr dc, IntPtr value);
    [DllImport("gdi32.dll")] static extern bool DeleteObject(IntPtr value);
    [DllImport("gdi32.dll")] static extern bool BitBlt(IntPtr target, int x, int y, int width, int height, IntPtr source, int sourceX, int sourceY, uint operation);
    [DllImport("gdi32.dll")] static extern bool GdiFlush();

    public sealed class Frame
    {
        public int Width;
        public int Height;
        public byte[] Bgra;
    }

    public static IntPtr WaitForWindow(int processId, int timeoutMs)
    {
        var watch = Stopwatch.StartNew();
        while (watch.ElapsedMilliseconds < timeoutMs)
        {
            IntPtr found = IntPtr.Zero;
            EnumWindows((window, parameter) =>
            {
                uint owner;
                GetWindowThreadProcessId(window, out owner);
                RECT client;
                if (owner == (uint)processId && IsWindowVisible(window)
                    && GetClientRect(window, out client)
                    && client.Right - client.Left > 200 && client.Bottom - client.Top > 200)
                {
                    found = window;
                    return false;
                }
                return true;
            }, IntPtr.Zero);
            if (found != IntPtr.Zero)
            {
                SetForegroundWindow(found);
                return found;
            }
            System.Threading.Thread.Sleep(50);
        }
        return IntPtr.Zero;
    }

    /// "screen" reads the composed desktop, which is what a user sees;
    /// "print" asks DWM for the window's own surface.
    public static Frame Capture(IntPtr window, string mode)
    {
        RECT client;
        GetClientRect(window, out client);
        int width = client.Right - client.Left;
        int height = client.Bottom - client.Top;
        var origin = new POINT();
        ClientToScreen(window, ref origin);
        int captureWidth = width, captureHeight = height, offsetX = 0, offsetY = 0;
        if (mode == "print")
        {
            RECT bounds;
            GetWindowRect(window, out bounds);
            captureWidth = bounds.Right - bounds.Left;
            captureHeight = bounds.Bottom - bounds.Top;
            offsetX = origin.X - bounds.Left;
            offsetY = origin.Y - bounds.Top;
        }
        if (width <= 0 || height <= 0 || offsetX < 0 || offsetY < 0
            || offsetX + width > captureWidth || offsetY + height > captureHeight)
        {
            throw new InvalidOperationException("window geometry cannot be captured");
        }
        IntPtr screen = GetDC(IntPtr.Zero);
        IntPtr memory = CreateCompatibleDC(screen);
        var header = new BITMAPINFOHEADER
        {
            biSize = 40, biWidth = captureWidth, biHeight = -captureHeight, biPlanes = 1, biBitCount = 32
        };
        IntPtr bits;
        IntPtr bitmap = CreateDIBSection(memory, ref header, 0, out bits, IntPtr.Zero, 0);
        IntPtr previous = SelectObject(memory, bitmap);
        bool captured = mode == "print"
            ? PrintWindow(window, memory, 2)
            : BitBlt(memory, 0, 0, width, height, screen, origin.X, origin.Y, 0x00CC0020 | 0x40000000);
        GdiFlush();
        var frame = new Frame { Width = width, Height = height, Bgra = new byte[width * height * 4] };
        if (captured)
        {
            for (int row = 0; row < height; row++)
            {
                Marshal.Copy(
                    IntPtr.Add(bits, ((row + offsetY) * captureWidth + offsetX) * 4),
                    frame.Bgra, row * width * 4, width * 4);
            }
        }
        SelectObject(memory, previous);
        DeleteObject(bitmap);
        DeleteDC(memory);
        ReleaseDC(IntPtr.Zero, screen);
        if (!captured)
        {
            throw new InvalidOperationException(mode + " capture failed");
        }
        return frame;
    }

    static uint Rgb(byte[] pixels, int offset)
    {
        return (uint)(pixels[offset] | pixels[offset + 1] << 8 | pixels[offset + 2] << 16);
    }

    public static uint DominantColor(Frame frame)
    {
        var counts = new Dictionary<uint, int>();
        uint best = 0;
        int bestCount = -1;
        for (int offset = 0; offset < frame.Bgra.Length; offset += 4 * 7)
        {
            uint color = Rgb(frame.Bgra, offset);
            int count;
            counts.TryGetValue(color, out count);
            counts[color] = ++count;
            if (count > bestCount)
            {
                best = color;
                bestCount = count;
            }
        }
        return best;
    }

    public static double ColorRatio(Frame frame, uint color)
    {
        int b = (int)(color & 0xff), g = (int)(color >> 8 & 0xff), r = (int)(color >> 16 & 0xff);
        long matches = 0;
        for (int offset = 0; offset < frame.Bgra.Length; offset += 4)
        {
            if (Math.Abs(frame.Bgra[offset] - b) <= 2 && Math.Abs(frame.Bgra[offset + 1] - g) <= 2
                && Math.Abs(frame.Bgra[offset + 2] - r) <= 2)
            {
                matches++;
            }
        }
        return matches / (double)(frame.Width * frame.Height);
    }

    public static void SaveBmp(Frame frame, string path)
    {
        int imageSize = frame.Width * frame.Height * 4;
        using (var writer = new BinaryWriter(File.Create(path)))
        {
            writer.Write((ushort)0x4D42); writer.Write(54 + imageSize); writer.Write(0); writer.Write(54);
            writer.Write(40); writer.Write(frame.Width); writer.Write(-frame.Height);
            writer.Write((ushort)1); writer.Write((ushort)32); writer.Write(0); writer.Write(imageSize);
            writer.Write(2835); writer.Write(2835); writer.Write(0); writer.Write(0);
            writer.Write(frame.Bgra);
        }
    }

    /// Captures continuously and reports how much of each frame shows only
    /// the page background. Erase-then-draw flicker appears as frames whose
    /// background share jumps above the steady value.
    public static string Probe(IntPtr window, string mode, int durationMs, string firstPath, string worstPath)
    {
        Frame first = Capture(window, mode);
        uint background = DominantColor(first);
        double steady = ColorRatio(first, background);
        var ratios = new List<double>();
        Frame worst = first;
        double worstRatio = steady;
        var watch = Stopwatch.StartNew();
        while (watch.ElapsedMilliseconds < durationMs)
        {
            Frame frame = Capture(window, mode);
            double ratio = ColorRatio(frame, background);
            ratios.Add(ratio);
            if (ratio > worstRatio)
            {
                worstRatio = ratio;
                worst = frame;
            }
        }
        double seconds = watch.Elapsed.TotalSeconds;
        ratios.Sort();
        double median = ratios.Count == 0 ? steady : ratios[ratios.Count / 2];
        int flickerFrames = 0;
        foreach (double ratio in ratios)
        {
            if (ratio > median + 0.02) flickerFrames++;
        }
        SaveBmp(first, firstPath);
        SaveBmp(worst, worstPath);
        return string.Format(CultureInfo.InvariantCulture,
            "{{\"mode\":\"{0}\",\"frames\":{1},\"fps\":{2:F1},\"background\":\"#{3:X6}\",\"first_background_ratio\":{4:F4},\"median_background_ratio\":{5:F4},\"max_background_ratio\":{6:F4},\"max_excess\":{7:F4},\"flicker_frames\":{8}}}",
            mode, ratios.Count, ratios.Count / Math.Max(seconds, 0.001), background, steady, median,
            worstRatio, worstRatio - median, flickerFrames);
    }

    public static string Compare(Frame left, Frame right, string diffPath)
    {
        if (left.Width != right.Width || left.Height != right.Height)
        {
            return string.Format(CultureInfo.InvariantCulture,
                "{{\"same_size\":false,\"left\":\"{0}x{1}\",\"right\":\"{2}x{3}\"}}",
                left.Width, left.Height, right.Width, right.Height);
        }
        var diff = new Frame { Width = left.Width, Height = left.Height, Bgra = new byte[left.Bgra.Length] };
        long differing = 0;
        int maxDelta = 0;
        for (int offset = 0; offset < left.Bgra.Length; offset += 4)
        {
            int delta = 0;
            for (int channel = 0; channel < 3; channel++)
            {
                delta = Math.Max(delta, Math.Abs(left.Bgra[offset + channel] - right.Bgra[offset + channel]));
            }
            maxDelta = Math.Max(maxDelta, delta);
            if (delta > 0)
            {
                differing++;
                diff.Bgra[offset + 2] = 255;
            }
            else
            {
                byte gray = (byte)(160 + (left.Bgra[offset] + left.Bgra[offset + 1] + left.Bgra[offset + 2]) / 9);
                diff.Bgra[offset] = gray; diff.Bgra[offset + 1] = gray; diff.Bgra[offset + 2] = gray;
            }
        }
        SaveBmp(diff, diffPath);
        return string.Format(CultureInfo.InvariantCulture,
            "{{\"same_size\":true,\"size\":\"{0}x{1}\",\"differing_pixels\":{2},\"max_channel_delta\":{3}}}",
            left.Width, left.Height, differing, maxDelta);
    }
}
'@

$root = New-Item -ItemType Directory -Force -Path $Output
$root = $root.FullName
$variants = [ordered]@{
    current = (Resolve-Path $OldExe).Path
    new = (Resolve-Path $NewExe).Path
}
$labels = @{ current = "现有方案 (main)"; new = "新方案 (本分支)" }

function Convert-BmpToPng([string]$Path) {
    $png = [IO.Path]::ChangeExtension($Path, ".png")
    $image = [System.Drawing.Bitmap]::new($Path)
    try {
        $image.Save($png, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $image.Dispose()
    }
    Remove-Item $Path
    return $png
}

function Start-Demo([string]$Exe, [string[]]$Arguments) {
    return Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru
}

function Wait-Demo($Process, [int]$TimeoutSeconds = 30) {
    if (-not $Process.WaitForExit($TimeoutSeconds * 1000)) {
        Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
        throw "the demo did not exit within $TimeoutSeconds seconds"
    }
}

function Median([double[]]$Values) {
    $sorted = $Values | Sort-Object
    return $sorted[[Math]::Floor($sorted.Count / 2)]
}

$screenWidth = [ZsuiProxyProbe]::GetSystemMetrics(0)
$screenHeight = [ZsuiProxyProbe]::GetSystemMetrics(1)
Write-Host "Desktop: ${screenWidth}x${screenHeight}"

# 1. Memory while the simulated connection repaints every 100 ms.
$sizes = @(@{ name = "small"; width = 680; height = 560 }, @{ name = "large"; width = 1000; height = 720 })
$memory = @{}
foreach ($size in $sizes) {
    foreach ($variant in $variants.Keys) {
        $memory["$variant-$($size.name)"] = [System.Collections.Generic.List[object]]::new()
    }
    for ($run = 1; $run -le $Runs; $run++) {
        foreach ($variant in $variants.Keys) {
            $json = Join-Path $root "memory-$variant-$($size.name)-run$run.json"
            $process = Start-Demo $variants[$variant] @(
                "--demo-node", "--connect", "--tick-ms", "100", "--duration-ms", "4000",
                "--width", "$($size.width)", "--height", "$($size.height)", "--measure", $json
            )
            Wait-Demo $process
            $memory["$variant-$($size.name)"].Add((Get-Content $json -Raw | ConvertFrom-Json))
        }
    }
}

# 2. Pixel identity: a static frame and a frame after 40 scripted partial updates.
$pixels = [ordered]@{}
$scenarios = [ordered]@{
    static = @("--demo-node", "--duration-ms", "6000")
    updated = @("--demo-node", "--connect", "--ticks", "40", "--tick-ms", "40", "--duration-ms", "6000")
}
foreach ($scenario in $scenarios.Keys) {
    $frames = @{}
    foreach ($variant in $variants.Keys) {
        $json = Join-Path $root "pixels-$scenario-$variant.json"
        $process = Start-Demo $variants[$variant] ($scenarios[$scenario] + @(
            "--always-on-top", "--width", "680", "--height", "560", "--measure", $json
        ))
        $window = [ZsuiProxyProbe]::WaitForWindow($process.Id, 10000)
        if ($window -eq [IntPtr]::Zero) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
            throw "the $variant demo window did not appear"
        }
        Start-Sleep -Milliseconds 3000
        $frames[$variant] = [ZsuiProxyProbe]::Capture($window, "print")
        $bmp = Join-Path $root "screen-$scenario-$variant.bmp"
        [ZsuiProxyProbe]::SaveBmp($frames[$variant], $bmp)
        [void](Convert-BmpToPng $bmp)
        Wait-Demo $process
    }
    $diff = Join-Path $root "diff-$scenario.bmp"
    $pixels[$scenario] = [ZsuiProxyProbe]::Compare($frames.current, $frames.new, $diff) | ConvertFrom-Json
    [void](Convert-BmpToPng $diff)
}

# 3. Flicker while repainting every 30 ms, plus CPU time spent painting.
function Measure-Flicker([string]$Name, $Process, [string]$Mode) {
    $window = [ZsuiProxyProbe]::WaitForWindow($Process.Id, 10000)
    if ($window -eq [IntPtr]::Zero) {
        Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
        throw "the $Name window did not appear"
    }
    Start-Sleep -Milliseconds 1000
    $Process.Refresh()
    $cpuBefore = $Process.TotalProcessorTime
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $first = Join-Path $root "flicker-$Name-$Mode-first.bmp"
    $worst = Join-Path $root "flicker-$Name-$Mode-worst.bmp"
    $result = [ZsuiProxyProbe]::Probe($window, $Mode, 4000, $first, $worst) | ConvertFrom-Json
    $Process.Refresh()
    $cpu = ($Process.TotalProcessorTime - $cpuBefore).TotalMilliseconds / $watch.Elapsed.TotalSeconds
    $result | Add-Member -NotePropertyName cpu_ms_per_second -NotePropertyValue ([Math]::Round($cpu, 1))
    [void](Convert-BmpToPng $first)
    [void](Convert-BmpToPng $worst)
    return $result
}

$flicker = [ordered]@{}
foreach ($mode in "screen", "print") {
    foreach ($variant in $variants.Keys) {
        $json = Join-Path $root "flicker-$variant-$mode.json"
        $process = Start-Demo $variants[$variant] @(
            "--demo-node", "--connect", "--tick-ms", "30", "--duration-ms", "9000",
            "--always-on-top", "--width", "680", "--height", "560", "--measure", $json
        )
        $flicker["$variant-$mode"] = Measure-Flicker $variant $process $mode
        Wait-Demo $process
        $flicker["$variant-$mode"] | Add-Member -NotePropertyName memory -NotePropertyValue (Get-Content $json -Raw | ConvertFrom-Json)
    }
    $control = Start-Process -FilePath "powershell.exe" -ArgumentList @(
        "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
        (Join-Path $PSScriptRoot "proxy-demo-flicker-control.ps1")
    ) -PassThru
    try {
        $flicker["control-$mode"] = Measure-Flicker "control" $control $mode
    } finally {
        Stop-Process -Id $control.Id -Force -ErrorAction SilentlyContinue
    }
}

# 4. Summary.
$area = @{ small = 680 * 560; large = 1000 * 720 }
function Memory-Median([string]$Key, [string]$Field) {
    return Median ($memory[$Key] | ForEach-Object { [double]$_.$Field })
}
function MiB([double]$Bytes) { return "{0:N2} MiB" -f ($Bytes / 1MB) }
function Delta([double]$Before, [double]$After) {
    $change = $After - $Before
    $percent = if ($Before -ne 0) { $change / $Before * 100 } else { 0 }
    return "{0}{1:N2} MiB ({2:+0.0;-0.0}%)" -f ($(if ($change -ge 0) { "+" } else { "" })), ($change / 1MB), $percent
}

$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add("## ZSUI 代理演示：现有方案 vs 新方案（Windows Server 2025，$Runs 次中位数）")
$lines.Add("")
$lines.Add("桌面 ${screenWidth}x${screenHeight}。演示持续模拟流量，每 100 ms 刷新一次后采样内存。")
$lines.Add("")
$lines.Add("| 指标 | $($labels.current) | $($labels.new) | 变化 |")
$lines.Add("| --- | ---: | ---: | ---: |")
$results = [ordered]@{ desktop = "${screenWidth}x${screenHeight}"; runs = $Runs; memory = [ordered]@{} }
foreach ($size in $sizes) {
    foreach ($field in @(
        @{ key = "resident_bytes"; label = "工作集" },
        @{ key = "private_resident_bytes"; label = "私有字节" },
        @{ key = "peak_resident_bytes"; label = "峰值工作集" }
    )) {
        $before = Memory-Median "current-$($size.name)" $field.key
        $after = Memory-Median "new-$($size.name)" $field.key
        $results.memory["$($field.key)-$($size.name)"] = [ordered]@{ current = $before; new = $after }
        $lines.Add("| $($field.label) $($size.width)×$($size.height) | $(MiB $before) | $(MiB $after) | $(Delta $before $after) |")
    }
}
foreach ($field in @(@{ key = "resident_bytes"; label = "工作集" }, @{ key = "private_resident_bytes"; label = "私有字节" })) {
    $slopes = @{}
    foreach ($variant in $variants.Keys) {
        $slopes[$variant] = ((Memory-Median "$variant-large" $field.key) - (Memory-Median "$variant-small" $field.key)) / ($area.large - $area.small)
    }
    $results.memory["$($field.key)-bytes-per-pixel"] = $slopes
    $lines.Add(("| {0}随窗口面积增长 | {1:N2} 字节/像素 | {2:N2} 字节/像素 | |" -f $field.label, $slopes.current, $slopes.new))
}
$lines.Add("")
$lines.Add("### 显示是否一致")
$lines.Add("")
$lines.Add("| 画面 | 尺寸 | 不同像素数 | 最大通道差 |")
$lines.Add("| --- | --- | ---: | ---: |")
foreach ($scenario in $pixels.Keys) {
    $result = $pixels[$scenario]
    $name = if ($scenario -eq "static") { "静态首帧" } else { "40 次增量刷新后" }
    if ($result.same_size) {
        $lines.Add("| $name | $($result.size) | $($result.differing_pixels) | $($result.max_channel_delta) |")
    } else {
        $lines.Add("| $name | 尺寸不同：$($result.left) / $($result.right) | — | — |")
    }
}
$results.pixels = $pixels
$lines.Add("")
$lines.Add("### 闪屏探测（每 30 ms 刷新，连续截图 4 秒）")
$lines.Add("")
$lines.Add("「空白占比」是一帧中只显示页面底色的像素比例；先擦除再绘制的闪屏会让个别帧的空白占比突然升高。超出中位数 2% 记为闪屏帧。")
$lines.Add("")
$lines.Add("| 截图方式 | 对象 | 帧数 | 帧率 | 空白占比中位数 | 最大超出 | 闪屏帧 | CPU ms/s |")
$lines.Add("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
foreach ($key in $flicker.Keys) {
    $result = $flicker[$key]
    $parts = $key.Split("-")
    $who = switch ($parts[0]) { "current" { $labels.current } "new" { $labels.new } default { "对照：无双缓冲 WinForms" } }
    $how = if ($parts[1] -eq "screen") { "桌面截屏" } else { "PrintWindow" }
    $lines.Add(("| {0} | {1} | {2} | {3:N1} | {4:P1} | {5:P1} | {6} | {7:N1} |" -f $how, $who, $result.frames, $result.fps, $result.median_background_ratio, $result.max_excess, $result.flicker_frames, $result.cpu_ms_per_second))
}
$results.flicker = $flicker
$lines.Add("")
$lines.Add("截图与差异图在构件 proxy-demo-comparison-windows-2025 中。")

$summary = Join-Path $root "summary.md"
$lines | Set-Content -Path $summary -Encoding utf8
$results | ConvertTo-Json -Depth 8 | Set-Content -Path (Join-Path $root "summary.json") -Encoding utf8
Write-Host "Wrote $summary"
