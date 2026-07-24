//! Release-safe compilation of validated UI documents into the shared View tree.
//!
//! This module contains no file watching, preview transport, native host or
//! extra-process contract. Applications opt into only the component features
//! used by their embedded document.

use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

#[cfg(any(feature = "tree", feature = "document-shell"))]
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "password-box")]
use crate::ui_document::UiSecretValues;
#[cfg(feature = "menu-flyout")]
use crate::ui_document::{ui_menu_flyout_items_from_value, UiMenuFlyoutItem};
use crate::ui_document::{
    validate_ui_document_binding_values, UiAxis, UiBindingSchema, UiDiagnostic, UiDocument,
    UiFeatureSet, UiNode,
};
#[cfg(feature = "grid")]
use crate::ui_document::{UiGridPlacement, UiGridTrack};
#[cfg(any(feature = "icon", feature = "label"))]
use crate::ColorRole;
#[cfg(any(feature = "progress", feature = "progress-ring"))]
use crate::ProgressRange;
#[cfg(feature = "slider")]
use crate::SliderRange;
#[cfg(feature = "number-box")]
use crate::ZsNumberRange;
use crate::{column, row, Dp, ThemeColorToken, ViewNode};
#[cfg(feature = "label")]
use crate::{SemanticTextStyle, TextRole};
#[cfg(feature = "progress-ring")]
use crate::{ZsProgressRingSize, ZsProgressRingSpec};

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

/// Secure typed action emitted by a document-backed PasswordBox.
///
/// The payload intentionally cannot be serialized. Its owned allocation is
/// cleared on drop and `Debug` remains redacted through [`ZsPassword`].
#[cfg(feature = "password-box")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDocumentSecretAction {
    pub node_id: String,
    pub binding: String,
    pub property_binding: String,
    pub value: crate::ZsPassword,
}

#[cfg_attr(
    not(any(
        feature = "button",
        feature = "breadcrumb",
        feature = "split-view",
        feature = "canvas",
        feature = "flyout",
        feature = "menu-flyout",
        feature = "toggle-button",
        feature = "checkbox",
        feature = "toggle",
        feature = "textbox",
        feature = "radio",
        feature = "slider",
        feature = "number-box",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker",
        feature = "color-picker",
        feature = "auto-suggest",
        feature = "command-palette",
        feature = "tree",
        feature = "grid-view",
        feature = "table",
        feature = "shell",
        feature = "list",
        feature = "tabs",
        feature = "dialog",
        feature = "toast",
        feature = "info-bar",
        feature = "teaching-tip",
        feature = "scroll"
    )),
    allow(dead_code)
)]
struct UiDocumentActionMapper<Msg> {
    mapper: UiDocumentActionMapperKind<Msg>,
}

#[cfg(feature = "password-box")]
struct UiDocumentSecretActionMapper<Msg> {
    mapper: UiDocumentSecretActionMapperKind<Msg>,
}

#[cfg(feature = "password-box")]
enum UiDocumentSecretActionMapperKind<Msg> {
    Function(fn(UiDocumentSecretAction) -> Msg),
    Shared(Arc<dyn Fn(UiDocumentSecretAction) -> Msg + Send + Sync + 'static>),
}

#[cfg(feature = "password-box")]
impl<Msg> Clone for UiDocumentSecretActionMapper<Msg> {
    fn clone(&self) -> Self {
        Self {
            mapper: match &self.mapper {
                UiDocumentSecretActionMapperKind::Function(mapper) => {
                    UiDocumentSecretActionMapperKind::Function(*mapper)
                }
                UiDocumentSecretActionMapperKind::Shared(mapper) => {
                    UiDocumentSecretActionMapperKind::Shared(Arc::clone(mapper))
                }
            },
        }
    }
}

#[cfg(feature = "password-box")]
impl<Msg> UiDocumentSecretActionMapper<Msg> {
    fn from_function(mapper: fn(UiDocumentSecretAction) -> Msg) -> Self {
        Self {
            mapper: UiDocumentSecretActionMapperKind::Function(mapper),
        }
    }

    fn from_shared(mapper: impl Fn(UiDocumentSecretAction) -> Msg + Send + Sync + 'static) -> Self {
        Self {
            mapper: UiDocumentSecretActionMapperKind::Shared(Arc::new(mapper)),
        }
    }

    fn map(&self, action: UiDocumentSecretAction) -> Msg {
        match &self.mapper {
            UiDocumentSecretActionMapperKind::Function(mapper) => mapper(action),
            UiDocumentSecretActionMapperKind::Shared(mapper) => mapper(action),
        }
    }
}

#[cfg(feature = "password-box")]
struct UiDocumentSecureContext<'a, Msg> {
    values: &'a UiSecretValues,
    mapper: UiDocumentSecretActionMapper<Msg>,
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
            feature = "breadcrumb",
            feature = "flyout",
            feature = "menu-flyout",
            feature = "toggle-button",
            feature = "checkbox",
            feature = "toggle",
            feature = "textbox",
            feature = "radio",
            feature = "slider",
            feature = "number-box",
            feature = "combo",
            feature = "date-picker",
            feature = "time-picker",
            feature = "color-picker",
            feature = "auto-suggest",
            feature = "list",
            feature = "tabs",
            feature = "dialog",
            feature = "toast",
            feature = "info-bar",
            feature = "teaching-tip",
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

/// Compiles a document with isolated ordinary and password action channels.
///
/// Password values never enter `properties`, Serde JSON or
/// [`UiDocumentAction`]. Missing secure state resolves to an empty PasswordBox
/// value so authoring previews never need a plaintext fixture.
#[cfg(feature = "password-box")]
pub fn ui_document_view_with_secrets<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    secrets: &UiSecretValues,
    map_action: fn(UiDocumentAction) -> Msg,
    map_secret_action: fn(UiDocumentSecretAction) -> Msg,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    compile_validated_document_secure(
        document,
        bindings,
        properties,
        secrets,
        UiDocumentActionMapper::from_function(map_action),
        UiDocumentSecretActionMapper::from_function(map_secret_action),
    )
}

/// Capturing-callback variant of [`ui_document_view_with_secrets`].
#[cfg(feature = "password-box")]
pub fn ui_document_view_with_secrets_and<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    secrets: &UiSecretValues,
    map_action: impl Fn(UiDocumentAction) -> Msg + Send + Sync + 'static,
    map_secret_action: impl Fn(UiDocumentSecretAction) -> Msg + Send + Sync + 'static,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    compile_validated_document_secure(
        document,
        bindings,
        properties,
        secrets,
        UiDocumentActionMapper::from_shared(map_action),
        UiDocumentSecretActionMapper::from_shared(map_secret_action),
    )
}

fn compile_validated_document<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    map_action: UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let mut diagnostics = document
        .validate(&UiFeatureSet::compiled(), bindings)
        .diagnostics;
    diagnostics
        .extend(validate_ui_document_binding_values(document, bindings, properties).diagnostics);
    if !diagnostics.is_empty() {
        return Err(UiDocumentRuntimeError::Validation { diagnostics });
    }
    compile_node(
        &document.root,
        properties,
        &map_action,
        #[cfg(feature = "password-box")]
        None,
    )
}

#[cfg(feature = "password-box")]
fn compile_validated_document_secure<Msg: Clone + 'static>(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    properties: &BTreeMap<String, Value>,
    secrets: &UiSecretValues,
    map_action: UiDocumentActionMapper<Msg>,
    map_secret_action: UiDocumentSecretActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let mut diagnostics = document
        .validate(&UiFeatureSet::compiled(), bindings)
        .diagnostics;
    diagnostics
        .extend(validate_ui_document_binding_values(document, bindings, properties).diagnostics);
    if !diagnostics.is_empty() {
        return Err(UiDocumentRuntimeError::Validation { diagnostics });
    }
    let secure = UiDocumentSecureContext {
        values: secrets,
        mapper: map_secret_action,
    };
    compile_node(&document.root, properties, &map_action, Some(&secure))
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
    InvalidResolvedProperty {
        node_id: String,
        property: String,
        reason: String,
    },
    #[cfg(feature = "password-box")]
    SecureChannelRequired {
        node_id: String,
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
            Self::InvalidResolvedProperty {
                node_id,
                property,
                reason,
            } => write!(
                formatter,
                "UI document node {node_id:?} resolved invalid property {property:?}: {reason}"
            ),
            #[cfg(feature = "password-box")]
            Self::SecureChannelRequired { node_id } => write!(
                formatter,
                "UI document PasswordBox {node_id:?} requires ui_document_view_with_secrets"
            ),
        }
    }
}

impl Error for UiDocumentRuntimeError {}

fn compile_node<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
    #[cfg(feature = "password-box")] secure: Option<&UiDocumentSecureContext<'_, Msg>>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let children = node
        .children
        .iter()
        .map(|child| {
            compile_node(
                child,
                properties,
                mapper,
                #[cfg(feature = "password-box")]
                secure,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut view = match node.component.as_str() {
        "stack" => match node.layout.direction.unwrap_or(UiAxis::Vertical) {
            UiAxis::Horizontal => row(children),
            UiAxis::Vertical => column(children),
        },
        "border" => column(children).flex(0.0),
        #[cfg(feature = "badge")]
        "badge" => document_badge(node, properties)?,
        #[cfg(feature = "split-view")]
        "split_view" => document_split_view(node, properties, children, mapper)?,
        #[cfg(feature = "canvas")]
        "canvas" => document_canvas(node, properties, mapper)?,
        #[cfg(feature = "grid")]
        "grid" => document_grid(node, properties, children)?,
        #[cfg(feature = "document-shell")]
        "command_bar" => document_command_bar(node, properties, children)?,
        #[cfg(feature = "shell")]
        "navigation" => document_navigation(node, properties, children, mapper)?,
        #[cfg(feature = "list")]
        "list" => document_list(node, properties, children, mapper)?,
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
        #[cfg(feature = "menu-flyout")]
        "menu_flyout" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(page) = children.next() else {
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

            let target = string_property(node, properties, "target", "");
            let target = menu_flyout_document_target(node, &target).ok_or_else(|| {
                invalid_resolved_property(
                    node,
                    "target",
                    "menu_flyout target must reference a node in its page child",
                )
            })?;
            let menu = document_menu_flyout(node, properties)?;
            let mut control = crate::menu_flyout(
                node.id.widget_id(),
                bool_property(node, properties, "open", false),
                target,
                menu,
                page,
            );
            if let Some(binding) = node.action_bindings.get("invoke") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_menu_flyout_command_with(move |command| {
                    let crate::Command::Custom { id, payload: None } = command else {
                        unreachable!("document MenuFlyout emits only stable custom item IDs");
                    };
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(id),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("open_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("open").cloned();
                control = control.on_menu_flyout_open_change_with(move |open| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(open),
                    })
                });
            }
            control
        }
        #[cfg(feature = "flyout")]
        "flyout" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(page) = children.next() else {
                return Err(UiDocumentRuntimeError::InvalidChildCount {
                    component: node.component.clone(),
                    expected: 2,
                    actual,
                });
            };
            let Some(content) = children.next() else {
                return Err(UiDocumentRuntimeError::InvalidChildCount {
                    component: node.component.clone(),
                    expected: 2,
                    actual,
                });
            };
            if children.next().is_some() {
                return Err(UiDocumentRuntimeError::InvalidChildCount {
                    component: node.component.clone(),
                    expected: 2,
                    actual,
                });
            }

            let content_width = document_positive_extent(node, properties, "content_width")?;
            let content_height = document_positive_extent(node, properties, "content_height")?;
            let placement = match optional_string_property(node, properties, "placement").as_deref()
            {
                None | Some("auto") => crate::ZsFlyoutPlacement::Auto,
                Some("top") => crate::ZsFlyoutPlacement::Top,
                Some("bottom") => crate::ZsFlyoutPlacement::Bottom,
                Some("left") => crate::ZsFlyoutPlacement::Left,
                Some("right") => crate::ZsFlyoutPlacement::Right,
                Some(placement) => {
                    return Err(invalid_resolved_property(
                        node,
                        "placement",
                        format!("unsupported flyout placement {placement:?}"),
                    ));
                }
            };
            let target = string_property(node, properties, "target", "");
            let target = flyout_document_target(node, &target).ok_or_else(|| {
                invalid_resolved_property(
                    node,
                    "target",
                    "flyout target must reference a node in its page child",
                )
            })?;
            let spec = crate::ZsFlyoutSpec::new(content_width, content_height)
                .preferred_placement(placement);
            let mut control = crate::flyout(
                node.id.widget_id(),
                bool_property(node, properties, "open", false),
                target,
                spec,
                content,
                page,
            );
            if let Some(binding) = node.action_bindings.get("dismiss") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_flyout_dismiss_with(move |reason| {
                    let payload = match reason {
                        crate::ZsFlyoutDismissReason::LightDismiss => "light_dismiss",
                        crate::ZsFlyoutDismissReason::EscapeKey => "escape",
                    };
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(payload.to_owned()),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("open_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("open").cloned();
                control = control.on_flyout_open_change_with(move |open| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(open),
                    })
                });
            }
            control
        }
        #[cfg(feature = "tooltip")]
        "tooltip" => {
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

            let text = string_property(node, properties, "text", "");
            if text.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "text",
                    "tooltip text must not be empty",
                ));
            }
            let placement = match optional_string_property(node, properties, "placement").as_deref()
            {
                None | Some("auto") => crate::ZsTooltipPlacement::Auto,
                Some("top") => crate::ZsTooltipPlacement::Top,
                Some("bottom") => crate::ZsTooltipPlacement::Bottom,
                Some("left") => crate::ZsTooltipPlacement::Left,
                Some("right") => crate::ZsTooltipPlacement::Right,
                Some(placement) => {
                    return Err(invalid_resolved_property(
                        node,
                        "placement",
                        format!("unsupported tooltip placement {placement:?}"),
                    ));
                }
            };
            let mut spec = crate::ZsTooltipSpec::new(text).placement(placement);
            if let Some(open_delay_ms) =
                property_value(node, properties, "open_delay_ms").and_then(|value| value.as_u64())
            {
                spec = spec.open_delay_ms(open_delay_ms);
            }
            child.tooltip_spec(spec)
        }
        #[cfg(feature = "teaching-tip")]
        "teaching_tip" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(page) = children.next() else {
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

            let title = string_property(node, properties, "title", "");
            let subtitle = string_property(node, properties, "subtitle", "");
            for (property, value) in [("title", &title), ("subtitle", &subtitle)] {
                if node_has_property_source(node, property) && value.trim().is_empty() {
                    return Err(invalid_resolved_property(
                        node,
                        property,
                        "teaching tip text must not be empty when provided",
                    ));
                }
            }
            if title.trim().is_empty() && subtitle.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "title",
                    "teaching tip requires a title, subtitle or both",
                ));
            }
            let action_label = optional_string_property(node, properties, "action_label");
            if action_label
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(invalid_resolved_property(
                    node,
                    "action_label",
                    "teaching tip action label must not be empty when provided",
                ));
            }
            let placement = match optional_string_property(node, properties, "placement").as_deref()
            {
                None | Some("auto") => crate::ZsTeachingTipPlacement::Auto,
                Some("top") => crate::ZsTeachingTipPlacement::Top,
                Some("bottom") => crate::ZsTeachingTipPlacement::Bottom,
                Some("left") => crate::ZsTeachingTipPlacement::Left,
                Some("right") => crate::ZsTeachingTipPlacement::Right,
                Some(placement) => {
                    return Err(invalid_resolved_property(
                        node,
                        "placement",
                        format!("unsupported teaching tip placement {placement:?}"),
                    ));
                }
            };
            let target = string_property(node, properties, "target", "");
            let target = descendant_document_widget_id(node, &target).ok_or_else(|| {
                invalid_resolved_property(
                    node,
                    "target",
                    "teaching tip target must reference a node in its page subtree",
                )
            })?;

            let mut spec =
                crate::ZsTeachingTipSpec::new(title, subtitle).preferred_placement(placement);
            if let Some(action_label) = action_label {
                spec = spec.action(action_label);
            }
            let mut control = crate::teaching_tip(
                node.id.widget_id(),
                bool_property(node, properties, "open", false),
                target,
                spec,
                page,
            );
            if let Some(binding) = node.action_bindings.get("result") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_teaching_tip_result_with(move |result| {
                    let payload = match result.response {
                        crate::ZsTeachingTipResponse::Action => "action",
                        crate::ZsTeachingTipResponse::Dismissed(
                            crate::ZsTeachingTipDismissReason::CloseButton,
                        ) => "close",
                        crate::ZsTeachingTipResponse::Dismissed(
                            crate::ZsTeachingTipDismissReason::EscapeKey,
                        ) => "escape",
                    };
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(payload.to_owned()),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("open_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("open").cloned();
                control = control.on_teaching_tip_open_change_with(move |open| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(open),
                    })
                });
            }
            control
        }
        #[cfg(feature = "info-bar")]
        "info_bar" => {
            let message = string_property(node, properties, "message", "");
            if message.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "message",
                    "info bar message must not be empty",
                ));
            }
            let title = optional_string_property(node, properties, "title");
            if title
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(invalid_resolved_property(
                    node,
                    "title",
                    "info bar title must not be empty when provided",
                ));
            }
            let action_label = optional_string_property(node, properties, "action_label");
            if action_label
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(invalid_resolved_property(
                    node,
                    "action_label",
                    "info bar action label must not be empty when provided",
                ));
            }
            let severity = match optional_string_property(node, properties, "severity").as_deref() {
                Some("success") => crate::ZsInfoBarSeverity::Success,
                Some("warning") => crate::ZsInfoBarSeverity::Warning,
                Some("error") => crate::ZsInfoBarSeverity::Error,
                Some("informational") | None => crate::ZsInfoBarSeverity::Informational,
                Some(severity) => {
                    return Err(invalid_resolved_property(
                        node,
                        "severity",
                        format!("unsupported info bar severity {severity:?}"),
                    ));
                }
            };
            let mut spec = crate::ZsInfoBarSpec::new(message)
                .severity(severity)
                .closable(bool_property(node, properties, "closable", true));
            if let Some(title) = title {
                spec = spec.title(title);
            }
            if let Some(action_label) = action_label {
                spec = spec.action(action_label);
            }
            let mut control = crate::info_bar(node.id.widget_id(), spec);
            if let Some(binding) = node.action_bindings.get("event") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_info_bar_event_with(move |event| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(
                            match event {
                                crate::ZsInfoBarEvent::Action => "action",
                                crate::ZsInfoBarEvent::Close => "close",
                            }
                            .to_owned(),
                        ),
                    })
                });
            }
            control
        }
        #[cfg(feature = "toast")]
        "toast" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(page) = children.next() else {
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

            let message = string_property(node, properties, "message", "");
            if message.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "message",
                    "toast message must not be empty",
                ));
            }
            let action_label = optional_string_property(node, properties, "action_label");
            if action_label
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(invalid_resolved_property(
                    node,
                    "action_label",
                    "toast action label must not be empty when provided",
                ));
            }
            let duration = match optional_string_property(node, properties, "duration").as_deref() {
                Some("long") => crate::ZsToastDuration::Long,
                Some("persistent") => crate::ZsToastDuration::Persistent,
                Some("short") | None => crate::ZsToastDuration::Short,
                Some(duration) => {
                    return Err(invalid_resolved_property(
                        node,
                        "duration",
                        format!("unsupported toast duration {duration:?}"),
                    ));
                }
            };
            let mut spec =
                crate::ZsToastSpec::new(node.id.widget_id().0, message).duration(duration);
            if let Some(action_label) = action_label {
                spec = spec.action(action_label);
            }
            let mut control = crate::toast_presenter(
                node.id.widget_id(),
                bool_property(node, properties, "open", false).then_some(spec),
                page,
            );
            if let Some(binding) = node.action_bindings.get("result") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_toast_result_with(move |result| {
                    let payload = match result.response {
                        crate::ZsToastResponse::Action => "action",
                        crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::CloseButton,
                        ) => "close",
                        crate::ZsToastResponse::Dismissed(
                            crate::ZsToastDismissReason::EscapeKey,
                        ) => "escape",
                        crate::ZsToastResponse::Dismissed(crate::ZsToastDismissReason::Timeout) => {
                            "timeout"
                        }
                    };
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(payload.to_owned()),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("open_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("open").cloned();
                control = control.on_toast_open_change_with(move |open| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(open),
                    })
                });
            }
            control
        }
        #[cfg(feature = "dialog")]
        "content_dialog" => {
            let actual = children.len();
            let mut children = children.into_iter();
            let Some(page) = children.next() else {
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

            let content = string_property(node, properties, "content", "");
            if content.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "content",
                    "content dialog content must not be empty",
                ));
            }
            let close_button = string_property(node, properties, "close_button", "");
            if close_button.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "close_button",
                    "content dialog close button must not be empty",
                ));
            }
            let title = optional_string_property(node, properties, "title");
            let primary_button = optional_string_property(node, properties, "primary_button");
            let secondary_button = optional_string_property(node, properties, "secondary_button");
            let default_button = optional_string_property(node, properties, "default_button")
                .map(|value| document_dialog_button(node, "default_button", &value))
                .transpose()?;
            let destructive_button =
                optional_string_property(node, properties, "destructive_button")
                    .map(|value| document_dialog_button(node, "destructive_button", &value))
                    .transpose()?;

            let has_button = |button| match button {
                crate::ZsContentDialogButton::Primary => primary_button
                    .as_deref()
                    .is_some_and(|label| !label.trim().is_empty()),
                crate::ZsContentDialogButton::Secondary => secondary_button
                    .as_deref()
                    .is_some_and(|label| !label.trim().is_empty()),
                crate::ZsContentDialogButton::Close => true,
            };
            for (property, button) in [
                ("default_button", default_button),
                ("destructive_button", destructive_button),
            ] {
                if button.is_some_and(|button| !has_button(button)) {
                    return Err(invalid_resolved_property(
                        node,
                        property,
                        "content dialog role must address an available button",
                    ));
                }
            }
            if default_button.is_some() && default_button == destructive_button {
                return Err(invalid_resolved_property(
                    node,
                    "destructive_button",
                    "content dialog default and destructive buttons must differ",
                ));
            }

            let mut spec = crate::ZsContentDialogSpec::new(content, close_button);
            if let Some(title) = title {
                spec = spec.title(title);
            }
            if let Some(primary) = primary_button {
                spec = spec.primary_button(primary);
            }
            if let Some(secondary) = secondary_button {
                spec = spec.secondary_button(secondary);
            }
            if let Some(default_button) = default_button {
                spec = spec.default_button(default_button);
            }
            if let Some(destructive_button) = destructive_button {
                spec = spec.destructive_button(destructive_button);
            }

            let mut control = crate::content_dialog(
                node.id.widget_id(),
                bool_property(node, properties, "open", false),
                spec,
                page,
            );
            if let Some(binding) = node.action_bindings.get("result") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_dialog_result_with(move |result| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: None,
                        payload: Value::String(
                            match result {
                                crate::ZsContentDialogResult::Primary => "primary",
                                crate::ZsContentDialogResult::Secondary => "secondary",
                                crate::ZsContentDialogResult::Close => "close",
                            }
                            .to_owned(),
                        ),
                    })
                });
            }
            if let Some(binding) = node.action_bindings.get("open_change") {
                let mapper = mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                let property_binding = node.property_bindings.get("open").cloned();
                control = control.on_dialog_open_change_with(move |open| {
                    mapper.map(UiDocumentAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        payload: Value::Bool(open),
                    })
                });
            }
            control
        }
        #[cfg(feature = "label")]
        "text" => {
            let value = string_property(node, properties, "text", "");
            crate::styled_text(value, semantic_text_style(node, properties)?)
        }
        #[cfg(feature = "icon")]
        "icon" => document_icon(node, properties)?,
        #[cfg(feature = "button")]
        "button" => {
            let label = string_property(node, properties, "label", "Button");
            if label.trim().is_empty() {
                return Err(invalid_resolved_property(
                    node,
                    "label",
                    "button label must not be empty",
                ));
            }
            let presentation = string_property(node, properties, "presentation", "standard");
            let icon = optional_semantic_icon_property(node, properties, "icon")?;
            let mut control = match presentation.as_str() {
                "standard" if icon.is_none() => crate::button(label),
                "primary" if icon.is_none() => crate::primary_button(label),
                "toolbar" => crate::toolbar_button(
                    label,
                    icon.ok_or_else(|| {
                        invalid_resolved_property(
                            node,
                            "icon",
                            "toolbar button presentation requires a semantic icon",
                        )
                    })?,
                ),
                "icon" => crate::icon_button(
                    label,
                    icon.ok_or_else(|| {
                        invalid_resolved_property(
                            node,
                            "icon",
                            "icon button presentation requires a semantic icon",
                        )
                    })?,
                ),
                "standard" | "primary" => {
                    return Err(invalid_resolved_property(
                        node,
                        "icon",
                        "standard and primary button presentations do not accept an icon",
                    ));
                }
                _ => {
                    return Err(invalid_resolved_property(
                        node,
                        "presentation",
                        format!("unsupported button presentation {presentation:?}"),
                    ));
                }
            }
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
        #[cfg(feature = "breadcrumb")]
        "breadcrumb" => document_breadcrumb(node, properties, mapper)?,
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
        #[cfg(feature = "password-box")]
        "password_box" => {
            let property_binding = node.property_bindings.get("value").cloned();
            if property_binding.is_some() && secure.is_none() {
                return Err(UiDocumentRuntimeError::SecureChannelRequired {
                    node_id: node.id.as_str().to_owned(),
                });
            }
            let value = property_binding
                .as_deref()
                .and_then(|binding| secure.and_then(|secure| secure.values.get(binding)))
                .cloned()
                .unwrap_or_default();
            let reveal_mode = match optional_string_property(node, properties, "reveal_mode")
                .as_deref()
            {
                Some("hidden") => crate::ZsPasswordRevealMode::Hidden,
                Some("peek") => crate::ZsPasswordRevealMode::Peek,
                Some("visible") => crate::ZsPasswordRevealMode::Visible,
                Some("platform_default") | None => crate::ZsPasswordRevealMode::platform_default(),
                Some(mode) => {
                    return Err(invalid_resolved_property(
                        node,
                        "reveal_mode",
                        format!("unsupported password reveal mode {mode:?}"),
                    ));
                }
            };
            let mut control = crate::password_box(value).reveal_mode(reveal_mode);
            if let Some(binding) = node.action_bindings.get("change") {
                let Some(secure) = secure else {
                    return Err(UiDocumentRuntimeError::SecureChannelRequired {
                        node_id: node.id.as_str().to_owned(),
                    });
                };
                let Some(property_binding) = property_binding else {
                    return Err(invalid_resolved_property(
                        node,
                        "value",
                        "password change requires a secure value property binding",
                    ));
                };
                let mapper = secure.mapper.clone();
                let node_id = node.id.as_str().to_owned();
                let binding = binding.clone();
                control = control.on_password_change_with(move |value| {
                    mapper.map(UiDocumentSecretAction {
                        node_id: node_id.clone(),
                        binding: binding.clone(),
                        property_binding: property_binding.clone(),
                        value,
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
        #[cfg(feature = "auto-suggest")]
        "auto_suggest" => document_auto_suggest(node, properties, mapper)?,
        #[cfg(feature = "command-palette")]
        "command_palette" => document_command_palette(node, properties, children, mapper)?,
        #[cfg(feature = "tree")]
        "tree" => document_tree(node, properties, mapper)?,
        #[cfg(feature = "grid-view")]
        "grid_view" => document_grid_view(node, properties, mapper)?,
        #[cfg(feature = "table")]
        "table" => document_table(node, properties, mapper)?,
        #[cfg(feature = "date-picker")]
        "date_picker" => document_date_picker(node, properties, mapper)?,
        #[cfg(feature = "time-picker")]
        "time_picker" => document_time_picker(node, properties, mapper)?,
        #[cfg(feature = "color-picker")]
        "color_picker" => document_color_picker(node, properties, mapper)?,
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
        #[cfg(feature = "progress-ring")]
        "progress_ring" => document_progress_ring(node, properties)?,
        component => {
            return Err(UiDocumentRuntimeError::UnsupportedComponent {
                component: component.to_owned(),
            });
        }
    };
    // Tooltip is an attached modifier rather than another control, so the
    // wrapped control keeps the WidgetId used by hit testing and typed events.
    if node.component != "tooltip" {
        view = view.id(node.id.widget_id());
    }
    Ok(apply_layout(view, node))
}

#[cfg(feature = "teaching-tip")]
fn descendant_document_widget_id(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
    fn find(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
        if node.id.as_str() == id {
            return Some(node.id.widget_id());
        }
        node.children.iter().find_map(|child| find(child, id))
    }

    node.children.iter().find_map(|child| find(child, id))
}

#[cfg(feature = "flyout")]
fn flyout_document_target(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
    fn find(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
        if node.id.as_str() == id {
            return Some(node.id.widget_id());
        }
        node.children.iter().find_map(|child| find(child, id))
    }

    node.children.first().and_then(|page| find(page, id))
}

#[cfg(feature = "menu-flyout")]
fn menu_flyout_document_target(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
    fn find(node: &UiNode, id: &str) -> Option<crate::WidgetId> {
        if node.id.as_str() == id {
            return Some(node.id.widget_id());
        }
        node.children.iter().find_map(|child| find(child, id))
    }

    node.children.first().and_then(|page| find(page, id))
}

#[cfg(feature = "menu-flyout")]
fn document_menu_flyout(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<crate::MenuSpec, UiDocumentRuntimeError> {
    fn compile(items: Vec<UiMenuFlyoutItem>) -> Vec<crate::MenuItemSpec> {
        items
            .into_iter()
            .map(|item| match item {
                UiMenuFlyoutItem::Command {
                    id,
                    label,
                    enabled,
                    checked,
                    accelerator,
                } => {
                    let id = id.as_str().to_owned();
                    crate::MenuItemSpec::Command {
                        id: Some(id.clone()),
                        label,
                        command: crate::Command::custom(id),
                        enabled,
                        checked,
                        accelerator: accelerator.map(|accelerator| {
                            accelerator
                                .native_accelerator()
                                .expect("validated document accelerator")
                        }),
                    }
                }
                UiMenuFlyoutItem::Separator => crate::MenuItemSpec::Separator,
                UiMenuFlyoutItem::Submenu {
                    id,
                    label,
                    enabled,
                    items,
                } => {
                    let id = id.as_str().to_owned();
                    crate::MenuItemSpec::Submenu {
                        id: Some(id.clone()),
                        label,
                        enabled,
                        menu: crate::MenuSpec {
                            id: Some(id),
                            title: None,
                            items: compile(items),
                        },
                    }
                }
            })
            .collect()
    }

    let items = property_value(node, properties, "items")
        .and_then(|value| ui_menu_flyout_items_from_value(&value))
        .ok_or_else(|| {
            invalid_resolved_property(
                node,
                "items",
                "a valid non-empty menu_flyout item tree is required",
            )
        })?;
    Ok(crate::MenuSpec {
        id: Some(node.id.as_str().to_owned()),
        title: None,
        items: compile(items),
    })
}

#[cfg(feature = "flyout")]
fn document_positive_extent(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Dp, UiDocumentRuntimeError> {
    let value = property_value(node, properties, property)
        .and_then(|value| value.as_f64())
        .filter(|value| *value > 0.0 && *value <= f64::from(f32::MAX))
        .ok_or_else(|| {
            invalid_resolved_property(node, property, "a positive finite DP extent is required")
        })?;
    Ok(Dp::new(value as f32))
}

#[cfg(feature = "teaching-tip")]
fn node_has_property_source(node: &UiNode, property: &str) -> bool {
    node.properties.contains_key(property)
        || node.property_bindings.contains_key(property)
        || node.localization.contains_key(property)
}

#[cfg(feature = "dialog")]
fn document_dialog_button(
    node: &UiNode,
    property: &str,
    value: &str,
) -> Result<crate::ZsContentDialogButton, UiDocumentRuntimeError> {
    match value {
        "primary" => Ok(crate::ZsContentDialogButton::Primary),
        "secondary" => Ok(crate::ZsContentDialogButton::Secondary),
        "close" => Ok(crate::ZsContentDialogButton::Close),
        _ => Err(invalid_resolved_property(
            node,
            property,
            "content dialog button must be primary, secondary or close",
        )),
    }
}

#[cfg(feature = "auto-suggest")]
fn document_auto_suggest<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let authoring_suggestions = auto_suggestions_property(node, properties)?;
    let mut author_to_runtime = BTreeMap::new();
    let mut runtime_to_author = BTreeMap::new();
    let mut suggestions = Vec::with_capacity(authoring_suggestions.len());
    for suggestion in authoring_suggestions {
        let (author_id, text) = suggestion.into_parts();
        let runtime_id = crate::ui_document::ui_auto_suggestion_runtime_id(&node.id, &author_id);
        if let Some(first) = runtime_to_author.insert(runtime_id, author_id.as_str().to_owned()) {
            return Err(invalid_resolved_property(
                node,
                "suggestions",
                format!(
                    "suggestion ids {first:?} and {:?} collide after stable runtime mapping",
                    author_id.as_str()
                ),
            ));
        }
        author_to_runtime.insert(author_id.as_str().to_owned(), runtime_id);
        suggestions.push(crate::ZsAutoSuggestion::new(runtime_id, text));
    }

    let highlighted = optional_auto_suggestion_id_property(node, properties, "highlighted")?
        .map(|highlighted| {
            author_to_runtime
                .get(highlighted.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "highlighted",
                        format!(
                            "id {:?} does not address an available suggestion",
                            highlighted.as_str()
                        ),
                    )
                })
        })
        .transpose()?;

    let mut control =
        crate::auto_suggest_box(string_property(node, properties, "query", ""), suggestions)
            .highlighted_suggestion(highlighted)
            .expanded(bool_property(node, properties, "expanded", false))
            .query_icon(bool_property(node, properties, "query_icon", true));
    if let Some(placeholder) = optional_string_property(node, properties, "placeholder") {
        control = control.placeholder(placeholder);
    }
    if let Some(no_results_text) = optional_string_property(node, properties, "no_results_text") {
        control = control.no_results_text(no_results_text);
    }

    let runtime_to_author = Arc::new(runtime_to_author);
    if let Some(binding) = node.action_bindings.get("text_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("query").cloned();
        control = control.on_auto_suggest_text_change_with(move |change| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(change.text),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("choose") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("highlighted").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_suggestion_chosen_with(move |suggestion| {
            let author_id = runtime_to_author
                .get(&suggestion)
                .cloned()
                .expect("chosen runtime suggestion must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("submit") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_query_submit_with(move |submission| {
            let chosen = submission
                .chosen
                .and_then(|chosen| runtime_to_author.get(&chosen).cloned())
                .map(Value::String)
                .unwrap_or(Value::Null);
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload: serde_json::json!({
                    "query": submission.query,
                    "chosen": chosen,
                }),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        control = control.on_auto_suggest_expanded_change_with(move |expanded| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(expanded),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "command-palette")]
fn document_command_palette<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let actual = children.len();
    let mut children = children.into_iter();
    let Some(page) = children.next() else {
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

    let authoring_items = command_palette_items_property(node, properties)?;
    let mut author_to_runtime = BTreeMap::new();
    let mut runtime_to_author = BTreeMap::new();
    let mut items = Vec::with_capacity(authoring_items.len());
    for item in authoring_items {
        let author_id = item.id().clone();
        let runtime_id = crate::ui_document::ui_command_palette_runtime_id(&node.id, &author_id);
        if let Some(first) = runtime_to_author.insert(runtime_id, author_id.as_str().to_owned()) {
            return Err(invalid_resolved_property(
                node,
                "items",
                format!(
                    "command ids {first:?} and {:?} collide after stable runtime mapping",
                    author_id.as_str()
                ),
            ));
        }
        author_to_runtime.insert(author_id.as_str().to_owned(), runtime_id);
        let mut runtime_item =
            crate::ZsCommandPaletteItem::new(runtime_id, item.title()).enabled(item.is_enabled());
        if let Some(subtitle) = item.subtitle_text() {
            runtime_item = runtime_item.subtitle(subtitle);
        }
        if !item.keyword_values().is_empty() {
            runtime_item = runtime_item.keywords(item.keyword_values().iter().cloned());
        }
        if let Some(shortcut) = item.shortcut_text() {
            runtime_item = runtime_item.shortcut(shortcut);
        }
        if let Some(icon) = item.semantic_icon() {
            runtime_item = runtime_item.icon(icon);
        }
        items.push(runtime_item);
    }

    let query = string_property(node, properties, "query", "");
    let highlighted = optional_command_palette_item_id_property(node, properties, "highlighted")?
        .map(|highlighted| {
            author_to_runtime
                .get(highlighted.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "highlighted",
                        format!(
                            "id {:?} does not address an available command",
                            highlighted.as_str()
                        ),
                    )
                })
        })
        .transpose()?;
    if highlighted.is_some_and(|highlighted| {
        crate::command_palette::command_palette_state(true, &query, &items, Some(highlighted))
            .highlighted
            != Some(highlighted)
    }) {
        return Err(invalid_resolved_property(
            node,
            "highlighted",
            "highlighted command must be enabled and match the current query",
        ));
    }

    let mut control = crate::command_palette(
        node.id.widget_id(),
        bool_property(node, properties, "open", false),
        query,
        items,
        page,
    )
    .highlighted_command(highlighted);
    if let Some(placeholder) = optional_string_property(node, properties, "placeholder") {
        control = control.command_palette_placeholder(placeholder);
    }
    if let Some(no_results_text) = optional_string_property(node, properties, "no_results_text") {
        control = control.command_palette_no_results_text(no_results_text);
    }

    let runtime_to_author = Arc::new(runtime_to_author);
    if let Some(binding) = node.action_bindings.get("query_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("query").cloned();
        control = control.on_command_palette_query_change_with(move |query| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(query),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("highlight_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("highlighted").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_command_palette_highlight_change_with(move |highlighted| {
            let author_id = runtime_to_author
                .get(&highlighted)
                .cloned()
                .expect("highlighted runtime command must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("invoke") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("highlighted").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_command_palette_invoke_with(move |invoked| {
            let author_id = runtime_to_author
                .get(&invoked)
                .cloned()
                .expect("invoked runtime command must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("open_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("open").cloned();
        control = control.on_command_palette_open_change_with(move |open| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(open),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "tree")]
fn document_tree<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    fn compile_nodes(
        owner: &UiNode,
        authoring_nodes: &[crate::ui_document::UiTreeNode],
        author_to_runtime: &mut BTreeMap<String, crate::ZsTreeNodeId>,
        runtime_to_author: &mut BTreeMap<crate::ZsTreeNodeId, String>,
        expandable: &mut BTreeSet<String>,
    ) -> Result<Vec<crate::ZsTreeNode>, UiDocumentRuntimeError> {
        let mut nodes = Vec::with_capacity(authoring_nodes.len());
        for authoring_node in authoring_nodes {
            let author_id = authoring_node.id().as_str().to_owned();
            let runtime_id = crate::ui_document::ui_tree_runtime_id(&owner.id, authoring_node.id());
            if let Some(first) = runtime_to_author.insert(runtime_id, author_id.clone()) {
                return Err(invalid_resolved_property(
                    owner,
                    "nodes",
                    format!(
                        "tree node ids {first:?} and {author_id:?} collide after stable runtime mapping"
                    ),
                ));
            }
            author_to_runtime.insert(author_id.clone(), runtime_id);
            if authoring_node.is_expandable() {
                expandable.insert(author_id);
            }
            let children = compile_nodes(
                owner,
                authoring_node.child_nodes(),
                author_to_runtime,
                runtime_to_author,
                expandable,
            )?;
            let mut runtime_node = crate::ZsTreeNode::new(runtime_id, authoring_node.label())
                .children(children)
                .unrealized_children(authoring_node.has_unrealized_children());
            if let Some(icon) = authoring_node.semantic_icon() {
                runtime_node = runtime_node.icon(icon);
            }
            nodes.push(runtime_node);
        }
        Ok(nodes)
    }

    let authoring_nodes = tree_nodes_property(node, properties)?;
    let mut author_to_runtime = BTreeMap::new();
    let mut runtime_to_author = BTreeMap::new();
    let mut expandable = BTreeSet::new();
    let roots = compile_nodes(
        node,
        &authoring_nodes,
        &mut author_to_runtime,
        &mut runtime_to_author,
        &mut expandable,
    )?;

    let expanded_author = tree_node_ids_property(node, properties, "expanded")?;
    let mut expanded = BTreeSet::new();
    for author_id in &expanded_author {
        if !expandable.contains(author_id.as_str()) {
            return Err(invalid_resolved_property(
                node,
                "expanded",
                format!(
                    "id {:?} must address an available expandable tree node",
                    author_id.as_str()
                ),
            ));
        }
        expanded.insert(
            *author_to_runtime
                .get(author_id.as_str())
                .expect("expandable author tree id must have a runtime mapping"),
        );
    }
    let selected = optional_tree_node_id_property(node, properties, "selected")?
        .map(|selected| {
            author_to_runtime
                .get(selected.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "selected",
                        format!(
                            "id {:?} does not address an available tree node",
                            selected.as_str()
                        ),
                    )
                })
        })
        .transpose()?;

    let mut control = crate::tree_view(roots)
        .expanded_tree_nodes(expanded)
        .selected_tree_node(selected);
    let runtime_to_author = Arc::new(runtime_to_author);
    if let Some(binding) = node.action_bindings.get("select") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("selected").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_tree_select_with(move |selected| {
            let author_id = runtime_to_author
                .get(&selected)
                .cloned()
                .expect("selected runtime tree node must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_tree_expansion_change_with(move |change| {
            let author_id = runtime_to_author
                .get(&change.node)
                .cloned()
                .expect("expanded runtime tree node must address compiled document data");
            let author_id = crate::ui_document::UiTreeNodeId::new(author_id)
                .expect("compiled author tree id must remain valid");
            let mut next = expanded_author.clone();
            if change.expanded {
                next.insert(author_id);
            } else {
                next.remove(&author_id);
            }
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: serde_json::to_value(next)
                    .expect("validated expanded tree ids must serialize"),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("invoke") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_tree_invoke_with(move |invoked| {
            let author_id = runtime_to_author
                .get(&invoked)
                .cloned()
                .expect("invoked runtime tree node must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload: Value::String(author_id),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "document-shell")]
fn document_command_bar<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    if children.is_empty() {
        return Err(UiDocumentRuntimeError::InvalidChildCount {
            component: node.component.clone(),
            expected: 1,
            actual: 0,
        });
    }
    let trailing_ids = document_string_id_array(node, properties, "trailing")?;
    let trailing_ids = trailing_ids.into_iter().collect::<BTreeSet<_>>();
    let child_ids = node
        .children
        .iter()
        .map(|child| child.id.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    if let Some(unknown) = trailing_ids.iter().find(|id| !child_ids.contains(*id)) {
        return Err(invalid_resolved_property(
            node,
            "trailing",
            format!("trailing id {unknown:?} does not address a direct child"),
        ));
    }

    let mut leading = Vec::new();
    let mut trailing = Vec::new();
    for (authoring, child) in node.children.iter().zip(children) {
        if trailing_ids.contains(authoring.id.as_str()) {
            trailing.push(child);
        } else {
            leading.push(child);
        }
    }
    let mut spec = crate::ZsCommandBarSpec::new()
        .leading(leading)
        .trailing(trailing);
    if let Some(gap) = optional_non_negative_extent(node, properties, "gap")? {
        spec = spec.gap(Dp::new(gap));
    }
    Ok(crate::command_bar(spec))
}

#[cfg(feature = "shell")]
fn document_navigation<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let actual = children.len();
    let mut children = children.into_iter();
    let Some(content) = children.next() else {
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

    let author_items = navigation_items_property(node, properties, "items", true)?;
    let author_footer_items = navigation_items_property(node, properties, "footer_items", false)?;
    let selected = optional_navigation_item_id_property(node, properties, "selected")?;
    let mut runtime_ids = BTreeMap::new();
    let mut compile_group = |items: Vec<crate::ui_document::UiNavigationItem>| {
        items
            .into_iter()
            .map(|item| {
                let author_id = item.id().clone();
                let runtime_id = crate::ui_document::ui_navigation_runtime_id(&node.id, &author_id);
                if let Some(first) =
                    runtime_ids.insert(runtime_id.0, author_id.as_str().to_owned())
                {
                    return Err(invalid_resolved_property(
                        node,
                        "items",
                        format!(
                            "navigation item ids {first:?} and {:?} collide after stable runtime mapping",
                            author_id.as_str()
                        ),
                    ));
                }
                let is_selected = selected.as_ref() == Some(&author_id);
                if is_selected && !item.is_enabled() {
                    return Err(invalid_resolved_property(
                        node,
                        "selected",
                        "selected navigation item must be enabled",
                    ));
                }
                let mut runtime_item = crate::navigation_item(
                    item.label(),
                    item.semantic_icon(),
                    is_selected,
                )
                .id(runtime_id)
                .enabled(item.is_enabled());
                if let Some(binding) = node.action_bindings.get("select") {
                    let action = UiDocumentAction {
                        node_id: node.id.as_str().to_owned(),
                        binding: binding.clone(),
                        property_binding: node.property_bindings.get("selected").cloned(),
                        payload: Value::String(author_id.as_str().to_owned()),
                    };
                    runtime_item = runtime_item.on_click(mapper.map(action));
                }
                Ok(runtime_item)
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let items = compile_group(author_items)?;
    let footer_items = compile_group(author_footer_items)?;
    if let Some(selected) = selected.as_ref() {
        if !runtime_ids.values().any(|id| id == selected.as_str()) {
            return Err(invalid_resolved_property(
                node,
                "selected",
                "selected navigation item must address an available item id",
            ));
        }
    }

    let mut spec = crate::ZsNavigationViewSpec::new(
        string_property(node, properties, "title", ""),
        string_property(node, properties, "subtitle", ""),
    )
    .items(items)
    .footer_items(footer_items)
    .content(node.id.widget_id(), content);
    if let Some(width) = optional_non_negative_extent(node, properties, "pane_width")? {
        spec = spec.pane_width(Dp::new(width));
    }
    if let Some(width) = optional_non_negative_extent(node, properties, "minimum_content_width")? {
        spec = spec.minimum_content_width(Dp::new(width));
    }
    Ok(crate::navigation_view(spec))
}

#[cfg(feature = "grid-view")]
fn document_grid_view<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let authoring_items = grid_view_items_property(node, properties)?;
    let mut author_to_runtime = BTreeMap::new();
    let mut runtime_to_author = BTreeMap::new();
    let mut items = Vec::with_capacity(authoring_items.len());
    for item in authoring_items {
        let author_id = item.id().clone();
        let runtime_id = crate::ui_document::ui_grid_view_runtime_id(&node.id, &author_id);
        if let Some(first) = runtime_to_author.insert(runtime_id, author_id.as_str().to_owned()) {
            return Err(invalid_resolved_property(
                node,
                "items",
                format!(
                    "grid-view item ids {first:?} and {:?} collide after stable runtime mapping",
                    author_id.as_str()
                ),
            ));
        }
        author_to_runtime.insert(author_id.as_str().to_owned(), runtime_id);
        let mut runtime_item = crate::ZsGridViewItem::new(runtime_id, item.title());
        if let Some(subtitle) = item.subtitle_text() {
            runtime_item = runtime_item.subtitle(subtitle);
        }
        if let Some(icon) = item.semantic_icon() {
            runtime_item = runtime_item.icon(icon);
        }
        items.push(runtime_item);
    }

    let selected = optional_grid_view_item_id_property(node, properties, "selected")?
        .map(|selected| {
            author_to_runtime
                .get(selected.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "selected",
                        format!(
                            "id {:?} does not address an available grid-view item",
                            selected.as_str()
                        ),
                    )
                })
        })
        .transpose()?;

    let mut control = crate::grid_view(items).selected_grid_view_item(selected);
    let runtime_to_author = Arc::new(runtime_to_author);
    if let Some(binding) = node.action_bindings.get("select") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("selected").cloned();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_grid_view_select_with(move |selected| {
            let author_id = runtime_to_author
                .get(&selected)
                .cloned()
                .expect("selected runtime grid-view item must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("invoke") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_grid_view_invoke_with(move |invoked| {
            let author_id = runtime_to_author
                .get(&invoked)
                .cloned()
                .expect("invoked runtime grid-view item must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload: Value::String(author_id),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "table")]
fn document_table<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let author_columns = table_columns_property(node, properties)?;
    let author_rows = table_rows_property(node, properties)?;
    if !crate::ui_document::ui_table_data_is_compatible(&author_columns, &author_rows) {
        return Err(invalid_resolved_property(
            node,
            "rows",
            "every row must contain exactly one cell for every declared column id",
        ));
    }

    let mut author_to_runtime_columns = BTreeMap::new();
    let mut runtime_to_author_columns = BTreeMap::new();
    let mut columns = Vec::with_capacity(author_columns.len());
    for column in &author_columns {
        let runtime_id = crate::ui_document::ui_table_column_runtime_id(&node.id, column.id());
        if let Some(first) =
            runtime_to_author_columns.insert(runtime_id, column.id().as_str().to_owned())
        {
            return Err(invalid_resolved_property(
                node,
                "columns",
                format!(
                    "table column ids {first:?} and {:?} collide after stable runtime mapping",
                    column.id().as_str()
                ),
            ));
        }
        author_to_runtime_columns.insert(column.id().as_str().to_owned(), runtime_id);
        let mut runtime_column = crate::ZsTableColumn::new(runtime_id, column.header());
        runtime_column = match column.column_width() {
            crate::ui_document::UiTableColumnWidth::Fixed { width } => {
                runtime_column.fixed_width(width)
            }
            crate::ui_document::UiTableColumnWidth::Fill { weight } => {
                runtime_column.fill_width(weight)
            }
        };
        runtime_column = runtime_column
            .alignment(match column.column_alignment() {
                crate::ui_document::UiTableColumnAlignment::Start => crate::HorizontalAlign::Start,
                crate::ui_document::UiTableColumnAlignment::Center => {
                    crate::HorizontalAlign::Center
                }
                crate::ui_document::UiTableColumnAlignment::End => crate::HorizontalAlign::End,
            })
            .sortable(column.is_sortable());
        columns.push(runtime_column);
    }

    let mut author_to_runtime_rows = BTreeMap::new();
    let mut runtime_to_author_rows = BTreeMap::new();
    let mut rows = Vec::with_capacity(author_rows.len());
    for row in &author_rows {
        let runtime_id = crate::ui_document::ui_table_row_runtime_id(&node.id, row.id());
        if let Some(first) = runtime_to_author_rows.insert(runtime_id, row.id().as_str().to_owned())
        {
            return Err(invalid_resolved_property(
                node,
                "rows",
                format!(
                    "table row ids {first:?} and {:?} collide after stable runtime mapping",
                    row.id().as_str()
                ),
            ));
        }
        author_to_runtime_rows.insert(row.id().as_str().to_owned(), runtime_id);
        rows.push(crate::ZsTableRow::new(
            runtime_id,
            author_columns.iter().map(|column| {
                row.cells()
                    .get(column.id())
                    .cloned()
                    .expect("validated table row contains every column")
            }),
        ));
    }

    let selected = optional_table_row_id_property(node, properties, "selected")?
        .map(|selected| {
            author_to_runtime_rows
                .get(selected.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "selected",
                        format!(
                            "id {:?} does not address an available table row",
                            selected.as_str()
                        ),
                    )
                })
        })
        .transpose()?;
    let sort = optional_table_sort_property(node, properties, "sort")?
        .map(|sort| {
            let column = author_columns
                .iter()
                .find(|column| column.id() == sort.column() && column.is_sortable())
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "sort",
                        format!(
                            "column {:?} is unavailable or not sortable",
                            sort.column().as_str()
                        ),
                    )
                })?;
            let runtime_column = author_to_runtime_columns[column.id().as_str()];
            let direction = match sort.direction() {
                crate::ui_document::UiTableSortDirection::Ascending => {
                    crate::ZsTableSortDirection::Ascending
                }
                crate::ui_document::UiTableSortDirection::Descending => {
                    crate::ZsTableSortDirection::Descending
                }
            };
            Ok(crate::ZsTableSort::new(runtime_column, direction))
        })
        .transpose()?;

    let mut control = crate::data_grid(columns, rows)
        .selected_table_row(selected)
        .table_sort(sort);
    let runtime_to_author_rows = Arc::new(runtime_to_author_rows);
    let runtime_to_author_columns = Arc::new(runtime_to_author_columns);
    if let Some(binding) = node.action_bindings.get("select") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("selected").cloned();
        let runtime_to_author_rows = Arc::clone(&runtime_to_author_rows);
        control = control.on_table_select_with(move |selected| {
            let author_id = runtime_to_author_rows
                .get(&selected)
                .cloned()
                .expect("selected runtime table row must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("sort") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("sort").cloned();
        let runtime_to_author_columns = Arc::clone(&runtime_to_author_columns);
        control = control.on_table_sort_with(move |sort| {
            let author_id = runtime_to_author_columns
                .get(&sort.column)
                .cloned()
                .expect("sorted runtime table column must address compiled document data");
            let direction = match sort.direction {
                crate::ZsTableSortDirection::Ascending => {
                    crate::ui_document::UiTableSortDirection::Ascending
                }
                crate::ZsTableSortDirection::Descending => {
                    crate::ui_document::UiTableSortDirection::Descending
                }
            };
            let payload = serde_json::to_value(crate::ui_document::UiTableSort::new(
                crate::ui_document::UiTableColumnId::new(author_id)
                    .expect("compiled table author id remains valid"),
                direction,
            ))
            .expect("table sort action must serialize");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload,
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("invoke") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let runtime_to_author_rows = Arc::clone(&runtime_to_author_rows);
        control = control.on_table_invoke_with(move |invoked| {
            let author_id = runtime_to_author_rows
                .get(&invoked)
                .cloned()
                .expect("invoked runtime table row must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload: Value::String(author_id),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "breadcrumb")]
fn document_breadcrumb<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let authoring_items = breadcrumb_items_property(node, properties)?;
    let mut runtime_to_author = Vec::with_capacity(authoring_items.len());
    let mut items = Vec::with_capacity(authoring_items.len());
    for item in authoring_items {
        let runtime_id = crate::ui_document::ui_breadcrumb_runtime_id(&node.id, item.id());
        if let Some((_, first)) = runtime_to_author
            .iter()
            .find(|(candidate, _)| *candidate == runtime_id)
        {
            return Err(invalid_resolved_property(
                node,
                "items",
                format!(
                    "breadcrumb item ids {first:?} and {:?} collide after stable runtime mapping",
                    item.id().as_str()
                ),
            ));
        }
        runtime_to_author.push((runtime_id, item.id().as_str().to_owned()));
        items.push(crate::ZsBreadcrumbItem::new(runtime_id, item.label()));
    }

    let mut control =
        crate::breadcrumb_bar(items).expanded(bool_property(node, properties, "expanded", false));
    let runtime_to_author = Arc::new(runtime_to_author);
    if let Some(binding) = node.action_bindings.get("select") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let runtime_to_author = Arc::clone(&runtime_to_author);
        control = control.on_breadcrumb_select_with(move |selected| {
            let author_id = runtime_to_author
                .iter()
                .find(|(runtime_id, _)| *runtime_id == selected)
                .map(|(_, author_id)| author_id.clone())
                .expect("selected runtime breadcrumb item must address compiled document data");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload: Value::String(author_id),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        control = control.on_breadcrumb_expanded_change_with(move |expanded| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(expanded),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "date-picker")]
fn document_date_picker<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let minimum_fallback =
        crate::ZsDate::new(crate::ZsDate::MIN_YEAR, 1, 1).expect("minimum supported date is valid");
    let maximum_fallback = crate::ZsDate::new(crate::ZsDate::MAX_YEAR, 12, 31)
        .expect("maximum supported date is valid");
    let minimum = date_property(node, properties, "minimum")?.unwrap_or(minimum_fallback);
    let maximum = date_property(node, properties, "maximum")?.unwrap_or(maximum_fallback);
    if minimum > maximum {
        return Err(invalid_resolved_property(
            node,
            "maximum",
            "maximum must not be earlier than minimum",
        ));
    }
    let value = date_property(node, properties, "value")?
        .ok_or_else(|| invalid_resolved_property(node, "value", "a selected date is required"))?;
    if !(minimum..=maximum).contains(&value) {
        return Err(invalid_resolved_property(
            node,
            "value",
            "selected date must be within minimum and maximum",
        ));
    }
    let visible_month = date_property(node, properties, "visible_month")?
        .unwrap_or_else(|| value.first_day_of_month());
    if visible_month.day() != 1 {
        return Err(invalid_resolved_property(
            node,
            "visible_month",
            "visible month must use the first day of its month",
        ));
    }
    let first_month = minimum.first_day_of_month();
    let last_month = maximum.first_day_of_month();
    if !(first_month..=last_month).contains(&visible_month) {
        return Err(invalid_resolved_property(
            node,
            "visible_month",
            "visible month must intersect the configured date range",
        ));
    }

    let mut control = crate::date_picker(value)
        .date_range(minimum, maximum)
        .visible_month(visible_month)
        .expanded(bool_property(node, properties, "expanded", false));
    if let Some(today) = date_property(node, properties, "today")? {
        control = control.today(today);
    }
    if let Some(binding) = node.action_bindings.get("change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("value").cloned();
        control = control.on_date_change_with(move |value| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(value.iso_string()),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("month_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("visible_month").cloned();
        control = control.on_date_picker_month_change_with(move |month| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(month.iso_string()),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        control = control.on_date_picker_expanded_change_with(move |expanded| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(expanded),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "time-picker")]
fn document_time_picker<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let value = time_property(node, properties, "value")?
        .ok_or_else(|| invalid_resolved_property(node, "value", "a selected time is required"))?;
    let increment_value = property_value(node, properties, "minute_increment")
        .map(|value| {
            value.as_u64().ok_or_else(|| {
                invalid_resolved_property(node, "minute_increment", "increment must be an integer")
            })
        })
        .transpose()?
        .unwrap_or(1);
    let increment = u8::try_from(increment_value)
        .map_err(|_| {
            invalid_resolved_property(
                node,
                "minute_increment",
                "increment must fit in an unsigned byte",
            )
        })
        .and_then(|increment| {
            crate::ZsMinuteIncrement::new(increment).map_err(|error| {
                invalid_resolved_property(node, "minute_increment", error.to_string())
            })
        })?;
    if value.minute() % increment.get() != 0 {
        return Err(invalid_resolved_property(
            node,
            "value",
            "selected time minute must align with minute_increment",
        ));
    }

    let mut control = crate::time_picker(value)
        .minute_increment(increment)
        .expanded(bool_property(node, properties, "expanded", false));
    if let Some(clock) = optional_string_property(node, properties, "clock_format") {
        control = match clock.as_str() {
            "platform_default" => control,
            "twelve_hour" => control.clock_format(crate::ZsClockFormat::TwelveHour),
            "twenty_four_hour" => control.clock_format(crate::ZsClockFormat::TwentyFourHour),
            _ => {
                return Err(invalid_resolved_property(
                    node,
                    "clock_format",
                    format!("unsupported clock format {clock:?}"),
                ));
            }
        };
    }
    if let Some(binding) = node.action_bindings.get("change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("value").cloned();
        control = control.on_time_change_with(move |value| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(value.to_string()),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        control = control.on_time_picker_expanded_change_with(move |expanded| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(expanded),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "color-picker")]
fn document_color_picker<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let color = color_property(node, properties, "value")?
        .ok_or_else(|| invalid_resolved_property(node, "value", "an RGBA color is required"))?;
    let alpha_enabled = bool_property(node, properties, "alpha_enabled", true);
    if !alpha_enabled && color.a != 255 {
        return Err(invalid_resolved_property(
            node,
            "value",
            "alpha must be FF when alpha_enabled is false",
        ));
    }
    let channel = match optional_string_property(node, properties, "active_channel")
        .as_deref()
        .unwrap_or("red")
    {
        "red" => crate::ZsColorChannel::Red,
        "green" => crate::ZsColorChannel::Green,
        "blue" => crate::ZsColorChannel::Blue,
        "alpha" if alpha_enabled => crate::ZsColorChannel::Alpha,
        "alpha" => {
            return Err(invalid_resolved_property(
                node,
                "active_channel",
                "alpha channel cannot be active when alpha is disabled",
            ));
        }
        channel => {
            return Err(invalid_resolved_property(
                node,
                "active_channel",
                format!("unsupported color channel {channel:?}"),
            ));
        }
    };

    let mut state = crate::ZsColorPickerState::new(color)
        .with_expanded(bool_property(node, properties, "expanded", false))
        .with_active_channel(channel);
    if !alpha_enabled {
        state = state.without_alpha();
    }
    let mut control = crate::color_picker(state);
    if let Some(binding) = node.action_bindings.get("change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("value").cloned();
        control = control.on_color_change_with(move |color| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(color.hex_rgba()),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("expanded_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("expanded").cloned();
        control = control.on_color_picker_expanded_change_with(move |expanded| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(expanded),
            })
        });
    }
    if let Some(binding) = node.action_bindings.get("channel_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("active_channel").cloned();
        control = control.on_color_channel_change_with(move |channel| {
            let channel = match channel {
                crate::ZsColorChannel::Red => "red",
                crate::ZsColorChannel::Green => "green",
                crate::ZsColorChannel::Blue => "blue",
                crate::ZsColorChannel::Alpha => "alpha",
            };
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(channel.to_owned()),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "list")]
fn document_list<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let item_ids = node
        .children
        .iter()
        .map(|child| child.id.as_str().to_owned())
        .collect::<Vec<_>>();
    let selected_index = optional_string_property(node, properties, "selected")
        .map(|selected| {
            item_ids
                .iter()
                .position(|item_id| *item_id == selected)
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "selected",
                        format!("id {selected:?} does not address a direct child"),
                    )
                })
        })
        .transpose()?;
    let mut control = crate::list(children, |child| child).selected_index(selected_index);
    if let Some(binding) = node.action_bindings.get("select") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("selected").cloned();
        control = control.on_list_select_with(move |selected_index| {
            let selected = item_ids
                .get(selected_index)
                .cloned()
                .expect("selected document list item must address compiled content");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::String(selected),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "progress-ring")]
fn document_progress_ring<Msg>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let minimum = finite_f32_property(node, properties, "minimum", 0.0)?;
    let maximum = finite_f32_property(node, properties, "maximum", 100.0)?;
    if minimum >= maximum {
        return Err(invalid_resolved_property(
            node,
            "maximum",
            "maximum must be greater than minimum",
        ));
    }
    let value = nullable_number_property(node, properties, "value", None)
        .map(|value| {
            if !value.is_finite()
                || value < f64::from(minimum)
                || value > f64::from(maximum)
                || value.abs() > f64::from(f32::MAX)
            {
                Err(invalid_resolved_property(
                    node,
                    "value",
                    "value must be null or a finite number within minimum and maximum",
                ))
            } else {
                Ok(value as f32)
            }
        })
        .transpose()?;
    let size = match optional_string_property(node, properties, "size").as_deref() {
        None | Some("medium") => ZsProgressRingSize::Medium,
        Some("small") => ZsProgressRingSize::Small,
        Some("large") => ZsProgressRingSize::Large,
        Some(_) => {
            return Err(invalid_resolved_property(
                node,
                "size",
                "size must be small, medium or large",
            ));
        }
    };
    let spec = value.map_or_else(ZsProgressRingSpec::indeterminate, |value| {
        ZsProgressRingSpec::determinate(value, ProgressRange::new(minimum, maximum))
    });
    Ok(crate::progress_ring(
        spec.active(bool_property(node, properties, "active", true))
            .size(size),
    ))
}

#[cfg(feature = "progress-ring")]
fn finite_f32_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    fallback: f64,
) -> Result<f32, UiDocumentRuntimeError> {
    let value = number_property(node, properties, property, fallback);
    if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
        return Err(invalid_resolved_property(
            node,
            property,
            "value must be finite and fit in f32",
        ));
    }
    Ok(value as f32)
}

#[cfg(feature = "grid")]
fn document_grid<Msg>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let mut columns = grid_tracks_property(node, properties, "columns");
    let mut rows = grid_tracks_property(node, properties, "rows");
    let mut placements = grid_placements_property(node, properties, "placements");

    if let Some(resolved) = &placements {
        for child in &node.children {
            if !resolved.contains_key(child.id.as_str()) {
                return Err(invalid_resolved_property(
                    node,
                    "placements",
                    format!("missing child id {:?}", child.id.as_str()),
                ));
            }
        }
        for placement_id in resolved.keys() {
            if !node
                .children
                .iter()
                .any(|child| child.id.as_str() == placement_id)
            {
                return Err(invalid_resolved_property(
                    node,
                    "placements",
                    format!("id {placement_id:?} does not address a child"),
                ));
            }
        }
    }

    let required_columns = placements
        .as_ref()
        .and_then(|placements| grid_required_track_count(placements.values(), false))
        .unwrap_or(1)
        .max(1);
    let column_count = columns
        .as_ref()
        .map_or(required_columns, |columns| columns.len());
    if columns.is_none() {
        columns = Some(vec![UiGridTrack::Fraction { weight: 1 }; required_columns]);
    }

    let automatic_row_count = children.len().div_ceil(column_count.max(1)).max(1);
    let required_rows = placements
        .as_ref()
        .and_then(|placements| grid_required_track_count(placements.values(), true))
        .unwrap_or(automatic_row_count)
        .max(1);
    if rows.is_none() {
        rows = Some(vec![UiGridTrack::Fraction { weight: 1 }; required_rows]);
    }

    let columns = columns.expect("document Grid columns always have a resolved fallback");
    let mut rows = rows.expect("document Grid rows always have a resolved fallback");
    if placements.is_none() {
        let needed_rows = children.len().div_ceil(columns.len().max(1)).max(1);
        rows.resize(
            needed_rows.max(rows.len()),
            UiGridTrack::Fraction { weight: 1 },
        );
        placements = Some(
            node.children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    (
                        child.id.as_str().to_owned(),
                        UiGridPlacement::new(index / columns.len(), index % columns.len()),
                    )
                })
                .collect(),
        );
    }
    let placements = placements.expect("document Grid placements always have a fallback");

    let native_columns = columns
        .into_iter()
        .map(|track| native_grid_track(node, track))
        .collect::<Result<Vec<_>, _>>()?;
    let native_rows = rows
        .into_iter()
        .map(|track| native_grid_track(node, track))
        .collect::<Result<Vec<_>, _>>()?;
    let cells = node
        .children
        .iter()
        .zip(children)
        .map(|(child, content)| {
            let placement = placements
                .get(child.id.as_str())
                .copied()
                .expect("document Grid placement map was checked against child IDs");
            let column_end = placement
                .column
                .checked_add(usize::from(placement.column_span));
            if placement.column >= native_columns.len()
                || column_end.is_none_or(|column_end| column_end > native_columns.len())
            {
                return Err(invalid_resolved_property(
                    node,
                    "placements",
                    format!(
                        "child {:?} exceeds {} declared column(s)",
                        child.id.as_str(),
                        native_columns.len()
                    ),
                ));
            }
            let row_end = placement.row.checked_add(usize::from(placement.row_span));
            if placement.row >= native_rows.len()
                || row_end.is_none_or(|row_end| row_end > native_rows.len())
            {
                return Err(invalid_resolved_property(
                    node,
                    "placements",
                    format!(
                        "child {:?} exceeds {} declared row(s)",
                        child.id.as_str(),
                        native_rows.len()
                    ),
                ));
            }
            let row_span = crate::ZsGridSpan::new(placement.row_span).map_err(|_| {
                invalid_resolved_property(node, "placements", "row_span must be positive")
            })?;
            let column_span = crate::ZsGridSpan::new(placement.column_span).map_err(|_| {
                invalid_resolved_property(node, "placements", "column_span must be positive")
            })?;
            Ok(
                crate::ZsGridCell::new(placement.row, placement.column, content)
                    .row_span(row_span)
                    .column_span(column_span),
            )
        })
        .collect::<Result<Vec<_>, UiDocumentRuntimeError>>()?;

    let mut view = crate::grid(native_columns, native_rows, cells);
    if let Some(gap) = grid_gap_property(node, properties, "column_gap")? {
        view = view.column_gap(Dp::new(gap));
    }
    if let Some(gap) = grid_gap_property(node, properties, "row_gap")? {
        view = view.row_gap(Dp::new(gap));
    }
    Ok(view)
}

#[cfg(feature = "grid")]
fn grid_tracks_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Option<Vec<UiGridTrack>> {
    property_value(node, properties, property).and_then(|value| {
        serde_json::from_value::<Vec<UiGridTrack>>(value)
            .ok()
            .filter(|tracks| {
                !tracks.is_empty() && tracks.iter().copied().all(UiGridTrack::is_valid)
            })
    })
}

#[cfg(feature = "grid")]
fn grid_placements_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Option<BTreeMap<String, UiGridPlacement>> {
    property_value(node, properties, property).and_then(|value| {
        serde_json::from_value::<BTreeMap<String, UiGridPlacement>>(value)
            .ok()
            .filter(|placements| placements.values().copied().all(UiGridPlacement::is_valid))
    })
}

#[cfg(feature = "grid")]
fn grid_required_track_count<'a>(
    placements: impl Iterator<Item = &'a UiGridPlacement>,
    rows: bool,
) -> Option<usize> {
    placements
        .map(|placement| {
            if rows {
                placement.row.checked_add(usize::from(placement.row_span))
            } else {
                placement
                    .column
                    .checked_add(usize::from(placement.column_span))
            }
        })
        .try_fold(0usize, |maximum, end| end.map(|end| maximum.max(end)))
}

#[cfg(feature = "grid")]
fn native_grid_track(
    node: &UiNode,
    track: UiGridTrack,
) -> Result<crate::ZsGridTrack, UiDocumentRuntimeError> {
    match track {
        UiGridTrack::Fixed { size } if size.is_finite() && size >= 0.0 => {
            Ok(crate::ZsGridTrack::fixed(Dp::new(size)))
        }
        UiGridTrack::Fraction { weight } => crate::ZsGridFraction::new(weight)
            .map(crate::ZsGridTrack::fraction)
            .map_err(|_| {
                invalid_resolved_property(node, "columns/rows", "fraction weight must be positive")
            }),
        UiGridTrack::Fixed { .. } => Err(invalid_resolved_property(
            node,
            "columns/rows",
            "fixed size must be finite and non-negative",
        )),
    }
}

#[cfg(feature = "grid")]
fn grid_gap_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<f32>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property).and_then(|value| value.as_f64())
    else {
        return Ok(None);
    };
    if !value.is_finite() || value < 0.0 || value > f64::from(f32::MAX) {
        return Err(invalid_resolved_property(
            node,
            property,
            "gap must be finite and non-negative",
        ));
    }
    Ok(Some(value as f32))
}

#[cfg(any(
    feature = "badge",
    feature = "split-view",
    feature = "canvas",
    feature = "label",
    feature = "button",
    feature = "icon",
    feature = "breadcrumb",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "grid",
    feature = "list",
    feature = "progress-ring",
    feature = "password-box",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "auto-suggest",
    feature = "command-palette",
    feature = "dialog",
    feature = "tree",
    feature = "grid-view",
    feature = "table",
    feature = "tooltip",
    feature = "teaching-tip"
))]
fn invalid_resolved_property(
    node: &UiNode,
    property: impl Into<String>,
    reason: impl Into<String>,
) -> UiDocumentRuntimeError {
    UiDocumentRuntimeError::InvalidResolvedProperty {
        node_id: node.id.as_str().to_owned(),
        property: property.into(),
        reason: reason.into(),
    }
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
    } else if let Some(token) = node.layout.padding_token {
        view = view.padding(token.resolve());
    }
    if let Some(value) = node.layout.gap {
        view = view.gap(Dp::new(value));
    } else if let Some(token) = node.layout.gap_token {
        view = view.gap(token.resolve());
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
    feature = "badge",
    feature = "split-view",
    feature = "canvas",
    feature = "label",
    feature = "button",
    feature = "icon",
    feature = "breadcrumb",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "toggle-button",
    feature = "checkbox",
    feature = "toggle",
    feature = "textbox",
    feature = "password-box",
    feature = "radio",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "auto-suggest",
    feature = "command-palette",
    feature = "tree",
    feature = "grid-view",
    feature = "table",
    feature = "list",
    feature = "tabs",
    feature = "grid",
    feature = "progress",
    feature = "progress-ring",
    feature = "scroll",
    feature = "tooltip",
    feature = "teaching-tip"
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

#[cfg(feature = "date-picker")]
fn date_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ZsDate>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(invalid_resolved_property(
            node,
            property,
            "date must be a canonical YYYY-MM-DD string",
        ));
    };
    crate::ZsDate::parse_iso(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "time-picker")]
fn time_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ZsTime>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(invalid_resolved_property(
            node,
            property,
            "time must be a canonical HH:MM 24-hour string",
        ));
    };
    crate::ZsTime::parse_24_hour(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "color-picker")]
fn color_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::Color>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(invalid_resolved_property(
            node,
            property,
            "color must be a canonical #RRGGBBAA string",
        ));
    };
    crate::Color::parse_hex_rgba(value)
        .map(Some)
        .ok_or_else(|| {
            invalid_resolved_property(
                node,
                property,
                "color must use canonical uppercase #RRGGBBAA",
            )
        })
}

#[cfg(feature = "auto-suggest")]
fn auto_suggestions_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiAutoSuggestion>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get("suggestions")
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(Vec::new());
    }
    let value = property_value(node, properties, "suggestions").ok_or_else(|| {
        invalid_resolved_property(node, "suggestions", "a suggestion array is required")
    })?;
    crate::ui_document::ui_auto_suggestions_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "suggestions",
            "suggestions must use unique stable ids and string text values",
        )
    })
}

#[cfg(feature = "auto-suggest")]
fn optional_auto_suggestion_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiAutoSuggestionId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(None);
    }
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "suggestion id must be a string or null")
    })?;
    crate::ui_document::UiAutoSuggestionId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "command-palette")]
fn command_palette_items_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiCommandPaletteItem>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get("items")
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(Vec::new());
    }
    let value = property_value(node, properties, "items").ok_or_else(|| {
        invalid_resolved_property(node, "items", "a command item array is required")
    })?;
    crate::ui_document::ui_command_palette_items_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "items",
            "command items must use unique stable ids, non-empty titles and valid metadata",
        )
    })
}

#[cfg(feature = "command-palette")]
fn optional_command_palette_item_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiCommandPaletteItemId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(None);
    }
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "command item id must be a string or null")
    })?;
    crate::ui_document::UiCommandPaletteItemId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "tree")]
fn tree_nodes_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiTreeNode>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get("nodes")
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(Vec::new());
    }
    let value = property_value(node, properties, "nodes")
        .ok_or_else(|| invalid_resolved_property(node, "nodes", "a tree node array is required"))?;
    crate::ui_document::ui_tree_nodes_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "nodes",
            "tree nodes must use globally unique stable ids and non-empty labels",
        )
    })
}

#[cfg(feature = "tree")]
fn tree_node_ids_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<BTreeSet<crate::ui_document::UiTreeNodeId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(BTreeSet::new());
    }
    let value = property_value(node, properties, property).ok_or_else(|| {
        invalid_resolved_property(node, property, "a tree node id array is required")
    })?;
    crate::ui_document::ui_tree_node_ids_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            property,
            "tree node id arrays must contain unique stable string ids",
        )
    })
}

#[cfg(feature = "tree")]
fn optional_tree_node_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiTreeNodeId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(None);
    }
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "tree node id must be a string or null")
    })?;
    crate::ui_document::UiTreeNodeId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "document-shell")]
fn document_string_id_array(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Vec<String>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(Vec::new());
    };
    let values = value.as_array().ok_or_else(|| {
        invalid_resolved_property(node, property, "stable child IDs must be a string array")
    })?;
    let mut ids = Vec::with_capacity(values.len());
    let mut seen = BTreeSet::new();
    for value in values {
        let id = value.as_str().ok_or_else(|| {
            invalid_resolved_property(node, property, "stable child IDs must be strings")
        })?;
        if !seen.insert(id.to_owned()) {
            return Err(invalid_resolved_property(
                node,
                property,
                format!("stable child id {id:?} is duplicated"),
            ));
        }
        ids.push(id.to_owned());
    }
    Ok(ids)
}

#[cfg(feature = "shell")]
fn navigation_items_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
    required_non_empty: bool,
) -> Result<Vec<crate::ui_document::UiNavigationItem>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return if required_non_empty {
            Err(invalid_resolved_property(
                node,
                property,
                "a non-empty navigation item array is required",
            ))
        } else {
            Ok(Vec::new())
        };
    }
    let Some(value) = property_value(node, properties, property) else {
        return if required_non_empty {
            Err(invalid_resolved_property(
                node,
                property,
                "a non-empty navigation item array is required",
            ))
        } else {
            Ok(Vec::new())
        };
    };
    let items = crate::ui_document::ui_navigation_items_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            property,
            "navigation items must use unique stable ids and non-empty labels",
        )
    })?;
    if required_non_empty && items.is_empty() {
        return Err(invalid_resolved_property(
            node,
            property,
            "a non-empty navigation item array is required",
        ));
    }
    Ok(items)
}

#[cfg(feature = "shell")]
fn optional_navigation_item_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiNavigationItemId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(None);
    }
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(
            node,
            property,
            "navigation item id must be a string or null",
        )
    })?;
    crate::ui_document::UiNavigationItemId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(any(feature = "shell", feature = "document-shell"))]
fn optional_non_negative_extent(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<f32>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    let value = value.as_f64().ok_or_else(|| {
        invalid_resolved_property(node, property, "extent must be a finite number")
    })?;
    if !value.is_finite() || value < 0.0 || value > f64::from(f32::MAX) {
        return Err(invalid_resolved_property(
            node,
            property,
            "extent must be a finite non-negative DP value",
        ));
    }
    Ok(Some(value as f32))
}

#[cfg(feature = "grid-view")]
fn grid_view_items_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiGridViewItem>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get("items")
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(Vec::new());
    }
    let value = property_value(node, properties, "items").ok_or_else(|| {
        invalid_resolved_property(node, "items", "a grid-view item array is required")
    })?;
    crate::ui_document::ui_grid_view_items_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "items",
            "grid-view items must use unique stable ids, non-empty titles and valid metadata",
        )
    })
}

#[cfg(feature = "table")]
fn table_columns_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiTableColumn>, UiDocumentRuntimeError> {
    let value = property_value(node, properties, "columns").ok_or_else(|| {
        invalid_resolved_property(
            node,
            "columns",
            "a non-empty table column array is required",
        )
    })?;
    crate::ui_document::ui_table_columns_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "columns",
            "table columns must use unique stable ids, non-empty headers and valid widths",
        )
    })
}

#[cfg(feature = "table")]
fn table_rows_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiTableRow>, UiDocumentRuntimeError> {
    let value = property_value(node, properties, "rows")
        .ok_or_else(|| invalid_resolved_property(node, "rows", "a table row array is required"))?;
    crate::ui_document::ui_table_rows_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "rows",
            "table rows must use unique stable ids and valid column-id cell maps",
        )
    })
}

#[cfg(feature = "breadcrumb")]
fn breadcrumb_items_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<Vec<crate::ui_document::UiBreadcrumbItem>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get("items")
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(Vec::new());
    }
    let value = property_value(node, properties, "items").ok_or_else(|| {
        invalid_resolved_property(node, "items", "a breadcrumb item array is required")
    })?;
    crate::ui_document::ui_breadcrumb_items_from_value(&value).ok_or_else(|| {
        invalid_resolved_property(
            node,
            "items",
            "breadcrumb items must be non-empty and use unique stable ids and non-empty labels",
        )
    })
}

#[cfg(feature = "grid-view")]
fn optional_grid_view_item_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiGridViewItemId>, UiDocumentRuntimeError> {
    if node
        .property_bindings
        .get(property)
        .is_some_and(|binding| !properties.contains_key(binding))
    {
        return Ok(None);
    }
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "grid-view item id must be a string or null")
    })?;
    crate::ui_document::UiGridViewItemId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "table")]
fn optional_table_row_id_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiTableRowId>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "table row id must be a string or null")
    })?;
    crate::ui_document::UiTableRowId::new(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(feature = "table")]
fn optional_table_sort_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ui_document::UiTableSort>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value::<crate::ui_document::UiTableSort>(value)
        .map(Some)
        .map_err(|error| invalid_resolved_property(node, property, error.to_string()))
}

#[cfg(any(
    feature = "badge",
    feature = "split-view",
    feature = "auto-suggest",
    feature = "command-palette",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "label",
    feature = "icon",
    feature = "button",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tooltip",
    feature = "teaching-tip",
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

#[cfg(any(feature = "badge", feature = "button", feature = "icon"))]
fn optional_semantic_icon_property(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    property: &str,
) -> Result<Option<crate::ZsIcon>, UiDocumentRuntimeError> {
    let Some(value) = property_value(node, properties, property) else {
        return Ok(None);
    };
    let icon = value.as_str().ok_or_else(|| {
        invalid_resolved_property(node, property, "semantic icon must be a string")
    })?;
    serde_json::from_value::<crate::ZsIcon>(Value::String(icon.to_owned()))
        .map(Some)
        .map_err(|_| {
            invalid_resolved_property(
                node,
                property,
                format!("unknown ZsIcon semantic variant {icon:?}"),
            )
        })
}

#[cfg(feature = "split-view")]
fn document_split_view<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    children: Vec<ViewNode<Msg>>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let actual = children.len();
    let mut children = children.into_iter();
    let Some(pane) = children.next() else {
        return Err(UiDocumentRuntimeError::InvalidChildCount {
            component: node.component.clone(),
            expected: 2,
            actual,
        });
    };
    let Some(content) = children.next() else {
        return Err(UiDocumentRuntimeError::InvalidChildCount {
            component: node.component.clone(),
            expected: 2,
            actual,
        });
    };
    if children.next().is_some() {
        return Err(UiDocumentRuntimeError::InvalidChildCount {
            component: node.component.clone(),
            expected: 2,
            actual,
        });
    }

    let mode = match string_property(node, properties, "mode", "adaptive").as_str() {
        "adaptive" => crate::ZsSplitViewDisplayMode::Adaptive,
        "inline" => crate::ZsSplitViewDisplayMode::Inline,
        "overlay" => crate::ZsSplitViewDisplayMode::Overlay,
        value => {
            return Err(invalid_resolved_property(
                node,
                "mode",
                format!("unsupported split_view mode {value:?}"),
            ));
        }
    };
    let placement = match string_property(node, properties, "pane_placement", "leading").as_str() {
        "leading" => crate::ZsSplitViewPanePlacement::Leading,
        "trailing" => crate::ZsSplitViewPanePlacement::Trailing,
        value => {
            return Err(invalid_resolved_property(
                node,
                "pane_placement",
                format!("unsupported split_view pane placement {value:?}"),
            ));
        }
    };
    let extent = |property: &str, allow_zero: bool| -> Result<Option<Dp>, UiDocumentRuntimeError> {
        let Some(value) = property_value(node, properties, property) else {
            return Ok(None);
        };
        let value = value.as_f64().ok_or_else(|| {
            invalid_resolved_property(node, property, "a numeric DP extent is required")
        })?;
        if !value.is_finite()
            || value > f64::from(f32::MAX)
            || if allow_zero {
                value < 0.0
            } else {
                value <= 0.0
            }
        {
            return Err(invalid_resolved_property(
                node,
                property,
                if allow_zero {
                    "a finite nonnegative DP extent is required"
                } else {
                    "a finite positive DP extent is required"
                },
            ));
        }
        Ok(Some(Dp::new(value as f32)))
    };
    let mut spec = crate::ZsSplitViewSpec::new(bool_property(node, properties, "open", true))
        .display_mode(mode)
        .pane_placement(placement);
    if let Some(width) = extent("pane_width", false)? {
        spec = spec.pane_width(width);
    }
    if let Some(width) = extent("minimum_content_width", true)? {
        spec = spec.minimum_content_width(width);
    }
    let mut control = crate::split_view(node.id.widget_id(), spec, pane, content);
    if let Some(binding) = node.action_bindings.get("open_change") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        let property_binding = node.property_bindings.get("open").cloned();
        control = control.on_split_view_open_change_with(move |open| {
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: property_binding.clone(),
                payload: Value::Bool(open),
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "canvas")]
fn document_canvas<Msg: Clone + 'static>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
    mapper: &UiDocumentActionMapper<Msg>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let value = property_value(node, properties, "primitives").ok_or_else(|| {
        invalid_resolved_property(node, "primitives", "a Canvas primitive array is required")
    })?;
    let primitives =
        crate::ui_document::ui_canvas_primitives_from_value(&value).ok_or_else(|| {
            invalid_resolved_property(
                node,
                "primitives",
                "Canvas primitives must use finite local-DP geometry and semantic theme roles",
            )
        })?;
    let mut control = crate::canvas(crate::ui_document::ui_canvas_native_scene(primitives));
    if let Some(binding) = node.action_bindings.get("activate") {
        control = control.on_click(mapper.map(UiDocumentAction {
            node_id: node.id.as_str().to_owned(),
            binding: binding.clone(),
            property_binding: None,
            payload: Value::Null,
        }));
    }
    if let Some(binding) = node.action_bindings.get("pointer") {
        let mapper = mapper.clone();
        let node_id = node.id.as_str().to_owned();
        let binding = binding.clone();
        control = control.on_canvas_pointer_with(move |event| {
            let payload =
                serde_json::to_value(crate::ui_document::UiCanvasPointerEvent::from_native(event))
                    .expect("Canvas pointer event must serialize");
            mapper.map(UiDocumentAction {
                node_id: node_id.clone(),
                binding: binding.clone(),
                property_binding: None,
                payload,
            })
        });
    }
    Ok(control)
}

#[cfg(feature = "badge")]
fn document_badge<Msg>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let content = match string_property(node, properties, "kind", "").as_str() {
        "dot" => crate::ZsBadgeContent::Dot,
        "number" => {
            let value = property_value(node, properties, "value")
                .and_then(|value| value.as_u64())
                .ok_or_else(|| {
                    invalid_resolved_property(
                        node,
                        "value",
                        "number badges require a nonnegative integer value",
                    )
                })?;
            let value = u32::try_from(value).map_err(|_| {
                invalid_resolved_property(
                    node,
                    "value",
                    "badge value must fit in an unsigned 32-bit integer",
                )
            })?;
            crate::ZsBadgeContent::Number(value)
        }
        "icon" => crate::ZsBadgeContent::Icon(
            optional_semantic_icon_property(node, properties, "icon")?.ok_or_else(|| {
                invalid_resolved_property(node, "icon", "icon badges require a semantic ZsIcon")
            })?,
        ),
        value => {
            return Err(invalid_resolved_property(
                node,
                "kind",
                format!("unsupported badge kind {value:?}"),
            ));
        }
    };
    let tone = match string_property(node, properties, "tone", "accent").as_str() {
        "neutral" => crate::ZsBadgeTone::Neutral,
        "accent" => crate::ZsBadgeTone::Accent,
        "success" => crate::ZsBadgeTone::Success,
        "warning" => crate::ZsBadgeTone::Warning,
        "danger" => crate::ZsBadgeTone::Danger,
        value => {
            return Err(invalid_resolved_property(
                node,
                "tone",
                format!("unsupported badge tone {value:?}"),
            ));
        }
    };
    Ok(crate::badge(content).badge_tone(tone))
}

#[cfg(feature = "icon")]
fn document_icon<Msg>(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<ViewNode<Msg>, UiDocumentRuntimeError> {
    let icon = optional_semantic_icon_property(node, properties, "icon")?.ok_or_else(|| {
        invalid_resolved_property(node, "icon", "a semantic ZsIcon value is required")
    })?;
    let size = match string_property(node, properties, "size", "standard").as_str() {
        "small" => crate::ZsIconSize::Small,
        "standard" => crate::ZsIconSize::Standard,
        "large" => crate::ZsIconSize::Large,
        value => {
            return Err(invalid_resolved_property(
                node,
                "size",
                format!("unsupported semantic icon size {value:?}"),
            ));
        }
    };
    let bound_color = string_property(node, properties, "color", "primary");
    let color = match node
        .theme_tokens
        .get("foreground")
        .map(String::as_str)
        .unwrap_or(bound_color.as_str())
    {
        "primary" => ColorRole::PrimaryText,
        "secondary" => ColorRole::SecondaryText,
        "disabled" => ColorRole::DisabledText,
        "accent" => ColorRole::Accent,
        "accent_text" => ColorRole::AccentText,
        "surface" => ColorRole::Surface,
        "surface_raised" => ColorRole::SurfaceRaised,
        "control" => ColorRole::Control,
        "border" => ColorRole::Border,
        "success" => ColorRole::Success,
        "warning" => ColorRole::Warning,
        "danger" => ColorRole::Danger,
        token if node.theme_tokens.contains_key("foreground") => {
            color_role(token).ok_or_else(|| {
                invalid_resolved_property(
                    node,
                    "foreground",
                    "unsupported icon foreground theme token",
                )
            })?
        }
        value => {
            return Err(invalid_resolved_property(
                node,
                "color",
                format!("unsupported semantic icon color {value:?}"),
            ));
        }
    };
    Ok(crate::icon(icon).icon_size(size).icon_color(color))
}

#[cfg(any(
    feature = "label",
    feature = "split-view",
    feature = "button",
    feature = "breadcrumb",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "toggle-button",
    feature = "checkbox",
    feature = "toggle",
    feature = "textbox",
    feature = "radio",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "auto-suggest",
    feature = "command-palette",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "progress-ring",
    feature = "teaching-tip"
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
    feature = "progress-ring",
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

#[cfg(any(feature = "number-box", feature = "progress-ring"))]
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

#[cfg(any(
    feature = "label",
    feature = "combo",
    feature = "tabs",
    feature = "list",
    feature = "progress-ring",
    feature = "password-box",
    feature = "time-picker",
    feature = "color-picker",
    feature = "auto-suggest",
    feature = "command-palette",
    feature = "dialog",
    feature = "flyout",
    feature = "toast",
    feature = "info-bar",
    feature = "tooltip",
    feature = "teaching-tip"
))]
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
fn semantic_text_style(
    node: &UiNode,
    properties: &BTreeMap<String, Value>,
) -> Result<SemanticTextStyle, UiDocumentRuntimeError> {
    let role = match optional_string_property(node, properties, "text_role").as_deref() {
        None | Some("body") => TextRole::Body,
        Some("caption") => TextRole::Caption,
        Some("body_large") => TextRole::BodyLarge,
        Some("subtitle") => TextRole::Subtitle,
        Some("title") => TextRole::Title,
        Some("title_large") => TextRole::TitleLarge,
        Some("display") => TextRole::Display,
        Some(_) => {
            return Err(invalid_resolved_property(
                node,
                "text_role",
                "text_role must be body, caption, body_large, subtitle, title, title_large or display",
            ));
        }
    };
    let mut style = SemanticTextStyle::for_role(role);
    style.wrap = match optional_string_property(node, properties, "wrap").as_deref() {
        None | Some("no_wrap") => crate::TextWrap::NoWrap,
        Some("word") => crate::TextWrap::Word,
        Some(_) => {
            return Err(invalid_resolved_property(
                node,
                "wrap",
                "wrap must be no_wrap or word",
            ));
        }
    };
    style.ellipsis = property_value(node, properties, "ellipsis")
        .and_then(|value| value.as_bool())
        .unwrap_or(style.wrap == crate::TextWrap::NoWrap);
    style.weight = match optional_string_property(node, properties, "weight").as_deref() {
        None | Some("automatic") => crate::TextWeight::Automatic,
        Some("regular") => crate::TextWeight::Regular,
        Some("medium") => crate::TextWeight::Medium,
        Some("semibold") => crate::TextWeight::Semibold,
        Some("bold") => crate::TextWeight::Bold,
        Some(_) => {
            return Err(invalid_resolved_property(
                node,
                "weight",
                "weight must be automatic, regular, medium, semibold or bold",
            ));
        }
    };
    style.horizontal_align =
        match optional_string_property(node, properties, "horizontal_align").as_deref() {
            None | Some("start") => crate::HorizontalAlign::Start,
            Some("center") => crate::HorizontalAlign::Center,
            Some("end") => crate::HorizontalAlign::End,
            Some(_) => {
                return Err(invalid_resolved_property(
                    node,
                    "horizontal_align",
                    "horizontal_align must be start, center or end",
                ));
            }
        };
    style.vertical_align =
        match optional_string_property(node, properties, "vertical_align").as_deref() {
            None | Some("center") => crate::VerticalAlign::Center,
            Some("start") => crate::VerticalAlign::Start,
            Some("end") => crate::VerticalAlign::End,
            Some(_) => {
                return Err(invalid_resolved_property(
                    node,
                    "vertical_align",
                    "vertical_align must be start, center or end",
                ));
            }
        };
    if let Some(color) = node
        .theme_tokens
        .get("foreground")
        .and_then(|token| color_role(token))
    {
        style.color = color;
    }
    Ok(style)
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

#[cfg(any(feature = "icon", feature = "label"))]
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

    #[cfg(any(
        feature = "auto-suggest",
        feature = "canvas",
        feature = "label",
        feature = "date-picker",
        feature = "time-picker",
        feature = "color-picker",
        feature = "command-palette",
        feature = "breadcrumb",
        feature = "flyout",
        feature = "menu-flyout",
        feature = "tree",
        feature = "grid-view",
        feature = "table",
        feature = "grid",
        feature = "list",
        feature = "password-box",
        all(feature = "tooltip", feature = "button"),
        all(feature = "teaching-tip", feature = "button"),
        all(feature = "label", feature = "button")
    ))]
    use crate::View;
    #[cfg(any(
        feature = "grid",
        feature = "canvas",
        feature = "label",
        feature = "split-view",
        feature = "flyout",
        feature = "menu-flyout",
        all(feature = "tooltip", feature = "button"),
        all(feature = "teaching-tip", feature = "button")
    ))]
    use crate::{Dpi, Rect, ViewLayoutCx};

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Action(UiDocumentAction),
        #[cfg(feature = "password-box")]
        Secret(UiDocumentSecretAction),
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

    #[cfg(feature = "icon")]
    #[test]
    fn compiles_bound_semantic_icon_without_platform_values() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "status-icon",
                "component": "icon",
                "property_bindings": {
                  "icon": "status_symbol",
                  "size": "status_size",
                  "color": "status_color"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "status_symbol".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "status_size".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "status_color".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::new(),
        };
        let values = BTreeMap::from([
            (
                "status_symbol".to_owned(),
                Value::String("Success".to_owned()),
            ),
            ("status_size".to_owned(), Value::String("large".to_owned())),
            (
                "status_color".to_owned(),
                Value::String("success".to_owned()),
            ),
        ]);

        let view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            view.kind,
            crate::ViewNodeKind::Icon {
                icon: crate::ZsIcon::Success,
                size: crate::ZsIconSize::Large,
                color: ColorRole::Success,
            }
        ));
        assert_eq!(view.style.flex, 0.0);

        let mut invalid_values = values;
        invalid_values.insert("status_size".to_owned(), Value::String("huge".to_owned()));
        assert!(ui_document_view(&document, &bindings, &invalid_values, Msg::Action).is_err());
    }

    #[cfg(feature = "badge")]
    #[test]
    fn compiles_bound_numeric_badge_with_semantic_tone() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "unread-count",
                "component": "badge",
                "properties": { "kind": "number" },
                "property_bindings": {
                  "value": "unread_count",
                  "tone": "unread_tone"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "unread_count".to_owned(),
                    crate::ui_document::UiValueType::Integer,
                ),
                (
                    "unread_tone".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::new(),
        };
        let values = BTreeMap::from([
            ("unread_count".to_owned(), Value::from(42u64)),
            (
                "unread_tone".to_owned(),
                Value::String("success".to_owned()),
            ),
        ]);

        let view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            view.kind,
            crate::ViewNodeKind::Badge {
                content: crate::ZsBadgeContent::Number(42),
                tone: crate::ZsBadgeTone::Success,
            }
        ));
        assert_eq!(view.style.flex, 0.0);

        let mut invalid_values = values;
        invalid_values.insert("unread_count".to_owned(), Value::from(u64::MAX));
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "value"
        ));
    }

    #[cfg(all(feature = "split-view", feature = "label"))]
    #[test]
    fn compiles_bound_split_view_and_maps_controlled_light_dismissal() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "workspace-split",
                "component": "split_view",
                "properties": {
                  "pane_placement": "trailing",
                  "minimum_content_width": 360
                },
                "property_bindings": {
                  "open": "workspace_open",
                  "mode": "workspace_mode",
                  "pane_width": "workspace_pane_width"
                },
                "action_bindings": { "open_change": "workspace_open_changed" },
                "children": [
                  { "id": "workspace-pane", "component": "text", "properties": { "text": "Pane" } },
                  { "id": "workspace-content", "component": "text", "properties": { "text": "Content" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "workspace_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "workspace_mode".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "workspace_pane_width".to_owned(),
                    crate::ui_document::UiValueType::Number,
                ),
            ]),
            actions: BTreeMap::from([(
                "workspace_open_changed".to_owned(),
                crate::ui_document::UiValueType::Boolean,
            )]),
        };
        let values = BTreeMap::from([
            ("workspace_open".to_owned(), Value::Bool(true)),
            (
                "workspace_mode".to_owned(),
                Value::String("overlay".to_owned()),
            ),
            ("workspace_pane_width".to_owned(), Value::from(220.0)),
        ]);

        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action)
            .unwrap()
            .with_platform_style_override(crate::ZsBaseControlPlatformStyle::Windows);
        assert!(matches!(
            view.kind,
            crate::ViewNodeKind::SplitView { spec, .. }
                if spec.open()
                    && spec.mode() == crate::ZsSplitViewDisplayMode::Overlay
                    && spec.placement() == crate::ZsSplitViewPanePlacement::Trailing
                    && spec.requested_pane_width() == Some(Dp::new(220.0))
        ));
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 560,
                height: 400,
            },
            Dpi::standard(),
        ));
        assert_eq!(view.children[0].bounds().unwrap().x, 340);
        assert_eq!(view.children[1].bounds().unwrap().width, 560);

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
                node_id: "workspace-split".to_owned(),
                binding: "workspace_open_changed".to_owned(),
                property_binding: Some("workspace_open".to_owned()),
                payload: Value::Bool(false),
            })]
        );

        let mut invalid_values = values.clone();
        invalid_values.insert(
            "workspace_mode".to_owned(),
            Value::String("compact".to_owned()),
        );
        assert!(ui_document_view(&document, &bindings, &invalid_values, Msg::Action).is_err());
        invalid_values = values;
        invalid_values.insert("workspace_pane_width".to_owned(), Value::from(0.0));
        assert!(ui_document_view(&document, &bindings, &invalid_values, Msg::Action).is_err());
    }

    #[cfg(feature = "canvas")]
    #[test]
    fn compiles_bound_canvas_scene_and_maps_typed_pointer_payload() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "chart",
                "component": "canvas",
                "layout": { "width": 320, "height": 180 },
                "property_bindings": { "primitives": "chart_scene" },
                "action_bindings": {
                  "activate": "chart_activated",
                  "pointer": "chart_pointer"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "chart_scene".to_owned(),
                crate::ui_document::UiValueType::CanvasPrimitiveArray,
            )]),
            actions: BTreeMap::from([
                (
                    "chart_activated".to_owned(),
                    crate::ui_document::UiValueType::Null,
                ),
                (
                    "chart_pointer".to_owned(),
                    crate::ui_document::UiValueType::CanvasPointerEvent,
                ),
            ]),
        };
        let scene = serde_json::json!([
            {
                "kind": "round_fill",
                "rect": { "x": 4, "y": 6, "width": 80, "height": 32 },
                "fill": { "role": "accent", "alpha": 224 },
                "radius": 6
            },
            {
                "kind": "text",
                "text": "图表 / Chart",
                "rect": { "x": 96, "y": 6, "width": 180, "height": 32 },
                "style": { "role": "subtitle", "weight": "semibold" }
            }
        ]);
        let values = BTreeMap::from([("chart_scene".to_owned(), scene)]);

        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            &view.kind,
            crate::ViewNodeKind::Canvas { scene, .. } if scene.primitive_count() == 2
        ));
        let bounds = Rect {
            x: 20,
            y: 30,
            width: 320,
            height: 180,
        };
        view.layout(&mut ViewLayoutCx::new(bounds, Dpi::standard()));
        let mut paint = crate::ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            crate::NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: 24,
                    y: 36,
                    width: 80,
                    height: 32
                },
                ..
            }
        )));

        let widget = document.root.id.widget_id();
        let pointer = crate::ZsCanvasPointerEvent::new(
            widget,
            crate::ZsCanvasPointerPhase::Moved,
            crate::ZsCanvasPoint::new(Dp::new(-4.0), Dp::new(48.0)),
            crate::ZsPointerButton::Secondary,
            crate::ZsPointerModifiers::new(true, false, false, false),
            false,
        );
        let mut events = crate::ViewEventCx::new();
        view.event(&mut events, &crate::ViewEvent::Click { widget });
        view.event(
            &mut events,
            &crate::ViewEvent::CanvasPointer { event: pointer },
        );
        let messages = events.into_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            Msg::Action(UiDocumentAction {
                node_id: "chart".to_owned(),
                binding: "chart_activated".to_owned(),
                property_binding: None,
                payload: Value::Null,
            })
        );
        assert!(matches!(
            &messages[1],
            Msg::Action(UiDocumentAction { node_id, binding, payload, .. })
                if node_id == "chart"
                    && binding == "chart_pointer"
                    && payload["phase"] == "moved"
                    && payload["button"]["kind"] == "secondary"
                    && payload["position"]["x"] == -4.0
                    && payload["inside"] == false
        ));

        let invalid_values = BTreeMap::from([(
            "chart_scene".to_owned(),
            serde_json::json!([{
                "kind": "round_fill",
                "rect": { "x": 0, "y": 0, "width": 20, "height": 20 },
                "fill": { "role": "accent" },
                "radius": -1
            }]),
        )]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::Validation { diagnostics })
                if diagnostics.iter().any(|diagnostic|
                    diagnostic.code
                        == crate::ui_document::UiDiagnosticCode::BindingValueTypeMismatch)
        ));
    }

    #[cfg(feature = "label")]
    #[test]
    fn compiles_platform_spacing_tokens_into_real_page_geometry() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "page",
                "component": "stack",
                "layout": {
                  "padding_token": "page_padding",
                  "gap_token": "content_gap"
                },
                "children": [
                  {
                    "id": "title",
                    "component": "text",
                    "properties": { "text": "页面标题 / Page title" }
                  },
                  {
                    "id": "body",
                    "component": "text",
                    "properties": { "text": "页面内容 / Page content" }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let mut view = ui_document_view(
            &document,
            &UiBindingSchema::default(),
            &BTreeMap::new(),
            Msg::Action,
        )
        .unwrap();
        let spacing = crate::ZsuiSpacingTokens::default();
        assert_eq!(view.style.padding, Some(spacing.page_padding));
        assert_eq!(view.style.gap, Some(spacing.content_gap));

        let output = view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 240,
            },
            Dpi::standard(),
        ));
        let title = output
            .children
            .iter()
            .find(|node| node.component == document.root.children[0].id.widget_id().into())
            .unwrap()
            .bounds;
        let body = output
            .children
            .iter()
            .find(|node| node.component == document.root.children[1].id.widget_id().into())
            .unwrap()
            .bounds;
        let padding = spacing.page_padding.to_px(Dpi::standard()).round_i32();
        let gap = spacing.content_gap.to_px(Dpi::standard()).round_i32();
        assert_eq!((title.x, title.y), (padding, padding));
        assert_eq!(body.y, title.y + title.height + gap);
    }

    #[cfg(feature = "label")]
    #[test]
    fn compiles_wrapped_text_style_and_rejects_invalid_resolved_enums() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "description",
                "component": "text",
                "properties": {
                  "text": "中文说明需要完整换行 / This bilingual description must wrap without compression.",
                  "ellipsis": false,
                  "horizontal_align": "start",
                  "vertical_align": "start"
                },
                "property_bindings": { "wrap": "description_wrap" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "description_wrap".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
            actions: BTreeMap::new(),
        };
        let view = ui_document_view(
            &document,
            &bindings,
            &BTreeMap::from([(
                "description_wrap".to_owned(),
                Value::String("word".to_owned()),
            )]),
            Msg::Action,
        )
        .unwrap();
        let crate::ViewNodeKind::Text { style, .. } = &view.kind else {
            panic!("text document must compile to a text View node");
        };
        assert_eq!(style.wrap, crate::TextWrap::Word);
        assert!(!style.ellipsis);
        assert_eq!(style.vertical_align, crate::VerticalAlign::Start);
        let widget = view.id.unwrap();
        let mut page = crate::column([view]);
        let output = page.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 20,
            },
            Dpi::standard(),
        ));
        let text_bounds = output
            .children
            .iter()
            .find(|node| node.component == widget.into())
            .unwrap()
            .bounds;
        assert!(
            text_bounds.height
                > crate::TextRole::Body
                    .metrics_for(crate::ZsTypographyPlatformStyle::current())
                    .line_height as i32
        );

        let invalid_values = BTreeMap::from([(
            "description_wrap".to_owned(),
            Value::String("compress".to_owned()),
        )]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "wrap"
        ));
    }

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn compiles_controlled_auto_suggest_and_emits_semantic_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "country-search",
                "component": "auto_suggest",
                "properties": {
                  "placeholder": "Search countries",
                  "no_results_text": "No matches",
                  "query_icon": true
                },
                "property_bindings": {
                  "suggestions": "country_suggestions",
                  "query": "country_query",
                  "highlighted": "country_highlighted",
                  "expanded": "country_expanded"
                },
                "action_bindings": {
                  "text_change": "country_query_changed",
                  "choose": "country_chosen",
                  "submit": "country_submitted",
                  "expanded_change": "country_expanded_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "country_suggestions".to_owned(),
                    crate::ui_document::UiValueType::AutoSuggestionArray,
                ),
                (
                    "country_query".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "country_highlighted".to_owned(),
                    crate::ui_document::UiValueType::NullableAutoSuggestionId,
                ),
                (
                    "country_expanded".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "country_query_changed".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "country_chosen".to_owned(),
                    crate::ui_document::UiValueType::AutoSuggestionId,
                ),
                (
                    "country_submitted".to_owned(),
                    crate::ui_document::UiValueType::AutoSuggestSubmission,
                ),
                (
                    "country_expanded_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "country_suggestions".to_owned(),
                serde_json::json!([
                    { "id": "china", "text": "China" },
                    { "id": "chile", "text": "Chile" },
                    { "id": "chicago", "text": "Chicago" }
                ]),
            ),
            ("country_query".to_owned(), Value::String("Ch".to_owned())),
            ("country_highlighted".to_owned(), Value::Null),
            ("country_expanded".to_owned(), Value::Bool(true)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let china_author = crate::ui_document::UiAutoSuggestionId::new("china").unwrap();
        let china =
            crate::ui_document::ui_auto_suggestion_runtime_id(&document.root.id, &china_author);
        assert_eq!(
            view.widget_auto_suggest_state(widget),
            Some(crate::ZsAutoSuggestState {
                query: "Ch".to_owned(),
                suggestion_ids: vec![
                    china,
                    crate::ui_document::ui_auto_suggestion_runtime_id(
                        &document.root.id,
                        &crate::ui_document::UiAutoSuggestionId::new("chile").unwrap(),
                    ),
                    crate::ui_document::ui_auto_suggestion_runtime_id(
                        &document.root.id,
                        &crate::ui_document::UiAutoSuggestionId::new("chicago").unwrap(),
                    ),
                ],
                highlighted: None,
                expanded: true,
            })
        );

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::AutoSuggestHighlighted {
                widget,
                suggestion: china,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(query), Msg::Action(chosen)] = messages.as_slice() else {
            panic!("highlighting must emit query and stable-id actions");
        };
        assert_eq!(query.binding, "country_query_changed");
        assert_eq!(query.property_binding.as_deref(), Some("country_query"));
        assert_eq!(query.payload, Value::String("China".to_owned()));
        assert_eq!(chosen.binding, "country_chosen");
        assert_eq!(
            chosen.property_binding.as_deref(),
            Some("country_highlighted")
        );
        assert_eq!(chosen.payload, Value::String("china".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::AutoSuggestSubmitted {
                widget,
                suggestion: Some(china),
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(submitted), Msg::Action(expanded)] = messages.as_slice() else {
            panic!("submission must emit structured submit and controlled close actions");
        };
        assert_eq!(submitted.binding, "country_submitted");
        assert_eq!(submitted.property_binding, None);
        assert_eq!(
            submitted.payload,
            serde_json::json!({ "query": "China", "chosen": "china" })
        );
        assert_eq!(expanded.binding, "country_expanded_changed");
        assert_eq!(
            expanded.property_binding.as_deref(),
            Some("country_expanded")
        );
        assert_eq!(expanded.payload, Value::Bool(false));

        let unknown_highlight = BTreeMap::from([
            (
                "country_suggestions".to_owned(),
                serde_json::json!([{ "id": "china", "text": "China" }]),
            ),
            ("country_query".to_owned(), Value::String(String::new())),
            (
                "country_highlighted".to_owned(),
                Value::String("missing".to_owned()),
            ),
            ("country_expanded".to_owned(), Value::Bool(false)),
        ]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &unknown_highlight, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "highlighted"
        ));
    }

    #[cfg(feature = "command-palette")]
    #[test]
    fn compiles_controlled_command_palette_and_emits_semantic_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "app-commands",
                "component": "command_palette",
                "properties": {
                  "placeholder": "Type a command",
                  "no_results_text": "No matching commands"
                },
                "property_bindings": {
                  "items": "commands",
                  "query": "command_query",
                  "highlighted": "command_highlighted",
                  "open": "command_open"
                },
                "action_bindings": {
                  "query_change": "command_query_changed",
                  "highlight_change": "command_highlight_changed",
                  "invoke": "command_invoked",
                  "open_change": "command_open_changed"
                },
                "children": [
                  { "id": "page", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "commands".to_owned(),
                    crate::ui_document::UiValueType::CommandPaletteItemArray,
                ),
                (
                    "command_query".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "command_highlighted".to_owned(),
                    crate::ui_document::UiValueType::NullableCommandPaletteItemId,
                ),
                (
                    "command_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "command_query_changed".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "command_highlight_changed".to_owned(),
                    crate::ui_document::UiValueType::CommandPaletteItemId,
                ),
                (
                    "command_invoked".to_owned(),
                    crate::ui_document::UiValueType::CommandPaletteItemId,
                ),
                (
                    "command_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "commands".to_owned(),
                serde_json::json!([
                    {
                      "id": "open-settings",
                      "title": "Open settings",
                      "keywords": ["preferences"],
                      "shortcut": "Ctrl+,",
                      "icon": "Settings"
                    },
                    {
                      "id": "open-file",
                      "title": "Open file",
                      "subtitle": "Choose from disk",
                      "icon": "File"
                    },
                    {
                      "id": "open-disabled",
                      "title": "Open unavailable",
                      "enabled": false
                    }
                ]),
            ),
            ("command_query".to_owned(), Value::String("open".to_owned())),
            (
                "command_highlighted".to_owned(),
                Value::String("open-file".to_owned()),
            ),
            ("command_open".to_owned(), Value::Bool(true)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let settings_author =
            crate::ui_document::UiCommandPaletteItemId::new("open-settings").unwrap();
        let file_author = crate::ui_document::UiCommandPaletteItemId::new("open-file").unwrap();
        let settings =
            crate::ui_document::ui_command_palette_runtime_id(&document.root.id, &settings_author);
        let file =
            crate::ui_document::ui_command_palette_runtime_id(&document.root.id, &file_author);
        let state = view
            .widget_command_palette_state(widget)
            .expect("command palette state");
        assert_eq!(state.visible_items.len(), 3);
        assert_eq!(state.enabled_items, vec![settings, file]);
        assert_eq!(state.highlighted, Some(file));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::CommandPaletteHighlighted {
                widget,
                item: settings,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(highlighted)] = messages.as_slice() else {
            panic!("highlighting must emit one stable-id action");
        };
        assert_eq!(highlighted.binding, "command_highlight_changed");
        assert_eq!(
            highlighted.property_binding.as_deref(),
            Some("command_highlighted")
        );
        assert_eq!(
            highlighted.payload,
            Value::String("open-settings".to_owned())
        );

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::CommandPaletteInvoked {
                widget,
                item: settings,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(invoked), Msg::Action(open)] = messages.as_slice() else {
            panic!("invocation must emit a stable-id action and controlled close");
        };
        assert_eq!(invoked.binding, "command_invoked");
        assert_eq!(
            invoked.property_binding.as_deref(),
            Some("command_highlighted")
        );
        assert_eq!(invoked.payload, Value::String("open-settings".to_owned()));
        assert_eq!(open.binding, "command_open_changed");
        assert_eq!(open.property_binding.as_deref(), Some("command_open"));
        assert_eq!(open.payload, Value::Bool(false));

        let mut invalid_values = values;
        invalid_values.insert(
            "command_highlighted".to_owned(),
            Value::String("open-disabled".to_owned()),
        );
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "highlighted"
        ));
    }

    #[cfg(feature = "tree")]
    #[test]
    fn compiles_controlled_tree_and_emits_semantic_state_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "project-tree",
                "component": "tree",
                "property_bindings": {
                  "nodes": "project_nodes",
                  "expanded": "project_expanded",
                  "selected": "project_selected"
                },
                "action_bindings": {
                  "select": "project_selected_changed",
                  "expanded_change": "project_expanded_changed",
                  "invoke": "project_invoked"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "project_nodes".to_owned(),
                    crate::ui_document::UiValueType::TreeNodeArray,
                ),
                (
                    "project_expanded".to_owned(),
                    crate::ui_document::UiValueType::TreeNodeIdArray,
                ),
                (
                    "project_selected".to_owned(),
                    crate::ui_document::UiValueType::NullableTreeNodeId,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "project_selected_changed".to_owned(),
                    crate::ui_document::UiValueType::TreeNodeId,
                ),
                (
                    "project_expanded_changed".to_owned(),
                    crate::ui_document::UiValueType::TreeNodeIdArray,
                ),
                (
                    "project_invoked".to_owned(),
                    crate::ui_document::UiValueType::TreeNodeId,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "project_nodes".to_owned(),
                serde_json::json!([
                    {
                      "id": "workspace",
                      "label": "Workspace",
                      "icon": "Folder",
                      "children": [
                        { "id": "readme", "label": "README.md", "icon": "File" },
                        { "id": "cargo", "label": "Cargo.toml", "icon": "File" }
                      ]
                    }
                ]),
            ),
            (
                "project_expanded".to_owned(),
                serde_json::json!(["workspace"]),
            ),
            (
                "project_selected".to_owned(),
                Value::String("readme".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let workspace_author = crate::ui_document::UiTreeNodeId::new("workspace").unwrap();
        let readme_author = crate::ui_document::UiTreeNodeId::new("readme").unwrap();
        let cargo_author = crate::ui_document::UiTreeNodeId::new("cargo").unwrap();
        let workspace =
            crate::ui_document::ui_tree_runtime_id(&document.root.id, &workspace_author);
        let readme = crate::ui_document::ui_tree_runtime_id(&document.root.id, &readme_author);
        let cargo = crate::ui_document::ui_tree_runtime_id(&document.root.id, &cargo_author);
        let state = view.widget_tree_view_state(widget).expect("tree state");
        assert_eq!(state.rows.len(), 3);
        assert_eq!(state.selected, Some(readme));
        assert!(state.row(workspace).is_some_and(|row| row.expanded));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TreeNodeSelected {
                widget,
                node: cargo,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(selected)] = messages.as_slice() else {
            panic!("tree selection must emit one semantic-id action");
        };
        assert_eq!(selected.binding, "project_selected_changed");
        assert_eq!(
            selected.property_binding.as_deref(),
            Some("project_selected")
        );
        assert_eq!(selected.payload, Value::String("cargo".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TreeNodeInvoked {
                widget,
                node: cargo,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(invoked)] = messages.as_slice() else {
            panic!("tree invocation must emit one semantic-id action");
        };
        assert_eq!(invoked.binding, "project_invoked");
        assert_eq!(invoked.property_binding, None);
        assert_eq!(invoked.payload, Value::String("cargo".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TreeNodeExpandedChanged {
                widget,
                node: workspace,
                expanded: false,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(expanded)] = messages.as_slice() else {
            panic!("tree disclosure must emit the complete expanded-id state");
        };
        assert_eq!(expanded.binding, "project_expanded_changed");
        assert_eq!(
            expanded.property_binding.as_deref(),
            Some("project_expanded")
        );
        assert_eq!(expanded.payload, serde_json::json!([]));

        let mut invalid_values = values;
        invalid_values.insert("project_expanded".to_owned(), serde_json::json!(["cargo"]));
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "expanded"
        ));
    }

    #[cfg(all(feature = "menu-flyout", feature = "button", feature = "label"))]
    #[test]
    fn compiles_controlled_menu_flyout_and_emits_stable_nested_item_ids() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "file-menu",
                "component": "menu_flyout",
                "property_bindings": {
                  "open": "file_menu_open",
                  "target": "file_menu_target",
                  "items": "file_menu_items"
                },
                "action_bindings": {
                  "invoke": "file_menu_invoked",
                  "open_change": "file_menu_open_changed"
                },
                "children": [
                  {
                    "id": "page",
                    "component": "stack",
                    "children": [
                      {
                        "id": "open-file-menu",
                        "component": "button",
                        "properties": { "label": "File" }
                      },
                      {
                        "id": "page-copy",
                        "component": "text",
                        "properties": { "text": "Platform menu content" }
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "file_menu_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "file_menu_target".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "file_menu_items".to_owned(),
                    crate::ui_document::UiValueType::MenuFlyoutItemArray,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "file_menu_invoked".to_owned(),
                    crate::ui_document::UiValueType::MenuFlyoutItemId,
                ),
                (
                    "file_menu_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let items = serde_json::json!([
            {
              "kind": "command",
              "id": "save",
              "label": "Save",
              "accelerator": { "key": "S", "primary": true }
            },
            { "kind": "separator" },
            {
              "kind": "submenu",
              "id": "more",
              "label": "More",
              "items": [
                {
                  "kind": "command",
                  "id": "export-pdf",
                  "label": "Export PDF"
                }
              ]
            }
        ]);
        let values = BTreeMap::from([
            ("file_menu_open".to_owned(), Value::Bool(true)),
            (
                "file_menu_target".to_owned(),
                Value::String("open-file-menu".to_owned()),
            ),
            ("file_menu_items".to_owned(), items),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let target = document.root.children[0].children[0].id.widget_id();
        let (state, menu) = view
            .widget_menu_flyout_state(widget)
            .expect("compiled MenuFlyout state");
        assert!(state.open);
        assert_eq!(state.target, target);
        assert_eq!(menu.items.len(), 3);
        assert!(matches!(
            &menu.items[0],
            crate::MenuItemSpec::Command {
                id: Some(id),
                command: crate::Command::Custom { id: command_id, payload: None },
                accelerator: Some(accelerator),
                ..
            } if id == "save" && command_id == "save" && accelerator.uses_primary()
        ));

        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 720,
                height: 420,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        assert!(view.interaction_plan().hit_targets.iter().any(|target| {
            matches!(
                target.kind,
                crate::ViewHitTargetKind::MenuFlyoutItem { path, .. }
                    if path == crate::ZsMenuFlyoutPath::root(0)
            )
        }));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::MenuFlyoutInvoked {
                widget,
                path: crate::ZsMenuFlyoutPath::root(0),
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(invoked), Msg::Action(closed)] = messages.as_slice() else {
            panic!("menu invocation must emit the semantic item id and controlled close state");
        };
        assert_eq!(invoked.binding, "file_menu_invoked");
        assert_eq!(invoked.property_binding, None);
        assert_eq!(invoked.payload, Value::String("save".to_owned()));
        assert_eq!(closed.binding, "file_menu_open_changed");
        assert_eq!(closed.property_binding.as_deref(), Some("file_menu_open"));
        assert_eq!(closed.payload, Value::Bool(false));
        assert!(view
            .widget_menu_flyout_state(widget)
            .is_some_and(|(state, _)| !state.open));

        let mut invalid_values = values;
        invalid_values.insert(
            "file_menu_items".to_owned(),
            serde_json::json!([
                { "kind": "command", "id": "same", "label": "First" },
                { "kind": "command", "id": "same", "label": "Second" }
            ]),
        );
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::Validation { diagnostics })
                if diagnostics.iter().any(|diagnostic|
                    diagnostic.code
                        == crate::ui_document::UiDiagnosticCode::BindingValueTypeMismatch
                        && diagnostic.path == "$.file_menu_items")
        ));
    }

    #[cfg(all(feature = "flyout", feature = "button", feature = "label"))]
    #[test]
    fn compiles_controlled_flyout_and_routes_arbitrary_content_and_dismissal() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "details-flyout",
                "component": "flyout",
                "properties": {
                  "content_width": 280,
                  "content_height": 120,
                  "placement": "right"
                },
                "property_bindings": {
                  "open": "details_open",
                  "target": "details_target"
                },
                "action_bindings": {
                  "dismiss": "details_dismissed",
                  "open_change": "details_open_changed"
                },
                "children": [
                  {
                    "id": "page",
                    "component": "stack",
                    "children": [
                      {
                        "id": "open-details",
                        "component": "button",
                        "properties": { "label": "Details" }
                      }
                    ]
                  },
                  {
                    "id": "flyout-content",
                    "component": "stack",
                    "children": [
                      {
                        "id": "content-label",
                        "component": "text",
                        "properties": { "text": "Platform popover content" }
                      },
                      {
                        "id": "apply-details",
                        "component": "button",
                        "properties": { "label": "Apply" },
                        "action_bindings": { "click": "details_applied" }
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "details_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "details_target".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "details_applied".to_owned(),
                    crate::ui_document::UiValueType::Null,
                ),
                (
                    "details_dismissed".to_owned(),
                    crate::ui_document::UiValueType::FlyoutDismissReason,
                ),
                (
                    "details_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            ("details_open".to_owned(), Value::Bool(true)),
            (
                "details_target".to_owned(),
                Value::String("open-details".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let target = document.root.children[0].children[0].id.widget_id();
        let action = document.root.children[1].children[1].id.widget_id();
        assert_eq!(
            view.widget_flyout_state(widget),
            Some(crate::ZsFlyoutState { open: true, target })
        );

        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 720,
                height: 420,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        assert!(view
            .interaction_plan()
            .hit_target_for_widget(action)
            .is_some());

        let mut events = crate::ViewEventCx::new();
        view.event(&mut events, &crate::ViewEvent::Click { widget: action });
        let messages = events.into_messages();
        let [Msg::Action(applied)] = messages.as_slice() else {
            panic!("flyout content action must use the ordinary typed child route");
        };
        assert_eq!(applied.binding, "details_applied");
        assert_eq!(applied.payload, Value::Null);

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::FlyoutDismissed {
                widget,
                reason: crate::ZsFlyoutDismissReason::EscapeKey,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(dismissed), Msg::Action(closed)] = messages.as_slice() else {
            panic!("flyout dismissal must emit the reason and controlled open state");
        };
        assert_eq!(dismissed.binding, "details_dismissed");
        assert_eq!(dismissed.property_binding, None);
        assert_eq!(dismissed.payload, Value::String("escape".to_owned()));
        assert_eq!(closed.binding, "details_open_changed");
        assert_eq!(closed.property_binding.as_deref(), Some("details_open"));
        assert_eq!(closed.payload, Value::Bool(false));
        assert!(view
            .widget_flyout_state(widget)
            .is_some_and(|state| !state.open));

        let mut invalid_values = values;
        invalid_values.insert(
            "details_target".to_owned(),
            Value::String("flyout-content".to_owned()),
        );
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "target"
        ));
    }

    #[cfg(feature = "breadcrumb")]
    #[test]
    fn compiles_controlled_breadcrumb_and_emits_semantic_item_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "location-path",
                "component": "breadcrumb",
                "property_bindings": {
                  "items": "navigation_path",
                  "expanded": "navigation_overflow_open"
                },
                "action_bindings": {
                  "select": "navigation_selected",
                  "expanded_change": "navigation_overflow_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "navigation_path".to_owned(),
                    crate::ui_document::UiValueType::BreadcrumbItemArray,
                ),
                (
                    "navigation_overflow_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "navigation_selected".to_owned(),
                    crate::ui_document::UiValueType::BreadcrumbItemId,
                ),
                (
                    "navigation_overflow_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "navigation_path".to_owned(),
                serde_json::json!([
                    { "id": "home", "label": "Home" },
                    { "id": "projects", "label": "Projects" },
                    { "id": "framework", "label": "ZSUI Framework" }
                ]),
            ),
            ("navigation_overflow_open".to_owned(), Value::Bool(false)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let projects_author = crate::ui_document::UiBreadcrumbItemId::new("projects").unwrap();
        let projects =
            crate::ui_document::ui_breadcrumb_runtime_id(&document.root.id, &projects_author);
        let state = view
            .widget_breadcrumb_state(widget)
            .expect("breadcrumb state");
        assert_eq!(state.items.len(), 3);
        assert!(!state.overflow_open);

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::BreadcrumbExpandedChanged {
                widget,
                expanded: true,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(expanded)] = messages.as_slice() else {
            panic!("breadcrumb expansion must emit one controlled-state action");
        };
        assert_eq!(expanded.binding, "navigation_overflow_changed");
        assert_eq!(
            expanded.property_binding.as_deref(),
            Some("navigation_overflow_open")
        );
        assert_eq!(expanded.payload, Value::Bool(true));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::BreadcrumbSelected {
                widget,
                item: projects,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(closed), Msg::Action(selected)] = messages.as_slice() else {
            panic!("breadcrumb selection must close overflow and emit its semantic id");
        };
        assert_eq!(closed.binding, "navigation_overflow_changed");
        assert_eq!(closed.payload, Value::Bool(false));
        assert_eq!(selected.binding, "navigation_selected");
        assert_eq!(selected.property_binding, None);
        assert_eq!(selected.payload, Value::String("projects".to_owned()));

        let mut invalid_values = values;
        invalid_values.insert(
            "navigation_path".to_owned(),
            serde_json::json!([{ "id": "blank", "label": " " }]),
        );
        assert!(ui_document_view(&document, &bindings, &invalid_values, Msg::Action).is_err());
    }

    #[cfg(feature = "document-shell")]
    #[test]
    fn compiles_command_bar_groups_and_routes_child_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "editor-command-bar",
                "component": "command_bar",
                "properties": { "trailing": ["about"], "gap": 4 },
                "children": [
                  {
                    "id": "save",
                    "component": "button",
                    "properties": {
                      "label": "Save",
                      "presentation": "toolbar",
                      "icon": "Save"
                    },
                    "action_bindings": { "click": "save_clicked" }
                  },
                  {
                    "id": "undo",
                    "component": "button",
                    "properties": {
                      "label": "Undo",
                      "presentation": "toolbar",
                      "icon": "Undo"
                    }
                  },
                  {
                    "id": "about",
                    "component": "button",
                    "properties": {
                      "label": "About",
                      "presentation": "icon",
                      "icon": "Info"
                    }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::new(),
            actions: BTreeMap::from([(
                "save_clicked".to_owned(),
                crate::ui_document::UiValueType::Null,
            )]),
        };
        let mut view =
            ui_document_view(&document, &bindings, &BTreeMap::new(), Msg::Action).unwrap();
        assert_eq!(view.children.len(), 4);
        assert!(matches!(
            &view.children[0].kind,
            crate::ViewNodeKind::Button {
                presentation: crate::ZsButtonPresentation::Toolbar {
                    icon: crate::ZsIcon::Save,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &view.children[3].kind,
            crate::ViewNodeKind::Button {
                presentation: crate::ZsButtonPresentation::Icon {
                    icon: crate::ZsIcon::Info
                },
                ..
            }
        ));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::Click {
                widget: document.root.children[0].id.widget_id(),
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(clicked)] = messages.as_slice() else {
            panic!("command-bar child click must emit one typed action");
        };
        assert_eq!(clicked.node_id, "save");
        assert_eq!(clicked.binding, "save_clicked");
        assert_eq!(clicked.payload, Value::Null);
    }

    #[cfg(feature = "shell")]
    #[test]
    fn compiles_navigation_and_emits_semantic_selection() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "app-navigation",
                "component": "navigation",
                "properties": {
                  "title": "ZSUI",
                  "subtitle": "Native UI",
                  "items": [
                    { "id": "home", "label": "Home", "icon": "App" },
                    { "id": "files", "label": "Files", "icon": "Folder" }
                  ],
                  "footer_items": [
                    { "id": "settings", "label": "Settings", "icon": "Settings" }
                  ],
                  "selected": "home",
                  "pane_width": 240,
                  "minimum_content_width": 420
                },
                "action_bindings": { "select": "navigation_selected_changed" },
                "children": [
                  { "id": "navigation-content", "component": "text", "properties": { "text": "Home" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::new(),
            actions: BTreeMap::from([(
                "navigation_selected_changed".to_owned(),
                crate::ui_document::UiValueType::NavigationItemId,
            )]),
        };
        let mut view =
            ui_document_view(&document, &bindings, &BTreeMap::new(), Msg::Action).unwrap();
        let files_author = crate::ui_document::UiNavigationItemId::new("files").unwrap();
        let files = crate::ui_document::ui_navigation_runtime_id(&document.root.id, &files_author);

        let mut events = crate::ViewEventCx::new();
        view.event(&mut events, &crate::ViewEvent::Click { widget: files });
        let messages = events.into_messages();
        let [Msg::Action(selected)] = messages.as_slice() else {
            panic!("navigation selection must emit one semantic-id action");
        };
        assert_eq!(selected.binding, "navigation_selected_changed");
        assert_eq!(selected.property_binding, None);
        assert_eq!(selected.payload, Value::String("files".to_owned()));

        assert!(files.0 >> 62 == 3);
    }

    #[cfg(feature = "grid-view")]
    #[test]
    fn compiles_controlled_grid_view_and_emits_semantic_item_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "library-grid",
                "component": "grid_view",
                "property_bindings": {
                  "items": "library_items",
                  "selected": "library_selected"
                },
                "action_bindings": {
                  "select": "library_selected_changed",
                  "invoke": "library_invoked"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "library_items".to_owned(),
                    crate::ui_document::UiValueType::GridViewItemArray,
                ),
                (
                    "library_selected".to_owned(),
                    crate::ui_document::UiValueType::NullableGridViewItemId,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "library_selected_changed".to_owned(),
                    crate::ui_document::UiValueType::GridViewItemId,
                ),
                (
                    "library_invoked".to_owned(),
                    crate::ui_document::UiValueType::GridViewItemId,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "library_items".to_owned(),
                serde_json::json!([
                    {
                      "id": "documents",
                      "title": "Documents",
                      "subtitle": "12 folders",
                      "icon": "Folder"
                    },
                    { "id": "photos", "title": "Photos", "icon": "Image" },
                    { "id": "source", "title": "Source", "icon": "Code" }
                ]),
            ),
            (
                "library_selected".to_owned(),
                Value::String("documents".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let documents_author = crate::ui_document::UiGridViewItemId::new("documents").unwrap();
        let photos_author = crate::ui_document::UiGridViewItemId::new("photos").unwrap();
        let documents =
            crate::ui_document::ui_grid_view_runtime_id(&document.root.id, &documents_author);
        let photos = crate::ui_document::ui_grid_view_runtime_id(&document.root.id, &photos_author);
        let state = view
            .widget_grid_view_state(widget)
            .expect("grid-view state");
        assert_eq!(state.items.len(), 3);
        assert_eq!(state.selected, Some(documents));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::GridViewItemSelected {
                widget,
                item: photos,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(selected)] = messages.as_slice() else {
            panic!("grid-view selection must emit one semantic-id action");
        };
        assert_eq!(selected.binding, "library_selected_changed");
        assert_eq!(
            selected.property_binding.as_deref(),
            Some("library_selected")
        );
        assert_eq!(selected.payload, Value::String("photos".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::GridViewItemInvoked {
                widget,
                item: photos,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(invoked)] = messages.as_slice() else {
            panic!("grid-view invocation must emit one semantic-id action");
        };
        assert_eq!(invoked.binding, "library_invoked");
        assert_eq!(invoked.property_binding, None);
        assert_eq!(invoked.payload, Value::String("photos".to_owned()));

        let mut invalid_values = values;
        invalid_values.insert(
            "library_selected".to_owned(),
            Value::String("missing".to_owned()),
        );
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "selected"
        ));
    }

    #[cfg(feature = "table")]
    #[test]
    fn compiles_controlled_table_and_emits_semantic_row_and_sort_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "inventory",
                "component": "table",
                "property_bindings": {
                  "columns": "inventory_columns",
                  "rows": "inventory_rows",
                  "selected": "inventory_selected",
                  "sort": "inventory_sort"
                },
                "action_bindings": {
                  "select": "inventory_selected_changed",
                  "sort": "inventory_sort_changed",
                  "invoke": "inventory_invoked"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "inventory_columns".to_owned(),
                    crate::ui_document::UiValueType::TableColumnArray,
                ),
                (
                    "inventory_rows".to_owned(),
                    crate::ui_document::UiValueType::TableRowArray,
                ),
                (
                    "inventory_selected".to_owned(),
                    crate::ui_document::UiValueType::NullableTableRowId,
                ),
                (
                    "inventory_sort".to_owned(),
                    crate::ui_document::UiValueType::NullableTableSort,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "inventory_selected_changed".to_owned(),
                    crate::ui_document::UiValueType::TableRowId,
                ),
                (
                    "inventory_sort_changed".to_owned(),
                    crate::ui_document::UiValueType::TableSort,
                ),
                (
                    "inventory_invoked".to_owned(),
                    crate::ui_document::UiValueType::TableRowId,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "inventory_columns".to_owned(),
                serde_json::json!([
                    {
                      "id": "name",
                      "header": "Name",
                      "width": { "kind": "fill", "weight": 2 },
                      "sortable": true
                    },
                    {
                      "id": "status",
                      "header": "Status",
                      "width": { "kind": "fixed", "width": 120.0 },
                      "alignment": "center"
                    }
                ]),
            ),
            (
                "inventory_rows".to_owned(),
                serde_json::json!([
                    { "id": "alpha", "cells": { "name": "Alpha", "status": "Ready" } },
                    { "id": "beta", "cells": { "name": "Beta", "status": "Pending" } }
                ]),
            ),
            (
                "inventory_selected".to_owned(),
                Value::String("alpha".to_owned()),
            ),
            (
                "inventory_sort".to_owned(),
                serde_json::json!({ "column": "name", "direction": "ascending" }),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        let name_author = crate::ui_document::UiTableColumnId::new("name").unwrap();
        let alpha_author = crate::ui_document::UiTableRowId::new("alpha").unwrap();
        let beta_author = crate::ui_document::UiTableRowId::new("beta").unwrap();
        let name = crate::ui_document::ui_table_column_runtime_id(&document.root.id, &name_author);
        let alpha = crate::ui_document::ui_table_row_runtime_id(&document.root.id, &alpha_author);
        let beta = crate::ui_document::ui_table_row_runtime_id(&document.root.id, &beta_author);
        let state = view.widget_table_state(widget).expect("table state");
        assert_eq!(state.rows, vec![alpha, beta]);
        assert_eq!(state.selected, Some(alpha));
        assert_eq!(
            state.sort,
            Some(crate::ZsTableSort::new(
                name,
                crate::ZsTableSortDirection::Ascending
            ))
        );

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TableRowSelected { widget, row: beta },
        );
        view.event(
            &mut events,
            &crate::ViewEvent::TableRowInvoked { widget, row: beta },
        );
        view.event(
            &mut events,
            &crate::ViewEvent::TableSorted {
                widget,
                column: name,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(selected), Msg::Action(invoked), Msg::Action(sorted)] =
            messages.as_slice()
        else {
            panic!("table must emit stable row selection/invocation and sort actions");
        };
        assert_eq!(selected.binding, "inventory_selected_changed");
        assert_eq!(
            selected.property_binding.as_deref(),
            Some("inventory_selected")
        );
        assert_eq!(selected.payload, Value::String("beta".to_owned()));
        assert_eq!(invoked.binding, "inventory_invoked");
        assert_eq!(invoked.property_binding, None);
        assert_eq!(invoked.payload, Value::String("beta".to_owned()));
        assert_eq!(sorted.binding, "inventory_sort_changed");
        assert_eq!(sorted.property_binding.as_deref(), Some("inventory_sort"));
        assert_eq!(
            sorted.payload,
            serde_json::json!({ "column": "name", "direction": "descending" })
        );

        let mut invalid_values = values;
        invalid_values.insert(
            "inventory_rows".to_owned(),
            serde_json::json!([
                { "id": "alpha", "cells": { "name": "Alpha" } }
            ]),
        );
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "rows"
        ));
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn compiles_controlled_date_picker_and_emits_canonical_state_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "release-date",
                "component": "date_picker",
                "properties": {
                  "minimum": "2026-01-01",
                  "maximum": "2026-12-31",
                  "today": "2026-07-22"
                },
                "property_bindings": {
                  "value": "release_date",
                  "visible_month": "release_month",
                  "expanded": "release_expanded"
                },
                "action_bindings": {
                  "change": "release_date_changed",
                  "month_change": "release_month_changed",
                  "expanded_change": "release_expanded_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "release_date".to_owned(),
                    crate::ui_document::UiValueType::Date,
                ),
                (
                    "release_month".to_owned(),
                    crate::ui_document::UiValueType::Date,
                ),
                (
                    "release_expanded".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "release_date_changed".to_owned(),
                    crate::ui_document::UiValueType::Date,
                ),
                (
                    "release_month_changed".to_owned(),
                    crate::ui_document::UiValueType::Date,
                ),
                (
                    "release_expanded_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "release_date".to_owned(),
                Value::String("2026-07-22".to_owned()),
            ),
            (
                "release_month".to_owned(),
                Value::String("2026-07-01".to_owned()),
            ),
            ("release_expanded".to_owned(), Value::Bool(true)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        assert_eq!(
            view.widget_date_picker_state(widget),
            Some(crate::ZsDatePickerState {
                value: crate::ZsDate::new(2026, 7, 22).unwrap(),
                minimum: crate::ZsDate::new(2026, 1, 1).unwrap(),
                maximum: crate::ZsDate::new(2026, 12, 31).unwrap(),
                visible_month: crate::ZsDate::new(2026, 7, 1).unwrap(),
                expanded: true,
            })
        );

        let next_month = crate::ZsDate::new(2026, 8, 1).unwrap();
        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::DatePickerMonthChanged {
                widget,
                month: next_month,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("month navigation must emit one typed action");
        };
        assert_eq!(action.binding, "release_month_changed");
        assert_eq!(action.property_binding.as_deref(), Some("release_month"));
        assert_eq!(action.payload, Value::String("2026-08-01".to_owned()));

        let out_of_range = BTreeMap::from([
            (
                "release_date".to_owned(),
                Value::String("2027-01-01".to_owned()),
            ),
            (
                "release_month".to_owned(),
                Value::String("2026-07-01".to_owned()),
            ),
            ("release_expanded".to_owned(), Value::Bool(false)),
        ]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &out_of_range, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "value"
        ));
    }

    #[cfg(feature = "time-picker")]
    #[test]
    fn compiles_controlled_time_picker_and_emits_canonical_state_actions() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "meeting-time",
                "component": "time_picker",
                "properties": {
                  "minute_increment": 15,
                  "clock_format": "twenty_four_hour"
                },
                "property_bindings": {
                  "value": "meeting_time",
                  "expanded": "meeting_time_expanded"
                },
                "action_bindings": {
                  "change": "meeting_time_changed",
                  "expanded_change": "meeting_time_expanded_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "meeting_time".to_owned(),
                    crate::ui_document::UiValueType::Time,
                ),
                (
                    "meeting_time_expanded".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "meeting_time_changed".to_owned(),
                    crate::ui_document::UiValueType::Time,
                ),
                (
                    "meeting_time_expanded_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            ("meeting_time".to_owned(), Value::String("09:30".to_owned())),
            ("meeting_time_expanded".to_owned(), Value::Bool(true)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        assert_eq!(
            view.widget_time_picker_state(widget),
            Some(crate::ZsTimePickerState {
                value: crate::ZsTime::new(9, 30).unwrap(),
                minute_increment: crate::ZsMinuteIncrement::FIFTEEN,
                clock: crate::ZsClockFormat::TwentyFourHour,
                expanded: true,
            })
        );

        let selected = crate::ZsTime::new(10, 45).unwrap();
        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TimeChanged {
                widget,
                value: selected,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("time selection must emit one typed action");
        };
        assert_eq!(action.binding, "meeting_time_changed");
        assert_eq!(action.property_binding.as_deref(), Some("meeting_time"));
        assert_eq!(action.payload, Value::String("10:45".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TimePickerExpandedChanged {
                widget,
                expanded: false,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("expanded state must emit one typed action");
        };
        assert_eq!(action.binding, "meeting_time_expanded_changed");
        assert_eq!(
            action.property_binding.as_deref(),
            Some("meeting_time_expanded")
        );
        assert_eq!(action.payload, Value::Bool(false));

        let misaligned = BTreeMap::from([
            ("meeting_time".to_owned(), Value::String("09:17".to_owned())),
            ("meeting_time_expanded".to_owned(), Value::Bool(false)),
        ]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &misaligned, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "value"
        ));
    }

    #[cfg(feature = "color-picker")]
    #[test]
    fn compiles_controlled_color_picker_and_emits_canonical_state_actions() {
        let document = UiDocument::from_json(
            r##"{
              "schema_version": 1,
              "root": {
                "id": "accent-color",
                "component": "color_picker",
                "properties": { "alpha_enabled": true },
                "property_bindings": {
                  "value": "accent_color",
                  "expanded": "accent_color_expanded",
                  "active_channel": "accent_color_channel"
                },
                "action_bindings": {
                  "change": "accent_color_changed",
                  "expanded_change": "accent_color_expanded_changed",
                  "channel_change": "accent_color_channel_changed"
                }
              }
            }"##,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "accent_color".to_owned(),
                    crate::ui_document::UiValueType::Color,
                ),
                (
                    "accent_color_expanded".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "accent_color_channel".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "accent_color_changed".to_owned(),
                    crate::ui_document::UiValueType::Color,
                ),
                (
                    "accent_color_expanded_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "accent_color_channel_changed".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
        };
        let values = BTreeMap::from([
            (
                "accent_color".to_owned(),
                Value::String("#2060A0E0".to_owned()),
            ),
            ("accent_color_expanded".to_owned(), Value::Bool(true)),
            (
                "accent_color_channel".to_owned(),
                Value::String("red".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let widget = document.root.id.widget_id();
        assert_eq!(
            view.widget_color_picker_state(widget),
            Some(
                crate::ZsColorPickerState::new(crate::Color::rgba(32, 96, 160, 224))
                    .with_expanded(true)
            )
        );

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::ColorPickerChannelChanged {
                widget,
                channel: crate::ZsColorChannel::Green,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("channel selection must emit one typed action");
        };
        assert_eq!(action.binding, "accent_color_channel_changed");
        assert_eq!(
            action.property_binding.as_deref(),
            Some("accent_color_channel")
        );
        assert_eq!(action.payload, Value::String("green".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::ColorChanged {
                widget,
                color: crate::Color::rgba(12, 34, 56, 78),
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("color edit must emit one typed action");
        };
        assert_eq!(action.binding, "accent_color_changed");
        assert_eq!(action.property_binding.as_deref(), Some("accent_color"));
        assert_eq!(action.payload, Value::String("#0C22384E".to_owned()));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::ColorPickerExpandedChanged {
                widget,
                expanded: false,
            },
        );
        let messages = events.into_messages();
        let [Msg::Action(action)] = messages.as_slice() else {
            panic!("expanded state must emit one typed action");
        };
        assert_eq!(action.binding, "accent_color_expanded_changed");
        assert_eq!(
            action.property_binding.as_deref(),
            Some("accent_color_expanded")
        );
        assert_eq!(action.payload, Value::Bool(false));

        let alpha_disabled = UiDocument::from_json(
            r##"{
              "schema_version": 1,
              "root": {
                "id": "opaque-color",
                "component": "color_picker",
                "properties": {
                  "alpha_enabled": false
                },
                "property_bindings": { "value": "opaque_color" }
              }
            }"##,
        )
        .unwrap();
        let alpha_bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "opaque_color".to_owned(),
                crate::ui_document::UiValueType::Color,
            )]),
            actions: BTreeMap::new(),
        };
        let alpha_values = BTreeMap::from([(
            "opaque_color".to_owned(),
            Value::String("#2060A0E0".to_owned()),
        )]);
        assert!(matches!(
            ui_document_view(
                &alpha_disabled,
                &alpha_bindings,
                &alpha_values,
                Msg::Action,
            ),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "value"
        ));
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

    #[cfg(feature = "info-bar")]
    #[test]
    fn compiles_info_bar_and_maps_typed_action_and_close_events() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "sync-status",
                "component": "info_bar",
                "properties": {
                  "title": "Up to date",
                  "action_label": "View activity"
                },
                "property_bindings": {
                  "message": "sync_message",
                  "severity": "sync_severity",
                  "closable": "sync_closable"
                },
                "action_bindings": { "event": "sync_status_event" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "sync_message".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "sync_severity".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "sync_closable".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
            actions: BTreeMap::from([(
                "sync_status_event".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
        };
        let values = BTreeMap::from([
            (
                "sync_message".to_owned(),
                Value::String("All changes are synchronized.".to_owned()),
            ),
            (
                "sync_severity".to_owned(),
                Value::String("success".to_owned()),
            ),
            ("sync_closable".to_owned(), Value::Bool(true)),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let crate::ViewNodeKind::InfoBar { spec, .. } = &view.kind else {
            panic!("document must compile to an InfoBar");
        };
        assert_eq!(spec.title_text(), Some("Up to date"));
        assert_eq!(spec.message(), "All changes are synchronized.");
        assert_eq!(spec.info_bar_severity(), crate::ZsInfoBarSeverity::Success);
        assert_eq!(spec.action_label(), Some("View activity"));
        assert!(spec.is_closable());

        let mut events = crate::ViewEventCx::new();
        for event in [crate::ZsInfoBarEvent::Action, crate::ZsInfoBarEvent::Close] {
            view.event(
                &mut events,
                &crate::ViewEvent::InfoBarInvoked {
                    widget: document.root.id.widget_id(),
                    event,
                },
            );
        }
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::Action(UiDocumentAction {
                    node_id: "sync-status".to_owned(),
                    binding: "sync_status_event".to_owned(),
                    property_binding: None,
                    payload: Value::String("action".to_owned()),
                }),
                Msg::Action(UiDocumentAction {
                    node_id: "sync-status".to_owned(),
                    binding: "sync_status_event".to_owned(),
                    property_binding: None,
                    payload: Value::String("close".to_owned()),
                }),
            ]
        );

        let invalid_values = BTreeMap::from([
            (
                "sync_message".to_owned(),
                Value::String("All changes are synchronized.".to_owned()),
            ),
            (
                "sync_severity".to_owned(),
                Value::String("critical".to_owned()),
            ),
            ("sync_closable".to_owned(), Value::Bool(true)),
        ]);
        assert!(matches!(
            ui_document_view(
                &document,
                &bindings,
                &invalid_values,
                Msg::Action,
            ),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "severity"
        ));
    }

    #[cfg(all(feature = "toast", feature = "label"))]
    #[test]
    fn compiles_toast_and_maps_result_and_open_change_without_viewer_runtime() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "saved",
                "component": "toast",
                "properties": {
                  "action_label": "Undo",
                  "duration": "persistent"
                },
                "property_bindings": {
                  "open": "saved_open",
                  "message": "saved_message"
                },
                "action_bindings": {
                  "result": "saved_result",
                  "open_change": "saved_open_changed"
                },
                "children": [
                  {
                    "id": "page",
                    "component": "text",
                    "properties": { "text": "Saved page" }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "saved_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "saved_message".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "saved_result".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "saved_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            ("saved_open".to_owned(), Value::Bool(true)),
            (
                "saved_message".to_owned(),
                Value::String("Saved successfully".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let toast_id = {
            let crate::ViewNodeKind::ToastPresenter {
                toast: Some(spec), ..
            } = &view.kind
            else {
                panic!("document must compile to an active Toast");
            };
            assert_eq!(
                spec.id(),
                crate::ZsToastId::from(document.root.id.widget_id().0)
            );
            assert_eq!(spec.message(), "Saved successfully");
            assert_eq!(spec.action_label(), Some("Undo"));
            assert_eq!(spec.toast_duration(), crate::ZsToastDuration::Persistent);
            spec.id()
        };

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::ToastResponded {
                widget: document.root.id.widget_id(),
                toast: toast_id,
                response: crate::ZsToastResponse::Action,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::Action(UiDocumentAction {
                    node_id: "saved".to_owned(),
                    binding: "saved_result".to_owned(),
                    property_binding: None,
                    payload: Value::String("action".to_owned()),
                }),
                Msg::Action(UiDocumentAction {
                    node_id: "saved".to_owned(),
                    binding: "saved_open_changed".to_owned(),
                    property_binding: Some("saved_open".to_owned()),
                    payload: Value::Bool(false),
                }),
            ]
        );

        let invalid_document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "saved",
                "component": "toast",
                "properties": {
                  "open": true,
                  "message": "Saved successfully"
                },
                "property_bindings": { "duration": "saved_duration" },
                "children": [{ "id": "page", "component": "stack" }]
              }
            }"#,
        )
        .unwrap();
        assert!(matches!(
            ui_document_view(
                &invalid_document,
                &UiBindingSchema {
                    properties: BTreeMap::from([(
                        "saved_duration".to_owned(),
                        crate::ui_document::UiValueType::String,
                    )]),
                    actions: BTreeMap::new(),
                },
                &BTreeMap::from([(
                    "saved_duration".to_owned(),
                    Value::String("forever".to_owned()),
                )]),
                Msg::Action
            ),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "duration"
        ));
    }

    #[cfg(all(feature = "tooltip", feature = "button"))]
    #[test]
    fn compiles_tooltip_as_child_modifier_and_preserves_child_identity() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-help",
                "component": "tooltip",
                "properties": {
                  "placement": "bottom",
                  "open_delay_ms": 0
                },
                "property_bindings": { "text": "save_tooltip" },
                "children": [
                  {
                    "id": "save-button",
                    "component": "button",
                    "properties": { "label": "Save" }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "save_tooltip".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
            actions: BTreeMap::new(),
        };
        let values = BTreeMap::from([(
            "save_tooltip".to_owned(),
            Value::String("Save the current document".to_owned()),
        )]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let child_widget = document.root.children[0].id.widget_id();
        assert_eq!(view.id, Some(child_widget));
        assert_ne!(view.id, Some(document.root.id.widget_id()));

        let surface = Rect {
            x: 0,
            y: 0,
            width: 240,
            height: 120,
        };
        view.layout(&mut ViewLayoutCx::new(surface, Dpi::standard()));
        let interaction = view.interaction_plan();
        assert_eq!(interaction.hit_target_count(), 1);
        assert_eq!(interaction.tooltip_targets.len(), 1);
        assert_eq!(interaction.tooltip_targets[0].widget, child_widget);
        assert_eq!(
            interaction.tooltip_targets[0].spec.text,
            "Save the current document"
        );
        assert_eq!(
            interaction.tooltip_targets[0].spec.placement,
            crate::ZsTooltipPlacement::Bottom
        );
        assert_eq!(interaction.tooltip_targets[0].spec.open_delay_ms, Some(0));

        let invalid_values =
            BTreeMap::from([("save_tooltip".to_owned(), Value::String(" ".to_owned()))]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "text"
        ));
    }

    #[cfg(all(feature = "teaching-tip", feature = "button"))]
    #[test]
    fn compiles_teaching_tip_and_retains_controlled_dismissal() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-guidance",
                "component": "teaching_tip",
                "properties": {
                  "title": "Automatic saving",
                  "action_label": "Review settings",
                  "placement": "top"
                },
                "property_bindings": {
                  "open": "guidance_open",
                  "target": "guidance_target",
                  "subtitle": "guidance_subtitle"
                },
                "action_bindings": {
                  "result": "guidance_result",
                  "open_change": "guidance_open_changed"
                },
                "children": [
                  {
                    "id": "page",
                    "component": "stack",
                    "children": [
                      {
                        "id": "save-button",
                        "component": "button",
                        "properties": { "label": "Save" }
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                (
                    "guidance_open".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
                (
                    "guidance_target".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "guidance_subtitle".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "guidance_result".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "guidance_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([
            ("guidance_open".to_owned(), Value::Bool(true)),
            (
                "guidance_target".to_owned(),
                Value::String("save-button".to_owned()),
            ),
            (
                "guidance_subtitle".to_owned(),
                Value::String("Your changes are saved as you work.".to_owned()),
            ),
        ]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        let target = document.root.children[0].children[0].id.widget_id();
        let crate::ViewNodeKind::TeachingTip {
            spec,
            open,
            target: actual_target,
            ..
        } = &view.kind
        else {
            panic!("document must compile to a TeachingTip");
        };
        assert!(*open);
        assert_eq!(*actual_target, target);
        assert_eq!(spec.title(), "Automatic saving");
        assert_eq!(spec.subtitle(), "Your changes are saved as you work.");
        assert_eq!(spec.action_label(), Some("Review settings"));
        assert_eq!(spec.placement(), crate::ZsTeachingTipPlacement::Top);

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::TeachingTipResponded {
                widget: document.root.id.widget_id(),
                response: crate::ZsTeachingTipResponse::Action,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![
                Msg::Action(UiDocumentAction {
                    node_id: "save-guidance".to_owned(),
                    binding: "guidance_result".to_owned(),
                    property_binding: None,
                    payload: Value::String("action".to_owned()),
                }),
                Msg::Action(UiDocumentAction {
                    node_id: "save-guidance".to_owned(),
                    binding: "guidance_open_changed".to_owned(),
                    property_binding: Some("guidance_open".to_owned()),
                    payload: Value::Bool(false),
                }),
            ]
        );
        assert!(view
            .widget_teaching_tip_state(document.root.id.widget_id())
            .is_some_and(|(state, _)| !state.open));

        let invalid_values = BTreeMap::from([
            ("guidance_open".to_owned(), Value::Bool(true)),
            (
                "guidance_target".to_owned(),
                Value::String("missing".to_owned()),
            ),
            (
                "guidance_subtitle".to_owned(),
                Value::String("Your changes are saved as you work.".to_owned()),
            ),
        ]);
        assert!(matches!(
            ui_document_view(&document, &bindings, &invalid_values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "target"
        ));
    }

    #[cfg(all(feature = "dialog", feature = "label"))]
    #[test]
    fn compiles_content_dialog_and_maps_typed_result_without_viewer_runtime() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "confirm",
                "component": "content_dialog",
                "properties": {
                  "title": "Delete item",
                  "content": "This action cannot be undone.",
                  "primary_button": "Delete",
                  "close_button": "Cancel",
                  "default_button": "primary"
                },
                "property_bindings": { "open": "confirm_open" },
                "action_bindings": {
                  "result": "confirm_result",
                  "open_change": "confirm_open_changed"
                },
                "children": [
                  { "id": "page", "component": "text", "properties": { "text": "Page" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "confirm_open".to_owned(),
                crate::ui_document::UiValueType::Boolean,
            )]),
            actions: BTreeMap::from([
                (
                    "confirm_result".to_owned(),
                    crate::ui_document::UiValueType::String,
                ),
                (
                    "confirm_open_changed".to_owned(),
                    crate::ui_document::UiValueType::Boolean,
                ),
            ]),
        };
        let values = BTreeMap::from([("confirm_open".to_owned(), Value::Bool(true))]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            &view.kind,
            crate::ViewNodeKind::ContentDialog { open: true, .. }
        ));

        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::ContentDialogResponded {
                widget: document.root.id.widget_id(),
                button: crate::ZsContentDialogButton::Primary,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::Action(UiDocumentAction {
                    node_id: "confirm".to_owned(),
                    binding: "confirm_result".to_owned(),
                    property_binding: None,
                    payload: Value::String("primary".to_owned()),
                }),
                Msg::Action(UiDocumentAction {
                    node_id: "confirm".to_owned(),
                    binding: "confirm_open_changed".to_owned(),
                    property_binding: Some("confirm_open".to_owned()),
                    payload: Value::Bool(false),
                })
            ]
        );
    }

    #[cfg(feature = "password-box")]
    #[test]
    fn password_box_uses_only_the_secure_state_and_action_channels() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "account-password",
                "component": "password_box",
                "properties": { "reveal_mode": "peek" },
                "property_bindings": { "value": "account_password" },
                "action_bindings": { "change": "account_password_changed" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "account_password".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
            actions: BTreeMap::from([(
                "account_password_changed".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
        };
        assert!(matches!(
            ui_document_view(&document, &bindings, &BTreeMap::new(), Msg::Action),
            Err(UiDocumentRuntimeError::SecureChannelRequired { .. })
        ));
        let leaked = "secret-that-must-not-leak";
        assert!(matches!(
            ui_document_view(
                &document,
                &bindings,
                &BTreeMap::from([(
                    "account_password".to_owned(),
                    Value::String(leaked.to_owned()),
                )]),
                Msg::Action,
            ),
            Err(UiDocumentRuntimeError::Validation { diagnostics })
                if diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == crate::ui_document::UiDiagnosticCode::SensitiveBindingValue
                })
        ));

        let mut secrets = UiSecretValues::new();
        secrets.insert("account_password", leaked);
        let mut view = ui_document_view_with_secrets(
            &document,
            &bindings,
            &BTreeMap::new(),
            &secrets,
            Msg::Action,
            Msg::Secret,
        )
        .unwrap();
        assert_eq!(
            view.widget_password_value(document.root.id.widget_id())
                .map(crate::ZsPassword::as_str),
            Some(leaked)
        );

        let next = "next-secret-that-must-not-leak";
        let mut events = crate::ViewEventCx::new();
        view.event(
            &mut events,
            &crate::ViewEvent::PasswordChanged {
                widget: document.root.id.widget_id(),
                value: crate::ZsPassword::from(next),
            },
        );
        let messages = events.into_messages();
        let [Msg::Secret(action)] = messages.as_slice() else {
            panic!("password change must emit exactly one secure action");
        };
        assert_eq!(action.binding, "account_password_changed");
        assert_eq!(action.property_binding, "account_password");
        assert_eq!(action.value.as_str(), next);
        let debug = format!("{messages:?}");
        assert!(!debug.contains(leaked));
        assert!(!debug.contains(next));
        assert!(debug.contains("<redacted>"));
    }

    #[cfg(all(feature = "list", feature = "label"))]
    #[test]
    fn compiles_list_selection_by_stable_child_id() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "profiles",
                "component": "list",
                "property_bindings": { "selected": "selected_profile" },
                "action_bindings": { "select": "selected_profile_changed" },
                "children": [
                  { "id": "balanced", "component": "text", "properties": { "text": "Balanced" } },
                  { "id": "quiet", "component": "text", "properties": { "text": "Quiet" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "selected_profile".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
            actions: BTreeMap::from([(
                "selected_profile_changed".to_owned(),
                crate::ui_document::UiValueType::String,
            )]),
        };
        let values = BTreeMap::from([(
            "selected_profile".to_owned(),
            Value::String("quiet".to_owned()),
        )]);
        let mut view = ui_document_view(&document, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            &view.kind,
            crate::ViewNodeKind::List {
                selected_index: Some(1),
                ..
            }
        ));

        let balanced = document.root.children[0].id.widget_id();
        let mut events = crate::ViewEventCx::new();
        view.event(&mut events, &crate::ViewEvent::Click { widget: balanced });
        assert_eq!(
            events.into_messages(),
            vec![Msg::Action(UiDocumentAction {
                node_id: "profiles".to_owned(),
                binding: "selected_profile_changed".to_owned(),
                property_binding: Some("selected_profile".to_owned()),
                payload: Value::String("balanced".to_owned()),
            })]
        );

        let mut reordered = document.clone();
        reordered.root.children.reverse();
        let reordered_view = ui_document_view(&reordered, &bindings, &values, Msg::Action).unwrap();
        assert!(matches!(
            &reordered_view.kind,
            crate::ViewNodeKind::List {
                selected_index: Some(0),
                ..
            }
        ));
    }

    #[cfg(feature = "progress-ring")]
    #[test]
    fn compiles_progress_ring_modes_and_rejects_invalid_resolved_range_values() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "sync-progress",
                "component": "progress_ring",
                "properties": {
                  "minimum": 0.0,
                  "maximum": 1.0,
                  "active": true,
                  "size": "large"
                },
                "property_bindings": { "value": "sync_progress" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "sync_progress".to_owned(),
                crate::ui_document::UiValueType::NullableNumber,
            )]),
            actions: BTreeMap::new(),
        };
        let determinate = ui_document_view(
            &document,
            &bindings,
            &BTreeMap::from([("sync_progress".to_owned(), Value::from(0.25))]),
            Msg::Action,
        )
        .unwrap();
        let crate::ViewNodeKind::ProgressRing { spec } = determinate.kind else {
            panic!("document should compile to ProgressRing");
        };
        assert!(spec.is_active());
        assert_eq!(spec.size_value(), crate::ZsProgressRingSize::Large);
        assert_eq!(spec.mode().fraction(), Some(0.25));

        let indeterminate = ui_document_view(
            &document,
            &bindings,
            &BTreeMap::from([("sync_progress".to_owned(), Value::Null)]),
            Msg::Action,
        )
        .unwrap();
        let crate::ViewNodeKind::ProgressRing { spec } = indeterminate.kind else {
            panic!("null document value should compile to indeterminate ProgressRing");
        };
        assert_eq!(spec.mode().fraction(), None);

        assert!(matches!(
            ui_document_view(
                &document,
                &bindings,
                &BTreeMap::from([("sync_progress".to_owned(), Value::from(2.0))]),
                Msg::Action,
            ),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "value"
        ));
    }

    #[cfg(feature = "grid")]
    #[test]
    fn compiles_document_grid_geometry_and_rejects_invalid_resolved_placements() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "layout",
                "component": "grid",
                "properties": {
                  "columns": [
                    { "kind": "fixed", "size": 100.0 },
                    { "kind": "fraction", "weight": 1 }
                  ],
                  "rows": [
                    { "kind": "fraction", "weight": 1 },
                    { "kind": "fixed", "size": 40.0 }
                  ],
                  "placements": {
                    "navigation": { "row": 0, "column": 0, "row_span": 2 },
                    "content": { "row": 0, "column": 1 },
                    "actions": { "row": 1, "column": 1 }
                  },
                  "column_gap": 10.0,
                  "row_gap": 5.0
                },
                "children": [
                  { "id": "navigation", "component": "stack" },
                  { "id": "content", "component": "stack" },
                  { "id": "actions", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let mut view = ui_document_view(
            &document,
            &UiBindingSchema::default(),
            &BTreeMap::new(),
            Msg::Action,
        )
        .unwrap();
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 400,
                height: 200,
            },
            Dpi::standard(),
        );
        let output = view.layout(&mut layout);
        let bounds_for = |id: &str| {
            let widget = crate::ui_document::UiNodeId::new(id).unwrap().widget_id();
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("document Grid child should be laid out")
                .bounds
        };
        assert_eq!(
            bounds_for("navigation"),
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 200,
            }
        );
        assert_eq!(
            bounds_for("content"),
            Rect {
                x: 110,
                y: 0,
                width: 290,
                height: 155,
            }
        );
        assert_eq!(
            bounds_for("actions"),
            Rect {
                x: 110,
                y: 160,
                width: 290,
                height: 40,
            }
        );

        let mut reordered = document.clone();
        reordered.root.children.reverse();
        let mut reordered_view = ui_document_view(
            &reordered,
            &UiBindingSchema::default(),
            &BTreeMap::new(),
            Msg::Action,
        )
        .unwrap();
        let mut reordered_layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 400,
                height: 200,
            },
            Dpi::standard(),
        );
        let reordered_output = reordered_view.layout(&mut reordered_layout);
        let navigation = crate::ui_document::UiNodeId::new("navigation")
            .unwrap()
            .widget_id();
        assert_eq!(
            reordered_output
                .children
                .iter()
                .find(|node| node.component == navigation.into())
                .expect("stable Grid child should survive declaration reordering")
                .bounds,
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 200,
            }
        );

        let bound_document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "bound-layout",
                "component": "grid",
                "properties": {
                  "columns": [{ "kind": "fraction", "weight": 1 }],
                  "rows": [{ "kind": "fraction", "weight": 1 }]
                },
                "property_bindings": { "placements": "grid_cells" },
                "children": [{ "id": "content", "component": "stack" }]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([(
                "grid_cells".to_owned(),
                crate::ui_document::UiValueType::GridPlacementMap,
            )]),
            actions: BTreeMap::new(),
        };
        let values = BTreeMap::from([(
            "grid_cells".to_owned(),
            serde_json::json!({ "ghost": { "row": 0, "column": 0 } }),
        )]);
        assert!(matches!(
            ui_document_view(&bound_document, &bindings, &values, Msg::Action),
            Err(UiDocumentRuntimeError::InvalidResolvedProperty { property, .. })
                if property == "placements"
        ));
    }
}
