param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,button,label,tooltip,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the ToolTip accessibility probe application"
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
    throw "ToolTip accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "5000"
$proofRoot = Join-Path $targetRoot "windows-tooltip-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--tooltip-view"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "ToolTip accessibility probe exited before creating its main window"
        }
        $process.Refresh()
        $hwnd = $process.MainWindowHandle
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "ToolTip accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        )),
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::NameProperty,
            "Save document"
        ))
    )
    $owner = $null
    for ($attempt = 0; $attempt -lt 80 -and $null -eq $owner; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $owners = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($owners.Count -gt 1) {
                throw "expected one ToolTip owner semantic element, found $($owners.Count)"
            }
            if ($owners.Count -eq 1) {
                $owner = $owners.Item(0)
            }
        }
        if ($null -eq $owner) {
            Start-Sleep -Milliseconds 50
        }
    }
    if ($null -eq $owner) {
        throw "UI Automation did not expose the ToolTip owner before timeout"
    }
    if ($owner.Current.FrameworkId -ne "ZSUI" -or
        -not $owner.Current.IsKeyboardFocusable) {
        throw "ToolTip owner did not preserve the ZSUI Button focus contract"
    }
    if ($owner.Current.HelpText -ne "Save the current document") {
        throw "ToolTip text was not projected as UIA HelpText on its owner"
    }

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        owner_control_type = "Button"
        framework_id = $owner.Current.FrameworkId
        automation_id = $owner.Current.AutomationId
        name = $owner.Current.Name
        help_text = $owner.Current.HelpText
        duplicate_tooltip_node = $false
        errors = @()
    }

    if (-not $process.WaitForExit(10000)) {
        throw "ToolTip accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "ToolTip accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "ToolTip native proof did not produce the buffered Win32 screenshot"
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    Write-Output "Windows ToolTip accessibility passed: one UIA Button owner with matching HelpText"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
