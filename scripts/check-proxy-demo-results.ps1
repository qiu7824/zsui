param([Parameter(Mandatory)][string]$Summary)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$report=Get-Content -LiteralPath $Summary -Raw | ConvertFrom-Json
$failures=[Collections.Generic.List[string]]::new()
$checks=[Collections.Generic.List[object]]::new()
foreach($name in @('input','scroll','restore')) {
    $stat=$report.statistics."latency-$name"
    if($stat.pairs -lt 10){throw "Insufficient independent latency pairs: $name"}
    $limit=[Math]::Max($stat.baseline_median*0.05,1.0)
    $failed=$stat.median_difference -gt $limit -and $stat.ci_low -gt 0
    $checks.Add(@{metric="latency-$name";difference=$stat.median_difference;limit=$limit;ci_low=$stat.ci_low;passed=(-not $failed)})
    if($failed){$failures.Add("$name p95 increased by $($stat.median_difference) ms (limit $limit ms)")}
}
foreach($size in @('small','large')) {
    foreach($metric in @("private_bytes-visible-$size","private_bytes-hidden-$size","private_bytes-restored-$size","peak-$size")) {
        $stat=$report.statistics.$metric
        if($stat.pairs -lt 10){throw "Insufficient independent memory pairs: $metric"}
        $limit=[Math]::Max($stat.baseline_median*0.03,256KB)
        $failed=$stat.median_difference -gt $limit -and $stat.ci_low -gt 0
        $checks.Add(@{metric=$metric;difference=$stat.median_difference;limit=$limit;ci_low=$stat.ci_low;passed=(-not $failed)})
        if($failed){$failures.Add("$metric increased beyond $limit bytes")}
    }
}
foreach($scene in $report.pixels.PSObject.Properties) {
    if(-not $scene.Value.same_size -or $scene.Value.differing_pixels -ne 0){$failures.Add("Pixel mismatch: $($scene.Name)")}
}
foreach($mode in @('screen','print')) {
    $before=$report.probes."tear-baseline-$mode"
    $after=$report.probes."tear-candidate-$mode"
    if($before.color_changes_seen -eq 0 -or $after.color_changes_seen -eq 0){$failures.Add("Tearing probe did not observe changing frames: $mode")}
    if($after.torn_frames -gt $before.torn_frames){$failures.Add("More torn frames: $mode")}
}
$acceptance=[ordered]@{schema='zsui.paint-acceptance/v1';passed=($failures.Count -eq 0);checks=$checks.ToArray();failures=$failures.ToArray()}
$output=Join-Path (Split-Path -Parent ([IO.Path]::GetFullPath($Summary))) 'acceptance.json'
$acceptance | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $output -Encoding utf8
if($failures.Count -gt 0){throw "Paint acceptance failed: $($failures -join '; ')"}
Write-Output 'Paint memory, latency and native pixel acceptance passed'
