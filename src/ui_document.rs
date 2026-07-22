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
/// Schema version of the deterministic AI authoring handoff manifest.
pub const ZSUI_UI_AI_HANDOFF_SCHEMA_VERSION: u32 = 1;
/// Binary format version for release-embedded validated documents.
pub const ZSUI_UI_DOCUMENT_ARTIFACT_VERSION: u32 = 1;
const UI_DOCUMENT_ARTIFACT_MAGIC: &[u8; 8] = b"ZSUIUID\0";
const UI_DOCUMENT_ARTIFACT_HEADER_LENGTH: usize = 32;
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
        let hash = fnv1a64(self.0.as_bytes());
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

/// One platform-neutral track in a document-backed Grid declaration.
///
/// Fixed sizes use device-independent pixels. Fractional weights are positive
/// integers so the document never depends on a backend-specific layout type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UiGridTrack {
    Fixed { size: f32 },
    Fraction { weight: u16 },
}

impl UiGridTrack {
    pub(crate) fn is_valid(self) -> bool {
        match self {
            Self::Fixed { size } => size.is_finite() && size >= 0.0,
            Self::Fraction { weight } => weight > 0,
        }
    }
}

/// Stable child placement used by a document-backed Grid.
///
/// Grid properties key these values by child [`UiNodeId`], so inserting or
/// reordering siblings does not silently move an existing child to another
/// cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiGridPlacement {
    pub row: usize,
    pub column: usize,
    #[serde(default = "one_grid_span", skip_serializing_if = "is_one_grid_span")]
    pub row_span: u16,
    #[serde(default = "one_grid_span", skip_serializing_if = "is_one_grid_span")]
    pub column_span: u16,
}

impl UiGridPlacement {
    pub const fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            row_span: 1,
            column_span: 1,
        }
    }

    pub const fn with_spans(mut self, row_span: u16, column_span: u16) -> Self {
        self.row_span = row_span;
        self.column_span = column_span;
        self
    }

    pub(crate) const fn is_valid(self) -> bool {
        self.row_span > 0 && self.column_span > 0
    }
}

const fn one_grid_span() -> u16 {
    1
}

fn is_one_grid_span(value: &u16) -> bool {
    *value == 1
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
    NullableNumber,
    Integer,
    NullableInteger,
    String,
    StringArray,
    StringMap,
    GridTrackArray,
    GridPlacementMap,
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
            Self::NullableNumber => value.is_null() || value.is_number(),
            Self::Integer => value.as_u64().is_some(),
            Self::NullableInteger => value.is_null() || value.as_u64().is_some(),
            Self::String => value.is_string(),
            Self::StringArray => value
                .as_array()
                .is_some_and(|values| values.iter().all(Value::is_string)),
            Self::StringMap => value
                .as_object()
                .is_some_and(|values| values.values().all(Value::is_string)),
            Self::GridTrackArray => grid_tracks_from_value(value).is_some(),
            Self::GridPlacementMap => grid_placements_from_value(value).is_some(),
            Self::Array => value.is_array(),
            Self::Object => value.is_object(),
            Self::Any => true,
        }
    }
}

fn grid_tracks_from_value(value: &Value) -> Option<Vec<UiGridTrack>> {
    let tracks = serde_json::from_value::<Vec<UiGridTrack>>(value.clone()).ok()?;
    (!tracks.is_empty() && tracks.iter().copied().all(UiGridTrack::is_valid)).then_some(tracks)
}

fn grid_placements_from_value(value: &Value) -> Option<BTreeMap<String, UiGridPlacement>> {
    let placements =
        serde_json::from_value::<BTreeMap<String, UiGridPlacement>>(value.clone()).ok()?;
    placements
        .values()
        .copied()
        .all(UiGridPlacement::is_valid)
        .then_some(placements)
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

/// Validates an authoring value snapshot against declared property bindings.
///
/// Missing values are permitted so a preview can expose unresolved data. Extra
/// values and values of the wrong JSON shape are rejected deterministically.
pub fn validate_ui_binding_values(
    bindings: &UiBindingSchema,
    values: &BTreeMap<String, Value>,
) -> UiValidationReport {
    let mut diagnostics = Vec::new();
    for (name, value) in values {
        match bindings.properties.get(name) {
            None => push_diagnostic(
                &mut diagnostics,
                UiDiagnosticCode::UnknownBindingValue,
                format!("$.{name}"),
                format!("property binding value {name:?} is not declared"),
            ),
            Some(value_type) if !value_type.matches(value) => push_diagnostic(
                &mut diagnostics,
                UiDiagnosticCode::BindingValueTypeMismatch,
                format!("$.{name}"),
                format!("property binding value {name:?} must be {value_type:?}"),
            ),
            Some(_) => {}
        }
    }
    UiValidationReport { diagnostics }
}

/// One deterministic file entry in an AI authoring handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffFile {
    pub path: String,
    pub byte_length: u64,
    /// Stable change fingerprint; this is not a cryptographic integrity hash.
    pub content_fingerprint: String,
}

/// Optional final native-view PNG attached to an authoring handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffPreview {
    #[serde(flatten)]
    pub file: UiAiHandoffFile,
    pub media_type: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffFiles {
    pub document: UiAiHandoffFile,
    pub bindings: UiAiHandoffFile,
    pub values: Option<UiAiHandoffFile>,
    pub preview: Option<UiAiHandoffPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffFramework {
    pub name: String,
    pub version: String,
    pub producer: String,
}

/// Stable node index that lets an authoring tool address structure without
/// understanding any platform backend implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffNode {
    pub path: String,
    pub id: String,
    pub widget_id: u64,
    pub component: String,
    pub inline_properties: Vec<String>,
    pub property_bindings: BTreeMap<String, String>,
    pub action_bindings: BTreeMap<String, String>,
    pub child_count: usize,
    pub has_accessibility: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffPropertyContract {
    pub name: String,
    pub value_type: UiValueType,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffActionContract {
    pub name: String,
    pub payload_type: UiValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiAiHandoffChildPolicy {
    Any,
    AtLeast { minimum: usize },
    Exactly { count: usize },
    AtMost { maximum: usize },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffComponentContract {
    pub component: String,
    pub cargo_feature: Option<String>,
    pub properties: Vec<UiAiHandoffPropertyContract>,
    pub actions: Vec<UiAiHandoffActionContract>,
    pub children: UiAiHandoffChildPolicy,
}

/// Deterministic machine-readable index accompanying an editable UI document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAiHandoffManifest {
    pub handoff_schema_version: u32,
    pub document_schema_version: u32,
    pub framework: UiAiHandoffFramework,
    pub files: UiAiHandoffFiles,
    pub required_features: Vec<String>,
    pub provided_values: Vec<String>,
    pub missing_values: Vec<String>,
    pub nodes: Vec<UiAiHandoffNode>,
    pub component_contracts: Vec<UiAiHandoffComponentContract>,
}

/// Fully canonicalized in-memory handoff. File I/O remains the caller's
/// responsibility so library users can store it in any build or release flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAiHandoffPackage {
    pub manifest: UiAiHandoffManifest,
    pub handoff_json: String,
    pub document_json: String,
    pub bindings_json: String,
    pub values_json: Option<String>,
    pub preview_png: Option<Vec<u8>>,
}

impl UiAiHandoffPackage {
    pub fn build(
        document: &UiDocument,
        features: &UiFeatureSet,
        bindings: &UiBindingSchema,
        values: Option<&BTreeMap<String, Value>>,
        preview_png: Option<&[u8]>,
    ) -> Result<Self, UiAiHandoffBuildError> {
        let document_report = document.validate(features, bindings);
        if !document_report.is_valid() {
            return Err(UiAiHandoffBuildError::InvalidDocument(document_report));
        }
        if let Some(values) = values {
            let report = validate_ui_binding_values(bindings, values);
            if !report.is_valid() {
                return Err(UiAiHandoffBuildError::InvalidValues(report));
            }
        }
        let document_json = canonical_pretty_json(document)?;
        let bindings_json = canonical_pretty_json(bindings)?;
        let values_json = values.map(canonical_pretty_json).transpose()?;

        let document_file = handoff_file("document.json", document_json.as_bytes());
        let bindings_file = handoff_file("bindings.json", bindings_json.as_bytes());
        let values_file = values_json
            .as_ref()
            .map(|source| handoff_file("values.json", source.as_bytes()));
        let preview = preview_png
            .map(|bytes| {
                let (width, height) = png_dimensions(bytes)?;
                Ok::<_, UiAiHandoffBuildError>(UiAiHandoffPreview {
                    file: handoff_file("preview.png", bytes),
                    media_type: "image/png".to_owned(),
                    width,
                    height,
                })
            })
            .transpose()?;

        let mut nodes = Vec::new();
        collect_handoff_nodes(&document.root, "$.root", &mut nodes);
        let component_names = nodes
            .iter()
            .map(|node| node.component.clone())
            .collect::<BTreeSet<_>>();
        let catalog = crate::component_catalog::zsui_component_catalog();
        let mut required_features = BTreeSet::from(["ui-document".to_owned()]);
        let mut component_contracts = Vec::with_capacity(component_names.len());
        for component in component_names {
            let cargo_feature = catalog
                .iter()
                .find(|descriptor| descriptor.component_name == component)
                .and_then(|descriptor| descriptor.feature_name)
                .map(str::to_owned);
            if let Some(feature) = &cargo_feature {
                required_features.insert(feature.clone());
            }
            if let Some(schema) = component_schema(&component) {
                let mut properties = schema
                    .properties
                    .iter()
                    .map(|property| UiAiHandoffPropertyContract {
                        name: property.name.to_owned(),
                        value_type: property.value_type,
                        required: property.required,
                    })
                    .collect::<Vec<_>>();
                properties.sort_by(|left, right| left.name.cmp(&right.name));
                let mut actions = schema
                    .actions
                    .iter()
                    .map(|action| UiAiHandoffActionContract {
                        name: action.name.to_owned(),
                        payload_type: action.payload_type,
                    })
                    .collect::<Vec<_>>();
                actions.sort_by(|left, right| left.name.cmp(&right.name));
                component_contracts.push(UiAiHandoffComponentContract {
                    component,
                    cargo_feature,
                    properties,
                    actions,
                    children: match schema.children {
                        ChildPolicy::Any => UiAiHandoffChildPolicy::Any,
                        ChildPolicy::AtLeast(minimum) => {
                            UiAiHandoffChildPolicy::AtLeast { minimum }
                        }
                        ChildPolicy::Exactly(count) => UiAiHandoffChildPolicy::Exactly { count },
                        ChildPolicy::AtMost(maximum) => UiAiHandoffChildPolicy::AtMost { maximum },
                        ChildPolicy::None => UiAiHandoffChildPolicy::None,
                    },
                });
            }
        }

        let provided_values = values
            .into_iter()
            .flat_map(|values| values.keys().cloned())
            .collect::<Vec<_>>();
        let missing_values = bindings
            .properties
            .keys()
            .filter(|name| values.is_none_or(|values| !values.contains_key(*name)))
            .cloned()
            .collect();
        let manifest = UiAiHandoffManifest {
            handoff_schema_version: ZSUI_UI_AI_HANDOFF_SCHEMA_VERSION,
            document_schema_version: document.schema_version,
            framework: UiAiHandoffFramework {
                name: "zsui".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                producer: "zsui-uic".to_owned(),
            },
            files: UiAiHandoffFiles {
                document: document_file,
                bindings: bindings_file,
                values: values_file,
                preview,
            },
            required_features: required_features.into_iter().collect(),
            provided_values,
            missing_values,
            nodes,
            component_contracts,
        };
        let handoff_json = canonical_pretty_json(&manifest)?;
        Ok(Self {
            manifest,
            handoff_json,
            document_json,
            bindings_json,
            values_json,
            preview_png: preview_png.map(<[u8]>::to_vec),
        })
    }
}

#[derive(Debug)]
pub enum UiAiHandoffBuildError {
    Serialize(serde_json::Error),
    InvalidDocument(UiValidationReport),
    InvalidValues(UiValidationReport),
    InvalidPreviewPng,
}

impl fmt::Display for UiAiHandoffBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "cannot serialize AI handoff: {error}"),
            Self::InvalidDocument(report) => write!(
                formatter,
                "AI handoff document failed validation with {} diagnostic(s)",
                report.diagnostics.len()
            ),
            Self::InvalidValues(report) => write!(
                formatter,
                "AI handoff values failed validation with {} diagnostic(s)",
                report.diagnostics.len()
            ),
            Self::InvalidPreviewPng => {
                formatter.write_str("AI handoff preview must be a PNG with a valid IHDR")
            }
        }
    }
}

impl Error for UiAiHandoffBuildError {}

impl From<serde_json::Error> for UiAiHandoffBuildError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

/// Deterministic release artifact produced after schema, feature and binding
/// validation. It contains no source path, watcher state, preview or timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDocumentReleaseArtifact {
    bytes: Vec<u8>,
    content_fingerprint: String,
}

impl UiDocumentReleaseArtifact {
    pub fn compile(
        document: &UiDocument,
        features: &UiFeatureSet,
        bindings: &UiBindingSchema,
    ) -> Result<Self, UiDocumentArtifactError> {
        let report = document.validate(features, bindings);
        if !report.is_valid() {
            return Err(UiDocumentArtifactError::Validation(report));
        }
        let document_json = canonical_pretty_json(document)?;
        let bindings_json = canonical_pretty_json(bindings)?;
        let document_length = u32::try_from(document_json.len())
            .map_err(|_| UiDocumentArtifactError::ArtifactTooLarge)?;
        let bindings_length = u32::try_from(bindings_json.len())
            .map_err(|_| UiDocumentArtifactError::ArtifactTooLarge)?;
        let payload_length = document_json
            .len()
            .checked_add(bindings_json.len())
            .ok_or(UiDocumentArtifactError::ArtifactTooLarge)?;
        let total_length = UI_DOCUMENT_ARTIFACT_HEADER_LENGTH
            .checked_add(payload_length)
            .ok_or(UiDocumentArtifactError::ArtifactTooLarge)?;
        let mut bytes = Vec::with_capacity(total_length);
        bytes.extend_from_slice(UI_DOCUMENT_ARTIFACT_MAGIC);
        bytes.extend_from_slice(&ZSUI_UI_DOCUMENT_ARTIFACT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&document.schema_version.to_le_bytes());
        bytes.extend_from_slice(&document_length.to_le_bytes());
        bytes.extend_from_slice(&bindings_length.to_le_bytes());
        let payload_fingerprint = fnv1a64_two(document_json.as_bytes(), bindings_json.as_bytes());
        bytes.extend_from_slice(&payload_fingerprint.to_le_bytes());
        bytes.extend_from_slice(document_json.as_bytes());
        bytes.extend_from_slice(bindings_json.as_bytes());
        let content_fingerprint = format!("fnv1a64:{:016x}", fnv1a64(&bytes));
        Ok(Self {
            bytes,
            content_fingerprint,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Stable change fingerprint; this is not a cryptographic integrity hash.
    pub fn content_fingerprint(&self) -> &str {
        &self.content_fingerprint
    }
}

/// Decoded release document ready for the optional `ui-document-runtime`
/// compiler or another application-owned typed integration.
#[derive(Debug, Clone, PartialEq)]
pub struct UiEmbeddedDocument {
    pub document: UiDocument,
    pub bindings: UiBindingSchema,
}

impl UiEmbeddedDocument {
    pub fn decode(
        bytes: &[u8],
        features: &UiFeatureSet,
        expected_bindings: &UiBindingSchema,
    ) -> Result<Self, UiDocumentArtifactError> {
        if bytes.len() < UI_DOCUMENT_ARTIFACT_HEADER_LENGTH
            || &bytes[..UI_DOCUMENT_ARTIFACT_MAGIC.len()] != UI_DOCUMENT_ARTIFACT_MAGIC
        {
            return Err(UiDocumentArtifactError::InvalidHeader);
        }
        let artifact_version = read_artifact_u32(bytes, 8);
        if artifact_version != ZSUI_UI_DOCUMENT_ARTIFACT_VERSION {
            return Err(UiDocumentArtifactError::UnsupportedArtifactVersion(
                artifact_version,
            ));
        }
        let header_document_schema = read_artifact_u32(bytes, 12);
        let document_length = read_artifact_u32(bytes, 16) as usize;
        let bindings_length = read_artifact_u32(bytes, 20) as usize;
        let payload_fingerprint = read_artifact_u64(bytes, 24);
        let payload_length = document_length
            .checked_add(bindings_length)
            .ok_or(UiDocumentArtifactError::InvalidLength)?;
        if UI_DOCUMENT_ARTIFACT_HEADER_LENGTH.checked_add(payload_length) != Some(bytes.len()) {
            return Err(UiDocumentArtifactError::InvalidLength);
        }
        let document_start = UI_DOCUMENT_ARTIFACT_HEADER_LENGTH;
        let document_end = document_start + document_length;
        let document_bytes = &bytes[document_start..document_end];
        let bindings_bytes = &bytes[document_end..];
        if fnv1a64_two(document_bytes, bindings_bytes) != payload_fingerprint {
            return Err(UiDocumentArtifactError::FingerprintMismatch);
        }
        let document = serde_json::from_slice::<UiDocument>(document_bytes)
            .map_err(UiDocumentArtifactError::ParseDocument)?;
        let bindings = serde_json::from_slice::<UiBindingSchema>(bindings_bytes)
            .map_err(UiDocumentArtifactError::ParseBindings)?;
        if document.schema_version != header_document_schema {
            return Err(UiDocumentArtifactError::DocumentSchemaMismatch {
                header: header_document_schema,
                document: document.schema_version,
            });
        }
        if &bindings != expected_bindings {
            return Err(UiDocumentArtifactError::BindingSchemaMismatch);
        }
        let report = document.validate(features, &bindings);
        if !report.is_valid() {
            return Err(UiDocumentArtifactError::Validation(report));
        }
        Ok(Self { document, bindings })
    }
}

#[derive(Debug)]
pub enum UiDocumentArtifactError {
    Serialize(serde_json::Error),
    Validation(UiValidationReport),
    ArtifactTooLarge,
    InvalidHeader,
    UnsupportedArtifactVersion(u32),
    InvalidLength,
    FingerprintMismatch,
    ParseDocument(serde_json::Error),
    ParseBindings(serde_json::Error),
    DocumentSchemaMismatch { header: u32, document: u32 },
    BindingSchemaMismatch,
}

impl fmt::Display for UiDocumentArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "cannot serialize UI document: {error}"),
            Self::Validation(report) => write!(
                formatter,
                "UI document artifact validation failed with {} diagnostic(s)",
                report.diagnostics.len()
            ),
            Self::ArtifactTooLarge => formatter.write_str("UI document artifact is too large"),
            Self::InvalidHeader => formatter.write_str("invalid UI document artifact header"),
            Self::UnsupportedArtifactVersion(version) => write!(
                formatter,
                "UI document artifact version {version} is not supported"
            ),
            Self::InvalidLength => formatter.write_str("invalid UI document artifact length"),
            Self::FingerprintMismatch => {
                formatter.write_str("UI document artifact payload fingerprint mismatch")
            }
            Self::ParseDocument(error) => {
                write!(formatter, "cannot parse embedded UI document: {error}")
            }
            Self::ParseBindings(error) => {
                write!(formatter, "cannot parse embedded binding schema: {error}")
            }
            Self::DocumentSchemaMismatch { header, document } => write!(
                formatter,
                "embedded document schema {document} does not match artifact header {header}"
            ),
            Self::BindingSchemaMismatch => formatter.write_str(
                "embedded binding schema does not match the application binding manifest",
            ),
        }
    }
}

impl Error for UiDocumentArtifactError {}

impl From<serde_json::Error> for UiDocumentArtifactError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

fn read_artifact_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_artifact_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}

fn collect_handoff_nodes(node: &UiNode, path: &str, output: &mut Vec<UiAiHandoffNode>) {
    output.push(UiAiHandoffNode {
        path: path.to_owned(),
        id: node.id.as_str().to_owned(),
        widget_id: node.id.widget_id().0,
        component: node.component.clone(),
        inline_properties: node.properties.keys().cloned().collect(),
        property_bindings: node.property_bindings.clone(),
        action_bindings: node.action_bindings.clone(),
        child_count: node.children.len(),
        has_accessibility: node.accessibility.is_some(),
    });
    for (index, child) in node.children.iter().enumerate() {
        collect_handoff_nodes(child, &format!("{path}.children[{index}]"), output);
    }
}

fn canonical_pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|mut source| {
        source.push('\n');
        source
    })
}

fn handoff_file(path: &str, bytes: &[u8]) -> UiAiHandoffFile {
    UiAiHandoffFile {
        path: path.to_owned(),
        byte_length: bytes.len() as u64,
        content_fingerprint: format!("fnv1a64:{:016x}", fnv1a64(bytes)),
    }
}

fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), UiAiHandoffBuildError> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 33
        || &bytes[..8] != PNG_SIGNATURE
        || u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) != 13
        || &bytes[12..16] != b"IHDR"
    {
        return Err(UiAiHandoffBuildError::InvalidPreviewPng);
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 {
        return Err(UiAiHandoffBuildError::InvalidPreviewPng);
    }
    Ok((width, height))
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
    UnknownBindingValue,
    BindingValueTypeMismatch,
    InvalidPropertyValue,
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
            ChildPolicy::AtLeast(minimum) if node.children.len() < minimum => push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidChildCount,
                format!("{path}.children"),
                format!(
                    "component {:?} requires at least {minimum} child node(s)",
                    node.component
                ),
            ),
            ChildPolicy::Exactly(count) if node.children.len() != count => push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidChildCount,
                format!("{path}.children"),
                format!(
                    "component {:?} requires exactly {count} child node(s)",
                    node.component
                ),
            ),
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
            ChildPolicy::Any
            | ChildPolicy::AtLeast(_)
            | ChildPolicy::Exactly(_)
            | ChildPolicy::AtMost(_)
            | ChildPolicy::None => {}
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

        if node.component == "scroll" {
            for name in ["offset_y", "content_height"] {
                if node
                    .properties
                    .get(name)
                    .and_then(Value::as_f64)
                    .is_some_and(|value| value < 0.0)
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("scroll property {name:?} must not be negative"),
                    );
                }
            }
        }

        if node.component == "number_box" {
            let static_number = |name: &str, fallback: f64| {
                (!node.property_bindings.contains_key(name)).then(|| {
                    node.properties
                        .get(name)
                        .and_then(Value::as_f64)
                        .unwrap_or(fallback)
                })
            };
            let minimum = static_number("minimum", 0.0);
            let maximum = static_number("maximum", 100.0);
            if minimum
                .zip(maximum)
                .is_some_and(|(minimum, maximum)| minimum >= maximum)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.maximum"),
                    "number_box maximum must be greater than minimum".to_owned(),
                );
            }
            for name in ["step", "large_step"] {
                if node
                    .properties
                    .get(name)
                    .and_then(Value::as_f64)
                    .is_some_and(|value| value <= 0.0)
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("number_box property {name:?} must be greater than zero"),
                    );
                }
            }
            if node
                .properties
                .get("fraction_digits")
                .and_then(Value::as_f64)
                .is_some_and(|value| value.fract() != 0.0 || !(0.0..=12.0).contains(&value))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.fraction_digits"),
                    "number_box fraction_digits must be an integer from 0 through 12".to_owned(),
                );
            }
            if let (Some(value), Some(minimum), Some(maximum)) = (
                node.properties.get("value").and_then(Value::as_f64),
                minimum,
                maximum,
            ) {
                if minimum < maximum && !(minimum..=maximum).contains(&value) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.value"),
                        "number_box value must be within minimum and maximum".to_owned(),
                    );
                }
            }
        }

        if node.component == "combo_box" {
            if let (Some(options), Some(selected_index)) = (
                node.properties.get("options").and_then(Value::as_array),
                node.properties
                    .get("selected_index")
                    .and_then(Value::as_u64),
            ) {
                if usize::try_from(selected_index)
                    .ok()
                    .is_none_or(|selected_index| selected_index >= options.len())
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.selected_index"),
                        "combo_box selected_index must address an available option".to_owned(),
                    );
                }
            }
        }

        if node.component == "tabs" {
            let child_ids = node
                .children
                .iter()
                .map(|child| child.id.as_str())
                .collect::<BTreeSet<_>>();
            if let Some(labels) = node.properties.get("labels").and_then(Value::as_object) {
                for child_id in &child_ids {
                    if !labels.contains_key(*child_id) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.labels"),
                            format!("tabs labels must contain child id {child_id:?}"),
                        );
                    }
                }
                for label_id in labels.keys() {
                    if !child_ids.contains(label_id.as_str()) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.labels.{label_id}"),
                            format!("tabs label id {label_id:?} does not address a child"),
                        );
                    }
                }
            }
            if let Some(selected) = node.properties.get("selected").and_then(Value::as_str) {
                if !child_ids.contains(selected) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.selected"),
                        "tabs selected must address a child id".to_owned(),
                    );
                }
            }
            if let Some(icons) = node.properties.get("icons").and_then(Value::as_object) {
                for (icon_id, icon) in icons {
                    if !child_ids.contains(icon_id.as_str()) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.icons.{icon_id}"),
                            format!("tabs icon id {icon_id:?} does not address a child"),
                        );
                    } else if icon
                        .as_str()
                        .and_then(|icon| {
                            serde_json::from_value::<crate::ZsIcon>(Value::String(icon.to_owned()))
                                .ok()
                        })
                        .is_none()
                    {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.icons.{icon_id}"),
                            "tabs icon must name a ZsIcon semantic variant".to_owned(),
                        );
                    }
                }
            }
        }

        if node.component == "grid" {
            validate_grid_component(node, path, diagnostics);
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
    AtLeast(usize),
    Exactly(usize),
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
const NUMBER_BOX_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::NullableNumber,
        required: true,
    },
    PropertySpec {
        name: "minimum",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "maximum",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "step",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "large_step",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "fraction_digits",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "wraps",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const NUMBER_BOX_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "change",
    payload_type: UiValueType::NullableNumber,
}];
const COMBO_BOX_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "options",
        value_type: UiValueType::StringArray,
        required: true,
    },
    PropertySpec {
        name: "selected_index",
        value_type: UiValueType::NullableInteger,
        required: false,
    },
    PropertySpec {
        name: "placeholder",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const COMBO_BOX_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "select",
        payload_type: UiValueType::Integer,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
];
const TABS_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "labels",
        value_type: UiValueType::StringMap,
        required: true,
    },
    PropertySpec {
        name: "icons",
        value_type: UiValueType::StringMap,
        required: false,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::String,
        required: false,
    },
];
const TABS_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "select",
    payload_type: UiValueType::String,
}];
const GRID_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "columns",
        value_type: UiValueType::GridTrackArray,
        required: true,
    },
    PropertySpec {
        name: "rows",
        value_type: UiValueType::GridTrackArray,
        required: true,
    },
    PropertySpec {
        name: "placements",
        value_type: UiValueType::GridPlacementMap,
        required: true,
    },
    PropertySpec {
        name: "column_gap",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "row_gap",
        value_type: UiValueType::Number,
        required: false,
    },
];
const SCROLL_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "offset_y",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "content_height",
        value_type: UiValueType::Number,
        required: true,
    },
];
const SCROLL_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "scroll",
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
        "scroll" => Some(ComponentSchema {
            properties: SCROLL_PROPERTIES,
            actions: SCROLL_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "tabs" => Some(ComponentSchema {
            properties: TABS_PROPERTIES,
            actions: TABS_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "grid" => Some(ComponentSchema {
            properties: GRID_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "text" => Some(leaf(TEXT_PROPERTIES, NO_ACTIONS)),
        "button" => Some(leaf(BUTTON_PROPERTIES, BUTTON_ACTIONS)),
        "toggle_button" | "checkbox" | "toggle" => Some(leaf(CHECKED_PROPERTIES, TOGGLE_ACTIONS)),
        "textbox" => Some(leaf(TEXTBOX_PROPERTIES, TEXTBOX_ACTIONS)),
        "radio_button" => Some(leaf(RADIO_PROPERTIES, RADIO_ACTIONS)),
        "slider" => Some(leaf(VALUE_PROPERTIES, SLIDER_ACTIONS)),
        "number_box" => Some(leaf(NUMBER_BOX_PROPERTIES, NUMBER_BOX_ACTIONS)),
        "combo_box" => Some(leaf(COMBO_BOX_PROPERTIES, COMBO_BOX_ACTIONS)),
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

fn validate_grid_component(node: &UiNode, path: &str, diagnostics: &mut Vec<UiDiagnostic>) {
    for name in ["column_gap", "row_gap"] {
        if node
            .properties
            .get(name)
            .and_then(Value::as_f64)
            .is_some_and(|value| value < 0.0)
        {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidPropertyValue,
                format!("{path}.properties.{name}"),
                format!("grid property {name:?} must not be negative"),
            );
        }
    }

    let columns = node
        .properties
        .get("columns")
        .and_then(grid_tracks_from_value);
    let rows = node.properties.get("rows").and_then(grid_tracks_from_value);
    let placements = node
        .properties
        .get("placements")
        .and_then(grid_placements_from_value);
    let child_ids = node
        .children
        .iter()
        .map(|child| child.id.as_str())
        .collect::<BTreeSet<_>>();

    let Some(placements) = placements else {
        return;
    };
    for child_id in &child_ids {
        if !placements.contains_key(*child_id) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidPropertyValue,
                format!("{path}.properties.placements"),
                format!("grid placements must contain child id {child_id:?}"),
            );
        }
    }
    for placement_id in placements.keys() {
        if !child_ids.contains(placement_id.as_str()) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidPropertyValue,
                format!("{path}.properties.placements.{placement_id}"),
                format!("grid placement id {placement_id:?} does not address a child"),
            );
        }
    }

    let (Some(columns), Some(rows)) = (columns, rows) else {
        return;
    };
    for (child_id, placement) in &placements {
        if !child_ids.contains(child_id.as_str()) {
            continue;
        }
        let column_end = placement
            .column
            .checked_add(usize::from(placement.column_span));
        if placement.column >= columns.len()
            || column_end.is_none_or(|column_end| column_end > columns.len())
        {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidPropertyValue,
                format!("{path}.properties.placements.{child_id}.column"),
                format!("grid placement for {child_id:?} exceeds the declared columns"),
            );
        }
        let row_end = placement.row.checked_add(usize::from(placement.row_span));
        if placement.row >= rows.len() || row_end.is_none_or(|row_end| row_end > rows.len()) {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidPropertyValue,
                format!("{path}.properties.placements.{child_id}.row"),
                format!("grid placement for {child_id:?} exceeds the declared rows"),
            );
        }
    }
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

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn fnv1a64_two(first: &[u8], second: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in first.iter().chain(second) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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
        document.root.children[0].component = "password_box".to_owned();
        document.root.children[1].component = "imaginary".to_owned();
        let report = document.validate(
            &UiFeatureSet::new(["button", "label", "password-box"]),
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
    fn scroll_contract_requires_one_child_and_nonnegative_geometry() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "viewport",
                "component": "scroll",
                "properties": { "content_height": 480.0 },
                "property_bindings": { "offset_y": "scroll_offset" },
                "action_bindings": { "scroll": "scroll_changed" },
                "children": [
                  {
                    "id": "content",
                    "component": "text",
                    "properties": { "text": "Content" }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("scroll_offset".to_owned(), UiValueType::Number)]),
            actions: BTreeMap::from([("scroll_changed".to_owned(), UiValueType::Number)]),
        };
        assert!(valid
            .validate(&UiFeatureSet::new(["scroll", "label"]), &bindings)
            .is_valid());
        let handoff = UiAiHandoffPackage::build(
            &valid,
            &UiFeatureSet::new(["scroll", "label"]),
            &bindings,
            None,
            None,
        )
        .unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "scroll")
            .unwrap();
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::Exactly { count: 1 }
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "viewport",
                "component": "scroll",
                "properties": { "offset_y": -1.0, "content_height": -20.0 },
                "children": []
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&UiFeatureSet::new(["scroll"]), &UiBindingSchema::default());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidChildCount));
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| { diagnostic.code == UiDiagnosticCode::InvalidPropertyValue })
                .count(),
            2
        );
    }

    #[test]
    fn number_box_contract_validates_nullable_values_and_numeric_configuration() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "retry-count",
                "component": "number_box",
                "properties": {
                  "minimum": 0.0,
                  "maximum": 10.0,
                  "step": 0.5,
                  "large_step": 5.0,
                  "fraction_digits": 1.0,
                  "wraps": true
                },
                "property_bindings": { "value": "retry_count" },
                "action_bindings": { "change": "retry_count_changed" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("retry_count".to_owned(), UiValueType::NullableNumber)]),
            actions: BTreeMap::from([(
                "retry_count_changed".to_owned(),
                UiValueType::NullableNumber,
            )]),
        };
        assert!(valid
            .validate(&UiFeatureSet::new(["number-box"]), &bindings)
            .is_valid());
        assert!(validate_ui_binding_values(
            &bindings,
            &BTreeMap::from([("retry_count".to_owned(), Value::Null)])
        )
        .is_valid());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "retry-count",
                "component": "number_box",
                "properties": {
                  "value": 8.0,
                  "minimum": 10.0,
                  "maximum": 5.0,
                  "step": 0.0,
                  "large_step": -1.0,
                  "fraction_digits": 12.5
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(
            &UiFeatureSet::new(["number-box"]),
            &UiBindingSchema::default(),
        );
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            4
        );
    }

    #[test]
    fn combo_box_contract_validates_string_options_and_controlled_state() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "profile-mode",
                "component": "combo_box",
                "properties": {
                  "options": ["Balanced", "Fast", "Quiet"],
                  "placeholder": "Choose a mode"
                },
                "property_bindings": {
                  "selected_index": "profile_mode",
                  "expanded": "profile_mode_expanded"
                },
                "action_bindings": {
                  "select": "profile_mode_selected",
                  "expanded_change": "profile_mode_expanded_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                ("profile_mode".to_owned(), UiValueType::NullableInteger),
                ("profile_mode_expanded".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                ("profile_mode_selected".to_owned(), UiValueType::Integer),
                (
                    "profile_mode_expanded_changed".to_owned(),
                    UiValueType::Boolean,
                ),
            ]),
        };
        assert!(valid
            .validate(&UiFeatureSet::new(["combo"]), &bindings)
            .is_valid());
        assert!(validate_ui_binding_values(
            &UiBindingSchema {
                properties: BTreeMap::from([
                    ("options".to_owned(), UiValueType::StringArray),
                    ("selected".to_owned(), UiValueType::NullableInteger),
                ]),
                actions: BTreeMap::new(),
            },
            &BTreeMap::from([
                (
                    "options".to_owned(),
                    Value::Array(vec![Value::String("One".to_owned())]),
                ),
                ("selected".to_owned(), Value::Null),
            ])
        )
        .is_valid());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "profile-mode",
                "component": "combo_box",
                "properties": {
                  "options": ["Only", 2],
                  "selected_index": 4
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&UiFeatureSet::new(["combo"]), &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyType
                && diagnostic.path.ends_with("properties.options")
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path.ends_with("properties.selected_index")
        }));
    }

    #[test]
    fn tabs_contract_uses_child_ids_as_stable_slots_and_selection_values() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "settings-tabs",
                "component": "tabs",
                "properties": {
                  "labels": {
                    "general": "General",
                    "advanced": "Advanced"
                  },
                  "icons": { "general": "Settings" }
                },
                "property_bindings": { "selected": "active_tab" },
                "action_bindings": { "select": "active_tab_selected" },
                "children": [
                  { "id": "general", "component": "stack" },
                  { "id": "advanced", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("active_tab".to_owned(), UiValueType::String)]),
            actions: BTreeMap::from([("active_tab_selected".to_owned(), UiValueType::String)]),
        };
        assert!(valid
            .validate(&UiFeatureSet::new(["tabs"]), &bindings)
            .is_valid());
        let handoff =
            UiAiHandoffPackage::build(&valid, &UiFeatureSet::new(["tabs"]), &bindings, None, None)
                .unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "tabs")
            .unwrap();
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::AtLeast { minimum: 1 }
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "settings-tabs",
                "component": "tabs",
                "properties": {
                  "labels": { "general": "General", "missing": "Missing" },
                  "icons": { "general": "NotAnIcon" },
                  "selected": "missing"
                },
                "children": [
                  { "id": "general", "component": "stack" },
                  { "id": "advanced", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&UiFeatureSet::new(["tabs"]), &UiBindingSchema::default());
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            4
        );
    }

    #[test]
    fn grid_contract_uses_typed_tracks_and_stable_child_placements() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "settings-grid",
                "component": "grid",
                "properties": {
                  "columns": [
                    { "kind": "fixed", "size": 160.0 },
                    { "kind": "fraction", "weight": 2 }
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
                  "column_gap": 12.0,
                  "row_gap": 8.0
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
        assert!(valid
            .validate(&UiFeatureSet::new(["grid"]), &UiBindingSchema::default())
            .is_valid());
        let handoff = UiAiHandoffPackage::build(
            &valid,
            &UiFeatureSet::new(["grid"]),
            &UiBindingSchema::default(),
            None,
            None,
        )
        .unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "grid")
            .unwrap();
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::AtLeast { minimum: 1 }
        );
        assert_eq!(
            contract
                .properties
                .iter()
                .find(|property| property.name == "placements")
                .unwrap()
                .value_type,
            UiValueType::GridPlacementMap
        );

        let typed_bindings = UiBindingSchema {
            properties: BTreeMap::from([
                ("tracks".to_owned(), UiValueType::GridTrackArray),
                ("cells".to_owned(), UiValueType::GridPlacementMap),
            ]),
            actions: BTreeMap::new(),
        };
        assert!(validate_ui_binding_values(
            &typed_bindings,
            &BTreeMap::from([
                (
                    "tracks".to_owned(),
                    serde_json::json!([{ "kind": "fraction", "weight": 1 }]),
                ),
                (
                    "cells".to_owned(),
                    serde_json::json!({ "content": { "row": 0, "column": 0 } }),
                ),
            ])
        )
        .is_valid());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "settings-grid",
                "component": "grid",
                "properties": {
                  "columns": [{ "kind": "fraction", "weight": 1 }],
                  "rows": [{ "kind": "fraction", "weight": 1 }],
                  "placements": {
                    "first": { "row": 0, "column": 1 },
                    "ghost": { "row": 0, "column": 0 }
                  },
                  "column_gap": -1.0
                },
                "children": [
                  { "id": "first", "component": "stack" },
                  { "id": "second", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&UiFeatureSet::new(["grid"]), &UiBindingSchema::default());
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            4
        );
        let invalid_values = validate_ui_binding_values(
            &typed_bindings,
            &BTreeMap::from([
                (
                    "tracks".to_owned(),
                    serde_json::json!([{ "kind": "fraction", "weight": 0 }]),
                ),
                (
                    "cells".to_owned(),
                    serde_json::json!({ "content": { "row": 0, "column": 0, "row_span": 0 } }),
                ),
            ]),
        );
        assert_eq!(invalid_values.diagnostics.len(), 2);
        assert!(invalid_values
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == UiDiagnosticCode::BindingValueTypeMismatch));
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

    #[test]
    fn binding_value_validation_rejects_unknown_and_mismatched_values() {
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                ("enabled".to_owned(), UiValueType::Boolean),
                ("title".to_owned(), UiValueType::String),
            ]),
            actions: BTreeMap::new(),
        };
        let values = BTreeMap::from([
            ("enabled".to_owned(), Value::String("yes".to_owned())),
            ("extra".to_owned(), Value::Null),
        ]);

        let report = validate_ui_binding_values(&bindings, &values);
        assert_eq!(report.diagnostics.len(), 2);
        assert_eq!(
            report.diagnostics[0].code,
            UiDiagnosticCode::BindingValueTypeMismatch
        );
        assert_eq!(
            report.diagnostics[1].code,
            UiDiagnosticCode::UnknownBindingValue
        );
    }

    #[test]
    fn ai_handoff_is_deterministic_and_round_trips_authoring_files() {
        let document = valid_document();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("window_title".to_owned(), UiValueType::String)]),
            actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
        };
        let values =
            BTreeMap::from([("window_title".to_owned(), Value::String("Notes".to_owned()))]);

        let features = UiFeatureSet::new(["button", "label"]);
        let first = UiAiHandoffPackage::build(&document, &features, &bindings, Some(&values), None)
            .unwrap();
        let second =
            UiAiHandoffPackage::build(&document, &features, &bindings, Some(&values), None)
                .unwrap();

        assert_eq!(first, second);
        assert!(first.document_json.ends_with('\n'));
        assert!(first.handoff_json.ends_with('\n'));
        assert_eq!(
            UiDocument::from_json(&first.document_json).unwrap(),
            document
        );
        assert_eq!(
            serde_json::from_str::<UiBindingSchema>(&first.bindings_json).unwrap(),
            bindings
        );
        assert_eq!(
            serde_json::from_str::<BTreeMap<String, Value>>(first.values_json.as_ref().unwrap())
                .unwrap(),
            values
        );
        assert_eq!(
            first.manifest.required_features,
            vec![
                "button".to_owned(),
                "label".to_owned(),
                "ui-document".to_owned()
            ]
        );
        assert_eq!(first.manifest.nodes.len(), 3);
        assert_eq!(first.manifest.nodes[1].path, "$.root.children[0]");
        assert_eq!(first.manifest.component_contracts.len(), 3);
        assert!(first.manifest.missing_values.is_empty());
    }

    #[test]
    fn ai_handoff_records_png_dimensions_and_rejects_invalid_preview() {
        let mut png = vec![0_u8; 33];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[8..12].copy_from_slice(&13_u32.to_be_bytes());
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&960_u32.to_be_bytes());
        png[20..24].copy_from_slice(&640_u32.to_be_bytes());

        let package = UiAiHandoffPackage::build(
            &valid_document(),
            &UiFeatureSet::new(["button", "label"]),
            &UiBindingSchema {
                properties: BTreeMap::from([("window_title".to_owned(), UiValueType::String)]),
                actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
            },
            None,
            Some(&png),
        )
        .unwrap();
        let preview = package.manifest.files.preview.as_ref().unwrap();
        assert_eq!((preview.width, preview.height), (960, 640));
        assert_eq!(preview.file.byte_length, 33);
        assert_eq!(package.preview_png.as_deref(), Some(png.as_slice()));

        assert!(matches!(
            UiAiHandoffPackage::build(
                &valid_document(),
                &UiFeatureSet::new(["button", "label"]),
                &UiBindingSchema {
                    properties: BTreeMap::from([("window_title".to_owned(), UiValueType::String)]),
                    actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
                },
                None,
                Some(b"not a PNG"),
            ),
            Err(UiAiHandoffBuildError::InvalidPreviewPng)
        ));
    }

    #[test]
    fn ai_handoff_rejects_invalid_binding_value_snapshot() {
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                ("title".to_owned(), UiValueType::String),
                ("window_title".to_owned(), UiValueType::String),
            ]),
            actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
        };
        let values = BTreeMap::from([("title".to_owned(), Value::Bool(true))]);

        assert!(matches!(
            UiAiHandoffPackage::build(
                &valid_document(),
                &UiFeatureSet::new(["button", "label"]),
                &bindings,
                Some(&values),
                None
            ),
            Err(UiAiHandoffBuildError::InvalidValues(_))
        ));
    }

    #[test]
    fn release_artifact_is_deterministic_and_validates_on_decode() {
        let document = valid_document();
        let features = UiFeatureSet::new(["button", "label"]);
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("window_title".to_owned(), UiValueType::String)]),
            actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
        };

        let first = UiDocumentReleaseArtifact::compile(&document, &features, &bindings).unwrap();
        let second = UiDocumentReleaseArtifact::compile(&document, &features, &bindings).unwrap();
        assert_eq!(first, second);
        assert_eq!(&first.as_bytes()[..8], UI_DOCUMENT_ARTIFACT_MAGIC);

        let embedded = UiEmbeddedDocument::decode(first.as_bytes(), &features, &bindings).unwrap();
        assert_eq!(embedded.document, document);
        assert_eq!(embedded.bindings, bindings);
        assert!(first.content_fingerprint().starts_with("fnv1a64:"));
    }

    #[test]
    fn release_artifact_rejects_tampering_and_binding_drift() {
        let document = valid_document();
        let features = UiFeatureSet::new(["button", "label"]);
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("window_title".to_owned(), UiValueType::String)]),
            actions: BTreeMap::from([("save".to_owned(), UiValueType::Null)]),
        };
        let artifact = UiDocumentReleaseArtifact::compile(&document, &features, &bindings).unwrap();

        let mut tampered = artifact.as_bytes().to_vec();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        assert!(matches!(
            UiEmbeddedDocument::decode(&tampered, &features, &bindings),
            Err(UiDocumentArtifactError::FingerprintMismatch)
        ));

        assert!(matches!(
            UiEmbeddedDocument::decode(artifact.as_bytes(), &features, &UiBindingSchema::default()),
            Err(UiDocumentArtifactError::BindingSchemaMismatch)
        ));
    }
}
