param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"

$workspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$metadataArgs = @(
    "metadata",
    "--format-version", "1",
    "--manifest-path", (Join-Path $workspace "Cargo.toml")
)
if ($Locked) {
    $metadataArgs += "--locked"
}

$metadata = (& cargo @metadataArgs | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed while checking the native-only boundary"
}

$forbiddenPackagePattern = '^(wry|tauri($|-)|webview2($|-)|webkit2gtk($|-)|webkit6($|-)|cef($|-)|chromiumoxide($|-)|headless_chrome($|-))'
$forbiddenPackages = @(
    $metadata.packages |
        Where-Object { $_.name -match $forbiddenPackagePattern } |
        ForEach-Object { "$($_.name)@$($_.version)" } |
        Sort-Object -Unique
)
if ($forbiddenPackages.Count -gt 0) {
    throw "browser-shell packages are forbidden in the ZSUI root graph: $($forbiddenPackages -join ', ')"
}

$sourceRoots = @(
    (Join-Path $workspace "src"),
    (Join-Path $workspace "examples")
)
$buildScript = Join-Path $workspace "build.rs"
if (Test-Path -LiteralPath $buildScript) {
    $sourceRoots += $buildScript
}

$forbiddenApiPattern = 'ICoreWebView2|CreateCoreWebView2Environment|WKWebView|WebKitWebView|webview2::|webkit2gtk::|webkit6::|wry::|tauri::|cef::|chromiumoxide::|headless_chrome::'
$matches = @(& rg -n -i --glob '*.rs' -- $forbiddenApiPattern @sourceRoots 2>$null)
$rgExitCode = $LASTEXITCODE
if ($rgExitCode -gt 1) {
    throw "rg failed while checking browser-shell API usage"
}
if ($matches.Count -gt 0) {
    throw "browser-shell APIs are forbidden in ZSUI source:`n$($matches -join [Environment]::NewLine)"
}

Write-Host "native-only boundary passed: no WebView/browser-shell package or API"
exit 0
