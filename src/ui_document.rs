//! Versioned, platform-neutral UI documents and typed Rust binding contracts.
//!
//! The document contains semantic structure and visual data only. Application
//! state and messages remain ordinary Rust types connected through
//! [`UiBindingManifest`].

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema version accepted by this ZSUI release.
pub const ZSUI_UI_DOCUMENT_SCHEMA_VERSION: u32 = 1;
const DOCUMENT_WIDGET_ID_NAMESPACE: u64 = 1 << 62;
const DOCUMENT_WIDGET_ID_MASK: u64 = DOCUMENT_WIDGET_ID_NAMESPACE - 1;

/// A validated, stable author-facing identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiNodeId(String);

impl UiNodeId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiNodeIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiNodeIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Maps an author ID to the reserved document-backed `WidgetId` namespace.
    pub fn widget_id(&self) -> crate::view::WidgetId {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in self.0.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        crate::view::WidgetId::new(DOCUMENT_WIDGET_ID_NAMESPACE | (hash & DOCUMENT_WIDGET_ID_MASK))
    }
}

impl fmt::Display for UiNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiNodeIdError {
    value: String,
}

impl fmt::Display for UiNodeIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "UI node id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiNodeIdError {}

/// A versioned semantic document consumed by validation, preview and release
/// embedding paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiDocument {
    pub schema_version: u32,
    pub root: UiNode,
}

impl UiDocument {
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn validate(
        &self,
        features: &UiFeatureSet,
        bindings: &UiBindingSchema,
    ) -> UiValidationReport {
        UiDocumentValidator::new(features, bindings).validate(self)
    }
}

/// Semantic structure shared by every native backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiNode {
    pub id: UiNodeId,
    pub component: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub property_bindings: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub action_bindings: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "UiLayout::is_empty")]
    pub layout: UiLayout,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub theme_tokens: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub localization: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<UiAccessibility>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiNode>,
}

/// Backend-neutral layout constraints in device-independent pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiLayout {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub flex: Option<f32>,
    pub direction: Option<UiAxis>,
}

impl UiLayout {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAxis {
    Horizontal,
    Vertical,
}

/// Semantic accessibility metadata. Platform providers lower these values to
/// UIA, AppKit Accessibility or AT-SPI later in the pipeline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAccessibility {
    pub role: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub live_region: Option<UiLiveRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLiveRegion {
    Polite,
    Assertive,
}

/// JSON value shape expected by a state property or action payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
    Any,
}

impl UiValueType {
    pub fn matches(self, value: &Value) -> bool {
        match self {
            Self::Null => value.is_null(),
            Self::Boolean => value.is_boolean(),
            Self::Number => value.is_number(),
            Self::String => value.is_string(),
            Self::Array => value.is_array(),
            Self::Object => value.is_object(),
            Self::Any => true,
        }
    }
}

/// Serializable projection used by `zsui-uic` and deterministic AI handoff.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiBindingSchema {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, UiValueType>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub actions: BTreeMap<String, UiValueType>,
}

type UiStateReader<State> = Arc<dyn Fn(&State) -> Value + Send + Sync + 'static>;
type UiActionMapper<Msg> = Arc<dyn Fn(Value) -> Result<Msg, String> + Send + Sync + 'static>;

struct UiStateBinding<State> {
    value_type: UiValueType,
    read: UiStateReader<State>,
}

struct UiActionBinding<Msg> {
    payload_type: UiValueType,
    map: UiActionMapper<Msg>,
}

/// Strongly typed bridge between serialized slots and application-owned Rust
/// `State`/`Msg` types.
///
/// String keys are validated contract names, not a global event bus. Action
/// dispatch always returns the manifest's concrete `Msg` type.
pub struct UiBindingManifest<State, Msg> {
    properties: BTreeMap<String, UiStateBinding<State>>,
    actions: BTreeMap<String, UiActionBinding<Msg>>,
}

impl<State, Msg> Default for UiBindingManifest<State, Msg> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State, Msg> fmt::Debug for UiBindingManifest<State, Msg> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiBindingManifest")
            .field("properties", &self.properties.keys().collect::<Vec<_>>())
            .field("actions", &self.actions.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl<State, Msg> UiBindingManifest<State, Msg> {
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }

    pub fn register_property(
        &mut self,
        name: impl Into<String>,
        value_type: UiValueType,
        read: impl Fn(&State) -> Value + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.properties.contains_key(&name) || self.actions.contains_key(&name) {
            return Err(UiBindingRegistrationError::Duplicate(name));
        }
        self.properties.insert(
            name,
            UiStateBinding {
                value_type,
                read: Arc::new(read),
            },
        );
        Ok(())
    }

    pub fn register_action(
        &mut self,
        name: impl Into<String>,
        payload_type: UiValueType,
        map: impl Fn(Value) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.properties.contains_key(&name) || self.actions.contains_key(&name) {
            return Err(UiBindingRegistrationError::Duplicate(name));
        }
        self.actions.insert(
            name,
            UiActionBinding {
                payload_type,
                map: Arc::new(map),
            },
        );
        Ok(())
    }

    pub fn schema(&self) -> UiBindingSchema {
        UiBindingSchema {
            properties: self
                .properties
                .iter()
                .map(|(name, binding)| (name.clone(), binding.value_type))
                .collect(),
            actions: self
                .actions
                .iter()
                .map(|(name, binding)| (name.clone(), binding.payload_type))
                .collect(),
        }
    }

    pub fn read_property(&self, name: &str, state: &State) -> Option<Value> {
        self.properties
            .get(name)
            .map(|binding| (binding.read)(state))
    }

    pub fn map_action(&self, name: &str, payload: Value) -> Result<Msg, UiBindingDispatchError> {
        let binding = self
            .actions
            .get(name)
            .ok_or_else(|| UiBindingDispatchError::UnknownAction(name.to_owned()))?;
        if !binding.payload_type.matches(&payload) {
            return Err(UiBindingDispatchError::PayloadType {
                action: name.to_owned(),
                expected: binding.payload_type,
            });
        }
        (binding.map)(payload).map_err(|message| UiBindingDispatchError::Rejected {
            action: name.to_owned(),
            message,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiBindingRegistrationError {
    Empty,
    Duplicate(String),
}

impl fmt::Display for UiBindingRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("binding name must not be empty"),
            Self::Duplicate(name) => write!(formatter, "binding {name:?} is already registered"),
        }
    }
}

impl Error for UiBindingRegistrationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiBindingDispatchError {
    UnknownAction(String),
    PayloadType {
        action: String,
        expected: UiValueType,
    },
    Rejected {
        action: String,
        message: String,
    },
}

impl fmt::Display for UiBindingDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownAction(action) => {
                write!(formatter, "unknown UI action binding {action:?}")
            }
            Self::PayloadType { action, expected } => {
                write!(
                    formatter,
                    "UI action {action:?} expects a {expected:?} payload"
                )
            }
            Self::Rejected { action, message } => {
                write!(
                    formatter,
                    "UI action {action:?} rejected its payload: {message}"
                )
            }
        }
    }
}

impl Error for UiBindingDispatchError {}

/// Cargo capabilities available to a validation or preview run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiFeatureSet {
    names: BTreeSet<String>,
}

impl UiFeatureSet {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }

    pub fn default_profile() -> Self {
        Self::new(crate::feature_manifest::zsui_default_feature_names())
    }

    pub fn compiled() -> Self {
        let mut names = BTreeSet::new();
        macro_rules! include_feature {
            ($name:literal) => {
                if cfg!(feature = $name) {
                    names.insert($name.to_owned());
                }
            };
        }
        include_feature!("button");
        include_feature!("breadcrumb");
        include_feature!("canvas");
        include_feature!("flyout");
        include_feature!("menu-flyout");
        include_feature!("toggle-button");
        include_feature!("label");
        include_feature!("grid");
        include_feature!("grid-view");
        include_feature!("scroll");
        include_feature!("list");
        include_feature!("virtual-list");
        include_feature!("textbox");
        include_feature!("password-box");
        include_feature!("tooltip");
        include_feature!("dialog");
        include_feature!("toast");
        include_feature!("info-bar");
        include_feature!("teaching-tip");
        include_feature!("checkbox");
        include_feature!("toggle");
        include_feature!("slider");
        include_feature!("number-box");
        include_feature!("radio");
        include_feature!("progress");
        include_feature!("progress-ring");
        include_feature!("auto-suggest");
        include_feature!("command-palette");
        include_feature!("tree");
        include_feature!("combo");
        include_feature!("date-picker");
        include_feature!("time-picker");
        include_feature!("color-picker");
        include_feature!("tabs");
        include_feature!("table");
        include_feature!("shell");
        include_feature!("workbench");
        include_feature!("document-shell");
        include_feature!("image");
        include_feature!("image-preview");
        Self { names }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn insert(&mut self, name: impl Into<String>) {
        self.names.insert(name.into());
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDiagnosticCode {
    IncompatibleSchema,
    InvalidNodeId,
    DuplicateNodeId,
    WidgetIdCollision,
    UnknownComponent,
    ComponentNotDocumentReady,
    MissingFeature,
    UnknownProperty,
    InvalidPropertyType,
    MissingRequiredProperty,
    ConflictingPropertySource,
    UnknownAction,
    UnresolvedPropertyBinding,
    UnresolvedActionBinding,
    BindingTypeMismatch,
    InvalidLayout,
    InvalidThemeToken,
    InvalidLocalization,
    InvalidAccessibility,
    InvalidChildCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiDiagnostic {
    pub code: UiDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiValidationReport {
    pub diagnostics: Vec<UiDiagnostic>,
}

impl UiValidationReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn into_result(self) -> Result<(), UiDocumentValidationError> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(UiDocumentValidationError {
                diagnostics: self.diagnostics,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDocumentValidationError {
    pub diagnostics: Vec<UiDiagnostic>,
}

impl fmt::Display for UiDocumentValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "UI document validation failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for UiDocumentValidationError {}

pub struct UiDocumentValidator<'a> {
    features: &'a UiFeatureSet,
    bindings: &'a UiBindingSchema,
}

impl<'a> UiDocumentValidator<'a> {
    pub const fn new(features: &'a UiFeatureSet, bindings: &'a UiBindingSchema) -> Self {
        Self { features, bindings }
    }

    pub fn validate(&self, document: &UiDocument) -> UiValidationReport {
        let mut diagnostics = Vec::new();
        if document.schema_version != ZSUI_UI_DOCUMENT_SCHEMA_VERSION {
            diagnostics.push(UiDiagnostic {
                code: UiDiagnosticCode::IncompatibleSchema,
                path: "$.schema_version".to_owned(),
                message: format!(
                    "schema version {} is not supported; expected {}",
                    document.schema_version, ZSUI_UI_DOCUMENT_SCHEMA_VERSION
                ),
            });
        }

        let mut node_ids = BTreeMap::new();
        let mut widget_ids = BTreeMap::new();
        self.validate_node(
            &document.root,
            "$.root",
            &mut node_ids,
            &mut widget_ids,
            &mut diagnostics,
        );
        UiValidationReport { diagnostics }
    }

    fn validate_node(
        &self,
        node: &UiNode,
        path: &str,
        node_ids: &mut BTreeMap<String, String>,
        widget_ids: &mut BTreeMap<u64, (String, String)>,
        diagnostics: &mut Vec<UiDiagnostic>,
    ) {
        if !is_valid_node_id(node.id.as_str()) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidNodeId,
                format!("{path}.id"),
                format!("invalid stable node id {:?}", node.id.as_str()),
            );
        } else if let Some(first_path) = node_ids.insert(node.id.0.clone(), path.to_owned()) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::DuplicateNodeId,
                format!("{path}.id"),
                format!(
                    "node id {:?} is already used at {first_path}",
                    node.id.as_str()
                ),
            );
        }
        let widget_id = node.id.widget_id().0;
        if let Some((first_node_id, first_path)) =
            widget_ids.insert(widget_id, (node.id.0.clone(), path.to_owned()))
        {
            if first_node_id != node.id.0 {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::WidgetIdCollision,
                    format!("{path}.id"),
                    format!(
                        "node id {:?} collides with {first_node_id:?} at {first_path}",
                        node.id.as_str()
                    ),
                );
            }
        }

        let catalog = crate::component_catalog::zsui_component_catalog();
        let descriptor = catalog
            .iter()
            .find(|descriptor| descriptor.component_name == node.component);
        if descriptor.is_none() {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::UnknownComponent,
                format!("{path}.component"),
                format!("unknown component {:?}", node.component),
            );
        }
        if let Some(feature) = descriptor.and_then(|descriptor| descriptor.feature_name) {
            if !self.features.contains(feature) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::MissingFeature,
                    format!("{path}.component"),
                    format!(
                        "component {:?} requires unavailable Cargo feature {feature:?}",
                        node.component
                    ),
                );
            }
        }

        match component_schema(&node.component) {
            Some(schema) => self.validate_component(node, path, schema, diagnostics),
            None if descriptor.is_some() => push_diagnostic(
                diagnostics,
                UiDiagnosticCode::ComponentNotDocumentReady,
                format!("{path}.component"),
                format!(
                    "component {:?} is known to ZSUI but is not available in UiDocument schema v{}",
                    node.component, ZSUI_UI_DOCUMENT_SCHEMA_VERSION
                ),
            ),
            None => {}
        }

        validate_layout(&node.layout, path, diagnostics);
        validate_theme_tokens(&node.theme_tokens, path, diagnostics);
        if let Some(accessibility) = &node.accessibility {
            validate_accessibility(accessibility, path, diagnostics);
        }

        for (index, child) in node.children.iter().enumerate() {
            self.validate_node(
                child,
                &format!("{path}.children[{index}]"),
                node_ids,
                widget_ids,
                diagnostics,
            );
        }
    }

    fn validate_component(
        &self,
        node: &UiNode,
        path: &str,
        schema: ComponentSchema,
        diagnostics: &mut Vec<UiDiagnostic>,
    ) {
        match schema.children {
            ChildPolicy::None if !node.children.is_empty() => push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidChildCount,
                format!("{path}.children"),
                format!("component {:?} does not accept children", node.component),
            ),
            ChildPolicy::AtMost(maximum) if node.children.len() > maximum => push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidChildCount,
                format!("{path}.children"),
                format!(
                    "component {:?} accepts at most {maximum} child node(s)",
                    node.component
                ),
            ),
            ChildPolicy::Any | ChildPolicy::AtMost(_) | ChildPolicy::None => {}
        }

        for (name, value) in &node.properties {
            match find_property(schema, name) {
                Some(property) if !property.value_type.matches(value) => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyType,
                    format!("{path}.properties.{name}"),
                    format!(
                        "property {name:?} on {:?} expects {:?}",
                        node.component, property.value_type
                    ),
                ),
                Some(_) => {}
                None => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnknownProperty,
                    format!("{path}.properties.{name}"),
                    format!("property {name:?} is not valid on {:?}", node.component),
                ),
            }
        }

        for (property_name, binding_name) in &node.property_bindings {
            let property = find_property(schema, property_name);
            if property.is_none() {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnknownProperty,
                    format!("{path}.property_bindings.{property_name}"),
                    format!(
                        "property {property_name:?} is not valid on {:?}",
                        node.component
                    ),
                );
                continue;
            }
            if node.properties.contains_key(property_name)
                || node.localization.contains_key(property_name)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::ConflictingPropertySource,
                    format!("{path}.property_bindings.{property_name}"),
                    format!(
                        "property {property_name:?} must use exactly one literal, localization key or state binding"
                    ),
                );
            }
            match self.bindings.properties.get(binding_name) {
                Some(value_type) if *value_type != property.unwrap().value_type => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::BindingTypeMismatch,
                    format!("{path}.property_bindings.{property_name}"),
                    format!(
                        "binding {binding_name:?} has type {value_type:?}; property {property_name:?} expects {:?}",
                        property.unwrap().value_type
                    ),
                ),
                Some(_) => {}
                None => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnresolvedPropertyBinding,
                    format!("{path}.property_bindings.{property_name}"),
                    format!("state binding {binding_name:?} is not declared"),
                ),
            }
        }

        for (property_name, message_key) in &node.localization {
            if find_property(schema, property_name).is_none() {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnknownProperty,
                    format!("{path}.localization.{property_name}"),
                    format!(
                        "property {property_name:?} is not valid on {:?}",
                        node.component
                    ),
                );
            } else if message_key.trim().is_empty() {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.{property_name}"),
                    "localization message key must not be empty".to_owned(),
                );
            }
            if node.properties.contains_key(property_name) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::ConflictingPropertySource,
                    format!("{path}.localization.{property_name}"),
                    format!(
                        "property {property_name:?} must use exactly one literal, localization key or state binding"
                    ),
                );
            }
        }

        for property in schema.properties {
            if property.required
                && !node.properties.contains_key(property.name)
                && !node.property_bindings.contains_key(property.name)
                && !node.localization.contains_key(property.name)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::MissingRequiredProperty,
                    format!("{path}.properties"),
                    format!(
                        "component {:?} requires property {:?}",
                        node.component, property.name
                    ),
                );
            }
        }

        for (action_name, binding_name) in &node.action_bindings {
            match find_action(schema, action_name) {
                Some(action) => match self.bindings.actions.get(binding_name) {
                    Some(value_type) if *value_type != action.payload_type => push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::BindingTypeMismatch,
                        format!("{path}.action_bindings.{action_name}"),
                        format!(
                            "binding {binding_name:?} has payload type {value_type:?}; action {action_name:?} expects {:?}",
                            action.payload_type
                        ),
                    ),
                    Some(_) => {}
                    None => push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::UnresolvedActionBinding,
                        format!("{path}.action_bindings.{action_name}"),
                        format!("action binding {binding_name:?} is not declared"),
                    ),
                },
                None => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnknownAction,
                    format!("{path}.action_bindings.{action_name}"),
                    format!("action {action_name:?} is not valid on {:?}", node.component),
                ),
            }
        }
    }
}

#[derive(Clone, Copy)]
struct PropertySpec {
    name: &'static str,
    value_type: UiValueType,
    required: bool,
}

#[derive(Clone, Copy)]
struct ActionSpec {
    name: &'static str,
    payload_type: UiValueType,
}

#[derive(Clone, Copy)]
enum ChildPolicy {
    Any,
    AtMost(usize),
    None,
}

#[derive(Clone, Copy)]
struct ComponentSchema {
    properties: &'static [PropertySpec],
    actions: &'static [ActionSpec],
    children: ChildPolicy,
}

const NO_PROPERTIES: &[PropertySpec] = &[];
const NO_ACTIONS: &[ActionSpec] = &[];
const TEXT_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "text",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "text_role",
        value_type: UiValueType::String,
        required: false,
    },
];
const BUTTON_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "label",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "enabled",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const BUTTON_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "click",
    payload_type: UiValueType::Null,
}];
const CHECKED_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "label",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "checked",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const TOGGLE_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "toggle",
    payload_type: UiValueType::Boolean,
}];
const TEXTBOX_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "placeholder",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "multiline",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const TEXTBOX_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "change",
    payload_type: UiValueType::String,
}];
const RADIO_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "label",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const RADIO_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "choose",
    payload_type: UiValueType::Null,
}];
const VALUE_PROPERTIES: &[PropertySpec] = &[PropertySpec {
    name: "value",
    value_type: UiValueType::Number,
    required: true,
}];
const SLIDER_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "slide",
    payload_type: UiValueType::Number,
}];

fn component_schema(component: &str) -> Option<ComponentSchema> {
    let leaf = |properties, actions| ComponentSchema {
        properties,
        actions,
        children: ChildPolicy::None,
    };
    match component {
        "stack" => Some(ComponentSchema {
            properties: NO_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::Any,
        }),
        "border" => Some(ComponentSchema {
            properties: NO_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::AtMost(1),
        }),
        "text" => Some(leaf(TEXT_PROPERTIES, NO_ACTIONS)),
        "button" => Some(leaf(BUTTON_PROPERTIES, BUTTON_ACTIONS)),
        "toggle_button" | "checkbox" | "toggle" => Some(leaf(CHECKED_PROPERTIES, TOGGLE_ACTIONS)),
        "textbox" => Some(leaf(TEXTBOX_PROPERTIES, TEXTBOX_ACTIONS)),
        "radio_button" => Some(leaf(RADIO_PROPERTIES, RADIO_ACTIONS)),
        "slider" => Some(leaf(VALUE_PROPERTIES, SLIDER_ACTIONS)),
        "progress_bar" => Some(leaf(VALUE_PROPERTIES, NO_ACTIONS)),
        _ => None,
    }
}

fn find_property(schema: ComponentSchema, name: &str) -> Option<PropertySpec> {
    schema
        .properties
        .iter()
        .copied()
        .find(|property| property.name == name)
}

fn find_action(schema: ComponentSchema, name: &str) -> Option<ActionSpec> {
    schema
        .actions
        .iter()
        .copied()
        .find(|action| action.name == name)
}

fn validate_binding_name(name: String) -> Result<String, UiBindingRegistrationError> {
    if name.trim().is_empty() {
        Err(UiBindingRegistrationError::Empty)
    } else {
        Ok(name)
    }
}

fn is_valid_node_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-' | '.'))
}

fn validate_layout(layout: &UiLayout, path: &str, diagnostics: &mut Vec<UiDiagnostic>) {
    let non_negative = [
        ("width", layout.width),
        ("height", layout.height),
        ("min_width", layout.min_width),
        ("min_height", layout.min_height),
        ("max_width", layout.max_width),
        ("max_height", layout.max_height),
        ("padding", layout.padding),
        ("gap", layout.gap),
    ];
    for (name, value) in non_negative {
        if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidLayout,
                format!("{path}.layout.{name}"),
                format!("layout value {name:?} must be finite and non-negative"),
            );
        }
    }
    if layout
        .flex
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        push_diagnostic(
            diagnostics,
            UiDiagnosticCode::InvalidLayout,
            format!("{path}.layout.flex"),
            "layout flex must be finite and greater than zero".to_owned(),
        );
    }
    for (minimum_name, minimum, maximum_name, maximum) in [
        ("min_width", layout.min_width, "max_width", layout.max_width),
        (
            "min_height",
            layout.min_height,
            "max_height",
            layout.max_height,
        ),
    ] {
        if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
            if minimum > maximum {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLayout,
                    format!("{path}.layout.{maximum_name}"),
                    format!("{maximum_name} must be greater than or equal to {minimum_name}"),
                );
            }
        }
    }
}

fn validate_theme_tokens(
    tokens: &BTreeMap<String, String>,
    path: &str,
    diagnostics: &mut Vec<UiDiagnostic>,
) {
    const SLOTS: &[&str] = &["background", "foreground", "border", "accent"];
    for (slot, token) in tokens {
        if !SLOTS.contains(&slot.as_str()) || token.trim().is_empty() {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidThemeToken,
                format!("{path}.theme_tokens.{slot}"),
                format!(
                    "theme token slot must be one of {} and its semantic token name must not be empty",
                    SLOTS.join(", ")
                ),
            );
        }
    }
}

fn validate_accessibility(
    accessibility: &UiAccessibility,
    path: &str,
    diagnostics: &mut Vec<UiDiagnostic>,
) {
    for (name, value) in [
        ("role", accessibility.role.as_deref()),
        ("label", accessibility.label.as_deref()),
        ("description", accessibility.description.as_deref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidAccessibility,
                format!("{path}.accessibility.{name}"),
                format!("accessibility field {name:?} must not be empty"),
            );
        }
    }
}

fn push_diagnostic(
    diagnostics: &mut Vec<UiDiagnostic>,
    code: UiDiagnosticCode,
    path: String,
    message: String,
) {
    diagnostics.push(UiDiagnostic {
        code,
        path,
        message,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct State {
        title: String,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Msg {
        Save,
    }

    fn valid_document() -> UiDocument {
        UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "root",
                "component": "stack",
                "layout": { "gap": 8.0, "direction": "vertical" },
                "children": [
                  {
                    "id": "title",
                    "component": "text",
                    "property_bindings": { "text": "window_title" },
                    "theme_tokens": { "foreground": "text.primary" },
                    "accessibility": { "role": "heading", "label": "Title" }
                  },
                  {
                    "id": "save-button",
                    "component": "button",
                    "properties": { "label": "Save", "enabled": true },
                    "action_bindings": { "click": "save" }
                  }
                ]
              }
            }"#,
        )
        .expect("valid fixture should parse")
    }

    #[test]
    fn typed_manifest_validates_and_dispatches_concrete_messages() {
        let mut manifest = UiBindingManifest::<State, Msg>::new();
        manifest
            .register_property("window_title", UiValueType::String, |state| {
                Value::String(state.title.clone())
            })
            .unwrap();
        manifest
            .register_action("save", UiValueType::Null, |_| Ok(Msg::Save))
            .unwrap();
        let features = UiFeatureSet::new(["button", "label"]);

        let report = valid_document().validate(&features, &manifest.schema());

        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert_eq!(
            manifest.read_property(
                "window_title",
                &State {
                    title: "Notes".to_owned()
                }
            ),
            Some(Value::String("Notes".to_owned()))
        );
        assert_eq!(manifest.map_action("save", Value::Null), Ok(Msg::Save));
    }

    #[test]
    fn document_widget_ids_are_stable_distinct_and_reserved() {
        let root = UiNodeId::new("root").unwrap().widget_id();
        let root_again = UiNodeId::new("root").unwrap().widget_id();
        let child = UiNodeId::new("root.child").unwrap().widget_id();

        assert_eq!(root, root_again);
        assert_ne!(root, child);
        assert_eq!(
            root.0 & DOCUMENT_WIDGET_ID_NAMESPACE,
            DOCUMENT_WIDGET_ID_NAMESPACE
        );
        assert_eq!(root.0 & (1 << 63), 0);
    }

    #[test]
    fn validator_reports_schema_ids_features_properties_and_bindings() {
        let mut document = valid_document();
        document.schema_version = 99;
        document.root.children[1].id = UiNodeId("title".to_owned());
        document.root.children[1]
            .properties
            .insert("enabled".to_owned(), Value::String("yes".to_owned()));
        document.root.children[1]
            .action_bindings
            .insert("missing".to_owned(), "unknown".to_owned());

        let report = document.validate(&UiFeatureSet::new(["label"]), &UiBindingSchema::default());
        let codes = report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();

        assert!(codes.contains(&UiDiagnosticCode::IncompatibleSchema));
        assert!(codes.contains(&UiDiagnosticCode::DuplicateNodeId));
        assert!(codes.contains(&UiDiagnosticCode::MissingFeature));
        assert!(codes.contains(&UiDiagnosticCode::InvalidPropertyType));
        assert!(codes.contains(&UiDiagnosticCode::UnknownAction));
        assert!(codes.contains(&UiDiagnosticCode::UnresolvedPropertyBinding));
    }

    #[test]
    fn validator_distinguishes_unknown_and_not_yet_document_ready_components() {
        let mut document = valid_document();
        document.root.children[0].component = "tabs".to_owned();
        document.root.children[1].component = "imaginary".to_owned();
        let report = document.validate(
            &UiFeatureSet::new(["button", "label", "tabs"]),
            &UiBindingSchema::default(),
        );

        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == UiDiagnosticCode::ComponentNotDocumentReady }));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::UnknownComponent));
    }

    #[test]
    fn parser_rejects_unknown_structural_fields() {
        let error = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": { "id": "root", "component": "stack", "surprise": true }
            }"#,
        )
        .expect_err("unknown schema fields must fail parsing");

        assert!(error.to_string().contains("unknown field"));
    }
}
