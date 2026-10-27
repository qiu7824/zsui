param(
    [Parameter(Mandatory = $true)]
    [string]$BaselineProxy,
    [Parameter(Mandatory = $true)]
    [string]$CandidateProxy,
    [Parameter(Mandatory = $true)]
    [string]$BaselineNotepad,
    [Parameter(Mandatory = $true)]
    [string]$CandidateNotepad,
    [string]$Output = "target/proxy-demo-comparison",
    [int]$Pairs = 10,
    # Display scale the desktop is already set to; window sizes scale with it.
    [int]$Scale = 100,
    # When set, notepad and demo scenes also switch the running desktop to
    # this scale and back, as moving to a monitor with another DPI does.
    [int]$TransitionScale = 0
)

# Paired Windows comparison of two builds for the paint-path acceptance gate
# in docs/memory-stability-plan.md (G4):
# - private bytes, working set and paint CPU of examples/zsui_proxy_client.rs
#   in visible, hidden and restored phases at two window sizes, as $Pairs
#   alternating process pairs with 20 samples per phase and paired bootstrap
#   intervals of the per-process medians;
# - interaction latency through scripts/measure-windows-interaction-latency.ps1;
# - pixel identity after scrolling, a modal dialog with scrim and shadow, its
#   dismissal, rapid resizing and occlusion in examples/zsui_notepad.rs, and
#   after scripted partial updates in the proxy demo;
# - tearing (full-height bars that change color together) and erase flicker,
#   with an unbuffered Windows Forms window as the flicker negative control.

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;

public static class ZsuiPaintProbe
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
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out RECT rect);
    [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr window, ref POINT point);
    [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr window, IntPtr target, uint flags);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")] static extern bool RedrawWindow(IntPtr window, IntPtr rect, IntPtr region, uint flags);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr window);
    [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr window, IntPtr dc);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint action, uint parameter, out RECT rect, uint update);

    /// Desktop area outside the taskbar, in physical pixels.
    public static RECT WorkArea()
    {
        RECT area;
        SystemParametersInfoW(0x0030, 0, out area, 0);
        return area;
    }

    /// Client size the window actually received, in physical pixels.
    public static string ClientSize(IntPtr window)
    {
        RECT client;
        GetClientRect(window, out client);
        return (client.Right - client.Left) + "x" + (client.Bottom - client.Top);
    }

    /// Window rectangles, screen captures and UIA bounds must be physical
    /// pixels at every scale, so the measuring process is per-monitor aware.
    public static void BecomeDpiAware()
    {
        SetProcessDpiAwarenessContext(new IntPtr(-4));
        SetThreadDpiAwarenessContext(new IntPtr(-4));
    }
    [DllImport("dwmapi.dll")] static extern int DwmFlush();
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

    /// Paints only the already invalid region, then waits for composition.
    /// No invalidation is added, so missed damage stays visible to captures.
    public static void Settle(IntPtr window)
    {
        RedrawWindow(window, IntPtr.Zero, IntPtr.Zero, 0x0100 | 0x0080);
        DwmFlush();
    }

    public static void Place(IntPtr window, int x, int y)
    {
        SetWindowPos(window, IntPtr.Zero, x, y, 0, 0, 0x0001 | 0x0004 | 0x0010);
    }

    public static void Resize(IntPtr window, int width, int height)
    {
        SetWindowPos(window, IntPtr.Zero, 0, 0, width, height, 0x0002 | 0x0004 | 0x0010);
    }

    public static void Wheel(IntPtr window, int delta, int screenX, int screenY)
    {
        SendMessageW(window, 0x020A, new UIntPtr(unchecked((uint)(delta << 16))),
            new IntPtr((screenY << 16) | (screenX & 0xffff)));
    }

    public static void Close(IntPtr window)
    {
        PostMessageW(window, 0x0010, UIntPtr.Zero, IntPtr.Zero);
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

    static bool Near(uint left, uint right, int tolerance)
    {
        for (int shift = 0; shift < 24; shift += 8)
        {
            if (Math.Abs((int)(left >> shift & 0xff) - (int)(right >> shift & 0xff)) > tolerance)
            {
                return false;
            }
        }
        return true;
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
        long matches = 0;
        for (int offset = 0; offset < frame.Bgra.Length; offset += 4)
        {
            if (Near(Rgb(frame.Bgra, offset), color, 2))
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

    /// Erase-then-draw flicker shows up as frames whose share of bare page
    /// background jumps above the steady value.
    public static string FlickerProbe(IntPtr window, string mode, int durationMs, string firstPath, string worstPath)
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
            "{{\"mode\":\"{0}\",\"frames\":{1},\"fps\":{2:F1},\"background\":\"#{3:X6}\",\"median_background_ratio\":{4:F4},\"max_background_ratio\":{5:F4},\"max_excess\":{6:F4},\"flicker_frames\":{7}}}",
            mode, ratios.Count, ratios.Count / Math.Max(seconds, 0.001), background, median,
            worstRatio, worstRatio - median, flickerFrames);
    }

    /// Share of a pixel column that differs from the column's dominant color.
    static double MinorityShare(Frame frame, int x, out uint dominant)
    {
        var counts = new Dictionary<uint, int>();
        dominant = 0;
        int best = -1;
        for (int y = 0; y < frame.Height; y++)
        {
            uint color = Rgb(frame.Bgra, (y * frame.Width + x) * 4);
            int count;
            counts.TryGetValue(color, out count);
            counts[color] = ++count;
            if (count > best)
            {
                best = count;
                dominant = color;
            }
        }
        int minority = 0;
        for (int y = 0; y < frame.Height; y++)
        {
            if (!Near(Rgb(frame.Bgra, (y * frame.Width + x) * 4), dominant, 6)) minority++;
        }
        return minority / (double)frame.Height;
    }

    /// The tear probe draws full-height bars at both client edges that change
    /// color together on every update. A frame in which a bar shows two
    /// colors was presented while only part of an update had reached it.
    public static string TearProbe(IntPtr window, string mode, int durationMs, string worstPath)
    {
        var watch = Stopwatch.StartNew();
        int frames = 0, torn = 0, toggles = 0;
        uint previous = 0;
        bool hasPrevious = false;
        double worstShare = 0;
        Frame worst = null;
        while (watch.ElapsedMilliseconds < durationMs)
        {
            Frame frame = Capture(window, mode);
            frames++;
            uint left, right;
            double share = Math.Max(MinorityShare(frame, 3, out left), MinorityShare(frame, frame.Width - 4, out right));
            if (share >= 0.05) torn++;
            if (hasPrevious && !Near(left, previous, 6)) toggles++;
            previous = left;
            hasPrevious = true;
            if (worst == null || share > worstShare)
            {
                worstShare = share;
                worst = frame;
            }
        }
        if (worst != null) SaveBmp(worst, worstPath);
        return string.Format(CultureInfo.InvariantCulture,
            "{{\"mode\":\"{0}\",\"frames\":{1},\"fps\":{2:F1},\"color_changes_seen\":{3},\"torn_frames\":{4},\"max_minority_share\":{5:F4}}}",
            mode, frames, frames / Math.Max(watch.Elapsed.TotalSeconds, 0.001), toggles, torn, worstShare);
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

    static double Median(double[] values)
    {
        var sorted = (double[])values.Clone();
        Array.Sort(sorted);
        int middle = sorted.Length / 2;
        return sorted.Length % 2 == 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
    }

    /// Median paired difference (candidate minus baseline) with a 95% interval
    /// from resampling whole pairs.
    public static string Bootstrap(double[] baseline, double[] candidate, int iterations, int seed)
    {
        int count = Math.Min(baseline.Length, candidate.Length);
        var differences = new double[count];
        for (int index = 0; index < count; index++) differences[index] = candidate[index] - baseline[index];
        var random = new Random(seed);
        var statistics = new double[iterations];
        var sample = new double[count];
        for (int iteration = 0; iteration < iterations; iteration++)
        {
            for (int index = 0; index < count; index++) sample[index] = differences[random.Next(count)];
            statistics[iteration] = Median(sample);
        }
        Array.Sort(statistics);
        return string.Format(CultureInfo.InvariantCulture,
            "{{\"pairs\":{0},\"baseline_median\":{1:R},\"candidate_median\":{2:R},\"median_difference\":{3:R},\"ci_low\":{4:R},\"ci_high\":{5:R}}}",
            count, Median(baseline), Median(candidate), Median(differences),
            statistics[(int)(iterations * 0.025)], statistics[(int)(iterations * 0.975) - 1]);
    }
}
'@

$root = (New-Item -ItemType Directory -Force -Path $Output).FullName
[ZsuiPaintProbe]::BecomeDpiAware()
$displayScript = Join-Path $PSScriptRoot "set-windows-display-scale.ps1"
$display = & $displayScript -Query | ConvertFrom-Json
$monitors = @($display.monitors)
$secondMonitor = $monitors | Where-Object { $_.left -ne 0 -or $_.top -ne 0 } | Select-Object -First 1
$factor = $Scale / 100.0
function Px([int]$Logical) { return [int][Math]::Round($Logical * $factor) }
# Captured windows must lie inside the work area with their frame, or a
# composed-desktop capture would read the taskbar or off-screen pixels. When a
# scaled size does not fit, it shrinks evenly; both variants get the same size.
$workArea = [ZsuiPaintProbe]::WorkArea()
function Fit([int]$Width, [int]$Height) {
    $room = [Math]::Min(
        ($workArea.Right - $workArea.Left - (Px 40)) / (Px $Width),
        ($workArea.Bottom - $workArea.Top - (Px 60)) / (Px $Height))
    $shrink = [Math]::Min(1.0, $room)
    return @{ width = [int][Math]::Floor((Px $Width) * $shrink); height = [int][Math]::Floor((Px $Height) * $shrink) }
}
$captureSize = Fit 680 560
$notepadSize = Fit 900 620
$captureArguments = @("--always-on-top", "--width", "$($captureSize.width)", "--height", "$($captureSize.height)")
$variants = @("baseline", "candidate")
$proxy = @{ baseline = (Resolve-Path $BaselineProxy).Path; candidate = (Resolve-Path $CandidateProxy).Path }
$notepad = @{ baseline = (Resolve-Path $BaselineNotepad).Path; candidate = (Resolve-Path $CandidateNotepad).Path }
$labels = @{ baseline = "main（整窗重绘）"; candidate = "本分支（局部重绘，单次呈现）" }

function Convert-BmpToPng([string]$Path) {
    $png = [IO.Path]::ChangeExtension($Path, ".png")
    $image = [System.Drawing.Bitmap]::new($Path)
    try {
        $image.Save($png, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $image.Dispose()
    }
    Remove-Item $Path
}

function Save-Frame($Frame, [string]$Path) {
    [ZsuiPaintProbe]::SaveBmp($Frame, "$Path.bmp")
    Convert-BmpToPng "$Path.bmp"
}

# Windows run as ordinary applications and are stopped once measured, so no
# phase races an automatic close.
function Stop-Run($Run) {
    Stop-Process -Id $Run.process.Id -Force -ErrorAction SilentlyContinue
    [void]$Run.process.WaitForExit(10000)
}

function Start-Window([string]$Exe, [string[]]$Arguments) {
    $process = Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru
    $window = [ZsuiPaintProbe]::WaitForWindow($process.Id, 15000)
    if ($window -eq [IntPtr]::Zero) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        throw "no window appeared for $Exe"
    }
    return @{ process = $process; window = $window; birth = $process.StartTime }
}

function Get-Median([double[]]$Values) {
    $sorted = [double[]]($Values | Sort-Object)
    $middle = [Math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2) { return $sorted[$middle] }
    return ($sorted[$middle - 1] + $sorted[$middle]) / 2
}

function Get-P95([double[]]$Values) {
    $sorted = [double[]]($Values | Sort-Object)
    return $sorted[[Math]::Ceiling($sorted.Count * 0.95) - 1]
}

function Get-Bootstrap([double[]]$Baseline, [double[]]$Candidate) {
    return [ZsuiPaintProbe]::Bootstrap($Baseline, $Candidate, 10000, 20261027) | ConvertFrom-Json
}

$screenWidth = [ZsuiPaintProbe]::GetSystemMetrics(0)
$screenHeight = [ZsuiPaintProbe]::GetSystemMetrics(1)
Write-Host "Desktop: ${screenWidth}x${screenHeight}"
$results = [ordered]@{
    desktop = "${screenWidth}x${screenHeight}"
    scale = $Scale
    transition_scale = $TransitionScale
    monitors = $monitors
    pairs = $Pairs
    binaries = [ordered]@{}
}
foreach ($variant in $variants) {
    $results.binaries[$variant] = [ordered]@{
        proxy_sha256 = (Get-FileHash -LiteralPath $proxy[$variant] -Algorithm SHA256).Hash
        notepad_sha256 = (Get-FileHash -LiteralPath $notepad[$variant] -Algorithm SHA256).Hash
    }
}

# 1. Memory and paint CPU: visible, hidden and restored phases.
function Measure-Phase($Run) {
    $private = [Collections.Generic.List[double]]::new()
    $working = [Collections.Generic.List[double]]::new()
    $process = $Run.process
    $process.Refresh()
    $cpuBefore = $process.TotalProcessorTime
    $watch = [Diagnostics.Stopwatch]::StartNew()
    for ($sample = 0; $sample -lt 20; $sample++) {
        $process.Refresh()
        if ($process.HasExited -or $process.StartTime -ne $Run.birth) {
            throw "the measured process exited or its identity changed"
        }
        $private.Add($process.PrivateMemorySize64)
        $working.Add($process.WorkingSet64)
        Start-Sleep -Milliseconds 100
    }
    $process.Refresh()
    return [ordered]@{
        private_bytes = Get-Median $private.ToArray()
        working_set = Get-Median $working.ToArray()
        cpu_ms_per_second = ($process.TotalProcessorTime - $cpuBefore).TotalMilliseconds / $watch.Elapsed.TotalSeconds
    }
}

$sizes = @(
    @{ name = "small"; width = (Px 680); height = (Px 560) },
    @{ name = "large"; width = (Px 1000); height = (Px 720) }
)
$memory = [ordered]@{}
foreach ($size in $sizes) {
    foreach ($variant in $variants) {
        $memory["$variant-$($size.name)"] = [Collections.Generic.List[object]]::new()
    }
    for ($pair = 1; $pair -le $Pairs; $pair++) {
        $order = if ($pair % 2) { $variants } else { @("candidate", "baseline") }
        foreach ($variant in $order) {
            $run = Start-Window $proxy[$variant] @(
                "--demo-node", "--connect", "--tick-ms", "100",
                "--width", "$($size.width)", "--height", "$($size.height)"
            )
            Start-Sleep -Seconds 3
            $client = [ZsuiPaintProbe]::ClientSize($run.window)
            $visible = Measure-Phase $run
            [void][ZsuiPaintProbe]::ShowWindow($run.window, 0)
            Start-Sleep -Seconds 2
            $hidden = Measure-Phase $run
            [void][ZsuiPaintProbe]::ShowWindow($run.window, 8)
            Start-Sleep -Seconds 2
            $restored = Measure-Phase $run
            $run.process.Refresh()
            $peak = $run.process.PeakWorkingSet64
            Stop-Run $run
            $memory["$variant-$($size.name)"].Add([ordered]@{
                pair = $pair; client = $client; visible = $visible; hidden = $hidden; restored = $restored; peak_working_set = $peak
            })
        }
    }
    # Windows limits a new window to the desktop, so the size used for
    # labels and per-pixel growth is the client size the windows received.
    $clients = @($variants | ForEach-Object { $memory["$_-$($size.name)"] } | ForEach-Object { $_.client } | Sort-Object -Unique)
    if ($clients.Count -ne 1) { throw "$($size.name) windows received different client sizes: $($clients -join ', ')" }
    $received = $clients[0].Split("x")
    $size.width = [int]$received[0]
    $size.height = [int]$received[1]
}
$results.memory = $memory

# 2. Interaction latency with the repository's paired latency probe.
$latencyRoot = Join-Path $root "latency"
New-Item -ItemType Directory -Force -Path $latencyRoot | Out-Null
$latency = @{ baseline = [Collections.Generic.List[object]]::new(); candidate = [Collections.Generic.List[object]]::new() }
$latencyFailures = [Collections.Generic.List[string]]::new()
for ($pair = 1; $pair -le $Pairs; $pair++) {
    $order = if ($pair % 2) { $variants } else { @("candidate", "baseline") }
    foreach ($variant in $order) {
        $measured = $false
        for ($attempt = 0; $attempt -lt 3 -and -not $measured; $attempt++) {
            $trial = $pair + 100 * $attempt
            try {
                & (Join-Path $PSScriptRoot "measure-windows-interaction-latency.ps1") `
                    -Executable $notepad[$variant] -OutputDirectory $latencyRoot `
                    -Variant $variant -Trial $trial -Operations 200 -VerifyEmptyReplacement | Out-Host
                $record = Get-Content (Join-Path $latencyRoot "$variant-$trial/latency.json") -Raw | ConvertFrom-Json
                $latency[$variant].Add([ordered]@{
                    pair = $pair
                    input = Get-P95 $record.input_ms
                    scroll = Get-P95 $record.scroll_ms
                    restore = Get-P95 $record.restore_ms
                })
                $measured = $true
            } catch {
                $latencyFailures.Add("$variant pair $pair attempt $attempt`: $_")
            }
        }
        if (-not $measured) {
            throw "latency pair $pair could not be measured for $variant"
        }
    }
}
$results.latency = $latency
$results.latency_rejected_attempts = $latencyFailures

# 3. Pixel identity without forced full repaints.
$pixels = [ordered]@{}
function Compare-Scenes($Frames, [string]$Name) {
    $diff = Join-Path $root "diff-$Name"
    $result = [ZsuiPaintProbe]::Compare($Frames.baseline, $Frames.candidate, "$diff.bmp") | ConvertFrom-Json
    Convert-BmpToPng "$diff.bmp"
    $pixels[$Name] = $result
}

$proxyScenes = [ordered]@{
    "proxy-static" = @("--demo-node")
    "proxy-updated" = @("--demo-node", "--connect", "--ticks", "40", "--tick-ms", "40")
}
foreach ($scene in $proxyScenes.Keys) {
    $frames = @{}
    foreach ($variant in $variants) {
        $run = Start-Window $proxy[$variant] ($proxyScenes[$scene] + $captureArguments)
        try {
            Start-Sleep -Milliseconds 3000
            [ZsuiPaintProbe]::Settle($run.window)
            $frames[$variant] = [ZsuiPaintProbe]::Capture($run.window, "print")
            Save-Frame $frames[$variant] (Join-Path $root "$scene-$variant")
        } finally {
            Stop-Run $run
        }
    }
    Compare-Scenes $frames $scene
}

if ($TransitionScale -gt 0) {
    $frames = @{ switched = @{}; restored = @{} }
    foreach ($variant in $variants) {
        $run = Start-Window $proxy[$variant] (@(
            "--demo-node", "--connect", "--ticks", "20", "--tick-ms", "40"
        ) + $captureArguments)
        try {
            Start-Sleep -Milliseconds 2500
            & $displayScript -Scale $TransitionScale | Out-Null
            Start-Sleep -Milliseconds 2000
            [ZsuiPaintProbe]::Settle($run.window)
            $frames.switched[$variant] = [ZsuiPaintProbe]::Capture($run.window, "print")
            & $displayScript -Scale $Scale | Out-Null
            Start-Sleep -Milliseconds 2000
            [ZsuiPaintProbe]::Settle($run.window)
            $frames.restored[$variant] = [ZsuiPaintProbe]::Capture($run.window, "print")
            Save-Frame $frames.switched[$variant] (Join-Path $root "proxy-dpi-switched-$variant")
            Save-Frame $frames.restored[$variant] (Join-Path $root "proxy-dpi-restored-$variant")
        } finally {
            Stop-Run $run
        }
    }
    Compare-Scenes $frames.switched "proxy-dpi-switched"
    Compare-Scenes $frames.restored "proxy-dpi-restored"
}

function Find-Named($Root, [string]$Name, [int]$TimeoutMs) {
    $condition = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::NameProperty, $Name)
    $watch = [Diagnostics.Stopwatch]::StartNew()
    while ($watch.ElapsedMilliseconds -lt $TimeoutMs) {
        $element = $Root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
        if ($element) { return $element }
        Start-Sleep -Milliseconds 50
    }
    return $null
}

$notepadScenes = @("edited", "scrolled", "dialog", "dismissed", "resized", "uncovered")
if ($TransitionScale -gt 0) { $notepadScenes += @("dpi-switched", "dpi-restored") }
if ($secondMonitor) { $notepadScenes += @("monitor-2", "monitor-1") }

function Switch-Scale([int]$Percent) {
    & $displayScript -Scale $Percent | Out-Null
    Start-Sleep -Milliseconds 2000
}
$notepadFrames = @{}
foreach ($scene in $notepadScenes) { $notepadFrames[$scene] = @{} }
foreach ($variant in $variants) {
    $run = Start-Window $notepad[$variant] @("--width", "$($notepadSize.width)", "--height", "$($notepadSize.height)")
    try {
        $window = $run.window
        [ZsuiPaintProbe]::Place($window, $workArea.Left, $workArea.Top)
        Start-Sleep -Milliseconds 1500
        $uia = [System.Windows.Automation.AutomationElement]::FromHandle($window)
        $editCondition = [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Edit)
        $editor = $uia.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $editCondition)
        if (-not $editor) { throw "no accessible editor in the $variant notepad" }
        $content = (0..199 | ForEach-Object { "Line $_ / 中文 العربية" }) -join "`n"
        $editor.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($content)
        Start-Sleep -Milliseconds 500
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.edited[$variant] = [ZsuiPaintProbe]::Capture($window, "print")

        $rect = $editor.Current.BoundingRectangle
        $x = [int]($rect.Left + $rect.Width / 2)
        $y = [int]($rect.Top + $rect.Height / 2)
        for ($step = 0; $step -lt 6; $step++) {
            [ZsuiPaintProbe]::Wheel($window, -120, $x, $y)
            Start-Sleep -Milliseconds 60
            [ZsuiPaintProbe]::Settle($window)
        }
        Start-Sleep -Milliseconds 300
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.scrolled[$variant] = [ZsuiPaintProbe]::Capture($window, "print")

        [ZsuiPaintProbe]::Close($window)
        $cancel = Find-Named $uia "取消 / Cancel" 5000
        if (-not $cancel) { throw "the $variant save prompt did not appear" }
        Start-Sleep -Milliseconds 400
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.dialog[$variant] = [ZsuiPaintProbe]::Capture($window, "print")
        $cancel.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
        Start-Sleep -Milliseconds 400
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.dismissed[$variant] = [ZsuiPaintProbe]::Capture($window, "print")

        $bounds = New-Object ZsuiPaintProbe+RECT
        [void][ZsuiPaintProbe]::GetWindowRect($window, [ref]$bounds)
        $outerWidth = $bounds.Right - $bounds.Left
        $outerHeight = $bounds.Bottom - $bounds.Top
        for ($step = 0; $step -lt 16; $step++) {
            $delta = if ($step % 2) { 0 } else { 180 }
            [ZsuiPaintProbe]::Resize($window, $outerWidth - $delta, $outerHeight - [int]($delta / 2))
            Start-Sleep -Milliseconds 25
        }
        [ZsuiPaintProbe]::Resize($window, $outerWidth, $outerHeight)
        Start-Sleep -Milliseconds 800
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.resized[$variant] = [ZsuiPaintProbe]::Capture($window, "print")

        $occluder = Start-Process -FilePath "powershell.exe" -ArgumentList @(
            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
            (Join-Path $PSScriptRoot "proxy-demo-flicker-control.ps1"), "-Width", "420", "-Height", "300"
        ) -PassThru
        try {
            [void][ZsuiPaintProbe]::WaitForWindow($occluder.Id, 15000)
            Start-Sleep -Milliseconds 800
        } finally {
            Stop-Process -Id $occluder.Id -Force -ErrorAction SilentlyContinue
        }
        Start-Sleep -Milliseconds 500
        [ZsuiPaintProbe]::Settle($window)
        $notepadFrames.uncovered[$variant] = [ZsuiPaintProbe]::Capture($window, "print")

        if ($TransitionScale -gt 0) {
            Switch-Scale $TransitionScale
            [ZsuiPaintProbe]::Settle($window)
            $notepadFrames["dpi-switched"][$variant] = [ZsuiPaintProbe]::Capture($window, "print")
            Switch-Scale $Scale
            [ZsuiPaintProbe]::Settle($window)
            $notepadFrames["dpi-restored"][$variant] = [ZsuiPaintProbe]::Capture($window, "print")
        }
        if ($secondMonitor) {
            [ZsuiPaintProbe]::Place($window, $secondMonitor.left + 20, $secondMonitor.top + 20)
            Start-Sleep -Milliseconds 1500
            [ZsuiPaintProbe]::Settle($window)
            $notepadFrames["monitor-2"][$variant] = [ZsuiPaintProbe]::Capture($window, "print")
            [ZsuiPaintProbe]::Place($window, $workArea.Left, $workArea.Top)
            Start-Sleep -Milliseconds 1500
            [ZsuiPaintProbe]::Settle($window)
            $notepadFrames["monitor-1"][$variant] = [ZsuiPaintProbe]::Capture($window, "print")
        }
        foreach ($scene in $notepadScenes) {
            Save-Frame $notepadFrames[$scene][$variant] (Join-Path $root "notepad-$scene-$variant")
        }
    } finally {
        Stop-Run $run
    }
}
foreach ($scene in $notepadScenes) {
    Compare-Scenes $notepadFrames[$scene] "notepad-$scene"
}
$results.pixels = $pixels

# 4. Tearing and erase flicker while repainting every 30 ms.
function Measure-Probe([string]$Name, $Run, [string]$Kind, [string]$Mode) {
    Start-Sleep -Milliseconds 1000
    $Run.process.Refresh()
    $cpuBefore = $Run.process.TotalProcessorTime
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $prefix = Join-Path $root "$Kind-$Name-$Mode"
    if ($Kind -eq "tear") {
        $result = [ZsuiPaintProbe]::TearProbe($Run.window, $Mode, 4000, "$prefix-worst.bmp") | ConvertFrom-Json
        Convert-BmpToPng "$prefix-worst.bmp"
    } else {
        $result = [ZsuiPaintProbe]::FlickerProbe($Run.window, $Mode, 4000, "$prefix-first.bmp", "$prefix-worst.bmp") | ConvertFrom-Json
        Convert-BmpToPng "$prefix-first.bmp"
        Convert-BmpToPng "$prefix-worst.bmp"
    }
    $Run.process.Refresh()
    $cpu = ($Run.process.TotalProcessorTime - $cpuBefore).TotalMilliseconds / $watch.Elapsed.TotalSeconds
    $result | Add-Member -NotePropertyName cpu_ms_per_second -NotePropertyValue ([Math]::Round($cpu, 1))
    return $result
}

$probes = [ordered]@{}
foreach ($kind in "tear", "flicker") {
    foreach ($mode in "screen", "print") {
        foreach ($variant in $variants) {
            $arguments = @("--demo-node", "--connect", "--tick-ms", "30") + $captureArguments
            if ($kind -eq "tear") { $arguments += "--tear-probe" }
            $run = Start-Window $proxy[$variant] $arguments
            try {
                [ZsuiPaintProbe]::Place($run.window, $workArea.Left, $workArea.Top)
                $probes["$kind-$variant-$mode"] = Measure-Probe $variant $run $kind $mode
            } finally {
                Stop-Run $run
            }
        }
        if ($kind -eq "flicker") {
            $control = Start-Process -FilePath "powershell.exe" -ArgumentList @(
                "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                (Join-Path $PSScriptRoot "proxy-demo-flicker-control.ps1")
            ) -PassThru
            try {
                $window = [ZsuiPaintProbe]::WaitForWindow($control.Id, 15000)
                [ZsuiPaintProbe]::Place($window, $workArea.Left, $workArea.Top)
                $probes["flicker-control-$mode"] = Measure-Probe "control" @{ process = $control; window = $window } $kind $mode
            } finally {
                Stop-Process -Id $control.Id -Force -ErrorAction SilentlyContinue
            }
        }
    }
}
$results.probes = $probes

# 5. Statistics and summary.
$stats = [ordered]@{}
function Add-Statistic([string]$Name, [double[]]$Baseline, [double[]]$Candidate) {
    $stats[$Name] = Get-Bootstrap $Baseline $Candidate
}
function Field-Values([string]$Key, [scriptblock]$Select) {
    return [double[]]($memory[$Key] | Sort-Object { $_.pair } | ForEach-Object $Select)
}
$area = @{}
foreach ($size in $sizes) { $area[$size.name] = $size.width * $size.height }
foreach ($size in $sizes) {
    foreach ($phase in "visible", "hidden", "restored") {
        foreach ($field in "private_bytes", "working_set") {
            Add-Statistic "$field-$phase-$($size.name)" `
                (Field-Values "baseline-$($size.name)" { $_[$phase][$field] }) `
                (Field-Values "candidate-$($size.name)" { $_[$phase][$field] })
        }
    }
    Add-Statistic "cpu-visible-$($size.name)" `
        (Field-Values "baseline-$($size.name)" { $_.visible.cpu_ms_per_second }) `
        (Field-Values "candidate-$($size.name)" { $_.visible.cpu_ms_per_second })
    Add-Statistic "peak-$($size.name)" `
        (Field-Values "baseline-$($size.name)" { $_.peak_working_set }) `
        (Field-Values "candidate-$($size.name)" { $_.peak_working_set })
}
foreach ($field in "private_bytes", "working_set") {
    $slopes = @{}
    foreach ($variant in $variants) {
        $small = Field-Values "$variant-small" { $_.visible[$field] }
        $large = Field-Values "$variant-large" { $_.visible[$field] }
        $slopes[$variant] = [double[]](0..($small.Count - 1) | ForEach-Object { ($large[$_] - $small[$_]) / ($area.large - $area.small) })
    }
    Add-Statistic "$field-bytes-per-pixel" $slopes.baseline $slopes.candidate
}
foreach ($operation in "input", "scroll", "restore") {
    Add-Statistic "latency-$operation" `
        ([double[]]($latency.baseline | Sort-Object { $_.pair } | ForEach-Object { $_[$operation] })) `
        ([double[]]($latency.candidate | Sort-Object { $_.pair } | ForEach-Object { $_[$operation] }))
}
$results.statistics = $stats

function MiB([double]$Bytes) { return "{0:N3}" -f ($Bytes / 1MB) }
function Percent($Statistic) {
    if ($Statistic.baseline_median -eq 0) { return "—" }
    return "{0:+0.0;-0.0}%" -f (($Statistic.candidate_median - $Statistic.baseline_median) / $Statistic.baseline_median * 100)
}
function Verdict($Statistic) {
    if ($Statistic.ci_high -lt 0) { return "下降已确认" }
    if ($Statistic.ci_low -gt 0) { return "上升已确认" }
    return "尚未确认"
}

$lines = [Collections.Generic.List[string]]::new()
$lines.Add("## 局部重绘（每次更新单次呈现）vs main 整窗重绘（Windows Server 2025，$Pairs 对交替进程）")
$lines.Add("")
$monitorText = @($monitors | ForEach-Object { "$($_.width)x$($_.height) @ $($_.dpi) DPI" }) -join "；"
$lines.Add("桌面 ${screenWidth}x${screenHeight}，缩放 $Scale%（$monitorText）。窗口尺寸按缩放放大；截图用的窗口放不下工作区时等比缩小，基线与候选相同；内存表中的尺寸是窗口实际得到的客户区。每进程每阶段 20 次采样取中位数；区间为 10,000 次配对 bootstrap 的 95% 区间，差值为候选减基线。")
$lines.Add("")
$lines.Add("### 私有字节与工作集（MiB，演示每 100 ms 刷新）")
$lines.Add("")
$lines.Add("| 指标 | $($labels.baseline) | $($labels.candidate) | 变化 | 差值 95% CI（KiB） | 结论 |")
$lines.Add("| --- | ---: | ---: | ---: | ---: | --- |")
foreach ($size in $sizes) {
    foreach ($phase in @(@{ k = "visible"; n = "可见" }, @{ k = "hidden"; n = "隐藏" }, @{ k = "restored"; n = "恢复" })) {
        foreach ($field in @(@{ k = "private_bytes"; n = "私有字节" }, @{ k = "working_set"; n = "工作集" })) {
            $s = $stats["$($field.k)-$($phase.k)-$($size.name)"]
            $lines.Add(("| {0} {1} {2}×{3} | {4} | {5} | {6} | {7:N0} 至 {8:N0} | {9} |" -f $field.n, $phase.n, $size.width, $size.height, (MiB $s.baseline_median), (MiB $s.candidate_median), (Percent $s), ($s.ci_low / 1KB), ($s.ci_high / 1KB), (Verdict $s)))
        }
    }
    $s = $stats["peak-$($size.name)"]
    $lines.Add(("| 峰值工作集 {0}×{1} | {2} | {3} | {4} | {5:N0} 至 {6:N0} | {7} |" -f $size.width, $size.height, (MiB $s.baseline_median), (MiB $s.candidate_median), (Percent $s), ($s.ci_low / 1KB), ($s.ci_high / 1KB), (Verdict $s)))
}
foreach ($field in @(@{ k = "private_bytes"; n = "私有字节" }, @{ k = "working_set"; n = "工作集" })) {
    $s = $stats["$($field.k)-bytes-per-pixel"]
    $lines.Add(("| {0}随窗口面积增长（字节/像素） | {1:N2} | {2:N2} | | {3:N2} 至 {4:N2} 字节/像素 | {5} |" -f $field.n, $s.baseline_median, $s.candidate_median, $s.ci_low, $s.ci_high, (Verdict $s)))
}
$lines.Add("")
$lines.Add("### CPU 与交互延迟")
$lines.Add("")
$lines.Add("| 指标 | $($labels.baseline) | $($labels.candidate) | 变化 | 差值 95% CI | 结论 |")
$lines.Add("| --- | ---: | ---: | ---: | ---: | --- |")
foreach ($size in $sizes) {
    $s = $stats["cpu-visible-$($size.name)"]
    $lines.Add(("| 刷新 CPU ms/s {0}×{1} | {2:N1} | {3:N1} | {4} | {5:N1} 至 {6:N1} | {7} |" -f $size.width, $size.height, $s.baseline_median, $s.candidate_median, (Percent $s), $s.ci_low, $s.ci_high, (Verdict $s)))
}
foreach ($operation in @(@{ k = "input"; n = "输入" }, @{ k = "scroll"; n = "滚动" }, @{ k = "restore"; n = "恢复" })) {
    $s = $stats["latency-$($operation.k)"]
    $limit = [Math]::Max(0.05 * $s.baseline_median, 1.0)
    $gate = if ($s.median_difference -le $limit) { "未超过 max(5%, 1 ms)" } else { "超过 max(5%, 1 ms)" }
    $lines.Add(("| {0} p95 ms（记事本，整窗重绘） | {1:N3} | {2:N3} | {3} | {4:N3} 至 {5:N3} | {6} |" -f $operation.n, $s.baseline_median, $s.candidate_median, (Percent $s), $s.ci_low, $s.ci_high, $gate))
}
$lines.Add("")
$lines.Add("被剔除并补跑的延迟样本：$($latencyFailures.Count)。")
$lines.Add("")
$lines.Add("### 显示一致性（截图前只处理已有待绘区域，不强制整窗重绘）")
$lines.Add("")
$lines.Add("| 场景 | 尺寸 | 不同像素数 | 最大通道差 |")
$lines.Add("| --- | --- | ---: | ---: |")
$sceneNames = [ordered]@{
    "proxy-static" = "代理演示：静态首帧"
    "proxy-updated" = "代理演示：40 次局部刷新后"
    "notepad-edited" = "记事本：写入多语言长文本"
    "notepad-scrolled" = "记事本：滚动 6 格"
    "notepad-dialog" = "记事本：模态对话框（遮罩与阴影）"
    "notepad-dismissed" = "记事本：取消对话框后"
    "notepad-resized" = "记事本：快速改尺寸 16 次后"
    "notepad-uncovered" = "记事本：被遮挡后移开"
    "proxy-dpi-switched" = "代理演示：运行中切到 $TransitionScale% 后"
    "proxy-dpi-restored" = "代理演示：切回 $Scale% 后"
    "notepad-dpi-switched" = "记事本：运行中切到 $TransitionScale% 后"
    "notepad-dpi-restored" = "记事本：切回 $Scale% 后"
    "notepad-monitor-2" = "记事本：移到第二块显示器"
    "notepad-monitor-1" = "记事本：移回主显示器"
}
foreach ($scene in $sceneNames.Keys) {
    if (-not $pixels.Contains($scene)) { continue }
    $result = $pixels[$scene]
    if ($result.same_size) {
        $lines.Add("| $($sceneNames[$scene]) | $($result.size) | $($result.differing_pixels) | $($result.max_channel_delta) |")
    } else {
        $lines.Add("| $($sceneNames[$scene]) | 尺寸不同：$($result.left) / $($result.right) | — | — |")
    }
}
$lines.Add("")
$lines.Add("### 撕裂与闪屏（每 30 ms 刷新，连续截图 4 秒）")
$lines.Add("")
$lines.Add("撕裂探测在窗口两侧画全高色条，每次刷新同时换色；一帧中任一色条出现两种颜色（少数色占比不低于 5%）即记为撕裂帧。闪屏帧是页面底色占比超出中位数 2% 的帧。")
$lines.Add("")
$lines.Add("| 探测 | 截图方式 | 对象 | 帧数 | 帧率 | 观察到的换色 | 撕裂帧 / 闪屏帧 | CPU ms/s |")
$lines.Add("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
foreach ($key in $probes.Keys) {
    $result = $probes[$key]
    $parts = $key.Split("-")
    $who = if ($parts[1] -eq "control") { "对照：无双缓冲 WinForms" } else { $labels[$parts[1]] }
    $how = if ($parts[2] -eq "screen") { "桌面截屏" } else { "PrintWindow" }
    if ($parts[0] -eq "tear") {
        $lines.Add(("| 撕裂 | {0} | {1} | {2} | {3:N1} | {4} | {5} | {6:N1} |" -f $how, $who, $result.frames, $result.fps, $result.color_changes_seen, $result.torn_frames, $result.cpu_ms_per_second))
    } else {
        $lines.Add(("| 闪屏 | {0} | {1} | {2} | {3:N1} | — | {4} | {5:N1} |" -f $how, $who, $result.frames, $result.fps, $result.flicker_frames, $result.cpu_ms_per_second))
    }
}
$lines.Add("")
$uncovered = [Collections.Generic.List[string]]::new()
if (-not $secondMonitor) { $uncovered.Add("多显示器（本机只有 $($monitors.Count) 块显示器）") }
if ($TransitionScale -le 0) { $uncovered.Add("运行中 DPI 切换") }
$uncovered.Add("合成器首帧时间")
$uncovered.Add("30 分钟长时循环")
$lines.Add("未覆盖：$($uncovered -join '、')。截图、差异图与逐进程数据在本次运行的构件中。")

$summary = Join-Path $root "summary.md"
$lines | Set-Content -Path $summary -Encoding utf8
$results | ConvertTo-Json -Depth 10 | Set-Content -Path (Join-Path $root "summary.json") -Encoding utf8
Write-Host "Wrote $summary"
