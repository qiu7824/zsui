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

use crate::ui_document::{UiAxis, UiBindingSchema, UiDiagnostic, UiDocument, UiFeatureSet, UiNode};
use crate::{
    button, checkbox, column, progress_bar, radio_button, row, slider, styled_text, text,
    text_editor, textbox, toggle, toggle_button, AppCx, ColorRole, Dp, ProgressRange,
    SemanticTextStyle, SliderRange, TextRole, ThemeColorToken, ViewNode,
};

pub const ZSUI_UI_VIEWER_DEFAULT_POLL_INTERVAL_MS: u64 = 250;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerAction {
    pub node_id: String,
    pub binding: String,
    pub payload: Value,
}

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
            if state.actions.len() == MAX_ACTION_HISTORY {
                state.actions.remove(0);
            }
            state.actions.push(action);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiViewerSourceSnapshot {
    pub revision: u64,
    pub document_path: PathBuf,
    pub binding_path: Option<PathBuf>,
    pub node_count: usize,
    pub last_error: Option<String>,
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
                state.last_seen_hash = sources.hash;
                state.document = Arc::new(document);
                state.bindings = bindings;
                state.revision = state.revision.saturating_add(1);
                state.last_error = None;
                state.error_source_hash = None;
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
        UiViewerSourceSnapshot {
            revision: state.revision,
            document_path: state.document_path.clone(),
            binding_path: state.binding_path.clone(),
            node_count: count_nodes(&state.document.root),
            last_error: state.last_error.clone(),
        }
    }

    pub fn view(&self, viewer_state: &UiViewerState) -> ViewNode<UiViewerMessage> {
        self.refresh();
        let (document, last_error) = {
            let state = self.lock();
            (Arc::clone(&state.document), state.last_error.clone())
        };
        let content = compile_node(&document.root, viewer_state);
        let mut root = if let Some(error) = last_error {
            column([text(format!("UI document reload error: {error}")), content]).gap(Dp::new(8.0))
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

fn compile_node(node: &UiNode, state: &UiViewerState) -> ViewNode<UiViewerMessage> {
    let children = node
        .children
        .iter()
        .map(|child| compile_node(child, state))
        .collect::<Vec<_>>();
    let mut view = match node.component.as_str() {
        "stack" => match node.layout.direction.unwrap_or(UiAxis::Vertical) {
            UiAxis::Horizontal => row(children),
            UiAxis::Vertical => column(children),
        },
        "border" => column(children),
        "text" => {
            let value = string_property(node, state, "text", "");
            styled_text(value, semantic_text_style(node))
        }
        "button" => {
            let mut control = button(string_property(node, state, "label", "Button"))
                .enabled(bool_property(node, state, "enabled", true));
            if let Some(binding) = node.action_bindings.get("click") {
                control = control.on_click(UiViewerMessage::Action(UiViewerAction {
                    node_id: node.id.as_str().to_owned(),
                    binding: binding.clone(),
                    payload: Value::Null,
                }));
            }
            control
        }
        "toggle_button" => toggle_button(
            string_property(node, state, "label", "Toggle"),
            bool_property(node, state, "checked", false),
        ),
        "checkbox" => checkbox(
            string_property(node, state, "label", "Check box"),
            bool_property(node, state, "checked", false),
        ),
        "toggle" => toggle(bool_property(node, state, "checked", false)),
        "textbox" if bool_property(node, state, "multiline", false) => {
            text_editor(string_property(node, state, "value", ""))
        }
        "textbox" => textbox(string_property(node, state, "value", "")),
        "radio_button" => {
            let mut control = radio_button(
                string_property(node, state, "label", "Option"),
                bool_property(node, state, "selected", false),
            );
            if let Some(binding) = node.action_bindings.get("choose") {
                control = control.on_choose(UiViewerMessage::Action(UiViewerAction {
                    node_id: node.id.as_str().to_owned(),
                    binding: binding.clone(),
                    payload: Value::Null,
                }));
            }
            control
        }
        "slider" => slider(
            number_property(node, state, "value", 0.0) as f32,
            SliderRange::new(0.0, 100.0),
        ),
        "progress_bar" => progress_bar(
            number_property(node, state, "value", 0.0) as f32,
            ProgressRange::new(0.0, 100.0),
        ),
        _ => text(format!(
            "Unsupported UiDocument component: {}",
            node.component
        )),
    };
    view = view.id(node.id.widget_id());
    apply_layout(view, node)
}

fn apply_layout(mut view: ViewNode<UiViewerMessage>, node: &UiNode) -> ViewNode<UiViewerMessage> {
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

fn property_value(node: &UiNode, state: &UiViewerState, property: &str) -> Option<Value> {
    if let Some(value) = node.properties.get(property) {
        return Some(value.clone());
    }
    if let Some(binding) = node.property_bindings.get(property) {
        return state
            .properties
            .get(binding)
            .cloned()
            .or_else(|| Some(Value::String(format!("{{binding:{binding}}}"))));
    }
    node.localization
        .get(property)
        .map(|key| Value::String(format!("{{message:{key}}}")))
}

fn string_property(node: &UiNode, state: &UiViewerState, property: &str, fallback: &str) -> String {
    property_value(node, state, property)
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| fallback.to_owned())
}

fn bool_property(node: &UiNode, state: &UiViewerState, property: &str, fallback: bool) -> bool {
    property_value(node, state, property)
        .and_then(|value| value.as_bool())
        .unwrap_or(fallback)
}

fn number_property(node: &UiNode, state: &UiViewerState, property: &str, fallback: f64) -> f64 {
    property_value(node, state, property)
        .and_then(|value| value.as_f64())
        .unwrap_or(fallback)
}

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

fn count_nodes(node: &UiNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
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
        assert_eq!(source.snapshot().revision, 2);
        assert!(source.snapshot().last_error.is_none());
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
        fs::remove_dir_all(directory).unwrap();
    }
}
