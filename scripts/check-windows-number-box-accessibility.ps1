param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,number-box,label,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the NumberBox accessibility probe application"
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
    throw "NumberBox accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "8000"
$proofRoot = Join-Path $targetRoot "windows-number-box-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--number-box-view"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "NumberBox accessibility probe exited before creating its main window"
        }
        $process.Refresh()
        $hwnd = $process.MainWindowHandle
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "NumberBox accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Spinner
    )
    $numberBox = $null
    for ($attempt = 0; $attempt -lt 80 -and $null -eq $numberBox; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $numberBoxes = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($numberBoxes.Count -gt 1) {
                throw "expected one NumberBox Spinner element, found $($numberBoxes.Count)"
            }
            if ($numberBoxes.Count -eq 1) {
                $numberBox = $numberBoxes.Item(0)
            }
        }
        if ($null -eq $numberBox) {
            Start-Sleep -Milliseconds 50
        }
    }
    if ($null -eq $numberBox) {
        $available = @()
        if ($null -ne $root) {
            $all = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition
            )
            $available = @($all | ForEach-Object {
                "$($_.Current.ControlType.ProgrammaticName):$($_.Current.AutomationId):$($_.Current.Name)"
            })
        }
        throw "UI Automation did not expose the NumberBox Spinner before timeout; available elements: $($available -join ', ')"
    }
    if ($numberBox.Current.FrameworkId -ne "ZSUI" -or
        -not $numberBox.Current.IsKeyboardFocusable) {
        throw "NumberBox did not expose the ZSUI SpinButton focus contract"
    }

    $rangeObject = $null
    if (-not $numberBox.TryGetCurrentPattern(
        [System.Windows.Automation.RangeValuePattern]::Pattern,
        [ref]$rangeObject
    )) {
        throw "NumberBox did not expose RangeValuePattern"
    }
    $range = [System.Windows.Automation.RangeValuePattern]$rangeObject
    if ($range.Current.Minimum -ne -100.0 -or
        $range.Current.Maximum -ne 100.0 -or
        $range.Current.SmallChange -ne 0.5 -or
        $range.Current.LargeChange -ne 10.0 -or
        $range.Current.IsReadOnly) {
        throw "NumberBox exposed an invalid adjustable range contract"
    }
    $valueBefore = $range.Current.Value
    $range.SetValue(-7.5)

    $valueAfter = $null
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        $currentRangeObject = $null
        if ($numberBox.TryGetCurrentPattern(
            [System.Windows.Automation.RangeValuePattern]::Pattern,
            [ref]$currentRangeObject
        )) {
            $currentRange = [System.Windows.Automation.RangeValuePattern]$currentRangeObject
            if ($currentRange.Current.Value -eq -7.5) {
                $valueAfter = $currentRange.Current.Value
                break
            }
        }
        Start-Sleep -Milliseconds 50
    }
    if ($valueAfter -ne -7.5 -or $valueAfter -eq $valueBefore) {
        throw "UIA SetValue did not rebuild the retained NumberBox with value -7.5"
    }
    $valueObject = $null
    if (-not $numberBox.TryGetCurrentPattern(
        [System.Windows.Automation.ValuePattern]::Pattern,
        [ref]$valueObject
    )) {
        throw "editable NumberBox did not expose ValuePattern alongside RangeValuePattern"
    }
    $textValue = ([System.Windows.Automation.ValuePattern]$valueObject).Current.Value
    if ($textValue -ne "-7.5") {
        throw "NumberBox ValuePattern did not expose the same committed text as RangeValuePattern"
    }

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        control_type = "Spinner"
        framework_id = $numberBox.Current.FrameworkId
        automation_id = $numberBox.Current.AutomationId
        value_before = $valueBefore
        requested_value = -7.5
        value_after = $valueAfter
        minimum = $range.Current.Minimum
        maximum = $range.Current.Maximum
        small_change = $range.Current.SmallChange
        large_change = $range.Current.LargeChange
        read_only = $range.Current.IsReadOnly
        editable_text = $textValue
        value_pattern = $true
        errors = @()
    }

    if (-not $process.WaitForExit(10000)) {
        throw "NumberBox accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "NumberBox accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "NumberBox native proof did not produce the buffered Win32 screenshot"
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    Write-Output "Windows NumberBox accessibility passed: UIA Spinner RangeValue -> retained value -7.5"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
