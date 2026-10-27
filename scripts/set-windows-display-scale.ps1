param(
    [int]$Scale = 0,
    [int]$Width = 0,
    [int]$Height = 0,
    [switch]$Query
)

# Changes the primary display's resolution and per-monitor scale on the
# running desktop session, the way the Settings app does, and reports the
# effective DPI that a per-monitor-aware process observes afterwards. Used by
# scripts/measure-proxy-demo-comparison.ps1 for high-DPI and DPI-transition
# measurements; it does not require signing out.

$ErrorActionPreference = "Stop"
if (-not ('ZsuiDisplayScale' -as [type])) {
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Runtime.InteropServices;
using System.Threading;

public static class ZsuiDisplayScale
{
    [StructLayout(LayoutKind.Sequential)] struct LUID { public uint LowPart; public int HighPart; }

    [StructLayout(LayoutKind.Sequential)]
    struct HEADER { public int type; public uint size; public LUID adapterId; public uint id; }

    [StructLayout(LayoutKind.Sequential)]
    struct SCALE_GET { public HEADER header; public int minScaleRel; public int curScaleRel; public int maxScaleRel; }

    [StructLayout(LayoutKind.Sequential)]
    struct SCALE_SET { public HEADER header; public int scaleRel; }

    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DEVMODE
    {
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
        public short dmSpecVersion; public short dmDriverVersion; public short dmSize; public short dmDriverExtra;
        public int dmFields; public int dmPositionX; public int dmPositionY;
        public int dmDisplayOrientation; public int dmDisplayFixedOutput;
        public short dmColor; public short dmDuplex; public short dmYResolution; public short dmTTOption; public short dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
        public short dmLogPixels; public int dmBitsPerPel; public int dmPelsWidth; public int dmPelsHeight;
        public int dmDisplayFlags; public int dmDisplayFrequency; public int dmICMMethod; public int dmICMIntent;
        public int dmMediaType; public int dmDitherType; public int dmReserved1; public int dmReserved2;
        public int dmPanningWidth; public int dmPanningHeight;
    }

    delegate bool MonitorEnumProc(IntPtr monitor, IntPtr dc, ref RECT rect, IntPtr data);

    [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
    [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] byte[] paths, ref uint modeCount, [Out] byte[] modes, IntPtr topology);
    [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref SCALE_GET request);
    [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref SCALE_SET request);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern bool EnumDisplaySettingsW(string device, int mode, ref DEVMODE devMode);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int ChangeDisplaySettingsExW(string device, ref DEVMODE devMode, IntPtr window, uint flags, IntPtr parameter);
    [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, MonitorEnumProc callback, IntPtr data);
    [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr monitor, int type, out uint dpiX, out uint dpiY);

    static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

    static HEADER PrimarySource()
    {
        uint pathCount, modeCount;
        if (GetDisplayConfigBufferSizes(2, out pathCount, out modeCount) != 0) throw new InvalidOperationException("GetDisplayConfigBufferSizes failed");
        var paths = new byte[pathCount * 72];
        var modes = new byte[modeCount * 64];
        if (QueryDisplayConfig(2, ref pathCount, paths, ref modeCount, modes, IntPtr.Zero) != 0 || pathCount == 0) throw new InvalidOperationException("QueryDisplayConfig failed");
        return new HEADER
        {
            adapterId = new LUID { LowPart = BitConverter.ToUInt32(paths, 0), HighPart = BitConverter.ToInt32(paths, 4) },
            id = BitConverter.ToUInt32(paths, 8)
        };
    }

    /// Returns recommended, current and maximum scale in percent.
    public static int[] GetScale()
    {
        var request = new SCALE_GET { header = PrimarySource() };
        request.header.type = -3;
        request.header.size = (uint)Marshal.SizeOf(typeof(SCALE_GET));
        if (DisplayConfigGetDeviceInfo(ref request) != 0) throw new InvalidOperationException("reading the display scale failed");
        int recommended = Math.Abs(request.minScaleRel);
        Func<int, int> step = index => Steps[Math.Max(0, Math.Min(Steps.Length - 1, index))];
        return new[] { step(recommended), step(recommended + request.curScaleRel), step(recommended + request.maxScaleRel) };
    }

    public static void SetScale(int percent)
    {
        var current = new SCALE_GET { header = PrimarySource() };
        current.header.type = -3;
        current.header.size = (uint)Marshal.SizeOf(typeof(SCALE_GET));
        if (DisplayConfigGetDeviceInfo(ref current) != 0) throw new InvalidOperationException("reading the display scale failed");
        int recommended = Math.Abs(current.minScaleRel);
        int target = Array.IndexOf(Steps, percent);
        if (target < 0 || target - recommended > current.maxScaleRel || target - recommended < current.minScaleRel)
        {
            throw new InvalidOperationException(percent + "% is not offered by this display");
        }
        var request = new SCALE_SET { header = current.header, scaleRel = target - recommended };
        request.header.type = -4;
        request.header.size = (uint)Marshal.SizeOf(typeof(SCALE_SET));
        if (DisplayConfigSetDeviceInfo(ref request) != 0) throw new InvalidOperationException("setting the display scale failed");
    }

    public static string SetResolution(int width, int height)
    {
        var best = new DEVMODE { dmSize = (short)Marshal.SizeOf(typeof(DEVMODE)) };
        bool found = false;
        var mode = new DEVMODE { dmSize = (short)Marshal.SizeOf(typeof(DEVMODE)) };
        var offered = new List<string>();
        for (int index = 0; EnumDisplaySettingsW(null, index, ref mode); index++)
        {
            string name = mode.dmPelsWidth + "x" + mode.dmPelsHeight;
            if (!offered.Contains(name)) offered.Add(name);
            if (mode.dmPelsWidth == width && mode.dmPelsHeight == height && mode.dmBitsPerPel == 32)
            {
                best = mode;
                found = true;
            }
            mode = new DEVMODE { dmSize = (short)Marshal.SizeOf(typeof(DEVMODE)) };
        }
        if (!found) throw new InvalidOperationException(width + "x" + height + " is not offered; modes: " + string.Join(", ", offered));
        best.dmFields = 0x80000 | 0x100000 | 0x40000;
        int result = ChangeDisplaySettingsExW(null, ref best, IntPtr.Zero, 0, IntPtr.Zero);
        if (result != 0) throw new InvalidOperationException("ChangeDisplaySettingsEx returned " + result);
        return string.Join(", ", offered);
    }

    /// Effective DPI of every monitor as a per-monitor-aware process sees it.
    public static string Monitors()
    {
        IntPtr previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
        var entries = new List<string>();
        try
        {
            EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (IntPtr monitor, IntPtr dc, ref RECT rect, IntPtr data) =>
            {
                uint dpiX, dpiY;
                GetDpiForMonitor(monitor, 0, out dpiX, out dpiY);
                entries.Add(string.Format(CultureInfo.InvariantCulture,
                    "{{\"left\":{0},\"top\":{1},\"width\":{2},\"height\":{3},\"dpi\":{4}}}",
                    rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top, dpiX));
                return true;
            }, IntPtr.Zero);
        }
        finally
        {
            SetThreadDpiAwarenessContext(previous);
        }
        return "[" + string.Join(",", entries) + "]";
    }

    public static bool WaitForDpi(uint dpi, int timeoutMs)
    {
        var deadline = DateTime.UtcNow.AddMilliseconds(timeoutMs);
        while (DateTime.UtcNow < deadline)
        {
            if (Monitors().Contains("\"dpi\":" + dpi + "}")) return true;
            Thread.Sleep(100);
        }
        return false;
    }
}
'@
}

$report = [ordered]@{}
if ($Width -gt 0 -and $Height -gt 0) {
    $report.offered_modes = [ZsuiDisplayScale]::SetResolution($Width, $Height)
    Start-Sleep -Milliseconds 1500
}
if ($Scale -gt 0) {
    [ZsuiDisplayScale]::SetScale($Scale)
    $dpi = [uint32][Math]::Round(96 * $Scale / 100)
    if (-not [ZsuiDisplayScale]::WaitForDpi($dpi, 10000)) {
        throw "the display did not reach $Scale% ($dpi DPI): $([ZsuiDisplayScale]::Monitors())"
    }
    Start-Sleep -Milliseconds 1500
}
try {
    $scaleInfo = [ZsuiDisplayScale]::GetScale()
    $report.recommended_scale = $scaleInfo[0]
    $report.current_scale = $scaleInfo[1]
    $report.maximum_scale = $scaleInfo[2]
} catch {
    # A query must not stop a measurement on adapters without scale steps.
    $report.scale_error = "$_"
}
$report.monitors = [ZsuiDisplayScale]::Monitors() | ConvertFrom-Json
$report | ConvertTo-Json -Depth 4 -Compress
