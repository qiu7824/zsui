param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,dialog,label,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the ContentDialog accessibility probe application"
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
    throw "ContentDialog accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "8000"
$proofRoot = Join-Path $targetRoot "windows-content-dialog-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--content-dialog"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "ContentDialog accessibility probe exited before creating its main window"
        }
        $process.Refresh()
        $hwnd = $process.MainWindowHandle
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "ContentDialog accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::IsDialogProperty,
        $true
    )
    $dialog = $null
    for ($attempt = 0; $attempt -lt 80 -and $null -eq $dialog; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $dialogs = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($dialogs.Count -gt 1) {
                throw "expected one semantic ContentDialog, found $($dialogs.Count)"
            }
            if ($dialogs.Count -eq 1) {
                $dialog = $dialogs.Item(0)
                break
            }
        }
        Start-Sleep -Milliseconds 50
    }
    if ($null -eq $dialog) {
        throw "UI Automation did not expose the semantic ContentDialog before timeout"
    }

    $bounds = $dialog.Current.BoundingRectangle
    if ($dialog.Current.FrameworkId -ne "ZSUI" -or
        $dialog.Current.Name -ne "Save changes?" -or
        $dialog.Current.HelpText -ne "The framework owns the modal focus scope while the application owns whether the dialog is open." -or
        $bounds.Width -le 0 -or
        $bounds.Height -le 0) {
        throw "ContentDialog UIA role, name, description or visible-surface bounds were invalid"
    }

    $buttonCondition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Button
    )
    $buttons = $dialog.FindAll(
        [System.Windows.Automation.TreeScope]::Children,
        $buttonCondition
    )
    if ($buttons.Count -ne 3) {
        throw "expected three direct semantic ContentDialog buttons, found $($buttons.Count)"
    }
    $buttonByName = @{}
    foreach ($button in $buttons) {
        $buttonByName[$button.Current.Name] = $button
        $invokePattern = $null
        if ($button.Current.FrameworkId -ne "ZSUI" -or
            -not $button.Current.IsKeyboardFocusable -or
            -not $button.TryGetCurrentPattern(
                [System.Windows.Automation.InvokePattern]::Pattern,
                [ref]$invokePattern
            )) {
            throw "ContentDialog semantic button '$($button.Current.Name)' is missing ZSUI focus or InvokePattern"
        }
    }
    foreach ($name in @("Save", "Discard", "Cancel")) {
        if (-not $buttonByName.ContainsKey($name)) {
            throw "ContentDialog semantic button '$name' was not exposed"
        }
    }

    $discard = $buttonByName["Discard"]
    $discard.SetFocus()
    $focusedName = $null
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        $focused = [System.Windows.Automation.AutomationElement]::FocusedElement
        if ($null -ne $focused -and $focused.Current.Name -eq "Discard") {
            $focusedName = $focused.Current.Name
            break
        }
        Start-Sleep -Milliseconds 50
    }
    if ($focusedName -ne "Discard") {
        throw "ContentDialog semantic button focus did not round-trip through the native UIA fragment root"
    }

    $save = $buttonByName["Save"]
    $invoke = $save.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $invoke.Invoke()
    Start-Sleep -Milliseconds 100

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        role = "dialog"
        is_dialog = $dialog.GetCurrentPropertyValue(
            [System.Windows.Automation.AutomationElement]::IsDialogProperty
        )
        name = $dialog.Current.Name
        description = $dialog.Current.HelpText
        framework_id = $dialog.Current.FrameworkId
        automation_id = $dialog.Current.AutomationId
        buttons = @($buttons | ForEach-Object {
            [ordered]@{
                name = $_.Current.Name
                automation_id = $_.Current.AutomationId
                keyboard_focusable = $_.Current.IsKeyboardFocusable
                invoke_pattern = $true
            }
        })
        focused_button = $focusedName
        invoked_button = "Save"
        bounds = [ordered]@{
            x = $bounds.X
            y = $bounds.Y
            width = $bounds.Width
            height = $bounds.Height
        }
        errors = @()
    }

    if (-not $process.WaitForExit(10000)) {
        throw "ContentDialog accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "ContentDialog accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "ContentDialog native proof did not produce the buffered Win32 screenshot"
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    Write-Output "Windows ContentDialog accessibility passed: UIA dialog -> three focusable and invokable native semantic buttons"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
