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
            vec!["windows-win32", "desktop-winit"],
            "window declarations and the target-native Win32 or Winit desktop host path",
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
            "textbox",
            Widget,
            false,
            Vec::new(),
            vec!["widgets-input"],
            "text input component declarations",
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
            vec!["widgets-input", "widgets-list"],
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
            "all-widgets",
            Profile,
            false,
            Vec::new(),
            vec![
                "button", "label", "scroll", "list", "textbox", "checkbox", "table",
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
                "dark-mode",
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
        assert_eq!(window.enables, vec!["windows-win32", "desktop-winit"]);
    }

    #[test]
    fn optional_dependency_features_are_explicit() {
        let names = zsui_optional_dependency_feature_names();

        assert!(names.contains(&"clipboard"));
        assert!(names.contains(&"image"));
        assert!(names.contains(&"desktop-winit"));
        assert!(names.contains(&"windows-gdi"));
        assert!(!names.contains(&"button"));
        assert!(!names.contains(&"label"));
    }

    #[test]
    fn widget_profile_is_opt_in_not_default() {
        let manifest = zsui_feature_manifest();
        let all_widgets = manifest
            .iter()
            .find(|feature| feature.name == "all-widgets")
            .expect("all-widgets feature should be listed");

        assert!(!all_widgets.default_enabled);
        assert!(all_widgets.enables.contains(&"textbox"));
        assert!(all_widgets.enables.contains(&"table"));
    }
}
