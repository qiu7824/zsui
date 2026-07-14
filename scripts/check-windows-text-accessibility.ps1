param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "zsui_notepad",
    "--no-default-features",
    "--features", "notepad-demo,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the native accessibility probe application"
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
$notepad = Join-Path $targetRoot "debug\examples\zsui_notepad.exe"
if (-not (Test-Path -LiteralPath $notepad -PathType Leaf)) {
    throw "native accessibility probe executable was not produced: $notepad"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ZsuiWindowsTextAccessibilityProbeNative
{
    private delegate bool EnumWindowsCallback(IntPtr hwnd, IntPtr lparam);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr lparam);

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(IntPtr hwnd, StringBuilder className, int capacity);

    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr hwnd, uint message, IntPtr wparam, IntPtr lparam);

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

$process = Start-Process -FilePath $notepad -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 50 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 100
        if ($process.HasExited) {
            throw "native accessibility probe exited before creating its main window"
        }
        $hwnd = [ZsuiWindowsTextAccessibilityProbeNative]::FindMainWindow([uint32]$process.Id)
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "native accessibility probe did not create a ZsuiMainWindow HWND"
    }

    # Focus the editor through the real Win32 message route used by the notepad smoke.
    $editorPoint = [IntPtr](360 -bor (220 -shl 16))
    [void][ZsuiWindowsTextAccessibilityProbeNative]::SendMessage(
        $hwnd,
        0x0201,
        [IntPtr]1,
        $editorPoint
    )
    [void][ZsuiWindowsTextAccessibilityProbeNative]::SendMessage(
        $hwnd,
        0x0202,
        [IntPtr]0,
        $editorPoint
    )

    $element = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
    if ($null -eq $element) {
        throw "UI Automation returned no provider for the native HWND"
    }
    if ($element.Current.ControlType -ne [System.Windows.Automation.ControlType]::Edit) {
        throw "UI Automation did not expose the focused self-drawn editor as an Edit control"
    }
    if ($element.Current.FrameworkId -ne "ZSUI") {
        throw "UI Automation framework id was '$($element.Current.FrameworkId)', expected 'ZSUI'"
    }
    if ($element.Current.ClassName -ne "ZsuiTextInput") {
        throw "UI Automation class name was '$($element.Current.ClassName)', expected 'ZsuiTextInput'"
    }
    if ($element.Current.AutomationId -ne "zsui-widget-1") {
        throw "UI Automation id was '$($element.Current.AutomationId)', expected 'zsui-widget-1'"
    }

    $valuePattern = $element.GetCurrentPattern(
        [System.Windows.Automation.ValuePattern]::Pattern
    )
    if ($null -eq $valuePattern) {
        throw "UI Automation ValuePattern was not available"
    }
    if ($valuePattern.Current.IsReadOnly) {
        throw "UI Automation unexpectedly reported the editor as read-only"
    }
    if (-not $valuePattern.Current.Value.Contains("ZSUI Notepad")) {
        throw "UI Automation ValuePattern did not expose the application-owned editor text"
    }

    Write-Output "Windows native text accessibility passed: HWND WM_GETOBJECT -> ZSUI Edit/ValuePattern"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
}
