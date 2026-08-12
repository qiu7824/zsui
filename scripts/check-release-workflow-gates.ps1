param(
    [Parameter(Mandatory = $true)]
    [string]$Repository,
    [Parameter(Mandatory = $true)]
    [string]$Commit,
    [string]$Token
)

$ErrorActionPreference = "Stop"
$headers = @{
    Accept = "application/vnd.github+json"
    "User-Agent" = "ZSUI release gate"
}
if (-not [string]::IsNullOrWhiteSpace($Token)) {
    $headers.Authorization = "Bearer $Token"
}

$uri = "https://api.github.com/repos/$Repository/actions/runs?head_sha=$Commit&per_page=100"
$response = Invoke-RestMethod -Uri $uri -Headers $headers
$required = @(
    "ci",
    "Stable API and Rustdoc",
    "ARM64 Native Runtime Proof",
    "Native UI Proof",
    "UI Memory Comparison",
    "UI Performance Matrix"
)

foreach ($workflow in $required) {
    $runs = @($response.workflow_runs | Where-Object name -eq $workflow | Sort-Object run_attempt -Descending)
    $successful = @($runs | Where-Object { $_.status -eq "completed" -and $_.conclusion -eq "success" })
    if ($successful.Count -eq 0) {
        $observed = if ($runs.Count -eq 0) {
            "no run"
        } else {
            ($runs | ForEach-Object { "$($_.status)/$($_.conclusion) $($_.html_url)" }) -join "; "
        }
        throw "release gate '$workflow' has no successful run for $Commit ($observed)"
    }
}

Write-Output "Release workflow gates passed for $Commit"
