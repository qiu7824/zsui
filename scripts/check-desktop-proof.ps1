param(
    [switch]$RequireComplete
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$proofRoot = Join-Path $repoRoot "docs/platform-proof"
$manifestPath = Join-Path $proofRoot "manifest.json"

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw "desktop proof manifest is missing: $manifestPath"
}

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema_version -ne 2) {
    throw "unsupported desktop proof schema version: $($manifest.schema_version)"
}
if ($manifest.release -ne "0.2.0") {
    throw "desktop proof release identity is not v0.2.0"
}

$expectedPlatforms = @("windows", "macos", "linux")
$platformIds = @($manifest.platforms | ForEach-Object { $_.id })
if (($platformIds | Select-Object -Unique).Count -ne $platformIds.Count) {
    throw "desktop proof manifest contains duplicate platform ids"
}
foreach ($platform in $expectedPlatforms) {
    if ($platform -notin $platformIds) {
        throw "desktop proof manifest is missing platform: $platform"
    }
}

function Test-PngFile {
    param([string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 1024) {
        throw "desktop proof PNG is too small to be evidence: $Path"
    }
    $signature = @(137, 80, 78, 71, 13, 10, 26, 10)
    for ($index = 0; $index -lt $signature.Count; $index++) {
        if ($bytes[$index] -ne $signature[$index]) {
            throw "desktop proof file is not a PNG: $Path"
        }
    }
}

function Test-PositiveNumber {
    param(
        [object]$Value,
        [string]$Name
    )

    if ($null -eq $Value -or [double]$Value -le 0) {
        throw "desktop interaction check did not pass: $Name"
    }
}

foreach ($platform in $manifest.platforms) {
    $platformRoot = [IO.Path]::GetFullPath((Join-Path $proofRoot $platform.id))
    $readme = Join-Path $platformRoot "README.md"
    if (-not (Test-Path -LiteralPath $readme -PathType Leaf)) {
        throw "desktop proof README is missing for $($platform.id)"
    }

    if ($RequireComplete -and $platform.status -ne "complete") {
        throw "desktop backend is not marked complete: $($platform.id)"
    }
    if ($platform.status -ne "complete") {
        Write-Host "desktop proof incomplete: $($platform.id) ($($platform.status))"
        continue
    }

    foreach ($artifact in $platform.artifacts) {
        $artifactPath = [IO.Path]::GetFullPath((Join-Path $platformRoot $artifact))
        if (-not $artifactPath.StartsWith($platformRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
            throw "desktop proof artifact escapes its platform directory: $artifact"
        }
        if (-not (Test-Path -LiteralPath $artifactPath -PathType Leaf)) {
            throw "desktop proof artifact is missing: $artifactPath"
        }
        if ($artifactPath.EndsWith(".png", [StringComparison]::OrdinalIgnoreCase)) {
            Test-PngFile -Path $artifactPath
        } elseif ($artifactPath.EndsWith(".json", [StringComparison]::OrdinalIgnoreCase)) {
            $null = Get-Content -LiteralPath $artifactPath -Raw | ConvertFrom-Json
        }
    }

    $interactionPath = [IO.Path]::GetFullPath((Join-Path $platformRoot $platform.interaction_report))
    $interaction = Get-Content -LiteralPath $interactionPath -Raw | ConvertFrom-Json
    if ($interaction.platform -ne $platform.id -or $interaction.backend -ne $platform.report_backend) {
        throw "desktop interaction report identity mismatch: $interactionPath"
    }
    if (@($interaction.errors).Count -ne 0 -or $null -ne $interaction.runtime.startup_error) {
        throw "desktop interaction report contains runtime errors: $interactionPath"
    }
    foreach ($check in $manifest.required_interactions) {
        switch ($check) {
            "window_lifecycle" {
                Test-PositiveNumber $interaction.runtime.created_window_count "$($platform.id)/window_created"
                Test-PositiveNumber $interaction.runtime.native_view_window_close_request_count "$($platform.id)/window_close"
            }
            "final_surface_capture" {
                if ($interaction.runtime.screenshot_captured -ne $true) {
                    throw "desktop interaction check did not pass: $($platform.id)/final_surface_capture"
                }
            }
            "typed_messages" { Test-PositiveNumber $interaction.runtime.native_view_message_count "$($platform.id)/typed_messages" }
            "text_input" { Test-PositiveNumber $interaction.runtime.native_view_text_input_count "$($platform.id)/text_input" }
            "selection" { Test-PositiveNumber $interaction.runtime.native_view_text_selection_change_count "$($platform.id)/selection" }
            "scroll" { Test-PositiveNumber $interaction.runtime.native_view_scroll_count "$($platform.id)/scroll" }
            "keyboard" { Test-PositiveNumber $interaction.runtime.native_view_key_down_count "$($platform.id)/keyboard" }
            "native_menu" {
                if ($interaction.runtime.window_menu_command_routed -ne $true) {
                    throw "desktop interaction check did not pass: $($platform.id)/native_menu"
                }
            }
            default { throw "unsupported desktop interaction check: $check" }
        }
    }
    foreach ($counter in @(
        "native_view_unhandled_click_count",
        "native_view_unhandled_key_count",
        "native_view_unhandled_scroll_count"
    )) {
        if ([int64]$interaction.runtime.$counter -ne 0) {
            throw "desktop interaction report has unhandled input: $($platform.id)/$counter"
        }
    }

    $workflowPath = [IO.Path]::GetFullPath((Join-Path $repoRoot $platform.ci_evidence.workflow))
    if (-not (Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
        throw "desktop proof workflow is missing: $workflowPath"
    }
    $workflow = Get-Content -LiteralPath $workflowPath -Raw
    foreach ($job in $platform.ci_evidence.jobs) {
        if (-not $workflow.Contains("name: $job")) {
            throw "desktop proof workflow job is missing: $($platform.id)/$job"
        }
    }
    foreach ($step in $platform.ci_evidence.steps) {
        if (-not $workflow.Contains("- name: $step")) {
            throw "desktop proof workflow step is missing: $($platform.id)/$step"
        }
    }
    if ($RequireComplete) {
        if ($platform.ci_evidence.conclusion -ne "success") {
            throw "desktop proof CI has not completed successfully: $($platform.id)"
        }
        if ($platform.ci_evidence.run_url -notmatch '^https://github\.com/qiu7824/zsui/actions/runs/[0-9]+$') {
            throw "desktop proof CI run URL is invalid: $($platform.id)"
        }
    }
}

Write-Host "desktop proof contract passed"
