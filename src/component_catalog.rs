use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsuiComponentCategory {
    Layout,
    Navigation,
    Input,
    Collection,
    Feedback,
    Overlay,
    Media,
    Composite,
}

impl ZsuiComponentCategory {
    pub const fn category_name(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Navigation => "navigation",
            Self::Input => "input",
            Self::Collection => "collection",
            Self::Feedback => "feedback",
            Self::Overlay => "overlay",
            Self::Media => "media",
            Self::Composite => "composite",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsuiComponentStatus {
    Ready,
    FirstPass,
    ContractOnly,
    NotStarted,
}

impl ZsuiComponentStatus {
    pub const fn status_name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::FirstPass => "first_pass",
            Self::ContractOnly => "contract_only",
            Self::NotStarted => "not_started",
        }
    }

    pub const fn has_runtime_surface(self) -> bool {
        matches!(self, Self::Ready | Self::FirstPass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ZsuiComponentDescriptor {
    pub component_name: &'static str,
    pub winui_analogue: &'static str,
    pub category: ZsuiComponentCategory,
    pub status: ZsuiComponentStatus,
    pub feature_name: Option<&'static str>,
    pub source_path: &'static str,
    pub missing_before_ready: &'static [&'static str],
}

impl ZsuiComponentDescriptor {
    pub const fn category_name(self) -> &'static str {
        self.category.category_name()
    }

    pub const fn status_name(self) -> &'static str {
        self.status.status_name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZsuiComponentCatalogSummary {
    pub total_count: usize,
    pub ready_count: usize,
    pub first_pass_count: usize,
    pub contract_only_count: usize,
    pub not_started_count: usize,
    pub runtime_surface_count: usize,
    pub missing_component_names: Vec<&'static str>,
}

pub fn zsui_component_catalog() -> &'static [ZsuiComponentDescriptor] {
    ZSUI_COMPONENT_CATALOG
}

pub fn zsui_component_catalog_summary() -> ZsuiComponentCatalogSummary {
    let ready_count = ZSUI_COMPONENT_CATALOG
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::Ready)
        .count();
    let first_pass_count = ZSUI_COMPONENT_CATALOG
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::FirstPass)
        .count();
    let contract_only_count = ZSUI_COMPONENT_CATALOG
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::ContractOnly)
        .count();
    let not_started_count = ZSUI_COMPONENT_CATALOG
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::NotStarted)
        .count();

    ZsuiComponentCatalogSummary {
        total_count: ZSUI_COMPONENT_CATALOG.len(),
        ready_count,
        first_pass_count,
        contract_only_count,
        not_started_count,
        runtime_surface_count: ready_count + first_pass_count,
        missing_component_names: ZSUI_COMPONENT_CATALOG
            .iter()
            .filter(|component| !component.status.has_runtime_surface())
            .map(|component| component.component_name)
            .collect(),
    }
}

const INPUT_GAPS: &[&str] = &["complete IME", "accessibility", "non-Windows native input"];
const VIRTUAL_LIST_GAPS: &[&str] = &[
    "variable-height row metrics",
    "scrollbar thumb dragging",
    "non-Windows runtime smoke",
];
const PLATFORM_GAPS: &[&str] = &["native platform binding", "target interaction smoke"];
const DOCUMENT_SHELL_GAPS: &[&str] = &[
    "keyboard focus and accessibility provider",
    "dark and high-contrast target smoke",
    "non-Windows native host binding",
];
const ICON_GAPS: &[&str] = &[
    "GTK and macOS native icon-theme binding",
    "bundled vector fallback",
    "high-contrast target smoke",
];
const WORKBENCH_GAPS: &[&str] = &[
    "native editable composer input",
    "hover pressed focus-visible state matrix",
    "dark and high-contrast target smoke",
    "macOS and GTK native binding",
];
const NEW_COMPONENT_GAPS: &[&str] = &["declaration API", "layout", "paint", "input", "smoke"];

macro_rules! component {
    ($name:literal, $analogue:literal, $category:ident, $status:ident, $feature:expr, $path:literal, $gaps:expr) => {
        ZsuiComponentDescriptor {
            component_name: $name,
            winui_analogue: $analogue,
            category: ZsuiComponentCategory::$category,
            status: ZsuiComponentStatus::$status,
            feature_name: $feature,
            source_path: $path,
            missing_before_ready: $gaps,
        }
    };
}

pub const ZSUI_COMPONENT_CATALOG: &[ZsuiComponentDescriptor] = &[
    component!(
        "stack",
        "StackPanel",
        Layout,
        FirstPass,
        None,
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "grid",
        "Grid",
        Layout,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "border",
        "Border",
        Layout,
        FirstPass,
        None,
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "scroll",
        "ScrollViewer",
        Layout,
        FirstPass,
        Some("scroll"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "split_view",
        "SplitView",
        Layout,
        FirstPass,
        Some("shell"),
        "src/shell_layout.rs",
        PLATFORM_GAPS
    ),
    component!(
        "canvas",
        "Canvas",
        Layout,
        ContractOnly,
        None,
        "src/render_protocol.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "navigation",
        "NavigationView",
        Navigation,
        FirstPass,
        Some("shell"),
        "src/shell_layout.rs",
        PLATFORM_GAPS
    ),
    component!(
        "tabs",
        "TabView",
        Navigation,
        ContractOnly,
        None,
        "src/components.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "breadcrumb",
        "BreadcrumbBar",
        Navigation,
        NotStarted,
        None,
        "src/components.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "command_bar",
        "CommandBar",
        Navigation,
        FirstPass,
        Some("document-shell"),
        "src/document_shell.rs",
        DOCUMENT_SHELL_GAPS
    ),
    component!(
        "text",
        "TextBlock",
        Input,
        FirstPass,
        Some("label"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "button",
        "Button",
        Input,
        FirstPass,
        Some("button"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "toggle_button",
        "ToggleButton",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "checkbox",
        "CheckBox",
        Input,
        FirstPass,
        Some("checkbox"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "toggle",
        "ToggleSwitch",
        Input,
        FirstPass,
        Some("toggle"),
        "src/widget_render.rs",
        INPUT_GAPS
    ),
    component!(
        "textbox",
        "TextBox",
        Input,
        FirstPass,
        Some("textbox"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "password_box",
        "PasswordBox",
        Input,
        ContractOnly,
        Some("settings"),
        "src/control_protocol.rs",
        PLATFORM_GAPS
    ),
    component!(
        "combo_box",
        "ComboBox",
        Input,
        ContractOnly,
        Some("settings"),
        "src/native_hosts.rs",
        PLATFORM_GAPS
    ),
    component!(
        "radio_buttons",
        "RadioButtons",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "slider",
        "Slider",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "number_box",
        "NumberBox",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "auto_suggest",
        "AutoSuggestBox",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "date_picker",
        "DatePicker",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "time_picker",
        "TimePicker",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "color_picker",
        "ColorPicker",
        Input,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "list",
        "ListView",
        Collection,
        FirstPass,
        Some("list"),
        "src/view.rs",
        INPUT_GAPS
    ),
    component!(
        "grid_view",
        "GridView",
        Collection,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "tree",
        "TreeView",
        Collection,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "table",
        "DataGrid",
        Collection,
        ContractOnly,
        Some("table"),
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "items_repeater",
        "ItemsRepeater",
        Collection,
        FirstPass,
        Some("virtual-list"),
        "src/view.rs + src/paged_list.rs",
        VIRTUAL_LIST_GAPS
    ),
    component!(
        "badge",
        "InfoBadge",
        Feedback,
        FirstPass,
        Some("shell"),
        "src/shell_layout.rs",
        PLATFORM_GAPS
    ),
    component!(
        "progress_bar",
        "ProgressBar",
        Feedback,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "progress_ring",
        "ProgressRing",
        Feedback,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "info_bar",
        "InfoBar",
        Feedback,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "tooltip",
        "ToolTip",
        Feedback,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "content_dialog",
        "ContentDialog",
        Overlay,
        ContractOnly,
        None,
        "src/host_protocol.rs",
        PLATFORM_GAPS
    ),
    component!(
        "flyout",
        "Flyout",
        Overlay,
        ContractOnly,
        None,
        "src/host_protocol.rs",
        PLATFORM_GAPS
    ),
    component!(
        "menu_flyout",
        "MenuFlyout",
        Overlay,
        FirstPass,
        Some("tray"),
        "src/windows_win32_host.rs",
        PLATFORM_GAPS
    ),
    component!(
        "teaching_tip",
        "TeachingTip",
        Overlay,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "command_palette",
        "CommandPalette",
        Overlay,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "image",
        "Image",
        Media,
        ContractOnly,
        Some("image"),
        "src/render_protocol.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "icon",
        "FontIcon/ImageIcon",
        Media,
        FirstPass,
        Some("image"),
        "src/icon.rs",
        ICON_GAPS
    ),
    component!(
        "webview",
        "WebView2",
        Media,
        NotStarted,
        None,
        "src/view.rs",
        NEW_COMPONENT_GAPS
    ),
    component!(
        "settings_card",
        "SettingsCard",
        Composite,
        FirstPass,
        Some("shell"),
        "src/shell_layout.rs",
        PLATFORM_GAPS
    ),
    component!(
        "workbench_shell",
        "NavigationView + CommandBar",
        Composite,
        FirstPass,
        Some("workbench"),
        "src/workbench.rs",
        WORKBENCH_GAPS
    ),
    component!(
        "message_timeline",
        "ItemsRepeater + ScrollViewer",
        Composite,
        FirstPass,
        Some("workbench"),
        "src/workbench.rs",
        WORKBENCH_GAPS
    ),
    component!(
        "composer",
        "RichEditBox + CommandBar",
        Composite,
        FirstPass,
        Some("workbench"),
        "src/workbench.rs",
        WORKBENCH_GAPS
    ),
    component!(
        "inspector_panel",
        "SplitView Pane",
        Composite,
        FirstPass,
        Some("workbench"),
        "src/workbench.rs",
        WORKBENCH_GAPS
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_reports_runtime_and_missing_components_separately() {
        let summary = zsui_component_catalog_summary();

        assert_eq!(summary.total_count, ZSUI_COMPONENT_CATALOG.len());
        assert!(summary.runtime_surface_count >= 15);
        assert!(summary.not_started_count >= 15);
        assert!(summary.missing_component_names.contains(&"tree"));
        assert!(summary.missing_component_names.contains(&"progress_ring"));
        assert!(!summary.missing_component_names.contains(&"toggle"));
    }

    #[test]
    fn every_component_has_a_unique_name_and_evidence_path() {
        let mut names = std::collections::BTreeSet::new();
        for component in ZSUI_COMPONENT_CATALOG {
            assert!(names.insert(component.component_name));
            assert!(!component.source_path.is_empty());
            assert!(!component.winui_analogue.is_empty());
        }
    }
}
