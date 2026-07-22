//! Prebuilt native development viewer for validated [`UiDocument`] files.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ui_document::{
    UiBindingSchema, UiDiagnostic, UiDocument, UiFeatureSet, UiLayout, UiNode,
};
use crate::ui_document_runtime::ui_document_view;
pub use crate::ui_document_runtime::UiDocumentAction as UiViewerAction;
use crate::{column, text, AppCx, Dp, ViewNode};

pub const ZSUI_UI_VIEWER_DEFAULT_POLL_INTERVAL_MS: u64 = 250;
pub const ZSUI_UI_VIEWER_PROOF_SCHEMA: &str = "zsui.ui-viewer-proof/v1";
pub const ZSUI_UI_VIEWER_PROOF_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiViewerMessage {
    Action(UiViewerAction),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerState {
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    #[serde(default)]
    pub actions: Vec<UiViewerAction>,
}

impl UiViewerState {
    pub fn with_properties(properties: BTreeMap<String, Value>) -> Self {
        Self {
            properties,
            actions: Vec::new(),
        }
    }
}

pub fn ui_viewer_update(state: &mut UiViewerState, message: UiViewerMessage, _cx: &mut AppCx) {
    match message {
        UiViewerMessage::Action(action) => {
            const MAX_ACTION_HISTORY: usize = 64;
            if let Some(binding) = &action.property_binding {
                state
                    .properties
                    .insert(binding.clone(), action.payload.clone());
            }
            if state.actions.len() == MAX_ACTION_HISTORY {
                state.actions.remove(0);
            }
            state.actions.push(action);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerSourceSnapshot {
    pub revision: u64,
    pub document_schema_version: u32,
    pub document_path: PathBuf,
    pub binding_path: Option<PathBuf>,
    pub node_count: usize,
    pub nodes: Vec<UiViewerNodeSnapshot>,
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reload: Option<UiViewerReloadReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerNodeSnapshot {
    pub path: String,
    pub id: String,
    pub widget_id: u64,
    pub component: String,
    pub layout: UiLayout,
    pub child_count: usize,
}

/// Deterministic compatibility result for one accepted source reload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerReloadReport {
    pub from_revision: u64,
    pub to_revision: u64,
    pub preserved_node_ids: Vec<String>,
    pub added_node_ids: Vec<String>,
    pub state_resets: Vec<UiViewerStateReset>,
}

impl UiViewerReloadReport {
    pub fn preserves_all_existing_state(&self) -> bool {
        self.state_resets.is_empty()
    }
}

/// One stable author identity whose previous transient state cannot be reused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerStateReset {
    pub node_id: String,
    pub previous_component: String,
    pub current_component: Option<String>,
    pub reason: UiViewerStateResetReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiViewerStateResetReason {
    NodeRemoved,
    ComponentChanged,
    ComponentConfigurationChanged,
}

#[derive(Clone)]
pub struct UiViewerSource {
    inner: Arc<Mutex<UiViewerSourceState>>,
    poll_interval_ms: u64,
}

impl fmt::Debug for UiViewerSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiViewerSource")
            .field("snapshot", &self.snapshot())
            .field("poll_interval_ms", &self.poll_interval_ms)
            .finish()
    }
}

struct UiViewerSourceState {
    document_path: PathBuf,
    binding_path: Option<PathBuf>,
    last_seen_hash: u64,
    document: Arc<UiDocument>,
    bindings: UiBindingSchema,
    revision: u64,
    last_error: Option<String>,
    error_source_hash: Option<u64>,
    last_reload: Option<UiViewerReloadReport>,
}

impl UiViewerSource {
    pub fn open(
        document_path: impl Into<PathBuf>,
        binding_path: Option<impl Into<PathBuf>>,
    ) -> Result<Self, UiViewerError> {
        let document_path = document_path.into();
        let binding_path = binding_path.map(Into::into);
        let sources = read_sources(&document_path, binding_path.as_deref())?;
        let (document, bindings) = parse_and_validate_sources(
            &document_path,
            binding_path.as_deref(),
            &sources.document,
            &sources.bindings,
        )?;
        Ok(Self {
            inner: Arc::new(Mutex::new(UiViewerSourceState {
                document_path,
                binding_path,
                last_seen_hash: sources.hash,
                document: Arc::new(document),
                bindings,
                revision: 1,
                last_error: None,
                error_source_hash: None,
                last_reload: None,
            })),
            poll_interval_ms: ZSUI_UI_VIEWER_DEFAULT_POLL_INTERVAL_MS,
        })
    }

    pub fn poll_interval_ms(mut self, interval_ms: u64) -> Self {
        self.poll_interval_ms = interval_ms.max(16);
        self
    }

    /// Reloads changed source files. Invalid edits leave the last valid
    /// document active and expose a diagnostic through [`Self::snapshot`].
    pub fn refresh(&self) -> bool {
        let (document_path, binding_path, last_seen_hash) = {
            let state = self.lock();
            (
                state.document_path.clone(),
                state.binding_path.clone(),
                state.last_seen_hash,
            )
        };
        let sources = match read_sources(&document_path, binding_path.as_deref()) {
            Ok(sources) => sources,
            Err(error) => {
                self.record_error(error.to_string(), None);
                return false;
            }
        };
        if sources.hash == last_seen_hash {
            let mut state = self.lock();
            if state.last_error.is_some() && state.error_source_hash.is_none() {
                state.last_error = None;
            }
            return false;
        }

        match parse_and_validate_sources(
            &document_path,
            binding_path.as_deref(),
            &sources.document,
            &sources.bindings,
        ) {
            Ok((document, bindings)) => {
                let mut state = self.lock();
                let next_revision = state.revision.saturating_add(1);
                let reload = reload_compatibility_report(
                    &state.document,
                    &document,
                    state.revision,
                    next_revision,
                );
                state.last_seen_hash = sources.hash;
                state.document = Arc::new(document);
                state.bindings = bindings;
                state.revision = next_revision;
                state.last_error = None;
                state.error_source_hash = None;
                state.last_reload = Some(reload);
                true
            }
            Err(error) => {
                self.record_error(error.to_string(), Some(sources.hash));
                false
            }
        }
    }

    pub fn snapshot(&self) -> UiViewerSourceSnapshot {
        let state = self.lock();
        let mut nodes = Vec::new();
        collect_viewer_nodes(&state.document.root, "$.root", &mut nodes);
        UiViewerSourceSnapshot {
            revision: state.revision,
            document_schema_version: state.document.schema_version,
            document_path: state.document_path.clone(),
            binding_path: state.binding_path.clone(),
            node_count: nodes.len(),
            nodes,
            last_error: state.last_error.clone(),
            last_reload: state.last_reload.clone(),
        }
    }

    pub fn view(&self, viewer_state: &UiViewerState) -> ViewNode<UiViewerMessage> {
        self.refresh();
        let (document, bindings, last_error, state_reset_count) = {
            let state = self.lock();
            (
                Arc::clone(&state.document),
                state.bindings.clone(),
                state.last_error.clone(),
                state
                    .last_reload
                    .as_ref()
                    .map(|reload| reload.state_resets.len())
                    .unwrap_or(0),
            )
        };
        let content = ui_document_view(
            &document,
            &bindings,
            &viewer_state.properties,
            UiViewerMessage::Action,
        )
        .unwrap_or_else(|error| text(format!("UI document runtime error: {error}")));
        let mut root = if let Some(error) = last_error {
            column([text(format!("UI document reload error: {error}")), content]).gap(Dp::new(8.0))
        } else if state_reset_count > 0 {
            column([
                text(format!(
                    "UI reload reset state for {state_reset_count} incompatible node(s)"
                )),
                content,
            ])
            .gap(Dp::new(8.0))
        } else {
            content
        };
        root = root.with_document_poll_interval_ms(self.poll_interval_ms);
        root
    }

    fn record_error(&self, error: String, seen_hash: Option<u64>) {
        let mut state = self.lock();
        if let Some(hash) = seen_hash {
            state.last_seen_hash = hash;
        }
        state.last_error = Some(error);
        state.error_source_hash = seen_hash;
    }

    fn lock(&self) -> MutexGuard<'_, UiViewerSourceState> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn collect_viewer_nodes(node: &UiNode, path: &str, output: &mut Vec<UiViewerNodeSnapshot>) {
    output.push(UiViewerNodeSnapshot {
        path: path.to_owned(),
        id: node.id.as_str().to_owned(),
        widget_id: node.id.widget_id().0,
        component: node.component.clone(),
        layout: node.layout,
        child_count: node.children.len(),
    });
    for (index, child) in node.children.iter().enumerate() {
        collect_viewer_nodes(child, &format!("{path}.children[{index}]"), output);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiViewerError {
    Read {
        path: PathBuf,
        message: String,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
    Validation {
        path: PathBuf,
        diagnostics: Vec<UiDiagnostic>,
    },
}

impl fmt::Display for UiViewerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, message } => {
                write!(formatter, "cannot read {}: {message}", path.display())
            }
            Self::Parse { path, message } => {
                write!(formatter, "cannot parse {}: {message}", path.display())
            }
            Self::Validation { path, diagnostics } => write!(
                formatter,
                "{} failed UI document validation with {} diagnostic(s)",
                path.display(),
                diagnostics.len()
            ),
        }
    }
}

impl Error for UiViewerError {}

struct SourceText {
    document: String,
    bindings: String,
    hash: u64,
}

fn read_sources(
    document_path: &Path,
    binding_path: Option<&Path>,
) -> Result<SourceText, UiViewerError> {
    let document = fs::read_to_string(document_path).map_err(|error| UiViewerError::Read {
        path: document_path.to_owned(),
        message: error.to_string(),
    })?;
    let bindings = match binding_path {
        Some(path) => fs::read_to_string(path).map_err(|error| UiViewerError::Read {
            path: path.to_owned(),
            message: error.to_string(),
        })?,
        None => "{}".to_owned(),
    };
    let mut hash = fnv1a(document.as_bytes());
    hash ^= fnv1a(bindings.as_bytes()).rotate_left(17);
    Ok(SourceText {
        document,
        bindings,
        hash,
    })
}

fn parse_and_validate_sources(
    document_path: &Path,
    binding_path: Option<&Path>,
    document_source: &str,
    binding_source: &str,
) -> Result<(UiDocument, UiBindingSchema), UiViewerError> {
    let document =
        UiDocument::from_json(document_source).map_err(|error| UiViewerError::Parse {
            path: document_path.to_owned(),
            message: error.to_string(),
        })?;
    let bindings = serde_json::from_str::<UiBindingSchema>(binding_source).map_err(|error| {
        UiViewerError::Parse {
            path: binding_path
                .map(Path::to_owned)
                .unwrap_or_else(|| PathBuf::from("<empty binding schema>")),
            message: error.to_string(),
        }
    })?;
    let report = document.validate(&UiFeatureSet::compiled(), &bindings);
    if report.is_valid() {
        Ok((document, bindings))
    } else {
        Err(UiViewerError::Validation {
            path: document_path.to_owned(),
            diagnostics: report.diagnostics,
        })
    }
}

fn reload_compatibility_report(
    previous: &UiDocument,
    current: &UiDocument,
    from_revision: u64,
    to_revision: u64,
) -> UiViewerReloadReport {
    let mut previous_nodes = BTreeMap::new();
    let mut current_nodes = BTreeMap::new();
    collect_node_components(&previous.root, &mut previous_nodes);
    collect_node_components(&current.root, &mut current_nodes);

    let mut preserved_node_ids = Vec::new();
    let mut state_resets = Vec::new();
    for (node_id, previous_node) in &previous_nodes {
        match current_nodes.get(node_id) {
            Some(current_node) if current_node.state_class == previous_node.state_class => {
                preserved_node_ids.push(node_id.clone());
            }
            Some(current_node) => state_resets.push(UiViewerStateReset {
                node_id: node_id.clone(),
                previous_component: previous_node.component.clone(),
                current_component: Some(current_node.component.clone()),
                reason: if current_node.component == previous_node.component {
                    UiViewerStateResetReason::ComponentConfigurationChanged
                } else {
                    UiViewerStateResetReason::ComponentChanged
                },
            }),
            None => state_resets.push(UiViewerStateReset {
                node_id: node_id.clone(),
                previous_component: previous_node.component.clone(),
                current_component: None,
                reason: UiViewerStateResetReason::NodeRemoved,
            }),
        }
    }
    let added_node_ids = current_nodes
        .keys()
        .filter(|node_id| !previous_nodes.contains_key(*node_id))
        .cloned()
        .collect();

    UiViewerReloadReport {
        from_revision,
        to_revision,
        preserved_node_ids,
        added_node_ids,
        state_resets,
    }
}

#[derive(Clone)]
struct UiViewerNodeCompatibility {
    component: String,
    state_class: String,
}

fn collect_node_components(
    node: &UiNode,
    output: &mut BTreeMap<String, UiViewerNodeCompatibility>,
) {
    output.insert(
        node.id.as_str().to_owned(),
        UiViewerNodeCompatibility {
            component: node.component.clone(),
            state_class: node_state_class(node),
        },
    );
    for child in &node.children {
        collect_node_components(child, output);
    }
}

fn node_state_class(node: &UiNode) -> String {
    if node.component != "textbox" {
        return node.component.clone();
    }
    if let Some(binding) = node.property_bindings.get("multiline") {
        return format!("textbox:multiline-binding:{binding}");
    }
    if node
        .properties
        .get("multiline")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "textbox:multiline".to_owned()
    } else {
        "textbox:singleline".to_owned()
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_directory(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zsui-ui-viewer-{}-{test_name}-{unique}",
            std::process::id()
        ))
    }

    fn write_fixtures(directory: &Path) -> (PathBuf, PathBuf) {
        fs::create_dir_all(directory).expect("fixture directory should be created");
        let document = directory.join("basic.json");
        let bindings = directory.join("basic.bindings.json");
        fs::write(
            &document,
            include_str!("../examples/ui-documents/basic.json"),
        )
        .expect("document fixture should be written");
        fs::write(
            &bindings,
            include_str!("../examples/ui-documents/basic.bindings.json"),
        )
        .expect("binding fixture should be written");
        (document, bindings)
    }

    #[test]
    fn viewer_reloads_valid_changes_without_replacing_the_source() {
        let directory = fixture_directory("reload");
        let (document_path, binding_path) = write_fixtures(&directory);
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        assert_eq!(source.snapshot().revision, 1);

        let updated =
            include_str!("../examples/ui-documents/basic.json").replace("\"Save\"", "\"Save now\"");
        fs::write(&document_path, updated).unwrap();

        assert!(source.refresh());
        let snapshot = source.snapshot();
        assert_eq!(snapshot.revision, 2);
        assert!(snapshot.last_error.is_none());
        let reload = snapshot.last_reload.unwrap();
        assert!(reload.preserves_all_existing_state());
        assert_eq!(reload.preserved_node_ids, ["root", "save-button", "title"]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_reports_removed_and_component_changed_state() {
        let directory = fixture_directory("compatibility");
        let (document_path, binding_path) = write_fixtures(&directory);
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let updated = r#"{
          "schema_version": 1,
          "root": {
            "id": "root",
            "component": "stack",
            "children": [
              {
                "id": "title",
                "component": "button",
                "properties": { "label": "Title" }
              }
            ]
          }
        }"#;
        fs::write(&document_path, updated).unwrap();

        assert!(source.refresh());
        let reload = source.snapshot().last_reload.unwrap();
        assert_eq!(reload.from_revision, 1);
        assert_eq!(reload.to_revision, 2);
        assert_eq!(reload.preserved_node_ids, ["root"]);
        assert_eq!(reload.state_resets.len(), 2);
        assert!(reload.state_resets.iter().any(|reset| {
            reset.node_id == "title"
                && reset.reason == UiViewerStateResetReason::ComponentChanged
                && reset.current_component.as_deref() == Some("button")
        }));
        assert!(reload.state_resets.iter().any(|reset| {
            reset.node_id == "save-button"
                && reset.reason == UiViewerStateResetReason::NodeRemoved
                && reset.current_component.is_none()
        }));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_reports_textbox_control_class_changes() {
        let directory = fixture_directory("textbox-class");
        fs::create_dir_all(&directory).unwrap();
        let document_path = directory.join("textbox.json");
        let binding_path = directory.join("textbox.bindings.json");
        let document = |multiline| {
            format!(
                r#"{{
                  "schema_version": 1,
                  "root": {{
                    "id": "editor",
                    "component": "textbox",
                    "properties": {{ "value": "Text", "multiline": {multiline} }}
                  }}
                }}"#
            )
        };
        fs::write(&document_path, document(true)).unwrap();
        fs::write(&binding_path, "{}").unwrap();
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();

        fs::write(&document_path, document(false)).unwrap();
        assert!(source.refresh());

        let reload = source.snapshot().last_reload.unwrap();
        assert_eq!(reload.state_resets.len(), 1);
        assert_eq!(
            reload.state_resets[0].reason,
            UiViewerStateResetReason::ComponentConfigurationChanged
        );
        assert_eq!(reload.state_resets[0].previous_component, "textbox");
        assert_eq!(
            reload.state_resets[0].current_component.as_deref(),
            Some("textbox")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn native_reload_preserves_compatible_text_state_and_resets_changed_controls() {
        let directory = fixture_directory("native-state");
        fs::create_dir_all(&directory).unwrap();
        let document_path = directory.join("state.json");
        let binding_path = directory.join("state.bindings.json");
        let document = |padding: u32, component: &str| {
            let properties = if component == "textbox" {
                r#""value": "row0\nrow1\nrow2\nrow3\nrow4\nrow5", "multiline": true"#
            } else {
                r#""label": "Replaced""#
            };
            format!(
                r#"{{
                  "schema_version": 1,
                  "root": {{
                    "id": "root",
                    "component": "stack",
                    "layout": {{ "padding": {padding} }},
                    "children": [
                      {{
                        "id": "editor",
                        "component": "{component}",
                        "layout": {{ "height": 52.0 }},
                        "properties": {{ {properties} }}
                      }}
                    ]
                  }}
                }}"#
            )
        };
        fs::write(&document_path, document(16, "textbox")).unwrap();
        fs::write(&binding_path, "{}").unwrap();

        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let live_source = source.clone();
        let surface = crate::Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 220,
        };
        let live_view = crate::view::live_view_runtime(
            UiViewerState::default(),
            move |state| live_source.view(state),
            ui_viewer_update,
            surface,
            crate::Dpi::standard(),
        );
        let editor = crate::ui_document::UiNodeId::new("editor")
            .unwrap()
            .widget_id();
        let target = live_view
            .interaction_plan()
            .focus_target_for_widget(editor)
            .expect("editor should expose a native focus target");
        let mut runtime = crate::native::NativeViewInputRuntime::new(
            surface,
            Some(live_view.interaction_plan()),
            None,
            Some(live_view),
            crate::native::NativeWindowResourcePolicy::default(),
            None,
            None,
            None,
        );
        runtime.dispatch_pointer_click(crate::Point {
            x: target.bounds.x + target.bounds.width / 2,
            y: target.bounds.y + target.bounds.height / 2,
        });
        runtime.dispatch_key(crate::NativeViewKey::Home);
        runtime.dispatch_key_with_shift(crate::NativeViewKey::Right, true);
        runtime.dispatch_key_with_shift(crate::NativeViewKey::Right, true);
        assert_eq!(runtime.text_edit_selection().unwrap().ordered(), (0, 2));
        runtime.dispatch_pointer_scroll(
            crate::Point {
                x: target.bounds.x + target.bounds.width / 2,
                y: target.bounds.y + target.bounds.height / 2,
            },
            Dp::new(48.0),
        );
        let viewport = runtime.text_edit_viewport().unwrap();
        assert!(viewport.0 > 0);

        fs::write(&document_path, document(24, "textbox")).unwrap();
        let compatible = runtime.refresh_transient_view();
        assert_eq!(compatible.focused_widget, Some(editor.0));
        assert_eq!(compatible.text_selection, Some((0, 2)));
        assert_eq!(runtime.text_edit_viewport(), Some(viewport));
        assert!(source
            .snapshot()
            .last_reload
            .unwrap()
            .preserves_all_existing_state());

        fs::write(&document_path, document(24, "button")).unwrap();
        let incompatible = runtime.refresh_transient_view();
        assert!(incompatible.focus_visual_changed);
        assert_eq!(incompatible.focused_widget, None);
        assert_eq!(runtime.text_edit_selection(), None);
        let reload = source.snapshot().last_reload.unwrap();
        assert!(reload.state_resets.iter().any(|reset| {
            reset.node_id == "editor" && reset.reason == UiViewerStateResetReason::ComponentChanged
        }));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_routes_value_actions_through_owned_typed_callbacks() {
        let directory = fixture_directory("value-actions");
        fs::create_dir_all(&directory).unwrap();
        let document_path = directory.join("actions.json");
        let binding_path = directory.join("actions.bindings.json");
        fs::write(
            &document_path,
            r#"{
              "schema_version": 1,
              "root": {
                "id": "root",
                "component": "stack",
                "children": [
                  {
                    "id": "name",
                    "component": "textbox",
                    "property_bindings": { "value": "name" },
                    "action_bindings": { "change": "name_changed" }
                  },
                  {
                    "id": "dark",
                    "component": "toggle",
                    "property_bindings": { "checked": "dark" },
                    "action_bindings": { "toggle": "dark_changed" }
                  },
                  {
                    "id": "volume",
                    "component": "slider",
                    "property_bindings": { "value": "volume" },
                    "action_bindings": { "slide": "volume_changed" }
                  },
                  {
                    "id": "retry-count",
                    "component": "number_box",
                    "properties": {
                      "minimum": 0.0,
                      "maximum": 10.0,
                      "step": 0.5,
                      "large_step": 5.0,
                      "fraction_digits": 1.0
                    },
                    "property_bindings": { "value": "retry_count" },
                    "action_bindings": { "change": "retry_count_changed" }
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        fs::write(
            &binding_path,
            r#"{
              "properties": {
                "name": "string",
                "dark": "boolean",
                "volume": "number",
                "retry_count": "nullable_number"
              },
              "actions": {
                "name_changed": "string",
                "dark_changed": "boolean",
                "volume_changed": "number",
                "retry_count_changed": "nullable_number"
              }
            }"#,
        )
        .unwrap();
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let live_source = source.clone();
        let actions = Arc::new(Mutex::new(Vec::new()));
        let captured_actions = Arc::clone(&actions);
        let surface = crate::Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 240,
        };
        let live_view = crate::view::live_view_runtime(
            UiViewerState::with_properties(BTreeMap::from([
                ("name".to_owned(), Value::String("A".to_owned())),
                ("dark".to_owned(), Value::Bool(false)),
                ("volume".to_owned(), Value::from(10.0)),
                ("retry_count".to_owned(), Value::from(2.5)),
            ])),
            move |state| live_source.view(state),
            move |state, message, cx| {
                let UiViewerMessage::Action(action) = &message;
                captured_actions.lock().unwrap().push(action.clone());
                ui_viewer_update(state, message, cx);
            },
            surface,
            crate::Dpi::standard(),
        );
        let name = crate::ui_document::UiNodeId::new("name")
            .unwrap()
            .widget_id();
        let dark = crate::ui_document::UiNodeId::new("dark")
            .unwrap()
            .widget_id();
        let volume = crate::ui_document::UiNodeId::new("volume")
            .unwrap()
            .widget_id();
        let retry_count = crate::ui_document::UiNodeId::new("retry-count")
            .unwrap()
            .widget_id();

        let text_update = live_view.dispatch_event(&crate::ViewEvent::TextChanged {
            widget: name,
            value: "AB".to_owned(),
        });
        let toggle_update = live_view.dispatch_event(&crate::ViewEvent::Toggled {
            widget: dark,
            checked: true,
        });
        let slider_update = live_view.dispatch_event(&crate::ViewEvent::SliderChanged {
            widget: volume,
            value: 73.0,
        });
        let number_update = live_view.dispatch_event(&crate::ViewEvent::NumberBoxStep {
            widget: retry_count,
            steps: 1,
            large: false,
        });

        assert_eq!(text_update.message_count, 1);
        assert_eq!(toggle_update.message_count, 1);
        assert_eq!(slider_update.message_count, 1);
        assert_eq!(number_update.message_count, 1);
        assert_eq!(live_view.widget_text_value(name).as_deref(), Some("AB"));
        assert_eq!(live_view.widget_checked_value(dark), Some(true));
        assert_eq!(live_view.widget_slider_state(volume).unwrap().0, 73.0);
        let draft_update = live_view.dispatch_event(&crate::ViewEvent::TextChanged {
            widget: retry_count,
            value: String::new(),
        });
        let clear_update = live_view.dispatch_event(&crate::ViewEvent::NumberBoxCommit {
            widget: retry_count,
        });
        assert_eq!(draft_update.message_count, 0);
        assert_eq!(clear_update.message_count, 1);
        let actions = actions.lock().unwrap();
        assert_eq!(actions.len(), 5);
        assert_eq!(actions[0].binding, "name_changed");
        assert_eq!(actions[0].property_binding.as_deref(), Some("name"));
        assert_eq!(actions[0].payload, Value::String("AB".to_owned()));
        assert_eq!(actions[1].binding, "dark_changed");
        assert_eq!(actions[1].payload, Value::Bool(true));
        assert_eq!(actions[2].binding, "volume_changed");
        assert_eq!(actions[2].payload, Value::from(73.0));
        assert_eq!(actions[3].binding, "retry_count_changed");
        assert_eq!(actions[3].property_binding.as_deref(), Some("retry_count"));
        assert_eq!(actions[3].payload, Value::from(3.0));
        assert_eq!(actions[4].binding, "retry_count_changed");
        assert_eq!(actions[4].payload, Value::Null);
        drop(actions);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_preserves_controlled_scroll_offset_across_view_rebuilds() {
        let directory = fixture_directory("controlled-scroll");
        fs::create_dir_all(&directory).unwrap();
        let document_path = directory.join("scroll.json");
        let binding_path = directory.join("scroll.bindings.json");
        fs::write(
            &document_path,
            r#"{
              "schema_version": 1,
              "root": {
                "id": "results-scroll",
                "component": "scroll",
                "properties": { "content_height": 360.0 },
                "property_bindings": { "offset_y": "scroll_offset" },
                "action_bindings": { "scroll": "scroll_changed" },
                "children": [
                  {
                    "id": "results",
                    "component": "stack",
                    "layout": { "height": 360.0 },
                    "children": [
                      {
                        "id": "result-title",
                        "component": "text",
                        "properties": { "text": "Scrollable results" }
                      }
                    ]
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        fs::write(
            &binding_path,
            r#"{
              "properties": { "scroll_offset": "number" },
              "actions": { "scroll_changed": "number" }
            }"#,
        )
        .unwrap();

        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let live_source = source.clone();
        let actions = Arc::new(Mutex::new(Vec::new()));
        let captured_actions = Arc::clone(&actions);
        let live_view = crate::view::live_view_runtime(
            UiViewerState::with_properties(BTreeMap::from([(
                "scroll_offset".to_owned(),
                Value::from(20.0),
            )])),
            move |state| live_source.view(state),
            move |state, message, cx| {
                let UiViewerMessage::Action(action) = &message;
                captured_actions.lock().unwrap().push(action.clone());
                ui_viewer_update(state, message, cx);
            },
            crate::Rect {
                x: 0,
                y: 0,
                width: 400,
                height: 120,
            },
            crate::Dpi::standard(),
        );
        let scroll = crate::ui_document::UiNodeId::new("results-scroll")
            .unwrap()
            .widget_id();

        let first = live_view.dispatch_event(&crate::ViewEvent::ScrollBy {
            widget: scroll,
            delta_y: Dp::new(30.0),
        });
        let second = live_view.dispatch_event(&crate::ViewEvent::ScrollBy {
            widget: scroll,
            delta_y: Dp::new(10.0),
        });

        assert_eq!(first.message_count, 1);
        assert_eq!(second.message_count, 1);
        let actions = actions.lock().unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].binding, "scroll_changed");
        assert_eq!(
            actions[0].property_binding.as_deref(),
            Some("scroll_offset")
        );
        assert_eq!(actions[0].payload, Value::from(50.0));
        assert_eq!(actions[1].payload, Value::from(60.0));
        drop(actions);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_keeps_last_valid_document_after_invalid_edit() {
        let directory = fixture_directory("invalid");
        let (document_path, binding_path) = write_fixtures(&directory);
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        fs::write(&document_path, "{ invalid json").unwrap();

        assert!(!source.refresh());
        let snapshot = source.snapshot();
        assert_eq!(snapshot.revision, 1);
        assert!(snapshot.last_error.is_some());
        assert_eq!(snapshot.node_count, 3);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_recovers_from_a_transient_read_error_without_a_source_edit() {
        let directory = fixture_directory("read-recovery");
        let (document_path, binding_path) = write_fixtures(&directory);
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let hidden_document_path = directory.join("hidden.json");
        fs::rename(&document_path, &hidden_document_path).unwrap();

        assert!(!source.refresh());
        assert!(source.snapshot().last_error.is_some());
        fs::rename(&hidden_document_path, &document_path).unwrap();

        assert!(!source.refresh());
        assert!(source.snapshot().last_error.is_none());
        assert_eq!(source.snapshot().revision, 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn viewer_view_uses_stable_document_ids_and_requests_polling() {
        let directory = fixture_directory("view");
        let (document_path, binding_path) = write_fixtures(&directory);
        let source = UiViewerSource::open(&document_path, Some(&binding_path)).unwrap();
        let state = UiViewerState::with_properties(BTreeMap::from([(
            "window_title".to_owned(),
            Value::String("Native Viewer".to_owned()),
        )]));

        let view = source.view(&state);

        assert_eq!(
            view.id,
            Some(
                crate::ui_document::UiNodeId::new("root")
                    .unwrap()
                    .widget_id()
            )
        );
        assert_eq!(
            view.background_poll_interval_ms(),
            Some(ZSUI_UI_VIEWER_DEFAULT_POLL_INTERVAL_MS)
        );
        let snapshot = source.snapshot();
        assert_eq!(snapshot.document_schema_version, 1);
        assert_eq!(snapshot.node_count, 3);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| (
                    node.path.as_str(),
                    node.id.as_str(),
                    node.component.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("$.root", "root", "stack"),
                ("$.root.children[0]", "title", "text"),
                ("$.root.children[1]", "save-button", "button"),
            ]
        );
        assert_eq!(snapshot.nodes[0].widget_id, view.id.unwrap().0);
        assert_eq!(
            serde_json::to_vec(&snapshot).unwrap(),
            serde_json::to_vec(&source.snapshot()).unwrap()
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
