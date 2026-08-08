param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,slider,label,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the Slider accessibility probe application"
    }
} finally {
    Pop-Location
}

$targetRoot = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    Join-Path $workspace "target"
} elseif ([System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
    $env:CARGO_TARGET_DIR
} else {
    Join-Path $workspace $env:CARGO_TARGET_DIR
}
$probe = Join-Path $targetRoot "debug\examples\native_smoke_run.exe"
if (-not (Test-Path -LiteralPath $probe -PathType Leaf)) {
    throw "Slider accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "8000"
$proofRoot = Join-Path $targetRoot "windows-slider-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--slider-view"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "Slider accessibility probe exited before creating its main window"
        }
        $process.Refresh()
        $hwnd = $process.MainWindowHandle
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "Slider accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Slider
    )
    $slider = $null
    for ($attempt = 0; $attempt -lt 80 -and $null -eq $slider; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $sliders = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($sliders.Count -gt 1) {
                throw "expected one Slider element, found $($sliders.Count)"
            }
            if ($sliders.Count -eq 1) {
                $slider = $sliders.Item(0)
            }
        }
        if ($null -eq $slider) {
            Start-Sleep -Milliseconds 50
        }
    }
    if ($null -eq $slider) {
        throw "UI Automation did not expose the Slider before timeout"
    }
    if ($slider.Current.FrameworkId -ne "ZSUI" -or -not $slider.Current.IsKeyboardFocusable) {
        throw "Slider did not expose the ZSUI framework id and keyboard focus contract"
    }

    $rangeObject = $null
    if (-not $slider.TryGetCurrentPattern(
        [System.Windows.Automation.RangeValuePattern]::Pattern,
        [ref]$rangeObject
    )) {
        throw "Slider did not expose RangeValuePattern"
    }
    $range = [System.Windows.Automation.RangeValuePattern]$rangeObject
    if ($range.Current.Minimum -ne 0.0 -or
        $range.Current.Maximum -ne 100.0 -or
        $range.Current.SmallChange -ne 5.0 -or
        $range.Current.LargeChange -ne 50.0 -or
        $range.Current.IsReadOnly) {
        throw "Slider exposed an invalid adjustable range contract"
    }
    $valueBefore = $range.Current.Value
    $range.SetValue(42.0)

    $valueAfter = $null
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        $currentRangeObject = $null
        if ($slider.TryGetCurrentPattern(
            [System.Windows.Automation.RangeValuePattern]::Pattern,
            [ref]$currentRangeObject
        )) {
            $currentRange = [System.Windows.Automation.RangeValuePattern]$currentRangeObject
            if ($currentRange.Current.Value -eq 40.0) {
                $valueAfter = $currentRange.Current.Value
                break
            }
        }
        Start-Sleep -Milliseconds 50
    }
    if ($valueAfter -ne 40.0 -or $valueAfter -eq $valueBefore) {
        throw "UIA SetValue did not round 42 to the Slider step and rebuild the retained view as 40"
    }

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        control_type = "Slider"
        framework_id = $slider.Current.FrameworkId
        automation_id = $slider.Current.AutomationId
        value_before = $valueBefore
        requested_value = 42.0
        value_after = $valueAfter
        minimum = $range.Current.Minimum
        maximum = $range.Current.Maximum
        small_change = $range.Current.SmallChange
        large_change = $range.Current.LargeChange
        read_only = $range.Current.IsReadOnly
        errors = @()
    }

    if (-not $process.WaitForExit(10000)) {
        throw "Slider accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "Slider accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "Slider native proof did not produce the buffered Win32 screenshot"
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    Write-Output "Windows Slider accessibility passed: UIA adjustable range -> SetValue(42) -> retained Slider value 40"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
