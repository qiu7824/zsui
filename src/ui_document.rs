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

use crate::Dp;

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

#[cfg(all(feature = "ui-document-runtime", feature = "auto-suggest"))]
pub(crate) fn ui_auto_suggestion_runtime_id(
    node_id: &UiNodeId,
    suggestion_id: &UiAutoSuggestionId,
) -> crate::ZsAutoSuggestionId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(suggestion_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsAutoSuggestionId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "command-palette"))]
pub(crate) fn ui_command_palette_runtime_id(
    node_id: &UiNodeId,
    item_id: &UiCommandPaletteItemId,
) -> crate::ZsCommandPaletteItemId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(item_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsCommandPaletteItemId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "tree"))]
pub(crate) fn ui_tree_runtime_id(
    node_id: &UiNodeId,
    tree_node_id: &UiTreeNodeId,
) -> crate::ZsTreeNodeId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(tree_node_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsTreeNodeId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "grid-view"))]
pub(crate) fn ui_grid_view_runtime_id(
    node_id: &UiNodeId,
    item_id: &UiGridViewItemId,
) -> crate::ZsGridViewItemId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(item_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsGridViewItemId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "shell"))]
pub(crate) fn ui_navigation_runtime_id(
    node_id: &UiNodeId,
    item_id: &UiNavigationItemId,
) -> crate::WidgetId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(item_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::WidgetId::synthetic_child(node_id.widget_id(), hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "table"))]
pub(crate) fn ui_table_column_runtime_id(
    node_id: &UiNodeId,
    column_id: &UiTableColumnId,
) -> crate::ZsTableColumnId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(column_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsTableColumnId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "table"))]
pub(crate) fn ui_table_row_runtime_id(
    node_id: &UiNodeId,
    row_id: &UiTableRowId,
) -> crate::ZsTableRowId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(row_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsTableRowId::new(hash)
}

#[cfg(all(feature = "ui-document-runtime", feature = "breadcrumb"))]
pub(crate) fn ui_breadcrumb_runtime_id(
    node_id: &UiNodeId,
    item_id: &UiBreadcrumbItemId,
) -> crate::ZsBreadcrumbId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in node_id
        .as_str()
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .chain(item_id.as_str().as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::ZsBreadcrumbId::new(hash)
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
    pub padding_token: Option<UiSpacingToken>,
    pub gap: Option<f32>,
    pub gap_token: Option<UiSpacingToken>,
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

/// Platform-neutral spacing references resolved by the active desktop
/// experience profile. Numeric layout values remain available for deliberate
/// application-specific geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSpacingToken {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    ContentGap,
    ContentPadding,
    PagePadding,
}

impl UiSpacingToken {
    #[cfg(feature = "ui-document-runtime")]
    pub(crate) fn resolve(self) -> Dp {
        let spacing = crate::ZsuiSpacingTokens::default();
        match self {
            Self::Xs => spacing.xs,
            Self::Sm => spacing.sm,
            Self::Md => spacing.md,
            Self::Lg => spacing.lg,
            Self::Xl => spacing.xl,
            Self::ContentGap => spacing.content_gap,
            Self::ContentPadding => spacing.content_padding,
            Self::PagePadding => spacing.page_padding,
        }
    }
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

/// Stable author-facing identity for one document-backed breadcrumb item.
///
/// The identity is independent from declaration order. The runtime derives a
/// private [`crate::ZsBreadcrumbId`] from this value and the owning node ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiBreadcrumbItemId(String);

impl UiBreadcrumbItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiBreadcrumbItemIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiBreadcrumbItemIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiBreadcrumbItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiBreadcrumbItemIdError {
    value: String,
}

impl fmt::Display for UiBreadcrumbItemIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "breadcrumb item id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiBreadcrumbItemIdError {}

/// Application-owned display metadata for one document-backed breadcrumb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiBreadcrumbItem {
    id: UiBreadcrumbItemId,
    label: String,
}

impl UiBreadcrumbItem {
    pub fn new(id: UiBreadcrumbItemId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    pub fn id(&self) -> &UiBreadcrumbItemId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Stable author-facing identity for one document-backed MenuFlyout item.
///
/// Commands and submenus share one identity namespace across the complete
/// nested menu. Separators deliberately have no identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiMenuFlyoutItemId(String);

impl UiMenuFlyoutItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiMenuFlyoutItemIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiMenuFlyoutItemIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiMenuFlyoutItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMenuFlyoutItemIdError {
    value: String,
}

impl fmt::Display for UiMenuFlyoutItemIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "menu-flyout item id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiMenuFlyoutItemIdError {}

/// Platform-neutral accelerator declared by a document-backed MenuFlyout.
///
/// `primary` maps to Control on Windows/Linux and Command on macOS. Keys use a
/// canonical portable spelling: one uppercase ASCII alphanumeric character,
/// `enter`, `escape`, `tab`, `space`, `backspace`, `delete`, `up`, `down`,
/// `left`, `right`, `home`, `end`, `page_up`, `page_down`, or `f1` through
/// `f24`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiMenuFlyoutAccelerator {
    key: String,
    #[serde(default, skip_serializing_if = "is_false")]
    primary: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    shift: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    alt: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    super_key: bool,
}

impl UiMenuFlyoutAccelerator {
    pub fn new(value: impl Into<String>) -> Result<Self, UiMenuFlyoutAcceleratorError> {
        let value = value.into();
        let Some(key) = canonical_menu_flyout_accelerator_key(&value) else {
            return Err(UiMenuFlyoutAcceleratorError { value });
        };
        Ok(Self {
            key,
            primary: false,
            shift: false,
            alt: false,
            super_key: false,
        })
    }

    pub const fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    pub const fn shifted(mut self) -> Self {
        self.shift = true;
        self
    }

    pub const fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub const fn with_super(mut self) -> Self {
        self.super_key = true;
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub const fn uses_primary(&self) -> bool {
        self.primary
    }

    pub const fn uses_shift(&self) -> bool {
        self.shift
    }

    pub const fn uses_alt(&self) -> bool {
        self.alt
    }

    pub const fn uses_super(&self) -> bool {
        self.super_key
    }

    pub(crate) fn native_accelerator(&self) -> Option<crate::ZsAccelerator> {
        let canonical = canonical_menu_flyout_accelerator_key(&self.key)?;
        if canonical != self.key {
            return None;
        }
        let key = native_menu_flyout_accelerator_key(&canonical)?;
        let mut accelerator = crate::ZsAccelerator::new(key);
        if self.primary {
            accelerator = crate::ZsAccelerator::primary(key);
        }
        if self.shift {
            accelerator = accelerator.shifted();
        }
        if self.alt {
            accelerator = accelerator.with_alt();
        }
        if self.super_key {
            accelerator = accelerator.with_super();
        }
        accelerator.validate().ok().map(|()| accelerator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMenuFlyoutAcceleratorError {
    value: String,
}

impl fmt::Display for UiMenuFlyoutAcceleratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "menu-flyout accelerator key {:?} is not a canonical portable key",
            self.value
        )
    }
}

impl Error for UiMenuFlyoutAcceleratorError {}

/// One command, separator or submenu in a document-backed MenuFlyout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UiMenuFlyoutItem {
    Command {
        id: UiMenuFlyoutItemId,
        label: String,
        #[serde(default = "ui_menu_flyout_item_enabled_default")]
        enabled: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        checked: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        accelerator: Option<UiMenuFlyoutAccelerator>,
    },
    Separator,
    Submenu {
        id: UiMenuFlyoutItemId,
        label: String,
        #[serde(default = "ui_menu_flyout_item_enabled_default")]
        enabled: bool,
        items: Vec<Self>,
    },
}

impl UiMenuFlyoutItem {
    pub fn command(id: UiMenuFlyoutItemId, label: impl Into<String>) -> Self {
        Self::Command {
            id,
            label: label.into(),
            enabled: true,
            checked: false,
            accelerator: None,
        }
    }

    pub const fn separator() -> Self {
        Self::Separator
    }

    pub fn submenu(
        id: UiMenuFlyoutItemId,
        label: impl Into<String>,
        items: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self::Submenu {
            id,
            label: label.into(),
            enabled: true,
            items: items.into_iter().collect(),
        }
    }

    pub const fn enabled(mut self, value: bool) -> Self {
        match &mut self {
            Self::Command { enabled, .. } | Self::Submenu { enabled, .. } => *enabled = value,
            Self::Separator => {}
        }
        self
    }

    pub const fn checked(mut self, value: bool) -> Self {
        if let Self::Command { checked, .. } = &mut self {
            *checked = value;
        }
        self
    }

    pub fn accelerator(mut self, value: UiMenuFlyoutAccelerator) -> Self {
        if let Self::Command { accelerator, .. } = &mut self {
            *accelerator = Some(value);
        }
        self
    }

    pub fn id(&self) -> Option<&UiMenuFlyoutItemId> {
        match self {
            Self::Command { id, .. } | Self::Submenu { id, .. } => Some(id),
            Self::Separator => None,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Command { label, .. } | Self::Submenu { label, .. } => Some(label),
            Self::Separator => None,
        }
    }

    pub fn child_items(&self) -> &[Self] {
        match self {
            Self::Submenu { items, .. } => items,
            _ => &[],
        }
    }
}

const fn ui_menu_flyout_item_enabled_default() -> bool {
    true
}

/// Stable author-facing identity for one document-backed auto-suggest item.
///
/// The identity is independent from declaration order. The runtime derives a
/// private [`crate::ZsAutoSuggestionId`] from this value and the owning node ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiAutoSuggestionId(String);

impl UiAutoSuggestionId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiAutoSuggestionIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiAutoSuggestionIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiAutoSuggestionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiAutoSuggestionIdError {
    value: String,
}

impl fmt::Display for UiAutoSuggestionIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "auto-suggestion id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiAutoSuggestionIdError {}

/// One stable suggestion supplied by document state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAutoSuggestion {
    id: UiAutoSuggestionId,
    text: String,
}

impl UiAutoSuggestion {
    pub fn new(id: UiAutoSuggestionId, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
        }
    }

    pub fn id(&self) -> &UiAutoSuggestionId {
        &self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn into_parts(self) -> (UiAutoSuggestionId, String) {
        (self.id, self.text)
    }
}

/// Typed query-submission payload emitted by a document-backed AutoSuggestBox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAutoSuggestSubmission {
    query: String,
    chosen: Option<UiAutoSuggestionId>,
}

impl UiAutoSuggestSubmission {
    pub fn new(query: impl Into<String>, chosen: Option<UiAutoSuggestionId>) -> Self {
        Self {
            query: query.into(),
            chosen,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn chosen(&self) -> Option<&UiAutoSuggestionId> {
        self.chosen.as_ref()
    }

    pub fn into_parts(self) -> (String, Option<UiAutoSuggestionId>) {
        (self.query, self.chosen)
    }
}

/// Stable author-facing identity for one document-backed command.
///
/// The identity is independent from declaration order. The release runtime
/// derives a private [`crate::ZsCommandPaletteItemId`] from this value and the
/// owning node ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiCommandPaletteItemId(String);

impl UiCommandPaletteItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiCommandPaletteItemIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiCommandPaletteItemIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiCommandPaletteItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCommandPaletteItemIdError {
    value: String,
}

impl fmt::Display for UiCommandPaletteItemIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "command-palette item id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiCommandPaletteItemIdError {}

/// Application-owned display metadata for one document-backed command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiCommandPaletteItem {
    id: UiCommandPaletteItemId,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shortcut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<crate::ZsIcon>,
    #[serde(default = "ui_command_palette_item_enabled_default")]
    enabled: bool,
}

impl UiCommandPaletteItem {
    pub fn new(id: UiCommandPaletteItemId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            keywords: Vec::new(),
            shortcut: None,
            icon: None,
            enabled: true,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn keywords<T>(mut self, keywords: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn icon(mut self, icon: crate::ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &UiCommandPaletteItemId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn keyword_values(&self) -> &[String] {
        &self.keywords
    }

    pub fn shortcut_text(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }

    pub const fn semantic_icon(&self) -> Option<crate::ZsIcon> {
        self.icon
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn matches_query(&self, query: &str) -> bool {
        let terms = query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return true;
        }
        let mut searchable = self.title.to_lowercase();
        if let Some(subtitle) = &self.subtitle {
            searchable.push(' ');
            searchable.push_str(&subtitle.to_lowercase());
        }
        for keyword in &self.keywords {
            searchable.push(' ');
            searchable.push_str(&keyword.to_lowercase());
        }
        terms.iter().all(|term| searchable.contains(term))
    }
}

const fn ui_command_palette_item_enabled_default() -> bool {
    true
}

/// Stable author-facing identity for one node in a document-backed TreeView.
///
/// The identity is independent from hierarchy position and declaration order.
/// The release runtime derives a private [`crate::ZsTreeNodeId`] from this
/// value and the owning UiDocument node ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTreeNodeId(String);

impl UiTreeNodeId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiTreeNodeIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiTreeNodeIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiTreeNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTreeNodeIdError {
    value: String,
}

impl fmt::Display for UiTreeNodeIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "tree node id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiTreeNodeIdError {}

/// Application-owned hierarchy metadata for one document-backed TreeView node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiTreeNode {
    id: UiTreeNodeId,
    label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<crate::ZsIcon>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<Self>,
    #[serde(default, skip_serializing_if = "is_false")]
    has_unrealized_children: bool,
}

impl UiTreeNode {
    pub fn new(id: UiTreeNodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            children: Vec::new(),
            has_unrealized_children: false,
        }
    }

    pub fn icon(mut self, icon: crate::ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub const fn unrealized_children(mut self, has_unrealized_children: bool) -> Self {
        self.has_unrealized_children = has_unrealized_children;
        self
    }

    pub fn id(&self) -> &UiTreeNodeId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn semantic_icon(&self) -> Option<crate::ZsIcon> {
        self.icon
    }

    pub fn child_nodes(&self) -> &[Self] {
        &self.children
    }

    pub const fn has_unrealized_children(&self) -> bool {
        self.has_unrealized_children
    }

    pub fn is_expandable(&self) -> bool {
        self.has_unrealized_children || !self.children.is_empty()
    }
}

/// Stable author-facing identity for one document-backed NavigationView item.
///
/// The identity is independent from declaration order and from the target
/// platform's pane composition. The release runtime derives a private
/// [`crate::WidgetId`] in the reserved document namespace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiNavigationItemId(String);

impl UiNavigationItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiNavigationItemIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiNavigationItemIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiNavigationItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiNavigationItemIdError {
    value: String,
}

impl fmt::Display for UiNavigationItemIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "navigation item id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiNavigationItemIdError {}

/// Application-owned semantic metadata for one NavigationView row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiNavigationItem {
    id: UiNavigationItemId,
    label: String,
    icon: crate::ZsIcon,
    #[serde(default = "ui_navigation_item_enabled_default")]
    enabled: bool,
}

impl UiNavigationItem {
    pub fn new(id: UiNavigationItemId, label: impl Into<String>, icon: crate::ZsIcon) -> Self {
        Self {
            id,
            label: label.into(),
            icon,
            enabled: true,
        }
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &UiNavigationItemId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn semantic_icon(&self) -> crate::ZsIcon {
        self.icon
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

const fn ui_navigation_item_enabled_default() -> bool {
    true
}

/// Stable author-facing identity for one document-backed GridView item.
///
/// The identity is independent from declaration order. The release runtime
/// derives a private [`crate::ZsGridViewItemId`] from this value and the owning
/// UiDocument node ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiGridViewItemId(String);

impl UiGridViewItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiGridViewItemIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiGridViewItemIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiGridViewItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiGridViewItemIdError {
    value: String,
}

impl fmt::Display for UiGridViewItemIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "grid-view item id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiGridViewItemIdError {}

/// Application-owned display metadata for one document-backed GridView tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiGridViewItem {
    id: UiGridViewItemId,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<crate::ZsIcon>,
}

impl UiGridViewItem {
    pub fn new(id: UiGridViewItemId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            icon: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn icon(mut self, icon: crate::ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn id(&self) -> &UiGridViewItemId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub const fn semantic_icon(&self) -> Option<crate::ZsIcon> {
        self.icon
    }
}

/// Stable author-facing identity for one document-backed DataGrid column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTableColumnId(String);

impl UiTableColumnId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiTableColumnIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiTableColumnIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiTableColumnId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableColumnIdError {
    value: String,
}

impl fmt::Display for UiTableColumnIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "table column id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiTableColumnIdError {}

/// Stable author-facing identity for one document-backed DataGrid row.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTableRowId(String);

impl UiTableRowId {
    pub fn new(value: impl Into<String>) -> Result<Self, UiTableRowIdError> {
        let value = value.into();
        if is_valid_node_id(&value) {
            Ok(Self(value))
        } else {
            Err(UiTableRowIdError { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UiTableRowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableRowIdError {
    value: String,
}

impl fmt::Display for UiTableRowIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "table row id {:?} must be non-empty and contain only letters, numbers, '_', '-' or '.'",
            self.value
        )
    }
}

impl Error for UiTableRowIdError {}

/// Platform-neutral DataGrid column sizing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UiTableColumnWidth {
    Fixed { width: Dp },
    Fill { weight: u16 },
}

impl Default for UiTableColumnWidth {
    fn default() -> Self {
        Self::Fill { weight: 1 }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTableColumnAlignment {
    #[default]
    Start,
    Center,
    End,
}

/// Application-owned metadata for one document-backed DataGrid column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiTableColumn {
    id: UiTableColumnId,
    header: String,
    #[serde(default)]
    width: UiTableColumnWidth,
    #[serde(default)]
    alignment: UiTableColumnAlignment,
    #[serde(default, skip_serializing_if = "is_false")]
    sortable: bool,
}

impl UiTableColumn {
    pub fn new(id: UiTableColumnId, header: impl Into<String>) -> Self {
        Self {
            id,
            header: header.into(),
            width: UiTableColumnWidth::default(),
            alignment: UiTableColumnAlignment::default(),
            sortable: false,
        }
    }

    pub fn fixed_width(mut self, width: Dp) -> Self {
        self.width = UiTableColumnWidth::Fixed { width };
        self
    }

    pub fn fill_width(mut self, weight: u16) -> Self {
        self.width = UiTableColumnWidth::Fill { weight };
        self
    }

    pub const fn alignment(mut self, alignment: UiTableColumnAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub const fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn id(&self) -> &UiTableColumnId {
        &self.id
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub const fn column_width(&self) -> UiTableColumnWidth {
        self.width
    }

    pub const fn column_alignment(&self) -> UiTableColumnAlignment {
        self.alignment
    }

    pub const fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// Application-owned display values for one document-backed DataGrid row.
///
/// Cells are keyed by stable column ID so reordering columns does not move a
/// value into a different semantic field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiTableRow {
    id: UiTableRowId,
    cells: BTreeMap<UiTableColumnId, String>,
}

impl UiTableRow {
    pub fn new(
        id: UiTableRowId,
        cells: impl IntoIterator<Item = (UiTableColumnId, String)>,
    ) -> Self {
        Self {
            id,
            cells: cells.into_iter().collect(),
        }
    }

    pub fn id(&self) -> &UiTableRowId {
        &self.id
    }

    pub fn cells(&self) -> &BTreeMap<UiTableColumnId, String> {
        &self.cells
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTableSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiTableSort {
    column: UiTableColumnId,
    direction: UiTableSortDirection,
}

impl UiTableSort {
    pub const fn new(column: UiTableColumnId, direction: UiTableSortDirection) -> Self {
        Self { column, direction }
    }

    pub fn column(&self) -> &UiTableColumnId {
        &self.column
    }

    pub const fn direction(&self) -> UiTableSortDirection {
        self.direction
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
    Date,
    Time,
    Color,
    FlyoutDismissReason,
    MenuFlyoutItemId,
    MenuFlyoutItemArray,
    BreadcrumbItemId,
    BreadcrumbItemArray,
    AutoSuggestionId,
    NullableAutoSuggestionId,
    AutoSuggestionArray,
    AutoSuggestSubmission,
    CommandPaletteItemId,
    NullableCommandPaletteItemId,
    CommandPaletteItemArray,
    TreeNodeId,
    NullableTreeNodeId,
    TreeNodeIdArray,
    TreeNodeArray,
    NavigationItemId,
    NullableNavigationItemId,
    NavigationItemArray,
    GridViewItemId,
    NullableGridViewItemId,
    GridViewItemArray,
    TableColumnArray,
    TableRowId,
    NullableTableRowId,
    TableRowArray,
    TableSort,
    NullableTableSort,
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
            Self::Date => ui_date_from_value(value).is_some(),
            Self::Time => ui_time_from_value(value).is_some(),
            Self::Color => value
                .as_str()
                .and_then(crate::Color::parse_hex_rgba)
                .is_some(),
            Self::FlyoutDismissReason => value
                .as_str()
                .is_some_and(|value| matches!(value, "light_dismiss" | "escape")),
            Self::MenuFlyoutItemId => ui_menu_flyout_item_id_from_value(value).is_some(),
            Self::MenuFlyoutItemArray => ui_menu_flyout_items_from_value(value).is_some(),
            Self::BreadcrumbItemId => ui_breadcrumb_item_id_from_value(value).is_some(),
            Self::BreadcrumbItemArray => ui_breadcrumb_items_from_value(value).is_some(),
            Self::AutoSuggestionId => ui_auto_suggestion_id_from_value(value).is_some(),
            Self::NullableAutoSuggestionId => {
                value.is_null() || ui_auto_suggestion_id_from_value(value).is_some()
            }
            Self::AutoSuggestionArray => ui_auto_suggestions_from_value(value).is_some(),
            Self::AutoSuggestSubmission => ui_auto_suggest_submission_from_value(value).is_some(),
            Self::CommandPaletteItemId => ui_command_palette_item_id_from_value(value).is_some(),
            Self::NullableCommandPaletteItemId => {
                value.is_null() || ui_command_palette_item_id_from_value(value).is_some()
            }
            Self::CommandPaletteItemArray => ui_command_palette_items_from_value(value).is_some(),
            Self::TreeNodeId => ui_tree_node_id_from_value(value).is_some(),
            Self::NullableTreeNodeId => {
                value.is_null() || ui_tree_node_id_from_value(value).is_some()
            }
            Self::TreeNodeIdArray => ui_tree_node_ids_from_value(value).is_some(),
            Self::TreeNodeArray => ui_tree_nodes_from_value(value).is_some(),
            Self::NavigationItemId => ui_navigation_item_id_from_value(value).is_some(),
            Self::NullableNavigationItemId => {
                value.is_null() || ui_navigation_item_id_from_value(value).is_some()
            }
            Self::NavigationItemArray => ui_navigation_items_from_value(value).is_some(),
            Self::GridViewItemId => ui_grid_view_item_id_from_value(value).is_some(),
            Self::NullableGridViewItemId => {
                value.is_null() || ui_grid_view_item_id_from_value(value).is_some()
            }
            Self::GridViewItemArray => ui_grid_view_items_from_value(value).is_some(),
            Self::TableColumnArray => ui_table_columns_from_value(value).is_some(),
            Self::TableRowId => ui_table_row_id_from_value(value).is_some(),
            Self::NullableTableRowId => {
                value.is_null() || ui_table_row_id_from_value(value).is_some()
            }
            Self::TableRowArray => ui_table_rows_from_value(value).is_some(),
            Self::TableSort => ui_table_sort_from_value(value).is_some(),
            Self::NullableTableSort => value.is_null() || ui_table_sort_from_value(value).is_some(),
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

fn canonical_menu_flyout_accelerator_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() == 1 {
        let key = value.as_bytes()[0];
        return key
            .is_ascii_alphanumeric()
            .then(|| char::from(key).to_ascii_uppercase().to_string());
    }

    let value = value.to_ascii_lowercase();
    if matches!(
        value.as_str(),
        "enter"
            | "escape"
            | "tab"
            | "space"
            | "backspace"
            | "delete"
            | "up"
            | "down"
            | "left"
            | "right"
            | "home"
            | "end"
            | "page_up"
            | "page_down"
    ) {
        return Some(value);
    }
    value
        .strip_prefix('f')
        .and_then(|number| number.parse::<u8>().ok())
        .filter(|number| (1..=24).contains(number))
        .map(|number| format!("f{number}"))
}

fn native_menu_flyout_accelerator_key(value: &str) -> Option<crate::ZsAcceleratorKey> {
    Some(match value {
        "enter" => crate::ZsAcceleratorKey::Enter,
        "escape" => crate::ZsAcceleratorKey::Escape,
        "tab" => crate::ZsAcceleratorKey::Tab,
        "space" => crate::ZsAcceleratorKey::Space,
        "backspace" => crate::ZsAcceleratorKey::Backspace,
        "delete" => crate::ZsAcceleratorKey::Delete,
        "up" => crate::ZsAcceleratorKey::Up,
        "down" => crate::ZsAcceleratorKey::Down,
        "left" => crate::ZsAcceleratorKey::Left,
        "right" => crate::ZsAcceleratorKey::Right,
        "home" => crate::ZsAcceleratorKey::Home,
        "end" => crate::ZsAcceleratorKey::End,
        "page_up" => crate::ZsAcceleratorKey::PageUp,
        "page_down" => crate::ZsAcceleratorKey::PageDown,
        value if value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric() => {
            crate::ZsAcceleratorKey::Character(char::from(value.as_bytes()[0]))
        }
        value => crate::ZsAcceleratorKey::Function(
            value
                .strip_prefix('f')?
                .parse::<u8>()
                .ok()
                .filter(|number| (1..=24).contains(number))?,
        ),
    })
}

fn ui_menu_flyout_item_id_from_value(value: &Value) -> Option<UiMenuFlyoutItemId> {
    value
        .as_str()
        .and_then(|value| UiMenuFlyoutItemId::new(value).ok())
}

pub(crate) fn ui_menu_flyout_items_from_value(value: &Value) -> Option<Vec<UiMenuFlyoutItem>> {
    fn valid(
        items: &[UiMenuFlyoutItem],
        depth: usize,
        ids: &mut BTreeSet<UiMenuFlyoutItemId>,
    ) -> bool {
        if items.is_empty()
            || matches!(items.first(), Some(UiMenuFlyoutItem::Separator))
            || matches!(items.last(), Some(UiMenuFlyoutItem::Separator))
        {
            return false;
        }
        let mut previous_separator = false;
        items.iter().all(|item| {
            let separator = matches!(item, UiMenuFlyoutItem::Separator);
            if separator {
                let valid = !previous_separator;
                previous_separator = true;
                return valid;
            }
            previous_separator = false;
            match item {
                UiMenuFlyoutItem::Command {
                    id,
                    label,
                    accelerator,
                    ..
                } => {
                    is_valid_node_id(id.as_str())
                        && ids.insert(id.clone())
                        && !label.trim().is_empty()
                        && accelerator
                            .as_ref()
                            .is_none_or(|accelerator| accelerator.native_accelerator().is_some())
                }
                UiMenuFlyoutItem::Submenu {
                    id, label, items, ..
                } => {
                    depth < 8
                        && is_valid_node_id(id.as_str())
                        && ids.insert(id.clone())
                        && !label.trim().is_empty()
                        && valid(items, depth.saturating_add(1), ids)
                }
                UiMenuFlyoutItem::Separator => unreachable!(),
            }
        })
    }

    let items = serde_json::from_value::<Vec<UiMenuFlyoutItem>>(value.clone()).ok()?;
    valid(&items, 1, &mut BTreeSet::new()).then_some(items)
}

fn ui_breadcrumb_item_id_from_value(value: &Value) -> Option<UiBreadcrumbItemId> {
    value
        .as_str()
        .and_then(|value| UiBreadcrumbItemId::new(value).ok())
}

pub(crate) fn ui_breadcrumb_items_from_value(value: &Value) -> Option<Vec<UiBreadcrumbItem>> {
    let items = serde_json::from_value::<Vec<UiBreadcrumbItem>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    (!items.is_empty()
        && items.iter().all(|item| {
            is_valid_node_id(item.id.as_str())
                && ids.insert(item.id.as_str().to_owned())
                && !item.label.trim().is_empty()
        }))
    .then_some(items)
}

fn ui_auto_suggestion_id_from_value(value: &Value) -> Option<UiAutoSuggestionId> {
    value
        .as_str()
        .and_then(|value| UiAutoSuggestionId::new(value).ok())
}

pub(crate) fn ui_auto_suggestions_from_value(value: &Value) -> Option<Vec<UiAutoSuggestion>> {
    let suggestions = serde_json::from_value::<Vec<UiAutoSuggestion>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    suggestions
        .iter()
        .all(|suggestion| {
            is_valid_node_id(suggestion.id.as_str())
                && ids.insert(suggestion.id.as_str().to_owned())
        })
        .then_some(suggestions)
}

fn ui_auto_suggest_submission_from_value(value: &Value) -> Option<UiAutoSuggestSubmission> {
    let object = value.as_object()?;
    if !object.contains_key("query") || !object.contains_key("chosen") {
        return None;
    }
    let submission = serde_json::from_value::<UiAutoSuggestSubmission>(value.clone()).ok()?;
    submission
        .chosen
        .as_ref()
        .is_none_or(|chosen| is_valid_node_id(chosen.as_str()))
        .then_some(submission)
}

fn ui_command_palette_item_id_from_value(value: &Value) -> Option<UiCommandPaletteItemId> {
    value
        .as_str()
        .and_then(|value| UiCommandPaletteItemId::new(value).ok())
}

pub(crate) fn ui_command_palette_items_from_value(
    value: &Value,
) -> Option<Vec<UiCommandPaletteItem>> {
    let items = serde_json::from_value::<Vec<UiCommandPaletteItem>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    items
        .iter()
        .all(|item| {
            is_valid_node_id(item.id.as_str())
                && ids.insert(item.id.as_str().to_owned())
                && !item.title.trim().is_empty()
                && item
                    .subtitle
                    .as_ref()
                    .is_none_or(|subtitle| !subtitle.trim().is_empty())
                && item
                    .shortcut
                    .as_ref()
                    .is_none_or(|shortcut| !shortcut.trim().is_empty())
                && item
                    .keywords
                    .iter()
                    .all(|keyword| !keyword.trim().is_empty())
        })
        .then_some(items)
}

fn ui_tree_node_id_from_value(value: &Value) -> Option<UiTreeNodeId> {
    value
        .as_str()
        .and_then(|value| UiTreeNodeId::new(value).ok())
}

pub(crate) fn ui_tree_node_ids_from_value(value: &Value) -> Option<BTreeSet<UiTreeNodeId>> {
    let ids = serde_json::from_value::<Vec<UiTreeNodeId>>(value.clone()).ok()?;
    let unique = ids.iter().cloned().collect::<BTreeSet<_>>();
    (unique.len() == ids.len() && ids.iter().all(|id| is_valid_node_id(id.as_str())))
        .then_some(unique)
}

pub(crate) fn ui_tree_nodes_from_value(value: &Value) -> Option<Vec<UiTreeNode>> {
    fn valid(nodes: &[UiTreeNode], ids: &mut BTreeSet<UiTreeNodeId>) -> bool {
        nodes.iter().all(|node| {
            is_valid_node_id(node.id.as_str())
                && !node.label.trim().is_empty()
                && ids.insert(node.id.clone())
                && valid(&node.children, ids)
        })
    }

    let nodes = serde_json::from_value::<Vec<UiTreeNode>>(value.clone()).ok()?;
    valid(&nodes, &mut BTreeSet::new()).then_some(nodes)
}

fn find_ui_tree_node<'a>(nodes: &'a [UiTreeNode], id: &UiTreeNodeId) -> Option<&'a UiTreeNode> {
    for node in nodes {
        if node.id() == id {
            return Some(node);
        }
        if let Some(found) = find_ui_tree_node(node.child_nodes(), id) {
            return Some(found);
        }
    }
    None
}

fn ui_navigation_item_id_from_value(value: &Value) -> Option<UiNavigationItemId> {
    value
        .as_str()
        .and_then(|value| UiNavigationItemId::new(value).ok())
}

pub(crate) fn ui_navigation_items_from_value(value: &Value) -> Option<Vec<UiNavigationItem>> {
    let items = serde_json::from_value::<Vec<UiNavigationItem>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    (items.iter().all(|item| {
        is_valid_node_id(item.id.as_str())
            && ids.insert(item.id.as_str().to_owned())
            && !item.label.trim().is_empty()
    }))
    .then_some(items)
}

fn ui_grid_view_item_id_from_value(value: &Value) -> Option<UiGridViewItemId> {
    value
        .as_str()
        .and_then(|value| UiGridViewItemId::new(value).ok())
}

pub(crate) fn ui_grid_view_items_from_value(value: &Value) -> Option<Vec<UiGridViewItem>> {
    let items = serde_json::from_value::<Vec<UiGridViewItem>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    items
        .iter()
        .all(|item| {
            is_valid_node_id(item.id.as_str())
                && ids.insert(item.id.as_str().to_owned())
                && !item.title.trim().is_empty()
                && item
                    .subtitle
                    .as_ref()
                    .is_none_or(|subtitle| !subtitle.trim().is_empty())
        })
        .then_some(items)
}

fn ui_table_row_id_from_value(value: &Value) -> Option<UiTableRowId> {
    value
        .as_str()
        .and_then(|value| UiTableRowId::new(value).ok())
}

pub(crate) fn ui_table_columns_from_value(value: &Value) -> Option<Vec<UiTableColumn>> {
    let columns = serde_json::from_value::<Vec<UiTableColumn>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    (!columns.is_empty()
        && columns.iter().all(|column| {
            is_valid_node_id(column.id.as_str())
                && ids.insert(column.id.clone())
                && !column.header.trim().is_empty()
                && match column.width {
                    UiTableColumnWidth::Fixed { width } => width.0.is_finite() && width.0 > 0.0,
                    UiTableColumnWidth::Fill { weight } => weight > 0,
                }
        }))
    .then_some(columns)
}

pub(crate) fn ui_table_rows_from_value(value: &Value) -> Option<Vec<UiTableRow>> {
    let rows = serde_json::from_value::<Vec<UiTableRow>>(value.clone()).ok()?;
    let mut ids = BTreeSet::new();
    rows.iter()
        .all(|row| {
            is_valid_node_id(row.id.as_str())
                && ids.insert(row.id.clone())
                && row
                    .cells
                    .keys()
                    .all(|column_id| is_valid_node_id(column_id.as_str()))
        })
        .then_some(rows)
}

fn ui_table_sort_from_value(value: &Value) -> Option<UiTableSort> {
    let sort = serde_json::from_value::<UiTableSort>(value.clone()).ok()?;
    is_valid_node_id(sort.column.as_str()).then_some(sort)
}

pub(crate) fn ui_table_data_is_compatible(columns: &[UiTableColumn], rows: &[UiTableRow]) -> bool {
    let column_ids = columns
        .iter()
        .map(|column| column.id.clone())
        .collect::<BTreeSet<_>>();
    rows.iter()
        .all(|row| row.cells.keys().cloned().collect::<BTreeSet<_>>() == column_ids)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UiDocumentDate {
    year: u16,
    month: u8,
    day: u8,
}

impl UiDocumentDate {
    const MINIMUM: Self = Self {
        year: 1,
        month: 1,
        day: 1,
    };
    const MAXIMUM: Self = Self {
        year: 9999,
        month: 12,
        day: 31,
    };

    fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || !bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        {
            return None;
        }
        let year = value[0..4].parse::<u16>().ok()?;
        let month = value[5..7].parse::<u8>().ok()?;
        let day = value[8..10].parse::<u8>().ok()?;
        let maximum_day = ui_document_days_in_month(year, month);
        (year >= 1 && maximum_day > 0 && (1..=maximum_day).contains(&day)).then_some(Self {
            year,
            month,
            day,
        })
    }

    const fn day(self) -> u8 {
        self.day
    }

    const fn first_day_of_month(self) -> Self {
        Self { day: 1, ..self }
    }
}

const fn ui_document_days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn ui_date_from_value(value: &Value) -> Option<UiDocumentDate> {
    value.as_str().and_then(UiDocumentDate::parse)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UiDocumentTime {
    minute: u8,
}

impl UiDocumentTime {
    fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 5
            || bytes[2] != b':'
            || !bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
        {
            return None;
        }
        let hour = value[0..2].parse::<u8>().ok()?;
        let minute = value[3..5].parse::<u8>().ok()?;
        (hour <= 23 && minute <= 59).then_some(Self { minute })
    }
}

fn ui_time_from_value(value: &Value) -> Option<UiDocumentTime> {
    value.as_str().and_then(UiDocumentTime::parse)
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

/// Application-owned secure values used by document-backed PasswordBox nodes.
///
/// This store deliberately has no Serde implementation. Its `Debug` output
/// lists binding names only, while replaced and dropped values are cleared by
/// [`ZsPassword`](crate::ZsPassword).
#[cfg(feature = "password-box")]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct UiSecretValues {
    values: BTreeMap<String, crate::ZsPassword>,
}

#[cfg(feature = "password-box")]
impl fmt::Debug for UiSecretValues {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiSecretValues")
            .field("bindings", &self.values.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(feature = "password-box")]
impl UiSecretValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        binding: impl Into<String>,
        value: impl Into<crate::ZsPassword>,
    ) -> Option<crate::ZsPassword> {
        self.values.insert(binding.into(), value.into())
    }

    pub fn get(&self, binding: &str) -> Option<&crate::ZsPassword> {
        self.values.get(binding)
    }

    pub fn remove(&mut self, binding: &str) -> Option<crate::ZsPassword> {
        self.values.remove(binding)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
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

/// Validates ordinary JSON values against both the binding schema and the
/// document's sensitive-property boundary.
///
/// PasswordBox values are never accepted from JSON, even though the public
/// binding schema records their logical type as `string`. Applications pass
/// them through [`UiSecretValues`] instead.
pub fn validate_ui_document_binding_values(
    document: &UiDocument,
    bindings: &UiBindingSchema,
    values: &BTreeMap<String, Value>,
) -> UiValidationReport {
    let mut report = validate_ui_binding_values(bindings, values);
    for binding in ui_document_sensitive_bindings(document) {
        if values.contains_key(&binding) {
            push_diagnostic(
                &mut report.diagnostics,
                UiDiagnosticCode::SensitiveBindingValue,
                format!("$.{binding}"),
                format!(
                    "sensitive property binding {binding:?} must use UiSecretValues and cannot be stored in JSON"
                ),
            );
        }
    }
    report
}

/// Returns the stable binding names whose values must remain outside JSON,
/// handoff packages, action logs and proof reports.
pub fn ui_document_sensitive_bindings(document: &UiDocument) -> BTreeSet<String> {
    fn collect(node: &UiNode, output: &mut BTreeSet<String>) {
        if node.component == "password_box" {
            if let Some(binding) = node.property_bindings.get("value") {
                output.insert(binding.clone());
            }
        }
        for child in &node.children {
            collect(child, output);
        }
    }

    let mut output = BTreeSet::new();
    collect(&document.root, &mut output);
    output
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub sensitive: bool,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sensitive_values: Vec<String>,
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
            let report = validate_ui_document_binding_values(document, bindings, values);
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
                        sensitive: component == "password_box" && property.name == "value",
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
        let sensitive_values = ui_document_sensitive_bindings(document)
            .into_iter()
            .collect::<Vec<_>>();
        let missing_values = bindings
            .properties
            .keys()
            .filter(|name| !sensitive_values.contains(name))
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
            sensitive_values,
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

fn is_false(value: &bool) -> bool {
    !*value
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
#[cfg(feature = "password-box")]
type UiSecretStateReader<State> = Arc<dyn Fn(&State) -> crate::ZsPassword + Send + Sync + 'static>;
#[cfg(feature = "password-box")]
type UiSecretActionMapper<Msg> =
    Arc<dyn Fn(crate::ZsPassword) -> Result<Msg, String> + Send + Sync + 'static>;

struct UiStateBinding<State> {
    value_type: UiValueType,
    read: UiStateReader<State>,
}

struct UiActionBinding<Msg> {
    payload_type: UiValueType,
    map: UiActionMapper<Msg>,
}

#[cfg(feature = "password-box")]
struct UiSecretStateBinding<State> {
    read: UiSecretStateReader<State>,
}

#[cfg(feature = "password-box")]
struct UiSecretActionBinding<Msg> {
    map: UiSecretActionMapper<Msg>,
}

/// Strongly typed bridge between serialized slots and application-owned Rust
/// `State`/`Msg` types.
///
/// String keys are validated contract names, not a global event bus. Action
/// dispatch always returns the manifest's concrete `Msg` type.
pub struct UiBindingManifest<State, Msg> {
    properties: BTreeMap<String, UiStateBinding<State>>,
    actions: BTreeMap<String, UiActionBinding<Msg>>,
    #[cfg(feature = "password-box")]
    secret_properties: BTreeMap<String, UiSecretStateBinding<State>>,
    #[cfg(feature = "password-box")]
    secret_actions: BTreeMap<String, UiSecretActionBinding<Msg>>,
}

impl<State, Msg> Default for UiBindingManifest<State, Msg> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State, Msg> fmt::Debug for UiBindingManifest<State, Msg> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("UiBindingManifest");
        debug
            .field("properties", &self.properties.keys().collect::<Vec<_>>())
            .field("actions", &self.actions.keys().collect::<Vec<_>>());
        #[cfg(feature = "password-box")]
        {
            debug.field(
                "secret_properties",
                &self.secret_properties.keys().collect::<Vec<_>>(),
            );
            debug.field(
                "secret_actions",
                &self.secret_actions.keys().collect::<Vec<_>>(),
            );
        }
        debug.finish()
    }
}

impl<State, Msg> UiBindingManifest<State, Msg> {
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
            #[cfg(feature = "password-box")]
            secret_properties: BTreeMap::new(),
            #[cfg(feature = "password-box")]
            secret_actions: BTreeMap::new(),
        }
    }

    pub fn register_property(
        &mut self,
        name: impl Into<String>,
        value_type: UiValueType,
        read: impl Fn(&State) -> Value + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.contains_binding(&name) {
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

    /// Registers a strongly typed calendar-date property using the canonical
    /// platform-independent `YYYY-MM-DD` document representation.
    #[cfg(feature = "date-picker")]
    pub fn register_date_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> crate::ZsDate + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::Date, move |state| {
            Value::String(read(state).iso_string())
        })
    }

    /// Registers a strongly typed time property using the canonical
    /// platform-independent `HH:MM` 24-hour document representation.
    #[cfg(feature = "time-picker")]
    pub fn register_time_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> crate::ZsTime + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::Time, move |state| {
            Value::String(read(state).to_string())
        })
    }

    /// Registers a strongly typed color property using the canonical
    /// platform-independent `#RRGGBBAA` representation.
    #[cfg(feature = "color-picker")]
    pub fn register_color_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> crate::Color + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::Color, move |state| {
            Value::String(read(state).hex_rgba())
        })
    }

    /// Registers a root-to-current breadcrumb path with stable semantic IDs.
    #[cfg(feature = "breadcrumb")]
    pub fn register_breadcrumb_items_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiBreadcrumbItem> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::BreadcrumbItemArray, move |state| {
            serde_json::to_value(read(state)).expect("breadcrumb authoring metadata must serialize")
        })
    }

    /// Registers one complete MenuFlyout tree with stable semantic item IDs.
    #[cfg(feature = "menu-flyout")]
    pub fn register_menu_flyout_items_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiMenuFlyoutItem> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::MenuFlyoutItemArray, move |state| {
            serde_json::to_value(read(state)).expect("menu-flyout metadata must serialize")
        })
    }

    /// Registers application-owned suggestions with stable semantic IDs.
    #[cfg(feature = "auto-suggest")]
    pub fn register_auto_suggestions_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiAutoSuggestion> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::AutoSuggestionArray, move |state| {
            Value::Array(
                read(state)
                    .into_iter()
                    .map(|suggestion| {
                        let (id, text) = suggestion.into_parts();
                        serde_json::json!({ "id": id.as_str(), "text": text })
                    })
                    .collect(),
            )
        })
    }

    /// Registers the optional highlighted suggestion without exposing the
    /// runtime's numeric suggestion identity.
    #[cfg(feature = "auto-suggest")]
    pub fn register_auto_suggestion_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiAutoSuggestionId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableAutoSuggestionId, move |state| {
            read(state)
                .map(|id| Value::String(id.as_str().to_owned()))
                .unwrap_or(Value::Null)
        })
    }

    /// Registers application-owned command metadata with stable semantic IDs.
    #[cfg(feature = "command-palette")]
    pub fn register_command_palette_items_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiCommandPaletteItem> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::CommandPaletteItemArray, move |state| {
            serde_json::to_value(read(state))
                .expect("command-palette authoring metadata must serialize")
        })
    }

    /// Registers the optional highlighted command without exposing the
    /// runtime's numeric command identity.
    #[cfg(feature = "command-palette")]
    pub fn register_command_palette_item_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiCommandPaletteItemId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(
            name,
            UiValueType::NullableCommandPaletteItemId,
            move |state| {
                read(state)
                    .map(|id| Value::String(id.as_str().to_owned()))
                    .unwrap_or(Value::Null)
            },
        )
    }

    /// Registers application-owned TreeView hierarchy metadata with stable
    /// semantic node IDs.
    #[cfg(feature = "tree")]
    pub fn register_tree_nodes_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiTreeNode> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::TreeNodeArray, move |state| {
            serde_json::to_value(read(state)).expect("tree authoring metadata must serialize")
        })
    }

    /// Registers the complete expanded-node set for a controlled TreeView.
    #[cfg(feature = "tree")]
    pub fn register_tree_node_ids_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> BTreeSet<UiTreeNodeId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::TreeNodeIdArray, move |state| {
            serde_json::to_value(read(state)).expect("tree node ids must serialize")
        })
    }

    /// Registers the optional selected TreeView node without exposing the
    /// runtime's numeric node identity.
    #[cfg(feature = "tree")]
    pub fn register_tree_node_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiTreeNodeId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableTreeNodeId, move |state| {
            read(state)
                .map(|id| Value::String(id.as_str().to_owned()))
                .unwrap_or(Value::Null)
        })
    }

    /// Registers one NavigationView group with stable semantic item IDs.
    #[cfg(feature = "shell")]
    pub fn register_navigation_items_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiNavigationItem> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NavigationItemArray, move |state| {
            serde_json::to_value(read(state)).expect("navigation authoring metadata must serialize")
        })
    }

    /// Registers the optional selected NavigationView item without exposing
    /// the private runtime WidgetId.
    #[cfg(feature = "shell")]
    pub fn register_navigation_item_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiNavigationItemId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableNavigationItemId, move |state| {
            read(state)
                .map(|id| Value::String(id.as_str().to_owned()))
                .unwrap_or(Value::Null)
        })
    }

    /// Registers application-owned GridView tile metadata with stable
    /// semantic item IDs.
    #[cfg(feature = "grid-view")]
    pub fn register_grid_view_items_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiGridViewItem> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::GridViewItemArray, move |state| {
            serde_json::to_value(read(state)).expect("grid-view authoring metadata must serialize")
        })
    }

    /// Registers the optional selected GridView item without exposing the
    /// runtime's numeric item identity.
    #[cfg(feature = "grid-view")]
    pub fn register_grid_view_item_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiGridViewItemId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableGridViewItemId, move |state| {
            read(state)
                .map(|id| Value::String(id.as_str().to_owned()))
                .unwrap_or(Value::Null)
        })
    }

    /// Registers application-owned DataGrid column metadata with stable
    /// semantic column IDs.
    #[cfg(feature = "table")]
    pub fn register_table_columns_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiTableColumn> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::TableColumnArray, move |state| {
            serde_json::to_value(read(state)).expect("table column metadata must serialize")
        })
    }

    /// Registers application-owned DataGrid rows with cells keyed by stable
    /// semantic column IDs.
    #[cfg(feature = "table")]
    pub fn register_table_rows_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Vec<UiTableRow> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::TableRowArray, move |state| {
            serde_json::to_value(read(state)).expect("table row data must serialize")
        })
    }

    /// Registers the optional selected DataGrid row without exposing the
    /// runtime's numeric row identity.
    #[cfg(feature = "table")]
    pub fn register_table_row_id_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiTableRowId> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableTableRowId, move |state| {
            read(state)
                .map(|id| Value::String(id.as_str().to_owned()))
                .unwrap_or(Value::Null)
        })
    }

    /// Registers the optional application-owned DataGrid sort descriptor.
    #[cfg(feature = "table")]
    pub fn register_table_sort_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> Option<UiTableSort> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_property(name, UiValueType::NullableTableSort, move |state| {
            read(state)
                .map(|sort| serde_json::to_value(sort).expect("table sort must serialize"))
                .unwrap_or(Value::Null)
        })
    }

    pub fn register_action(
        &mut self,
        name: impl Into<String>,
        payload_type: UiValueType,
        map: impl Fn(Value) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.contains_binding(&name) {
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

    /// Registers a strongly typed calendar-date action. Invalid or
    /// noncanonical serialized dates are rejected before application update.
    #[cfg(feature = "date-picker")]
    pub fn register_date_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(crate::ZsDate) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::Date, move |payload| {
            let date = payload
                .as_str()
                .ok_or_else(|| "date payload must be a YYYY-MM-DD string".to_owned())
                .and_then(|value| {
                    crate::ZsDate::parse_iso(value).map_err(|error| error.to_string())
                })?;
            map(date)
        })
    }

    /// Registers a strongly typed wall-clock action. Invalid or noncanonical
    /// serialized times are rejected before application update.
    #[cfg(feature = "time-picker")]
    pub fn register_time_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(crate::ZsTime) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::Time, move |payload| {
            let time = payload
                .as_str()
                .ok_or_else(|| "time payload must be an HH:MM string".to_owned())
                .and_then(|value| {
                    crate::ZsTime::parse_24_hour(value).map_err(|error| error.to_string())
                })?;
            map(time)
        })
    }

    /// Registers a strongly typed RGBA action. Noncanonical serialized colors
    /// are rejected before application update.
    #[cfg(feature = "color-picker")]
    pub fn register_color_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(crate::Color) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::Color, move |payload| {
            let color = payload
                .as_str()
                .ok_or_else(|| "color payload must be a #RRGGBBAA string".to_owned())
                .and_then(|value| {
                    crate::Color::parse_hex_rgba(value)
                        .ok_or_else(|| "color payload must use canonical #RRGGBBAA".to_owned())
                })?;
            map(color)
        })
    }

    /// Registers a strongly typed Flyout dismissal action.
    #[cfg(feature = "flyout")]
    pub fn register_flyout_dismiss_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(crate::ZsFlyoutDismissReason) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::FlyoutDismissReason, move |payload| {
            let reason = match payload.as_str() {
                Some("light_dismiss") => crate::ZsFlyoutDismissReason::LightDismiss,
                Some("escape") => crate::ZsFlyoutDismissReason::EscapeKey,
                _ => {
                    return Err(
                        "flyout dismissal payload must be light_dismiss or escape".to_owned()
                    )
                }
            };
            map(reason)
        })
    }

    /// Registers a strongly typed MenuFlyout command invocation.
    #[cfg(feature = "menu-flyout")]
    pub fn register_menu_flyout_item_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiMenuFlyoutItemId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::MenuFlyoutItemId, move |payload| {
            let id = ui_menu_flyout_item_id_from_value(&payload)
                .ok_or_else(|| "menu-flyout payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed breadcrumb selection action.
    #[cfg(feature = "breadcrumb")]
    pub fn register_breadcrumb_item_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiBreadcrumbItemId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::BreadcrumbItemId, move |payload| {
            let id = ui_breadcrumb_item_id_from_value(&payload)
                .ok_or_else(|| "breadcrumb payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed suggestion-selection action.
    #[cfg(feature = "auto-suggest")]
    pub fn register_auto_suggestion_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiAutoSuggestionId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::AutoSuggestionId, move |payload| {
            let id = ui_auto_suggestion_id_from_value(&payload)
                .ok_or_else(|| "suggestion payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers the structured query-submission action containing the query
    /// and optional stable suggestion ID.
    #[cfg(feature = "auto-suggest")]
    pub fn register_auto_suggest_submission_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiAutoSuggestSubmission) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::AutoSuggestSubmission, move |payload| {
            let submission = ui_auto_suggest_submission_from_value(&payload).ok_or_else(|| {
                "auto-suggest submission must contain query and chosen fields".to_owned()
            })?;
            map(submission)
        })
    }

    /// Registers a strongly typed command highlight or invocation action.
    #[cfg(feature = "command-palette")]
    pub fn register_command_palette_item_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiCommandPaletteItemId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::CommandPaletteItemId, move |payload| {
            let id = ui_command_palette_item_id_from_value(&payload)
                .ok_or_else(|| "command payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed TreeView selection or invocation action.
    #[cfg(feature = "tree")]
    pub fn register_tree_node_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiTreeNodeId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::TreeNodeId, move |payload| {
            let id = ui_tree_node_id_from_value(&payload)
                .ok_or_else(|| "tree payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers the complete expanded-node set emitted after one disclosure
    /// state change.
    #[cfg(feature = "tree")]
    pub fn register_tree_node_ids_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(BTreeSet<UiTreeNodeId>) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::TreeNodeIdArray, move |payload| {
            let ids = ui_tree_node_ids_from_value(&payload)
                .ok_or_else(|| "tree payload must contain unique stable string ids".to_owned())?;
            map(ids)
        })
    }

    /// Registers a strongly typed NavigationView selection action.
    #[cfg(feature = "shell")]
    pub fn register_navigation_item_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiNavigationItemId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::NavigationItemId, move |payload| {
            let id = ui_navigation_item_id_from_value(&payload)
                .ok_or_else(|| "navigation payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed GridView selection or invocation action.
    #[cfg(feature = "grid-view")]
    pub fn register_grid_view_item_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiGridViewItemId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::GridViewItemId, move |payload| {
            let id = ui_grid_view_item_id_from_value(&payload)
                .ok_or_else(|| "grid-view payload must be a valid stable string id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed DataGrid row selection or invocation.
    #[cfg(feature = "table")]
    pub fn register_table_row_id_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiTableRowId) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::TableRowId, move |payload| {
            let id = ui_table_row_id_from_value(&payload)
                .ok_or_else(|| "table payload must be a valid stable row id".to_owned())?;
            map(id)
        })
    }

    /// Registers a strongly typed DataGrid sort action.
    #[cfg(feature = "table")]
    pub fn register_table_sort_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(UiTableSort) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        self.register_action(name, UiValueType::TableSort, move |payload| {
            let sort = ui_table_sort_from_value(&payload)
                .ok_or_else(|| "table payload must be a valid sort descriptor".to_owned())?;
            map(sort)
        })
    }

    pub fn schema(&self) -> UiBindingSchema {
        #[allow(unused_mut)]
        let mut properties = self
            .properties
            .iter()
            .map(|(name, binding)| (name.clone(), binding.value_type))
            .collect::<BTreeMap<_, _>>();
        #[allow(unused_mut)]
        let mut actions = self
            .actions
            .iter()
            .map(|(name, binding)| (name.clone(), binding.payload_type))
            .collect::<BTreeMap<_, _>>();
        #[cfg(feature = "password-box")]
        {
            properties.extend(
                self.secret_properties
                    .keys()
                    .map(|name| (name.clone(), UiValueType::String)),
            );
            actions.extend(
                self.secret_actions
                    .keys()
                    .map(|name| (name.clone(), UiValueType::String)),
            );
        }
        UiBindingSchema {
            properties,
            actions,
        }
    }

    fn contains_binding(&self, name: &str) -> bool {
        let ordinary = self.properties.contains_key(name) || self.actions.contains_key(name);
        #[cfg(feature = "password-box")]
        {
            ordinary
                || self.secret_properties.contains_key(name)
                || self.secret_actions.contains_key(name)
        }
        #[cfg(not(feature = "password-box"))]
        {
            ordinary
        }
    }

    /// Registers a password state reader that never creates a JSON `Value`.
    #[cfg(feature = "password-box")]
    pub fn register_secret_property(
        &mut self,
        name: impl Into<String>,
        read: impl Fn(&State) -> crate::ZsPassword + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.contains_binding(&name) {
            return Err(UiBindingRegistrationError::Duplicate(name));
        }
        self.secret_properties.insert(
            name,
            UiSecretStateBinding {
                read: Arc::new(read),
            },
        );
        Ok(())
    }

    /// Registers a password action mapper without lowering its payload to
    /// Serde JSON or a printable string.
    #[cfg(feature = "password-box")]
    pub fn register_secret_action(
        &mut self,
        name: impl Into<String>,
        map: impl Fn(crate::ZsPassword) -> Result<Msg, String> + Send + Sync + 'static,
    ) -> Result<(), UiBindingRegistrationError> {
        let name = validate_binding_name(name.into())?;
        if self.contains_binding(&name) {
            return Err(UiBindingRegistrationError::Duplicate(name));
        }
        self.secret_actions
            .insert(name, UiSecretActionBinding { map: Arc::new(map) });
        Ok(())
    }

    #[cfg(feature = "password-box")]
    pub fn read_secret_property(&self, name: &str, state: &State) -> Option<crate::ZsPassword> {
        self.secret_properties
            .get(name)
            .map(|binding| (binding.read)(state))
    }

    #[cfg(feature = "password-box")]
    pub fn read_secret_values(&self, state: &State) -> UiSecretValues {
        let mut values = UiSecretValues::new();
        for (name, binding) in &self.secret_properties {
            values.insert(name.clone(), (binding.read)(state));
        }
        values
    }

    #[cfg(feature = "password-box")]
    pub fn map_secret_action(
        &self,
        name: &str,
        payload: crate::ZsPassword,
    ) -> Result<Msg, UiBindingDispatchError> {
        let binding = self
            .secret_actions
            .get(name)
            .ok_or_else(|| UiBindingDispatchError::UnknownAction(name.to_owned()))?;
        (binding.map)(payload).map_err(|message| UiBindingDispatchError::Rejected {
            action: name.to_owned(),
            message,
        })
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
        include_feature!("badge");
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
        include_feature!("icon");
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
    SensitiveBindingValue,
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

        if node.component == "badge" {
            let has_value = node.properties.contains_key("value")
                || node.property_bindings.contains_key("value")
                || node.localization.contains_key("value");
            let has_icon = node.properties.contains_key("icon")
                || node.property_bindings.contains_key("icon")
                || node.localization.contains_key("icon");
            if node.property_bindings.contains_key("kind") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.kind"),
                    "badge kind is structural metadata and must be a static dot, number or icon value"
                        .to_owned(),
                );
            }
            if node.localization.contains_key("kind") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.kind"),
                    "badge kind is semantic metadata and cannot be localized".to_owned(),
                );
            }
            let kind = node.properties.get("kind").and_then(Value::as_str);
            if kind.is_some_and(|kind| !matches!(kind, "dot" | "number" | "icon")) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.kind"),
                    "badge kind must be dot, number or icon".to_owned(),
                );
            }
            match kind {
                Some("dot") if has_value || has_icon => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties"),
                    "dot badges do not accept value or icon content".to_owned(),
                ),
                Some("number") if !has_value => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::MissingRequiredProperty,
                    format!("{path}.properties.value"),
                    "number badges require a nonnegative integer value".to_owned(),
                ),
                Some("number") if has_icon => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.icon"),
                    "number badges do not accept an icon".to_owned(),
                ),
                Some("icon") if !has_icon => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::MissingRequiredProperty,
                    format!("{path}.properties.icon"),
                    "icon badges require a semantic ZsIcon value".to_owned(),
                ),
                Some("icon") if has_value => push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.value"),
                    "icon badges do not accept a numeric value".to_owned(),
                ),
                _ => {}
            }
            if node
                .properties
                .get("value")
                .and_then(Value::as_u64)
                .is_some_and(|value| value > u64::from(u32::MAX))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.value"),
                    "badge value must fit in an unsigned 32-bit integer".to_owned(),
                );
            }
            let static_icon = (!node.property_bindings.contains_key("icon")
                && !node.localization.contains_key("icon"))
            .then(|| node.properties.get("icon").and_then(Value::as_str))
            .flatten();
            if static_icon.is_some_and(|icon| {
                serde_json::from_value::<crate::ZsIcon>(Value::String(icon.to_owned())).is_err()
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.icon"),
                    "badge icon must name a ZsIcon semantic variant".to_owned(),
                );
            }
            let static_tone = (!node.property_bindings.contains_key("tone")
                && !node.localization.contains_key("tone"))
            .then(|| {
                node.properties
                    .get("tone")
                    .and_then(Value::as_str)
                    .unwrap_or("accent")
            });
            if static_tone.is_some_and(|tone| {
                !matches!(
                    tone,
                    "neutral" | "accent" | "success" | "warning" | "danger"
                )
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.tone"),
                    "badge tone must be neutral, accent, success, warning or danger".to_owned(),
                );
            }
            for name in ["icon", "tone"] {
                if node.localization.contains_key(name) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidLocalization,
                        format!("{path}.localization.{name}"),
                        format!("badge {name} is semantic metadata and cannot be localized"),
                    );
                }
            }
        }

        if node.component == "icon" {
            let static_value = (!node.property_bindings.contains_key("icon")
                && !node.localization.contains_key("icon"))
            .then(|| node.properties.get("icon").and_then(Value::as_str))
            .flatten();
            let static_size = (!node.property_bindings.contains_key("size")
                && !node.localization.contains_key("size"))
            .then(|| {
                node.properties
                    .get("size")
                    .and_then(Value::as_str)
                    .unwrap_or("standard")
            });
            let static_color = (!node.property_bindings.contains_key("color")
                && !node.localization.contains_key("color"))
            .then(|| {
                node.properties
                    .get("color")
                    .and_then(Value::as_str)
                    .unwrap_or("primary")
            });
            if static_value.is_some_and(|icon| {
                serde_json::from_value::<crate::ZsIcon>(Value::String(icon.to_owned())).is_err()
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.icon"),
                    "icon must name a ZsIcon semantic variant".to_owned(),
                );
            }
            if static_size.is_some_and(|size| !matches!(size, "small" | "standard" | "large")) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.size"),
                    "icon size must be small, standard or large".to_owned(),
                );
            }
            if static_color.is_some_and(|color| {
                !matches!(
                    color,
                    "primary"
                        | "secondary"
                        | "disabled"
                        | "accent"
                        | "accent_text"
                        | "surface"
                        | "surface_raised"
                        | "control"
                        | "border"
                        | "success"
                        | "warning"
                        | "danger"
                )
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.color"),
                    "icon color must name a semantic theme color role".to_owned(),
                );
            }
            if (node.properties.contains_key("color")
                || node.property_bindings.contains_key("color"))
                && node.theme_tokens.contains_key("foreground")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::ConflictingPropertySource,
                    format!("{path}.theme_tokens.foreground"),
                    "icon color must use either the color property/binding or the foreground theme token"
                        .to_owned(),
                );
            }
            for name in ["icon", "size", "color"] {
                if node.localization.contains_key(name) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidLocalization,
                        format!("{path}.localization.{name}"),
                        format!("icon {name} is semantic metadata and cannot be localized"),
                    );
                }
            }
        }

        if node.component == "button" {
            let static_presentation = (!node.property_bindings.contains_key("presentation")
                && !node.localization.contains_key("presentation"))
            .then(|| {
                node.properties
                    .get("presentation")
                    .and_then(Value::as_str)
                    .unwrap_or("standard")
            });
            let static_icon = (!node.property_bindings.contains_key("icon")
                && !node.localization.contains_key("icon"))
            .then(|| node.properties.get("icon").and_then(Value::as_str))
            .flatten();
            if node
                .properties
                .get("label")
                .and_then(Value::as_str)
                .is_some_and(|label| label.trim().is_empty())
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.label"),
                    "button label must not be empty".to_owned(),
                );
            }
            if static_presentation.is_some_and(|presentation| {
                !matches!(presentation, "standard" | "primary" | "toolbar" | "icon")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.presentation"),
                    "button presentation must be standard, primary, toolbar or icon".to_owned(),
                );
            }
            if static_icon.is_some_and(|icon| {
                serde_json::from_value::<crate::ZsIcon>(Value::String(icon.to_owned())).is_err()
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.icon"),
                    "button icon must name a ZsIcon semantic variant".to_owned(),
                );
            }
            if static_presentation.is_some_and(|presentation| {
                matches!(presentation, "toolbar" | "icon")
                    && !node.properties.contains_key("icon")
                    && !node.property_bindings.contains_key("icon")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::MissingRequiredProperty,
                    format!("{path}.properties.icon"),
                    "toolbar and icon button presentations require a semantic icon".to_owned(),
                );
            }
            if static_presentation.is_some_and(|presentation| {
                matches!(presentation, "standard" | "primary") && static_icon.is_some()
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.icon"),
                    "standard and primary button presentations do not accept an icon".to_owned(),
                );
            }
            for name in ["presentation", "icon"] {
                if node.localization.contains_key(name) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidLocalization,
                        format!("{path}.localization.{name}"),
                        format!("button {name} is semantic metadata and cannot be localized"),
                    );
                }
            }
        }

        if node.component == "command_bar" {
            let child_ids = node
                .children
                .iter()
                .map(|child| child.id.as_str())
                .collect::<BTreeSet<_>>();
            if let Some(trailing) = node.properties.get("trailing").and_then(Value::as_array) {
                let mut seen = BTreeSet::new();
                for (index, item) in trailing.iter().enumerate() {
                    let Some(item) = item.as_str() else {
                        continue;
                    };
                    if !seen.insert(item) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.trailing[{index}]"),
                            format!("command_bar trailing child id {item:?} is duplicated"),
                        );
                    } else if !child_ids.contains(item) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.trailing[{index}]"),
                            format!("command_bar trailing id {item:?} does not address a child"),
                        );
                    }
                }
            }
            if node
                .properties
                .get("gap")
                .and_then(Value::as_f64)
                .is_some_and(|gap| !gap.is_finite() || gap < 0.0)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.gap"),
                    "command_bar gap must be a finite non-negative DP value".to_owned(),
                );
            }
            if node.localization.contains_key("trailing") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.trailing"),
                    "command_bar trailing contains stable child IDs and cannot be localized"
                        .to_owned(),
                );
            }
        }

        if node.component == "text" {
            for (name, allowed) in [
                (
                    "text_role",
                    &[
                        "body",
                        "caption",
                        "body_large",
                        "subtitle",
                        "title",
                        "title_large",
                        "display",
                    ][..],
                ),
                ("wrap", &["no_wrap", "word"][..]),
                (
                    "weight",
                    &["automatic", "regular", "medium", "semibold", "bold"][..],
                ),
                ("horizontal_align", &["start", "center", "end"][..]),
                ("vertical_align", &["start", "center", "end"][..]),
            ] {
                if let Some(value) = node.properties.get(name).and_then(Value::as_str) {
                    if !allowed.contains(&value) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.{name}"),
                            format!(
                                "text property {name:?} must be one of {}",
                                allowed.join(", ")
                            ),
                        );
                    }
                }
            }
        }

        if node.component == "password_box" {
            if node.properties.contains_key("value") || node.localization.contains_key("value") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::SensitiveBindingValue,
                    format!("{path}.properties.value"),
                    "password_box value must use a secure property binding; literals and localization values are not allowed"
                        .to_owned(),
                );
            }
            if node.action_bindings.contains_key("change")
                && !node.property_bindings.contains_key("value")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::SensitiveBindingValue,
                    format!("{path}.action_bindings.change"),
                    "password_box change requires a secure value property binding so rebuilt views retain the secret safely"
                        .to_owned(),
                );
            }
            if let Some(mode) = node.properties.get("reveal_mode").and_then(Value::as_str) {
                if !matches!(mode, "platform_default" | "hidden" | "peek" | "visible") {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.reveal_mode"),
                        "password_box reveal_mode must be platform_default, hidden, peek or visible"
                            .to_owned(),
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

        if node.component == "auto_suggest" {
            let static_suggestions = (!node.property_bindings.contains_key("suggestions"))
                .then(|| {
                    node.properties
                        .get("suggestions")
                        .and_then(ui_auto_suggestions_from_value)
                })
                .flatten();
            let static_highlighted = (!node.property_bindings.contains_key("highlighted"))
                .then(|| {
                    node.properties
                        .get("highlighted")
                        .filter(|value| !value.is_null())
                        .and_then(ui_auto_suggestion_id_from_value)
                })
                .flatten();
            if static_highlighted
                .as_ref()
                .zip(static_suggestions.as_ref())
                .is_some_and(|(highlighted, suggestions)| {
                    !suggestions
                        .iter()
                        .any(|suggestion| suggestion.id() == highlighted)
                })
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.highlighted"),
                    "auto_suggest highlighted must address an available suggestion id".to_owned(),
                );
            }
        }

        if node.component == "command_palette" {
            let static_items = (!node.property_bindings.contains_key("items"))
                .then(|| {
                    node.properties
                        .get("items")
                        .and_then(ui_command_palette_items_from_value)
                })
                .flatten();
            let static_query = (!node.property_bindings.contains_key("query")).then(|| {
                node.properties
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            });
            let static_highlighted = (!node.property_bindings.contains_key("highlighted"))
                .then(|| {
                    node.properties
                        .get("highlighted")
                        .filter(|value| !value.is_null())
                        .and_then(ui_command_palette_item_id_from_value)
                })
                .flatten();
            if let (Some(highlighted), Some(items)) =
                (static_highlighted.as_ref(), static_items.as_ref())
            {
                match items.iter().find(|item| item.id() == highlighted) {
                    None => push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.highlighted"),
                        "command_palette highlighted must address an available command id"
                            .to_owned(),
                    ),
                    Some(item) if !item.is_enabled() => push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.highlighted"),
                        "command_palette highlighted must address an enabled command".to_owned(),
                    ),
                    Some(item) if static_query.is_some_and(|query| !item.matches_query(query)) => {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.highlighted"),
                            "command_palette highlighted must match the current query".to_owned(),
                        )
                    }
                    Some(_) => {}
                }
            }
        }

        if node.component == "tree" {
            let static_nodes = (!node.property_bindings.contains_key("nodes"))
                .then(|| {
                    node.properties
                        .get("nodes")
                        .and_then(ui_tree_nodes_from_value)
                })
                .flatten();
            let static_expanded = (!node.property_bindings.contains_key("expanded"))
                .then(|| {
                    node.properties
                        .get("expanded")
                        .and_then(ui_tree_node_ids_from_value)
                })
                .flatten();
            let static_selected = (!node.property_bindings.contains_key("selected"))
                .then(|| {
                    node.properties
                        .get("selected")
                        .filter(|value| !value.is_null())
                        .and_then(ui_tree_node_id_from_value)
                })
                .flatten();
            if let Some(nodes) = static_nodes.as_ref() {
                if static_selected
                    .as_ref()
                    .is_some_and(|selected| find_ui_tree_node(nodes, selected).is_none())
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.selected"),
                        "tree selected must address an available node id".to_owned(),
                    );
                }
                if let Some(expanded) = static_expanded.as_ref() {
                    for id in expanded {
                        match find_ui_tree_node(nodes, id) {
                            None => push_diagnostic(
                                diagnostics,
                                UiDiagnosticCode::InvalidPropertyValue,
                                format!("{path}.properties.expanded"),
                                format!(
                                    "tree expanded id {:?} does not address an available node",
                                    id.as_str()
                                ),
                            ),
                            Some(tree_node) if !tree_node.is_expandable() => push_diagnostic(
                                diagnostics,
                                UiDiagnosticCode::InvalidPropertyValue,
                                format!("{path}.properties.expanded"),
                                format!(
                                    "tree expanded id {:?} must address an expandable node",
                                    id.as_str()
                                ),
                            ),
                            Some(_) => {}
                        }
                    }
                }
            }
        }

        if node.component == "navigation" {
            let static_items = (!node.property_bindings.contains_key("items"))
                .then(|| {
                    node.properties
                        .get("items")
                        .and_then(ui_navigation_items_from_value)
                })
                .flatten();
            let static_footer_items =
                (!node.property_bindings.contains_key("footer_items")).then(|| {
                    node.properties
                        .get("footer_items")
                        .and_then(ui_navigation_items_from_value)
                        .unwrap_or_default()
                });
            let static_selected = (!node.property_bindings.contains_key("selected"))
                .then(|| {
                    node.properties
                        .get("selected")
                        .filter(|value| !value.is_null())
                        .and_then(ui_navigation_item_id_from_value)
                })
                .flatten();
            if static_items.as_ref().is_some_and(Vec::is_empty) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.items"),
                    "navigation items must contain at least one item".to_owned(),
                );
            }
            if let (Some(items), Some(footer_items)) =
                (static_items.as_ref(), static_footer_items.as_ref())
            {
                let item_ids = items
                    .iter()
                    .map(UiNavigationItem::id)
                    .collect::<BTreeSet<_>>();
                if let Some(duplicate) = footer_items
                    .iter()
                    .map(UiNavigationItem::id)
                    .find(|id| item_ids.contains(id))
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.footer_items"),
                        format!(
                            "navigation item id {:?} is duplicated across items and footer_items",
                            duplicate.as_str()
                        ),
                    );
                }
            }
            if let Some(selected) = static_selected.as_ref() {
                let selected_item = static_items
                    .iter()
                    .flat_map(|items| items.iter())
                    .chain(static_footer_items.iter().flat_map(|items| items.iter()))
                    .find(|item| item.id() == selected);
                match selected_item {
                    None if static_items.is_some() && static_footer_items.is_some() => {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.selected"),
                            "navigation selected must address an available item id".to_owned(),
                        );
                    }
                    Some(item) if !item.is_enabled() => push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.selected"),
                        "navigation selected must address an enabled item".to_owned(),
                    ),
                    None | Some(_) => {}
                }
            }
            for name in ["pane_width", "minimum_content_width"] {
                if node
                    .properties
                    .get(name)
                    .and_then(Value::as_f64)
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("navigation {name} must be a finite non-negative DP extent"),
                    );
                }
            }
            if node.property_bindings.contains_key("selected")
                && !node.action_bindings.contains_key("select")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.selected"),
                    "bound navigation selected state requires a select action".to_owned(),
                );
            }
        }

        if node.component == "grid_view" {
            let static_items = (!node.property_bindings.contains_key("items"))
                .then(|| {
                    node.properties
                        .get("items")
                        .and_then(ui_grid_view_items_from_value)
                })
                .flatten();
            let static_selected = (!node.property_bindings.contains_key("selected"))
                .then(|| {
                    node.properties
                        .get("selected")
                        .filter(|value| !value.is_null())
                        .and_then(ui_grid_view_item_id_from_value)
                })
                .flatten();
            if static_selected
                .as_ref()
                .zip(static_items.as_ref())
                .is_some_and(|(selected, items)| !items.iter().any(|item| item.id() == selected))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.selected"),
                    "grid_view selected must address an available item id".to_owned(),
                );
            }
        }

        if node.component == "table" {
            let static_columns = (!node.property_bindings.contains_key("columns"))
                .then(|| {
                    node.properties
                        .get("columns")
                        .and_then(ui_table_columns_from_value)
                })
                .flatten();
            let static_rows = (!node.property_bindings.contains_key("rows"))
                .then(|| {
                    node.properties
                        .get("rows")
                        .and_then(ui_table_rows_from_value)
                })
                .flatten();
            let static_selected = (!node.property_bindings.contains_key("selected"))
                .then(|| {
                    node.properties
                        .get("selected")
                        .filter(|value| !value.is_null())
                        .and_then(ui_table_row_id_from_value)
                })
                .flatten();
            let static_sort = (!node.property_bindings.contains_key("sort"))
                .then(|| {
                    node.properties
                        .get("sort")
                        .filter(|value| !value.is_null())
                        .and_then(ui_table_sort_from_value)
                })
                .flatten();
            if static_columns
                .as_ref()
                .zip(static_rows.as_ref())
                .is_some_and(|(columns, rows)| !ui_table_data_is_compatible(columns, rows))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.rows"),
                    "table rows must contain exactly one cell for every declared column id"
                        .to_owned(),
                );
            }
            if static_selected
                .as_ref()
                .zip(static_rows.as_ref())
                .is_some_and(|(selected, rows)| !rows.iter().any(|row| row.id() == selected))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.selected"),
                    "table selected must address an available row id".to_owned(),
                );
            }
            if let (Some(sort), Some(columns)) = (static_sort.as_ref(), static_columns.as_ref()) {
                if !columns
                    .iter()
                    .any(|column| column.id() == sort.column() && column.is_sortable())
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.sort"),
                        "table sort must address an available sortable column id".to_owned(),
                    );
                }
            }
        }

        if node.component == "date_picker" {
            let static_date = |name: &str, fallback: Option<UiDocumentDate>| {
                (!node.property_bindings.contains_key(name))
                    .then(|| {
                        node.properties
                            .get(name)
                            .and_then(ui_date_from_value)
                            .or(fallback)
                    })
                    .flatten()
            };
            let minimum = static_date("minimum", Some(UiDocumentDate::MINIMUM));
            let maximum = static_date("maximum", Some(UiDocumentDate::MAXIMUM));
            if minimum
                .zip(maximum)
                .is_some_and(|(minimum, maximum)| minimum > maximum)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.maximum"),
                    "date_picker maximum must not be earlier than minimum".to_owned(),
                );
            }
            if let (Some(value), Some(minimum), Some(maximum)) =
                (static_date("value", None), minimum, maximum)
            {
                if minimum <= maximum && !(minimum..=maximum).contains(&value) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.value"),
                        "date_picker value must be within minimum and maximum".to_owned(),
                    );
                }
            }
            if let Some(month) = static_date("visible_month", None) {
                if month.day() != 1 {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.visible_month"),
                        "date_picker visible_month must use the first day of its month".to_owned(),
                    );
                } else if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
                    let first = minimum.first_day_of_month();
                    let last = maximum.first_day_of_month();
                    if first <= last && !(first..=last).contains(&month) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.visible_month"),
                            "date_picker visible_month must intersect the configured date range"
                                .to_owned(),
                        );
                    }
                }
            }
        }

        if node.component == "time_picker" {
            let static_increment = (!node.property_bindings.contains_key("minute_increment"))
                .then(|| {
                    node.properties
                        .get("minute_increment")
                        .map(Value::as_u64)
                        .unwrap_or(Some(1))
                })
                .flatten();
            let valid_increment = static_increment.and_then(|increment| {
                u8::try_from(increment)
                    .ok()
                    .filter(|increment| *increment > 0 && *increment < 60 && 60 % increment == 0)
            });
            if static_increment.is_some() && valid_increment.is_none() {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.minute_increment"),
                    "time_picker minute_increment must be a non-zero divisor of 60 smaller than 60"
                        .to_owned(),
                );
            }
            if !node.property_bindings.contains_key("clock_format") {
                if let Some(clock) = node.properties.get("clock_format").and_then(Value::as_str) {
                    if !matches!(
                        clock,
                        "platform_default" | "twelve_hour" | "twenty_four_hour"
                    ) {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.clock_format"),
                            "time_picker clock_format must be platform_default, twelve_hour or twenty_four_hour"
                                .to_owned(),
                        );
                    }
                }
            }
            let static_value = (!node.property_bindings.contains_key("value"))
                .then(|| node.properties.get("value").and_then(ui_time_from_value))
                .flatten();
            if static_value
                .zip(valid_increment)
                .is_some_and(|(time, increment)| time.minute % increment != 0)
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.value"),
                    "time_picker value minute must align with minute_increment".to_owned(),
                );
            }
        }

        if node.component == "color_picker" {
            let static_alpha_enabled = (!node.property_bindings.contains_key("alpha_enabled"))
                .then(|| {
                    node.properties
                        .get("alpha_enabled")
                        .map(Value::as_bool)
                        .unwrap_or(Some(true))
                })
                .flatten();
            let static_channel =
                (!node.property_bindings.contains_key("active_channel")).then(|| {
                    node.properties
                        .get("active_channel")
                        .and_then(Value::as_str)
                        .unwrap_or("red")
                });
            if static_channel
                .is_some_and(|channel| !matches!(channel, "red" | "green" | "blue" | "alpha"))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.active_channel"),
                    "color_picker active_channel must be red, green, blue or alpha".to_owned(),
                );
            }
            if static_alpha_enabled == Some(false) {
                if (!node.property_bindings.contains_key("value"))
                    .then(|| node.properties.get("value").and_then(Value::as_str))
                    .flatten()
                    .and_then(crate::Color::parse_hex_rgba)
                    .is_some_and(|color| color.a != 255)
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.value"),
                        "color_picker value alpha must be FF when alpha_enabled is false"
                            .to_owned(),
                    );
                }
                if static_channel == Some("alpha") {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.active_channel"),
                        "color_picker active_channel cannot be alpha when alpha is disabled"
                            .to_owned(),
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

        if node.component == "list" {
            let child_ids = node
                .children
                .iter()
                .map(|child| child.id.as_str())
                .collect::<BTreeSet<_>>();
            if let Some(selected) = node.properties.get("selected").and_then(Value::as_str) {
                if !child_ids.contains(selected) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.selected"),
                        "list selected must address a direct child id".to_owned(),
                    );
                }
            }
        }

        if node.component == "progress_ring" {
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
                    "progress_ring maximum must be greater than minimum".to_owned(),
                );
            }
            if let Some(size) = node.properties.get("size").and_then(Value::as_str) {
                if !matches!(size, "small" | "medium" | "large") {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.size"),
                        "progress_ring size must be small, medium or large".to_owned(),
                    );
                }
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
                        "progress_ring value must be null or within minimum and maximum".to_owned(),
                    );
                }
            }
        }

        if node.component == "grid" {
            validate_grid_component(node, path, diagnostics);
        }

        if node.component == "info_bar" {
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name))
                    .then(|| node.properties.get(name).and_then(Value::as_str))
                    .flatten()
            };
            for name in ["message", "title", "action_label"] {
                if static_string(name).is_some_and(|value| value.trim().is_empty()) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("info_bar {name} must not be empty when provided"),
                    );
                }
            }
            if static_string("severity").is_some_and(|severity| {
                !matches!(severity, "informational" | "success" | "warning" | "error")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.severity"),
                    "info_bar severity must be informational, success, warning or error".to_owned(),
                );
            }
        }

        if node.component == "toast" {
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name))
                    .then(|| node.properties.get(name).and_then(Value::as_str))
                    .flatten()
            };
            for name in ["message", "action_label"] {
                if static_string(name).is_some_and(|value| value.trim().is_empty()) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("toast {name} must not be empty when provided"),
                    );
                }
            }
            if static_string("duration")
                .is_some_and(|duration| !matches!(duration, "short" | "long" | "persistent"))
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.duration"),
                    "toast duration must be short, long or persistent".to_owned(),
                );
            }
            if node.property_bindings.contains_key("open")
                && !node.action_bindings.contains_key("open_change")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.open"),
                    "bound toast open state requires an open_change action".to_owned(),
                );
            }
        }

        if node.component == "tooltip" {
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name))
                    .then(|| node.properties.get(name).and_then(Value::as_str))
                    .flatten()
            };
            if static_string("text").is_some_and(|value| value.trim().is_empty()) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.text"),
                    "tooltip text must not be empty".to_owned(),
                );
            }
            if static_string("placement").is_some_and(|placement| {
                !matches!(placement, "auto" | "top" | "bottom" | "left" | "right")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.placement"),
                    "tooltip placement must be auto, top, bottom, left or right".to_owned(),
                );
            }
        }

        if node.component == "teaching_tip" {
            let has_source = |name: &str| {
                node.properties.contains_key(name)
                    || node.property_bindings.contains_key(name)
                    || node.localization.contains_key(name)
            };
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name)
                    && !node.localization.contains_key(name))
                .then(|| node.properties.get(name).and_then(Value::as_str))
                .flatten()
            };
            for name in ["title", "subtitle", "action_label"] {
                if static_string(name).is_some_and(|value| value.trim().is_empty()) {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("teaching_tip {name} must not be empty when provided"),
                    );
                }
            }
            if !has_source("title") && !has_source("subtitle") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties"),
                    "teaching_tip requires a title, subtitle or both".to_owned(),
                );
            }
            if static_string("placement").is_some_and(|placement| {
                !matches!(placement, "auto" | "top" | "bottom" | "left" | "right")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.placement"),
                    "teaching_tip placement must be auto, top, bottom, left or right".to_owned(),
                );
            }
            if node.localization.contains_key("target") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.target"),
                    "teaching_tip target is a stable node id and cannot be localized".to_owned(),
                );
            }
            if let Some(target) = static_string("target") {
                let target_exists = node
                    .children
                    .iter()
                    .any(|child| ui_node_contains_id(child, target));
                if !target_exists {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.target"),
                        "teaching_tip target must reference a node in its page subtree".to_owned(),
                    );
                }
            }
            if node.property_bindings.contains_key("open")
                && !node.action_bindings.contains_key("open_change")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.open"),
                    "bound teaching_tip open state requires an open_change action".to_owned(),
                );
            }
        }

        if node.component == "flyout" {
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name)
                    && !node.localization.contains_key(name))
                .then(|| node.properties.get(name).and_then(Value::as_str))
                .flatten()
            };
            for name in ["content_width", "content_height"] {
                if node
                    .properties
                    .get(name)
                    .and_then(Value::as_f64)
                    .is_some_and(|value| value <= 0.0 || value > f64::from(f32::MAX))
                {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.{name}"),
                        format!("flyout {name} must be a positive finite DP extent"),
                    );
                }
            }
            if static_string("placement").is_some_and(|placement| {
                !matches!(placement, "auto" | "top" | "bottom" | "left" | "right")
            }) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.placement"),
                    "flyout placement must be auto, top, bottom, left or right".to_owned(),
                );
            }
            if node.localization.contains_key("target") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.target"),
                    "flyout target is a stable node id and cannot be localized".to_owned(),
                );
            }
            if let Some(target) = static_string("target") {
                let target_exists = node
                    .children
                    .first()
                    .is_some_and(|page| ui_node_contains_id(page, target));
                if !target_exists {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.target"),
                        "flyout target must reference a node in its page child".to_owned(),
                    );
                }
            }
            if node.property_bindings.contains_key("open")
                && !node.action_bindings.contains_key("open_change")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.open"),
                    "bound flyout open state requires an open_change action".to_owned(),
                );
            }
        }

        if node.component == "menu_flyout" {
            let static_target = (!node.property_bindings.contains_key("target")
                && !node.localization.contains_key("target"))
            .then(|| node.properties.get("target").and_then(Value::as_str))
            .flatten();
            if node.localization.contains_key("target") {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.target"),
                    "menu_flyout target is a stable node id and cannot be localized".to_owned(),
                );
            }
            if let Some(target) = static_target {
                let target_exists = node
                    .children
                    .first()
                    .is_some_and(|page| ui_node_contains_id(page, target));
                if !target_exists {
                    push_diagnostic(
                        diagnostics,
                        UiDiagnosticCode::InvalidPropertyValue,
                        format!("{path}.properties.target"),
                        "menu_flyout target must reference a node in its page child".to_owned(),
                    );
                }
            }
            if node.property_bindings.contains_key("open")
                && !node.action_bindings.contains_key("open_change")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.open"),
                    "bound menu_flyout open state requires an open_change action".to_owned(),
                );
            }
        }

        if node.component == "content_dialog" {
            let static_string = |name: &str| {
                (!node.property_bindings.contains_key(name))
                    .then(|| node.properties.get(name).and_then(Value::as_str))
                    .flatten()
            };
            if static_string("content").is_some_and(|value| value.trim().is_empty()) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.content"),
                    "content_dialog content must not be empty".to_owned(),
                );
            }
            if static_string("close_button").is_some_and(|value| value.trim().is_empty()) {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.close_button"),
                    "content_dialog close_button must not be empty".to_owned(),
                );
            }
            let has_button = |name: &str| {
                node.property_bindings.contains_key(name)
                    || node.localization.contains_key(name)
                    || static_string(name).is_some_and(|label| !label.trim().is_empty())
            };
            let validate_role =
                |name: &str, role: Option<&str>, diagnostics: &mut Vec<UiDiagnostic>| {
                    let Some(role) = role else {
                        return;
                    };
                    if !matches!(role, "primary" | "secondary" | "close") {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.{name}"),
                            format!("content_dialog {name} must be primary, secondary or close"),
                        );
                        return;
                    }
                    let available = match role {
                        "primary" => has_button("primary_button"),
                        "secondary" => has_button("secondary_button"),
                        "close" => has_button("close_button"),
                        _ => false,
                    };
                    if !available {
                        push_diagnostic(
                            diagnostics,
                            UiDiagnosticCode::InvalidPropertyValue,
                            format!("{path}.properties.{name}"),
                            format!(
                                "content_dialog {name} must address an available dialog button"
                            ),
                        );
                    }
                };
            let default_button = static_string("default_button");
            let destructive_button = static_string("destructive_button");
            validate_role("default_button", default_button, diagnostics);
            validate_role("destructive_button", destructive_button, diagnostics);
            if default_button.is_some() && default_button == destructive_button {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.properties.destructive_button"),
                    "content_dialog default_button and destructive_button must differ".to_owned(),
                );
            }
            if node.property_bindings.contains_key("open")
                && !node.action_bindings.contains_key("open_change")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidPropertyValue,
                    format!("{path}.property_bindings.open"),
                    "bound content_dialog open state requires an open_change action".to_owned(),
                );
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
            let property = find_property(schema, property_name);
            if property.is_none() {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::UnknownProperty,
                    format!("{path}.localization.{property_name}"),
                    format!(
                        "property {property_name:?} is not valid on {:?}",
                        node.component
                    ),
                );
            } else if property.is_some_and(|property| property.value_type != UiValueType::String)
                || (node.component == "text" && property_name != "text")
            {
                push_diagnostic(
                    diagnostics,
                    UiDiagnosticCode::InvalidLocalization,
                    format!("{path}.localization.{property_name}"),
                    format!(
                        "property {property_name:?} expects {:?} and cannot be supplied by localization",
                        property.expect("checked property").value_type
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
    PropertySpec {
        name: "wrap",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "ellipsis",
        value_type: UiValueType::Boolean,
        required: false,
    },
    PropertySpec {
        name: "weight",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "horizontal_align",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "vertical_align",
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
    PropertySpec {
        name: "presentation",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "icon",
        value_type: UiValueType::String,
        required: false,
    },
];
const ICON_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "icon",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "size",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "color",
        value_type: UiValueType::String,
        required: false,
    },
];
const BADGE_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "kind",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "value",
        value_type: UiValueType::Integer,
        required: false,
    },
    PropertySpec {
        name: "icon",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "tone",
        value_type: UiValueType::String,
        required: false,
    },
];
const BUTTON_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "click",
    payload_type: UiValueType::Null,
}];
const BREADCRUMB_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "items",
        value_type: UiValueType::BreadcrumbItemArray,
        required: true,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const BREADCRUMB_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "select",
        payload_type: UiValueType::BreadcrumbItemId,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
];
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
const PASSWORD_BOX_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "reveal_mode",
        value_type: UiValueType::String,
        required: false,
    },
];
const PASSWORD_BOX_ACTIONS: &[ActionSpec] = &[ActionSpec {
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
const AUTO_SUGGEST_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "suggestions",
        value_type: UiValueType::AutoSuggestionArray,
        required: true,
    },
    PropertySpec {
        name: "query",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "highlighted",
        value_type: UiValueType::NullableAutoSuggestionId,
        required: false,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
    PropertySpec {
        name: "placeholder",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "no_results_text",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "query_icon",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const AUTO_SUGGEST_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "text_change",
        payload_type: UiValueType::String,
    },
    ActionSpec {
        name: "choose",
        payload_type: UiValueType::AutoSuggestionId,
    },
    ActionSpec {
        name: "submit",
        payload_type: UiValueType::AutoSuggestSubmission,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
];
const COMMAND_PALETTE_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "items",
        value_type: UiValueType::CommandPaletteItemArray,
        required: true,
    },
    PropertySpec {
        name: "query",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "highlighted",
        value_type: UiValueType::NullableCommandPaletteItemId,
        required: true,
    },
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "placeholder",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "no_results_text",
        value_type: UiValueType::String,
        required: false,
    },
];
const COMMAND_PALETTE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "query_change",
        payload_type: UiValueType::String,
    },
    ActionSpec {
        name: "highlight_change",
        payload_type: UiValueType::CommandPaletteItemId,
    },
    ActionSpec {
        name: "invoke",
        payload_type: UiValueType::CommandPaletteItemId,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];
const TREE_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "nodes",
        value_type: UiValueType::TreeNodeArray,
        required: true,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::TreeNodeIdArray,
        required: true,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::NullableTreeNodeId,
        required: true,
    },
];
const TREE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "select",
        payload_type: UiValueType::TreeNodeId,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::TreeNodeIdArray,
    },
    ActionSpec {
        name: "invoke",
        payload_type: UiValueType::TreeNodeId,
    },
];
const GRID_VIEW_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "items",
        value_type: UiValueType::GridViewItemArray,
        required: true,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::NullableGridViewItemId,
        required: true,
    },
];
const GRID_VIEW_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "select",
        payload_type: UiValueType::GridViewItemId,
    },
    ActionSpec {
        name: "invoke",
        payload_type: UiValueType::GridViewItemId,
    },
];
const TABLE_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "columns",
        value_type: UiValueType::TableColumnArray,
        required: true,
    },
    PropertySpec {
        name: "rows",
        value_type: UiValueType::TableRowArray,
        required: true,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::NullableTableRowId,
        required: true,
    },
    PropertySpec {
        name: "sort",
        value_type: UiValueType::NullableTableSort,
        required: true,
    },
];
const TABLE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "select",
        payload_type: UiValueType::TableRowId,
    },
    ActionSpec {
        name: "sort",
        payload_type: UiValueType::TableSort,
    },
    ActionSpec {
        name: "invoke",
        payload_type: UiValueType::TableRowId,
    },
];
const DATE_PICKER_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::Date,
        required: true,
    },
    PropertySpec {
        name: "minimum",
        value_type: UiValueType::Date,
        required: false,
    },
    PropertySpec {
        name: "maximum",
        value_type: UiValueType::Date,
        required: false,
    },
    PropertySpec {
        name: "visible_month",
        value_type: UiValueType::Date,
        required: false,
    },
    PropertySpec {
        name: "today",
        value_type: UiValueType::Date,
        required: false,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const DATE_PICKER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "change",
        payload_type: UiValueType::Date,
    },
    ActionSpec {
        name: "month_change",
        payload_type: UiValueType::Date,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
];
const TIME_PICKER_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::Time,
        required: true,
    },
    PropertySpec {
        name: "minute_increment",
        value_type: UiValueType::Integer,
        required: false,
    },
    PropertySpec {
        name: "clock_format",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const TIME_PICKER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "change",
        payload_type: UiValueType::Time,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
];
const COLOR_PICKER_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::Color,
        required: true,
    },
    PropertySpec {
        name: "expanded",
        value_type: UiValueType::Boolean,
        required: false,
    },
    PropertySpec {
        name: "active_channel",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "alpha_enabled",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const COLOR_PICKER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "change",
        payload_type: UiValueType::Color,
    },
    ActionSpec {
        name: "expanded_change",
        payload_type: UiValueType::Boolean,
    },
    ActionSpec {
        name: "channel_change",
        payload_type: UiValueType::String,
    },
];
const NAVIGATION_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "title",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "subtitle",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "items",
        value_type: UiValueType::NavigationItemArray,
        required: true,
    },
    PropertySpec {
        name: "footer_items",
        value_type: UiValueType::NavigationItemArray,
        required: false,
    },
    PropertySpec {
        name: "selected",
        value_type: UiValueType::NullableNavigationItemId,
        required: true,
    },
    PropertySpec {
        name: "pane_width",
        value_type: UiValueType::Number,
        required: false,
    },
    PropertySpec {
        name: "minimum_content_width",
        value_type: UiValueType::Number,
        required: false,
    },
];
const NAVIGATION_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "select",
    payload_type: UiValueType::NavigationItemId,
}];
const COMMAND_BAR_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "trailing",
        value_type: UiValueType::StringArray,
        required: false,
    },
    PropertySpec {
        name: "gap",
        value_type: UiValueType::Number,
        required: false,
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
const LIST_PROPERTIES: &[PropertySpec] = &[PropertySpec {
    name: "selected",
    value_type: UiValueType::String,
    required: false,
}];
const LIST_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "select",
    payload_type: UiValueType::String,
}];
const PROGRESS_RING_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "value",
        value_type: UiValueType::NullableNumber,
        required: false,
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
        name: "active",
        value_type: UiValueType::Boolean,
        required: false,
    },
    PropertySpec {
        name: "size",
        value_type: UiValueType::String,
        required: false,
    },
];
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
const FLYOUT_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "target",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "content_width",
        value_type: UiValueType::Number,
        required: true,
    },
    PropertySpec {
        name: "content_height",
        value_type: UiValueType::Number,
        required: true,
    },
    PropertySpec {
        name: "placement",
        value_type: UiValueType::String,
        required: false,
    },
];
const FLYOUT_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "dismiss",
        payload_type: UiValueType::FlyoutDismissReason,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];
const MENU_FLYOUT_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "target",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "items",
        value_type: UiValueType::MenuFlyoutItemArray,
        required: true,
    },
];
const MENU_FLYOUT_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "invoke",
        payload_type: UiValueType::MenuFlyoutItemId,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];
const INFO_BAR_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "message",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "title",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "severity",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "action_label",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "closable",
        value_type: UiValueType::Boolean,
        required: false,
    },
];
const INFO_BAR_ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "event",
    payload_type: UiValueType::String,
}];
const TOAST_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "message",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "action_label",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "duration",
        value_type: UiValueType::String,
        required: false,
    },
];
const TOAST_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "result",
        payload_type: UiValueType::String,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];
const TOOLTIP_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "text",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "placement",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "open_delay_ms",
        value_type: UiValueType::Integer,
        required: false,
    },
];
const TEACHING_TIP_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "target",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "title",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "subtitle",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "action_label",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "placement",
        value_type: UiValueType::String,
        required: false,
    },
];
const TEACHING_TIP_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "result",
        payload_type: UiValueType::String,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];
const CONTENT_DIALOG_PROPERTIES: &[PropertySpec] = &[
    PropertySpec {
        name: "open",
        value_type: UiValueType::Boolean,
        required: true,
    },
    PropertySpec {
        name: "title",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "content",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "primary_button",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "secondary_button",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "close_button",
        value_type: UiValueType::String,
        required: true,
    },
    PropertySpec {
        name: "default_button",
        value_type: UiValueType::String,
        required: false,
    },
    PropertySpec {
        name: "destructive_button",
        value_type: UiValueType::String,
        required: false,
    },
];
const CONTENT_DIALOG_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "result",
        payload_type: UiValueType::String,
    },
    ActionSpec {
        name: "open_change",
        payload_type: UiValueType::Boolean,
    },
];

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
        "navigation" => Some(ComponentSchema {
            properties: NAVIGATION_PROPERTIES,
            actions: NAVIGATION_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "command_bar" => Some(ComponentSchema {
            properties: COMMAND_BAR_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "tabs" => Some(ComponentSchema {
            properties: TABS_PROPERTIES,
            actions: TABS_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "list" => Some(ComponentSchema {
            properties: LIST_PROPERTIES,
            actions: LIST_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "grid" => Some(ComponentSchema {
            properties: GRID_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::AtLeast(1),
        }),
        "content_dialog" => Some(ComponentSchema {
            properties: CONTENT_DIALOG_PROPERTIES,
            actions: CONTENT_DIALOG_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "flyout" => Some(ComponentSchema {
            properties: FLYOUT_PROPERTIES,
            actions: FLYOUT_ACTIONS,
            children: ChildPolicy::Exactly(2),
        }),
        "menu_flyout" => Some(ComponentSchema {
            properties: MENU_FLYOUT_PROPERTIES,
            actions: MENU_FLYOUT_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "info_bar" => Some(leaf(INFO_BAR_PROPERTIES, INFO_BAR_ACTIONS)),
        "toast" => Some(ComponentSchema {
            properties: TOAST_PROPERTIES,
            actions: TOAST_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "tooltip" => Some(ComponentSchema {
            properties: TOOLTIP_PROPERTIES,
            actions: NO_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "teaching_tip" => Some(ComponentSchema {
            properties: TEACHING_TIP_PROPERTIES,
            actions: TEACHING_TIP_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "text" => Some(leaf(TEXT_PROPERTIES, NO_ACTIONS)),
        "badge" => Some(leaf(BADGE_PROPERTIES, NO_ACTIONS)),
        "icon" => Some(leaf(ICON_PROPERTIES, NO_ACTIONS)),
        "button" => Some(leaf(BUTTON_PROPERTIES, BUTTON_ACTIONS)),
        "breadcrumb" => Some(leaf(BREADCRUMB_PROPERTIES, BREADCRUMB_ACTIONS)),
        "toggle_button" | "checkbox" | "toggle" => Some(leaf(CHECKED_PROPERTIES, TOGGLE_ACTIONS)),
        "textbox" => Some(leaf(TEXTBOX_PROPERTIES, TEXTBOX_ACTIONS)),
        "password_box" => Some(leaf(PASSWORD_BOX_PROPERTIES, PASSWORD_BOX_ACTIONS)),
        "radio_button" => Some(leaf(RADIO_PROPERTIES, RADIO_ACTIONS)),
        "slider" => Some(leaf(VALUE_PROPERTIES, SLIDER_ACTIONS)),
        "number_box" => Some(leaf(NUMBER_BOX_PROPERTIES, NUMBER_BOX_ACTIONS)),
        "combo_box" => Some(leaf(COMBO_BOX_PROPERTIES, COMBO_BOX_ACTIONS)),
        "auto_suggest" => Some(leaf(AUTO_SUGGEST_PROPERTIES, AUTO_SUGGEST_ACTIONS)),
        "command_palette" => Some(ComponentSchema {
            properties: COMMAND_PALETTE_PROPERTIES,
            actions: COMMAND_PALETTE_ACTIONS,
            children: ChildPolicy::Exactly(1),
        }),
        "tree" => Some(leaf(TREE_PROPERTIES, TREE_ACTIONS)),
        "grid_view" => Some(leaf(GRID_VIEW_PROPERTIES, GRID_VIEW_ACTIONS)),
        "table" => Some(leaf(TABLE_PROPERTIES, TABLE_ACTIONS)),
        "date_picker" => Some(leaf(DATE_PICKER_PROPERTIES, DATE_PICKER_ACTIONS)),
        "time_picker" => Some(leaf(TIME_PICKER_PROPERTIES, TIME_PICKER_ACTIONS)),
        "color_picker" => Some(leaf(COLOR_PICKER_PROPERTIES, COLOR_PICKER_ACTIONS)),
        "progress_bar" => Some(leaf(VALUE_PROPERTIES, NO_ACTIONS)),
        "progress_ring" => Some(leaf(PROGRESS_RING_PROPERTIES, NO_ACTIONS)),
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

fn ui_node_contains_id(node: &UiNode, id: &str) -> bool {
    node.id.as_str() == id
        || node
            .children
            .iter()
            .any(|child| ui_node_contains_id(child, id))
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
    for (numeric_name, numeric, token_name, token) in [
        (
            "padding",
            layout.padding,
            "padding_token",
            layout.padding_token,
        ),
        ("gap", layout.gap, "gap_token", layout.gap_token),
    ] {
        if numeric.is_some() && token.is_some() {
            push_diagnostic(
                diagnostics,
                UiDiagnosticCode::InvalidLayout,
                format!("{path}.layout.{token_name}"),
                format!("layout must use either {numeric_name:?} or {token_name:?}, not both"),
            );
        }
    }
    if layout
        .flex
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        push_diagnostic(
            diagnostics,
            UiDiagnosticCode::InvalidLayout,
            format!("{path}.layout.flex"),
            "layout flex must be finite and non-negative".to_owned(),
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
    fn layout_spacing_tokens_round_trip_and_conflict_with_numeric_overrides() {
        let document = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "root",
                "component": "stack",
                "layout": {
                  "padding_token": "page_padding",
                  "gap_token": "content_gap",
                  "direction": "vertical"
                }
              }
            }"#,
        )
        .unwrap();
        assert_eq!(
            document.root.layout.padding_token,
            Some(UiSpacingToken::PagePadding)
        );
        assert_eq!(
            document.root.layout.gap_token,
            Some(UiSpacingToken::ContentGap)
        );
        let round_trip =
            UiDocument::from_json(&serde_json::to_string_pretty(&document).unwrap()).unwrap();
        assert_eq!(round_trip, document);
        assert!(document
            .validate(&UiFeatureSet::default(), &UiBindingSchema::default())
            .is_valid());

        let conflicting = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "root",
                "component": "stack",
                "layout": {
                  "padding": 12.0,
                  "padding_token": "page_padding",
                  "gap": 8.0,
                  "gap_token": "content_gap"
                }
              }
            }"#,
        )
        .unwrap();
        let report = conflicting.validate(&UiFeatureSet::default(), &UiBindingSchema::default());
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidLayout)
                .count(),
            2
        );
    }

    #[test]
    fn text_layout_contract_validates_wrapping_alignment_and_semantic_style() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "description",
                "component": "text",
                "properties": {
                  "text": "中文长说明 / A long bilingual description",
                  "text_role": "body",
                  "wrap": "word",
                  "ellipsis": false,
                  "weight": "regular",
                  "horizontal_align": "start",
                  "vertical_align": "start"
                }
              }
            }"#,
        )
        .unwrap();
        let features = UiFeatureSet::new(["label"]);
        assert!(valid
            .validate(&features, &UiBindingSchema::default())
            .is_valid());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "description",
                "component": "text",
                "properties": {
                  "text": "Description",
                  "text_role": "fluent_title",
                  "wrap": "compress",
                  "weight": "heavy",
                  "horizontal_align": "left",
                  "vertical_align": "baseline"
                },
                "localization": { "wrap": "layout.wrap" }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            5
        );
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidLocalization
                && diagnostic.path == "$.root.localization.wrap"
        }));
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn typed_date_bindings_use_canonical_document_values() {
        struct DateState {
            selected: crate::ZsDate,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum DateMsg {
            Selected(crate::ZsDate),
        }

        let date = crate::ZsDate::new(2026, 7, 22).unwrap();
        let mut manifest = UiBindingManifest::<DateState, DateMsg>::new();
        manifest
            .register_date_property("selected_date", |state| state.selected)
            .unwrap();
        manifest
            .register_date_action("date_changed", |date| Ok(DateMsg::Selected(date)))
            .unwrap();

        assert_eq!(
            manifest.schema().properties["selected_date"],
            UiValueType::Date
        );
        assert_eq!(manifest.schema().actions["date_changed"], UiValueType::Date);
        assert_eq!(
            manifest.read_property("selected_date", &DateState { selected: date }),
            Some(Value::String("2026-07-22".to_owned()))
        );
        assert_eq!(
            manifest.map_action("date_changed", Value::String("2026-07-22".to_owned())),
            Ok(DateMsg::Selected(date))
        );
        assert!(matches!(
            manifest.map_action("date_changed", Value::String("2026-7-22".to_owned())),
            Err(UiBindingDispatchError::PayloadType { .. })
        ));
    }

    #[cfg(feature = "time-picker")]
    #[test]
    fn typed_time_bindings_use_canonical_document_values() {
        struct TimeState {
            selected: crate::ZsTime,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum TimeMsg {
            Selected(crate::ZsTime),
        }

        let time = crate::ZsTime::new(18, 5).unwrap();
        let mut manifest = UiBindingManifest::<TimeState, TimeMsg>::new();
        manifest
            .register_time_property("selected_time", |state| state.selected)
            .unwrap();
        manifest
            .register_time_action("time_changed", |time| Ok(TimeMsg::Selected(time)))
            .unwrap();

        assert_eq!(
            manifest.schema().properties["selected_time"],
            UiValueType::Time
        );
        assert_eq!(manifest.schema().actions["time_changed"], UiValueType::Time);
        assert_eq!(
            manifest.read_property("selected_time", &TimeState { selected: time }),
            Some(Value::String("18:05".to_owned()))
        );
        assert_eq!(
            manifest.map_action("time_changed", Value::String("18:05".to_owned())),
            Ok(TimeMsg::Selected(time))
        );
        assert!(matches!(
            manifest.map_action("time_changed", Value::String("8:05".to_owned())),
            Err(UiBindingDispatchError::PayloadType { .. })
        ));
    }

    #[cfg(feature = "color-picker")]
    #[test]
    fn typed_color_bindings_use_canonical_document_values() {
        struct ColorState {
            selected: crate::Color,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum ColorMsg {
            Selected(crate::Color),
        }

        let color = crate::Color::rgba(32, 96, 160, 224);
        let mut manifest = UiBindingManifest::<ColorState, ColorMsg>::new();
        manifest
            .register_color_property("selected_color", |state| state.selected)
            .unwrap();
        manifest
            .register_color_action("color_changed", |color| Ok(ColorMsg::Selected(color)))
            .unwrap();

        assert_eq!(
            manifest.schema().properties["selected_color"],
            UiValueType::Color
        );
        assert_eq!(
            manifest.schema().actions["color_changed"],
            UiValueType::Color
        );
        assert_eq!(
            manifest.read_property("selected_color", &ColorState { selected: color }),
            Some(Value::String("#2060A0E0".to_owned()))
        );
        assert_eq!(
            manifest.map_action("color_changed", Value::String("#2060A0E0".to_owned())),
            Ok(ColorMsg::Selected(color))
        );
        assert!(matches!(
            manifest.map_action("color_changed", Value::String("#2060a0e0".to_owned())),
            Err(UiBindingDispatchError::PayloadType { .. })
        ));
    }

    #[cfg(feature = "auto-suggest")]
    #[test]
    fn typed_auto_suggest_bindings_preserve_semantic_ids() {
        struct SuggestState {
            suggestions: Vec<UiAutoSuggestion>,
            highlighted: Option<UiAutoSuggestionId>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum SuggestMsg {
            Chosen(UiAutoSuggestionId),
            Submitted(UiAutoSuggestSubmission),
        }

        let china = UiAutoSuggestionId::new("china").unwrap();
        let chile = UiAutoSuggestionId::new("chile").unwrap();
        let mut manifest = UiBindingManifest::<SuggestState, SuggestMsg>::new();
        manifest
            .register_auto_suggestions_property("search_suggestions", |state| {
                state.suggestions.clone()
            })
            .unwrap();
        manifest
            .register_auto_suggestion_id_property("search_highlighted", |state| {
                state.highlighted.clone()
            })
            .unwrap();
        manifest
            .register_auto_suggestion_id_action("search_chosen", |id| Ok(SuggestMsg::Chosen(id)))
            .unwrap();
        manifest
            .register_auto_suggest_submission_action("search_submitted", |submission| {
                Ok(SuggestMsg::Submitted(submission))
            })
            .unwrap();

        assert_eq!(
            manifest.schema().properties["search_suggestions"],
            UiValueType::AutoSuggestionArray
        );
        assert_eq!(
            manifest.schema().properties["search_highlighted"],
            UiValueType::NullableAutoSuggestionId
        );
        assert_eq!(
            manifest.schema().actions["search_chosen"],
            UiValueType::AutoSuggestionId
        );
        assert_eq!(
            manifest.schema().actions["search_submitted"],
            UiValueType::AutoSuggestSubmission
        );
        assert_eq!(
            manifest.read_property(
                "search_suggestions",
                &SuggestState {
                    suggestions: vec![
                        UiAutoSuggestion::new(china.clone(), "China"),
                        UiAutoSuggestion::new(chile.clone(), "Chile"),
                    ],
                    highlighted: Some(china.clone()),
                },
            ),
            Some(serde_json::json!([
                { "id": "china", "text": "China" },
                { "id": "chile", "text": "Chile" }
            ]))
        );
        assert_eq!(
            manifest.map_action("search_chosen", Value::String("china".to_owned())),
            Ok(SuggestMsg::Chosen(china.clone()))
        );
        assert_eq!(
            manifest.map_action(
                "search_submitted",
                serde_json::json!({ "query": "China", "chosen": "china" }),
            ),
            Ok(SuggestMsg::Submitted(UiAutoSuggestSubmission::new(
                "China",
                Some(china)
            )))
        );
        assert!(
            !UiValueType::AutoSuggestionArray.matches(&serde_json::json!([
                { "id": "duplicate", "text": "First" },
                { "id": "duplicate", "text": "Second" }
            ]))
        );
        assert!(
            !UiValueType::AutoSuggestSubmission.matches(&serde_json::json!({ "query": "China" }))
        );
    }

    #[cfg(feature = "flyout")]
    #[test]
    fn typed_flyout_binding_maps_only_semantic_dismiss_reasons() {
        #[derive(Debug, PartialEq, Eq)]
        enum Msg {
            Dismissed(crate::ZsFlyoutDismissReason),
        }

        let mut manifest = UiBindingManifest::<(), Msg>::new();
        manifest
            .register_flyout_dismiss_action("details_dismissed", |reason| {
                Ok(Msg::Dismissed(reason))
            })
            .unwrap();

        assert_eq!(
            manifest.schema().actions["details_dismissed"],
            UiValueType::FlyoutDismissReason
        );
        assert_eq!(
            manifest.map_action(
                "details_dismissed",
                Value::String("light_dismiss".to_owned())
            ),
            Ok(Msg::Dismissed(crate::ZsFlyoutDismissReason::LightDismiss))
        );
        assert_eq!(
            manifest.map_action("details_dismissed", Value::String("escape".to_owned())),
            Ok(Msg::Dismissed(crate::ZsFlyoutDismissReason::EscapeKey))
        );
        assert!(!UiValueType::FlyoutDismissReason.matches(&Value::String("resize".to_owned())));
    }

    #[cfg(feature = "menu-flyout")]
    #[test]
    fn typed_menu_flyout_bindings_preserve_nested_ids_and_portable_accelerators() {
        struct MenuState {
            items: Vec<UiMenuFlyoutItem>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum Msg {
            Invoked(UiMenuFlyoutItemId),
        }

        let save = UiMenuFlyoutItemId::new("save").unwrap();
        let more = UiMenuFlyoutItemId::new("more").unwrap();
        let export = UiMenuFlyoutItemId::new("export-pdf").unwrap();
        let mut manifest = UiBindingManifest::<MenuState, Msg>::new();
        manifest
            .register_menu_flyout_items_property("file_items", |state| state.items.clone())
            .unwrap();
        manifest
            .register_menu_flyout_item_id_action("file_invoked", |id| Ok(Msg::Invoked(id)))
            .unwrap();

        let state = MenuState {
            items: vec![
                UiMenuFlyoutItem::command(save, "Save")
                    .accelerator(UiMenuFlyoutAccelerator::new("s").unwrap().primary()),
                UiMenuFlyoutItem::separator(),
                UiMenuFlyoutItem::submenu(
                    more,
                    "More",
                    [UiMenuFlyoutItem::command(export.clone(), "Export PDF")],
                ),
            ],
        };
        let value = manifest.read_property("file_items", &state).unwrap();
        assert!(UiValueType::MenuFlyoutItemArray.matches(&value));
        assert_eq!(value[0]["accelerator"]["key"], "S");
        assert_eq!(value[0]["accelerator"]["primary"], true);
        assert_eq!(
            manifest.map_action("file_invoked", Value::String("export-pdf".to_owned())),
            Ok(Msg::Invoked(export))
        );
        assert_eq!(
            manifest.schema().properties["file_items"],
            UiValueType::MenuFlyoutItemArray
        );
        assert_eq!(
            manifest.schema().actions["file_invoked"],
            UiValueType::MenuFlyoutItemId
        );

        assert!(
            !UiValueType::MenuFlyoutItemArray.matches(&serde_json::json!([
                { "kind": "command", "id": "duplicate", "label": "First" },
                {
                  "kind": "submenu",
                  "id": "more",
                  "label": "More",
                  "items": [
                    { "kind": "command", "id": "duplicate", "label": "Second" }
                  ]
                }
            ]))
        );
        assert!(
            !UiValueType::MenuFlyoutItemArray.matches(&serde_json::json!([
                {
                  "kind": "command",
                  "id": "save",
                  "label": "Save",
                  "accelerator": { "key": "s", "primary": true }
                }
            ]))
        );
        assert!(
            !UiValueType::MenuFlyoutItemArray.matches(&serde_json::json!([
                { "kind": "separator" },
                { "kind": "command", "id": "save", "label": "Save" }
            ]))
        );
    }

    #[cfg(feature = "breadcrumb")]
    #[test]
    fn typed_breadcrumb_bindings_preserve_path_ids_and_labels() {
        struct BreadcrumbState {
            items: Vec<UiBreadcrumbItem>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum BreadcrumbMsg {
            Selected(UiBreadcrumbItemId),
        }

        let home = UiBreadcrumbItemId::new("home").unwrap();
        let settings = UiBreadcrumbItemId::new("settings").unwrap();
        let mut manifest = UiBindingManifest::<BreadcrumbState, BreadcrumbMsg>::new();
        manifest
            .register_breadcrumb_items_property("navigation_path", |state| state.items.clone())
            .unwrap();
        manifest
            .register_breadcrumb_item_id_action("navigation_selected", |id| {
                Ok(BreadcrumbMsg::Selected(id))
            })
            .unwrap();

        assert_eq!(
            manifest.schema().properties["navigation_path"],
            UiValueType::BreadcrumbItemArray
        );
        assert_eq!(
            manifest.schema().actions["navigation_selected"],
            UiValueType::BreadcrumbItemId
        );
        assert_eq!(
            manifest.read_property(
                "navigation_path",
                &BreadcrumbState {
                    items: vec![
                        UiBreadcrumbItem::new(home, "Home"),
                        UiBreadcrumbItem::new(settings.clone(), "Settings"),
                    ],
                },
            ),
            Some(serde_json::json!([
                { "id": "home", "label": "Home" },
                { "id": "settings", "label": "Settings" }
            ]))
        );
        assert_eq!(
            manifest.map_action("navigation_selected", Value::String("settings".to_owned())),
            Ok(BreadcrumbMsg::Selected(settings))
        );
        assert!(!UiValueType::BreadcrumbItemArray.matches(&serde_json::json!([])));
        assert!(
            !UiValueType::BreadcrumbItemArray.matches(&serde_json::json!([
                { "id": "same", "label": "First" },
                { "id": "same", "label": "Second" }
            ]))
        );
        assert!(
            !UiValueType::BreadcrumbItemArray.matches(&serde_json::json!([
                { "id": "blank", "label": "  " }
            ]))
        );
    }

    #[cfg(feature = "command-palette")]
    #[test]
    fn typed_command_palette_bindings_preserve_semantic_ids_and_metadata() {
        struct CommandState {
            items: Vec<UiCommandPaletteItem>,
            highlighted: Option<UiCommandPaletteItemId>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum CommandMsg {
            Selected(UiCommandPaletteItemId),
        }

        let settings = UiCommandPaletteItemId::new("open-settings").unwrap();
        let file = UiCommandPaletteItemId::new("open-file").unwrap();
        let mut manifest = UiBindingManifest::<CommandState, CommandMsg>::new();
        manifest
            .register_command_palette_items_property("commands", |state| state.items.clone())
            .unwrap();
        manifest
            .register_command_palette_item_id_property("command_highlighted", |state| {
                state.highlighted.clone()
            })
            .unwrap();
        manifest
            .register_command_palette_item_id_action("command_invoked", |id| {
                Ok(CommandMsg::Selected(id))
            })
            .unwrap();

        assert_eq!(
            manifest.schema().properties["commands"],
            UiValueType::CommandPaletteItemArray
        );
        assert_eq!(
            manifest.schema().properties["command_highlighted"],
            UiValueType::NullableCommandPaletteItemId
        );
        assert_eq!(
            manifest.schema().actions["command_invoked"],
            UiValueType::CommandPaletteItemId
        );
        assert_eq!(
            manifest.read_property(
                "commands",
                &CommandState {
                    items: vec![
                        UiCommandPaletteItem::new(settings.clone(), "Open settings")
                            .keywords(["preferences"])
                            .shortcut("Ctrl+,")
                            .icon(crate::ZsIcon::Settings),
                        UiCommandPaletteItem::new(file.clone(), "Open file")
                            .subtitle("Choose from disk"),
                    ],
                    highlighted: Some(file.clone()),
                },
            ),
            Some(serde_json::json!([
                {
                  "id": "open-settings",
                  "title": "Open settings",
                  "keywords": ["preferences"],
                  "shortcut": "Ctrl+,",
                  "icon": "Settings",
                  "enabled": true
                },
                {
                  "id": "open-file",
                  "title": "Open file",
                  "subtitle": "Choose from disk",
                  "enabled": true
                }
            ]))
        );
        assert_eq!(
            manifest.map_action("command_invoked", Value::String("open-file".to_owned())),
            Ok(CommandMsg::Selected(file))
        );
        assert!(
            !UiValueType::CommandPaletteItemArray.matches(&serde_json::json!([
                { "id": "duplicate", "title": "First" },
                { "id": "duplicate", "title": "Second" }
            ]))
        );
        assert!(
            !UiValueType::CommandPaletteItemArray.matches(&serde_json::json!([
                { "id": "empty-title", "title": "  " }
            ]))
        );
    }

    #[cfg(feature = "tree")]
    #[test]
    fn typed_tree_bindings_preserve_hierarchy_and_semantic_ids() {
        struct TreeState {
            nodes: Vec<UiTreeNode>,
            expanded: BTreeSet<UiTreeNodeId>,
            selected: Option<UiTreeNodeId>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum TreeMsg {
            Selected(UiTreeNodeId),
            Expanded(BTreeSet<UiTreeNodeId>),
        }

        let workspace = UiTreeNodeId::new("workspace").unwrap();
        let source = UiTreeNodeId::new("source").unwrap();
        let mut manifest = UiBindingManifest::<TreeState, TreeMsg>::new();
        manifest
            .register_tree_nodes_property("project_nodes", |state| state.nodes.clone())
            .unwrap();
        manifest
            .register_tree_node_ids_property("project_expanded", |state| state.expanded.clone())
            .unwrap();
        manifest
            .register_tree_node_id_property("project_selected", |state| state.selected.clone())
            .unwrap();
        manifest
            .register_tree_node_id_action("project_selected_changed", |id| {
                Ok(TreeMsg::Selected(id))
            })
            .unwrap();
        manifest
            .register_tree_node_ids_action("project_expanded_changed", |ids| {
                Ok(TreeMsg::Expanded(ids))
            })
            .unwrap();

        let state = TreeState {
            nodes: vec![UiTreeNode::new(workspace.clone(), "Workspace")
                .icon(crate::ZsIcon::Folder)
                .children([UiTreeNode::new(source.clone(), "src").unrealized_children(true)])],
            expanded: BTreeSet::from([workspace.clone()]),
            selected: Some(source.clone()),
        };
        assert_eq!(
            manifest.schema().properties["project_nodes"],
            UiValueType::TreeNodeArray
        );
        assert_eq!(
            manifest.schema().properties["project_expanded"],
            UiValueType::TreeNodeIdArray
        );
        assert_eq!(
            manifest.schema().properties["project_selected"],
            UiValueType::NullableTreeNodeId
        );
        assert_eq!(
            manifest.read_property("project_nodes", &state),
            Some(serde_json::json!([
                {
                    "id": "workspace",
                    "label": "Workspace",
                    "icon": "Folder",
                    "children": [
                        {
                            "id": "source",
                            "label": "src",
                            "has_unrealized_children": true
                        }
                    ]
                }
            ]))
        );
        assert_eq!(
            manifest.map_action(
                "project_selected_changed",
                Value::String("source".to_owned())
            ),
            Ok(TreeMsg::Selected(source))
        );
        assert_eq!(
            manifest.map_action(
                "project_expanded_changed",
                serde_json::json!(["source", "workspace"])
            ),
            Ok(TreeMsg::Expanded(BTreeSet::from([
                UiTreeNodeId::new("source").unwrap(),
                workspace
            ])))
        );
        assert!(!UiValueType::TreeNodeArray.matches(&serde_json::json!([
            {
                "id": "root",
                "label": "Root",
                "children": [{ "id": "root", "label": "Duplicate" }]
            }
        ])));
        assert!(
            !UiValueType::TreeNodeIdArray.matches(&serde_json::json!(["workspace", "workspace"]))
        );
    }

    #[cfg(feature = "shell")]
    #[test]
    fn typed_navigation_bindings_preserve_semantic_item_ids() {
        struct NavigationState {
            items: Vec<UiNavigationItem>,
            selected: Option<UiNavigationItemId>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum NavigationMsg {
            Selected(UiNavigationItemId),
        }

        let home = UiNavigationItemId::new("home").unwrap();
        let settings = UiNavigationItemId::new("settings").unwrap();
        let mut manifest = UiBindingManifest::<NavigationState, NavigationMsg>::new();
        manifest
            .register_navigation_items_property("navigation_items", |state| state.items.clone())
            .unwrap();
        manifest
            .register_navigation_item_id_property("navigation_selected", |state| {
                state.selected.clone()
            })
            .unwrap();
        manifest
            .register_navigation_item_id_action("navigation_selected_changed", |id| {
                Ok(NavigationMsg::Selected(id))
            })
            .unwrap();

        let state = NavigationState {
            items: vec![
                UiNavigationItem::new(home, "Home", crate::ZsIcon::App),
                UiNavigationItem::new(settings.clone(), "Settings", crate::ZsIcon::Settings)
                    .enabled(false),
            ],
            selected: Some(settings.clone()),
        };
        assert_eq!(
            manifest.schema().properties["navigation_items"],
            UiValueType::NavigationItemArray
        );
        assert_eq!(
            manifest.schema().properties["navigation_selected"],
            UiValueType::NullableNavigationItemId
        );
        assert_eq!(
            manifest.schema().actions["navigation_selected_changed"],
            UiValueType::NavigationItemId
        );
        assert_eq!(
            manifest.read_property("navigation_items", &state),
            Some(serde_json::json!([
                { "id": "home", "label": "Home", "icon": "App", "enabled": true },
                { "id": "settings", "label": "Settings", "icon": "Settings", "enabled": false }
            ]))
        );
        assert_eq!(
            manifest.map_action(
                "navigation_selected_changed",
                Value::String("settings".to_owned())
            ),
            Ok(NavigationMsg::Selected(settings))
        );
        assert!(
            !UiValueType::NavigationItemArray.matches(&serde_json::json!([
                { "id": "same", "label": "One", "icon": "App" },
                { "id": "same", "label": "Two", "icon": "Settings" }
            ]))
        );
    }

    #[cfg(feature = "grid-view")]
    #[test]
    fn typed_grid_view_bindings_preserve_metadata_and_semantic_ids() {
        struct GridViewState {
            items: Vec<UiGridViewItem>,
            selected: Option<UiGridViewItemId>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum GridViewMsg {
            Selected(UiGridViewItemId),
        }

        let documents = UiGridViewItemId::new("documents").unwrap();
        let photos = UiGridViewItemId::new("photos").unwrap();
        let mut manifest = UiBindingManifest::<GridViewState, GridViewMsg>::new();
        manifest
            .register_grid_view_items_property("library_items", |state| state.items.clone())
            .unwrap();
        manifest
            .register_grid_view_item_id_property("library_selected", |state| state.selected.clone())
            .unwrap();
        manifest
            .register_grid_view_item_id_action("library_selected_changed", |id| {
                Ok(GridViewMsg::Selected(id))
            })
            .unwrap();

        let state = GridViewState {
            items: vec![
                UiGridViewItem::new(documents, "Documents")
                    .subtitle("12 folders")
                    .icon(crate::ZsIcon::Folder),
                UiGridViewItem::new(photos.clone(), "Photos").icon(crate::ZsIcon::Image),
            ],
            selected: Some(photos.clone()),
        };
        assert_eq!(
            manifest.schema().properties["library_items"],
            UiValueType::GridViewItemArray
        );
        assert_eq!(
            manifest.schema().properties["library_selected"],
            UiValueType::NullableGridViewItemId
        );
        assert_eq!(
            manifest.schema().actions["library_selected_changed"],
            UiValueType::GridViewItemId
        );
        assert_eq!(
            manifest.read_property("library_items", &state),
            Some(serde_json::json!([
                {
                    "id": "documents",
                    "title": "Documents",
                    "subtitle": "12 folders",
                    "icon": "Folder"
                },
                {
                    "id": "photos",
                    "title": "Photos",
                    "icon": "Image"
                }
            ]))
        );
        assert_eq!(
            manifest.map_action(
                "library_selected_changed",
                Value::String("photos".to_owned())
            ),
            Ok(GridViewMsg::Selected(photos))
        );
        assert!(!UiValueType::GridViewItemArray.matches(&serde_json::json!([
            { "id": "duplicate", "title": "First" },
            { "id": "duplicate", "title": "Second" }
        ])));
        assert!(!UiValueType::GridViewItemArray.matches(&serde_json::json!([
            { "id": "empty-title", "title": "  " }
        ])));
    }

    #[cfg(feature = "table")]
    #[test]
    fn typed_table_bindings_preserve_column_row_and_sort_identity() {
        struct TableState {
            columns: Vec<UiTableColumn>,
            rows: Vec<UiTableRow>,
            selected: Option<UiTableRowId>,
            sort: Option<UiTableSort>,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum TableMsg {
            Row(UiTableRowId),
            Sort(UiTableSort),
        }

        let name = UiTableColumnId::new("name").unwrap();
        let status = UiTableColumnId::new("status").unwrap();
        let alpha = UiTableRowId::new("alpha").unwrap();
        let mut manifest = UiBindingManifest::<TableState, TableMsg>::new();
        manifest
            .register_table_columns_property("inventory_columns", |state| state.columns.clone())
            .unwrap();
        manifest
            .register_table_rows_property("inventory_rows", |state| state.rows.clone())
            .unwrap();
        manifest
            .register_table_row_id_property("inventory_selected", |state| state.selected.clone())
            .unwrap();
        manifest
            .register_table_sort_property("inventory_sort", |state| state.sort.clone())
            .unwrap();
        manifest
            .register_table_row_id_action("inventory_selected_changed", |id| Ok(TableMsg::Row(id)))
            .unwrap();
        manifest
            .register_table_sort_action("inventory_sort_changed", |sort| Ok(TableMsg::Sort(sort)))
            .unwrap();

        let state = TableState {
            columns: vec![
                UiTableColumn::new(name.clone(), "Name")
                    .fill_width(2)
                    .sortable(true),
                UiTableColumn::new(status.clone(), "Status")
                    .fixed_width(Dp::new(120.0))
                    .alignment(UiTableColumnAlignment::Center),
            ],
            rows: vec![UiTableRow::new(
                alpha.clone(),
                [
                    (name.clone(), "Alpha".to_owned()),
                    (status.clone(), "Ready".to_owned()),
                ],
            )],
            selected: Some(alpha.clone()),
            sort: Some(UiTableSort::new(
                name.clone(),
                UiTableSortDirection::Ascending,
            )),
        };
        assert_eq!(
            manifest.schema().properties["inventory_columns"],
            UiValueType::TableColumnArray
        );
        assert_eq!(
            manifest.schema().properties["inventory_rows"],
            UiValueType::TableRowArray
        );
        assert_eq!(
            manifest.schema().properties["inventory_selected"],
            UiValueType::NullableTableRowId
        );
        assert_eq!(
            manifest.schema().properties["inventory_sort"],
            UiValueType::NullableTableSort
        );
        assert_eq!(
            manifest.read_property("inventory_rows", &state),
            Some(serde_json::json!([{
                "id": "alpha",
                "cells": { "name": "Alpha", "status": "Ready" }
            }]))
        );
        assert_eq!(
            manifest.map_action(
                "inventory_selected_changed",
                Value::String("alpha".to_owned())
            ),
            Ok(TableMsg::Row(alpha))
        );
        assert_eq!(
            manifest.map_action(
                "inventory_sort_changed",
                serde_json::json!({ "column": "name", "direction": "descending" })
            ),
            Ok(TableMsg::Sort(UiTableSort::new(
                name,
                UiTableSortDirection::Descending
            )))
        );
        assert!(!UiValueType::TableColumnArray.matches(&serde_json::json!([
            { "id": "same", "header": "First" },
            { "id": "same", "header": "Second" }
        ])));
        assert!(!UiValueType::TableColumnArray.matches(&serde_json::json!([
            { "id": "bad", "header": "Bad", "width": { "kind": "fill", "weight": 0 } }
        ])));
    }

    #[cfg(feature = "table")]
    #[test]
    fn table_contract_rejects_misaligned_cells_selection_and_sort() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "inventory",
                "component": "table",
                "properties": {
                  "columns": [
                    { "id": "name", "header": "Name", "sortable": true },
                    { "id": "status", "header": "Status" }
                  ],
                  "rows": [
                    { "id": "alpha", "cells": { "name": "Alpha", "status": "Ready" } }
                  ],
                  "selected": "alpha",
                  "sort": { "column": "name", "direction": "ascending" }
                }
              }
            }"#,
        )
        .unwrap();
        let report = valid.validate(&UiFeatureSet::new(["table"]), &UiBindingSchema::default());
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "inventory",
                "component": "table",
                "properties": {
                  "columns": [
                    { "id": "name", "header": "Name", "sortable": true },
                    { "id": "status", "header": "Status" }
                  ],
                  "rows": [
                    { "id": "alpha", "cells": { "name": "Alpha" } }
                  ],
                  "selected": "missing",
                  "sort": { "column": "status", "direction": "ascending" }
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&UiFeatureSet::new(["table"]), &UiBindingSchema::default());
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            3
        );
    }

    #[test]
    fn auto_suggest_contract_validates_controlled_semantic_state() {
        let valid = UiDocument::from_json(
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
                    UiValueType::AutoSuggestionArray,
                ),
                ("country_query".to_owned(), UiValueType::String),
                (
                    "country_highlighted".to_owned(),
                    UiValueType::NullableAutoSuggestionId,
                ),
                ("country_expanded".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                ("country_query_changed".to_owned(), UiValueType::String),
                ("country_chosen".to_owned(), UiValueType::AutoSuggestionId),
                (
                    "country_submitted".to_owned(),
                    UiValueType::AutoSuggestSubmission,
                ),
                ("country_expanded_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["auto-suggest"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "country-search",
                "component": "auto_suggest",
                "properties": {
                  "suggestions": [
                    { "id": "china", "text": "China" },
                    { "id": "chile", "text": "Chile" }
                  ],
                  "highlighted": "missing"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.highlighted"
        }));
    }

    #[test]
    fn menu_flyout_contract_validates_target_tree_and_controlled_open_state() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "file-menu",
                "component": "menu_flyout",
                "properties": {
                  "target": "open-file-menu",
                  "items": [
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
                  ]
                },
                "property_bindings": { "open": "file_menu_open" },
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
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("file_menu_open".to_owned(), UiValueType::Boolean)]),
            actions: BTreeMap::from([
                (
                    "file_menu_invoked".to_owned(),
                    UiValueType::MenuFlyoutItemId,
                ),
                ("file_menu_open_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["menu-flyout", "button", "label"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let mut invalid_target = valid.clone();
        invalid_target.root.properties.insert(
            "target".to_owned(),
            Value::String("missing-target".to_owned()),
        );
        let report = invalid_target.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.target"
        }));

        let mut invalid_items = valid.clone();
        invalid_items.root.properties.insert(
            "items".to_owned(),
            serde_json::json!([
                { "kind": "command", "id": "same", "label": "First" },
                {
                  "kind": "submenu",
                  "id": "more",
                  "label": "More",
                  "items": [
                    { "kind": "command", "id": "same", "label": "Second" }
                  ]
                }
            ]),
        );
        let report = invalid_items.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyType
                && diagnostic.path == "$.root.properties.items"
        }));

        let mut uncontrolled = valid;
        uncontrolled.root.action_bindings.remove("open_change");
        let report = uncontrolled.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.property_bindings.open"
        }));
    }

    #[test]
    fn flyout_contract_validates_target_geometry_and_controlled_dismissal() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "details-flyout",
                "component": "flyout",
                "properties": {
                  "target": "open-details",
                  "content_width": 280,
                  "content_height": 120,
                  "placement": "right"
                },
                "property_bindings": { "open": "details_open" },
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
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("details_open".to_owned(), UiValueType::Boolean)]),
            actions: BTreeMap::from([
                (
                    "details_dismissed".to_owned(),
                    UiValueType::FlyoutDismissReason,
                ),
                ("details_open_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["flyout", "button", "label"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let mut invalid_target = valid.clone();
        invalid_target.root.properties.insert(
            "target".to_owned(),
            Value::String("flyout-content".to_owned()),
        );
        let report = invalid_target.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.target"
        }));

        let mut invalid_geometry = valid;
        invalid_geometry
            .root
            .properties
            .insert("content_height".to_owned(), Value::from(0));
        invalid_geometry
            .root
            .properties
            .insert("placement".to_owned(), Value::String("center".to_owned()));
        let report = invalid_geometry.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.content_height"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.placement"
        }));
    }

    #[test]
    fn breadcrumb_contract_validates_stable_path_and_controlled_overflow() {
        let valid = UiDocument::from_json(
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
                    UiValueType::BreadcrumbItemArray,
                ),
                ("navigation_overflow_open".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                (
                    "navigation_selected".to_owned(),
                    UiValueType::BreadcrumbItemId,
                ),
                (
                    "navigation_overflow_changed".to_owned(),
                    UiValueType::Boolean,
                ),
            ]),
        };
        let features = UiFeatureSet::new(["breadcrumb"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "location-path",
                "component": "breadcrumb",
                "properties": {
                  "items": [
                    { "id": "same", "label": "Home" },
                    { "id": "same", "label": "Settings" }
                  ]
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyType
                && diagnostic.path == "$.root.properties.items"
        }));
    }

    #[test]
    fn command_palette_contract_validates_controlled_semantic_state() {
        let valid = UiDocument::from_json(
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
                ("commands".to_owned(), UiValueType::CommandPaletteItemArray),
                ("command_query".to_owned(), UiValueType::String),
                (
                    "command_highlighted".to_owned(),
                    UiValueType::NullableCommandPaletteItemId,
                ),
                ("command_open".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                ("command_query_changed".to_owned(), UiValueType::String),
                (
                    "command_highlight_changed".to_owned(),
                    UiValueType::CommandPaletteItemId,
                ),
                (
                    "command_invoked".to_owned(),
                    UiValueType::CommandPaletteItemId,
                ),
                ("command_open_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["command-palette"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "app-commands",
                "component": "command_palette",
                "properties": {
                  "items": [
                    { "id": "open-file", "title": "Open file" },
                    { "id": "disabled", "title": "Disabled", "enabled": false }
                  ],
                  "query": "open",
                  "highlighted": "disabled",
                  "open": true
                },
                "children": [
                  { "id": "page", "component": "stack" }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.highlighted"
        }));
    }

    #[test]
    fn tree_contract_validates_controlled_hierarchy_state() {
        let valid = UiDocument::from_json(
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
                ("project_nodes".to_owned(), UiValueType::TreeNodeArray),
                ("project_expanded".to_owned(), UiValueType::TreeNodeIdArray),
                (
                    "project_selected".to_owned(),
                    UiValueType::NullableTreeNodeId,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "project_selected_changed".to_owned(),
                    UiValueType::TreeNodeId,
                ),
                (
                    "project_expanded_changed".to_owned(),
                    UiValueType::TreeNodeIdArray,
                ),
                ("project_invoked".to_owned(), UiValueType::TreeNodeId),
            ]),
        };
        let features = UiFeatureSet::new(["tree"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "project-tree",
                "component": "tree",
                "properties": {
                  "nodes": [
                    {
                      "id": "workspace",
                      "label": "Workspace",
                      "children": [{ "id": "readme", "label": "README.md" }]
                    }
                  ],
                  "expanded": ["readme"],
                  "selected": "missing"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.selected"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.expanded"
        }));
    }

    #[test]
    fn icon_contract_validates_semantic_symbol_size_and_color() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "status-icon",
                "component": "icon",
                "properties": {
                  "icon": "Info",
                  "size": "large",
                  "color": "accent"
                }
              }
            }"#,
        )
        .unwrap();
        let features = UiFeatureSet::new(["icon"]);
        let report = valid.validate(&features, &UiBindingSchema::default());
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(
            UiDocumentReleaseArtifact::compile(&valid, &features, &UiBindingSchema::default())
                .is_ok()
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "status-icon",
                "component": "icon",
                "properties": {
                  "icon": "NotAnIcon",
                  "size": "huge",
                  "color": "brand"
                },
                "theme_tokens": {
                  "foreground": "accent"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for property in ["icon", "size", "color"] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                    && diagnostic.path == format!("$.root.properties.{property}")
            }));
        }
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::ConflictingPropertySource
                && diagnostic.path == "$.root.theme_tokens.foreground"
        }));
    }

    #[test]
    fn badge_contract_validates_kind_content_tone_and_structural_kind() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "unread-count",
                "component": "badge",
                "properties": {
                  "kind": "number",
                  "value": 12,
                  "tone": "accent"
                }
              }
            }"#,
        )
        .unwrap();
        let features = UiFeatureSet::new(["badge"]);
        let report = valid.validate(&features, &UiBindingSchema::default());
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(
            UiDocumentReleaseArtifact::compile(&valid, &features, &UiBindingSchema::default())
                .is_ok()
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "invalid-badge",
                "component": "badge",
                "properties": {
                  "kind": "icon",
                  "value": 4294967296,
                  "icon": "NotAnIcon",
                  "tone": "brand"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for property in ["value", "icon", "tone"] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                    && diagnostic.path == format!("$.root.properties.{property}")
            }));
        }

        let bound_kind = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "dynamic-kind",
                "component": "badge",
                "property_bindings": { "kind": "badge_kind" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("badge_kind".to_owned(), UiValueType::String)]),
            actions: BTreeMap::new(),
        };
        let report = bound_kind.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.property_bindings.kind"
        }));
    }

    #[test]
    fn command_bar_contract_validates_stable_groups_and_button_presentations() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "editor-command-bar",
                "component": "command_bar",
                "properties": {
                  "trailing": ["about"],
                  "gap": 4
                },
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
            actions: BTreeMap::from([("save_clicked".to_owned(), UiValueType::Null)]),
        };
        let features = UiFeatureSet::new(["document-shell", "button"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "editor-command-bar",
                "component": "command_bar",
                "properties": {
                  "trailing": ["missing", "missing"],
                  "gap": -1
                },
                "children": [
                  {
                    "id": "save",
                    "component": "button",
                    "properties": {
                      "label": "Save",
                      "presentation": "toolbar"
                    }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for path in [
            "$.root.properties.trailing[0]",
            "$.root.properties.trailing[1]",
            "$.root.properties.gap",
            "$.root.children[0].properties.icon",
        ] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                matches!(
                    diagnostic.code,
                    UiDiagnosticCode::InvalidPropertyValue
                        | UiDiagnosticCode::MissingRequiredProperty
                ) && diagnostic.path == path
            }));
        }
    }

    #[test]
    fn navigation_contract_validates_groups_selection_and_content_slot() {
        let valid = UiDocument::from_json(
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
                UiValueType::NavigationItemId,
            )]),
        };
        let features = UiFeatureSet::new(["shell", "label"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "app-navigation",
                "component": "navigation",
                "properties": {
                  "title": "ZSUI",
                  "items": [{ "id": "same", "label": "Home", "icon": "App" }],
                  "footer_items": [{ "id": "same", "label": "Settings", "icon": "Settings" }],
                  "selected": "missing",
                  "pane_width": -1
                },
                "children": [
                  { "id": "navigation-content", "component": "text", "properties": { "text": "Home" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for path in [
            "$.root.properties.footer_items",
            "$.root.properties.selected",
            "$.root.properties.pane_width",
        ] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == UiDiagnosticCode::InvalidPropertyValue && diagnostic.path == path
            }));
        }
    }

    #[test]
    fn grid_view_contract_validates_controlled_semantic_selection() {
        let valid = UiDocument::from_json(
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
                ("library_items".to_owned(), UiValueType::GridViewItemArray),
                (
                    "library_selected".to_owned(),
                    UiValueType::NullableGridViewItemId,
                ),
            ]),
            actions: BTreeMap::from([
                (
                    "library_selected_changed".to_owned(),
                    UiValueType::GridViewItemId,
                ),
                ("library_invoked".to_owned(), UiValueType::GridViewItemId),
            ]),
        };
        let features = UiFeatureSet::new(["grid-view"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        assert!(UiDocumentReleaseArtifact::compile(&valid, &features, &bindings).is_ok());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "library-grid",
                "component": "grid_view",
                "properties": {
                  "items": [
                    { "id": "documents", "title": "Documents" },
                    { "id": "photos", "title": "Photos" }
                  ],
                  "selected": "missing"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.selected"
        }));
    }

    #[test]
    fn date_picker_contract_validates_range_and_controlled_state() {
        let valid = UiDocument::from_json(
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
                  "expanded": "release_date_expanded"
                },
                "action_bindings": {
                  "change": "release_date_changed",
                  "month_change": "release_month_changed",
                  "expanded_change": "release_date_expanded_changed"
                }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([
                ("release_date".to_owned(), UiValueType::Date),
                ("release_month".to_owned(), UiValueType::Date),
                ("release_date_expanded".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                ("release_date_changed".to_owned(), UiValueType::Date),
                ("release_month_changed".to_owned(), UiValueType::Date),
                (
                    "release_date_expanded_changed".to_owned(),
                    UiValueType::Boolean,
                ),
            ]),
        };
        let report = valid.validate(&UiFeatureSet::new(["date-picker"]), &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        let handoff = UiAiHandoffPackage::build(
            &valid,
            &UiFeatureSet::new(["date-picker"]),
            &bindings,
            Some(&BTreeMap::from([
                (
                    "release_date".to_owned(),
                    Value::String("2026-07-22".to_owned()),
                ),
                (
                    "release_month".to_owned(),
                    Value::String("2026-07-01".to_owned()),
                ),
                ("release_date_expanded".to_owned(), Value::Bool(false)),
            ])),
            None,
        )
        .unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "date_picker")
            .unwrap();
        assert!(contract
            .properties
            .iter()
            .any(|property| property.name == "value" && property.value_type == UiValueType::Date));

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "invalid-date",
                "component": "date_picker",
                "properties": {
                  "value": "2026-02-30",
                  "minimum": "2026-12-31",
                  "maximum": "2026-01-01",
                  "visible_month": "2026-07-22"
                },
                "localization": { "expanded": "date.expanded" }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(
            &UiFeatureSet::new(["date-picker"]),
            &UiBindingSchema::default(),
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyType));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidLocalization));
    }

    #[test]
    fn time_picker_contract_validates_increment_clock_and_controlled_state() {
        let valid = UiDocument::from_json(
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
                ("meeting_time".to_owned(), UiValueType::Time),
                ("meeting_time_expanded".to_owned(), UiValueType::Boolean),
            ]),
            actions: BTreeMap::from([
                ("meeting_time_changed".to_owned(), UiValueType::Time),
                (
                    "meeting_time_expanded_changed".to_owned(),
                    UiValueType::Boolean,
                ),
            ]),
        };
        let report = valid.validate(&UiFeatureSet::new(["time-picker"]), &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "invalid-time",
                "component": "time_picker",
                "properties": {
                  "value": "09:17",
                  "minute_increment": 15,
                  "clock_format": "windows"
                },
                "localization": { "expanded": "time.expanded" }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(
            &UiFeatureSet::new(["time-picker"]),
            &UiBindingSchema::default(),
        );
        assert!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count()
                >= 2
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidLocalization));
    }

    #[test]
    fn color_picker_contract_validates_rgba_channel_and_alpha_policy() {
        let valid = UiDocument::from_json(
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
                ("accent_color".to_owned(), UiValueType::Color),
                ("accent_color_expanded".to_owned(), UiValueType::Boolean),
                ("accent_color_channel".to_owned(), UiValueType::String),
            ]),
            actions: BTreeMap::from([
                ("accent_color_changed".to_owned(), UiValueType::Color),
                (
                    "accent_color_expanded_changed".to_owned(),
                    UiValueType::Boolean,
                ),
                (
                    "accent_color_channel_changed".to_owned(),
                    UiValueType::String,
                ),
            ]),
        };
        let report = valid.validate(&UiFeatureSet::new(["color-picker"]), &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let invalid = UiDocument::from_json(
            r##"{
              "schema_version": 1,
              "root": {
                "id": "invalid-color",
                "component": "color_picker",
                "properties": {
                  "value": "#2060A0E0",
                  "active_channel": "alpha",
                  "alpha_enabled": false
                }
              }
            }"##,
        )
        .unwrap();
        let report = invalid.validate(
            &UiFeatureSet::new(["color-picker"]),
            &UiBindingSchema::default(),
        );
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            2
        );

        let noncanonical = UiDocument::from_json(
            r##"{
              "schema_version": 1,
              "root": {
                "id": "noncanonical-color",
                "component": "color_picker",
                "properties": { "value": "#2060a0e0" }
              }
            }"##,
        )
        .unwrap();
        assert!(noncanonical
            .validate(
                &UiFeatureSet::new(["color-picker"]),
                &UiBindingSchema::default(),
            )
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyType));
    }

    #[cfg(feature = "password-box")]
    #[test]
    fn password_box_contract_and_manifest_keep_secrets_out_of_json() {
        #[derive(Default)]
        struct SecretState {
            password: crate::ZsPassword,
        }
        #[derive(Debug, PartialEq, Eq)]
        enum SecretMsg {
            Changed(crate::ZsPassword),
        }

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
        let mut manifest = UiBindingManifest::<SecretState, SecretMsg>::new();
        manifest
            .register_secret_property("account_password", |state| state.password.clone())
            .unwrap();
        manifest
            .register_secret_action("account_password_changed", |password| {
                Ok(SecretMsg::Changed(password))
            })
            .unwrap();
        let schema = manifest.schema();
        let features = UiFeatureSet::new(["password-box"]);
        assert!(document.validate(&features, &schema).is_valid());
        assert_eq!(schema.properties["account_password"], UiValueType::String);
        assert_eq!(
            schema.actions["account_password_changed"],
            UiValueType::String
        );

        let secret = "never-serialize-this-password";
        let state = SecretState {
            password: crate::ZsPassword::from(secret),
        };
        assert_eq!(manifest.read_property("account_password", &state), None);
        assert_eq!(
            manifest
                .read_secret_property("account_password", &state)
                .as_ref()
                .map(crate::ZsPassword::as_str),
            Some(secret)
        );
        let secure_values = manifest.read_secret_values(&state);
        assert_eq!(
            secure_values.get("account_password").unwrap().as_str(),
            secret
        );
        assert_eq!(
            manifest.map_secret_action(
                "account_password_changed",
                crate::ZsPassword::from("changed")
            ),
            Ok(SecretMsg::Changed(crate::ZsPassword::from("changed")))
        );
        assert!(!format!("{manifest:?}").contains(secret));
        assert!(!format!("{secure_values:?}").contains(secret));

        let leaked_values = BTreeMap::from([(
            "account_password".to_owned(),
            Value::String(secret.to_owned()),
        )]);
        let report = validate_ui_document_binding_values(&document, &schema, &leaked_values);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == UiDiagnosticCode::SensitiveBindingValue }));
        assert!(matches!(
            UiAiHandoffPackage::build(&document, &features, &schema, Some(&leaked_values), None,),
            Err(UiAiHandoffBuildError::InvalidValues(_))
        ));

        let handoff = UiAiHandoffPackage::build(&document, &features, &schema, None, None).unwrap();
        assert_eq!(
            handoff.manifest.sensitive_values,
            vec!["account_password".to_owned()]
        );
        assert!(handoff.manifest.missing_values.is_empty());
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "password_box")
            .unwrap();
        assert!(contract
            .properties
            .iter()
            .any(|property| property.name == "value" && property.sensitive));
        assert!(!handoff.handoff_json.contains(secret));
        assert!(handoff.values_json.is_none());

        let mut literal = document.clone();
        literal.root.property_bindings.remove("value");
        literal
            .root
            .properties
            .insert("value".to_owned(), Value::String("unsafe".to_owned()));
        let report = literal.validate(&features, &schema);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == UiDiagnosticCode::SensitiveBindingValue }));

        let mut invalid_mode = document.clone();
        invalid_mode.root.properties.insert(
            "reveal_mode".to_owned(),
            Value::String("always-on".to_owned()),
        );
        assert!(invalid_mode
            .validate(&features, &schema)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue));
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
        document.root.children[0].component = "split_view".to_owned();
        document.root.children[1].component = "imaginary".to_owned();
        let report = document.validate(
            &UiFeatureSet::new(["button", "label", "shell"]),
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
    fn list_contract_uses_child_ids_as_stable_selection_values() {
        let valid = UiDocument::from_json(
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
            properties: BTreeMap::from([("selected_profile".to_owned(), UiValueType::String)]),
            actions: BTreeMap::from([("selected_profile_changed".to_owned(), UiValueType::String)]),
        };
        let features = UiFeatureSet::new(["list", "label"]);
        assert!(valid.validate(&features, &bindings).is_valid());
        let handoff = UiAiHandoffPackage::build(&valid, &features, &bindings, None, None).unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "list")
            .unwrap();
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::AtLeast { minimum: 1 }
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "profiles",
                "component": "list",
                "properties": { "selected": "missing" },
                "children": [
                  { "id": "balanced", "component": "text", "properties": { "text": "Balanced" } }
                ]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path.ends_with("properties.selected")
        }));
    }

    #[test]
    fn content_dialog_contract_requires_controlled_close_and_available_roles() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "confirm",
                "component": "content_dialog",
                "properties": {
                  "content": "This action cannot be undone.",
                  "primary_button": "Delete",
                  "close_button": "Cancel",
                  "default_button": "primary"
                },
                "property_bindings": { "open": "confirm_open" },
                "action_bindings": { "open_change": "confirm_open_changed" },
                "children": [{ "id": "page", "component": "stack" }]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("confirm_open".to_owned(), UiValueType::Boolean)]),
            actions: BTreeMap::from([("confirm_open_changed".to_owned(), UiValueType::Boolean)]),
        };
        let features = UiFeatureSet::new(["dialog"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "confirm",
                "component": "content_dialog",
                "properties": {
                  "content": "This action cannot be undone.",
                  "close_button": "Cancel",
                  "default_button": "secondary"
                },
                "property_bindings": { "open": "confirm_open" },
                "children": [{ "id": "page", "component": "stack" }]
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &bindings);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.property_bindings.open"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.default_button"
        }));
    }

    #[test]
    fn info_bar_contract_validates_semantics_and_typed_event() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "sync-status",
                "component": "info_bar",
                "properties": {
                  "message": "All changes are synchronized.",
                  "title": "Up to date",
                  "severity": "success",
                  "action_label": "View activity",
                  "closable": true
                },
                "action_bindings": { "event": "sync_status_event" }
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::new(),
            actions: BTreeMap::from([("sync_status_event".to_owned(), UiValueType::String)]),
        };
        let features = UiFeatureSet::new(["info-bar"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        let handoff = UiAiHandoffPackage::build(&valid, &features, &bindings, None, None).unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "info_bar")
            .unwrap();
        assert_eq!(contract.cargo_feature.as_deref(), Some("info-bar"));
        assert_eq!(
            contract.actions,
            vec![UiAiHandoffActionContract {
                name: "event".to_owned(),
                payload_type: UiValueType::String,
            }]
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "sync-status",
                "component": "info_bar",
                "properties": {
                  "message": "",
                  "severity": "critical",
                  "action_label": " "
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for property in ["message", "severity", "action_label"] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                    && diagnostic.path == format!("$.root.properties.{property}")
            }));
        }
    }

    #[test]
    fn toast_contract_requires_one_page_and_controlled_open_state() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "saved",
                "component": "toast",
                "properties": {
                  "message": "Saved",
                  "action_label": "Undo",
                  "duration": "long"
                },
                "property_bindings": { "open": "saved_open" },
                "action_bindings": {
                  "result": "saved_result",
                  "open_change": "saved_open_changed"
                },
                "children": [{ "id": "page", "component": "stack" }]
              }
            }"#,
        )
        .unwrap();
        let bindings = UiBindingSchema {
            properties: BTreeMap::from([("saved_open".to_owned(), UiValueType::Boolean)]),
            actions: BTreeMap::from([
                ("saved_result".to_owned(), UiValueType::String),
                ("saved_open_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["toast"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);
        let handoff = UiAiHandoffPackage::build(&valid, &features, &bindings, None, None).unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "toast")
            .unwrap();
        assert_eq!(contract.cargo_feature.as_deref(), Some("toast"));
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::Exactly { count: 1 }
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "saved",
                "component": "toast",
                "properties": {
                  "message": "",
                  "duration": "forever"
                },
                "property_bindings": { "open": "saved_open" },
                "children": []
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.message"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.properties.duration"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                && diagnostic.path == "$.root.property_bindings.open"
        }));
    }

    #[test]
    fn tooltip_contract_requires_one_child_and_valid_semantics() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-help",
                "component": "tooltip",
                "properties": {
                  "placement": "bottom",
                  "open_delay_ms": 250
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
            properties: BTreeMap::from([("save_tooltip".to_owned(), UiValueType::String)]),
            actions: BTreeMap::new(),
        };
        let features = UiFeatureSet::new(["tooltip", "button"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let handoff = UiAiHandoffPackage::build(&valid, &features, &bindings, None, None).unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "tooltip")
            .unwrap();
        assert_eq!(contract.cargo_feature.as_deref(), Some("tooltip"));
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::Exactly { count: 1 }
        );
        assert!(contract.properties.iter().any(|property| {
            property.name == "open_delay_ms"
                && property.value_type == UiValueType::Integer
                && !property.required
        }));

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-help",
                "component": "tooltip",
                "properties": {
                  "text": " ",
                  "placement": "center",
                  "open_delay_ms": -1
                },
                "children": []
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &UiBindingSchema::default());
        for property in ["text", "placement"] {
            assert!(report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == UiDiagnosticCode::InvalidPropertyValue
                    && diagnostic.path == format!("$.root.properties.{property}")
            }));
        }
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidPropertyType
                && diagnostic.path == "$.root.properties.open_delay_ms"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == UiDiagnosticCode::InvalidChildCount
                && diagnostic.path == "$.root.children"
        }));
    }

    #[test]
    fn teaching_tip_contract_validates_target_content_and_controlled_open_state() {
        let valid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-guidance",
                "component": "teaching_tip",
                "properties": {
                  "target": "save-button",
                  "title": "Automatic saving",
                  "action_label": "Review settings",
                  "placement": "top"
                },
                "property_bindings": {
                  "open": "guidance_open",
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
                ("guidance_open".to_owned(), UiValueType::Boolean),
                ("guidance_subtitle".to_owned(), UiValueType::String),
            ]),
            actions: BTreeMap::from([
                ("guidance_result".to_owned(), UiValueType::String),
                ("guidance_open_changed".to_owned(), UiValueType::Boolean),
            ]),
        };
        let features = UiFeatureSet::new(["teaching-tip", "button"]);
        let report = valid.validate(&features, &bindings);
        assert!(report.is_valid(), "{:#?}", report.diagnostics);

        let handoff = UiAiHandoffPackage::build(&valid, &features, &bindings, None, None).unwrap();
        let contract = handoff
            .manifest
            .component_contracts
            .iter()
            .find(|contract| contract.component == "teaching_tip")
            .unwrap();
        assert_eq!(contract.cargo_feature.as_deref(), Some("teaching-tip"));
        assert_eq!(
            contract.children,
            UiAiHandoffChildPolicy::Exactly { count: 1 }
        );
        assert_eq!(
            contract.actions,
            vec![
                UiAiHandoffActionContract {
                    name: "open_change".to_owned(),
                    payload_type: UiValueType::Boolean,
                },
                UiAiHandoffActionContract {
                    name: "result".to_owned(),
                    payload_type: UiValueType::String,
                },
            ]
        );

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "save-guidance",
                "component": "teaching_tip",
                "properties": {
                  "target": "missing",
                  "action_label": " ",
                  "placement": "center"
                },
                "property_bindings": { "open": "guidance_open" },
                "children": []
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(&features, &bindings);
        for path in [
            "$.root.properties",
            "$.root.properties.target",
            "$.root.properties.action_label",
            "$.root.properties.placement",
            "$.root.property_bindings.open",
            "$.root.children",
        ] {
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.path == path),
                "missing diagnostic at {path}: {:#?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn progress_ring_contract_validates_nullable_range_and_native_size() {
        let valid = UiDocument::from_json(
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
            properties: BTreeMap::from([("sync_progress".to_owned(), UiValueType::NullableNumber)]),
            actions: BTreeMap::new(),
        };
        assert!(valid
            .validate(&UiFeatureSet::new(["progress-ring"]), &bindings)
            .is_valid());
        assert!(validate_ui_binding_values(
            &bindings,
            &BTreeMap::from([("sync_progress".to_owned(), Value::Null)])
        )
        .is_valid());

        let invalid = UiDocument::from_json(
            r#"{
              "schema_version": 1,
              "root": {
                "id": "sync-progress",
                "component": "progress_ring",
                "properties": {
                  "value": 2.0,
                  "minimum": 0.0,
                  "maximum": 1.0,
                  "size": "huge"
                }
              }
            }"#,
        )
        .unwrap();
        let report = invalid.validate(
            &UiFeatureSet::new(["progress-ring"]),
            &UiBindingSchema::default(),
        );
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == UiDiagnosticCode::InvalidPropertyValue)
                .count(),
            2
        );
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
