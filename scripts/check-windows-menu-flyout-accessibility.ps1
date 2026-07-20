param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$cargoArguments = @(
    "build",
    "--example", "component_gallery",
    "--no-default-features",
    "--features", "component-gallery-demo,accessibility"
)
if ($Locked) {
    $cargoArguments = @("build", "--locked") + $cargoArguments[1..($cargoArguments.Length - 1)]
}

Push-Location $workspace
try {
    & cargo @cargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "failed to build the MenuFlyout accessibility probe application"
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
$gallery = Join-Path $targetRoot "debug\examples\component_gallery.exe"
if (-not (Test-Path -LiteralPath $gallery -PathType Leaf)) {
    throw "MenuFlyout accessibility probe executable was not produced: $gallery"
}

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ZsuiWindowsMenuAccessibilityProbeNative
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

function Find-AccessibleElement($root, [string]$nameSuffix) {
    $elements = $root.FindAll(
        [System.Windows.Automation.TreeScope]::Subtree,
        [System.Windows.Automation.Condition]::TrueCondition
    )
    foreach ($element in $elements) {
        if ($element.Current.Name.EndsWith($nameSuffix, [System.StringComparison]::Ordinal)) {
            return $element
        }
    }
    return $null
}

$previousDuration = $env:ZSUI_NATIVE_PROOF_DURATION_MS
$env:ZSUI_NATIVE_PROOF_DURATION_MS = "15000"
$proofRoot = Join-Path $targetRoot "windows-menu-accessibility-proof"
$process = Start-Process -FilePath $gallery -ArgumentList @(
    "--native-proof",
    "--page", "feedback",
    "--width", "1024",
    "--height", "640",
    "--output", $proofRoot
) -PassThru -WindowStyle Hidden
try {
    $hwnd = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 80 -and $hwnd -eq [IntPtr]::Zero; $attempt++) {
        Start-Sleep -Milliseconds 100
        if ($process.HasExited) {
            throw "MenuFlyout accessibility probe exited before creating its main window"
        }
        $hwnd = [ZsuiWindowsMenuAccessibilityProbeNative]::FindMainWindow([uint32]$process.Id)
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "MenuFlyout accessibility probe did not create a ZsuiMainWindow HWND"
    }

    $root = $null
    for ($attempt = 0; $attempt -lt 80; $attempt++) {
        $candidate = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $candidate -and
            $candidate.Current.ControlType -eq [System.Windows.Automation.ControlType]::Menu) {
            $root = $candidate
            break
        }
        Start-Sleep -Milliseconds 100
    }
    if ($null -eq $root) {
        throw "UI Automation did not expose the open self-drawn MenuFlyout as a Menu"
    }
    if ($root.Current.FrameworkId -ne "ZSUI") {
        throw "MenuFlyout UI Automation framework id was '$($root.Current.FrameworkId)'"
    }
    if ($root.Current.ClassName -ne "ZsuiMenuFlyout") {
        throw "MenuFlyout UI Automation class name was '$($root.Current.ClassName)'"
    }

    $checked = Find-AccessibleElement $root "Auto save"
    if ($null -eq $checked) {
        throw "UI Automation did not expose the checked MenuFlyout command"
    }
    $toggle = [System.Windows.Automation.TogglePattern]$checked.GetCurrentPattern(
        [System.Windows.Automation.TogglePattern]::Pattern
    )
    if ($toggle.Current.ToggleState -ne [System.Windows.Automation.ToggleState]::On) {
        throw "UI Automation did not expose the checked MenuFlyout ToggleState"
    }

    $more = Find-AccessibleElement $root "More"
    if ($null -eq $more) {
        throw "UI Automation did not expose the MenuFlyout submenu"
    }
    $moreExpansion = [System.Windows.Automation.ExpandCollapsePattern]$more.GetCurrentPattern(
        [System.Windows.Automation.ExpandCollapsePattern]::Pattern
    )
    if ($moreExpansion.Current.ExpandCollapseState -eq [System.Windows.Automation.ExpandCollapseState]::Collapsed) {
        $moreExpansion.Expand()
    } elseif ($moreExpansion.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Expanded) {
        throw "UI Automation exposed an invalid submenu state: $($moreExpansion.Current.ExpandCollapseState)"
    }

    $export = $null
    for ($attempt = 0; $attempt -lt 30 -and $null -eq $export; $attempt++) {
        Start-Sleep -Milliseconds 50
        $export = Find-AccessibleElement $root "Export"
    }
    if ($null -eq $export) {
        throw "UI Automation did not expose the expanded nested MenuFlyout surface"
    }
    $exportExpansion = [System.Windows.Automation.ExpandCollapsePattern]$export.GetCurrentPattern(
        [System.Windows.Automation.ExpandCollapsePattern]::Pattern
    )
    if ($exportExpansion.Current.ExpandCollapseState -eq [System.Windows.Automation.ExpandCollapseState]::Collapsed) {
        $exportExpansion.Expand()
    } elseif ($exportExpansion.Current.ExpandCollapseState -ne [System.Windows.Automation.ExpandCollapseState]::Expanded) {
        throw "UI Automation exposed an invalid nested submenu state: $($exportExpansion.Current.ExpandCollapseState)"
    }

    $pdf = $null
    for ($attempt = 0; $attempt -lt 30 -and $null -eq $pdf; $attempt++) {
        Start-Sleep -Milliseconds 50
        $pdf = Find-AccessibleElement $root "PDF document"
    }
    if ($null -eq $pdf) {
        throw "UI Automation did not expose the third-level MenuFlyout command"
    }
    if ($pdf.Current.ControlType -ne [System.Windows.Automation.ControlType]::MenuItem) {
        throw "UI Automation did not expose the third-level command as a MenuItem"
    }

    Write-Output "Windows MenuFlyout accessibility passed: UIA Fragment tree -> checked and recursive ExpandCollapse providers"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    $env:ZSUI_NATIVE_PROOF_DURATION_MS = $previousDuration
}
