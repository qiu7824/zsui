param(
    [double]$MinimumPercent = 70.0,
    [switch]$Locked
)

$ErrorActionPreference = "Stop"

if ($MinimumPercent -lt 0.0 -or $MinimumPercent -gt 100.0) {
    throw "MinimumPercent must be between 0 and 100"
}

$metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed"
}

$arguments = @("+nightly", "rustdoc")
if ($Locked) {
    $arguments += "--locked"
}
$arguments += @(
    "--lib",
    "--all-features",
    "--",
    "-Z", "unstable-options",
    "--show-coverage"
)

& cargo @arguments
if ($LASTEXITCODE -ne 0) {
    throw "rustdoc coverage generation failed"
}

$coveragePath = Join-Path $metadata.target_directory "doc/zsui.txt"
if (-not (Test-Path -LiteralPath $coveragePath -PathType Leaf)) {
    throw "rustdoc coverage report was not produced: $coveragePath"
}

$total = Get-Content -LiteralPath $coveragePath | Where-Object {
    $_ -match '^\|\s+Total\s+\|\s+\d+\s+\|\s+([0-9.]+)%'
} | Select-Object -Last 1

if (-not $total -or $total -notmatch '^\|\s+Total\s+\|\s+\d+\s+\|\s+([0-9.]+)%') {
    throw "rustdoc coverage total is missing from $coveragePath"
}

$actual = [double]::Parse(
    $Matches[1],
    [System.Globalization.CultureInfo]::InvariantCulture
)
if ($actual -lt $MinimumPercent) {
    throw "Rustdoc coverage is $actual%, below the required $MinimumPercent%"
}

Write-Output "Rustdoc coverage passed: $actual% (required: $MinimumPercent%)"
