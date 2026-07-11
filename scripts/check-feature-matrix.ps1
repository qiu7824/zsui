param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"

$cargoArgs = @("check", "--no-default-features")
if ($Locked) {
    $cargoArgs += "--locked"
}

$singleFeatures = @(
    "window",
    "button",
    "label",
    "widgets-base",
    "widgets-input",
    "widgets-list",
    "scroll",
    "list",
    "textbox",
    "checkbox",
    "toggle",
    "table",
    "dark-mode",
    "style",
    "shell",
    "workbench",
    "document-shell",
    "calculator",
    "tray",
    "hotkey",
    "settings",
    "product-adapter",
    "android",
    "harmony",
    "mobile",
    "clipboard",
    "image",
    "native-smoke",
    "notepad-demo",
    "calculator-demo",
    "desktop-winit",
    "windows-gdi",
    "windows-win32",
    "all-widgets",
    "full"
)

$featureSets = @(
    "button,label",
    "textbox,checkbox,toggle",
    "list,table",
    "window,shell,tray,hotkey",
    "window,product-adapter,button,label",
    "all-widgets,style,dark-mode",
    "workbench,window",
    "document-shell,windows-win32",
    "notepad-demo,style",
    "calculator,windows-gdi"
)

function Invoke-CargoFeatureCheck {
    param([string]$Features)

    Write-Host "cargo feature check: $Features"
    & cargo @cargoArgs --features $Features
    if ($LASTEXITCODE -ne 0) {
        throw "cargo check failed for features: $Features"
    }
}

$metadataArgs = @("metadata", "--format-version", "1", "--no-deps")
if ($Locked) {
    $metadataArgs += "--locked"
}
$metadata = (& cargo @metadataArgs | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed"
}
$manifestFeatures = @(
    $metadata.packages[0].features.PSObject.Properties.Name |
        Where-Object { $_ -ne "default" } |
        Sort-Object
)
$checkedFeatures = @($singleFeatures | Sort-Object)
$missingFeatures = @($manifestFeatures | Where-Object { $_ -notin $checkedFeatures })
$unknownFeatures = @($checkedFeatures | Where-Object { $_ -notin $manifestFeatures })
if ($missingFeatures.Count -gt 0 -or $unknownFeatures.Count -gt 0) {
    throw "feature matrix mismatch; missing=[$($missingFeatures -join ',')], unknown=[$($unknownFeatures -join ',')]"
}

foreach ($feature in $singleFeatures) {
    Invoke-CargoFeatureCheck -Features $feature
}

foreach ($features in $featureSets) {
    Invoke-CargoFeatureCheck -Features $features
}

Write-Host "cargo feature matrix passed"
