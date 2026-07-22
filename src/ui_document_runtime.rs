//! Release-safe compilation of validated UI documents into the shared View tree.
//!
//! This module contains no file watching, preview transport, native host or
//! extra-process contract. Applications opt into only the component features
//! used by their embedded document.

use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ui_document::{UiAxis, UiBindingSchema, UiDiagnostic, UiDocument, UiFeatureSet, UiNode};
#[cfg(feature = "label")]
use crate::ColorRole;
#[cfg(feature = "progress")]
use crate::ProgressRange;
#[cfg(feature = "slider")]
use crate::SliderRange;
#[cfg(feature = "number-box")]
use crate::ZsNumberRange;
use crate::{column, row, Dp, ThemeColorToken, ViewNode};
#[cfg(feature = "label")]
use crate::{SemanticTextStyle, TextRole};

/// Typed semantic action emitted by a document-backed View subtree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiDocumentAction {
    pub node_id: String,
    pub binding: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_binding: Option<String>,
    pub payload: Value,
}

#[cfg_attr(
    not(any(
        feature = "button",
        feature = "toggle-button",
        feature = "checkbox",
        feature = "toggle",
        feature = "textbox",
        feature = "radio",
        feature = "slider",
        feature = "number-box",
        feature = "combo",
        feature = "tabs",
        feature = "scroll"
    )),
    allow(dead_code)
)]
struct UiDocumentActionMapper<Msg> {
    mapper: UiDocumentActionMapperKind<Msg>,
}

enum UiDocumentActionMapperKind<Msg> {
    Function(fn(UiDocumentAction) -> Msg),
    Shared(Arc<dyn Fn(UiDocumentAction) -> Msg + Send + Sync + 'static>),
}

impl<Msg> Clone for UiDocumentActionMapper<Msg> {
    fn clone(&self) -> Self {
        Self {
            mapper: match &self.mapper {
                UiDocumentActionMapperKind::Function(mapper) => {
                    UiDocumentActionMapperKind::Function(*mapper)
                }
                UiDocumentActionMapperKind::Shared(mapper) => {
                    UiDocumentActionMapperKind::Shared(Arc::clone(mapper))
                }
            },
        }
    }
}

impl<Msg> UiDocumentActionMapper<Msg> {
    fn from_function(mapper: fn(UiDocumentAction) -> Msg) -> Self {
        Self {
            mapper: UiDocumentActionMapperKind::Function(mapper),
        }
    }

    fn from_shared(mapper: impl Fn(UiDocumentAction) -> Msg + Send + Sync + 'static) -> Self {
        Self {
            mapper: UiDocumentActionMapperKind::Shared(Arc::new(mapper)),
        }
    }

    #[cfg_attr(
        not(any(
            feature = "button",
            feature = "toggle-button",
            feature = "checkbox",
            feature = "toggle",
            feature = "textbox",
            feature = "radio",
            feature = "slider",
            feature = "number-box",
            feature = "combo",
            feature = "tabs",
            feature = "scroll"
        )),
        allow(dead_code)
    )]
    fn map(&self, action: UiDocumentAction) -> Msg {
        match &self.mapper {
            UiDocumentActionMapperKind::Function(mapper) => mapper(action),
            UiDocumentActionMapperKind::Shared(mapper) => mapper(action),
        }
    }
}

/// Compiles a document with a non-capturing typed action mapper.
pub fn ui_document_view<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    map_action: fn(UiDocumentAction) -> Msg,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    compile_validated_document(
        document,
        bindings,
        properties,
        UiDocumentActionMapper::from_function(map_action),
    )
}

/// Compiles a document with an application-owned typed action mapper.
///
/// The returned View contains only shared component nodes. Platform hosts still
/// apply Win32, AppKit or Linux experience profiles during layout and paint.
pub fn ui_document_view_with<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    map_action: impl Fn(UiDocumentAction) -> Msg + Send + Sync + 'static,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    compile_validated_document(
        document,
        bindings,
        properties,
        UiDocumentActionMapper::from_shared(map_action),
    )
}

fn compile_validated_document<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    map_action: UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let report = document.validate(&UiFeatureSet::compiled(), bindings);
    if !report.is_valid() {
        return Err(UiDocumentRuntimeError::Validation {
            diagnostics: report.diagnostics,
        });
    }
    compile_node(&document.root, properties, &map_action)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDocumentRuntimeError {
    Validation {
        diagnostics: Vec<UiDiagnostic>,
    },
    UnsupportedComponent {
        component: String,
    },
    InvalidChildCount {
        component: String,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for UiDocumentRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation { diagnostics } => write!(
                formatter,
                "UI document runtime validation failed with {} diagnostic(s)",
                diagnostics.len()
            ),
            Self::UnsupportedComponent { component } => write!(
                formatter,
                "UI document component {component:?} has no compiled View runtime"
            ),
            Self::InvalidChildCount {
                component,
                expected,
                actual,
            } => write!(
                formatter,
                "UI document component {component:?} requires {expected} child node(s), found {actual}"
            ),
        }
    }
}

impl Error for UiDocumentRuntimeError {}

fn compile_node<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let children = node
        .children
        .iter()
        .map(|child| compile_node(child, properties, mapper))
        .collect::<Result<Vec<_>, _>>()?;
    let mut view = match node.component.as_str() {
        "stack" => match node.layout.direction.unwrap_or(UiAxis::Vertical) {
            UiAxis::Horizontal => row(children),
            UiAxis::Vertical => column(children),
        },
        "border" => column(children),
        #[cfg(feature = "scroll")]
        "scroll" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(child) = children.next() else {
                return Err(UiDocumentRuntimeError::InvalidChildCount {
                    component: node.component.clone(),
                    expected: 1,
                    actual,
                });
            };
            if children.next().is_some() {
                return Err(UiDocumentRuntimeError::InvalidChildCount {
                    component: node.component.clone(),
                    expected: 1,
                    actual,
                });
            }
            let mut control = crate::scroll(child)
                .scroll_y(Dp::new(
                    number_property(node, properties, "offset_y", 0.0).max(0.0) as f32,
                ))
                .content_height(Dp::new(
                    number_property(node, properties, "content_height", 0.0).max(0.0) as f32,
                ));
            if let Some(binding) = node.action_bindings.get("scroll") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("offset_y").cloned();
                control = control.on_scroll_with(move |offset| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::from(offset.0),
                    })
                });
            }
            control
        }
        #[cfg(feature = "label")]
        "text" => {
            let value = string_property(node, properties, "text", "");
            crate::styled_text(value, semantic_text_style(node))
        }
        #[cfg(feature = "button")]
        "button" => {
            let mut control = crate::button(string_property(node, properties, "label", "Button"))
                .enabled(bool_property(node, properties, "enabled", true));
            if let Some(binding) = node.action_bindings.get("click") {
                control = control.on_click(mapper.map(UiDocumentAction {
                    node_id: node.id.as_str().to_owned(),
                    binding: binding.clone(),
                    property_binding: None,
                    payload: Value::Null,
                }));
            }
            control
        }
        #[cfg(feature = "toggle-button")]
        "toggle_button" => {
            let mut control = crate::toggle_button(
                string_property(node, properties, "label", "Toggle"),
                bool_property(node, properties, "checked", false),
            );
            if let Some(binding) = node.action_bindings.get("toggle") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("checked").cloned();
                control = control.on_toggle_with(move |checked| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(checked),
                    })
                });
            }
            control
        }
        #[cfg(feature = "checkbox")]
        "checkbox" => {
            let mut control = crate::checkbox(
                string_property(node, properties, "label", "Check box"),
                bool_property(node, properties, "checked", false),
            );
            if let Some(binding) = node.action_bindings.get("toggle") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("checked").cloned();
                control = control.on_toggle_with(move |checked| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(checked),
                    })
                });
            }
            control
        }
        #[cfg(feature = "toggle")]
        "toggle" => {
            let mut control = crate::toggle(bool_property(node, properties, "checked", false));
            if let Some(binding) = node.action_bindings.get("toggle") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("checked").cloned();
                control = control.on_toggle_with(move |checked| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(checked),
                    })
                });
            }
            control
        }
        #[cfg(feature = "textbox")]
        "textbox" if bool_property(node, properties, "multiline", false) => {
            let mut control = crate::text_editor(string_property(node, properties, "value", ""));
            if let Some(binding) = node.action_bindings.get("change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("value").cloned();
                control = control.on_change_with(move |value| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::String(value),
                    })
                });
            }
            control
        }
        #[cfg(feature = "textbox")]
        "textbox" => {
            let mut control = crate::textbox(string_property(node, properties, "value", ""));
            if let Some(binding) = node.action_bindings.get("change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("value").cloned();
                control = control.on_change_with(move |value| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::String(value),
                    })
                });
            }
            control
        }
        #[cfg(feature = "radio")]
        "radio_button" => {
            let mut control = crate::radio_button(
                string_property(node, properties, "label", "Option"),
                bool_property(node, properties, "selected", false),
            );
            if let Some(binding) = node.action_bindings.get("choose") {
                control = control.on_choose(mapper.map(UiDocumentAction {
                    node_id: node.id.as_str().to_owned(),
                    binding: binding.clone(),
                    property_binding: None,
                    payload: Value::Null,
                }));
            }
            control
        }
        #[cfg(feature = "slider")]
        "slider" => {
            let mut control = crate::slider(
                number_property(node, properties, "value", 0.0) as f32,
                SliderRange::new(0.0, 100.0),
            );
            if let Some(binding) = node.action_bindings.get("slide") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("value").cloned();
                control = control.on_slide_with(move |value| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::from(value),
                    })
                });
            }
            control
        }
        #[cfg(feature = "number-box")]
        "number_box" => {
            let minimum = number_property(node, properties, "minimum", 0.0);
            let maximum = number_property(node, properties, "maximum", 100.0);
            let range = ZsNumberRange::new(minimum, maximum)
                .step(number_property(
                    node,
                    properties,
                    "step",
                    (maximum - minimum).abs() / 100.0,
                ))
                .large_step(number_property(
                    node,
                    properties,
                    "large_step",
                    (maximum - minimum).abs() / 10.0,
                ));
            let value = nullable_number_property(node, properties, "value", Some(0.0));
            let mut control = crate::number_box(value, range)
                .fraction_digits(number_property(node, properties, "fraction_digits", 0.0) as u8)
                .wraps(bool_property(node, properties, "wraps", false));
            if let Some(binding) = node.action_bindings.get("change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("value").cloned();
                control = control.on_number_change_with(move |value| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: value.map_or(Value::Null, Value::from),
                    })
                });
            }
            control
        }
        #[cfg(feature = "combo")]
        "combo_box" => {
            let options = string_array_property(node, properties, "options");
            let selected_index = nullable_index_property(node, properties, "selected_index")
                .filter(|selected_index| *selected_index < options.len());
            let mut control = crate::combo_box(options, selected_index)
                .expanded(bool_property(node, properties, "expanded", false));
            if let Some(placeholder) = optional_string_property(node, properties, "placeholder") {
                control = control.placeholder(placeholder);
            }
            if let Some(binding) = node.action_bindings.get("select") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("selected_index").cloned();
                control = control.on_combo_select_with(move |selected_index| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::from(selected_index as u64),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("expanded_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("expanded").cloned();
                control = control.on_combo_expanded_change_with(move |expanded| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(expanded),
                    })
                });
            }
            control
        }
        #[cfg(feature = "tabs")]
        "tabs" => {
            let labels = string_map_property(node, properties, "labels");
            let icons = string_map_property(node, properties, "icons");
            let tab_ids = node
                .children
                .iter()
                .map(|child| (document_tab_id(child), child.id.as_str().to_owned()))
                .collect::<Vec<_>>();
            let items = node
                .children
                .iter()
                .zip(children)
                .zip(&tab_ids)
                .map(|((child, content), (tab_id, _))| {
                    let label = labels
                        .get(child.id.as_str())
                        .cloned()
                        .unwrap_or_else(|| child.id.as_str().to_owned());
                    let mut item = crate::ZsTabItem::new(*tab_id, label, content);
                    if let Some(icon) = icons.get(child.id.as_str()).and_then(|icon| {
                        serde_json::from_value::<crate::ZsIcon>(Value::String(icon.clone())).ok()
                    }) {
                        item = item.icon(icon);
                    }
                    item
                })
                .collect::<Vec<_>>();
            let selected =
                optional_string_property(node, properties, "selected").and_then(|selected| {
                    tab_ids
                        .iter()
                        .find(|(_, node_id)| *node_id == selected)
                        .map(|(tab_id, _)| *tab_id)
                });
            let mut control = crate::tab_view(items, selected);
            if let Some(binding) = node.action_bindings.get("select") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("selected").cloned();
                control = control.on_tab_select_with(move |selected| {
                    let selected = tab_ids
                        .iter()
                        .find(|(tab_id, _)| *tab_id == selected)
                        .map(|(_, node_id)| node_id.clone())
                        .expect("selected document tab must address compiled content");
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::String(selected),
                    })
                });
            }
            control
        }
        #[cfg(feature = "progress")]
        "progress_bar" => crate::progress_bar(
            number_property(node, properties, "value", 0.0) as f32,
            ProgressRange::new(0.0, 100.0),
        ),
        component => {
            return Err(UiDocumentRuntimeError::UnsupportedComponent {
                component: component.to_owned(),
            });
        }
    };
    view = view.id(node.id.widget_id());
    Ok(apply_layout(view, node))
}

fn apply_layout<Msg>(mut view: ViewNode<Msg>, node: &UiNode) -> ViewNode<Msg> {
    if let Some(value) = node.layout.width {
        view = view.width(Dp::new(value));
    }
    if let Some(value) = node.layout.height {
        view = view.height(Dp::new(value));
    }
    if let Some(value) = node.layout.min_width {
        view = view.min_width(Dp::new(value));
    }
    if let Some(value) = node.layout.min_height {
        view = view.min_height(Dp::new(value));
    }
    if let Some(value) = node.layout.padding {
        view = view.padding(Dp::new(value));
    }
    if let Some(value) = node.layout.gap {
        view = view.gap(Dp::new(value));
    }
    if let Some(value) = node.layout.flex {
        view = view.flex(value);
    }
    if let Some(token) = node
        .theme_tokens
        .get("background")
        .and_then(|token| theme_color_token(token))
    {
        view = view.bg(token);
    }
    view
}

#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "toggle-button",
    feature = "checkbox",
    feature = "toggle",
    feature = "textbox",
    feature = "radio",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "tabs",
    feature = "progress",
    feature = "scroll"
))]
fn property_value(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Option<Value> {
    if let Some(value) = node.properties.get(property) {
        return Some(value.clone());
    }
    if let Some(binding) = node.property_bindings.get(property) {
        return properties
            .get(binding)
            .cloned()
            .or_else(|| Some(Value::String(format!("{{binding:{binding}}}"))));
    }
    node.localization
        .get(property)
        .map(|key| Value::String(format!("{{message:{key}}}")))
}

#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "toggle-button",
    feature = "checkbox",
    feature = "textbox",
    feature = "radio"
))]
fn string_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    fallback: &str,
) -> String {
    property_value(node, properties, property)
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| fallback.to_owned())
}

#[cfg(any(
    feature = "button",
    feature = "toggle-button",
    feature = "checkbox",
    feature = "toggle",
    feature = "textbox",
    feature = "radio",
    feature = "number-box",
    feature = "combo"
))]
fn bool_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    fallback: bool,
) -> bool {
    property_value(node, properties, property)
        .and_then(|value| value.as_bool())
        .unwrap_or(fallback)
}

#[cfg(any(
    feature = "slider",
    feature = "number-box",
    feature = "progress",
    feature = "scroll"
))]
fn number_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    fallback: f64,
) -> f64 {
    property_value(node, properties, property)
        .and_then(|value| value.as_f64())
        .unwrap_or(fallback)
}

#[cfg(feature = "number-box")]
fn nullable_number_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    fallback: Option<f64>,
) -> Option<f64> {
    property_value(node, properties, property)
        .map(|value| value.as_f64())
        .unwrap_or(fallback)
}

#[cfg(any(feature = "combo", feature = "tabs"))]
fn optional_string_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Option<String> {
    property_value(node, properties, property).and_then(|value| value.as_str().map(str::to_owned))
}

#[cfg(feature = "tabs")]
fn string_map_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> BTreeMap<String, String> {
    property_value(node, properties, property)
        .and_then(|value| {
            value.as_object().map(|values| {
                values
                    .iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_owned()))
                    })
                    .collect()
            })
        })
        .unwrap_or_default()
}

#[cfg(feature = "tabs")]
fn document_tab_id(node: &UiNode) -> crate::ZsTabId {
    const DOCUMENT_ID_PAYLOAD_MASK: u64 = (1 << 62) - 1;
    crate::ZsTabId::new(node.id.widget_id().0 & DOCUMENT_ID_PAYLOAD_MASK)
}

#[cfg(feature = "combo")]
fn string_array_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Vec<String> {
    property_value(node, properties, property)
        .and_then(|value| {
            value.as_array().map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .collect()
            })
        })
        .unwrap_or_default()
}

#[cfg(feature = "combo")]
fn nullable_index_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Option<usize> {
    property_value(node, properties, property)
        .and_then(|value| value.as_u64())
        .and_then(|value| usize::try_from(value).ok())
}

#[cfg(feature = "label")]
fn semantic_text_style(node: &UiNode) -> SemanticTextStyle {
    let role = match node.properties.get("text_role").and_then(Value::as_str) {
        Some("caption") => TextRole::Caption,
        Some("body_large") => TextRole::BodyLarge,
        Some("subtitle") => TextRole::Subtitle,
        Some("title") => TextRole::Title,
        Some("title_large") => TextRole::TitleLarge,
        Some("display") => TextRole::Display,
        _ => TextRole::Body,
    };
    let mut style = SemanticTextStyle::for_role(role);
    if let Some(color) = node
        .theme_tokens
        .get("foreground")
        .and_then(|token| color_role(token))
    {
        style.color = color;
    }
    style
}

fn theme_color_token(token: &str) -> Option<ThemeColorToken> {
    match token {
        "surface" => Some(ThemeColorToken::Surface),
        "surface.raised" => Some(ThemeColorToken::SurfaceRaised),
        "text.primary" => Some(ThemeColorToken::TextPrimary),
        "text.secondary" => Some(ThemeColorToken::TextSecondary),
        "accent" => Some(ThemeColorToken::Accent),
        "control" => Some(ThemeColorToken::Control),
        "border" => Some(ThemeColorToken::Border),
        "accent.text" => Some(ThemeColorToken::AccentText),
        "success" => Some(ThemeColorToken::Success),
        "warning" => Some(ThemeColorToken::Warning),
        "danger" => Some(ThemeColorToken::Danger),
        _ => None,
    }
}

#[cfg(feature = "label")]
fn color_role(token: &str) -> Option<ColorRole> {
    match token {
        "surface" => Some(ColorRole::Surface),
        "surface.raised" => Some(ColorRole::SurfaceRaised),
        "text.primary" => Some(ColorRole::PrimaryText),
        "text.secondary" => Some(ColorRole::SecondaryText),
        "accent" => Some(ColorRole::Accent),
        "control" => Some(ColorRole::Control),
        "border" => Some(ColorRole::Border),
        "accent.text" => Some(ColorRole::AccentText),
        "success" => Some(ColorRole::Success),
        "warning" => Some(ColorRole::Warning),
        "danger" => Some(ColorRole::Danger),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(feature = "label", feature = "button"))]
    use crate::View;

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Action(UiDocumentAction),
    }

    #[test]
    fn compiles_feature_pruned_stack_without_viewer_or_widget_features() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": { "id": "root", "component": "stack" }
            }"#,
        )
        .unwrap();

        let view = ui_document_view(
            &document,
            &UiBindingSchema::default(),
            &BTreeMap::new(),
            Msg::Action,
        )
        .unwrap();

        assert_eq!(view.id, Some(document.root.id.widget_id()));
        assert!(view.children.is_empty());
    }

    #[cfg(all(feature = "label", feature = "button"))]
    #[test]
    fn compiles_typed_button_action_without_viewer_runtime() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save",
                "component": "button",
                "properties": { "label": "Save" },
                "action_bindings": { "click": "save" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::new(),
            actions: BTreeMap::from([("save".to_owned(), crate::ui_document::UiValueType::Null)]),
        };
        let features = UiFeatureSet::compiled();
        let artifact =
            crate::ui_document::UiDocumentReleaseArtifact::compile(&document, &features, &bindings)
                .unwrap();
        let embedded = crate::ui_document::UiEmbeddedDocument::decode(
            artifact.as_bytes(),
            &features,
            &bindings,
        )
        .unwrap();

        let mut view = ui_document_view(
            &embedded.document,
            &embedded.bindings,
            &BTreeMap::new(),
            Msg::Action,
        )
        .unwrap();
        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::Click {
                widget: document.root.id.widget_id(),
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![Msg::Action(UiDocumentAction {
                node_id: "save".to_owned(),
                binding: "save".to_owned(),
                property_binding: None,
                payload: Value::Null,
            })]
        );
    }
}
