param(
    [Parameter(Mandatory = $true)]
    [string]$ManifestPath,
    [string]$ExpectedVersion = "0.2.0"
)

$ErrorActionPreference = "Stop"
$manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
if ($manifest.schema -ne "zsui.update-manifest/v1") {
    throw "unexpected update manifest schema"
}
if ($manifest.version -ne $ExpectedVersion) {
    throw "update manifest version is '$($manifest.version)', expected '$ExpectedVersion'"
}
if ($manifest.minimum_updater_schema -ne 1) {
    throw "unsupported updater schema requirement"
}

$identities = @{}
foreach ($target in @($manifest.targets)) {
    $identity = "$($target.platform)/$($target.architecture)/$($target.kind)"
    if ($identities.ContainsKey($identity)) {
        throw "duplicate update target: $identity"
    }
    $identities[$identity] = $true
    if ($target.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "invalid SHA-256 for $identity"
    }
    if ([int64]$target.size -le 0) {
        throw "invalid file size for $identity"
    }
    if ($target.url -notlike "https://github.com/$($manifest.repository)/releases/download/v$ExpectedVersion/*") {
        throw "target URL is outside the pinned GitHub release: $identity"
    }
    if ($target.signature.provenance -ne "github-artifact-attestation") {
        throw "target does not require GitHub artifact attestation: $identity"
    }
}

foreach ($platform in @("windows", "macos", "linux")) {
    if (@($manifest.targets | Where-Object platform -eq $platform).Count -eq 0) {
        throw "update manifest is missing $platform"
    }
}

Write-Output "Update manifest passed: $(@($manifest.targets).Count) unique signed targets"
