param(
    [int]$DurationSeconds = 8,
    [int]$SampleCount = 6,
    [int]$WarmupSeconds = 5,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$outputDir = Join-Path $workspace "target\notepad-comparison"
New-Item -ItemType Directory -Force $outputDir | Out-Null

if (-not $SkipBuild) {
    & cargo build --release --example zsui_notepad --features notepad-demo --manifest-path (Join-Path $workspace "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "ZSUI release build failed" }
    & cargo build --release --manifest-path (Join-Path $workspace "comparisons\egui_notepad\Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "egui release build failed" }
}

$zsuiExe = Join-Path $workspace "target\release\examples\zsui_notepad.exe"
$eguiExe = Join-Path $workspace "comparisons\egui_notepad\target\release\egui-notepad-baseline.exe"
if (-not (Test-Path -LiteralPath $zsuiExe)) { throw "missing ZSUI release executable" }
if (-not (Test-Path -LiteralPath $eguiExe)) { throw "missing egui release executable" }

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ZsuiBenchmarkWindow {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")]
    public static extern bool BringWindowToTop(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindowAsync(IntPtr hwnd, int command);
    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(IntPtr hwnd, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
}
"@

function Wait-MainWindow {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutMs = 5000)
    $deadline = [DateTime]::UtcNow.AddMilliseconds($TimeoutMs)
    do {
        Start-Sleep -Milliseconds 100
        $Process.Refresh()
        if ($Process.HasExited) { return $false }
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) { return $true }
    } while ([DateTime]::UtcNow -lt $deadline)
    return $false
}

function Save-WindowScreenshot {
    param([System.Diagnostics.Process]$Process, [string]$Path)
    $Process.Refresh()
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) { return $false }
    [void][ZsuiBenchmarkWindow]::ShowWindowAsync($Process.MainWindowHandle, 5)
    [void][ZsuiBenchmarkWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0043)
    [void][ZsuiBenchmarkWindow]::BringWindowToTop($Process.MainWindowHandle)
    [void][ZsuiBenchmarkWindow]::SetForegroundWindow($Process.MainWindowHandle)
    Start-Sleep -Milliseconds 250
    $rect = New-Object ZsuiBenchmarkWindow+RECT
    if (-not [ZsuiBenchmarkWindow]::GetWindowRect($Process.MainWindowHandle, [ref]$rect)) {
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
        [void][ZsuiBenchmarkWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-2), 0, 0, 0, 0, 0x0043)
        $graphics.Dispose()
        $bitmap.Dispose()
    }
    return $true
}

function Measure-RunningProcess {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$Name,
        [string]$ScreenshotPath,
        [int]$Samples
    )
    $hasWindow = Wait-MainWindow -Process $Process
    Start-Sleep -Seconds $WarmupSeconds
    $workingSets = @()
    $privateWorkingSets = @()
    $privateBytes = @()
    $peaks = @()
    for ($index = 0; $index -lt $Samples; $index++) {
        if ($Process.HasExited) { break }
        $Process.Refresh()
        $workingSets += $Process.WorkingSet64
        $performance = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($Process.Id)" | Select-Object -First 1
        if ($performance) {
            $privateWorkingSets += $performance.WorkingSetPrivate
        }
        $privateBytes += $Process.PrivateMemorySize64
        $peaks += $Process.PeakWorkingSet64
        Start-Sleep -Milliseconds 350
    }
    $captured = $false
    if ($hasWindow -and $ScreenshotPath) {
        $captured = Save-WindowScreenshot -Process $Process -Path $ScreenshotPath
    }
    [pscustomobject]@{
        name = $Name
        process_id = $Process.Id
        sample_count = $workingSets.Count
        working_set_bytes = if ($workingSets.Count) { [long](($workingSets | Measure-Object -Average).Average) } else { 0 }
        private_working_set_bytes = if ($privateWorkingSets.Count) { [long](($privateWorkingSets | Measure-Object -Average).Average) } else { 0 }
        private_bytes = if ($privateBytes.Count) { [long](($privateBytes | Measure-Object -Average).Average) } else { 0 }
        peak_working_set_bytes = if ($peaks.Count) { [long](($peaks | Measure-Object -Maximum).Maximum) } else { 0 }
        main_window_found = $hasWindow
        screenshot_captured = $captured
        screenshot = if ($captured) { $ScreenshotPath } else { $null }
    }
}

function Start-AndMeasure {
    param([string]$Executable, [string]$Name, [string]$Screenshot)
    $lifetimeSeconds = $WarmupSeconds + [Math]::Ceiling($SampleCount * 0.35) + 6
    $process = Start-Process -FilePath $Executable -ArgumentList @("--benchmark-seconds", "$lifetimeSeconds") -WorkingDirectory $workspace -PassThru
    try {
        return Measure-RunningProcess -Process $process -Name $Name -ScreenshotPath $Screenshot -Samples $SampleCount
    }
    finally {
        if (-not $process.HasExited) {
            $process.CloseMainWindow() | Out-Null
            if (-not $process.WaitForExit(2000)) {
                Stop-Process -Id $process.Id -Force
            }
        }
    }
}

function Measure-WindowsNotepad {
    $before = @(Get-Process -Name Notepad -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
    $sampleFile = Join-Path $outputDir "windows-notepad-sample.txt"
    Set-Content -LiteralPath $sampleFile -Value "Windows Notepad comparison sample" -Encoding utf8
    $launcher = Start-Process -FilePath (Join-Path $env:WINDIR "System32\notepad.exe") -ArgumentList @("`"$sampleFile`"") -PassThru
    $candidate = $null
    $deadline = [DateTime]::UtcNow.AddSeconds(6)
    do {
        Start-Sleep -Milliseconds 150
        $candidate = Get-Process -Name Notepad -ErrorAction SilentlyContinue |
            Where-Object { $_.Id -notin $before } |
            Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
            Sort-Object StartTime |
            Select-Object -Last 1
    } while (-not $candidate -and [DateTime]::UtcNow -lt $deadline)
    if (-not $candidate) {
        return [pscustomobject]@{
            name = "Windows Notepad"
            process_id = 0
            sample_count = 0
            working_set_bytes = 0
            private_working_set_bytes = 0
            private_bytes = 0
            peak_working_set_bytes = 0
            main_window_found = $false
            screenshot_captured = $false
            screenshot = $null
            executable = $null
            binary_bytes = 0
        }
    }
    try {
        $result = Measure-RunningProcess -Process $candidate -Name "Windows Notepad" -ScreenshotPath (Join-Path $outputDir "windows-notepad.png") -Samples $SampleCount
        $executable = $null
        try { $executable = $candidate.MainModule.FileName } catch {}
        $result | Add-Member -NotePropertyName executable -NotePropertyValue $executable
        $result | Add-Member -NotePropertyName binary_bytes -NotePropertyValue $(if ($executable -and (Test-Path -LiteralPath $executable)) { (Get-Item -LiteralPath $executable).Length } else { 0 })
        return $result
    }
    finally {
        if (-not $candidate.HasExited) {
            $candidate.CloseMainWindow() | Out-Null
            if (-not $candidate.WaitForExit(2500)) {
                Stop-Process -Id $candidate.Id -Force
            }
        }
    }
}

function Get-SourceStats {
    param([string[]]$Paths)
    $files = @($Paths | ForEach-Object { Get-Item -LiteralPath $_ })
    $lineCount = 0
    foreach ($file in $files) {
        $lineCount += (Get-Content -LiteralPath $file.FullName | Measure-Object -Line).Lines
    }
    [pscustomobject]@{
        source_file_count = $files.Count
        source_lines = $lineCount
        source_bytes = [long](($files | Measure-Object Length -Sum).Sum)
        files = @($files | ForEach-Object { $_.FullName.Substring($workspace.Length + 1) })
    }
}

function Get-PackageCount {
    param([string]$Manifest, [string[]]$Features)
    $arguments = @("metadata", "--format-version", "1", "--manifest-path", $Manifest)
    if ($Features.Count) {
        $arguments += @("--no-default-features", "--features", ($Features -join ","))
    }
    $metadata = (& cargo @arguments | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed for $Manifest" }
    return @($metadata.resolve.nodes).Count
}

$zsui = Start-AndMeasure -Executable $zsuiExe -Name "ZSUI Notepad" -Screenshot (Join-Path $outputDir "zsui-notepad.png")
$egui = Start-AndMeasure -Executable $eguiExe -Name "eframe/egui baseline" -Screenshot (Join-Path $outputDir "egui-notepad.png")
$windowsNotepad = Measure-WindowsNotepad

$zsui | Add-Member -NotePropertyName executable -NotePropertyValue $zsuiExe
$zsui | Add-Member -NotePropertyName binary_bytes -NotePropertyValue (Get-Item -LiteralPath $zsuiExe).Length
$egui | Add-Member -NotePropertyName executable -NotePropertyValue $eguiExe
$egui | Add-Member -NotePropertyName binary_bytes -NotePropertyValue (Get-Item -LiteralPath $eguiExe).Length

$zsuiSource = Get-SourceStats -Paths @(
    (Join-Path $workspace "examples\zsui_notepad.rs"),
    (Join-Path $workspace "examples\zsui_notepad\document.rs"),
    (Join-Path $workspace "examples\zsui_notepad\windows.rs")
)
$eguiSource = Get-SourceStats -Paths @(
    (Join-Path $workspace "comparisons\egui_notepad\Cargo.toml"),
    (Join-Path $workspace "comparisons\egui_notepad\src\main.rs")
)

$report = [ordered]@{
    measured_at = [DateTime]::UtcNow.ToString("o")
    machine = [ordered]@{
        os = [System.Environment]::OSVersion.VersionString
        logical_processors = [System.Environment]::ProcessorCount
        rustc = (& rustc --version)
        sample_count = $SampleCount
        duration_seconds = $DurationSeconds
        warmup_seconds = $WarmupSeconds
    }
    zsui = [ordered]@{
        runtime = $zsui
        source = $zsuiSource
        cargo_package_count = Get-PackageCount -Manifest (Join-Path $workspace "Cargo.toml") -Features @("notepad-demo")
        asset_file_count = 2
    }
    egui = [ordered]@{
        runtime = $egui
        source = $eguiSource
        cargo_package_count = Get-PackageCount -Manifest (Join-Path $workspace "comparisons\egui_notepad\Cargo.toml") -Features @()
        asset_file_count = 1
    }
    windows_notepad = $windowsNotepad
    interpretation = [ordered]@{
        zsui_advantages = @(
            "smaller dependency graph and binary",
            "lower renderer/runtime overhead for this native-text-service sample",
            "framework-controlled feature pruning and product-specific composite UI",
            "safe Rust document model around isolated Win32 host code"
        )
        egui_advantages = @(
            "much less platform plumbing for menus, layout, dialogs and editor presentation",
            "cross-platform application code",
            "faster AI implementation for generic utility UI at current ZSUI maturity"
        )
        windows_notepad_advantages = @(
            "mature text engine, IME and accessibility integration",
            "tabs, session restore, search/replace, printing, encoding and line-ending options",
            "spell checking and operating-system lifecycle integration"
        )
    }
}

$jsonPath = Join-Path $outputDir "report.json"
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding utf8

function MiB([long]$Bytes) { return [Math]::Round($Bytes / 1MB, 2) }
$markdown = @"
# Notepad implementation comparison

Measured on ``$($report.machine.os)`` with ``$($report.machine.rustc)`` after a $WarmupSeconds-second warmup. Memory is the average of $SampleCount steady-state samples and will vary between runs.

| Implementation | App files | App lines | Cargo packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ZSUI Notepad | $($zsuiSource.source_file_count) | $($zsuiSource.source_lines) | $($report.zsui.cargo_package_count) | $(MiB $zsui.binary_bytes) MiB | $(MiB $zsui.private_working_set_bytes) MiB | $(MiB $zsui.working_set_bytes) MiB | $(MiB $zsui.private_bytes) MiB |
| eframe/egui baseline | $($eguiSource.source_file_count) | $($eguiSource.source_lines) | $($report.egui.cargo_package_count) | $(MiB $egui.binary_bytes) MiB | $(MiB $egui.private_working_set_bytes) MiB | $(MiB $egui.working_set_bytes) MiB | $(MiB $egui.private_bytes) MiB |
| Windows Notepad | system app | system app | n/a | $(MiB $windowsNotepad.binary_bytes) MiB* | $(MiB $windowsNotepad.private_working_set_bytes) MiB | $(MiB $windowsNotepad.working_set_bytes) MiB | $(MiB $windowsNotepad.private_bytes) MiB |

`*` The packaged Windows Notepad executable size may not include framework and package files, so it is not directly comparable to a self-contained Rust executable.

Task Manager memory is the private working set. Working set includes resident shared pages; private bytes measures committed private virtual memory. These counters are not interchangeable.

## Result

- ZSUI wins when dependency control, a small native host, feature pruning and custom product composites matter.
- The ZSUI sample currently needs $($zsuiSource.source_lines) app-level lines versus $($eguiSource.source_lines) for the egui baseline because native editor, file-dialog, accelerator and lifecycle plumbing are not yet reusable framework services.
- eframe/egui currently requires substantially less platform code for a generic cross-platform utility, so it is easier for AI to produce quickly.
- Windows Notepad remains the product-quality reference for text editing, accessibility, IME, tabs, restore, search, print and system integration. ZSUI does not currently have a functional advantage over it as a general-purpose notepad.
"@
$markdownPath = Join-Path $outputDir "report.md"
Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding utf8

Write-Host "comparison report: $jsonPath"
Write-Host "comparison summary: $markdownPath"
Write-Output ($report | ConvertTo-Json -Depth 8)
