param(
    [string]$Repository = "qiu7824/zsui",
    [string]$OutputPath = "docs/images/star-progress.svg",
    [ValidateRange(1, [int]::MaxValue)]
    [int]$Goal = 100
)

$ErrorActionPreference = "Stop"

$headers = @{
    Accept = "application/vnd.github+json"
    "User-Agent" = "zsui-star-progress"
    "X-GitHub-Api-Version" = "2022-11-28"
}
if ($env:GITHUB_TOKEN) {
    $headers.Authorization = "Bearer $env:GITHUB_TOKEN"
}

$repositoryData = Invoke-RestMethod `
    -Uri "https://api.github.com/repos/$Repository" `
    -Headers $headers
$stars = [int]$repositoryData.stargazers_count

if ($stars -ge $Goal) {
    $Goal = [int]([Math]::Ceiling(($stars + 1) / 100.0) * 100)
}

$trackX = 40.0
$trackWidth = 680.0
$ratio = [Math]::Min(1.0, $stars / [double]$Goal)
$fillWidth = if ($stars -gt 0) {
    [Math]::Max(10.0, $trackWidth * $ratio)
} else {
    0.0
}
$headX = $trackX + $fillWidth
$culture = [Globalization.CultureInfo]::InvariantCulture
$fillWidthText = $fillWidth.ToString("0.##", $culture)
$headXText = $headX.ToString("0.##", $culture)
$quarter = [int][Math]::Round($Goal * 0.25)
$half = [int][Math]::Round($Goal * 0.50)
$threeQuarters = [int][Math]::Round($Goal * 0.75)
$repositoryLabel = [System.Security.SecurityElement]::Escape($Repository)

$svg = @"
<svg xmlns="http://www.w3.org/2000/svg" width="760" height="128" viewBox="0 0 760 128" role="img" aria-labelledby="title description">
  <title id="title">$repositoryLabel star journey: $stars of $Goal stars</title>
  <desc id="description">A milestone progress line showing the repository's current GitHub star count.</desc>
  <style>
    .card { fill: #f6f8fa; stroke: #d0d7de; }
    .title { fill: #1f2328; font: 700 15px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; letter-spacing: .5px; }
    .subtitle { fill: #59636e; font: 500 11px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    .count { fill: #1f2328; font: 700 20px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    .track { fill: #d8dee4; }
    .marker { fill: #f6f8fa; stroke: #8c959f; stroke-width: 2; }
    .marker-label { fill: #59636e; font: 600 10px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    @media (prefers-color-scheme: dark) {
      .card { fill: #0d1117; stroke: #30363d; }
      .title, .count { fill: #f0f6fc; }
      .subtitle, .marker-label { fill: #8b949e; }
      .track { fill: #30363d; }
      .marker { fill: #0d1117; stroke: #6e7681; }
    }
  </style>
  <defs>
    <linearGradient id="accent" x1="0" x2="1">
      <stop offset="0" stop-color="#2f81f7"/>
      <stop offset=".55" stop-color="#8250df"/>
      <stop offset="1" stop-color="#d2a8ff"/>
    </linearGradient>
    <filter id="glow" x="-50%" y="-100%" width="200%" height="300%">
      <feGaussianBlur stdDeviation="3" result="blur"/>
      <feMerge>
        <feMergeNode in="blur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <clipPath id="track-clip">
      <rect x="40" y="76" width="680" height="12" rx="6"/>
    </clipPath>
  </defs>
  <rect class="card" x=".5" y=".5" width="759" height="127" rx="16"/>
  <circle cx="46" cy="37" r="19" fill="url(#accent)"/>
  <path d="m46 24.5 3.8 7.7 8.5 1.2-6.1 6 1.4 8.4-7.6-4-7.6 4 1.4-8.4-6.1-6 8.5-1.2z" fill="#fff"/>
  <text class="title" x="76" y="34">ZSUI STAR JOURNEY · 收藏进度</text>
  <text class="subtitle" x="76" y="53">NEXT MILESTONE · 下一目标</text>
  <text class="count" x="716" y="44" text-anchor="end">$stars / $Goal</text>
  <rect class="track" x="40" y="76" width="680" height="12" rx="6"/>
  <rect x="40" y="76" width="$fillWidthText" height="12" fill="url(#accent)" clip-path="url(#track-clip)"/>
  <circle cx="$headXText" cy="82" r="5" fill="#d2a8ff" filter="url(#glow)"/>
  <g>
    <circle class="marker" cx="210" cy="82" r="5"/>
    <circle class="marker" cx="380" cy="82" r="5"/>
    <circle class="marker" cx="550" cy="82" r="5"/>
    <circle class="marker" cx="720" cy="82" r="5"/>
    <text class="marker-label" x="210" y="108" text-anchor="middle">$quarter</text>
    <text class="marker-label" x="380" y="108" text-anchor="middle">$half</text>
    <text class="marker-label" x="550" y="108" text-anchor="middle">$threeQuarters</text>
    <text class="marker-label" x="720" y="108" text-anchor="middle">$Goal ★</text>
  </g>
</svg>
"@

$resolvedOutput = if ([IO.Path]::IsPathFullyQualified($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path (Get-Location) $OutputPath))
}
$outputDirectory = [IO.Path]::GetDirectoryName($resolvedOutput)
[IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
[IO.File]::WriteAllText(
    $resolvedOutput,
    $svg + [Environment]::NewLine,
    [Text.UTF8Encoding]::new($false)
)

Write-Host "star progress updated: repository=$Repository stars=$stars goal=$Goal output=$resolvedOutput"
