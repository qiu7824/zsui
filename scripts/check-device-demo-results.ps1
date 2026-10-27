param([Parameter(Mandatory)][string]$Report)
$ErrorActionPreference='Stop'
$data=Get-Content -LiteralPath $Report -Raw | ConvertFrom-Json
if($data.rounds.Count -lt 5){throw 'Missing independent measurement rounds'}
foreach($framework in @('zsui','egui','iced','slint','tauri','gpui')) {
  foreach($round in $data.rounds) {
    $entry=$round.$framework.device
    if(-not $entry.screenshot_captured -or -not (Test-Path -LiteralPath $entry.screenshot)){throw "Missing $framework surface"}
    if($entry.full_page_memory.private_bytes -le 0 -or $entry.full_page_memory.rss_bytes -le 0 -or $entry.warm_start.first_presented_frame_median_ms -le 0){throw "Invalid $framework measurement"}
    if([Math]::Abs($entry.client.width_pixels*96/$entry.client.dpi-960) -gt 1 -or [Math]::Abs($entry.client.height_pixels*96/$entry.client.dpi-680) -gt 1){throw "Mismatched $framework client geometry"}
    if($entry.full_page_memory.samples.Count -lt 10 -or $entry.hidden_memory.samples.Count -lt 10){throw "Insufficient $framework samples"}
  }
}
Write-Output 'Six-framework desktop measurements, client geometry and native captures validated'
