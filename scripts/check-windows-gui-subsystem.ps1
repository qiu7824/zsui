param(
    [string[]]$Executable = @()
)

$ErrorActionPreference = "Stop"

$workspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$examplesRoot = Join-Path $workspace "examples"
$requiredAttribute = '#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]'
$consoleToolMarker = 'ZSUI_WINDOWS_CONSOLE_TOOL:'
$additionalGuiExamples = @(
    "zsui_calculator.rs"
)

$guiExamples = [System.Collections.Generic.List[string]]::new()
$violations = [System.Collections.Generic.List[string]]::new()

foreach ($source in Get-ChildItem -LiteralPath $examplesRoot -Filter "*.rs" -File) {
    $lines = @(Get-Content -LiteralPath $source.FullName)
    $text = $lines -join [Environment]::NewLine
    $isConsoleTool = $text.Contains($consoleToolMarker)
    $launchesNativeWindow = @(
        $lines | Where-Object {
            $_ -notmatch '^\s*//' -and $_ -match '\bnative_window\s*\('
        }
    ).Count -gt 0
    $isAdditionalGuiExample = $additionalGuiExamples -contains $source.Name

    if ($isConsoleTool) {
        continue
    }
    if (-not $launchesNativeWindow -and -not $isAdditionalGuiExample) {
        continue
    }

    $guiExamples.Add($source.Name)
    if (-not $text.Contains($requiredAttribute)) {
        $violations.Add(
            "$($source.FullName): Windows GUI examples must declare $requiredAttribute"
        )
    }
}

function Get-PeSubsystem([string]$Path) {
    $resolved = [IO.Path]::GetFullPath((Join-Path $workspace $Path))
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Windows GUI executable does not exist: $resolved"
    }

    $bytes = [IO.File]::ReadAllBytes($resolved)
    if ($bytes.Length -lt 0x40 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
        throw "Windows GUI executable is not a PE file: $resolved"
    }

    $peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
    $optionalHeaderOffset = $peOffset + 24
    $subsystemOffset = $optionalHeaderOffset + 68
    if ($peOffset -lt 0 -or $subsystemOffset + 2 -gt $bytes.Length) {
        throw "Windows GUI executable has an invalid PE header: $resolved"
    }
    if ($bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45) {
        throw "Windows GUI executable has an invalid PE signature: $resolved"
    }

    [BitConverter]::ToUInt16($bytes, $subsystemOffset)
}

foreach ($path in $Executable) {
    $subsystem = Get-PeSubsystem $path
    if ($subsystem -ne 2) {
        $violations.Add(
            "${path}: PE subsystem is $subsystem; user-facing Windows GUI artifacts require subsystem 2 (Windows GUI)"
        )
    }
}

if ($violations.Count -gt 0) {
    throw "Windows GUI subsystem boundary failed:`n$($violations -join [Environment]::NewLine)"
}

$artifactCount = $Executable.Count
Write-Host "Windows GUI subsystem boundary passed: $($guiExamples.Count) GUI example source(s), $artifactCount PE artifact(s)"
