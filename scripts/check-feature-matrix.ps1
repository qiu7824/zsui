param(
    [switch]$Locked
)

$ErrorActionPreference = "Stop"

$cargoArgs = @("check", "--no-default-features")
if ($Locked) {
    $cargoArgs += "--locked"
}

$singleFeatures = @(
    "accessibility",
    "window",
    "button",
    "breadcrumb",
    "canvas",
    "flyout",
    "menu-flyout",
    "toggle-button",
    "label",
    "grid",
    "grid-view",
    "widgets-base",
    "widgets-input",
    "text-input-core",
    "widgets-list",
    "scroll",
    "list",
    "virtual-list",
    "paged-list",
    "textbox",
    "password-box",
    "tooltip",
    "dialog",
    "toast",
    "info-bar",
    "teaching-tip",
    "checkbox",
    "toggle",
    "slider",
    "number-box",
    "radio",
    "progress",
    "progress-ring",
    "auto-suggest",
    "command-palette",
    "tree",
    "combo",
    "date-picker",
    "time-picker",
    "color-picker",
    "tabs",
    "table",
    "dark-mode",
    "style",
    "localization",
    "shell",
    "workbench",
    "document-shell",
    "calculator",
    "tray",
    "hotkey",
    "settings",
    "product-adapter",
    "android",
    "mobile",
    "clipboard",
    "image",
    "image-preview",
    "native-smoke",
    "fluent-icons",
    "notepad-demo",
    "notepad-demo-lite",
    "calculator-demo",
    "component-gallery-demo",
    "desktop-winit",
    "windows-gdi",
    "windows-win32",
    "macos-appkit",
    "linux-direct",
    "linux-direct-host",
    "linux-direct-lite",
    "linux-system-icons",
    "linux-gtk",
    "desktop-native",
    "all-widgets",
    "full"
)

$featureSets = @(
    "button,breadcrumb,canvas,flyout,menu-flyout,label,grid",
    "textbox,password-box,tooltip,dialog,toast,info-bar,teaching-tip,checkbox,toggle,toggle-button,slider,number-box,radio,progress,progress-ring,auto-suggest,command-palette,combo,date-picker,time-picker,color-picker,tabs",
    "list,grid-view,tree,table",
    "virtual-list,paged-list,label",
    "image-preview,window",
    "window,shell,tray,hotkey",
    "window,product-adapter,button,label",
    "localization,button,label",
    "all-widgets,style,dark-mode",
    "workbench,window",
    "document-shell,windows-win32",
    "notepad-demo,style",
    "notepad-demo-lite,style",
    "calculator,window",
    "component-gallery-demo",
    "desktop-native,all-widgets,style,dark-mode"
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
