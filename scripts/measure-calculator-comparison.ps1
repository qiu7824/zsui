param(
    [int]$SampleCount = 6,
    [int]$WarmupSeconds = 5,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$outputDir = Join-Path $workspace "target\calculator-comparison"
New-Item -ItemType Directory -Force $outputDir | Out-Null

if (-not $SkipBuild) {
    & cargo build --release --example zsui_calculator --no-default-features --features calculator-demo --manifest-path (Join-Path $workspace "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "ZSUI calculator release build failed" }
}

$zsuiExe = Join-Path $workspace "target\release\examples\zsui_calculator.exe"
if (-not (Test-Path -LiteralPath $zsuiExe)) { throw "missing ZSUI calculator executable" }

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public sealed class ZsuiCalculatorWindowInfo {
    public IntPtr Handle;
    public uint ProcessId;
    public string Title;
    public int Width;
    public int Height;
}

public static class ZsuiCalculatorBenchmarkWindow {
    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);

    public static ZsuiCalculatorWindowInfo[] VisibleWindows() {
        var result = new List<ZsuiCalculatorWindowInfo>();
        EnumWindows((hwnd, data) => {
            if (!IsWindowVisible(hwnd)) return true;
            var title = new StringBuilder(512);
            GetWindowText(hwnd, title, title.Capacity);
            if (title.Length == 0) return true;
            uint processId;
            GetWindowThreadProcessId(hwnd, out processId);
            RECT rect;
            GetWindowRect(hwnd, out rect);
            result.Add(new ZsuiCalculatorWindowInfo {
                Handle = hwnd,
                ProcessId = processId,
                Title = title.ToString(),
                Width = Math.Max(0, rect.Right - rect.Left),
                Height = Math.Max(0, rect.Bottom - rect.Top)
            });
            return true;
        }, IntPtr.Zero);
        return result.ToArray();
    }
}
"@

function Wait-MainWindow {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutMs = 6000)
    $deadline = [DateTime]::UtcNow.AddMilliseconds($TimeoutMs)
    do {
        Start-Sleep -Milliseconds 100
        $Process.Refresh()
        if ($Process.HasExited) { return [IntPtr]::Zero }
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) { return $Process.MainWindowHandle }
    } while ([DateTime]::UtcNow -lt $deadline)
    return [IntPtr]::Zero
}

function Save-WindowScreenshot {
    param([IntPtr]$WindowHandle, [string]$Path)
    if ($WindowHandle -eq [IntPtr]::Zero) { return $false }
    [void][ZsuiCalculatorBenchmarkWindow]::SetCursorPos(0, 0)
    [void][ZsuiCalculatorBenchmarkWindow]::SetWindowPos($WindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0013)
    Start-Sleep -Milliseconds 900
    $rect = New-Object ZsuiCalculatorBenchmarkWindow+RECT
    if (-not [ZsuiCalculatorBenchmarkWindow]::GetWindowRect($WindowHandle, [ref]$rect)) {
        return $false
    }
    $width = [Math]::Max(1, $rect.Right - $rect.Left)
    $height = [Math]::Max(1, $rect.Bottom - $rect.Top)
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        [void][ZsuiCalculatorBenchmarkWindow]::SetWindowPos($WindowHandle, [IntPtr](-2), 0, 0, 0, 0, 0x0013)
        $graphics.Dispose()
        $bitmap.Dispose()
    }
    return $true
}

function Get-ProcessGroupSnapshot {
    param([System.Diagnostics.Process[]]$Processes)
    $workingSet = 0L
    $privateWorkingSet = 0L
    $privateBytes = 0L
    $activeIds = @()
    $components = @()
    foreach ($process in @($Processes | Sort-Object Id -Unique)) {
        if (-not $process -or $process.HasExited) { continue }
        $process.Refresh()
        $performance = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($process.Id)" | Select-Object -First 1
        $workingSet += $process.WorkingSet64
        $privateBytes += $process.PrivateMemorySize64
        $processPrivateWorkingSet = if ($performance) { [long]$performance.WorkingSetPrivate } else { 0L }
        $privateWorkingSet += $processPrivateWorkingSet
        $activeIds += $process.Id
        $components += [pscustomobject]@{
            process_id = $process.Id
            process_name = $process.ProcessName
            working_set_bytes = $process.WorkingSet64
            private_working_set_bytes = $processPrivateWorkingSet
            private_bytes = $process.PrivateMemorySize64
        }
    }
    [pscustomobject]@{
        process_ids = $activeIds
        working_set_bytes = $workingSet
        private_working_set_bytes = $privateWorkingSet
        private_bytes = $privateBytes
        components = $components
    }
}

function Measure-ProcessGroup {
    param(
        [System.Diagnostics.Process[]]$Processes,
        [string]$Name,
        [IntPtr]$WindowHandle,
        [string]$ScreenshotPath
    )
    Start-Sleep -Seconds $WarmupSeconds
    $samples = @()
    for ($index = 0; $index -lt $SampleCount; $index++) {
        $samples += Get-ProcessGroupSnapshot -Processes $Processes
        Start-Sleep -Milliseconds 350
    }
    $captured = Save-WindowScreenshot -WindowHandle $WindowHandle -Path $ScreenshotPath
    $allIds = @($samples | ForEach-Object process_ids | Sort-Object -Unique)
    $componentRows = @()
    foreach ($sample in $samples) { $componentRows += @($sample.components) }
    $components = @(
        foreach ($processId in $allIds) {
            $rows = @($componentRows | Where-Object process_id -eq $processId)
            [pscustomobject]@{
                process_id = $processId
                process_name = ($rows | Select-Object -First 1).process_name
                private_working_set_bytes = [long](($rows | Measure-Object private_working_set_bytes -Average).Average)
                working_set_bytes = [long](($rows | Measure-Object working_set_bytes -Average).Average)
                private_bytes = [long](($rows | Measure-Object private_bytes -Average).Average)
            }
        }
    )
    [pscustomobject]@{
        name = $Name
        process_ids = $allIds
        process_count = $allIds.Count
        sample_count = $samples.Count
        private_working_set_bytes = [long](($samples | Measure-Object private_working_set_bytes -Average).Average)
        working_set_bytes = [long](($samples | Measure-Object working_set_bytes -Average).Average)
        private_bytes = [long](($samples | Measure-Object private_bytes -Average).Average)
        components = $components
        screenshot_captured = $captured
        screenshot = if ($captured) { $ScreenshotPath } else { $null }
    }
}

function Measure-ZsuiCalculator {
    $lifetimeSeconds = $WarmupSeconds + [Math]::Ceiling($SampleCount * 0.8) + 6
    $process = Start-Process -FilePath $zsuiExe -ArgumentList @("--benchmark-seconds", "$lifetimeSeconds") -WorkingDirectory $workspace -PassThru
    try {
        $window = Wait-MainWindow -Process $process
        if ($window -eq [IntPtr]::Zero) { throw "ZSUI calculator window not found" }
        $result = Measure-ProcessGroup -Processes @($process) -Name "ZSUI Calculator" -WindowHandle $window -ScreenshotPath (Join-Path $outputDir "zsui-calculator.png")
        foreach ($character in "5*6=".ToCharArray()) {
            [void][ZsuiCalculatorBenchmarkWindow]::PostMessage($window, 0x0102, [IntPtr][int][char]$character, [IntPtr]::Zero)
            Start-Sleep -Milliseconds 80
        }
        Start-Sleep -Milliseconds 3000
        $interactionScreenshot = Join-Path $outputDir "zsui-calculator-interaction.png"
        $interactionCaptured = Save-WindowScreenshot -WindowHandle $window -Path $interactionScreenshot
        $result | Add-Member -NotePropertyName executable -NotePropertyValue $zsuiExe
        $result | Add-Member -NotePropertyName binary_bytes -NotePropertyValue (Get-Item -LiteralPath $zsuiExe).Length
        $result | Add-Member -NotePropertyName interaction_screenshot_captured -NotePropertyValue $interactionCaptured
        $result | Add-Member -NotePropertyName interaction_screenshot -NotePropertyValue $(if ($interactionCaptured) { $interactionScreenshot } else { $null })
        return $result
    }
    finally {
        if (-not $process.HasExited) {
            $process.CloseMainWindow() | Out-Null
            if (-not $process.WaitForExit(2500)) { Stop-Process -Id $process.Id -Force }
        }
    }
}

function Measure-WindowsCalculator {
    $existing = @(Get-Process -Name CalculatorApp -ErrorAction SilentlyContinue)
    if ($existing.Count -gt 0) {
        throw "Close the existing Windows Calculator before running the comparison"
    }
    $visibleBefore = @([ZsuiCalculatorBenchmarkWindow]::VisibleWindows() | ForEach-Object { $_.Handle.ToInt64() })
    $launcher = Start-Process -FilePath (Join-Path $env:WINDIR "System32\calc.exe") -PassThru
    $logic = $null
    $window = $null
    $deadline = [DateTime]::UtcNow.AddSeconds(8)
    do {
        Start-Sleep -Milliseconds 150
        $logic = Get-Process -Name CalculatorApp -ErrorAction SilentlyContinue | Sort-Object StartTime | Select-Object -Last 1
        $window = [ZsuiCalculatorBenchmarkWindow]::VisibleWindows() |
            Where-Object {
                $_.Handle.ToInt64() -notin $visibleBefore -and
                $_.Title -match "Calculator|计算器" -and
                $_.Width -ge 280 -and
                $_.Height -ge 400
            } |
            Sort-Object @{ Expression = { $_.Width * $_.Height }; Descending = $true } |
            Select-Object -First 1
    } while ((-not $logic -or -not $window) -and [DateTime]::UtcNow -lt $deadline)
    if (-not $logic -or -not $window) { throw "Windows Calculator process group was not found" }

    $windowHost = Get-Process -Id $window.ProcessId
    try {
        $result = Measure-ProcessGroup -Processes @($logic, $windowHost) -Name "Windows Calculator process group" -WindowHandle $window.Handle -ScreenshotPath (Join-Path $outputDir "windows-calculator.png")
        $package = Get-AppxPackage Microsoft.WindowsCalculator
        $executable = if ($package) {
            Join-Path $package.InstallLocation "CalculatorApp.exe"
        } else {
            $logic.MainModule.FileName
        }
        $result | Add-Member -NotePropertyName executable -NotePropertyValue $executable
        $result | Add-Member -NotePropertyName binary_bytes -NotePropertyValue (Get-Item -LiteralPath $executable).Length
        $result | Add-Member -NotePropertyName logic_process_id -NotePropertyValue $logic.Id
        $result | Add-Member -NotePropertyName window_host_process_id -NotePropertyValue $windowHost.Id
        $result | Add-Member -NotePropertyName window_host_process_name -NotePropertyValue $windowHost.ProcessName
        $result | Add-Member -NotePropertyName window_host_is_separate -NotePropertyValue ($windowHost.Id -ne $logic.Id)
        return $result
    }
    finally {
        [void][ZsuiCalculatorBenchmarkWindow]::PostMessage($window.Handle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
        if (-not $logic.WaitForExit(3000)) { Stop-Process -Id $logic.Id -Force }
        if (-not $launcher.HasExited) { Stop-Process -Id $launcher.Id -Force }
    }
}

function Get-SourceStats {
    param([string[]]$Paths)
    $files = @($Paths | ForEach-Object { Get-Item -LiteralPath $_ })
    [pscustomobject]@{
        source_file_count = $files.Count
        source_lines = [long](($files | ForEach-Object { (Get-Content -LiteralPath $_.FullName | Measure-Object -Line).Lines } | Measure-Object -Sum).Sum)
        source_bytes = [long](($files | Measure-Object Length -Sum).Sum)
        files = @($files | ForEach-Object { $_.FullName.Substring($workspace.Length + 1) })
    }
}

function Get-PackageCount {
    $packages = @(
        & cargo tree --manifest-path (Join-Path $workspace "Cargo.toml") --no-default-features --features calculator-demo -e normal,build --prefix none --format "{p}" |
            ForEach-Object { $_ -replace ' \(\*\)$', '' } |
            Sort-Object -Unique
    )
    if ($LASTEXITCODE -ne 0) { throw "cargo tree failed" }
    return $packages.Count
}

$zsui = Measure-ZsuiCalculator
$windows = Measure-WindowsCalculator
$appSource = Get-SourceStats -Paths @(
    (Join-Path $workspace "examples\zsui_calculator.rs")
)
$frameworkSource = Get-SourceStats -Paths @((Join-Path $workspace "src\calculator.rs"))
$package = Get-AppxPackage Microsoft.WindowsCalculator

$report = [ordered]@{
    measured_at = [DateTime]::UtcNow.ToString("o")
    machine = [ordered]@{
        os = [System.Environment]::OSVersion.VersionString
        rustc = (& rustc --version)
        sample_count = $SampleCount
        warmup_seconds = $WarmupSeconds
    }
    zsui = [ordered]@{
        runtime = $zsui
        app_source = $appSource
        framework_source = $frameworkSource
        cargo_package_count = Get-PackageCount
        asset_file_count = 2
    }
    windows_calculator = [ordered]@{
        package_version = if ($package) { $package.Version.ToString() } else { $null }
        runtime = $windows
    }
    interpretation = [ordered]@{
        zsui_advantages = @(
            "small single-process runtime and dependency graph",
            "decimal arithmetic instead of binary floating-point input errors",
            "framework-owned Fluent shell, keyboard routing and product customization"
        )
        windows_calculator_advantages = @(
            "scientific, graphing, programmer and date modes",
            "currency and unit conversion",
            "mature localization, accessibility and operating-system integration"
        )
    }
}

$jsonPath = Join-Path $outputDir "report.json"
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding utf8

function MiB([long]$Bytes) { return [Math]::Round($Bytes / 1MB, 2) }
$windowsComponentRows = @(
    $windows.components | ForEach-Object {
        "| ``$($_.process_name)`` | $($_.process_id) | $(MiB $_.private_working_set_bytes) MiB | $(MiB $_.working_set_bytes) MiB | $(MiB $_.private_bytes) MiB |"
    }
) -join "`r`n"
$windowOwnershipNote = if ($windows.window_host_is_separate) {
    "This run used a separate ``$($windows.window_host_process_name)`` for the visible window. That host can be shared by packaged applications, so the summed row is a process-group observation rather than an isolated framework allocation."
} else {
    "This run kept the visible window in ``CalculatorApp`` itself, so the Windows row contains one process."
}
$markdown = @"
# Calculator implementation comparison

Measured on ``$($report.machine.os)`` after a $WarmupSeconds-second warmup. "Task Manager memory" is the summed private working set for each implementation's process group.

| Implementation | Processes | App files | App lines | Cargo packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI Calculator | $($zsui.process_count) | $($appSource.source_file_count) | $($appSource.source_lines) | $($report.zsui.cargo_package_count) | $(MiB $zsui.binary_bytes) MiB | $(MiB $zsui.private_working_set_bytes) MiB | $(MiB $zsui.working_set_bytes) MiB | $(MiB $zsui.private_bytes) MiB |
| Windows Calculator | $($windows.process_count) | system app | system app | n/a | $(MiB $windows.binary_bytes) MiB* | $(MiB $windows.private_working_set_bytes) MiB | $(MiB $windows.working_set_bytes) MiB | $(MiB $windows.private_bytes) MiB |

The reusable ZSUI calculator engine and shell contain $($frameworkSource.source_lines) framework lines. `*` The packaged CalculatorApp executable excludes package assets and the ApplicationFrameHost executable, so binary sizes are not directly comparable.

Windows Calculator is measured as CalculatorApp plus the process that owns its visible window when that process is separate. $windowOwnershipNote

## Windows process components

| Process | PID | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: |
$windowsComponentRows

## Result

- ZSUI covers the complete standard four-function workflow, unary functions, memory, history, keyboard input and decimal arithmetic in a small customizable process.
- Windows Calculator remains substantially broader in modes, converters, localization, accessibility and system integration.
- File count and memory do not establish product parity; screenshots and function coverage must be reviewed together.
"@
$markdownPath = Join-Path $outputDir "report.md"
Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding utf8

Write-Host "calculator report: $jsonPath"
Write-Host "calculator summary: $markdownPath"
Write-Output ($report | ConvertTo-Json -Depth 8)
