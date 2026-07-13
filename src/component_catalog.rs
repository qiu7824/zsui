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
const TOOLTIP_GAPS: &[&str] = &[
    "accessibility relationship",
    "top-level overflow popup",
    "macOS and Linux target interaction smoke",
];
const SLIDER_GAPS: &[&str] = &[
    "accessibility range-value provider",
    "AppKit and GTK4 target interaction smoke",
    "touch and precision-trackpad tuning",
];
const NUMBER_BOX_GAPS: &[&str] = &[
    "localized decimal formatting and expression evaluation",
    "accessibility spin-button and range-value provider",
    "button hover/pressed polish, press-and-hold autorepeat, mouse-wheel stepping and macOS modifier stepping",
    "AppKit and GTK4 target interaction smoke",
];
const AUTO_SUGGEST_GAPS: &[&str] = &[
    "accessibility search-field, expanded-state and active-descendant providers",
    "mouse-wheel paging for long suggestion lists",
    "AppKit and GTK4 target interaction smoke",
];
const TREE_GAPS: &[&str] = &[
    "accessibility tree level, expanded-state and selection providers",
    "multi-selection, drag-and-drop and large-tree virtualization",
    "AppKit and GTK4 target interaction smoke",
];
const TOGGLE_BUTTON_GAPS: &[&str] = &[
    "optional indeterminate state and grouped selection behavior",
    "accessibility toggle-button role and checked-state provider",
    "AppKit and GTK4 target interaction smoke",
];
const PASSWORD_BOX_GAPS: &[&str] = &[
    "Caps Lock warning and accessibility secure-text role/provider",
    "Windows Alt+F8 press-and-hold reveal shortcut",
    "platform memory-lock integration beyond owned-value zeroization",
    "AppKit and GTK4 target interaction smoke",
];
const RADIO_GAPS: &[&str] = &[
    "accessibility selection provider",
    "AppKit and GTK4 target interaction smoke",
];
const PROGRESS_GAPS: &[&str] = &[
    "indeterminate animation",
    "accessibility range-value provider",
    "AppKit and GTK4 target screenshot smoke",
];
const PROGRESS_RING_GAPS: &[&str] = &[
    "accessibility progress role and determinate value provider",
    "system reduced-motion preference",
    "AppKit and GTK4 target animation screenshot smoke",
];
const COMBO_GAPS: &[&str] = &[
    "accessibility expanded and selection providers",
    "AppKit and GTK4 target interaction smoke",
];
const DATE_PICKER_GAPS: &[&str] = &[
    "localized date, month and weekday formatting",
    "accessibility value and calendar-grid providers",
    "AppKit and GTK4 platform-style metrics and target interaction smoke",
];
const TIME_PICKER_GAPS: &[&str] = &[
    "system-locale clock selection and localized labels",
    "accessibility value and picker-column providers",
    "AppKit and GTK4 target interaction smoke",
];
const GRID_GAPS: &[&str] = &[
    "content-sized automatic tracks and baseline alignment",
    "accessibility grouping semantics",
    "AppKit and GTK4 target layout smoke",
];
const TABS_GAPS: &[&str] = &[
    "accessibility tab-list and tab-panel providers",
    "document-tab close, reorder and overflow behavior",
    "AppKit and GTK4 target interaction smoke",
];
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
        FirstPass,
        Some("grid"),
        "src/view.rs",
        GRID_GAPS
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
        FirstPass,
        Some("tabs"),
        "src/components.rs + src/view.rs + src/widget_render.rs",
        TABS_GAPS
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
        FirstPass,
        Some("toggle-button"),
        "src/view.rs + src/widget_render.rs + src/native_input_visuals.rs",
        TOGGLE_BUTTON_GAPS
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
        FirstPass,
        Some("password-box"),
        "src/password_box.rs + src/view.rs + src/native_input_visuals.rs",
        PASSWORD_BOX_GAPS
    ),
    component!(
        "combo_box",
        "ComboBox",
        Input,
        FirstPass,
        Some("combo"),
        "src/view.rs",
        COMBO_GAPS
    ),
    component!(
        "radio_button",
        "RadioButton",
        Input,
        FirstPass,
        Some("radio"),
        "src/view.rs",
        RADIO_GAPS
    ),
    component!(
        "slider",
        "Slider",
        Input,
        FirstPass,
        Some("slider"),
        "src/view.rs",
        SLIDER_GAPS
    ),
    component!(
        "number_box",
        "NumberBox",
        Input,
        FirstPass,
        Some("number-box"),
        "src/view.rs + src/widget_render.rs",
        NUMBER_BOX_GAPS
    ),
    component!(
        "auto_suggest",
        "AutoSuggestBox",
        Input,
        FirstPass,
        Some("auto-suggest"),
        "src/auto_suggest.rs + src/view.rs + src/widget_render.rs + three desktop input runtimes",
        AUTO_SUGGEST_GAPS
    ),
    component!(
        "date_picker",
        "DatePicker",
        Input,
        FirstPass,
        Some("date-picker"),
        "src/date.rs + src/view.rs + src/widget_render.rs",
        DATE_PICKER_GAPS
    ),
    component!(
        "time_picker",
        "TimePicker",
        Input,
        FirstPass,
        Some("time-picker"),
        "src/time.rs + src/view.rs + src/widget_render.rs",
        TIME_PICKER_GAPS
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
        FirstPass,
        Some("tree"),
        "src/tree.rs + src/view.rs + src/widget_render.rs + three desktop input runtimes",
        TREE_GAPS
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
        FirstPass,
        Some("progress"),
        "src/widget_render.rs",
        PROGRESS_GAPS
    ),
    component!(
        "progress_ring",
        "ProgressRing",
        Feedback,
        FirstPass,
        Some("progress-ring"),
        "src/progress.rs + src/view.rs + three desktop renderers",
        PROGRESS_RING_GAPS
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
        FirstPass,
        Some("tooltip"),
        "src/tooltip.rs + src/view.rs + src/native.rs",
        TOOLTIP_GAPS
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
        assert!(summary.not_started_count >= 7);
        assert!(!summary.missing_component_names.contains(&"tree"));
        assert!(!summary.missing_component_names.contains(&"progress_ring"));
        assert!(!summary.missing_component_names.contains(&"toggle"));
        assert!(!summary.missing_component_names.contains(&"toggle_button"));
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
