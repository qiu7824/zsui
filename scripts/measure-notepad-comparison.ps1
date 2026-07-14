param(
    [int]$DurationSeconds = 8,
    [int]$SampleCount = 6,
    [int]$WarmupSeconds = 5,
    [string]$SupportRoot = "",
    [switch]$SkipBuild,
    [switch]$SkipSystemNotepad,
    [switch]$PublishGallery
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

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
$outputDir = Join-Path $support "results\latest"
New-Item -ItemType Directory -Force -Path $targetDir, $outputDir | Out-Null

$manifests = [ordered]@{
    zsui = Join-Path $workspace "Cargo.toml"
    egui = Join-Path $workspace "comparisons\egui_notepad\Cargo.toml"
    iced = Join-Path $workspace "comparisons\iced_notepad\Cargo.toml"
    slint = Join-Path $workspace "comparisons\slint_notepad\Cargo.toml"
    tauri = Join-Path $workspace "comparisons\tauri_notepad\Cargo.toml"
}

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

$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir
try {
    if (-not $SkipBuild) {
        $builds = @(
            [pscustomobject]@{
                Name = "ZSUI"
                Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.zsui, "--example", "zsui_notepad", "--no-default-features", "--features", "notepad-demo")
            },
            [pscustomobject]@{ Name = "egui"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.egui) },
            [pscustomobject]@{ Name = "Iced"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.iced) },
            [pscustomobject]@{ Name = "Slint"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.slint) },
            [pscustomobject]@{ Name = "Tauri 2"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.tauri) }
        )
        foreach ($build in $builds) {
            Write-Host "release build: $($build.Name)"
            $arguments = $build.Arguments
            & cargo @arguments
            if ($LASTEXITCODE -ne 0) {
                throw "$($build.Name) release build failed"
            }
        }
    }
}
finally {
    Remove-TauriGeneratedSchemas
    $env:CARGO_TARGET_DIR = $previousTarget
}

$executables = [ordered]@{
    zsui = Join-Path $targetDir "release\examples\zsui_notepad.exe"
    egui = Join-Path $targetDir "release\egui-notepad-baseline.exe"
    iced = Join-Path $targetDir "release\iced-notepad-baseline.exe"
    slint = Join-Path $targetDir "release\slint-notepad-baseline.exe"
    tauri = Join-Path $targetDir "release\tauri-notepad-baseline.exe"
}
foreach ($entry in $executables.GetEnumerator()) {
    if (-not (Test-Path -LiteralPath $entry.Value)) {
        throw "missing $($entry.Key) release executable: $($entry.Value)"
    }
}

Add-Type -AssemblyName System.Drawing
if (-not ("ZsuiBenchmarkWindow" -as [type])) {
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
}

function Wait-MainWindow {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutMs = 8000)
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
    Start-Sleep -Milliseconds 300
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

function Get-ProcessGroup {
    param([int]$RootProcessId)
    $records = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
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
        $performance = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($process.Id)" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($performance) { $total += [long]$performance.WorkingSetPrivate }
    }
    $total
}

function Measure-RunningProcessGroup {
    param(
        [System.Diagnostics.Process]$RootProcess,
        [string]$Name,
        [string]$ScreenshotPath,
        [int]$Samples
    )
    $hasWindow = Wait-MainWindow -Process $RootProcess
    Start-Sleep -Seconds $WarmupSeconds
    $workingSets = @()
    $privateWorkingSets = @()
    $privateBytes = @()
    $processCounts = @()
    $lastProcesses = @()
    for ($index = 0; $index -lt $Samples; $index++) {
        $RootProcess.Refresh()
        if ($RootProcess.HasExited) { break }
        $processes = @(Get-ProcessGroup -RootProcessId $RootProcess.Id)
        $lastProcesses = $processes
        $processCounts += $processes.Count
        $workingSets += [long](($processes | Measure-Object WorkingSet64 -Sum).Sum)
        $privateBytes += [long](($processes | Measure-Object PrivateMemorySize64 -Sum).Sum)
        $privateWorkingSets += Get-PrivateWorkingSet -Processes $processes
        Start-Sleep -Milliseconds 450
    }
    $captured = $false
    if ($hasWindow -and $ScreenshotPath -and -not $RootProcess.HasExited) {
        $captured = Save-WindowScreenshot -Process $RootProcess -Path $ScreenshotPath
    }
    [pscustomobject]@{
        name = $Name
        root_process_id = $RootProcess.Id
        process_ids = @($lastProcesses | ForEach-Object Id)
        process_names = @($lastProcesses | ForEach-Object ProcessName | Sort-Object -Unique)
        process_count = if ($processCounts.Count) { [int](($processCounts | Measure-Object -Maximum).Maximum) } else { 0 }
        sample_count = $workingSets.Count
        working_set_bytes = if ($workingSets.Count) { [long](($workingSets | Measure-Object -Average).Average) } else { 0 }
        private_working_set_bytes = if ($privateWorkingSets.Count) { [long](($privateWorkingSets | Measure-Object -Average).Average) } else { 0 }
        private_bytes = if ($privateBytes.Count) { [long](($privateBytes | Measure-Object -Average).Average) } else { 0 }
        peak_group_working_set_bytes = if ($workingSets.Count) { [long](($workingSets | Measure-Object -Maximum).Maximum) } else { 0 }
        main_window_found = $hasWindow
        screenshot_captured = $captured
        screenshot = if ($captured) { $ScreenshotPath } else { $null }
    }
}

function Stop-ProcessGroup {
    param([System.Diagnostics.Process]$RootProcess)
    if (-not $RootProcess.HasExited) {
        $RootProcess.CloseMainWindow() | Out-Null
        [void]$RootProcess.WaitForExit(2000)
    }
    foreach ($process in @(Get-ProcessGroup -RootProcessId $RootProcess.Id)) {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
    }
}

function Start-AndMeasure {
    param([string]$Executable, [string]$Name, [string]$Screenshot)
    $lifetimeSeconds = [Math]::Max(
        $DurationSeconds,
        $WarmupSeconds + ($SampleCount * 4) + 15
    )
    $process = Start-Process -FilePath $Executable -ArgumentList @("--benchmark-seconds", "$lifetimeSeconds") -WorkingDirectory $workspace -PassThru
    try {
        Measure-RunningProcessGroup -RootProcess $process -Name $Name -ScreenshotPath $Screenshot -Samples $SampleCount
    }
    finally {
        Stop-ProcessGroup -RootProcess $process
    }
}

function Empty-SystemResult {
    [pscustomobject]@{
        name = "Windows Notepad"
        root_process_id = 0
        process_ids = @()
        process_names = @()
        process_count = 0
        sample_count = 0
        working_set_bytes = 0
        private_working_set_bytes = 0
        private_bytes = 0
        peak_group_working_set_bytes = 0
        main_window_found = $false
        screenshot_captured = $false
        screenshot = $null
        executable = $null
        binary_bytes = 0
    }
}

function Measure-WindowsNotepad {
    if ($SkipSystemNotepad) { return Empty-SystemResult }
    $before = @(Get-Process -Name Notepad -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
    $sampleFile = Join-Path $outputDir "windows-notepad-sample.txt"
    Set-Content -LiteralPath $sampleFile -Value "Windows Notepad comparison sample" -Encoding utf8
    $launcher = Start-Process -FilePath (Join-Path $env:WINDIR "System32\notepad.exe") -ArgumentList @("`"$sampleFile`"") -PassThru
    $candidate = $null
    $deadline = [DateTime]::UtcNow.AddSeconds(8)
    do {
        Start-Sleep -Milliseconds 150
        $candidate = Get-Process -Name Notepad -ErrorAction SilentlyContinue |
            Where-Object { $_.Id -notin $before } |
            Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
            Sort-Object StartTime |
            Select-Object -Last 1
    } while (-not $candidate -and [DateTime]::UtcNow -lt $deadline)
    if (-not $candidate) {
        if ($launcher -and -not $launcher.HasExited) { Stop-Process -Id $launcher.Id -Force -ErrorAction SilentlyContinue }
        return Empty-SystemResult
    }
    try {
        $result = Measure-RunningProcessGroup -RootProcess $candidate -Name "Windows Notepad" -ScreenshotPath (Join-Path $outputDir "windows-notepad.png") -Samples $SampleCount
        $executable = $null
        try { $executable = $candidate.MainModule.FileName } catch {}
        $result | Add-Member -NotePropertyName executable -NotePropertyValue $executable
        $result | Add-Member -NotePropertyName binary_bytes -NotePropertyValue $(if ($executable -and (Test-Path -LiteralPath $executable)) { (Get-Item -LiteralPath $executable).Length } else { 0 })
        return $result
    }
    finally {
        Stop-ProcessGroup -RootProcess $candidate
    }
}

function Get-SourceStats {
    param([string[]]$Paths)
    $files = @($Paths | ForEach-Object { Get-Item -LiteralPath $_ })
    $lineCount = 0
    foreach ($file in $files) {
        $lineCount += @(
            Get-Content -LiteralPath $file.FullName |
                Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
        ).Count
    }
    [pscustomobject]@{
        source_file_count = $files.Count
        source_lines = $lineCount
        source_bytes = [long](($files | Measure-Object Length -Sum).Sum)
        files = @($files | ForEach-Object { $_.FullName.Substring($workspace.Length + 1) })
    }
}

function Get-PackageCount {
    param([string]$Manifest, [string[]]$Features = @())
    $arguments = @("metadata", "--format-version", "1", "--locked", "--manifest-path", $Manifest)
    if ($null -ne $Features -and @($Features).Count -gt 0) {
        $arguments += @("--no-default-features", "--features", (@($Features) -join ","))
    }
    $metadata = (& cargo @arguments | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed for $Manifest" }
    @($metadata.resolve.nodes).Count
}

$runtime = [ordered]@{
    zsui = Start-AndMeasure -Executable $executables.zsui -Name "ZSUI Notepad" -Screenshot (Join-Path $outputDir "zsui-notepad.png")
    egui = Start-AndMeasure -Executable $executables.egui -Name "eframe/egui baseline" -Screenshot (Join-Path $outputDir "egui-notepad.png")
    iced = Start-AndMeasure -Executable $executables.iced -Name "Iced baseline" -Screenshot (Join-Path $outputDir "iced-notepad.png")
    slint = Start-AndMeasure -Executable $executables.slint -Name "Slint baseline" -Screenshot (Join-Path $outputDir "slint-notepad.png")
    tauri = Start-AndMeasure -Executable $executables.tauri -Name "Tauri 2 baseline" -Screenshot (Join-Path $outputDir "tauri-notepad.png")
}
$windowsNotepad = Measure-WindowsNotepad

$source = [ordered]@{
    zsui = Get-SourceStats -Paths @(
        (Join-Path $workspace "examples\zsui_notepad.rs")
    )
    egui = Get-SourceStats -Paths @($manifests.egui, (Join-Path $workspace "comparisons\egui_notepad\src\main.rs"))
    iced = Get-SourceStats -Paths @($manifests.iced, (Join-Path $workspace "comparisons\iced_notepad\src\main.rs"))
    slint = Get-SourceStats -Paths @($manifests.slint, (Join-Path $workspace "comparisons\slint_notepad\src\main.rs"))
    tauri = Get-SourceStats -Paths @(
        $manifests.tauri,
        (Join-Path $workspace "comparisons\tauri_notepad\build.rs"),
        (Join-Path $workspace "comparisons\tauri_notepad\tauri.conf.json"),
        (Join-Path $workspace "comparisons\tauri_notepad\capabilities\default.json"),
        (Join-Path $workspace "comparisons\tauri_notepad\src\main.rs"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\index.html"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\styles.css"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\app.js")
    )
}

$implementations = [ordered]@{}
foreach ($key in @("zsui", "egui", "iced", "slint", "tauri")) {
    $implementations[$key] = [ordered]@{
        runtime = $runtime[$key]
        source = $source[$key]
        cargo_package_count = Get-PackageCount -Manifest $manifests[$key] -Features $(if ($key -eq "zsui") { @("notepad-demo") } else { @() })
        executable = $executables[$key]
        binary_bytes = (Get-Item -LiteralPath $executables[$key]).Length
    }
}

$report = [ordered]@{
    measured_at = [DateTime]::UtcNow.ToString("o")
    machine = [ordered]@{
        os = [System.Environment]::OSVersion.VersionString
        logical_processors = [System.Environment]::ProcessorCount
        rustc = (& rustc --version)
        sample_count = $SampleCount
        duration_seconds = $DurationSeconds
        warmup_seconds = $WarmupSeconds
        support_root = $support
    }
    implementations = $implementations
    windows_notepad = $windowsNotepad
    methodology = [ordered]@{
        memory_scope = "root process plus all recursive child processes"
        task_manager_memory = "summed private working set"
        source_scope = "nonblank lines in demo-owned Rust, manifests, UI markup and frontend source; generated files excluded"
        tauri_note = "binary size excludes the Windows WebView2 system runtime; memory includes WebView2 descendants"
    }
}

$jsonPath = Join-Path $outputDir "report.json"
$report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $jsonPath -Encoding utf8

function MiB([long]$Bytes) { [Math]::Round($Bytes / 1MB, 2) }
$labels = [ordered]@{
    zsui = "ZSUI Notepad"
    egui = "eframe/egui baseline"
    iced = "Iced baseline"
    slint = "Slint baseline"
    tauri = "Tauri 2 baseline"
}
$rows = foreach ($key in $labels.Keys) {
    $item = $implementations[$key]
    "| $($labels[$key]) | $($item.runtime.process_count) | $($item.source.source_file_count) | $($item.source.source_lines) | $($item.cargo_package_count) | $(MiB $item.binary_bytes) MiB | $(MiB $item.runtime.private_working_set_bytes) MiB | $(MiB $item.runtime.working_set_bytes) MiB | $(MiB $item.runtime.private_bytes) MiB |"
}
$rows += "| Windows Notepad | $($windowsNotepad.process_count) | system app | system app | n/a | $(MiB $windowsNotepad.binary_bytes) MiB* | $(MiB $windowsNotepad.private_working_set_bytes) MiB | $(MiB $windowsNotepad.working_set_bytes) MiB | $(MiB $windowsNotepad.private_bytes) MiB |"

$markdown = @"
# Notepad UI framework comparison

Measured on ``$($report.machine.os)`` with ``$($report.machine.rustc)`` after a $WarmupSeconds-second warmup. Memory is the average of $SampleCount steady-state samples and includes each root process plus recursive child processes.

| Implementation | Processes | App files | Nonblank app lines | Cargo packages | Binary | Task Manager memory | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
$($rows -join "`n")

``*`` Windows Notepad and Tauri binary sizes exclude package/system runtime files, so they are not directly comparable to self-contained executables. Tauri memory includes WebView2 child processes.

Task Manager memory means summed private working set. Working set includes resident shared pages; private bytes is committed private virtual memory.

## Implementation shape

- ZSUI: hybrid native text service plus a self-drawn Rust document shell.
- egui: immediate-mode Rust UI.
- Iced: typed Elm-style state, message, update and view.
- Slint: declarative Slint markup with safe Rust callbacks.
- Tauri 2: HTML/CSS/JavaScript UI in the system WebView2 runtime with Rust commands.

Source counts include nonblank lines in demo-owned manifests and UI/frontend files for standalone baselines. ZSUI reuses the workspace manifest and counts only its two application source files. Generated schemas, Cargo downloads, ``target`` directories and measurement output are excluded.
"@
$markdownPath = Join-Path $outputDir "report.md"
Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding utf8

if ($PublishGallery) {
    $gallery = Join-Path $workspace "docs\images"
    $publish = [ordered]@{
        "zsui-notepad.png" = "notepad.png"
        "egui-notepad.png" = "notepad-egui.png"
        "iced-notepad.png" = "notepad-iced.png"
        "slint-notepad.png" = "notepad-slint.png"
        "tauri-notepad.png" = "notepad-tauri.png"
        "windows-notepad.png" = "notepad-windows.png"
    }
    foreach ($entry in $publish.GetEnumerator()) {
        $sourcePath = Join-Path $outputDir $entry.Key
        if (Test-Path -LiteralPath $sourcePath) {
            Copy-Item -LiteralPath $sourcePath -Destination (Join-Path $gallery $entry.Value) -Force
        }
    }
}

Write-Host "comparison support root: $support"
Write-Host "comparison report: $jsonPath"
Write-Host "comparison summary: $markdownPath"
Write-Output ($report | ConvertTo-Json -Depth 10)
