use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ZsuiFeatureCategory {
    Core,
    Shell,
    Widget,
    Platform,
    Backend,
    Service,
    Tooling,
    Profile,
}

impl ZsuiFeatureCategory {
    pub const fn category_name(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Shell => "shell",
            Self::Widget => "widget",
            Self::Platform => "platform",
            Self::Backend => "backend",
            Self::Service => "service",
            Self::Tooling => "tooling",
            Self::Profile => "profile",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiCargoFeature {
    pub name: &'static str,
    pub category: ZsuiFeatureCategory,
    pub category_name: &'static str,
    pub default_enabled: bool,
    pub optional_dependency_names: Vec<&'static str>,
    pub enables: Vec<&'static str>,
    pub compile_boundary: &'static str,
}

impl ZsuiCargoFeature {
    pub fn new(
        name: &'static str,
        category: ZsuiFeatureCategory,
        default_enabled: bool,
        optional_dependency_names: Vec<&'static str>,
        enables: Vec<&'static str>,
        compile_boundary: &'static str,
    ) -> Self {
        Self {
            name,
            category,
            category_name: category.category_name(),
            default_enabled,
            optional_dependency_names,
            enables,
            compile_boundary,
        }
    }

    pub fn enables_optional_dependency(&self) -> bool {
        !self.optional_dependency_names.is_empty()
    }
}

pub fn zsui_default_feature_names() -> Vec<&'static str> {
    vec!["window", "button", "label"]
}

pub fn zsui_optional_dependency_feature_names() -> Vec<&'static str> {
    zsui_feature_manifest()
        .into_iter()
        .filter(ZsuiCargoFeature::enables_optional_dependency)
        .map(|feature| feature.name)
        .collect()
}

pub fn zsui_feature_manifest() -> Vec<ZsuiCargoFeature> {
    use ZsuiFeatureCategory::{Backend, Platform, Profile, Service, Shell, Tooling, Widget};

    vec![
        ZsuiCargoFeature::new(
            "window",
            Shell,
            true,
            Vec::new(),
            vec!["windows-win32", "macos-appkit", "linux-direct"],
            "window declarations and target-native Win32, AppKit or lightweight Wayland/X11 host paths",
        ),
        ZsuiCargoFeature::new(
            "button",
            Widget,
            true,
            Vec::new(),
            vec!["widgets-base"],
            "button component declarations and base widget surface",
        ),
        ZsuiCargoFeature::new(
            "label",
            Widget,
            true,
            Vec::new(),
            vec!["widgets-base"],
            "label/text component declarations and renderer-backed Label component",
        ),
        ZsuiCargoFeature::new(
            "grid",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "typed row/column grid layout with fixed and fractional tracks, gaps and spans",
        ),
        ZsuiCargoFeature::new(
            "widgets-base",
            Widget,
            false,
            Vec::new(),
            Vec::new(),
            "small built-in component declarations shared by button and label",
        ),
        ZsuiCargoFeature::new(
            "widgets-input",
            Widget,
            false,
            Vec::new(),
            Vec::new(),
            "input widget declarations such as checkboxes and toggles",
        ),
        ZsuiCargoFeature::new(
            "text-input-core",
            Widget,
            false,
            vec!["unicode-segmentation"],
            vec!["widgets-input"],
            "shared Unicode grapheme editing plus target-native shaped caret, visual navigation and hit geometry",
        ),
        ZsuiCargoFeature::new(
            "widgets-list",
            Widget,
            false,
            Vec::new(),
            Vec::new(),
            "list-like widget declarations",
        ),
        ZsuiCargoFeature::new(
            "scroll",
            Widget,
            false,
            Vec::new(),
            Vec::new(),
            "scroll container declarations and future host contracts",
        ),
        ZsuiCargoFeature::new(
            "list",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-list", "scroll"],
            "list widget declarations",
        ),
        ZsuiCargoFeature::new(
            "tree",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-list"],
            "typed hierarchical rows with strong IDs and self-drawn platform disclosure profiles",
        ),
        ZsuiCargoFeature::new(
            "virtual-list",
            Widget,
            false,
            Vec::new(),
            vec!["list", "scroll"],
            "viewport-based list layout that materializes and paints visible rows only",
        ),
        ZsuiCargoFeature::new(
            "paged-list",
            Widget,
            false,
            Vec::new(),
            vec!["virtual-list"],
            "background page loading, request deduplication and bounded LRU page caching",
        ),
        ZsuiCargoFeature::new(
            "textbox",
            Widget,
            false,
            Vec::new(),
            vec!["text-input-core"],
            "text input component declarations",
        ),
        ZsuiCargoFeature::new(
            "password-box",
            Widget,
            false,
            vec!["zeroize"],
            vec!["text-input-core"],
            "single-line secure input with redacted state, platform reveal policy and self-drawn native profiles",
        ),
        ZsuiCargoFeature::new(
            "tooltip",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "attached noninteractive help overlay with platform metrics and native hover/focus timing",
        ),
        ZsuiCargoFeature::new(
            "dialog",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "modal self-drawn content dialog with semantic responses and platform-specific action layout",
        ),
        ZsuiCargoFeature::new(
            "toggle-button",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "explicit-state toggle button with self-drawn platform profiles and typed activation",
        ),
        ZsuiCargoFeature::new(
            "checkbox",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "checkbox component declarations",
        ),
        ZsuiCargoFeature::new(
            "toggle",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "owner-drawn toggle declarations and typed View input",
        ),
        ZsuiCargoFeature::new(
            "slider",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "range-normalized slider layout, paint and typed pointer or keyboard input",
        ),
        ZsuiCargoFeature::new(
            "number-box",
            Widget,
            false,
            Vec::new(),
            vec!["text-input-core"],
            "editable finite number input with validated range, platform-style steppers and typed commit events",
        ),
        ZsuiCargoFeature::new(
            "radio",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "explicit-state radio button layout, paint and typed selection input",
        ),
        ZsuiCargoFeature::new(
            "progress",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "determinate progress range, semantic paint and feedback-only hit behavior",
        ),
        ZsuiCargoFeature::new(
            "progress-ring",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "independently selectable determinate/indeterminate ring with platform metrics and native animation timers",
        ),
        ZsuiCargoFeature::new(
            "auto-suggest",
            Widget,
            false,
            Vec::new(),
            vec!["text-input-core"],
            "application-owned strong-id suggestions with platform search-field metrics, popup overlay and typed text, choice or submission events",
        ),
        ZsuiCargoFeature::new(
            "command-palette",
            Widget,
            false,
            Vec::new(),
            vec!["text-input-core"],
            "application-owned strong-id commands with self-drawn platform search/list overlays and typed query, highlight, invoke or dismiss events",
        ),
        ZsuiCargoFeature::new(
            "combo",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "explicit-state combo header, popup overlay and typed pointer or keyboard selection",
        ),
        ZsuiCargoFeature::new(
            "date-picker",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "calendar date picker with typed date state, popup month navigation and selection",
        ),
        ZsuiCargoFeature::new(
            "time-picker",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "time picker with typed wall-clock state, minute increments and platform-adaptive popup selection",
        ),
        ZsuiCargoFeature::new(
            "color-picker",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "application-owned RGBA color well with platform-adaptive self-drawn channel editor and typed input",
        ),
        ZsuiCargoFeature::new(
            "tabs",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-base"],
            "self-drawn tab view with strong tab ids, selected content and platform keyboard behavior",
        ),
        ZsuiCargoFeature::new(
            "table",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-list"],
            "typed read-only data grid with strong row and column IDs, sorting and platform-adaptive self-drawn metrics",
        ),
        ZsuiCargoFeature::new(
            "dark-mode",
            Service,
            false,
            Vec::new(),
            Vec::new(),
            "theme selection contracts without forcing a renderer backend",
        ),
        ZsuiCargoFeature::new(
            "style",
            Service,
            false,
            Vec::new(),
            Vec::new(),
            "style tokens, theme data and future renderer style binding",
        ),
        ZsuiCargoFeature::new(
            "localization",
            Service,
            false,
            vec!["fluent-bundle", "sys-locale", "unic-langid"],
            Vec::new(),
            "application-owned Fluent catalogs, locale fallback, runtime language changes and text-direction metadata",
        ),
        ZsuiCargoFeature::new(
            "shell",
            Shell,
            false,
            Vec::new(),
            Vec::new(),
            "desktop shell integration boundary",
        ),
        ZsuiCargoFeature::new(
            "workbench",
            Shell,
            false,
            Vec::new(),
            vec!["button", "label", "scroll", "textbox", "style"],
            "conversation and task workbench shell with navigation, timeline, composer and inspector",
        ),
        ZsuiCargoFeature::new(
            "document-shell",
            Shell,
            false,
            Vec::new(),
            vec!["style"],
            "self-drawn document chrome plus reusable UTF-8/UTF-16 text document lifecycle",
        ),
        ZsuiCargoFeature::new(
            "calculator",
            Shell,
            false,
            vec!["rust_decimal"],
            vec!["style", "button", "label", "grid"],
            "decimal calculator engine and platform-adaptive typed View shell",
        ),
        ZsuiCargoFeature::new(
            "tray",
            Shell,
            false,
            Vec::new(),
            vec!["shell"],
            "tray/status item declarations and host contracts",
        ),
        ZsuiCargoFeature::new(
            "hotkey",
            Shell,
            false,
            Vec::new(),
            vec!["shell"],
            "global hotkey declarations and host contracts",
        ),
        ZsuiCargoFeature::new(
            "settings",
            Shell,
            false,
            Vec::new(),
            vec!["widgets-input", "widgets-list", "toggle"],
            "settings page model and settings control declarations",
        ),
        ZsuiCargoFeature::new(
            "product-adapter",
            Service,
            false,
            Vec::new(),
            Vec::new(),
            "product adapter runtime harness and AI/tool boundary contracts",
        ),
        ZsuiCargoFeature::new(
            "android",
            Platform,
            false,
            Vec::new(),
            Vec::new(),
            "Android Activity host scaffold and future runtime bridge",
        ),
        ZsuiCargoFeature::new(
            "harmony",
            Platform,
            false,
            Vec::new(),
            Vec::new(),
            "Harmony Ability host scaffold and future runtime bridge",
        ),
        ZsuiCargoFeature::new(
            "mobile",
            Platform,
            false,
            Vec::new(),
            vec!["android", "harmony"],
            "combined mobile platform scaffolds",
        ),
        ZsuiCargoFeature::new(
            "clipboard",
            Service,
            false,
            vec!["arboard"],
            Vec::new(),
            "system text clipboard bridge; disabled builds use MemoryHost storage",
        ),
        ZsuiCargoFeature::new(
            "accessibility",
            Service,
            false,
            vec!["windows", "windows-core"],
            Vec::new(),
            "optional native text accessibility adapters: Win32 UI Automation Edit/Value, AppKit text selectors and GTK4 textbox/value semantics",
        ),
        ZsuiCargoFeature::new(
            "image",
            Service,
            false,
            vec!["png"],
            Vec::new(),
            "PNG decode/encode helpers used by smoke screenshots and GDI icons",
        ),
        ZsuiCargoFeature::new(
            "image-preview",
            Widget,
            false,
            Vec::new(),
            vec!["image", "widgets-base"],
            "retained image preview with coalesced background PNG decode and atomic frame replacement",
        ),
        ZsuiCargoFeature::new(
            "native-smoke",
            Tooling,
            false,
            Vec::new(),
            vec!["image"],
            "native target-smoke artifact writing and review helpers",
        ),
        ZsuiCargoFeature::new(
            "fluent-icons",
            Widget,
            false,
            Vec::new(),
            Vec::new(),
            "MIT-licensed Fluent System Icons SVG fallback assets for missing native symbols",
        ),
        ZsuiCargoFeature::new(
            "notepad-demo",
            Tooling,
            false,
            Vec::new(),
            vec![
                "window",
                "button",
                "label",
                "textbox",
                "tabs",
                "dialog",
                "clipboard",
                "document-shell",
            ],
            "shared self-drawn notepad acceptance example on target-native Win32, AppKit and Linux hosts",
        ),
        ZsuiCargoFeature::new(
            "notepad-demo-lite",
            Tooling,
            false,
            Vec::new(),
            vec![
                "linux-direct-lite",
                "button",
                "label",
                "textbox",
                "tabs",
                "dialog",
                "clipboard",
                "document-shell",
                "native-smoke",
            ],
            "same shared notepad source built against the opt-in pure-Rust Linux renderer",
        ),
        ZsuiCargoFeature::new(
            "calculator-demo",
            Tooling,
            false,
            Vec::new(),
            vec!["window", "calculator", "native-smoke"],
            "shared calculator acceptance example on target-native Win32, AppKit and Linux hosts",
        ),
        ZsuiCargoFeature::new(
            "component-gallery-demo",
            Tooling,
            false,
            Vec::new(),
            vec!["window", "all-widgets", "native-smoke", "style", "dark-mode"],
            "complete opt-in component gallery and native smoke acceptance surface",
        ),
        ZsuiCargoFeature::new(
            "desktop-winit",
            Backend,
            false,
            vec!["winit"],
            Vec::new(),
            "first-pass macOS/Linux desktop event loop fallback",
        ),
        ZsuiCargoFeature::new(
            "windows-gdi",
            Backend,
            false,
            vec!["windows-sys"],
            vec!["image"],
            "Windows GDI renderer, text layout and no-flicker paint foundation",
        ),
        ZsuiCargoFeature::new(
            "windows-win32",
            Backend,
            false,
            Vec::new(),
            vec!["windows-gdi"],
            "direct Win32 HWND host, transient window host and message loop",
        ),
        ZsuiCargoFeature::new(
            "macos-appkit",
            Backend,
            false,
            vec!["objc2", "objc2-app-kit", "objc2-foundation"],
            Vec::new(),
            "macOS AppKit backend boundary with native window, clipboard, file-dialog and typed menu services",
        ),
        ZsuiCargoFeature::new(
            "linux-direct-host",
            Backend,
            false,
            vec!["rfd", "softbuffer", "winit"],
            Vec::new(),
            "shared Wayland/X11 window, input, IME, menu, portal and software-presentation host",
        ),
        ZsuiCargoFeature::new(
            "linux-direct",
            Backend,
            false,
            vec!["cairo-rs", "pango", "pangocairo"],
            vec!["linux-direct-host"],
            "lightweight Linux native-window backend with direct software presentation, Pango text, built-in symbolic vectors and XDG portal dialogs",
        ),
        ZsuiCargoFeature::new(
            "linux-direct-lite",
            Backend,
            false,
            vec!["cosmic-text", "tiny-skia"],
            vec!["linux-direct-host"],
            "opt-in pure-Rust Linux renderer using cosmic-text, swash and tiny-skia on the shared Wayland/X11 host",
        ),
        ZsuiCargoFeature::new(
            "linux-system-icons",
            Backend,
            false,
            vec!["freedesktop-icons", "gdk-pixbuf"],
            vec!["linux-direct"],
            "optional freedesktop icon-theme lookup and raster decoding for Linux applications that require exact desktop theme icons",
        ),
        ZsuiCargoFeature::new(
            "linux-gtk",
            Backend,
            false,
            vec!["gtk4"],
            Vec::new(),
            "Linux GTK4 backend boundary with native window, clipboard, file-dialog and typed menu services",
        ),
        ZsuiCargoFeature::new(
            "desktop-native",
            Profile,
            false,
            Vec::new(),
            vec!["windows-win32", "macos-appkit", "linux-direct"],
            "target-native desktop backend profile without exposing platform handles to applications",
        ),
        ZsuiCargoFeature::new(
            "all-widgets",
            Profile,
            false,
            Vec::new(),
            vec![
                "button",
                "breadcrumb",
                "toggle-button",
                "label",
                "grid",
                "grid-view",
                "scroll",
                "list",
                "virtual-list",
                "paged-list",
                "image-preview",
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
                "workbench",
            ],
            "explicit opt-in profile for all current widget feature gates",
        ),
        ZsuiCargoFeature::new(
            "full",
            Profile,
            false,
            Vec::new(),
            vec![
                "accessibility",
                "all-widgets",
                "clipboard",
                "calculator",
                "dark-mode",
                "document-shell",
                "desktop-native",
                "desktop-winit",
                "hotkey",
                "localization",
                "linux-system-icons",
                "mobile",
                "native-smoke",
                "product-adapter",
                "settings",
                "style",
                "tray",
                "window",
            ],
            "developer profile for checking all current optional surfaces together",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_feature_set_stays_small() {
        assert_eq!(
            zsui_default_feature_names(),
            vec!["window", "button", "label"]
        );
        let defaults: Vec<_> = zsui_feature_manifest()
            .into_iter()
            .filter(|feature| feature.default_enabled)
            .map(|feature| feature.name)
            .collect();
        assert_eq!(defaults, zsui_default_feature_names());

        let window = zsui_feature_manifest()
            .into_iter()
            .find(|feature| feature.name == "window")
            .expect("window feature should be listed");
        assert_eq!(
            window.enables,
            vec!["windows-win32", "macos-appkit", "linux-direct"]
        );
    }

    #[test]
    fn optional_dependency_features_are_explicit() {
        let names = zsui_optional_dependency_feature_names();

        assert!(names.contains(&"clipboard"));
        assert!(names.contains(&"accessibility"));
        assert!(names.contains(&"calculator"));
        assert!(names.contains(&"image"));
        assert!(names.contains(&"desktop-winit"));
        assert!(names.contains(&"windows-gdi"));
        assert!(names.contains(&"macos-appkit"));
        assert!(names.contains(&"linux-direct"));
        assert!(names.contains(&"linux-system-icons"));
        assert!(names.contains(&"linux-gtk"));
        assert!(names.contains(&"password-box"));
        assert!(names.contains(&"text-input-core"));
        assert!(names.contains(&"localization"));
        assert!(!names.contains(&"button"));
        assert!(!names.contains(&"label"));
    }

    #[test]
    fn unicode_segmentation_stays_inside_text_capable_input_slices() {
        let manifest = zsui_feature_manifest();
        let text_core = manifest
            .iter()
            .find(|feature| feature.name == "text-input-core")
            .expect("text input core should be listed");
        let textbox = manifest
            .iter()
            .find(|feature| feature.name == "textbox")
            .expect("textbox should be listed");
        let checkbox = manifest
            .iter()
            .find(|feature| feature.name == "checkbox")
            .expect("checkbox should be listed");

        assert_eq!(
            text_core.optional_dependency_names,
            vec!["unicode-segmentation"]
        );
        assert_eq!(text_core.enables, vec!["widgets-input"]);
        assert_eq!(textbox.enables, vec!["text-input-core"]);
        assert_eq!(checkbox.enables, vec!["widgets-input"]);
        assert!(!checkbox.enables.contains(&"text-input-core"));
    }

    #[test]
    fn native_accessibility_stays_out_of_defaults_and_widget_profiles() {
        let manifest = zsui_feature_manifest();
        let accessibility = manifest
            .iter()
            .find(|feature| feature.name == "accessibility")
            .expect("accessibility feature should be listed");
        let all_widgets = manifest
            .iter()
            .find(|feature| feature.name == "all-widgets")
            .expect("all-widgets feature should be listed");
        let full = manifest
            .iter()
            .find(|feature| feature.name == "full")
            .expect("full feature should be listed");

        assert!(!accessibility.default_enabled);
        assert_eq!(
            accessibility.optional_dependency_names,
            vec!["windows", "windows-core"]
        );
        assert!(!all_widgets.enables.contains(&"accessibility"));
        assert!(full.enables.contains(&"accessibility"));
    }

    #[test]
    fn localization_is_an_opt_in_service_included_by_full() {
        let manifest = zsui_feature_manifest();
        let localization = manifest
            .iter()
            .find(|feature| feature.name == "localization")
            .expect("localization feature should be listed");
        let all_widgets = manifest
            .iter()
            .find(|feature| feature.name == "all-widgets")
            .expect("all-widgets feature should be listed");
        let full = manifest
            .iter()
            .find(|feature| feature.name == "full")
            .expect("full feature should be listed");

        assert!(!localization.default_enabled);
        assert_eq!(
            localization.optional_dependency_names,
            vec!["fluent-bundle", "sys-locale", "unic-langid"]
        );
        assert!(!all_widgets.enables.contains(&"localization"));
        assert!(full.enables.contains(&"localization"));
    }

    #[test]
    fn widget_profile_is_opt_in_not_default() {
        let manifest = zsui_feature_manifest();
        let grid = manifest
            .iter()
            .find(|feature| feature.name == "grid")
            .expect("grid feature should be listed");
        let table = manifest
            .iter()
            .find(|feature| feature.name == "table")
            .expect("table feature should be listed");
        let dialog = manifest
            .iter()
            .find(|feature| feature.name == "dialog")
            .expect("dialog feature should be listed");
        let image_preview = manifest
            .iter()
            .find(|feature| feature.name == "image-preview")
            .expect("image preview feature should be listed");
        let all_widgets = manifest
            .iter()
            .find(|feature| feature.name == "all-widgets")
            .expect("all-widgets feature should be listed");

        assert!(!grid.default_enabled);
        assert!(!table.default_enabled);
        assert!(!dialog.default_enabled);
        assert_eq!(dialog.enables, vec!["widgets-base"]);
        assert_eq!(table.enables, vec!["widgets-list"]);
        assert_eq!(image_preview.enables, vec!["image", "widgets-base"]);
        assert!(!table.enables.contains(&"list"));
        assert!(!table.enables.contains(&"scroll"));
        assert!(!all_widgets.default_enabled);
        assert!(all_widgets.enables.contains(&"grid"));
        assert!(all_widgets.enables.contains(&"textbox"));
        assert!(all_widgets.enables.contains(&"password-box"));
        assert!(all_widgets.enables.contains(&"image-preview"));
        assert!(all_widgets.enables.contains(&"tooltip"));
        assert!(all_widgets.enables.contains(&"dialog"));
        assert!(all_widgets.enables.contains(&"toggle-button"));
        assert!(all_widgets.enables.contains(&"toggle"));
        assert!(all_widgets.enables.contains(&"slider"));
        assert!(all_widgets.enables.contains(&"number-box"));
        assert!(all_widgets.enables.contains(&"radio"));
        assert!(all_widgets.enables.contains(&"progress"));
        assert!(all_widgets.enables.contains(&"progress-ring"));
        assert!(all_widgets.enables.contains(&"auto-suggest"));
        assert!(all_widgets.enables.contains(&"command-palette"));
        assert!(all_widgets.enables.contains(&"tree"));
        assert!(all_widgets.enables.contains(&"combo"));
        assert!(all_widgets.enables.contains(&"date-picker"));
        assert!(all_widgets.enables.contains(&"time-picker"));
        assert!(all_widgets.enables.contains(&"color-picker"));
        assert!(all_widgets.enables.contains(&"tabs"));
        assert!(all_widgets.enables.contains(&"table"));
        assert!(all_widgets.enables.contains(&"workbench"));
    }
}
