param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$InputDirectory,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [string]$Channel = "stable",
    [string]$Repository = "qiu7824/zsui"
)

$ErrorActionPreference = "Stop"

if ($Version -notmatch '^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$') {
    throw "Version must be a semantic version without a leading v"
}
if (-not (Test-Path -LiteralPath $InputDirectory -PathType Container)) {
    throw "input directory does not exist: $InputDirectory"
}

$targets = foreach ($file in Get-ChildItem -LiteralPath $InputDirectory -File | Sort-Object Name) {
    $platform = $null
    $architecture = $null
    $kind = $null
    if ($file.Name -match '-windows-(x86_64|aarch64)(-setup)?\.exe$') {
        $platform = "windows"
        $architecture = $Matches[1]
        $kind = if ($Matches[2]) { "installer" } else { "portable" }
    } elseif ($file.Name -match '-macos-(x86_64|arm64)\.dmg$') {
        $platform = "macos"
        $architecture = $Matches[1]
        $kind = "disk_image"
    } elseif ($file.Name -match '-linux-(x86_64|aarch64)\.deb$') {
        $platform = "linux"
        $architecture = $Matches[1]
        $kind = "deb"
    } elseif ($file.Name -match '-linux-(x86_64|aarch64)\.tar\.gz$') {
        $platform = "linux"
        $architecture = $Matches[1]
        $kind = "portable"
    } else {
        continue
    }

    $digest = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    [ordered]@{
        platform = $platform
        architecture = $architecture
        kind = $kind
        file_name = $file.Name
        size = $file.Length
        sha256 = $digest
        url = "https://github.com/$Repository/releases/download/v$Version/$($file.Name)"
        signature = [ordered]@{
            provenance = "github-artifact-attestation"
            repository = $Repository
            os_code_signature = if ($platform -eq "windows") {
                "authenticode-when-certificate-secret-is-configured"
            } elseif ($platform -eq "macos") {
                "developer-id-and-notarization-when-apple-secrets-are-configured"
            } else {
                "not-applicable"
            }
        }
    }
}

if (@($targets).Count -lt 3) {
    throw "update manifest requires installable targets for all desktop platforms"
}
foreach ($platform in @("windows", "macos", "linux")) {
    if (@($targets | Where-Object platform -eq $platform).Count -eq 0) {
        throw "update manifest is missing $platform targets"
    }
}

$manifest = [ordered]@{
    schema = "zsui.update-manifest/v1"
    channel = $Channel
    version = $Version
    published_at = [DateTimeOffset]::UtcNow.ToString("O")
    repository = $Repository
    minimum_updater_schema = 1
    targets = @($targets)
}

$parent = Split-Path -Parent $OutputPath
if ($parent) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding utf8NoBOM
Write-Output "Update manifest written: $OutputPath ($(@($targets).Count) targets)"
