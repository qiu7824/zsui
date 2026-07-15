param(
    [int]$SampleCount = 5,
    [int]$StartupRuns = 3,
    [int]$WarmupSeconds = 2,
    [string]$SupportRoot = "",
    [switch]$SkipBuild
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
$outputDir = Join-Path $support "results\invoice-workbench\latest"
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
            [pscustomobject]@{ Name = "ZSUI"; Arguments = @("build", "--release", "--locked", "--example", "invoice_workbench") },
            [pscustomobject]@{ Name = "egui"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.egui, "--bin", "egui-invoice-tool") },
            [pscustomobject]@{ Name = "Iced"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.iced, "--bin", "iced-invoice-tool") },
            [pscustomobject]@{ Name = "Slint"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.slint, "--bin", "slint-invoice-tool") },
            [pscustomobject]@{ Name = "Tauri 2"; Arguments = @("build", "--release", "--locked", "--manifest-path", $manifests.tauri, "--bin", "tauri-invoice-tool") }
        )
        foreach ($build in $builds) {
            Write-Host "release build: $($build.Name)"
            & cargo @($build.Arguments)
            if ($LASTEXITCODE -ne 0) { throw "$($build.Name) release build failed" }
        }
    }
}
finally {
    Remove-TauriGeneratedSchemas
    $env:CARGO_TARGET_DIR = $previousTarget
}

$executables = [ordered]@{
    zsui = Join-Path $targetDir "release\examples\invoice_workbench.exe"
    egui = Join-Path $targetDir "release\egui-invoice-tool.exe"
    iced = Join-Path $targetDir "release\iced-invoice-tool.exe"
    slint = Join-Path $targetDir "release\slint-invoice-tool.exe"
    tauri = Join-Path $targetDir "release\tauri-invoice-tool.exe"
}
foreach ($entry in $executables.GetEnumerator()) {
    if (-not (Test-Path -LiteralPath $entry.Value)) {
        throw "missing $($entry.Key) executable: $($entry.Value)"
    }
}

Add-Type -AssemblyName System.Drawing
if (-not ("InvoiceBenchmarkWindow" -as [type])) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class InvoiceBenchmarkWindow {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr hwnd, int command);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int width, int height, uint flags);
}
"@
}

function Wait-MainWindow {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutMs = 12000)
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    do {
        Start-Sleep -Milliseconds 50
        $Process.Refresh()
        if ($Process.HasExited) { return [long]0 }
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) { return [long]$stopwatch.ElapsedMilliseconds }
    } while ($stopwatch.ElapsedMilliseconds -lt $TimeoutMs)
    return [long]0
}

function Save-WindowScreenshot {
    param([System.Diagnostics.Process]$Process, [string]$Path)
    $Process.Refresh()
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) { return $false }
    [void][InvoiceBenchmarkWindow]::ShowWindowAsync($Process.MainWindowHandle, 5)
    # Make the measured window topmost while it is captured. SWP_NOZORDER and
    # SWP_NOACTIVATE would leave a browser or editor above it and could capture
    # unrelated desktop content instead of the benchmark window.
    [void][InvoiceBenchmarkWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-1), 30, 30, 0, 0, 0x0041)
    [void][InvoiceBenchmarkWindow]::BringWindowToTop($Process.MainWindowHandle)
    [void][InvoiceBenchmarkWindow]::SetForegroundWindow($Process.MainWindowHandle)
    Start-Sleep -Milliseconds 350
    $rect = New-Object InvoiceBenchmarkWindow+RECT
    if (-not [InvoiceBenchmarkWindow]::GetWindowRect($Process.MainWindowHandle, [ref]$rect)) { return $false }
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
        [void][InvoiceBenchmarkWindow]::SetWindowPos($Process.MainWindowHandle, [IntPtr](-2), 0, 0, 0, 0, 0x0043)
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
        $sample = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($process.Id)" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($sample) { $total += [long]$sample.WorkingSetPrivate }
    }
    return $total
}

function Measure-Application {
    param([string]$Executable, [string]$Name, [string]$Screenshot)
    $startupSamples = @()
    $working = @()
    $privateWorking = @()
    $privateBytes = @()
    $processCounts = @()
    $observedProcessNames = @()
    $captured = $false
    for ($run = 0; $run -lt $StartupRuns; $run++) {
        $process = Start-Process -FilePath $Executable -ArgumentList @("--benchmark-seconds", "30") -WorkingDirectory $workspace -PassThru
        try {
            $startupMs = Wait-MainWindow -Process $process
            if ($startupMs -eq 0) { throw "$Name did not create a main window" }
            $startupSamples += $startupMs
            if ($run -eq 0) {
                Start-Sleep -Seconds $WarmupSeconds
                for ($index = 0; $index -lt $SampleCount; $index++) {
                    $process.Refresh()
                    if ($process.HasExited) { break }
                    $processes = @(Get-ProcessGroup -RootProcessId $process.Id)
                    $processCounts += $processes.Count
                    $observedProcessNames += @($processes | ForEach-Object ProcessName)
                    $working += [long](($processes | Measure-Object WorkingSet64 -Sum).Sum)
                    $privateWorking += Get-PrivateWorkingSet -Processes $processes
                    $privateBytes += [long](($processes | Measure-Object PrivateMemorySize64 -Sum).Sum)
                    Start-Sleep -Milliseconds 400
                }
                $captured = Save-WindowScreenshot -Process $process -Path $Screenshot
            }
        }
        finally {
            $processGroup = @(Get-ProcessGroup -RootProcessId $process.Id)
            if (-not $process.HasExited) {
                $process.CloseMainWindow() | Out-Null
                [void]$process.WaitForExit(1500)
            }
            foreach ($groupProcess in $processGroup) {
                if (-not $groupProcess.HasExited) {
                    Stop-Process -Id $groupProcess.Id -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }
    $orderedStartup = @($startupSamples | Sort-Object)
    $medianStartup = $orderedStartup[[Math]::Floor($orderedStartup.Count / 2)]
    [pscustomobject]@{
        name = $Name
        startup_to_window_ms = $medianStartup
        startup_samples_ms = $startupSamples
        process_count = if ($processCounts.Count) { [int](($processCounts | Measure-Object -Maximum).Maximum) } else { 1 }
        process_names = @($observedProcessNames | Sort-Object -Unique)
        sample_count = $working.Count
        working_set_bytes = [long](($working | Measure-Object -Average).Average)
        private_working_set_bytes = [long](($privateWorking | Measure-Object -Average).Average)
        private_bytes = [long](($privateBytes | Measure-Object -Average).Average)
        screenshot_captured = $captured
        screenshot = if ($captured) { $Screenshot } else { $null }
    }
}

function Get-SourceStats {
    param([string[]]$Paths)
    $files = @($Paths | ForEach-Object { Get-Item -LiteralPath $_ })
    $lineCount = 0
    foreach ($file in $files) {
        $lineCount += @(Get-Content -LiteralPath $file.FullName | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count
    }
    [pscustomobject]@{
        source_file_count = $files.Count
        nonblank_lines = $lineCount
        source_bytes = [long](($files | Measure-Object Length -Sum).Sum)
        files = @($files | ForEach-Object { $_.FullName.Substring($workspace.Length + 1) })
    }
}

function Get-PackageCount {
    param([string]$Manifest)
    $metadata = (& cargo metadata --format-version 1 --locked --manifest-path $Manifest | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed for $Manifest" }
    @($metadata.resolve.nodes).Count
}

$labels = [ordered]@{
    zsui = "ZSUI"
    egui = "eframe/egui"
    iced = "Iced"
    slint = "Slint"
    tauri = "Tauri 2"
}
$sources = [ordered]@{
    zsui = @((Join-Path $workspace "examples\invoice_workbench.rs"))
    egui = @((Join-Path $workspace "comparisons\egui_notepad\src\bin\invoice_tool.rs"))
    iced = @((Join-Path $workspace "comparisons\iced_notepad\src\bin\invoice_tool.rs"))
    slint = @((Join-Path $workspace "comparisons\slint_notepad\src\bin\invoice_tool.rs"))
    tauri = @(
        (Join-Path $workspace "comparisons\tauri_notepad\src\bin\invoice_tool.rs"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\invoice.html"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\invoice.css"),
        (Join-Path $workspace "comparisons\tauri_notepad\frontend\invoice.js")
    )
}
$development = [ordered]@{
    zsui = [ordered]@{ started_at = "2026-07-15T12:40:26.8147102+08:00"; first_compile_at = "2026-07-15T12:41:51.1110913+08:00"; elapsed_seconds = 84.30 }
    egui = [ordered]@{ started_at = "2026-07-15T12:42:01.2242065+08:00"; first_compile_at = "2026-07-15T12:44:09.7923247+08:00"; elapsed_seconds = 128.57 }
    iced = [ordered]@{ started_at = "2026-07-15T12:44:16.8969830+08:00"; first_compile_at = "2026-07-15T12:46:10.8257665+08:00"; elapsed_seconds = 113.93 }
    slint = [ordered]@{ started_at = "2026-07-15T12:46:15.0214194+08:00"; first_compile_at = "2026-07-15T12:49:02.3620278+08:00"; elapsed_seconds = 167.34 }
    tauri = [ordered]@{ started_at = "2026-07-15T13:11:05.1433514+08:00"; first_compile_at = "2026-07-15T13:14:00.6438236+08:00"; elapsed_seconds = 175.50 }
}

$implementations = [ordered]@{}
foreach ($key in $labels.Keys) {
    $screenshot = Join-Path $outputDir "$key.png"
    $implementations[$key] = [ordered]@{
        runtime = Measure-Application -Executable $executables[$key] -Name $labels[$key] -Screenshot $screenshot
        source = Get-SourceStats -Paths $sources[$key]
        cargo_package_count = Get-PackageCount -Manifest $manifests[$key]
        binary_bytes = (Get-Item -LiteralPath $executables[$key]).Length
        development = $development[$key]
    }
    if ($implementations[$key].runtime.process_names -contains "conhost") {
        throw "$($labels[$key]) launched conhost.exe; GUI-subsystem comparison is invalid"
    }
}

$report = [ordered]@{
    measured_at = [DateTime]::UtcNow.ToString("o")
    machine = [ordered]@{
        os = [Environment]::OSVersion.VersionString
        logical_processors = [Environment]::ProcessorCount
        rustc = (& rustc --version)
        sample_count = $SampleCount
        startup_runs = $StartupRuns
        warmup_seconds = $WarmupSeconds
    }
    implementations = $implementations
    methodology = [ordered]@{
        runtime = "release builds; startup is the median time until the process exposes its first main window; memory is the average of steady-state samples from the first run and includes recursive child processes"
        source = "nonblank lines in the application-owned source file; shared framework and generated code excluded"
        development = "wall-clock prototype time from first file edit to first successful debug compile, including cold compile time and correction passes"
        tauri = "binary size excludes the installed WebView2 system runtime; runtime memory includes WebView2 child processes"
    }
}

$jsonPath = Join-Path $outputDir "report.json"
$report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $jsonPath -Encoding utf8

function MiB([long]$Bytes) { [Math]::Round($Bytes / 1MB, 2) }
$rows = foreach ($key in $labels.Keys) {
    $item = $implementations[$key]
    "| $($labels[$key]) | $($item.development.elapsed_seconds) s | $($item.runtime.startup_to_window_ms) ms | $($item.source.nonblank_lines) | $($item.cargo_package_count) | $(MiB $item.binary_bytes) MiB | $(MiB $item.runtime.private_working_set_bytes) MiB | $(MiB $item.runtime.working_set_bytes) MiB |"
}
$markdown = @"
# Invoice workbench UI comparison

| Framework | Prototype to first compile | Startup to window | App lines | Cargo packages | Binary | Private working set | Working set |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
$($rows -join "`n")

Development time is the wall-clock interval from the first source edit to the first successful debug compile. It includes cold dependency compilation and correction passes. Startup is the median of $StartupRuns launches. Runtime memory uses release builds after a $WarmupSeconds-second warmup and $SampleCount steady-state samples.
"@
$markdownPath = Join-Path $outputDir "report.md"
Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding utf8

Write-Host "comparison report: $jsonPath"
Write-Host "comparison summary: $markdownPath"
