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
if ($manifest.schema_version -ne 1) {
    throw "unsupported desktop proof schema version: $($manifest.schema_version)"
}

$expectedPlatforms = @("windows", "macos", "linux")
$platformIds = @($manifest.platforms | ForEach-Object { $_.id })
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

foreach ($platform in $manifest.platforms) {
    $platformRoot = Join-Path $proofRoot $platform.id
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

    foreach ($artifact in $manifest.required_artifacts) {
        $artifactPath = Join-Path $platformRoot $artifact
        if (-not (Test-Path -LiteralPath $artifactPath -PathType Leaf)) {
            throw "desktop proof artifact is missing: $artifactPath"
        }
        if ($artifact.EndsWith(".png")) {
            Test-PngFile -Path $artifactPath
        }
    }

    $interactionPath = Join-Path $platformRoot "interaction-report.json"
    $interaction = Get-Content -LiteralPath $interactionPath -Raw | ConvertFrom-Json
    if ($interaction.platform -ne $platform.id -or $interaction.backend -ne $platform.backend) {
        throw "desktop interaction report identity mismatch: $interactionPath"
    }
    foreach ($check in $manifest.required_interactions) {
        $property = $interaction.checks.PSObject.Properties[$check]
        if ($null -eq $property -or $property.Value -ne $true) {
            throw "desktop interaction check did not pass: $($platform.id)/$check"
        }
    }
}

Write-Host "desktop proof contract passed"
