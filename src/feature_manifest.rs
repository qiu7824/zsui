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
            vec!["windows-win32", "macos-appkit", "linux-gtk"],
            "window declarations and target-native Win32, AppKit or GTK4 desktop host paths",
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
            "input widget declarations such as text boxes and checkboxes",
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
            vec!["widgets-input"],
            "text input component declarations",
        ),
        ZsuiCargoFeature::new(
            "password-box",
            Widget,
            false,
            vec!["zeroize"],
            vec!["widgets-input"],
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
            vec!["widgets-input"],
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
            vec!["list", "scroll"],
            "table widget declarations layered on list and scroll support",
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
            "self-drawn document tab, command bar, editor frame and status layout",
        ),
        ZsuiCargoFeature::new(
            "calculator",
            Shell,
            false,
            vec!["rust_decimal"],
            vec!["style"],
            "decimal calculator engine, Fluent shell layout, semantic draw plan and typed actions",
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
            "image",
            Service,
            false,
            vec!["png"],
            Vec::new(),
            "PNG decode/encode helpers used by smoke screenshots and GDI icons",
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
            vec!["windows-win32", "document-shell"],
            "Windows native text-service benchmark and notepad application example",
        ),
        ZsuiCargoFeature::new(
            "calculator-demo",
            Tooling,
            false,
            Vec::new(),
            vec!["windows-gdi", "calculator"],
            "interactive Windows calculator example and local comparison target",
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
            vec!["windows-win32", "macos-appkit", "linux-gtk"],
            "target-native desktop backend profile without exposing platform handles to applications",
        ),
        ZsuiCargoFeature::new(
            "all-widgets",
            Profile,
            false,
            Vec::new(),
            vec![
                "button",
                "toggle-button",
                "label",
                "grid",
                "scroll",
                "list",
                "virtual-list",
                "paged-list",
                "textbox",
                "password-box",
                "tooltip",
                "checkbox",
                "toggle",
                "slider",
                "number-box",
                "radio",
                "progress",
                "progress-ring",
                "combo",
                "date-picker",
                "time-picker",
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
                "all-widgets",
                "clipboard",
                "calculator",
                "dark-mode",
                "document-shell",
                "desktop-native",
                "desktop-winit",
                "hotkey",
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
            vec!["windows-win32", "macos-appkit", "linux-gtk"]
        );
    }

    #[test]
    fn optional_dependency_features_are_explicit() {
        let names = zsui_optional_dependency_feature_names();

        assert!(names.contains(&"clipboard"));
        assert!(names.contains(&"calculator"));
        assert!(names.contains(&"image"));
        assert!(names.contains(&"desktop-winit"));
        assert!(names.contains(&"windows-gdi"));
        assert!(names.contains(&"macos-appkit"));
        assert!(names.contains(&"linux-gtk"));
        assert!(names.contains(&"password-box"));
        assert!(!names.contains(&"button"));
        assert!(!names.contains(&"label"));
    }

    #[test]
    fn widget_profile_is_opt_in_not_default() {
        let manifest = zsui_feature_manifest();
        let grid = manifest
            .iter()
            .find(|feature| feature.name == "grid")
            .expect("grid feature should be listed");
        let all_widgets = manifest
            .iter()
            .find(|feature| feature.name == "all-widgets")
            .expect("all-widgets feature should be listed");

        assert!(!grid.default_enabled);
        assert!(!all_widgets.default_enabled);
        assert!(all_widgets.enables.contains(&"grid"));
        assert!(all_widgets.enables.contains(&"textbox"));
        assert!(all_widgets.enables.contains(&"password-box"));
        assert!(all_widgets.enables.contains(&"tooltip"));
        assert!(all_widgets.enables.contains(&"toggle-button"));
        assert!(all_widgets.enables.contains(&"toggle"));
        assert!(all_widgets.enables.contains(&"slider"));
        assert!(all_widgets.enables.contains(&"number-box"));
        assert!(all_widgets.enables.contains(&"radio"));
        assert!(all_widgets.enables.contains(&"progress"));
        assert!(all_widgets.enables.contains(&"progress-ring"));
        assert!(all_widgets.enables.contains(&"combo"));
        assert!(all_widgets.enables.contains(&"date-picker"));
        assert!(all_widgets.enables.contains(&"time-picker"));
        assert!(all_widgets.enables.contains(&"tabs"));
        assert!(all_widgets.enables.contains(&"table"));
        assert!(all_widgets.enables.contains(&"workbench"));
    }
}
