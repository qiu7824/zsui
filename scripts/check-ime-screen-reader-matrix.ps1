$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$matrixPath = Join-Path $workspace "tests/quality/ime-screen-reader-matrix.json"

$matrix = Get-Content -LiteralPath $matrixPath -Raw | ConvertFrom-Json
if ($matrix.schema -ne "zsui.ime-screen-reader-matrix/v1") {
    throw "unexpected IME/screen-reader matrix schema"
}
if ($matrix.release -ne "0.2.0") {
    throw "IME/screen-reader matrix does not target v0.2.0"
}

$platforms = @($matrix.platforms)
if (($platforms.platform | Sort-Object) -join "," -ne "linux,macos,windows") {
    throw "IME/screen-reader matrix must contain Windows, macOS and Linux exactly once"
}

$requiredEvents = @("preedit", "commit", "cancel", "caret_anchor")
foreach ($entry in $platforms) {
    foreach ($event in $requiredEvents) {
        if (@($entry.ime.required_events) -notcontains $event) {
            throw "$($entry.platform) IME matrix is missing $event"
        }
    }

    $workflowPath = Join-Path $workspace $entry.ime.automated_workflow
    if (-not (Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
        throw "$($entry.platform) IME workflow is missing: $workflowPath"
    }
    $workflowText = Get-Content -LiteralPath $workflowPath -Raw
    if ($workflowText -notmatch [regex]::Escape($entry.ime.automated_step)) {
        throw "$($entry.platform) IME workflow does not contain step '$($entry.ime.automated_step)'"
    }

    $probePath = Join-Path $workspace $entry.accessibility.automated_probe
    if (-not (Test-Path -LiteralPath $probePath -PathType Leaf)) {
        throw "$($entry.platform) accessibility probe is missing: $probePath"
    }
    if ([string]::IsNullOrWhiteSpace($entry.accessibility.automated_client)) {
        throw "$($entry.platform) accessibility matrix has no independent client"
    }
    if ([string]::IsNullOrWhiteSpace($entry.accessibility.manual_reader)) {
        throw "$($entry.platform) accessibility matrix has no real screen-reader gate"
    }
}

if ($matrix.release_manual_gate.required -ne $true) {
    throw "real IME candidates and screen readers must remain a release gate"
}
$manualPath = Join-Path $workspace $matrix.release_manual_gate.evidence_template
if (-not (Test-Path -LiteralPath $manualPath -PathType Leaf)) {
    throw "manual assistive-technology checklist is missing: $manualPath"
}

Write-Output "IME and screen-reader matrix passed: Windows UIA, AppKit Accessibility and Linux AT-SPI"
