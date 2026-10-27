param(
    [int]$MemorySamples = 6,
    [int]$StartupRuns = 5,
    [int]$WarmupSeconds = 3,
    [int]$CpuSampleSeconds = 3,
    [string]$SupportRoot = "",
    [switch]$SkipBuild,
    [switch]$DeviceDemo,
    [switch]$BuildOnly,
    [ValidateRange(1, 10)][int]$RepeatRuns = 1,
    [switch]$IncludeGpuMemory
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$script:processIdentities = @{}
$script:measurementRound = 1
$script:launchNumber = 0
$script:deviceBuildFailures = [Collections.Generic.List[string]]::new()

function Get-Median {
    param([object[]]$Values)
    $sorted = @($Values | Sort-Object)
    if ($sorted.Count -eq 0) { throw "Cannot summarize an empty sample" }
    $middle = [int][Math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2) { return [double]$sorted[$middle] }
    return ([double]$sorted[$middle - 1] + [double]$sorted[$middle]) / 2.0
}

if (-not $IsWindows) {
    throw "The first performance-matrix runner uses Windows process and DWM counters"
}
if ($MemorySamples -lt 1 -or $StartupRuns -lt 2 -or $WarmupSeconds -lt 1 -or $CpuSampleSeconds -lt 1) {
    throw "MemorySamples must be >= 1, StartupRuns >= 2, WarmupSeconds >= 1 and CpuSampleSeconds >= 1"
}

$workspace = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
if ([string]::IsNullOrWhiteSpace($SupportRoot)) {
    $SupportRoot = Join-Path (Split-Path -Parent $workspace) "zsui-ui-benchmark-support"
}
$support = [IO.Path]::GetFullPath($SupportRoot)
if (
    $support.Equals($workspace, [StringComparison]::OrdinalIgnoreCase) -or
    $support.StartsWith($workspace + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
) {
    throw "SupportRoot must stay outside the zsui Git workspace"
}

$targetDir = Join-Path $support "target"
$binaryDir = Join-Path $support "bin-performance-matrix"
$outputDir = Join-Path $support "results\ui-performance-matrix\latest"
New-Item -ItemType Directory -Force -Path $binaryDir, $outputDir | Out-Null
if (-not $SkipBuild) { New-Item -ItemType Directory -Force -Path $targetDir | Out-Null }

$manifests = [ordered]@{
    zsui = Join-Path $workspace "Cargo.toml"
    egui = Join-Path $workspace "comparisons\egui_notepad\Cargo.toml"
    iced = Join-Path $workspace "comparisons\iced_notepad\Cargo.toml"
    slint = Join-Path $workspace "comparisons\slint_notepad\Cargo.toml"
    tauri = Join-Path $workspace "comparisons\tauri_notepad\Cargo.toml"
    gpui = Join-Path $workspace "comparisons\gpui_kit\Cargo.toml"
}
$viewerDocument = Join-Path $workspace "examples\ui-documents\performance-viewer.json"

function Remove-TauriGeneratedSchemas {
    $tauriRoot = [IO.Path]::GetFullPath((Join-Path $workspace "comparisons\tauri_notepad"))
    $generated = [IO.Path]::GetFullPath((Join-Path $tauriRoot "gen"))
    if (-not $generated.StartsWith($tauriRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "unsafe Tauri generated-schema path"
    }
    if (Test-Path -LiteralPath $generated) {
        Remove-Item -LiteralPath $generated -Recurse -Force
    }
}

function Invoke-CargoBuildAndCopy {
    param(
        [string]$Name,
        [string[]]$Arguments,
        [string]$BuiltExecutable,
        [string]$OutputExecutable
    )
    Write-Host "release build: $Name"
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        if ($DeviceDemo) {
            $script:deviceBuildFailures.Add($Name)
            Write-Warning "$Name release build failed; collecting diagnostics from remaining frameworks"
            return
        }
        throw "$Name release build failed"
    }
    if (-not (Test-Path -LiteralPath $BuiltExecutable)) {
        throw "$Name did not produce $BuiltExecutable"
    }
    Copy-Item -LiteralPath $BuiltExecutable -Destination $OutputExecutable -Force
}

function Build-PerformanceMatrix {
    $release = Join-Path $targetDir "release"
    $examples = Join-Path $release "examples"

    if ($DeviceDemo) {
        Invoke-CargoBuildAndCopy "ZSUI Device Console" @(
            "build", "--release", "--locked", "--manifest-path", $manifests.zsui,
            "--example", "device_console", "--no-default-features", "--features", "window,button,label,textbox,checkbox,slider,progress,grid"
        ) (Join-Path $examples "device_console.exe") (Join-Path $binaryDir "zsui-device.exe")
        foreach ($framework in @("egui", "iced", "slint")) {
            Invoke-CargoBuildAndCopy "$framework Device Console" @(
                "build", "--release", "--locked", "--manifest-path", $manifests[$framework],
                "--bin", "$framework-device-console"
            ) (Join-Path $release "$framework-device-console.exe") (Join-Path $binaryDir "$framework-device.exe")
        }
        Invoke-CargoBuildAndCopy "Tauri Device Console" @(
            "build", "--release", "--locked", "--manifest-path", $manifests.tauri,
            "--bin", "tauri-device-console", "--no-default-features", "--features", "device-demo"
        ) (Join-Path $release "tauri-device-console.exe") (Join-Path $binaryDir "tauri-device.exe")
        Invoke-CargoBuildAndCopy "GPUI Kit Device Console" @(
            "build", "--release", "--locked", "--manifest-path", $manifests.gpui,
            "--bin", "gpui-device-console"
        ) (Join-Path $release "gpui-device-console.exe") (Join-Path $binaryDir "gpui-device.exe")
        if ($script:deviceBuildFailures.Count -gt 0) {
            throw "Device demo builds failed: $($script:deviceBuildFailures -join ', ')"
        }
        return
    }

    Invoke-CargoBuildAndCopy "ZSUI Minimal" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.zsui,
        "--example", "ui_performance_minimal", "--no-default-features", "--features", "window,button,label"
    ) (Join-Path $examples "ui_performance_minimal.exe") (Join-Path $binaryDir "zsui-minimal.exe")
    Invoke-CargoBuildAndCopy "ZSUI Common" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.zsui,
        "--example", "invoice_tool", "--no-default-features", "--features", "window,shell,list,dialog"
    ) (Join-Path $examples "invoice_tool.exe") (Join-Path $binaryDir "zsui-common.exe")
    Invoke-CargoBuildAndCopy "ZSUI Full Native App" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.zsui,
        "--example", "component_gallery", "--no-default-features", "--features", "component-gallery-demo"
    ) (Join-Path $examples "component_gallery.exe") (Join-Path $binaryDir "zsui-full.exe")
    Invoke-CargoBuildAndCopy "ZSUI Viewer" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.zsui,
        "--bin", "zsui-viewer", "--no-default-features", "--features", "ui-viewer"
    ) (Join-Path $release "zsui-viewer.exe") (Join-Path $binaryDir "zsui-viewer.exe")

    foreach ($framework in @("egui", "iced", "slint")) {
        $prefix = if ($framework -eq "egui") { "egui" } else { $framework }
        $matrixBin = "$prefix-ui-performance"
        $invoiceBin = "$prefix-invoice-tool"
        Invoke-CargoBuildAndCopy "$framework Minimal" @(
            "build", "--release", "--locked", "--manifest-path", $manifests[$framework],
            "--bin", $matrixBin, "--features", "perf-minimal"
        ) (Join-Path $release "$matrixBin.exe") (Join-Path $binaryDir "$framework-minimal.exe")
        Invoke-CargoBuildAndCopy "$framework Common" @(
            "build", "--release", "--locked", "--manifest-path", $manifests[$framework],
            "--bin", $invoiceBin
        ) (Join-Path $release "$invoiceBin.exe") (Join-Path $binaryDir "$framework-common.exe")
        Invoke-CargoBuildAndCopy "$framework Full Native App" @(
            "build", "--release", "--locked", "--manifest-path", $manifests[$framework],
            "--bin", $matrixBin, "--features", "perf-full"
        ) (Join-Path $release "$matrixBin.exe") (Join-Path $binaryDir "$framework-full.exe")
        Invoke-CargoBuildAndCopy "$framework Viewer" @(
            "build", "--release", "--locked", "--manifest-path", $manifests[$framework],
            "--bin", $matrixBin, "--features", "perf-viewer"
        ) (Join-Path $release "$matrixBin.exe") (Join-Path $binaryDir "$framework-viewer.exe")
    }

    Invoke-CargoBuildAndCopy "Tauri Minimal" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.tauri,
        "--bin", "tauri-ui-performance", "--no-default-features", "--features", "perf-minimal"
    ) (Join-Path $release "tauri-ui-performance.exe") (Join-Path $binaryDir "tauri-minimal.exe")
    Invoke-CargoBuildAndCopy "Tauri Common" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.tauri,
        "--bin", "tauri-invoice-tool", "--no-default-features", "--features", "perf-common"
    ) (Join-Path $release "tauri-invoice-tool.exe") (Join-Path $binaryDir "tauri-common.exe")
    Invoke-CargoBuildAndCopy "Tauri Full Native App" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.tauri,
        "--bin", "tauri-ui-performance", "--no-default-features", "--features", "perf-full"
    ) (Join-Path $release "tauri-ui-performance.exe") (Join-Path $binaryDir "tauri-full.exe")
    Invoke-CargoBuildAndCopy "Tauri Viewer" @(
        "build", "--release", "--locked", "--manifest-path", $manifests.tauri,
        "--bin", "tauri-ui-performance", "--no-default-features", "--features", "perf-viewer"
    ) (Join-Path $release "tauri-ui-performance.exe") (Join-Path $binaryDir "tauri-viewer.exe")
}

$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir
try {
    if (-not $SkipBuild) {
        Build-PerformanceMatrix
    }
}
finally {
    if (-not $SkipBuild) { Remove-TauriGeneratedSchemas }
    $env:CARGO_TARGET_DIR = $previousTarget
}
if ($BuildOnly) { return }

$labels = [ordered]@{
    zsui = "ZSUI"
    egui = "eframe/egui"
    iced = "Iced"
    slint = "Slint"
    tauri = "Tauri 2 / WebView2"
}
if ($DeviceDemo) { $labels.gpui = "GPUI Kit" }
$profiles = [ordered]@{
    minimal = [ordered]@{
        name = "Minimal"
        workload = "Window + Text + Button"
        comparison_contract = "one 1000x700 window, bilingual title/body text and one button"
    }
    common = [ordered]@{
        name = "Common"
        workload = "Navigation + Form + List + Dialog"
        comparison_contract = "invoice assistant with navigation, editable form, two-row list and confirmation surface"
    }
    full = [ordered]@{
        name = "Full Native App"
        workload = "20-30 common component instances"
        comparison_contract = "invoice dashboard with 24 visible control instances across navigation, input, selection, collection, progress and action families"
    }
    viewer = [ordered]@{
        name = "Viewer"
        workload = "UiDocument + hot reload + all document components"
        comparison_contract = "single document surface, 250 ms source polling and the 26 component kinds supported by the current UiDocument schema; no editor chrome is added to comparison implementations"
    }
}

$applications = @()
if ($DeviceDemo) {
    $profiles = [ordered]@{ device = [ordered]@{
        name = "Device Console"
        workload = "Shared device data, filter, online checkbox, six selectable rows, load slider, progress and reset"
        comparison_contract = "960x680 logical client; six identical bilingual devices from one shared Rust module; initial load 42 and selection 0; no idle polling"
    } }
}
foreach ($framework in $labels.Keys) {
    foreach ($profile in $profiles.Keys) {
        $arguments = @()
        if ($profile -eq "full" -and $framework -eq "zsui") {
            $arguments = @("--page", "inputs", "--width", "1000", "--height", "700", "--benchmark-static")
        } elseif ($profile -eq "viewer") {
            if ($framework -eq "zsui") {
                $arguments = @($viewerDocument, "--width", "1000", "--height", "700", "--poll-ms", "250")
            } else {
                $arguments = @("--document", $viewerDocument)
            }
        }
        $applications += [pscustomobject]@{
            framework = $framework
            framework_name = $labels[$framework]
            profile = $profile
            profile_name = $profiles[$profile].name
            executable = Join-Path $binaryDir "$framework-$profile.exe"
            arguments = $arguments
        }
    }
}
foreach ($application in $applications) {
    if (-not (Test-Path -LiteralPath $application.executable)) {
        throw "missing matrix executable: $($application.executable)"
    }
}
$buildManifest = $null
$manifestPath = Join-Path $binaryDir 'build-manifest.json'
if ($DeviceDemo -and (Test-Path -LiteralPath $manifestPath)) {
    $buildManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    foreach ($artifact in $buildManifest.artifacts) {
        if ((Get-FileHash -LiteralPath (Join-Path $binaryDir $artifact.file)).Hash -ne $artifact.sha256) { throw 'Demo binary differs from the build manifest' }
    }
}

Add-Type -AssemblyName System.Drawing
if (-not ("ZsuiPerformanceMatrixWindow" -as [type])) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ZsuiPerformanceMatrixWindow {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool GetUpdateRect(IntPtr hwnd, IntPtr rect, bool erase);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr hwnd, int command);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] public static extern bool RedrawWindow(IntPtr hwnd, IntPtr updateRect, IntPtr updateRegion, uint flags);
    [DllImport("user32.dll")] public static extern void keybd_event(byte virtualKey, byte scanCode, uint flags, UIntPtr extraInfo);
    [DllImport("dwmapi.dll")] public static extern int DwmFlush();
}
"@
}

function Start-BenchmarkProcess {
    param([string]$Executable, [string[]]$Arguments)
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    $script:launchNumber++
    $logPrefix = "{0}-{1}-{2}" -f [IO.Path]::GetFileNameWithoutExtension($Executable), $script:measurementRound, $script:launchNumber
    $start = @{
        FilePath = $Executable
        WorkingDirectory = $workspace
        PassThru = $true
        WindowStyle = "Normal"
        RedirectStandardOutput = Join-Path $outputDir "$logPrefix.stdout.log"
        RedirectStandardError = Join-Path $outputDir "$logPrefix.stderr.log"
    }
    if ($Arguments.Count -gt 0) { $start.ArgumentList = $Arguments }
    $process = Start-Process @start
    $script:processIdentities[$process.Id] = $process.StartTime.ToFileTimeUtc()
    [pscustomobject]@{ process = $process; stopwatch = $stopwatch }
}

function Wait-MainWindow {
    param([System.Diagnostics.Process]$Process, [Diagnostics.Stopwatch]$Stopwatch, [int]$TimeoutMs = 15000)
    do {
        Start-Sleep -Milliseconds 10
        $Process.Refresh()
        if ($Process.HasExited) { return [long]0 }
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) {
            [void][ZsuiPerformanceMatrixWindow]::ShowWindowAsync($Process.MainWindowHandle, 4)
            [void][ZsuiPerformanceMatrixWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-1), 30, 30, 0, 0, 0x0051)
            if ($DeviceDemo) {
                $outer = New-Object ZsuiPerformanceMatrixWindow+RECT
                $client = New-Object ZsuiPerformanceMatrixWindow+RECT
                $dpi = [ZsuiPerformanceMatrixWindow]::GetDpiForWindow($Process.MainWindowHandle)
                if ($dpi -eq 0) { throw "Window DPI unavailable" }
                [void][ZsuiPerformanceMatrixWindow]::GetWindowRect($Process.MainWindowHandle, [ref]$outer)
                [void][ZsuiPerformanceMatrixWindow]::GetClientRect($Process.MainWindowHandle, [ref]$client)
                $width = [int][Math]::Round(960 * $dpi / 96) + $outer.Right - $outer.Left - $client.Right
                $height = [int][Math]::Round(680 * $dpi / 96) + $outer.Bottom - $outer.Top - $client.Bottom
                if (-not [ZsuiPerformanceMatrixWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr]::Zero, 0, 0, $width, $height, 0x0016)) {
                    throw "Cannot establish the fixed device-demo client size"
                }
            }
            return [long]$Stopwatch.ElapsedMilliseconds
        }
    } while ($Stopwatch.ElapsedMilliseconds -lt $TimeoutMs)
    return [long]0
}

function Wait-FirstPresentedFrame {
    param([System.Diagnostics.Process]$Process, [Diagnostics.Stopwatch]$Stopwatch, [int]$TimeoutMs = 15000)
    do {
        $Process.Refresh()
        if ($Process.HasExited -or $Process.MainWindowHandle -eq [IntPtr]::Zero) { return [long]0 }
        $client = New-Object ZsuiPerformanceMatrixWindow+RECT
        if (
            [ZsuiPerformanceMatrixWindow]::IsWindowVisible($Process.MainWindowHandle) -and
            [ZsuiPerformanceMatrixWindow]::GetClientRect($Process.MainWindowHandle, [ref]$client) -and
            ($client.Right - $client.Left) -gt 0 -and ($client.Bottom - $client.Top) -gt 0
        ) {
            [void][ZsuiPerformanceMatrixWindow]::RedrawWindow(
                $Process.MainWindowHandle,
                [IntPtr]::Zero,
                [IntPtr]::Zero,
                0x0001 -bor 0x0100 -bor 0x0080
            )
            [void][ZsuiPerformanceMatrixWindow]::DwmFlush()
            if (-not [ZsuiPerformanceMatrixWindow]::GetUpdateRect($Process.MainWindowHandle, [IntPtr]::Zero, $false)) {
                return [long]$Stopwatch.ElapsedMilliseconds
            }
        }
        Start-Sleep -Milliseconds 5
    } while ($Stopwatch.ElapsedMilliseconds -lt $TimeoutMs)
    return [long]0
}

function Get-ProcessGroup {
    param([int]$RootProcessId)
    $root = Get-Process -Id $RootProcessId -ErrorAction Stop
    if (-not $script:processIdentities.ContainsKey($RootProcessId) -or
        $root.StartTime.ToFileTimeUtc() -ne $script:processIdentities[$RootProcessId]) {
        throw "Benchmark root process identity changed"
    }
    $earliestCreation = $root.StartTime.AddSeconds(-1)
    $records = @(
        Get-CimInstance Win32_Process |
            Where-Object { $_.CreationDate -ge $earliestCreation } |
            Select-Object ProcessId, ParentProcessId, CreationDate
    )
    $ids = @($RootProcessId)
    do {
        $children = @(
            $records |
                Where-Object { [int]$_.ParentProcessId -in $ids -and [int]$_.ProcessId -notin $ids } |
                ForEach-Object { [int]$_.ProcessId }
        )
        if ($children.Count -gt 0) { $ids += $children }
    } while ($children.Count -gt 0)
    @($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
}

function Get-PrivateWorkingSet {
    param([System.Diagnostics.Process[]]$Processes)
    [long]$total = 0
    foreach ($process in $Processes) {
        $sample = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($process.Id)" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($sample) { $total += [long]$sample.WorkingSetPrivate }
    }
    return $total
}

function Get-ProcessSnapshot {
    param([int]$RootProcessId)
    $processes = @(Get-ProcessGroup -RootProcessId $RootProcessId)
    foreach ($process in $processes) { try { $process.Refresh() } catch {} }
    [pscustomobject]@{
        process_count = $processes.Count
        process_names = @($processes | ForEach-Object ProcessName | Sort-Object -Unique)
        working_set_bytes = [long](($processes | Measure-Object WorkingSet64 -Sum).Sum)
        private_working_set_bytes = Get-PrivateWorkingSet -Processes $processes
        private_bytes = [long](($processes | Measure-Object PrivateMemorySize64 -Sum).Sum)
        peak_working_set_bytes = [long](($processes | Measure-Object PeakWorkingSet64 -Sum).Sum)
        cpu_seconds = [double](($processes | ForEach-Object { try { $_.TotalProcessorTime.TotalSeconds } catch { 0.0 } } | Measure-Object -Sum).Sum)
    }
}

function Get-GpuMemory {
    param([int]$RootProcessId)
    try {
        $ids = @((Get-ProcessGroup -RootProcessId $RootProcessId).Id)
        $records = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory -ErrorAction Stop |
            Where-Object { $_.Name -match '^pid_(\d+)_' -and [int]$Matches[1] -in $ids })
        [ordered]@{
            available = $true
            dedicated_bytes = [long](($records | Measure-Object DedicatedUsage -Sum).Sum)
            shared_bytes = [long](($records | Measure-Object SharedUsage -Sum).Sum)
            counter_instances = $records.Count
        }
    } catch {
        [ordered]@{ available = $false; dedicated_bytes = $null; shared_bytes = $null; counter_instances = 0; reason = $_.Exception.Message }
    }
}

function Measure-Memory {
    param([int]$RootProcessId)
    $samples = @()
    for ($index = 0; $index -lt $MemorySamples; $index++) {
        $samples += Get-ProcessSnapshot -RootProcessId $RootProcessId
        if ($index + 1 -lt $MemorySamples) { Start-Sleep -Milliseconds 350 }
    }
    [pscustomobject]@{
        sample_count = $samples.Count
        process_count = [int](($samples.process_count | Measure-Object -Maximum).Maximum)
        process_names = @($samples.process_names | Sort-Object -Unique)
        rss_bytes = [long](Get-Median $samples.working_set_bytes)
        private_rss_bytes = [long](Get-Median $samples.private_working_set_bytes)
        private_bytes = [long](Get-Median $samples.private_bytes)
        peak_rss_bytes = [long](($samples.peak_working_set_bytes | Measure-Object -Maximum).Maximum)
        pss_bytes = $null
        samples = $samples
    }
}

function Measure-Cpu {
    param([int]$RootProcessId, [IntPtr]$Window, [bool]$ForceRepaint)
    $before = Get-ProcessSnapshot -RootProcessId $RootProcessId
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    if ($ForceRepaint) {
        while ($stopwatch.Elapsed.TotalSeconds -lt $CpuSampleSeconds) {
            [void][ZsuiPerformanceMatrixWindow]::RedrawWindow(
                $Window,
                [IntPtr]::Zero,
                [IntPtr]::Zero,
                0x0001 -bor 0x0100 -bor 0x0080
            )
            Start-Sleep -Milliseconds 16
        }
    } else {
        Start-Sleep -Seconds $CpuSampleSeconds
    }
    $after = Get-ProcessSnapshot -RootProcessId $RootProcessId
    $elapsed = [Math]::Max(0.001, $stopwatch.Elapsed.TotalSeconds)
    $cpuSeconds = [Math]::Max(0.0, $after.cpu_seconds - $before.cpu_seconds)
    [pscustomobject]@{
        duration_seconds = [Math]::Round($elapsed, 4)
        cpu_seconds = [Math]::Round($cpuSeconds, 4)
        machine_percent = [Math]::Round(($cpuSeconds / ($elapsed * [Environment]::ProcessorCount)) * 100.0, 3)
        one_core_percent = [Math]::Round(($cpuSeconds / $elapsed) * 100.0, 3)
        forced_repaint = $ForceRepaint
        requested_hz = if ($ForceRepaint) { 60 } else { 0 }
    }
}

function Save-WindowScreenshot {
    param([System.Diagnostics.Process]$Process, [string]$Path)
    $Process.Refresh()
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) { return $false }
    [void][ZsuiPerformanceMatrixWindow]::ShowWindowAsync($Process.MainWindowHandle, 5)
    [void][ZsuiPerformanceMatrixWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-1), 30, 30, 0, 0, 0x0041)
    [void][ZsuiPerformanceMatrixWindow]::BringWindowToTop($Process.MainWindowHandle)
    [void][ZsuiPerformanceMatrixWindow]::SetForegroundWindow($Process.MainWindowHandle)
    Start-Sleep -Milliseconds 400
    $rect = New-Object ZsuiPerformanceMatrixWindow+RECT
    if (-not [ZsuiPerformanceMatrixWindow]::GetWindowRect($Process.MainWindowHandle, [ref]$rect)) { return $false }
    $inset = 8
    $width = [Math]::Max(1, $rect.Right - $rect.Left - (2 * $inset))
    $height = [Math]::Max(1, $rect.Bottom - $rect.Top - (2 * $inset))
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left + $inset, $rect.Top + $inset, 0, 0, $bitmap.Size)
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        [void][ZsuiPerformanceMatrixWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-2), 0, 0, 0, 0, 0x0043)
        $graphics.Dispose()
        $bitmap.Dispose()
    }
    $saved = [System.Drawing.Image]::FromFile($Path)
    try {
        if ($saved.Width -lt 700 -or $saved.Height -lt 500) {
            throw "captured screenshot is unexpectedly small: $($saved.Width)x$($saved.Height)"
        }
    }
    finally {
        $saved.Dispose()
    }
    return $true
}

function Stop-BenchmarkProcess {
    param([System.Diagnostics.Process]$Process)
    try { $Process.Refresh() } catch {}
    if ($Process.HasExited) { return }
    $group = @(Get-ProcessGroup -RootProcessId $Process.Id)
    if (-not $Process.HasExited) {
        $Process.CloseMainWindow() | Out-Null
        [void]$Process.WaitForExit(1200)
    }
    foreach ($item in $group) {
        try {
            $item.Refresh()
            if (-not $item.HasExited) { $item.Kill(); [void]$item.WaitForExit(2000) }
        } catch {}
    }
}

function Measure-EmptyWindow {
    param([pscustomobject]$Application)
    $launch = Start-BenchmarkProcess -Executable $Application.executable -Arguments @($Application.arguments + "--benchmark-empty")
    try {
        $windowMs = Wait-MainWindow -Process $launch.process -Stopwatch $launch.stopwatch
        $frameMs = Wait-FirstPresentedFrame -Process $launch.process -Stopwatch $launch.stopwatch
        if ($windowMs -eq 0 -or $frameMs -eq 0) { throw "$($Application.framework_name) $($Application.profile_name) empty window did not present" }
        Start-Sleep -Seconds $WarmupSeconds
        $memory = Measure-Memory -RootProcessId $launch.process.Id
        [pscustomobject]@{ startup_to_window_ms = $windowMs; first_frame_ms = $frameMs; memory = $memory }
    }
    finally {
        Stop-BenchmarkProcess -Process $launch.process
    }
}

function Measure-RepaintApplication {
    param([pscustomobject]$Application)
    $repaintArguments = @($Application.arguments)
    if ($Application.framework -ne "zsui" -or $DeviceDemo) {
        $repaintArguments += "--benchmark-repaint"
    }
    $launch = Start-BenchmarkProcess -Executable $Application.executable -Arguments $repaintArguments
    try {
        $windowMs = Wait-MainWindow -Process $launch.process -Stopwatch $launch.stopwatch
        $frameMs = Wait-FirstPresentedFrame -Process $launch.process -Stopwatch $launch.stopwatch
        if ($windowMs -eq 0 -or $frameMs -eq 0) {
            throw "$($Application.framework_name) $($Application.profile_name) repaint process did not present"
        }
        Start-Sleep -Seconds $WarmupSeconds
        $externalRedraw = $Application.framework -eq "zsui" -and -not $DeviceDemo
        $cpu = Measure-Cpu -RootProcessId $launch.process.Id -Window $launch.process.MainWindowHandle -ForceRepaint $externalRedraw
        $cpu.requested_hz = 60
        $cpu | Add-Member -NotePropertyName driver -NotePropertyValue $(if ($externalRedraw) { "win32_redraw_window" } else { "application_render_loop" })
        $snapshot = Get-ProcessSnapshot -RootProcessId $launch.process.Id
        $cpu | Add-Member -NotePropertyName peak_rss_bytes -NotePropertyValue $snapshot.peak_working_set_bytes
        return $cpu
    }
    finally {
        Stop-BenchmarkProcess -Process $launch.process
    }
}

function Measure-Application {
    param([pscustomobject]$Application)
    Write-Host "measure: $($Application.framework_name) / $($Application.profile_name)"
    $startupSamples = @()
    $frameSamples = @()
    $coldLikeWindow = $null
    $coldLikeFrame = $null
    $pageMemory = $null
    $hiddenMemory = $null
    $idleCpu = $null
    $repaintCpu = $null
    $screenshot = Join-Path $outputDir "$($Application.framework)-$($Application.profile)-$script:measurementRound.png"
    $captured = $false
    $clientSize = $null
    $gpuMemory = $null

    for ($run = 0; $run -lt $StartupRuns; $run++) {
        $launch = Start-BenchmarkProcess -Executable $Application.executable -Arguments $Application.arguments
        try {
            $windowMs = Wait-MainWindow -Process $launch.process -Stopwatch $launch.stopwatch
            $frameMs = Wait-FirstPresentedFrame -Process $launch.process -Stopwatch $launch.stopwatch
            if ($windowMs -eq 0 -or $frameMs -eq 0) {
                throw "$($Application.framework_name) $($Application.profile_name) did not present a main window"
            }
            $startupSamples += $windowMs
            $frameSamples += $frameMs
            if ($run -eq 0) {
                $coldLikeWindow = $windowMs
                $coldLikeFrame = $frameMs
                Start-Sleep -Seconds $WarmupSeconds
                $pageMemory = Measure-Memory -RootProcessId $launch.process.Id
                if ($IncludeGpuMemory) { $gpuMemory = Get-GpuMemory -RootProcessId $launch.process.Id }
                $client = New-Object ZsuiPerformanceMatrixWindow+RECT
                if (-not [ZsuiPerformanceMatrixWindow]::GetClientRect($launch.process.MainWindowHandle, [ref]$client)) { throw "Missing measured client rectangle" }
                $clientSize = @{ width_pixels = $client.Right; height_pixels = $client.Bottom; dpi = [ZsuiPerformanceMatrixWindow]::GetDpiForWindow($launch.process.MainWindowHandle) }
                $captured = Save-WindowScreenshot -Process $launch.process -Path $screenshot
                $idleCpu = Measure-Cpu -RootProcessId $launch.process.Id -Window $launch.process.MainWindowHandle -ForceRepaint $false
                [void][ZsuiPerformanceMatrixWindow]::ShowWindowAsync($launch.process.MainWindowHandle, 0)
                Start-Sleep -Seconds $WarmupSeconds
                if ($launch.process.HasExited -or [ZsuiPerformanceMatrixWindow]::IsWindowVisible($launch.process.MainWindowHandle)) {
                    throw "Hide phase did not reach a live hidden window"
                }
                $hiddenMemory = Measure-Memory -RootProcessId $launch.process.Id
            }
        }
        finally {
            Stop-BenchmarkProcess -Process $launch.process
        }
    }

    $empty = Measure-EmptyWindow -Application $Application
    $repaintCpu = Measure-RepaintApplication -Application $Application
    $pageMemory.peak_rss_bytes = @(
        $empty.memory.peak_rss_bytes,
        $pageMemory.peak_rss_bytes,
        $hiddenMemory.peak_rss_bytes,
        $repaintCpu.peak_rss_bytes
    ) | Measure-Object -Maximum | Select-Object -ExpandProperty Maximum
    $warmWindow = @($startupSamples | Select-Object -Skip 1 | Sort-Object)
    $warmFrame = @($frameSamples | Select-Object -Skip 1 | Sort-Object)
    $windowMedian = Get-Median $warmWindow
    $frameMedian = Get-Median $warmFrame
    [ordered]@{
        framework = $Application.framework_name
        profile = $Application.profile_name
        executable = $Application.executable
        arguments = $Application.arguments
        executable_bytes = (Get-Item -LiteralPath $Application.executable).Length
        executable_sha256 = (Get-FileHash -LiteralPath $Application.executable -Algorithm SHA256).Hash
        client = $clientSize
        gpu_memory = $gpuMemory
        cold_start = [ordered]@{
            method = "best_effort_first_launch_after_release_build"
            file_cache_purged = $false
            startup_to_window_ms = $coldLikeWindow
            first_presented_frame_ms = $coldLikeFrame
        }
        warm_start = [ordered]@{
            runs = $StartupRuns - 1
            startup_to_window_median_ms = $windowMedian
            first_presented_frame_median_ms = $frameMedian
            startup_samples_ms = @($startupSamples | Select-Object -Skip 1)
            first_frame_samples_ms = @($frameSamples | Select-Object -Skip 1)
        }
        empty_window = [ordered]@{
            startup_to_window_ms = $empty.startup_to_window_ms
            first_presented_frame_ms = $empty.first_frame_ms
            memory = $empty.memory
        }
        full_page_memory = $pageMemory
        hidden_memory = $hiddenMemory
        idle_cpu = $idleCpu
        repaint_cpu = $repaintCpu
        screenshot_captured = $captured
        screenshot = if ($captured) { $screenshot } else { $null }
    }
}

$rounds = @()
for ($roundIndex = 0; $roundIndex -lt $RepeatRuns; $roundIndex++) {
    $script:measurementRound = $roundIndex + 1
    $results = [ordered]@{}
    $frameworkOrder = @($labels.Keys)
    $frameworkOrder = @(0..($frameworkOrder.Count - 1) | ForEach-Object { $frameworkOrder[($_ + $roundIndex) % $frameworkOrder.Count] })
    foreach ($framework in $frameworkOrder) {
      $results[$framework] = [ordered]@{}
      foreach ($profile in $profiles.Keys) {
        $application = $applications | Where-Object { $_.framework -eq $framework -and $_.profile -eq $profile } | Select-Object -First 1
        $results[$framework][$profile] = Measure-Application -Application $application
        $memoryPhases = @(
            $results[$framework][$profile].empty_window.memory,
            $results[$framework][$profile].full_page_memory,
            $results[$framework][$profile].hidden_memory
        )
        if ($memoryPhases.process_names -contains "conhost") {
            throw "$($application.framework_name) $($application.profile_name) process tree is contaminated by conhost.exe"
        }
        if ($framework -ne "tauri" -and ($memoryPhases.process_count | Measure-Object -Maximum).Maximum -ne 1) {
            throw "$($application.framework_name) $($application.profile_name) unexpectedly used more than one process"
        }
      }
    }
    $rounds += ,$results
    $results | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath (Join-Path $outputDir "round-$script:measurementRound.json") -Encoding utf8
}

# Keep every process observation; summarize numeric phase metrics across runs.
$results = $rounds[0] | ConvertTo-Json -Depth 16 | ConvertFrom-Json -AsHashtable
foreach ($framework in $labels.Keys) {
  foreach ($profile in $profiles.Keys) {
    $observations = @($rounds | ForEach-Object { $_[$framework][$profile] })
    $item = $results[$framework][$profile]
    if ($IncludeGpuMemory -and @($observations | Where-Object { -not $_.gpu_memory.available }).Count -eq 0) {
        foreach ($metric in @('dedicated_bytes','shared_bytes')) {
            $item.gpu_memory[$metric] = Get-Median @($observations | ForEach-Object { $_.gpu_memory[$metric] })
        }
    }
    foreach ($phase in @('full_page_memory','hidden_memory')) {
      foreach ($metric in @('rss_bytes','private_rss_bytes','private_bytes','peak_rss_bytes')) {
        $item[$phase][$metric] = Get-Median @($observations | ForEach-Object { $_[$phase].$metric })
      }
    }
    foreach ($metric in @('rss_bytes','private_rss_bytes','private_bytes','peak_rss_bytes')) {
      $item.empty_window.memory[$metric] = Get-Median @($observations | ForEach-Object { $_.empty_window.memory.$metric })
    }
    foreach ($phase in @('idle_cpu','repaint_cpu')) {
      foreach ($metric in @('machine_percent','one_core_percent')) {
        $item[$phase][$metric] = Get-Median @($observations | ForEach-Object { $_[$phase].$metric })
      }
    }
    $item.warm_start.first_frame_samples_ms = @($observations | ForEach-Object { $_.warm_start.first_frame_samples_ms })
    $item.warm_start.startup_samples_ms = @($observations | ForEach-Object { $_.warm_start.startup_samples_ms })
    $item.warm_start.runs = $item.warm_start.first_frame_samples_ms.Count
    $item.warm_start.first_presented_frame_median_ms = Get-Median $item.warm_start.first_frame_samples_ms
    $item.warm_start.startup_to_window_median_ms = Get-Median $item.warm_start.startup_samples_ms
    $sortedFrames = @($item.warm_start.first_frame_samples_ms | Sort-Object)
    $item.warm_start.first_presented_frame_p95_ms = $sortedFrames[[Math]::Ceiling($sortedFrames.Count * 0.95) - 1]
  }
}

$gitHead = (& git -C $workspace rev-parse HEAD).Trim()
$gitDirty = -not [string]::IsNullOrWhiteSpace((& git -C $workspace status --porcelain) -join "`n")
$report = [ordered]@{
    schema = "zsui.ui-performance-matrix/v2"
    measured_at = [DateTime]::UtcNow.ToString("o")
    git = [ordered]@{ head = $gitHead; dirty = $gitDirty }
    machine = [ordered]@{
        os = [Environment]::OSVersion.VersionString
        logical_processors = [Environment]::ProcessorCount
        rustc = (& rustc --version)
        memory_samples = $MemorySamples
        startup_runs = $StartupRuns
        warmup_seconds = $WarmupSeconds
        cpu_sample_seconds = $CpuSampleSeconds
        repeat_runs = $RepeatRuns
        cpu = @(Get-CimInstance Win32_Processor | Select-Object Name, NumberOfLogicalProcessors)
        display_adapters = @(Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion)
    }
    profiles = $profiles
    implementations = $results
    build = $buildManifest
    rounds = $rounds
    methodology = [ordered]@{
        fairness = "compare frameworks only within the same profile; never compare ZSUI Minimal against another framework's Common, Full or Viewer surface"
        viewer_boundary = "ZSUI formal applications and zsui-viewer are separate build artifacts and report rows; Viewer-only polling, validation and document-component coverage do not enter formal application builds"
        cold_start = "best-effort first launch after release build; Windows file cache is not purged, so this is not a reboot-grade cold start"
        first_frame = "elapsed process-start time until a visible nonzero client area completes a top-level and descendant redraw followed by DwmFlush"
        memory = "per-process sample medians, then median across independent runs in rotated framework order; RSS is working set, Private RSS is Windows private working set, PSS is unavailable on Windows"
        hidden = "same full-page process after SW_HIDE and a second warmup interval"
        idle_cpu = "recursive process-tree CPU delta while the page is stationary"
        repaint_cpu = $(if ($DeviceDemo) { "recursive process-tree CPU delta in a separate process with timer-driven load values, slider and progress updates requested every 16 ms; actual presentation rate is not implied" } else { "recursive process-tree CPU delta in a separate process at approximately 60 Hz; ZSUI uses Win32 redraw requests and the comparison apps use their own continuous render loop" })
        tauri = "memory includes WebView2 descendants; executable size excludes the system-installed WebView2 runtime"
        gpu_memory = "optional Windows GPU Process Memory counter totals for the application process tree; shared allocations may be counted in more than one process; compositor-owned resources are not attributed to the application"
    }
}

$jsonPath = Join-Path $outputDir "report.json"
$report | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $jsonPath -Encoding utf8

function MiB([long]$Bytes) { [Math]::Round($Bytes / 1MB, 2) }
$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# UI performance matrix")
$markdown.Add("")
$markdown.Add("Each table compares equal-complexity workloads. ZSUI formal applications and Viewer are separate artifacts.")
foreach ($profile in $profiles.Keys) {
    $markdown.Add("")
    $markdown.Add("## $($profiles[$profile].name)")
    $markdown.Add("")
    $markdown.Add("| Framework | Binary | Cold-like first frame | Warm first frame | Empty RSS | Page RSS | Hidden RSS | Peak RSS | Private RSS | Idle CPU | Repaint CPU |")
    $markdown.Add("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    foreach ($framework in $labels.Keys) {
        $item = $results[$framework][$profile]
        $markdown.Add("| $($labels[$framework]) | $(MiB $item.executable_bytes) MiB | $($item.cold_start.first_presented_frame_ms) ms | $($item.warm_start.first_presented_frame_median_ms) ms | $(MiB $item.empty_window.memory.rss_bytes) MiB | $(MiB $item.full_page_memory.rss_bytes) MiB | $(MiB $item.hidden_memory.rss_bytes) MiB | $(MiB $item.full_page_memory.peak_rss_bytes) MiB | $(MiB $item.full_page_memory.private_rss_bytes) MiB | $($item.idle_cpu.machine_percent)% | $($item.repaint_cpu.machine_percent)% |")
    }
}
$markdown.Add("")
$markdown.Add("Cold-like launch does not purge the Windows file cache. PSS is unavailable on Windows; Private RSS reports private working set. Tauri memory includes WebView2 descendants.")
$markdownPath = Join-Path $outputDir "report.md"
Set-Content -LiteralPath $markdownPath -Value ($markdown -join "`n") -Encoding utf8

Write-Host "performance matrix report: $jsonPath"
Write-Host "performance matrix summary: $markdownPath"
