param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "native_smoke_run",
    "--no-default-features",
    "--features", "windows-win32,progress,label,native-smoke,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the ProgressBar accessibility probe application"
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
    throw "ProgressBar accessibility probe executable was not produced: $probe"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ZsuiWindowsProgressAccessibilityProbeNative
{
    private delegate bool EnumWindowsCallback(IntPtr hwnd, IntPtr lparam);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr lparam);

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(IntPtr hwnd, StringBuilder className, int capacity);

    public static IntPtr FindMainWindow(uint expectedProcessId)
    {
        IntPtr result = IntPtr.Zero;
        EnumWindows((hwnd, _) =>
        {
            uint processId;
            GetWindowThreadProcessId(hwnd, out processId);
            if (processId != expectedProcessId)
            {
                return true;
            }

            StringBuilder className = new StringBuilder(256);
            GetClassName(hwnd, className, className.Capacity);
            if (className.ToString() != "ZsuiMainWindow")
            {
                return true;
            }

            result = hwnd;
            return false;
        }, IntPtr.Zero);
        return result;
    }
}
'@

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "5000"
$proofRoot = Join-Path $targetRoot "windows-progress-accessibility-proof"
$process = Start-Process -FilePath $probe -ArgumentList @(
    "windows",
    $proofRoot,
    "--progress-view"
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 50
        if ($process.HasExited) {
            throw "ProgressBar accessibility probe exited before creating its main window"
        }
        $hwnd = [ZsuiWindowsProgressAccessibilityProbeNative]::FindMainWindow([uint32]$process.Id)
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "ProgressBar accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::ProgressBar
    )
    $elements = $null
    for ($attempt = 0; $attempt -lt 80; $attempt++) {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $candidate = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                $condition
            )
            if ($candidate.Count -eq 4) {
                $elements = $candidate
                break
            }
            if ($candidate.Count -gt 4) {
                throw "expected four ProgressBar elements, found $($candidate.Count)"
            }
        }
        Start-Sleep -Milliseconds 50
    }
    if ($null -eq $elements) {
        throw "UI Automation did not expose four ProgressBar elements before timeout"
    }

    $expectedValues = @(65.0, 45.0, 30.0)
    $results = @()
    for ($index = 0; $index -lt $elements.Count; $index++) {
        $element = $elements.Item($index)
        if ($element.Current.FrameworkId -ne "ZSUI") {
            throw "ProgressBar $index framework id was '$($element.Current.FrameworkId)'"
        }
        $rangeObject = $null
        $hasRange = $element.TryGetCurrentPattern(
            [System.Windows.Automation.RangeValuePattern]::Pattern,
            [ref]$rangeObject
        )
        if ($index -lt $expectedValues.Count) {
            if (-not $hasRange) {
                throw "determinate ProgressBar $index did not expose RangeValuePattern"
            }
            $range = [System.Windows.Automation.RangeValuePattern]$rangeObject
            if ($range.Current.Value -ne $expectedValues[$index] -or
                $range.Current.Minimum -ne 0.0 -or
                $range.Current.Maximum -ne 100.0 -or
                -not $range.Current.IsReadOnly) {
                throw "determinate ProgressBar $index exposed an invalid read-only range"
            }
            $results += [ordered]@{
                automation_id = $element.Current.AutomationId
                value = $range.Current.Value
                minimum = $range.Current.Minimum
                maximum = $range.Current.Maximum
                read_only = $range.Current.IsReadOnly
            }
        } elseif ($hasRange) {
            throw "indeterminate ProgressBar unexpectedly exposed RangeValuePattern"
        }
    }

    $proof = [ordered]@{
        platform = "windows"
        backend = "UI Automation"
        control_type = "ProgressBar"
        determinate = $results
        indeterminate_has_range = $false
        errors = @()
    }
    New-Item -ItemType Directory -Force -Path $proofRoot | Out-Null
    $proof | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $proofRoot "proof.json") -Encoding utf8
    if (-not $process.WaitForExit(10000)) {
        throw "ProgressBar accessibility probe did not finish after its native proof window closed"
    }
    if ($process.ExitCode -ne 0) {
        throw "ProgressBar accessibility probe exited with code $($process.ExitCode)"
    }
    $screenshot = Join-Path $proofRoot "windows\window.png"
    if (-not (Test-Path -LiteralPath $screenshot -PathType Leaf)) {
        throw "ProgressBar native proof did not produce the buffered Win32 screenshot"
    }
    Write-Output "Windows ProgressBar accessibility passed: four UIA ProgressBar nodes -> three read-only ranges plus one indeterminate role"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
