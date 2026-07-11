param(
    [string]$Pack,
    [switch]$List,
    [switch]$IncludeOptional,
    [switch]$Validate,
    [ValidateSet("Text", "Paths", "Json")]
    [string]$Format = "Text"
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $workspace "docs\ai\context-packs.json"

if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "AI context manifest not found: $manifestPath"
}

$manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding utf8 | ConvertFrom-Json
if ($manifest.schema_version -ne 1) {
    throw "Unsupported AI context schema version: $($manifest.schema_version)"
}

function Assert-ContextManifest {
    $ids = @($manifest.packs | ForEach-Object { $_.id })
    $duplicates = @($ids | Group-Object | Where-Object Count -gt 1)
    if ($duplicates.Count -gt 0) {
        throw "Duplicate AI context pack ids: $($duplicates.Name -join ', ')"
    }

    $paths = @($manifest.bootstrap)
    foreach ($entry in $manifest.packs) {
        if (-not $entry.id -or -not $entry.purpose) {
            throw "Every AI context pack requires id and purpose"
        }
        if (@($entry.required).Count -eq 0) {
            throw "AI context pack '$($entry.id)' has no required files"
        }
        $paths += @($entry.required)
        $paths += @($entry.optional)
    }

    $missing = @(
        $paths |
            Sort-Object -Unique |
            Where-Object { -not (Test-Path -LiteralPath (Join-Path $workspace $_)) }
    )
    if ($missing.Count -gt 0) {
        throw "AI context manifest references missing paths: $($missing -join ', ')"
    }
}

Assert-ContextManifest

if ($Validate) {
    [pscustomobject]@{
        schema_version = $manifest.schema_version
        bootstrap = $manifest.bootstrap
        pack_count = @($manifest.packs).Count
        status = "valid"
    } | Format-List
    exit 0
}

if ($List) {
    if ($Format -eq "Json") {
        @($manifest.packs | Select-Object id, purpose) | ConvertTo-Json -Depth 4
    } else {
        $manifest.packs | Select-Object id, purpose | Format-Table -AutoSize
    }
    exit 0
}

if (-not $Pack) {
    throw "Specify -List, -Validate, or -Pack <id>"
}

$selected = $manifest.packs |
    Where-Object { $_.id -eq $Pack } |
    Select-Object -First 1
if (-not $selected) {
    $known = @($manifest.packs | ForEach-Object { $_.id }) -join ", "
    throw "Unknown AI context pack '$Pack'. Available packs: $known"
}

$selectedPaths = @($selected.required)
if ($IncludeOptional) {
    $selectedPaths += @($selected.optional)
}

switch ($Format) {
    "Paths" {
        $selectedPaths | ForEach-Object { $_ }
    }
    "Json" {
        [ordered]@{
            id = $selected.id
            purpose = $selected.purpose
            required = @($selected.required)
            optional = @($selected.optional)
            include_optional = [bool]$IncludeOptional
            selected_paths = $selectedPaths
            verify = @($selected.verify)
        } | ConvertTo-Json -Depth 6
    }
    default {
        Write-Output "pack: $($selected.id)"
        Write-Output "purpose: $($selected.purpose)"
        Write-Output ""
        Write-Output "required:"
        $selected.required | ForEach-Object { Write-Output "  $_" }
        Write-Output ""
        Write-Output "optional (load only when needed):"
        $selected.optional | ForEach-Object { Write-Output "  $_" }
        Write-Output ""
        Write-Output "verify:"
        $selected.verify | ForEach-Object { Write-Output "  $_" }
    }
}
