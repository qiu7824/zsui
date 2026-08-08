param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,toggle-button,label,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the ToggleButton accessibility probe application"
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
    throw "ToggleButton accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "8000"
$proofRoot = Join-Path $targetRoot "windows-toggle-button-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--toggle-button-view"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "ToggleButton accessibility probe exited before creating its main window"
        }
        $process.Refresh()
        $hwnd = $process.MainWindowHandle
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "ToggleButton accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        )),
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::NameProperty,
            "Pin panel"
        ))
    )
    $toggleButton = $null
    for ($attempt = 0; $attempt -lt 80 -and $null -eq $toggleButton; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $buttons = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($buttons.Count -gt 1) {
                throw "expected one ToggleButton element, found $($buttons.Count)"
            }
            if ($buttons.Count -eq 1) {
                $toggleButton = $buttons.Item(0)
            }
        }
        if ($null -eq $toggleButton) {
            Start-Sleep -Milliseconds 50
        }
    }
    if ($null -eq $toggleButton) {
        throw "UI Automation did not expose the ToggleButton before timeout"
    }
    if ($toggleButton.Current.FrameworkId -ne "ZSUI" -or
        -not $toggleButton.Current.IsKeyboardFocusable) {
        throw "ToggleButton did not expose the ZSUI Button focus contract"
    }
    if ($toggleButton.Current.ClassName -ne "ToggleButton") {
        throw "ToggleButton did not expose the native ToggleButton automation class name"
    }

    $invokeObject = $null
    if ($toggleButton.TryGetCurrentPattern(
        [System.Windows.Automation.InvokePattern]::Pattern,
        [ref]$invokeObject
    )) {
        throw "ToggleButton incorrectly exposed InvokePattern instead of its stateful TogglePattern"
    }
    $toggleObject = $null
    if (-not $toggleButton.TryGetCurrentPattern(
        [System.Windows.Automation.TogglePattern]::Pattern,
        [ref]$toggleObject
    )) {
        throw "ToggleButton did not expose TogglePattern"
    }
    $toggle = [System.Windows.Automation.TogglePattern]$toggleObject
    $stateBefore = $toggle.Current.ToggleState.ToString()
    if ($stateBefore -ne "On") {
        throw "scripted native interaction did not leave ToggleButton checked before UIA verification"
    }
    $toggle.Toggle()

    $stateAfter = $null
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        $currentButton = $root.FindFirst(
            [System.Windows.Automation.TreeScope]::Descendants,
            $condition
        )
        $currentToggleObject = $null
        if ($null -ne $currentButton -and $currentButton.TryGetCurrentPattern(
            [System.Windows.Automation.TogglePattern]::Pattern,
            [ref]$currentToggleObject
        )) {
            $currentToggle = [System.Windows.Automation.TogglePattern]$currentToggleObject
            if ($currentToggle.Current.ToggleState.ToString() -eq "Off") {
                $stateAfter = "Off"
                break
            }
        }
        Start-Sleep -Milliseconds 50
    }
    if ($stateAfter -ne "Off") {
        throw "UIA Toggle did not rebuild the retained ToggleButton in the unchecked state"
    }

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        control_type = "Button"
        class_name = $toggleButton.Current.ClassName
        framework_id = $toggleButton.Current.FrameworkId
        automation_id = $toggleButton.Current.AutomationId
        name = $toggleButton.Current.Name
        pattern = "Toggle"
        invoke_pattern = $false
        state_before = $stateBefore
        state_after = $stateAfter
        errors = @()
    }

    if (-not $process.WaitForExit(10000)) {
        throw "ToggleButton accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "ToggleButton accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "ToggleButton native proof did not produce the buffered Win32 screenshot"
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    Write-Output "Windows ToggleButton accessibility passed: UIA TogglePattern On -> Off"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
