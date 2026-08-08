use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{Rect, WidgetId};

/// Platform-neutral semantic role lowered by native accessibility providers.
///
/// The role set intentionally follows common desktop semantics instead of
/// exposing UIA, AppKit or AT-SPI constants to application code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsAccessibilityRole {
    Application,
    Article,
    Button,
    Canvas,
    ColorWell,
    ComboBox,
    Complementary,
    DatePicker,
    Dialog,
    Form,
    Grid,
    Group,
    Heading,
    Image,
    List,
    ListItem,
    Log,
    Main,
    Navigation,
    ProgressBar,
    Region,
    Slider,
    SpinButton,
    Status,
    Tab,
    TabList,
    TabPanel,
    Text,
    TextBox,
    TimePicker,
    Tree,
}

impl ZsAccessibilityRole {
    pub const NAMES: &'static [&'static str] = &[
        "application",
        "article",
        "button",
        "canvas",
        "color_well",
        "combo_box",
        "complementary",
        "date_picker",
        "dialog",
        "form",
        "grid",
        "group",
        "heading",
        "image",
        "list",
        "list_item",
        "log",
        "main",
        "navigation",
        "progress_bar",
        "region",
        "slider",
        "spin_button",
        "status",
        "tab",
        "tab_list",
        "tab_panel",
        "text",
        "text_box",
        "time_picker",
        "tree",
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Article => "article",
            Self::Button => "button",
            Self::Canvas => "canvas",
            Self::ColorWell => "color_well",
            Self::ComboBox => "combo_box",
            Self::Complementary => "complementary",
            Self::DatePicker => "date_picker",
            Self::Dialog => "dialog",
            Self::Form => "form",
            Self::Grid => "grid",
            Self::Group => "group",
            Self::Heading => "heading",
            Self::Image => "image",
            Self::List => "list",
            Self::ListItem => "list_item",
            Self::Log => "log",
            Self::Main => "main",
            Self::Navigation => "navigation",
            Self::ProgressBar => "progress_bar",
            Self::Region => "region",
            Self::Slider => "slider",
            Self::SpinButton => "spin_button",
            Self::Status => "status",
            Self::Tab => "tab",
            Self::TabList => "tab_list",
            Self::TabPanel => "tab_panel",
            Self::Text => "text",
            Self::TextBox => "text_box",
            Self::TimePicker => "time_picker",
            Self::Tree => "tree",
        }
    }
}

impl fmt::Display for ZsAccessibilityRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseZsAccessibilityRoleError {
    value: String,
}

impl fmt::Display for ParseZsAccessibilityRoleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown accessibility role {:?}; expected one of {}",
            self.value,
            ZsAccessibilityRole::NAMES.join(", ")
        )
    }
}

impl std::error::Error for ParseZsAccessibilityRoleError {}

impl FromStr for ZsAccessibilityRole {
    type Err = ParseZsAccessibilityRoleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let role = match value.trim() {
            "application" => Self::Application,
            "article" => Self::Article,
            "button" => Self::Button,
            "canvas" => Self::Canvas,
            "color_well" | "colorwell" => Self::ColorWell,
            "combo_box" | "combobox" => Self::ComboBox,
            "complementary" => Self::Complementary,
            "date_picker" | "datepicker" => Self::DatePicker,
            "dialog" => Self::Dialog,
            "form" => Self::Form,
            "grid" => Self::Grid,
            "group" => Self::Group,
            "heading" => Self::Heading,
            "image" => Self::Image,
            "list" => Self::List,
            "list_item" | "listitem" => Self::ListItem,
            "log" => Self::Log,
            "main" => Self::Main,
            "navigation" => Self::Navigation,
            "progress_bar" | "progressbar" => Self::ProgressBar,
            "region" => Self::Region,
            "slider" => Self::Slider,
            "spin_button" | "spinbutton" => Self::SpinButton,
            "status" => Self::Status,
            "tab" => Self::Tab,
            "tab_list" | "tablist" => Self::TabList,
            "tab_panel" | "tabpanel" => Self::TabPanel,
            "text" => Self::Text,
            "text_box" | "textbox" => Self::TextBox,
            "time_picker" | "timepicker" => Self::TimePicker,
            "tree" => Self::Tree,
            _ => {
                return Err(ParseZsAccessibilityRoleError {
                    value: value.to_owned(),
                });
            }
        };
        Ok(role)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsAccessibilityLiveRegion {
    Polite,
    Assertive,
}

/// Interaction policy for a semantic numeric range.
///
/// Progress indicators remain read-only, while controls such as Slider expose
/// finite small and large changes to the native accessibility backend.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ZsAccessibilityRangeInteraction {
    #[default]
    ReadOnly,
    Adjustable {
        small_change: f64,
        large_change: f64,
    },
}

impl ZsAccessibilityRangeInteraction {
    pub const fn is_read_only(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    pub const fn small_change(self) -> Option<f64> {
        match self {
            Self::ReadOnly => None,
            Self::Adjustable { small_change, .. } => Some(small_change),
        }
    }

    pub const fn large_change(self) -> Option<f64> {
        match self {
            Self::ReadOnly => None,
            Self::Adjustable { large_change, .. } => Some(large_change),
        }
    }
}

/// Numeric range exposed by progress indicators and adjustable controls.
///
/// Construction normalizes non-finite and reversed inputs so every backend
/// receives the same finite `minimum <= value <= maximum` contract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsAccessibilityRangeValue {
    pub value: f64,
    pub minimum: f64,
    pub maximum: f64,
    #[serde(
        default,
        skip_serializing_if = "ZsAccessibilityRangeInteraction::is_read_only"
    )]
    pub interaction: ZsAccessibilityRangeInteraction,
}

/// Framework-owned action route for a semantic child of a composite control.
///
/// Applications describe the composite control once; ZSUI emits these routes
/// only for the platform accessibility backends that need to address an
/// internal interactive surface independently from its owner widget.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsAccessibilityActionTarget {
    ContentDialogPrimary { dialog: WidgetId },
    ContentDialogSecondary { dialog: WidgetId },
    ContentDialogClose { dialog: WidgetId },
}

impl ZsAccessibilityRangeValue {
    pub fn new(value: f64, minimum: f64, maximum: f64) -> Self {
        let minimum = if minimum.is_finite() { minimum } else { 0.0 };
        let maximum = if maximum.is_finite() { maximum } else { 100.0 };
        let (minimum, mut maximum) = if minimum <= maximum {
            (minimum, maximum)
        } else {
            (maximum, minimum)
        };
        if (maximum - minimum).abs() <= f64::EPSILON {
            maximum = minimum + 1.0;
        }
        let value = if value.is_finite() {
            value.clamp(minimum, maximum)
        } else {
            minimum
        };
        Self {
            value,
            minimum,
            maximum,
            interaction: ZsAccessibilityRangeInteraction::ReadOnly,
        }
    }

    /// Marks this range as adjustable and normalizes both changes into the
    /// finite positive range span shared by every platform backend.
    pub fn adjustable(mut self, small_change: f64, large_change: f64) -> Self {
        let span = self.maximum - self.minimum;
        let fallback_small = (span / 100.0).max(f64::EPSILON);
        let small_change = normalize_range_change(small_change, fallback_small, span);
        let fallback_large = (small_change * 10.0).min(span);
        let large_change = normalize_range_change(large_change, fallback_large, span);
        self.interaction = ZsAccessibilityRangeInteraction::Adjustable {
            small_change,
            large_change,
        };
        self
    }
}

fn normalize_range_change(value: f64, fallback: f64, span: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value.min(span)
    } else {
        fallback.min(span)
    }
}

/// Semantic metadata attached to one retained View node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsAccessibilitySpec {
    pub role: ZsAccessibilityRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_region: Option<ZsAccessibilityLiveRegion>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    /// Persistent on/off state for button-like controls that implement a
    /// native toggle protocol. This is distinct from collection selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_value: Option<ZsAccessibilityRangeValue>,
}

impl ZsAccessibilitySpec {
    pub const fn new(role: ZsAccessibilityRole) -> Self {
        Self {
            role,
            label: None,
            description: None,
            live_region: None,
            enabled: true,
            selected: None,
            checked: None,
            range_value: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = non_empty(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = non_empty(description.into());
        self
    }

    pub const fn live_region(mut self, live_region: ZsAccessibilityLiveRegion) -> Self {
        self.live_region = Some(live_region);
        self
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Exposes a persistent binary state without changing the semantic role.
    /// Native providers lower this to UIA Toggle, AppKit button value and
    /// AccessKit toggled state respectively.
    pub const fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub const fn range_value(mut self, range_value: ZsAccessibilityRangeValue) -> Self {
        self.range_value = Some(range_value);
        self
    }
}

/// Laid-out semantic node consumed by UIA, AppKit Accessibility and AccessKit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsAccessibilityNode {
    pub widget: WidgetId,
    pub parent: Option<WidgetId>,
    pub bounds: Rect,
    pub role: ZsAccessibilityRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_region: Option<ZsAccessibilityLiveRegion>,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_value: Option<ZsAccessibilityRangeValue>,
    #[doc(hidden)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_target: Option<ZsAccessibilityActionTarget>,
}

impl ZsAccessibilityNode {
    #[cfg(feature = "accessibility")]
    pub(crate) fn from_spec(
        widget: WidgetId,
        parent: Option<WidgetId>,
        bounds: Rect,
        spec: &ZsAccessibilitySpec,
    ) -> Self {
        Self {
            widget,
            parent,
            bounds,
            role: spec.role,
            label: spec.label.clone(),
            description: spec.description.clone(),
            live_region: spec.live_region,
            enabled: spec.enabled,
            selected: spec.selected,
            checked: spec.checked,
            range_value: spec.range_value,
            action_target: None,
        }
    }
}

const fn default_true() -> bool {
    true
}

const fn is_true(value: &bool) -> bool {
    *value
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_round_trip_through_stable_schema_names() {
        for name in ZsAccessibilityRole::NAMES {
            let role = name.parse::<ZsAccessibilityRole>().unwrap();
            assert_eq!(role.as_str(), *name);
            assert_eq!(serde_json::to_string(&role).unwrap(), format!("\"{name}\""));
        }
        assert_eq!(
            "textbox".parse::<ZsAccessibilityRole>().unwrap(),
            ZsAccessibilityRole::TextBox
        );
        assert!("NSImage".parse::<ZsAccessibilityRole>().is_err());
    }

    #[test]
    fn empty_optional_text_is_not_retained() {
        let spec = ZsAccessibilitySpec::new(ZsAccessibilityRole::Image)
            .label("  ")
            .description("  Preview image  ");
        assert_eq!(spec.label, None);
        assert_eq!(spec.description.as_deref(), Some("Preview image"));
    }

    #[test]
    fn range_values_are_finite_ordered_and_clamped() {
        assert_eq!(
            ZsAccessibilityRangeValue::new(125.0, 100.0, 0.0),
            ZsAccessibilityRangeValue {
                value: 100.0,
                minimum: 0.0,
                maximum: 100.0,
                interaction: ZsAccessibilityRangeInteraction::ReadOnly,
            }
        );
        assert_eq!(
            ZsAccessibilityRangeValue::new(f64::NAN, f64::NAN, f64::INFINITY),
            ZsAccessibilityRangeValue {
                value: 0.0,
                minimum: 0.0,
                maximum: 100.0,
                interaction: ZsAccessibilityRangeInteraction::ReadOnly,
            }
        );
    }

    #[test]
    fn adjustable_range_changes_are_finite_positive_and_bounded() {
        let range = ZsAccessibilityRangeValue::new(25.0, 0.0, 100.0).adjustable(5.0, f64::INFINITY);
        assert!(!range.interaction.is_read_only());
        assert_eq!(range.interaction.small_change(), Some(5.0));
        assert_eq!(range.interaction.large_change(), Some(50.0));
    }

    #[test]
    fn checked_state_is_independent_from_collection_selection() {
        let spec = ZsAccessibilitySpec::new(ZsAccessibilityRole::Button).checked(true);
        assert_eq!(spec.checked, Some(true));
        assert_eq!(spec.selected, None);
        assert!(serde_json::to_string(&spec)
            .unwrap()
            .contains("\"checked\":true"));
    }
}
