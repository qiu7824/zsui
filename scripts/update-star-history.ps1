param(
    [string]$Repository = "qiu7824/zsui",
    [string]$OutputPath = "docs/images/star-history.svg"
)

$ErrorActionPreference = "Stop"

if (-not $env:GITHUB_TOKEN) {
    throw "GITHUB_TOKEN is required to read the repository's stargazer timeline"
}

$headers = @{
    Accept = "application/vnd.github.star+json"
    Authorization = "Bearer $env:GITHUB_TOKEN"
    "User-Agent" = "zsui-star-history"
    "X-GitHub-Api-Version" = "2022-11-28"
}

$starredAt = [System.Collections.Generic.List[DateTimeOffset]]::new()
$page = 1
do {
    $response = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/$Repository/stargazers?per_page=100&page=$page" `
        -Headers $headers
    $batch = @($response | ForEach-Object { $_ })
    foreach ($entry in $batch) {
        if ($null -ne $entry.starred_at) {
            $starredAt.Add([DateTimeOffset]$entry.starred_at)
        }
    }
    $page++
} while ($batch.Count -eq 100)

$dates = @($starredAt | Sort-Object)
if ($dates.Count -eq 0) {
    throw "the repository stargazer timeline is empty"
}

$culture = [System.Globalization.CultureInfo]::InvariantCulture
function Format-Number {
    param([double]$Value)
    $Value.ToString("0.##", $culture)
}

$width = 760.0
$height = 360.0
$chartLeft = 66.0
$chartTop = 66.0
$chartRight = 732.0
$chartBottom = 302.0
$chartWidth = $chartRight - $chartLeft
$chartHeight = $chartBottom - $chartTop

$firstDate = $dates[0].ToUniversalTime()
$lastDate = $dates[-1].ToUniversalTime()
$startDate = $firstDate.AddHours(-12)
$endDate = $lastDate.AddHours(1)
if (($endDate - $startDate).TotalHours -lt 24.0) {
    $startDate = $endDate.AddDays(-1)
}
$spanSeconds = [Math]::Max(1.0, ($endDate - $startDate).TotalSeconds)
$xLabelFormat = if (($endDate - $startDate).TotalDays -lt 14.0) {
    "MM-dd HH:mm"
} elseif (($endDate - $startDate).TotalDays -lt 730.0) {
    "yyyy-MM-dd"
} else {
    "yyyy-MM"
}

$starCount = $dates.Count
$yStep = if ($starCount -le 10) { 2 } elseif ($starCount -le 25) { 5 } elseif ($starCount -le 100) { 10 } else { 50 }
$yMax = [Math]::Max($yStep, [Math]::Ceiling($starCount / [double]$yStep) * $yStep)

function Get-ChartX {
    param([DateTimeOffset]$Date)
    $chartLeft + (($Date.ToUniversalTime() - $startDate).TotalSeconds / $spanSeconds) * $chartWidth
}

function Get-ChartY {
    param([double]$CountValue)
    $chartBottom - ($CountValue / $yMax) * $chartHeight
}

$grid = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -le 5; $i++) {
    $ratio = $i / 5.0
    $y = $chartBottom - $ratio * $chartHeight
    $value = [int][Math]::Round($ratio * $yMax)
    $grid.Add("  <line class=`"grid`" x1=`"$chartLeft`" y1=`"$(Format-Number $y)`" x2=`"$chartRight`" y2=`"$(Format-Number $y)`"/>")
    $grid.Add("  <text class=`"axis-label`" x=`"54`" y=`"$(Format-Number ($y + 4))`" text-anchor=`"end`">$value</text>")
}

for ($i = 0; $i -le 4; $i++) {
    $ratio = $i / 4.0
    $x = $chartLeft + $ratio * $chartWidth
    $tickDate = $startDate.AddSeconds($ratio * $spanSeconds)
    $anchor = if ($i -eq 0) { "start" } elseif ($i -eq 4) { "end" } else { "middle" }
    $grid.Add("  <line class=`"grid vertical`" x1=`"$(Format-Number $x)`" y1=`"$chartTop`" x2=`"$(Format-Number $x)`" y2=`"$chartBottom`"/>")
    $grid.Add("  <text class=`"axis-label`" x=`"$(Format-Number $x)`" y=`"326`" text-anchor=`"$anchor`">$($tickDate.ToString($xLabelFormat, $culture))</text>")
}

$firstX = Get-ChartX $dates[0]
$firstY = Get-ChartY 1
$lineParts = [System.Collections.Generic.List[string]]::new()
$areaParts = [System.Collections.Generic.List[string]]::new()
$lineParts.Add("M $(Format-Number $firstX) $(Format-Number $firstY)")
$areaParts.Add("M $(Format-Number $firstX) $(Format-Number $chartBottom)")
$areaParts.Add("L $(Format-Number $firstX) $(Format-Number $firstY)")

for ($i = 1; $i -lt $dates.Count; $i++) {
    $x = Get-ChartX $dates[$i]
    $previousY = Get-ChartY $i
    $currentY = Get-ChartY ($i + 1)
    $lineParts.Add("L $(Format-Number $x) $(Format-Number $previousY)")
    $lineParts.Add("L $(Format-Number $x) $(Format-Number $currentY)")
    $areaParts.Add("L $(Format-Number $x) $(Format-Number $previousY)")
    $areaParts.Add("L $(Format-Number $x) $(Format-Number $currentY)")
}

$lastY = Get-ChartY $starCount
$lineParts.Add("L $(Format-Number $chartRight) $(Format-Number $lastY)")
$areaParts.Add("L $(Format-Number $chartRight) $(Format-Number $lastY)")
$areaParts.Add("L $(Format-Number $chartRight) $(Format-Number $chartBottom) Z")

$repositoryLabel = [System.Security.SecurityElement]::Escape($Repository)
$gridMarkup = $grid -join "`n"
$linePath = $lineParts -join " "
$areaPath = $areaParts -join " "
$updatedAt = $lastDate.ToString("yyyy-MM-dd HH:mm 'UTC'", $culture)
$lastX = $chartRight

$svg = @"
<svg xmlns="http://www.w3.org/2000/svg" width="760" height="360" viewBox="0 0 760 360" role="img" aria-labelledby="title description">
  <title id="title">$repositoryLabel GitHub star history: $starCount stars</title>
  <desc id="description">A cumulative timeline of GitHub stars received by $repositoryLabel.</desc>
  <style>
    .card { fill: #f3f3f3; stroke: #d1d1d1; }
    .title { fill: #1a1a1a; font: 600 18px "Segoe UI Variable", "Segoe UI", sans-serif; }
    .subtitle, .axis-label, .updated { fill: #616161; font: 400 12px "Segoe UI Variable", "Segoe UI", sans-serif; }
    .count { fill: #1a1a1a; font: 600 20px "Segoe UI Variable", "Segoe UI", sans-serif; }
    .grid { stroke: #d9d9d9; stroke-width: 1; }
    .vertical { stroke-dasharray: 2 5; }
    .line { fill: none; stroke: #005fb8; stroke-width: 2; stroke-linejoin: round; }
    .area { fill: #005fb8; fill-opacity: .08; }
    .point { fill: #f3f3f3; stroke: #005fb8; stroke-width: 2; }
    @media (prefers-color-scheme: dark) {
      .card { fill: #202020; stroke: #3b3b3b; }
      .title, .count { fill: #ffffff; }
      .subtitle, .axis-label, .updated { fill: #c5c5c5; }
      .grid { stroke: #3b3b3b; }
      .line { stroke: #60cdff; }
      .area { fill: #60cdff; }
      .point { fill: #202020; stroke: #60cdff; }
    }
  </style>
  <rect class="card" x=".5" y=".5" width="759" height="359" rx="8"/>
  <text class="title" x="24" y="31">Star history · 收藏量趋势图</text>
  <text class="subtitle" x="24" y="51">$repositoryLabel · cumulative GitHub stars by date</text>
  <text class="count" x="732" y="34" text-anchor="end">$starCount stars</text>
$gridMarkup
  <path class="area" d="$areaPath"/>
  <path class="line" d="$linePath"/>
  <circle class="point" cx="$(Format-Number $lastX)" cy="$(Format-Number $lastY)" r="4"/>
  <text class="updated" x="732" y="347" text-anchor="end">Last star: $updatedAt</text>
</svg>
"@

$resolvedOutput = Join-Path (Get-Location) $OutputPath
$outputDirectory = Split-Path -Parent $resolvedOutput
if ($outputDirectory) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}
[System.IO.File]::WriteAllText(
    $resolvedOutput,
    $svg,
    [System.Text.UTF8Encoding]::new($false)
)

Write-Host "star history updated: repository=$Repository stars=$starCount output=$resolvedOutput"
