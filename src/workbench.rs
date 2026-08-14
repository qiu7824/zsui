use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, Point, Rect, SemanticTextStyle,
    TextRole, TextWeight, TextWrap, VerticalAlign, ZsIcon,
};

pub const ZS_WORKBENCH_BASE_SIDEBAR_WIDTH: i32 = 272;
pub const ZS_WORKBENCH_COLLAPSED_SIDEBAR_WIDTH: i32 = 56;
pub const ZS_WORKBENCH_TOP_BAR_HEIGHT: i32 = 64;
pub const ZS_WORKBENCH_COMPOSER_HEIGHT: i32 = 120;
pub const ZS_WORKBENCH_INSPECTOR_WIDTH: i32 = 336;
pub const ZS_WORKBENCH_CONTENT_MAX_WIDTH: i32 = 760;
pub const ZS_WORKBENCH_FLOATING_RADIUS: i32 = 12;

fn scale(value: i32, dpi: Dpi) -> i32 {
    ((value as f32) * dpi.scale_factor()).round() as i32
}

fn scale_dp(value: Dp, dpi: Dpi) -> i32 {
    value.to_px(dpi).round_i32()
}

fn workbench_style_tokens() -> crate::platform_component_profile::PlatformStyleTokenProfile {
    crate::platform_component_profile::PlatformComponentProfile::current().style_tokens
}

fn workbench_profile() -> crate::platform_component_profile::PlatformWorkbenchProfile {
    crate::platform_component_profile::PlatformWorkbenchProfile::for_platform(
        crate::ZsPlatformStyle::current(),
    )
}

fn workbench_floating_radius(dpi: Dpi) -> i32 {
    scale_dp(workbench_profile().floating_radius, dpi)
}

pub type ZsWorkbenchIcon = ZsIcon;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchActionSpec {
    pub id: String,
    pub label: String,
    pub icon: ZsWorkbenchIcon,
    pub enabled: bool,
    pub selected: bool,
}

impl ZsWorkbenchActionSpec {
    pub fn new(id: impl Into<String>, label: impl Into<String>, icon: ZsWorkbenchIcon) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            enabled: true,
            selected: false,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchConversationSpec {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub selected: bool,
    pub pinned: bool,
    pub unread: bool,
}

impl ZsWorkbenchConversationSpec {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            selected: false,
            pinned: false,
            unread: false,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchConversationGroupSpec {
    pub id: String,
    pub label: String,
    pub conversations: Vec<ZsWorkbenchConversationSpec>,
}

impl ZsWorkbenchConversationGroupSpec {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            conversations: Vec::new(),
        }
    }

    pub fn conversation(mut self, conversation: ZsWorkbenchConversationSpec) -> Self {
        self.conversations.push(conversation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchSidebarSpec {
    pub title: String,
    pub collapsed: bool,
    #[serde(default)]
    pub force_expanded: bool,
    #[serde(default)]
    pub scroll_y: i32,
    pub primary_actions: Vec<ZsWorkbenchActionSpec>,
    pub groups: Vec<ZsWorkbenchConversationGroupSpec>,
    pub footer_actions: Vec<ZsWorkbenchActionSpec>,
}

impl ZsWorkbenchSidebarSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            collapsed: false,
            force_expanded: false,
            scroll_y: 0,
            primary_actions: Vec::new(),
            groups: Vec::new(),
            footer_actions: Vec::new(),
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        if collapsed {
            self.force_expanded = false;
        }
        self
    }

    pub fn force_expanded(mut self, force_expanded: bool) -> Self {
        self.force_expanded = force_expanded;
        if force_expanded {
            self.collapsed = false;
        }
        self
    }

    pub fn scroll_y(mut self, scroll_y: i32) -> Self {
        self.scroll_y = scroll_y.max(0);
        self
    }

    pub fn primary_action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.primary_actions.push(action);
        self
    }

    pub fn group(mut self, group: ZsWorkbenchConversationGroupSpec) -> Self {
        self.groups.push(group);
        self
    }

    pub fn footer_action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.footer_actions.push(action);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchMessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchToolStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl ZsWorkbenchToolStatus {
    /// Returns the legacy English status copy for applications that explicitly
    /// choose it. Workbench painting does not inject this text automatically.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Succeeded => "Completed",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchNoticeLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchContentBlock {
    Paragraph {
        text: String,
    },
    Code {
        language: String,
        code: String,
    },
    Tool {
        title: String,
        summary: String,
        status: ZsWorkbenchToolStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status_label: Option<String>,
    },
    Notice {
        text: String,
        level: ZsWorkbenchNoticeLevel,
    },
}

impl ZsWorkbenchContentBlock {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::Paragraph { text: text.into() }
    }

    pub fn code(language: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Code {
            language: language.into(),
            code: code.into(),
        }
    }

    pub fn tool(
        title: impl Into<String>,
        summary: impl Into<String>,
        status: ZsWorkbenchToolStatus,
    ) -> Self {
        Self::Tool {
            title: title.into(),
            summary: summary.into(),
            status,
            status_label: None,
        }
    }

    /// Creates a tool block with application-owned localized status copy.
    pub fn tool_with_status_label(
        title: impl Into<String>,
        summary: impl Into<String>,
        status: ZsWorkbenchToolStatus,
        status_label: impl Into<String>,
    ) -> Self {
        Self::Tool {
            title: title.into(),
            summary: summary.into(),
            status,
            status_label: Some(status_label.into()),
        }
    }

    pub fn notice(text: impl Into<String>, level: ZsWorkbenchNoticeLevel) -> Self {
        Self::Notice {
            text: text.into(),
            level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchMessageSpec {
    pub id: String,
    pub role: ZsWorkbenchMessageRole,
    pub author: Option<String>,
    pub blocks: Vec<ZsWorkbenchContentBlock>,
    pub actions: Vec<ZsWorkbenchActionSpec>,
    pub streaming: bool,
}

impl ZsWorkbenchMessageSpec {
    pub fn new(id: impl Into<String>, role: ZsWorkbenchMessageRole) -> Self {
        Self {
            id: id.into(),
            role,
            author: None,
            blocks: Vec::new(),
            actions: Vec::new(),
            streaming: false,
        }
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn block(mut self, block: ZsWorkbenchContentBlock) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.actions.push(action);
        self
    }

    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }
}

/// Application-owned state for the scrollable message region of a
/// [`ZsWorkbenchShellSpec`].
///
/// The timeline is a typed child contract instead of a second window or a
/// platform widget handle. ZSUI retains stable message IDs and lets the
/// target backend choose native typography, spacing and scrollbar metrics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsMessageTimelineSpec {
    pub messages: Vec<ZsWorkbenchMessageSpec>,
    pub scroll_y: i32,
}

impl ZsMessageTimelineSpec {
    pub const fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_y: 0,
        }
    }

    pub fn message(mut self, message: ZsWorkbenchMessageSpec) -> Self {
        self.messages.push(message);
        self
    }

    pub fn messages(mut self, messages: impl IntoIterator<Item = ZsWorkbenchMessageSpec>) -> Self {
        self.messages.extend(messages);
        self
    }

    pub fn scroll_y(mut self, scroll_y: i32) -> Self {
        self.scroll_y = scroll_y.max(0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchComposerSpec {
    pub draft: String,
    pub placeholder: String,
    pub mode_label: Option<String>,
    pub model_label: Option<String>,
    pub busy: bool,
    pub actions: Vec<ZsWorkbenchActionSpec>,
}

impl ZsWorkbenchComposerSpec {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            draft: String::new(),
            placeholder: placeholder.into(),
            mode_label: None,
            model_label: None,
            busy: false,
            actions: Vec::new(),
        }
    }

    pub fn draft(mut self, draft: impl Into<String>) -> Self {
        self.draft = draft.into();
        self
    }

    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode_label = Some(mode.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model_label = Some(model.into());
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.actions.push(action);
        self
    }
}

/// Public component name for the typed composer child of a workbench shell.
pub type ZsComposerSpec = ZsWorkbenchComposerSpec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchInspectorSpec {
    pub title: String,
    pub selected_tab_id: Option<String>,
    pub tabs: Vec<ZsWorkbenchActionSpec>,
    pub body: String,
    #[serde(default)]
    pub scroll_y: i32,
}

impl ZsWorkbenchInspectorSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            selected_tab_id: None,
            tabs: Vec::new(),
            body: String::new(),
            scroll_y: 0,
        }
    }

    pub fn selected_tab(mut self, id: impl Into<String>) -> Self {
        self.selected_tab_id = Some(id.into());
        self
    }

    pub fn tab(mut self, tab: ZsWorkbenchActionSpec) -> Self {
        self.tabs.push(tab);
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn scroll_y(mut self, scroll_y: i32) -> Self {
        self.scroll_y = scroll_y.max(0);
        self
    }
}

/// Public component name for the typed inspector child of a workbench shell.
pub type ZsInspectorPanelSpec = ZsWorkbenchInspectorSpec;

/// A reusable workbench composition assembled from explicit child component
/// contracts.
///
/// `MessageTimeline`, `Composer` and `InspectorPanel` remain independently
/// configurable application state, while the shell owns their relationship,
/// adaptive layout and cross-region keyboard/focus behavior. This keeps one
/// Rust declaration across Win32, AppKit and Linux without exposing target
/// objects or duplicating a platform-specific component tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchShellSpec {
    pub title: String,
    pub subtitle: Option<String>,
    pub sidebar: ZsWorkbenchSidebarSpec,
    pub toolbar_actions: Vec<ZsWorkbenchActionSpec>,
    pub timeline: ZsMessageTimelineSpec,
    pub composer: ZsComposerSpec,
    pub inspector: Option<ZsInspectorPanelSpec>,
}

impl ZsWorkbenchShellSpec {
    pub fn new(
        title: impl Into<String>,
        sidebar: ZsWorkbenchSidebarSpec,
        composer: ZsComposerSpec,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            sidebar,
            toolbar_actions: Vec::new(),
            timeline: ZsMessageTimelineSpec::new(),
            composer,
            inspector: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn toolbar_action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.toolbar_actions.push(action);
        self
    }

    pub fn toolbar_actions(
        mut self,
        actions: impl IntoIterator<Item = ZsWorkbenchActionSpec>,
    ) -> Self {
        self.toolbar_actions.extend(actions);
        self
    }

    pub fn timeline(mut self, timeline: ZsMessageTimelineSpec) -> Self {
        self.timeline = timeline;
        self
    }

    pub fn inspector(mut self, inspector: ZsInspectorPanelSpec) -> Self {
        self.inspector = Some(inspector);
        self
    }

    pub fn into_workbench(self) -> ZsWorkbenchSpec {
        self.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchSpec {
    pub title: String,
    pub subtitle: Option<String>,
    pub sidebar: ZsWorkbenchSidebarSpec,
    pub toolbar_actions: Vec<ZsWorkbenchActionSpec>,
    pub messages: Vec<ZsWorkbenchMessageSpec>,
    pub composer: ZsWorkbenchComposerSpec,
    pub inspector: Option<ZsWorkbenchInspectorSpec>,
    pub message_scroll_y: i32,
}

impl ZsWorkbenchSpec {
    pub fn new(
        title: impl Into<String>,
        sidebar: ZsWorkbenchSidebarSpec,
        composer: ZsWorkbenchComposerSpec,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            sidebar,
            toolbar_actions: Vec::new(),
            messages: Vec::new(),
            composer,
            inspector: None,
            message_scroll_y: 0,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn toolbar_action(mut self, action: ZsWorkbenchActionSpec) -> Self {
        self.toolbar_actions.push(action);
        self
    }

    pub fn message(mut self, message: ZsWorkbenchMessageSpec) -> Self {
        self.messages.push(message);
        self
    }

    pub fn inspector(mut self, inspector: ZsWorkbenchInspectorSpec) -> Self {
        self.inspector = Some(inspector);
        self
    }

    pub fn message_scroll_y(mut self, message_scroll_y: i32) -> Self {
        self.message_scroll_y = message_scroll_y.max(0);
        self
    }

    pub fn layout(&self, surface: Rect, dpi: Dpi) -> ZsWorkbenchLayoutPlan {
        zs_workbench_layout(self, surface, dpi)
    }

    pub(crate) fn layout_with_text_measurements(
        &self,
        surface: Rect,
        dpi: Dpi,
        typography_scale: f32,
        measurements: &crate::view::ViewTextMeasurements,
    ) -> ZsWorkbenchLayoutPlan {
        zs_workbench_layout_with_text_measurements(
            self,
            surface,
            dpi,
            typography_scale,
            measurements,
        )
    }

    pub fn native_draw_plan(&self, surface: Rect, dpi: Dpi) -> NativeDrawPlan {
        let layout = self.layout(surface, dpi);
        zs_workbench_native_draw_plan(self, &layout)
    }
}

impl From<ZsWorkbenchShellSpec> for ZsWorkbenchSpec {
    fn from(shell: ZsWorkbenchShellSpec) -> Self {
        Self {
            title: shell.title,
            subtitle: shell.subtitle,
            sidebar: shell.sidebar,
            toolbar_actions: shell.toolbar_actions,
            messages: shell.timeline.messages,
            composer: shell.composer,
            inspector: shell.inspector,
            message_scroll_y: shell.timeline.scroll_y.max(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchLayoutMetrics {
    pub surface: Rect,
    pub sidebar: Rect,
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(default)]
    pub sidebar_auto_collapsed: bool,
    #[serde(default)]
    pub sidebar_list_viewport: Option<Rect>,
    pub top_bar: Rect,
    pub timeline: Rect,
    pub composer_band: Rect,
    pub composer: Rect,
    pub inspector: Option<Rect>,
    pub content: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchRegionKind {
    SidebarToggle,
    SidebarViewport,
    SidebarAction,
    Conversation,
    ToolbarAction,
    MessageAction,
    ComposerInput,
    ComposerAction,
    Submit,
    Stop,
    InspectorTab,
    InspectorViewport,
    Timeline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchLayoutRegion {
    pub kind: ZsWorkbenchRegionKind,
    pub id: String,
    pub bounds: Rect,
    pub enabled: bool,
}

impl ZsWorkbenchLayoutRegion {
    pub fn contains(&self, point: Point) -> bool {
        self.bounds.contains(point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchBlockLayout {
    pub block_index: usize,
    pub bounds: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchMessageLayout {
    pub message_index: usize,
    pub bounds: Rect,
    pub content_bounds: Rect,
    pub blocks: Vec<ZsWorkbenchBlockLayout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsWorkbenchLayoutPlan {
    pub dpi: Dpi,
    pub metrics: ZsWorkbenchLayoutMetrics,
    pub regions: Vec<ZsWorkbenchLayoutRegion>,
    pub messages: Vec<ZsWorkbenchMessageLayout>,
    #[serde(default)]
    pub sidebar_content_height: i32,
    #[serde(default)]
    pub sidebar_scroll_y: i32,
    #[serde(default)]
    pub sidebar_scroll_max: i32,
    #[serde(default)]
    pub sidebar_scrollbar: Option<ZsWorkbenchScrollbarGeometry>,
    #[serde(default)]
    pub composer_mode_label_bounds: Option<Rect>,
    #[serde(default)]
    pub composer_model_label_bounds: Option<Rect>,
    #[serde(default)]
    pub inspector_body_bounds: Option<Rect>,
    #[serde(default)]
    pub inspector_body_viewport: Option<Rect>,
    #[serde(default)]
    pub inspector_body_content_height: i32,
    #[serde(default)]
    pub inspector_scroll_y: i32,
    #[serde(default)]
    pub inspector_scroll_max: i32,
    #[serde(default)]
    pub inspector_scrollbar: Option<ZsWorkbenchScrollbarGeometry>,
    pub message_content_height: i32,
    pub message_scroll_y: i32,
    pub message_scroll_max: i32,
}

/// Shared scrollbar geometry for Workbench-owned scroll surfaces.
///
/// The track is the wheel hit surface and the thumb is the painted position
/// indicator. Workbench does not currently expose thumb dragging as a public
/// interaction contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchScrollbarGeometry {
    pub track: Rect,
    pub thumb: Rect,
}

impl ZsWorkbenchLayoutPlan {
    pub fn region_at(&self, point: Point) -> Option<&ZsWorkbenchLayoutRegion> {
        self.regions
            .iter()
            .rev()
            .find(|region| region.enabled && region.contains(point))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsWorkbenchInteractionEvent {
    ToggleSidebar,
    SetSidebarExpanded {
        expanded: bool,
    },
    InvokeSidebarAction {
        action_id: String,
    },
    SelectConversation {
        conversation_id: String,
    },
    InvokeToolbarAction {
        action_id: String,
    },
    InvokeMessageAction {
        message_id: String,
        action_id: String,
    },
    FocusComposer,
    InvokeComposerAction {
        action_id: String,
    },
    Submit,
    Stop,
    SelectInspectorTab {
        tab_id: String,
    },
    ChangeComposerDraft {
        draft: String,
    },
    ScrollSidebar {
        offset_y: i32,
    },
    ScrollInspector {
        offset_y: i32,
    },
    ScrollMessages {
        offset_y: i32,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsWorkbenchInteractionUpdate {
    pub redraw: bool,
    pub events: Vec<ZsWorkbenchInteractionEvent>,
}

pub fn zs_workbench_event_for_region(
    region: &ZsWorkbenchLayoutRegion,
) -> Option<ZsWorkbenchInteractionEvent> {
    match region.kind {
        ZsWorkbenchRegionKind::SidebarToggle => {
            if region.id.ends_with(".expand") {
                Some(ZsWorkbenchInteractionEvent::SetSidebarExpanded { expanded: true })
            } else if region.id.ends_with(".collapse") {
                Some(ZsWorkbenchInteractionEvent::SetSidebarExpanded { expanded: false })
            } else {
                Some(ZsWorkbenchInteractionEvent::ToggleSidebar)
            }
        }
        ZsWorkbenchRegionKind::SidebarViewport => None,
        ZsWorkbenchRegionKind::SidebarAction => {
            Some(ZsWorkbenchInteractionEvent::InvokeSidebarAction {
                action_id: region.id.clone(),
            })
        }
        ZsWorkbenchRegionKind::Conversation => {
            Some(ZsWorkbenchInteractionEvent::SelectConversation {
                conversation_id: region.id.clone(),
            })
        }
        ZsWorkbenchRegionKind::ToolbarAction => {
            Some(ZsWorkbenchInteractionEvent::InvokeToolbarAction {
                action_id: region.id.clone(),
            })
        }
        ZsWorkbenchRegionKind::MessageAction => {
            let (message_id, action_id) = region.id.split_once(':')?;
            Some(ZsWorkbenchInteractionEvent::InvokeMessageAction {
                message_id: message_id.to_string(),
                action_id: action_id.to_string(),
            })
        }
        ZsWorkbenchRegionKind::ComposerInput => Some(ZsWorkbenchInteractionEvent::FocusComposer),
        ZsWorkbenchRegionKind::ComposerAction => {
            Some(ZsWorkbenchInteractionEvent::InvokeComposerAction {
                action_id: region.id.clone(),
            })
        }
        ZsWorkbenchRegionKind::Submit => Some(ZsWorkbenchInteractionEvent::Submit),
        ZsWorkbenchRegionKind::Stop => Some(ZsWorkbenchInteractionEvent::Stop),
        ZsWorkbenchRegionKind::InspectorTab => {
            Some(ZsWorkbenchInteractionEvent::SelectInspectorTab {
                tab_id: region.id.clone(),
            })
        }
        ZsWorkbenchRegionKind::InspectorViewport => None,
        ZsWorkbenchRegionKind::Timeline => None,
    }
}

pub(crate) fn zs_workbench_region_widget_id(
    parent: crate::WidgetId,
    region: &ZsWorkbenchLayoutRegion,
) -> crate::WidgetId {
    let local_kind = match region.kind {
        ZsWorkbenchRegionKind::SidebarToggle => 1_u64,
        ZsWorkbenchRegionKind::SidebarViewport => 12,
        ZsWorkbenchRegionKind::SidebarAction => 2,
        ZsWorkbenchRegionKind::Conversation => 3,
        ZsWorkbenchRegionKind::ToolbarAction => 4,
        ZsWorkbenchRegionKind::MessageAction => 5,
        ZsWorkbenchRegionKind::ComposerInput => 6,
        ZsWorkbenchRegionKind::ComposerAction => 7,
        ZsWorkbenchRegionKind::Submit => 8,
        ZsWorkbenchRegionKind::Stop => 9,
        ZsWorkbenchRegionKind::InspectorTab => 10,
        ZsWorkbenchRegionKind::InspectorViewport => 13,
        ZsWorkbenchRegionKind::Timeline => 11,
    };
    zs_workbench_named_widget_id(parent, local_kind, &region.id)
}

pub(crate) fn zs_workbench_named_widget_id(
    parent: crate::WidgetId,
    local_kind: u64,
    id: &str,
) -> crate::WidgetId {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64 ^ local_kind;
    for byte in id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    crate::WidgetId::synthetic_child(parent, hash)
}

pub(crate) fn zs_workbench_apply_interaction(
    spec: &mut ZsWorkbenchSpec,
    event: &ZsWorkbenchInteractionEvent,
) -> bool {
    match event {
        ZsWorkbenchInteractionEvent::ToggleSidebar => {
            spec.sidebar.collapsed = !spec.sidebar.collapsed;
            spec.sidebar.force_expanded = !spec.sidebar.collapsed;
            true
        }
        ZsWorkbenchInteractionEvent::SetSidebarExpanded { expanded } => {
            spec.sidebar.collapsed = !expanded;
            spec.sidebar.force_expanded = *expanded;
            true
        }
        ZsWorkbenchInteractionEvent::SelectConversation { conversation_id } => {
            let mut changed = false;
            for conversation in spec
                .sidebar
                .groups
                .iter_mut()
                .flat_map(|group| group.conversations.iter_mut())
            {
                let selected = conversation.id == *conversation_id;
                changed |= conversation.selected != selected;
                conversation.selected = selected;
            }
            changed
        }
        ZsWorkbenchInteractionEvent::SelectInspectorTab { tab_id } => spec
            .inspector
            .as_mut()
            .map(|inspector| {
                let changed = inspector.selected_tab_id.as_deref() != Some(tab_id.as_str());
                inspector.selected_tab_id = Some(tab_id.clone());
                changed
            })
            .unwrap_or(false),
        ZsWorkbenchInteractionEvent::ChangeComposerDraft { draft } => {
            if spec.composer.draft == *draft {
                false
            } else {
                spec.composer.draft.clone_from(draft);
                true
            }
        }
        ZsWorkbenchInteractionEvent::ScrollSidebar { offset_y } => {
            let next = (*offset_y).max(0);
            if spec.sidebar.scroll_y == next {
                false
            } else {
                spec.sidebar.scroll_y = next;
                true
            }
        }
        ZsWorkbenchInteractionEvent::ScrollInspector { offset_y } => spec
            .inspector
            .as_mut()
            .map(|inspector| {
                let next = (*offset_y).max(0);
                if inspector.scroll_y == next {
                    false
                } else {
                    inspector.scroll_y = next;
                    true
                }
            })
            .unwrap_or(false),
        ZsWorkbenchInteractionEvent::ScrollMessages { offset_y } => {
            let next = (*offset_y).max(0);
            if spec.message_scroll_y == next {
                false
            } else {
                spec.message_scroll_y = next;
                true
            }
        }
        _ => false,
    }
}

pub fn zs_workbench_layout(
    spec: &ZsWorkbenchSpec,
    surface: Rect,
    dpi: Dpi,
) -> ZsWorkbenchLayoutPlan {
    zs_workbench_layout_internal(spec, surface, dpi, 1.0, None)
}

pub(crate) fn zs_workbench_layout_with_text_measurements(
    spec: &ZsWorkbenchSpec,
    surface: Rect,
    dpi: Dpi,
    typography_scale: f32,
    measurements: &crate::view::ViewTextMeasurements,
) -> ZsWorkbenchLayoutPlan {
    zs_workbench_layout_internal(spec, surface, dpi, typography_scale, Some(measurements))
}

fn zs_workbench_layout_internal(
    spec: &ZsWorkbenchSpec,
    surface: Rect,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> ZsWorkbenchLayoutPlan {
    let profile = workbench_profile();
    let expanded_sidebar_width = scale_dp(profile.sidebar_width, dpi);
    let automatic_collapse_breakpoint = scale_dp(profile.inspector_breakpoint, dpi) * 3 / 4;
    let sidebar_auto_collapsed = !spec.sidebar.collapsed
        && !spec.sidebar.force_expanded
        && surface.width.max(0) < automatic_collapse_breakpoint.max(expanded_sidebar_width);
    let sidebar_collapsed = spec.sidebar.collapsed || sidebar_auto_collapsed;
    let sidebar_width = scale_dp(
        if sidebar_collapsed {
            profile.collapsed_sidebar_width
        } else {
            profile.sidebar_width
        },
        dpi,
    )
    .min(surface.width.max(0));
    let inspector_width = if spec.inspector.is_some()
        && surface.width >= scale_dp(profile.inspector_breakpoint, dpi)
    {
        scale_dp(profile.inspector_width, dpi).min((surface.width / 3).max(0))
    } else {
        0
    };
    let main_x = surface.x + sidebar_width;
    let main_width = (surface.width - sidebar_width - inspector_width).max(0);
    let main_right = main_x + main_width;
    let content_horizontal_inset = scale_dp(profile.content_horizontal_inset, dpi);
    let content_width = main_width
        .saturating_sub(content_horizontal_inset.saturating_mul(2))
        .min(scale_dp(profile.content_max_width, dpi))
        .max(0);
    let top_height = scale_dp(profile.top_bar_height, dpi).min(surface.height.max(0));
    let composer_height = composer_band_height(
        spec,
        content_width,
        dpi,
        typography_scale,
        measurements,
        profile,
        surface.height.max(0),
        top_height,
    );
    let inspector = (inspector_width > 0).then_some(Rect {
        x: main_right,
        y: surface.y,
        width: inspector_width,
        height: surface.height,
    });
    let top_bar = Rect {
        x: main_x,
        y: surface.y,
        width: main_width,
        height: top_height,
    };
    let composer_band = Rect {
        x: main_x,
        y: surface.y + surface.height - composer_height,
        width: main_width,
        height: composer_height,
    };
    let timeline = Rect {
        x: main_x,
        y: surface.y + top_height,
        width: main_width,
        height: (surface.height - top_height - composer_height).max(0),
    };
    let content = Rect {
        x: main_x + (main_width - content_width) / 2,
        y: timeline.y,
        width: content_width,
        height: timeline.height,
    };
    let composer_inset = scale_dp(profile.composer_vertical_inset, dpi);
    let composer = Rect {
        x: content.x,
        y: composer_band.y + composer_inset,
        width: content.width,
        height: (composer_band.height - composer_inset * 2).max(0),
    };
    let sidebar = Rect {
        x: surface.x,
        y: surface.y,
        width: sidebar_width,
        height: surface.height,
    };
    let sidebar_list = sidebar_list_geometry(spec, sidebar, sidebar_collapsed, dpi);
    let metrics = ZsWorkbenchLayoutMetrics {
        surface,
        sidebar,
        sidebar_collapsed,
        sidebar_auto_collapsed,
        sidebar_list_viewport: (!sidebar_collapsed).then_some(sidebar_list.viewport),
        top_bar,
        timeline,
        composer_band,
        composer,
        inspector,
        content,
    };
    let composer_labels = composer_label_layout(spec, &metrics, dpi, measurements);
    let inspector_body =
        inspector_body_geometry(spec, &metrics, dpi, typography_scale, measurements);
    let sidebar_scrollbar = workbench_scrollbar_geometry(
        sidebar_list.viewport,
        sidebar_list.content_height,
        sidebar_list.scroll_y,
        sidebar_list.scroll_max,
        sidebar.x + sidebar.width - scale(6, dpi),
        dpi,
    );
    let inspector_scrollbar = inspector_body.viewport.and_then(|viewport| {
        let body = inspector_body.body?;
        workbench_scrollbar_geometry(
            viewport,
            inspector_body.content_height,
            inspector_body.scroll_y,
            inspector_body.scroll_max,
            body.x + body.width - scale(6, dpi),
            dpi,
        )
    });

    let mut regions = Vec::new();
    layout_sidebar_regions(spec, &metrics, &sidebar_list, dpi, &mut regions);
    layout_toolbar_regions(spec, &metrics, dpi, &mut regions);
    layout_composer_regions(
        spec,
        &metrics,
        &composer_labels,
        dpi,
        measurements,
        &mut regions,
    );
    layout_inspector_regions(
        spec,
        &metrics,
        &inspector_body,
        dpi,
        measurements,
        &mut regions,
    );
    regions.push(ZsWorkbenchLayoutRegion {
        kind: ZsWorkbenchRegionKind::Timeline,
        id: "timeline".to_string(),
        bounds: timeline,
        enabled: true,
    });

    let gap = scale_dp(profile.message_gap, dpi);
    let top_padding = scale_dp(profile.message_vertical_inset, dpi);
    let message_heights: Vec<_> = spec
        .messages
        .iter()
        .map(|message| message_height(message, content.width, dpi, typography_scale, measurements))
        .collect();
    let message_content_height = message_heights
        .iter()
        .copied()
        .fold(top_padding.saturating_mul(2), i32::saturating_add)
        .saturating_add(gap.saturating_mul(
            i32::try_from(spec.messages.len().saturating_sub(1)).unwrap_or(i32::MAX),
        ));
    let message_scroll_max = (message_content_height - timeline.height).max(0);
    let message_scroll_y = spec.message_scroll_y.clamp(0, message_scroll_max);
    let mut message_y = timeline
        .y
        .saturating_add(top_padding)
        .saturating_sub(message_scroll_y);
    let materialization_bounds = Rect {
        x: timeline.x,
        y: timeline.y.saturating_sub(timeline.height),
        width: timeline.width,
        height: timeline.height.saturating_mul(3),
    };
    let estimated_visible_messages = spec.messages.len().min(64);
    let mut messages = Vec::with_capacity(estimated_visible_messages);

    for (message_index, message) in spec.messages.iter().enumerate() {
        let height = message_heights[message_index];
        let is_user = message.role == ZsWorkbenchMessageRole::User;
        let width = workbench_message_width(message.role, content.width, dpi);
        let x = if is_user {
            content.x + content.width - width
        } else {
            content.x
        };
        let bounds = Rect {
            x,
            y: message_y,
            width,
            height,
        };
        if !rects_intersect(bounds, materialization_bounds) {
            message_y = message_y.saturating_add(height).saturating_add(gap);
            continue;
        }
        let horizontal_padding = workbench_message_horizontal_padding(message.role, dpi);
        let content_bounds = Rect {
            x: bounds.x + horizontal_padding,
            y: bounds.y + scale(8, dpi),
            width: (bounds.width - horizontal_padding * 2).max(0),
            height: (bounds.height - scale(16, dpi)).max(0),
        };
        let mut blocks = Vec::with_capacity(message.blocks.len());
        let mut block_y = content_bounds.y;
        for (block_index, block) in message.blocks.iter().enumerate() {
            let block_height = block_height(
                block,
                content_bounds.width,
                dpi,
                typography_scale,
                measurements,
            );
            blocks.push(ZsWorkbenchBlockLayout {
                block_index,
                bounds: Rect {
                    x: content_bounds.x,
                    y: block_y,
                    width: content_bounds.width,
                    height: block_height,
                },
            });
            block_y += block_height + scale(8, dpi);
        }
        if !message.actions.is_empty() {
            let action_y = bounds.y + bounds.height - scale(30, dpi);
            let mut action_x = if is_user { bounds.x } else { content_bounds.x };
            let action_right = bounds.x + bounds.width;
            for action in &message.actions {
                let action_width = scale(30, dpi)
                    .saturating_add(measured_no_wrap_text_width(
                        &action.label,
                        workbench_text_style(
                            TextRole::Caption,
                            ColorRole::SecondaryText,
                            TextWeight::Regular,
                            HorizontalAlign::Start,
                            TextWrap::NoWrap,
                        ),
                        dpi,
                        measurements,
                    ))
                    .clamp(scale(32, dpi), scale(128, dpi));
                if action_x.saturating_add(action_width) > action_right {
                    break;
                }
                let action_bounds = Rect {
                    x: action_x,
                    y: action_y,
                    width: action_width,
                    height: scale(28, dpi),
                };
                if rects_intersect(action_bounds, timeline) {
                    regions.push(ZsWorkbenchLayoutRegion {
                        kind: ZsWorkbenchRegionKind::MessageAction,
                        id: format!("{}:{}", message.id, action.id),
                        bounds: clipped_to(action_bounds, timeline),
                        enabled: action.enabled,
                    });
                }
                action_x += action_width + scale(4, dpi);
            }
        }
        messages.push(ZsWorkbenchMessageLayout {
            message_index,
            bounds,
            content_bounds,
            blocks,
        });
        message_y = message_y.saturating_add(height).saturating_add(gap);
    }

    ZsWorkbenchLayoutPlan {
        dpi,
        metrics,
        regions,
        messages,
        sidebar_content_height: sidebar_list.content_height,
        sidebar_scroll_y: sidebar_list.scroll_y,
        sidebar_scroll_max: sidebar_list.scroll_max,
        sidebar_scrollbar,
        composer_mode_label_bounds: composer_labels.mode,
        composer_model_label_bounds: composer_labels.model,
        inspector_body_bounds: inspector_body.body,
        inspector_body_viewport: inspector_body.viewport,
        inspector_body_content_height: inspector_body.content_height,
        inspector_scroll_y: inspector_body.scroll_y,
        inspector_scroll_max: inspector_body.scroll_max,
        inspector_scrollbar,
        message_content_height,
        message_scroll_y,
        message_scroll_max,
    }
}

#[derive(Debug, Clone, Copy)]
struct SidebarListGeometry {
    viewport: Rect,
    content_height: i32,
    scroll_y: i32,
    scroll_max: i32,
}

fn sidebar_list_geometry(
    spec: &ZsWorkbenchSpec,
    sidebar: Rect,
    collapsed: bool,
    dpi: Dpi,
) -> SidebarListGeometry {
    let pad = scale(12, dpi);
    let row_height = scale_dp(workbench_style_tokens().controls.navigation_row_height, dpi);
    let list_top = sidebar.y
        + scale(68, dpi)
        + spec.sidebar.primary_actions.len() as i32 * (row_height + scale(4, dpi))
        + scale(14, dpi);
    let footer_height = spec.sidebar.footer_actions.len() as i32 * (row_height + scale(4, dpi));
    let footer_top = sidebar.y + sidebar.height - pad - footer_height;
    let viewport = Rect {
        x: sidebar.x,
        y: list_top,
        width: sidebar.width,
        height: if collapsed {
            0
        } else {
            (footer_top - scale(8, dpi) - list_top).max(0)
        },
    };
    let content_height = if collapsed {
        0
    } else {
        spec.sidebar.groups.iter().fold(0_i32, |height, group| {
            let rows = i32::try_from(group.conversations.len()).unwrap_or(i32::MAX);
            height
                .saturating_add(scale(26, dpi))
                .saturating_add(rows.saturating_mul(scale(52, dpi)))
                .saturating_add(scale(10, dpi))
        })
    };
    let scroll_max = (content_height - viewport.height).max(0);
    SidebarListGeometry {
        viewport,
        content_height,
        scroll_y: spec.sidebar.scroll_y.clamp(0, scroll_max),
        scroll_max,
    }
}

fn composer_band_height(
    spec: &ZsWorkbenchSpec,
    content_width: i32,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
    profile: crate::platform_component_profile::PlatformWorkbenchProfile,
    surface_height: i32,
    top_height: i32,
) -> i32 {
    let text = if spec.composer.draft.is_empty() {
        spec.composer.placeholder.as_str()
    } else {
        spec.composer.draft.as_str()
    };
    let color = if spec.composer.draft.is_empty() {
        ColorRole::SecondaryText
    } else {
        ColorRole::PrimaryText
    };
    let input_width = (content_width - scale(32, dpi)).max(1);
    let text_height = measured_text_height(
        text,
        input_width,
        TextRole::Body,
        color,
        TextWeight::Regular,
        TextWrap::Word,
        dpi,
        typography_scale,
        measurements,
    );
    let base_height = scale_dp(profile.composer_height, dpi).max(0);
    let inset = scale_dp(profile.composer_vertical_inset, dpi);
    let desired = text_height
        .saturating_add(scale(62, dpi))
        .saturating_add(inset.saturating_mul(2));
    let maximum = base_height.saturating_mul(2).max(base_height);
    let available = (surface_height - top_height - scale(48, dpi)).max(0);
    desired
        .clamp(base_height.min(maximum), maximum)
        .min(available)
}

#[derive(Debug, Clone, Copy, Default)]
struct ComposerLabelLayout {
    mode: Option<Rect>,
    model: Option<Rect>,
    actions_right: i32,
}

#[derive(Debug, Clone, Copy, Default)]
struct InspectorBodyGeometry {
    body: Option<Rect>,
    viewport: Option<Rect>,
    content_height: i32,
    scroll_y: i32,
    scroll_max: i32,
}

fn inspector_body_geometry(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> InspectorBodyGeometry {
    let (Some(inspector), Some(panel)) = (&spec.inspector, metrics.inspector) else {
        return InspectorBodyGeometry::default();
    };
    let body_top = panel.y + scale(86, dpi);
    let body_width = (panel.width - scale(20, dpi)).max(0);
    let available_height = (panel.y + panel.height - scale(12, dpi) - body_top).max(0);
    let text_width = (body_width - scale(24, dpi)).max(1);
    let content_height = measured_text_height(
        &inspector.body,
        text_width,
        TextRole::Body,
        ColorRole::PrimaryText,
        TextWeight::Regular,
        TextWrap::Word,
        dpi,
        typography_scale,
        measurements,
    );
    let body_height = (content_height + scale(24, dpi))
        .max(scale(112, dpi))
        .min(available_height);
    let body = Rect {
        x: panel.x + scale(10, dpi),
        y: body_top,
        width: body_width,
        height: body_height,
    };
    let viewport = Rect {
        x: body.x + scale(12, dpi),
        y: body.y + scale(8, dpi),
        width: (body.width - scale(24, dpi)).max(0),
        height: (body.height - scale(16, dpi)).max(0),
    };
    let scroll_max = (content_height - viewport.height).max(0);
    InspectorBodyGeometry {
        body: Some(body),
        viewport: Some(viewport),
        content_height,
        scroll_y: inspector.scroll_y.clamp(0, scroll_max),
        scroll_max,
    }
}

fn composer_label_layout(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    dpi: Dpi,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> ComposerLabelLayout {
    let bounds = metrics.composer;
    let submit_size = scale(36, dpi).min((bounds.width - scale(24, dpi)).max(0));
    let submit_x = bounds.x + bounds.width - scale(12, dpi) - submit_size;
    let left_limit = bounds.x + scale(12, dpi);
    let mut right = submit_x - scale(8, dpi);
    if bounds.width < scale(104, dpi) || bounds.height < scale(44, dpi) {
        return ComposerLabelLayout {
            actions_right: left_limit,
            ..ComposerLabelLayout::default()
        };
    }
    let label_y = bounds.y + bounds.height - scale(42, dpi);
    let label_height = scale(32, dpi);
    let style = workbench_text_style(
        TextRole::Caption,
        ColorRole::SecondaryText,
        TextWeight::Regular,
        HorizontalAlign::End,
        TextWrap::NoWrap,
    );
    let mut result = ComposerLabelLayout {
        actions_right: right,
        ..ComposerLabelLayout::default()
    };
    let label_width = |label: &str, maximum: i32| {
        measurements
            .and_then(|entries| entries.measure(label, style, 0))
            .map(|size| size.width.saturating_add(scale(6, dpi)))
            .unwrap_or_else(|| {
                crate::widget_render::zs_estimated_text_width_px(label, scale(7, dpi))
                    .saturating_add(scale(6, dpi))
            })
            .clamp(scale(40, dpi), maximum)
    };
    if let Some(model) = spec
        .composer
        .model_label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
    {
        let width = label_width(model, scale(144, dpi));
        if right - width >= left_limit {
            result.model = Some(Rect {
                x: right - width,
                y: label_y,
                width,
                height: label_height,
            });
            right -= width + scale(8, dpi);
        }
    }
    if let Some(mode) = spec
        .composer
        .mode_label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
    {
        let width = label_width(mode, scale(120, dpi));
        if right - width >= left_limit {
            result.mode = Some(Rect {
                x: right - width,
                y: label_y,
                width,
                height: label_height,
            });
            right -= width + scale(8, dpi);
        }
    }
    result.actions_right = right;
    result
}

fn layout_sidebar_regions(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    list: &SidebarListGeometry,
    dpi: Dpi,
    regions: &mut Vec<ZsWorkbenchLayoutRegion>,
) {
    let pad = scale(12, dpi);
    let style = workbench_style_tokens();
    let row_height = scale_dp(style.controls.navigation_row_height, dpi);
    let touch_target = scale_dp(style.controls.touch_target, dpi);
    regions.push(ZsWorkbenchLayoutRegion {
        kind: ZsWorkbenchRegionKind::SidebarToggle,
        id: if metrics.sidebar_collapsed {
            "sidebar.toggle.expand"
        } else {
            "sidebar.toggle.collapse"
        }
        .to_string(),
        bounds: Rect {
            x: metrics.sidebar.x + pad,
            y: metrics.sidebar.y + scale(12, dpi),
            width: touch_target,
            height: touch_target,
        },
        enabled: true,
    });
    let mut y = metrics.sidebar.y + scale(68, dpi);
    for action in &spec.sidebar.primary_actions {
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::SidebarAction,
            id: action.id.clone(),
            bounds: Rect {
                x: metrics.sidebar.x + pad,
                y,
                width: (metrics.sidebar.width - pad * 2).max(0),
                height: row_height,
            },
            enabled: action.enabled,
        });
        y += row_height + scale(4, dpi);
    }
    if !metrics.sidebar_collapsed && list.viewport.height > 0 {
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::SidebarViewport,
            id: "sidebar.viewport".to_string(),
            bounds: list.viewport,
            enabled: list.scroll_max > 0,
        });
        y = list.viewport.y - list.scroll_y;
        for group in &spec.sidebar.groups {
            y += scale(26, dpi);
            for conversation in &group.conversations {
                let bounds = Rect {
                    x: metrics.sidebar.x + pad,
                    y,
                    width: (metrics.sidebar.width - pad * 2).max(0),
                    height: scale(48, dpi),
                };
                if rects_intersect(bounds, list.viewport) {
                    regions.push(ZsWorkbenchLayoutRegion {
                        kind: ZsWorkbenchRegionKind::Conversation,
                        id: conversation.id.clone(),
                        bounds: clipped_to(bounds, list.viewport),
                        enabled: true,
                    });
                }
                y += scale(52, dpi);
            }
            y += scale(10, dpi);
        }
    }
    let mut footer_y = metrics.sidebar.y + metrics.sidebar.height
        - pad
        - spec.sidebar.footer_actions.len() as i32 * (row_height + scale(4, dpi));
    for action in &spec.sidebar.footer_actions {
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::SidebarAction,
            id: action.id.clone(),
            bounds: Rect {
                x: metrics.sidebar.x + pad,
                y: footer_y,
                width: (metrics.sidebar.width - pad * 2).max(0),
                height: row_height,
            },
            enabled: action.enabled,
        });
        footer_y += row_height + scale(4, dpi);
    }
}

fn layout_toolbar_regions(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    dpi: Dpi,
    regions: &mut Vec<ZsWorkbenchLayoutRegion>,
) {
    let button = scale_dp(workbench_style_tokens().controls.standard_height, dpi);
    let mut x = metrics.top_bar.x + metrics.top_bar.width - scale(16, dpi) - button;
    for action in spec.toolbar_actions.iter().rev() {
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::ToolbarAction,
            id: action.id.clone(),
            bounds: Rect {
                x,
                y: metrics.top_bar.y + scale(16, dpi),
                width: button,
                height: button,
            },
            enabled: action.enabled,
        });
        x -= button + scale(4, dpi);
    }
}

fn layout_composer_regions(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    labels: &ComposerLabelLayout,
    dpi: Dpi,
    measurements: Option<&crate::view::ViewTextMeasurements>,
    regions: &mut Vec<ZsWorkbenchLayoutRegion>,
) {
    let bottom_y =
        (metrics.composer.y + metrics.composer.height - scale(44, dpi)).max(metrics.composer.y);
    regions.push(ZsWorkbenchLayoutRegion {
        kind: ZsWorkbenchRegionKind::ComposerInput,
        id: "composer.input".to_string(),
        bounds: Rect {
            x: metrics.composer.x + scale(16, dpi),
            y: metrics.composer.y + scale(12, dpi),
            width: (metrics.composer.width - scale(32, dpi)).max(0),
            height: (metrics.composer.height - scale(62, dpi)).max(0),
        },
        enabled: true,
    });
    let mut action_x = metrics.composer.x + scale(12, dpi);
    for action in &spec.composer.actions {
        let width = scale(34, dpi)
            .saturating_add(measured_no_wrap_text_width(
                &action.label,
                workbench_text_style(
                    TextRole::Caption,
                    ColorRole::SecondaryText,
                    TextWeight::Regular,
                    HorizontalAlign::Start,
                    TextWrap::NoWrap,
                ),
                dpi,
                measurements,
            ))
            .clamp(scale(34, dpi), scale(140, dpi));
        if action_x.saturating_add(width) > labels.actions_right {
            break;
        }
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::ComposerAction,
            id: action.id.clone(),
            bounds: Rect {
                x: action_x,
                y: bottom_y,
                width,
                height: scale(32, dpi),
            },
            enabled: action.enabled,
        });
        action_x += width + scale(4, dpi);
    }
    let submit_size = scale(36, dpi).min((metrics.composer.width - scale(24, dpi)).max(0));
    regions.push(ZsWorkbenchLayoutRegion {
        kind: if spec.composer.busy {
            ZsWorkbenchRegionKind::Stop
        } else {
            ZsWorkbenchRegionKind::Submit
        },
        id: if spec.composer.busy {
            "composer.stop"
        } else {
            "composer.submit"
        }
        .to_string(),
        bounds: Rect {
            x: metrics.composer.x + metrics.composer.width - scale(12, dpi) - submit_size,
            y: bottom_y - scale(2, dpi),
            width: submit_size,
            height: submit_size,
        },
        enabled: spec.composer.busy || !spec.composer.draft.trim().is_empty(),
    });
}

fn layout_inspector_regions(
    spec: &ZsWorkbenchSpec,
    metrics: &ZsWorkbenchLayoutMetrics,
    body: &InspectorBodyGeometry,
    dpi: Dpi,
    measurements: Option<&crate::view::ViewTextMeasurements>,
    regions: &mut Vec<ZsWorkbenchLayoutRegion>,
) {
    let (Some(inspector), Some(bounds)) = (&spec.inspector, metrics.inspector) else {
        return;
    };
    let mut x = bounds.x + scale(20, dpi);
    let right = bounds.x + bounds.width - scale(20, dpi);
    for tab in &inspector.tabs {
        let selected = inspector.selected_tab_id.as_deref() == Some(tab.id.as_str());
        let width = scale(24, dpi)
            .saturating_add(measured_no_wrap_text_width(
                &tab.label,
                workbench_text_style(
                    TextRole::Caption,
                    if selected {
                        ColorRole::PrimaryText
                    } else {
                        ColorRole::SecondaryText
                    },
                    if selected {
                        TextWeight::Semibold
                    } else {
                        TextWeight::Regular
                    },
                    HorizontalAlign::Center,
                    TextWrap::NoWrap,
                ),
                dpi,
                measurements,
            ))
            .clamp(scale(48, dpi), scale(128, dpi));
        if x.saturating_add(width) > right {
            break;
        }
        regions.push(ZsWorkbenchLayoutRegion {
            kind: ZsWorkbenchRegionKind::InspectorTab,
            id: tab.id.clone(),
            bounds: Rect {
                x,
                y: bounds.y + scale(58, dpi),
                width,
                height: scale(32, dpi),
            },
            enabled: tab.enabled,
        });
        x += width + scale(4, dpi);
    }
    if body.scroll_max > 0 {
        if let Some(bounds) = body
            .body
            .filter(|bounds| bounds.width > 0 && bounds.height > 0)
        {
            regions.push(ZsWorkbenchLayoutRegion {
                kind: ZsWorkbenchRegionKind::InspectorViewport,
                id: "inspector.viewport".to_string(),
                bounds,
                enabled: true,
            });
        }
    }
}

fn message_height(
    message: &ZsWorkbenchMessageSpec,
    width: i32,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> i32 {
    let content_width = workbench_message_width(message.role, width, dpi)
        .saturating_sub(workbench_message_horizontal_padding(message.role, dpi).saturating_mul(2));
    let blocks_height = message.blocks.iter().fold(0_i32, |height, block| {
        height.saturating_add(block_height(
            block,
            content_width.max(1),
            dpi,
            typography_scale,
            measurements,
        ))
    });
    let block_gaps = i32::try_from(message.blocks.len().saturating_sub(1))
        .unwrap_or(i32::MAX)
        .saturating_mul(scale(8, dpi));
    let action_height = if message.actions.is_empty() {
        0
    } else {
        scale(34, dpi)
    };
    blocks_height
        .saturating_add(block_gaps)
        .saturating_add(action_height)
        .saturating_add(scale(16, dpi))
        .max(scale(44, dpi))
}

fn workbench_message_width(role: ZsWorkbenchMessageRole, available_width: i32, dpi: Dpi) -> i32 {
    let available_width = available_width.max(0);
    if role == ZsWorkbenchMessageRole::User {
        (available_width.saturating_mul(3) / 4)
            .max(scale(220, dpi))
            .min(available_width)
    } else {
        available_width
    }
}

fn workbench_message_horizontal_padding(role: ZsWorkbenchMessageRole, dpi: Dpi) -> i32 {
    if role == ZsWorkbenchMessageRole::User {
        scale(14, dpi)
    } else {
        scale(16, dpi)
    }
}

fn block_height(
    block: &ZsWorkbenchContentBlock,
    width: i32,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> i32 {
    match block {
        ZsWorkbenchContentBlock::Paragraph { text } => measured_text_height(
            text,
            width,
            TextRole::Body,
            ColorRole::PrimaryText,
            TextWeight::Regular,
            TextWrap::Word,
            dpi,
            typography_scale,
            measurements,
        ),
        ZsWorkbenchContentBlock::Code { code, .. } => {
            scale(42, dpi)
                + measured_text_height(
                    code,
                    (width - scale(24, dpi)).max(1),
                    TextRole::Monospace,
                    ColorRole::PrimaryText,
                    TextWeight::Regular,
                    TextWrap::Word,
                    dpi,
                    typography_scale,
                    measurements,
                )
        }
        ZsWorkbenchContentBlock::Tool { .. } => scale(68, dpi),
        ZsWorkbenchContentBlock::Notice { text, .. } => {
            measured_text_height(
                text,
                (width - scale(24, dpi)).max(1),
                TextRole::Caption,
                ColorRole::PrimaryText,
                TextWeight::Regular,
                TextWrap::Word,
                dpi,
                typography_scale,
                measurements,
            ) + scale(22, dpi)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn measured_text_height(
    text: &str,
    width: i32,
    role: TextRole,
    color: ColorRole,
    weight: TextWeight,
    wrap: TextWrap,
    dpi: Dpi,
    typography_scale: f32,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> i32 {
    let width = width.max(1);
    let style = workbench_text_style(role, color, weight, HorizontalAlign::Start, wrap);
    let max_width = if wrap == TextWrap::Word { width } else { 0 };
    if let Some(size) = measurements.and_then(|entries| entries.measure(text, style, max_width)) {
        if size.height > 0 {
            return size.height;
        }
    }
    let metrics = role.metrics_for(crate::ZsTypographyPlatformStyle::current());
    let typography_scale = if typography_scale.is_finite() {
        typography_scale.clamp(0.5, 3.0)
    } else {
        1.0
    };
    let line_height = ((metrics.line_height * typography_scale) * dpi.scale_factor())
        .round()
        .max(1.0) as i32;
    estimate_text_height(text, width, line_height, dpi)
}

fn measured_no_wrap_text_width(
    text: &str,
    style: SemanticTextStyle,
    dpi: Dpi,
    measurements: Option<&crate::view::ViewTextMeasurements>,
) -> i32 {
    measurements
        .and_then(|entries| entries.measure(text, style, 0))
        .map(|size| size.width.max(0))
        .unwrap_or_else(|| {
            crate::widget_render::zs_estimated_text_width_px(text, scale(7, dpi)).max(0)
        })
}

fn estimate_text_height(text: &str, width: i32, line_height: i32, dpi: Dpi) -> i32 {
    let chars_per_line = (width / scale(7, dpi).max(1)).max(1) as usize;
    let lines = text
        .lines()
        .map(|line| {
            (crate::widget_render::zs_estimated_text_flow_units(line).max(1) as usize)
                .div_ceil(chars_per_line)
        })
        .sum::<usize>()
        .max(1);
    i32::try_from(lines)
        .unwrap_or(i32::MAX)
        .saturating_mul(line_height)
}

pub fn zs_workbench_native_draw_plan(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
) -> NativeDrawPlan {
    let mut commands = Vec::new();
    let metrics = layout.metrics;
    commands.push(fill(
        metrics.surface,
        NativeDrawFill::role(ColorRole::Surface),
    ));
    commands.push(fill(
        metrics.sidebar,
        NativeDrawFill::role(ColorRole::Surface),
    ));
    commands.push(stroke(
        Rect {
            x: metrics.sidebar.x + metrics.sidebar.width - 1,
            y: metrics.sidebar.y,
            width: 1,
            height: metrics.sidebar.height,
        },
        NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Border,
            alpha: 42,
        },
    ));
    paint_sidebar(spec, layout, &mut commands);
    paint_top_bar(spec, layout, &mut commands);
    paint_messages(spec, layout, &mut commands);
    paint_composer(spec, layout, &mut commands);
    paint_inspector(spec, layout, &mut commands);
    NativeDrawPlan::new(commands)
}

fn paint_sidebar(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let dpi = layout.dpi;
    let bounds = layout.metrics.sidebar;
    let style = workbench_style_tokens();
    let card_radius = scale_dp(style.radius.medium, dpi);
    if !layout.metrics.sidebar_collapsed {
        commands.push(text_command(
            &spec.sidebar.title,
            Rect {
                x: bounds.x + scale(56, dpi),
                y: bounds.y + scale(14, dpi),
                width: (bounds.width - scale(72, dpi)).max(0),
                height: scale(36, dpi),
            },
            TextRole::Button,
            ColorRole::PrimaryText,
            TextWeight::Semibold,
            HorizontalAlign::Start,
            TextWrap::NoWrap,
        ));
    }
    for region in layout.regions.iter().filter(|region| {
        matches!(
            region.kind,
            ZsWorkbenchRegionKind::SidebarToggle | ZsWorkbenchRegionKind::SidebarAction
        )
    }) {
        let action = spec
            .sidebar
            .primary_actions
            .iter()
            .chain(spec.sidebar.footer_actions.iter())
            .find(|action| action.id == region.id);
        let (icon, label, selected) = match region.kind {
            ZsWorkbenchRegionKind::SidebarToggle => (ZsWorkbenchIcon::Sidebar, "", false),
            _ => action
                .map(|action| (action.icon, action.label.as_str(), action.selected))
                .unwrap_or((ZsWorkbenchIcon::More, "", false)),
        };
        if selected {
            commands.push(round_fill(
                region.bounds,
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 22,
                },
                card_radius,
            ));
        }
        let icon_bounds = if layout.metrics.sidebar_collapsed
            || region.kind == ZsWorkbenchRegionKind::SidebarToggle
        {
            icon_bounds(region.bounds, dpi)
        } else {
            Rect {
                x: region.bounds.x + scale(10, dpi),
                y: region.bounds.y + (region.bounds.height - scale(20, dpi)) / 2,
                width: scale(20, dpi),
                height: scale(20, dpi),
            }
        };
        commands.push(icon_command(
            icon,
            icon_bounds,
            if region.enabled {
                ColorRole::PrimaryText
            } else {
                ColorRole::DisabledText
            },
        ));
        if !layout.metrics.sidebar_collapsed && !label.is_empty() {
            commands.push(text_command(
                label,
                Rect {
                    x: region.bounds.x + scale(40, dpi),
                    y: region.bounds.y,
                    width: (region.bounds.width - scale(46, dpi)).max(0),
                    height: region.bounds.height,
                },
                TextRole::Body,
                ColorRole::PrimaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
                TextWrap::NoWrap,
            ));
        }
    }
    if layout.metrics.sidebar_collapsed {
        return;
    }
    let Some(viewport) = layout.metrics.sidebar_list_viewport else {
        return;
    };
    commands.push(NativeDrawCommand::PushClip { rect: viewport });
    let mut group_label_y = viewport.y - layout.sidebar_scroll_y;
    for group in &spec.sidebar.groups {
        let group_label_bounds = Rect {
            x: bounds.x + scale(16, dpi),
            y: group_label_y,
            width: (bounds.width - scale(32, dpi)).max(0),
            height: scale(24, dpi),
        };
        if rects_intersect(group_label_bounds, viewport) {
            commands.push(text_command(
                &group.label,
                group_label_bounds,
                TextRole::Caption,
                ColorRole::SecondaryText,
                TextWeight::Semibold,
                HorizontalAlign::Start,
                TextWrap::NoWrap,
            ));
        }
        group_label_y += scale(26, dpi);
        for conversation in &group.conversations {
            let conversation_bounds = Rect {
                x: bounds.x + scale(12, dpi),
                y: group_label_y,
                width: (bounds.width - scale(24, dpi)).max(0),
                height: scale(48, dpi),
            };
            if rects_intersect(conversation_bounds, viewport) {
                if conversation.selected {
                    commands.push(round_rect(
                        conversation_bounds,
                        NativeDrawFill::role(ColorRole::SurfaceRaised),
                        Some(NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Border,
                            alpha: 30,
                        }),
                        card_radius,
                    ));
                    commands.push(round_fill(
                        Rect {
                            x: conversation_bounds.x + scale(3, dpi),
                            y: conversation_bounds.y + scale(13, dpi),
                            width: scale(3, dpi),
                            height: scale(22, dpi),
                        },
                        NativeDrawFill::role(ColorRole::Accent),
                        scale(2, dpi),
                    ));
                }
                commands.push(text_command(
                    &conversation.title,
                    Rect {
                        x: conversation_bounds.x + scale(14, dpi),
                        y: conversation_bounds.y + scale(2, dpi),
                        width: (conversation_bounds.width - scale(26, dpi)).max(0),
                        height: if conversation.subtitle.is_some() {
                            scale(22, dpi)
                        } else {
                            scale(42, dpi)
                        },
                    },
                    TextRole::Body,
                    ColorRole::PrimaryText,
                    if conversation.unread {
                        TextWeight::Semibold
                    } else {
                        TextWeight::Regular
                    },
                    HorizontalAlign::Start,
                    TextWrap::NoWrap,
                ));
                if let Some(subtitle) = &conversation.subtitle {
                    commands.push(text_command(
                        subtitle,
                        Rect {
                            x: conversation_bounds.x + scale(14, dpi),
                            y: conversation_bounds.y + scale(22, dpi),
                            width: (conversation_bounds.width - scale(26, dpi)).max(0),
                            height: scale(20, dpi),
                        },
                        TextRole::Caption,
                        ColorRole::SecondaryText,
                        TextWeight::Regular,
                        HorizontalAlign::Start,
                        TextWrap::NoWrap,
                    ));
                }
                if conversation.pinned {
                    commands.push(icon_command(
                        ZsIcon::Pin,
                        Rect {
                            x: conversation_bounds.x + conversation_bounds.width - scale(20, dpi),
                            y: conversation_bounds.y + scale(14, dpi),
                            width: scale(14, dpi),
                            height: scale(18, dpi),
                        },
                        ColorRole::SecondaryText,
                    ));
                }
            }
            group_label_y += scale(52, dpi);
        }
        group_label_y += scale(10, dpi);
    }
    commands.push(NativeDrawCommand::PopClip);
    paint_workbench_scrollbar(commands, layout.sidebar_scrollbar);
}

fn paint_top_bar(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let dpi = layout.dpi;
    let bounds = layout.metrics.top_bar;
    commands.push(fill(bounds, NativeDrawFill::role(ColorRole::Surface)));
    commands.push(text_command(
        &spec.title,
        Rect {
            x: bounds.x + scale(24, dpi),
            y: bounds.y + scale(10, dpi),
            width: (bounds.width - scale(180, dpi)).max(0),
            height: if spec.subtitle.is_some() {
                scale(26, dpi)
            } else {
                scale(44, dpi)
            },
        },
        TextRole::Button,
        ColorRole::PrimaryText,
        TextWeight::Semibold,
        HorizontalAlign::Start,
        TextWrap::NoWrap,
    ));
    if let Some(subtitle) = &spec.subtitle {
        commands.push(text_command(
            subtitle,
            Rect {
                x: bounds.x + scale(24, dpi),
                y: bounds.y + scale(32, dpi),
                width: (bounds.width - scale(180, dpi)).max(0),
                height: scale(20, dpi),
            },
            TextRole::Caption,
            ColorRole::SecondaryText,
            TextWeight::Regular,
            HorizontalAlign::Start,
            TextWrap::NoWrap,
        ));
    }
    for region in layout
        .regions
        .iter()
        .filter(|region| region.kind == ZsWorkbenchRegionKind::ToolbarAction)
    {
        if let Some(action) = spec
            .toolbar_actions
            .iter()
            .find(|action| action.id == region.id)
        {
            commands.push(round_rect(
                region.bounds,
                if action.selected {
                    NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 24,
                    }
                } else {
                    NativeDrawFill::role(ColorRole::Control)
                },
                Some(NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Border,
                    alpha: 28,
                }),
                scale(6, dpi),
            ));
            commands.push(icon_command(
                action.icon,
                icon_bounds(region.bounds, dpi),
                if region.enabled {
                    ColorRole::PrimaryText
                } else {
                    ColorRole::DisabledText
                },
            ));
        }
    }
}

fn paint_messages(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let dpi = layout.dpi;
    let card_radius = scale_dp(workbench_style_tokens().radius.medium, dpi);
    commands.push(NativeDrawCommand::PushClip {
        rect: layout.metrics.timeline,
    });
    for message_layout in &layout.messages {
        let message = &spec.messages[message_layout.message_index];
        match message.role {
            ZsWorkbenchMessageRole::User => commands.push(round_fill(
                message_layout.bounds,
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 18,
                },
                workbench_floating_radius(dpi),
            )),
            ZsWorkbenchMessageRole::System => commands.push(round_fill(
                message_layout.bounds,
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::SecondaryText,
                    alpha: 18,
                },
                scale(6, dpi),
            )),
            ZsWorkbenchMessageRole::Tool => commands.push(round_rect(
                message_layout.bounds,
                NativeDrawFill::role(ColorRole::SurfaceRaised),
                Some(NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::SecondaryText,
                    alpha: 32,
                }),
                card_radius,
            )),
            ZsWorkbenchMessageRole::Assistant => commands.push(round_rect(
                message_layout.bounds,
                NativeDrawFill::role(ColorRole::SurfaceRaised),
                Some(NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Border,
                    alpha: 24,
                }),
                workbench_floating_radius(dpi),
            )),
        }
        for block_layout in &message_layout.blocks {
            let block = &message.blocks[block_layout.block_index];
            paint_message_block(block, block_layout.bounds, dpi, commands);
        }
        for region in layout.regions.iter().filter(|region| {
            region.kind == ZsWorkbenchRegionKind::MessageAction
                && region.id.starts_with(&format!("{}:", message.id))
        }) {
            let action_id = region.id.split_once(':').map(|(_, id)| id).unwrap_or("");
            if let Some(action) = message.actions.iter().find(|action| action.id == action_id) {
                commands.push(round_rect(
                    region.bounds,
                    NativeDrawFill::role(ColorRole::SurfaceRaised),
                    Some(NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Border,
                        alpha: 24,
                    }),
                    scale(6, dpi),
                ));
                commands.push(icon_command(
                    action.icon,
                    Rect {
                        x: region.bounds.x + scale(6, dpi),
                        y: region.bounds.y + scale(6, dpi),
                        width: scale(16, dpi),
                        height: scale(16, dpi),
                    },
                    if action.enabled {
                        ColorRole::SecondaryText
                    } else {
                        ColorRole::DisabledText
                    },
                ));
                if !action.label.is_empty() {
                    commands.push(text_command(
                        &action.label,
                        Rect {
                            x: region.bounds.x + scale(24, dpi),
                            y: region.bounds.y,
                            width: (region.bounds.width - scale(28, dpi)).max(0),
                            height: region.bounds.height,
                        },
                        TextRole::Caption,
                        ColorRole::SecondaryText,
                        TextWeight::Regular,
                        HorizontalAlign::Start,
                        TextWrap::NoWrap,
                    ));
                }
            }
        }
        if message.streaming {
            commands.push(round_fill(
                Rect {
                    x: message_layout.content_bounds.x,
                    y: message_layout.bounds.y + message_layout.bounds.height - scale(8, dpi),
                    width: scale(6, dpi),
                    height: scale(6, dpi),
                },
                NativeDrawFill::role(ColorRole::Accent),
                scale(3, dpi),
            ));
        }
    }
    commands.push(NativeDrawCommand::PopClip);
}

fn paint_message_block(
    block: &ZsWorkbenchContentBlock,
    bounds: Rect,
    dpi: Dpi,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let card_radius = scale_dp(workbench_style_tokens().radius.medium, dpi);
    match block {
        ZsWorkbenchContentBlock::Paragraph { text } => commands.push(text_command(
            text,
            bounds,
            TextRole::Body,
            ColorRole::PrimaryText,
            TextWeight::Regular,
            HorizontalAlign::Start,
            TextWrap::Word,
        )),
        ZsWorkbenchContentBlock::Code { language, code } => {
            commands.push(round_rect(
                bounds,
                NativeDrawFill::role(ColorRole::Surface),
                Some(NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Border,
                    alpha: 26,
                }),
                card_radius,
            ));
            commands.push(text_command(
                language,
                Rect {
                    x: bounds.x + scale(12, dpi),
                    y: bounds.y + scale(4, dpi),
                    width: (bounds.width - scale(24, dpi)).max(0),
                    height: scale(26, dpi),
                },
                TextRole::Caption,
                ColorRole::SecondaryText,
                TextWeight::Semibold,
                HorizontalAlign::Start,
                TextWrap::NoWrap,
            ));
            commands.push(text_command(
                code,
                Rect {
                    x: bounds.x + scale(12, dpi),
                    y: bounds.y + scale(30, dpi),
                    width: (bounds.width - scale(24, dpi)).max(0),
                    height: (bounds.height - scale(38, dpi)).max(0),
                },
                TextRole::Monospace,
                ColorRole::PrimaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
                TextWrap::Word,
            ));
        }
        ZsWorkbenchContentBlock::Tool {
            title,
            summary,
            status,
            status_label,
        } => {
            let status_label = status_label
                .as_deref()
                .filter(|label| !label.trim().is_empty());
            paint_elevated_surface(
                commands,
                bounds,
                dpi,
                workbench_style_tokens().radius.medium,
                NativeDrawFill::role(ColorRole::Surface),
            );
            commands.push(icon_command(
                if *status == ZsWorkbenchToolStatus::Succeeded {
                    ZsWorkbenchIcon::Check
                } else {
                    ZsWorkbenchIcon::Tool
                },
                Rect {
                    x: bounds.x + scale(12, dpi),
                    y: bounds.y + scale(12, dpi),
                    width: scale(20, dpi),
                    height: scale(20, dpi),
                },
                if *status == ZsWorkbenchToolStatus::Succeeded {
                    ColorRole::Success
                } else {
                    ColorRole::PrimaryText
                },
            ));
            commands.push(text_command(
                title,
                Rect {
                    x: bounds.x + scale(40, dpi),
                    y: bounds.y + scale(7, dpi),
                    width: (bounds.width
                        - if status_label.is_some() {
                            scale(140, dpi)
                        } else {
                            scale(52, dpi)
                        })
                    .max(0),
                    height: scale(26, dpi),
                },
                TextRole::Button,
                ColorRole::PrimaryText,
                TextWeight::Semibold,
                HorizontalAlign::Start,
                TextWrap::NoWrap,
            ));
            if let Some(status_label) = status_label {
                commands.push(text_command(
                    status_label,
                    Rect {
                        x: bounds.x + bounds.width - scale(100, dpi),
                        y: bounds.y + scale(7, dpi),
                        width: scale(88, dpi),
                        height: scale(24, dpi),
                    },
                    TextRole::Caption,
                    if *status == ZsWorkbenchToolStatus::Failed {
                        ColorRole::Danger
                    } else {
                        ColorRole::SecondaryText
                    },
                    TextWeight::Regular,
                    HorizontalAlign::End,
                    TextWrap::NoWrap,
                ));
            }
            commands.push(text_command(
                summary,
                Rect {
                    x: bounds.x + scale(40, dpi),
                    y: bounds.y + scale(32, dpi),
                    width: (bounds.width - scale(52, dpi)).max(0),
                    height: scale(24, dpi),
                },
                TextRole::Caption,
                ColorRole::SecondaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
                TextWrap::NoWrap,
            ));
        }
        ZsWorkbenchContentBlock::Notice { text, level } => {
            let role = match level {
                ZsWorkbenchNoticeLevel::Error => ColorRole::Danger,
                _ => ColorRole::Accent,
            };
            commands.push(round_rect(
                bounds,
                NativeDrawFill::RoleWithAlpha { role, alpha: 20 },
                Some(NativeDrawFill::RoleWithAlpha { role, alpha: 42 }),
                card_radius,
            ));
            commands.push(round_fill(
                Rect {
                    x: bounds.x + scale(3, dpi),
                    y: bounds.y + scale(8, dpi),
                    width: scale(3, dpi),
                    height: (bounds.height - scale(16, dpi)).max(0),
                },
                NativeDrawFill::role(role),
                scale(2, dpi),
            ));
            commands.push(text_command(
                text,
                Rect {
                    x: bounds.x + scale(12, dpi),
                    y: bounds.y + scale(8, dpi),
                    width: (bounds.width - scale(24, dpi)).max(0),
                    height: (bounds.height - scale(16, dpi)).max(0),
                },
                TextRole::Caption,
                ColorRole::PrimaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
                TextWrap::Word,
            ));
        }
    }
}

fn paint_composer(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let dpi = layout.dpi;
    let bounds = layout.metrics.composer;
    paint_elevated_surface(
        commands,
        bounds,
        dpi,
        workbench_profile().floating_radius,
        NativeDrawFill::role(ColorRole::SurfaceRaised),
    );
    let input = layout
        .regions
        .iter()
        .find(|region| region.kind == ZsWorkbenchRegionKind::ComposerInput)
        .map(|region| region.bounds)
        .unwrap_or(bounds);
    commands.push(text_command(
        if spec.composer.draft.is_empty() {
            &spec.composer.placeholder
        } else {
            &spec.composer.draft
        },
        input,
        TextRole::Body,
        if spec.composer.draft.is_empty() {
            ColorRole::SecondaryText
        } else {
            ColorRole::PrimaryText
        },
        TextWeight::Regular,
        HorizontalAlign::Start,
        TextWrap::Word,
    ));
    for region in layout
        .regions
        .iter()
        .filter(|region| region.kind == ZsWorkbenchRegionKind::ComposerAction)
    {
        if let Some(action) = spec
            .composer
            .actions
            .iter()
            .find(|action| action.id == region.id)
        {
            commands.push(round_rect(
                region.bounds,
                if action.selected {
                    NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::Accent,
                        alpha: 22,
                    }
                } else {
                    NativeDrawFill::RoleWithAlpha {
                        role: ColorRole::SecondaryText,
                        alpha: 8,
                    }
                },
                action.selected.then_some(NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 38,
                }),
                scale(7, dpi),
            ));
            commands.push(icon_command(
                action.icon,
                Rect {
                    x: region.bounds.x + scale(7, dpi),
                    y: region.bounds.y + scale(7, dpi),
                    width: scale(18, dpi),
                    height: scale(18, dpi),
                },
                if action.enabled {
                    ColorRole::SecondaryText
                } else {
                    ColorRole::DisabledText
                },
            ));
            if !action.label.is_empty() {
                commands.push(text_command(
                    &action.label,
                    Rect {
                        x: region.bounds.x + scale(28, dpi),
                        y: region.bounds.y,
                        width: (region.bounds.width - scale(34, dpi)).max(0),
                        height: region.bounds.height,
                    },
                    TextRole::Caption,
                    ColorRole::SecondaryText,
                    TextWeight::Regular,
                    HorizontalAlign::Start,
                    TextWrap::NoWrap,
                ));
            }
        }
    }
    if let (Some(label), Some(label_bounds)) = (
        spec.composer.mode_label.as_deref(),
        layout.composer_mode_label_bounds,
    ) {
        commands.push(text_command(
            label,
            label_bounds,
            TextRole::Caption,
            ColorRole::SecondaryText,
            TextWeight::Regular,
            HorizontalAlign::End,
            TextWrap::NoWrap,
        ));
    }
    if let (Some(label), Some(label_bounds)) = (
        spec.composer.model_label.as_deref(),
        layout.composer_model_label_bounds,
    ) {
        commands.push(text_command(
            label,
            label_bounds,
            TextRole::Caption,
            ColorRole::SecondaryText,
            TextWeight::Regular,
            HorizontalAlign::End,
            TextWrap::NoWrap,
        ));
    }
    let submit = layout.regions.iter().find(|region| {
        matches!(
            region.kind,
            ZsWorkbenchRegionKind::Submit | ZsWorkbenchRegionKind::Stop
        )
    });
    if let Some(region) = submit {
        commands.push(round_fill(
            Rect {
                x: region.bounds.x,
                y: region.bounds.y + scale(2, dpi),
                ..region.bounds
            },
            NativeDrawFill::RoleWithAlpha {
                role: ColorRole::SecondaryText,
                alpha: 24,
            },
            scale(10, dpi),
        ));
        commands.push(round_fill(
            region.bounds,
            if region.enabled {
                NativeDrawFill::role(ColorRole::Accent)
            } else {
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::SecondaryText,
                    alpha: 28,
                }
            },
            scale(10, dpi),
        ));
        commands.push(icon_command(
            if region.kind == ZsWorkbenchRegionKind::Stop {
                ZsWorkbenchIcon::Stop
            } else {
                ZsWorkbenchIcon::Enter
            },
            Rect {
                x: region.bounds.x + scale(4, dpi),
                y: region.bounds.y + scale(4, dpi),
                width: (region.bounds.width - scale(8, dpi)).max(0),
                height: (region.bounds.height - scale(8, dpi)).max(0),
            },
            if region.enabled {
                ColorRole::AccentText
            } else {
                ColorRole::DisabledText
            },
        ));
    }
}

fn paint_inspector(
    spec: &ZsWorkbenchSpec,
    layout: &ZsWorkbenchLayoutPlan,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let dpi = layout.dpi;
    let (Some(inspector), Some(bounds)) = (&spec.inspector, layout.metrics.inspector) else {
        return;
    };
    let panel = Rect {
        x: bounds.x + scale(10, dpi),
        y: bounds.y + scale(10, dpi),
        width: (bounds.width - scale(20, dpi)).max(0),
        height: (bounds.height - scale(20, dpi)).max(0),
    };
    paint_elevated_surface(
        commands,
        panel,
        dpi,
        workbench_profile().floating_radius,
        NativeDrawFill::role(ColorRole::SurfaceRaised),
    );
    commands.push(stroke(
        Rect {
            x: bounds.x,
            y: bounds.y,
            width: 1,
            height: bounds.height,
        },
        NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Border,
            alpha: 34,
        },
    ));
    commands.push(text_command(
        &inspector.title,
        Rect {
            x: panel.x + scale(12, dpi),
            y: panel.y + scale(6, dpi),
            width: (panel.width - scale(24, dpi)).max(0),
            height: scale(36, dpi),
        },
        TextRole::Button,
        ColorRole::PrimaryText,
        TextWeight::Semibold,
        HorizontalAlign::Start,
        TextWrap::NoWrap,
    ));
    for region in layout
        .regions
        .iter()
        .filter(|region| region.kind == ZsWorkbenchRegionKind::InspectorTab)
    {
        let selected = inspector.selected_tab_id.as_deref() == Some(region.id.as_str());
        if selected {
            commands.push(round_fill(
                region.bounds,
                NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::SecondaryText,
                    alpha: 10,
                },
                scale(6, dpi),
            ));
            commands.push(round_fill(
                Rect {
                    x: region.bounds.x + scale(12, dpi),
                    y: region.bounds.y + region.bounds.height - scale(3, dpi),
                    width: (region.bounds.width - scale(24, dpi)).max(0),
                    height: scale(3, dpi),
                },
                NativeDrawFill::role(ColorRole::Accent),
                scale(2, dpi),
            ));
        }
        if let Some(tab) = inspector.tabs.iter().find(|tab| tab.id == region.id) {
            commands.push(text_command(
                &tab.label,
                region.bounds,
                TextRole::Caption,
                if selected {
                    ColorRole::PrimaryText
                } else {
                    ColorRole::SecondaryText
                },
                if selected {
                    TextWeight::Semibold
                } else {
                    TextWeight::Regular
                },
                HorizontalAlign::Center,
                TextWrap::NoWrap,
            ));
        }
    }
    let (Some(body_card), Some(body_viewport)) =
        (layout.inspector_body_bounds, layout.inspector_body_viewport)
    else {
        return;
    };
    commands.push(round_rect(
        body_card,
        NativeDrawFill::role(ColorRole::Surface),
        Some(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Border,
            alpha: 22,
        }),
        scale_dp(workbench_style_tokens().radius.medium, dpi),
    ));
    commands.push(NativeDrawCommand::PushClip {
        rect: body_viewport,
    });
    commands.push(text_command(
        &inspector.body,
        Rect {
            x: body_viewport.x,
            y: body_viewport.y - layout.inspector_scroll_y,
            width: body_viewport.width,
            height: layout.inspector_body_content_height,
        },
        TextRole::Body,
        ColorRole::PrimaryText,
        TextWeight::Regular,
        HorizontalAlign::Start,
        TextWrap::Word,
    ));
    commands.push(NativeDrawCommand::PopClip);
    paint_workbench_scrollbar(commands, layout.inspector_scrollbar);
}

fn fill(rect: Rect, fill: NativeDrawFill) -> NativeDrawCommand {
    NativeDrawCommand::FillRect { rect, fill }
}

fn stroke(rect: Rect, stroke: NativeDrawFill) -> NativeDrawCommand {
    NativeDrawCommand::StrokeRect {
        rect,
        stroke,
        width: 1,
    }
}

fn round_fill(rect: Rect, fill: NativeDrawFill, radius: i32) -> NativeDrawCommand {
    NativeDrawCommand::RoundFill { rect, fill, radius }
}

fn round_rect(
    rect: Rect,
    fill: NativeDrawFill,
    stroke: Option<NativeDrawFill>,
    radius: i32,
) -> NativeDrawCommand {
    NativeDrawCommand::RoundRect {
        rect,
        fill,
        stroke,
        radius,
    }
}

fn paint_elevated_surface(
    commands: &mut Vec<NativeDrawCommand>,
    rect: Rect,
    dpi: Dpi,
    radius: Dp,
    fill: NativeDrawFill,
) {
    commands.push(round_fill(
        Rect {
            x: rect.x,
            y: rect.y + scale(2, dpi),
            ..rect
        },
        NativeDrawFill::RoleWithAlpha {
            role: ColorRole::SecondaryText,
            alpha: 15,
        },
        scale_dp(radius, dpi),
    ));
    commands.push(round_rect(
        rect,
        fill,
        Some(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Border,
            alpha: 28,
        }),
        scale_dp(radius, dpi),
    ));
}

fn workbench_scrollbar_geometry(
    viewport: Rect,
    content_height: i32,
    scroll_y: i32,
    scroll_max: i32,
    x: i32,
    dpi: Dpi,
) -> Option<ZsWorkbenchScrollbarGeometry> {
    if scroll_max <= 0 || viewport.height <= 0 {
        return None;
    }
    let track_width = scale(3, dpi).max(1);
    let track = Rect {
        x,
        y: viewport.y,
        width: track_width,
        height: viewport.height,
    };
    let minimum_thumb = scale(24, dpi).min(viewport.height);
    let thumb_height = ((i64::from(viewport.height) * i64::from(viewport.height))
        / i64::from(content_height.max(1)))
    .clamp(i64::from(minimum_thumb), i64::from(viewport.height)) as i32;
    let travel = (viewport.height - thumb_height).max(0);
    let thumb_y = viewport.y
        + ((i64::from(scroll_y.clamp(0, scroll_max)) * i64::from(travel))
            / i64::from(scroll_max.max(1))) as i32;
    Some(ZsWorkbenchScrollbarGeometry {
        track,
        thumb: Rect {
            x,
            y: thumb_y,
            width: track_width,
            height: thumb_height,
        },
    })
}

fn paint_workbench_scrollbar(
    commands: &mut Vec<NativeDrawCommand>,
    geometry: Option<ZsWorkbenchScrollbarGeometry>,
) {
    let Some(geometry) = geometry else {
        return;
    };
    commands.push(round_fill(
        geometry.thumb,
        NativeDrawFill::RoleWithAlpha {
            role: ColorRole::SecondaryText,
            alpha: 88,
        },
        (geometry.thumb.width / 2).max(1),
    ));
}

fn icon_command(icon: ZsWorkbenchIcon, bounds: Rect, color: ColorRole) -> NativeDrawCommand {
    NativeDrawCommand::Icon(
        NativeDrawIconCommand::new(icon, bounds, NativeIconColorMode::ThemeAware).with_color(color),
    )
}

fn icon_bounds(bounds: Rect, dpi: Dpi) -> Rect {
    let size = scale_dp(workbench_style_tokens().controls.standard_icon, dpi);
    Rect {
        x: bounds.x + (bounds.width - size) / 2,
        y: bounds.y + (bounds.height - size) / 2,
        width: size,
        height: size,
    }
}

#[allow(clippy::too_many_arguments)]
fn text_command(
    text: &str,
    bounds: Rect,
    role: TextRole,
    color: ColorRole,
    weight: TextWeight,
    horizontal_align: HorizontalAlign,
    wrap: TextWrap,
) -> NativeDrawCommand {
    NativeDrawCommand::Text(NativeDrawTextCommand::new(
        text,
        bounds,
        workbench_text_style(role, color, weight, horizontal_align, wrap),
    ))
}

fn workbench_text_style(
    role: TextRole,
    color: ColorRole,
    weight: TextWeight,
    horizontal_align: HorizontalAlign,
    wrap: TextWrap,
) -> SemanticTextStyle {
    SemanticTextStyle {
        role,
        color,
        weight,
        horizontal_align,
        vertical_align: if wrap == TextWrap::Word {
            VerticalAlign::Start
        } else {
            VerticalAlign::Center
        },
        wrap,
        ellipsis: wrap == TextWrap::NoWrap,
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn clipped_to(rect: Rect, clip: Rect) -> Rect {
    let left = rect.x.max(clip.x);
    let top = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    Rect {
        x: left,
        y: top,
        width: (right - left).max(0),
        height: (bottom - top).max(0),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZsWorkbenchRuntime {
    spec: ZsWorkbenchSpec,
    surface: Rect,
    dpi: Dpi,
    layout: ZsWorkbenchLayoutPlan,
    pressed_region_id: Option<String>,
}

impl ZsWorkbenchRuntime {
    pub fn new(spec: ZsWorkbenchSpec, surface: Rect, dpi: Dpi) -> Self {
        let layout = spec.layout(surface, dpi);
        Self {
            spec,
            surface,
            dpi,
            layout,
            pressed_region_id: None,
        }
    }

    pub fn spec(&self) -> &ZsWorkbenchSpec {
        &self.spec
    }

    pub fn layout(&self) -> &ZsWorkbenchLayoutPlan {
        &self.layout
    }

    pub fn set_surface(&mut self, surface: Rect, dpi: Dpi) {
        self.surface = surface;
        self.dpi = dpi;
        self.rebuild();
    }

    pub fn replace_spec(&mut self, spec: ZsWorkbenchSpec) {
        self.spec = spec;
        self.rebuild();
    }

    pub fn scroll_messages(&mut self, delta_y: i32) -> bool {
        let next = (self.spec.message_scroll_y + delta_y).clamp(0, self.layout.message_scroll_max);
        if next == self.spec.message_scroll_y {
            return false;
        }
        self.spec.message_scroll_y = next;
        self.rebuild();
        true
    }

    pub fn scroll_sidebar(&mut self, delta_y: i32) -> bool {
        let next = (self.spec.sidebar.scroll_y + delta_y).clamp(0, self.layout.sidebar_scroll_max);
        if next == self.spec.sidebar.scroll_y {
            return false;
        }
        self.spec.sidebar.scroll_y = next;
        self.rebuild();
        true
    }

    pub fn scroll_inspector(&mut self, delta_y: i32) -> bool {
        let Some(inspector) = self.spec.inspector.as_mut() else {
            return false;
        };
        let next = (inspector.scroll_y + delta_y).clamp(0, self.layout.inspector_scroll_max);
        if next == inspector.scroll_y {
            return false;
        }
        inspector.scroll_y = next;
        self.rebuild();
        true
    }

    pub fn pointer_down(&mut self, point: Point) -> bool {
        self.pressed_region_id = self.layout.region_at(point).map(|region| region.id.clone());
        self.pressed_region_id.is_some()
    }

    pub fn pointer_up(&mut self, point: Point) -> Option<ZsWorkbenchInteractionEvent> {
        let pressed = self.pressed_region_id.take()?;
        let region = self.layout.region_at(point)?;
        let event = (region.id == pressed)
            .then(|| zs_workbench_event_for_region(region))
            .flatten()?;
        self.apply_interaction(&event);
        Some(event)
    }

    pub fn apply_interaction(&mut self, event: &ZsWorkbenchInteractionEvent) -> bool {
        let changed = zs_workbench_apply_interaction(&mut self.spec, event);
        if changed {
            self.rebuild();
        }
        changed
    }

    pub fn pointer_down_update(&mut self, point: Point) -> ZsWorkbenchInteractionUpdate {
        ZsWorkbenchInteractionUpdate {
            redraw: self.pointer_down(point),
            events: Vec::new(),
        }
    }

    pub fn pointer_up_update(&mut self, point: Point) -> ZsWorkbenchInteractionUpdate {
        match self.pointer_up(point) {
            Some(event) => ZsWorkbenchInteractionUpdate {
                redraw: true,
                events: vec![event],
            },
            None => ZsWorkbenchInteractionUpdate::default(),
        }
    }

    pub fn scroll_update(&mut self, delta_y: i32) -> ZsWorkbenchInteractionUpdate {
        ZsWorkbenchInteractionUpdate {
            redraw: self.scroll_messages(delta_y),
            events: Vec::new(),
        }
    }

    pub fn sidebar_scroll_update(&mut self, delta_y: i32) -> ZsWorkbenchInteractionUpdate {
        ZsWorkbenchInteractionUpdate {
            redraw: self.scroll_sidebar(delta_y),
            events: Vec::new(),
        }
    }

    pub fn inspector_scroll_update(&mut self, delta_y: i32) -> ZsWorkbenchInteractionUpdate {
        ZsWorkbenchInteractionUpdate {
            redraw: self.scroll_inspector(delta_y),
            events: Vec::new(),
        }
    }

    pub fn draw_plan(&self) -> NativeDrawPlan {
        zs_workbench_native_draw_plan(&self.spec, &self.layout)
    }

    fn rebuild(&mut self) {
        self.layout = self.spec.layout(self.surface, self.dpi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> ZsWorkbenchSpec {
        let sidebar = ZsWorkbenchSidebarSpec::new("Workspace")
            .primary_action(ZsWorkbenchActionSpec::new(
                "new",
                "New task",
                ZsWorkbenchIcon::Add,
            ))
            .group(
                ZsWorkbenchConversationGroupSpec::new("today", "Today")
                    .conversation(
                        ZsWorkbenchConversationSpec::new("thread-1", "Build native UI")
                            .selected(true),
                    )
                    .conversation(ZsWorkbenchConversationSpec::new(
                        "thread-2",
                        "Review platform support",
                    )),
            );
        let composer = ZsWorkbenchComposerSpec::new("Ask or describe a task")
            .draft("Add a reusable workbench shell")
            .action(ZsWorkbenchActionSpec::new(
                "attach",
                "",
                ZsWorkbenchIcon::Attach,
            ));
        ZsWorkbenchSpec::new("Native UI workbench", sidebar, composer)
            .message(
                ZsWorkbenchMessageSpec::new("user-1", ZsWorkbenchMessageRole::User)
                    .block(ZsWorkbenchContentBlock::paragraph(
                        "Create a reusable desktop workbench layout.",
                    )),
            )
            .message(
                ZsWorkbenchMessageSpec::new("assistant-1", ZsWorkbenchMessageRole::Assistant)
                    .block(ZsWorkbenchContentBlock::paragraph(
                        "The shell is separated into navigation, timeline, composer and inspector regions.",
                    ))
                    .block(ZsWorkbenchContentBlock::tool_with_status_label(
                        "Update files",
                        "Added shared layout and draw plans",
                        ZsWorkbenchToolStatus::Succeeded,
                        "Completed",
                    ))
                    .action(ZsWorkbenchActionSpec::new(
                        "copy",
                        "Copy",
                        ZsWorkbenchIcon::Copy,
                    )),
            )
            .inspector(
                ZsWorkbenchInspectorSpec::new("Inspector")
                    .selected_tab("changes")
                    .tab(ZsWorkbenchActionSpec::new(
                        "changes",
                        "Changes",
                        ZsWorkbenchIcon::Code,
                    ))
                    .body("Changed files and runtime output are application-provided content."),
            )
    }

    #[test]
    fn tool_status_copy_is_application_owned() {
        let implicit =
            ZsWorkbenchContentBlock::tool("Build", "Finished", ZsWorkbenchToolStatus::Succeeded);
        let explicit = ZsWorkbenchContentBlock::tool_with_status_label(
            "Build",
            "Finished",
            ZsWorkbenchToolStatus::Succeeded,
            "已完成",
        );

        assert!(matches!(
            implicit,
            ZsWorkbenchContentBlock::Tool {
                status_label: None,
                ..
            }
        ));
        assert!(matches!(
            explicit,
            ZsWorkbenchContentBlock::Tool {
                status_label: Some(label),
                ..
            } if label == "已完成"
        ));
    }

    #[test]
    fn explicit_child_specs_flatten_into_the_compatible_workbench_runtime() {
        let sidebar = ZsWorkbenchSidebarSpec::new("Workspace");
        let composer = ZsComposerSpec::new("Write a message").draft("Hello");
        let timeline = ZsMessageTimelineSpec::new()
            .message(
                ZsWorkbenchMessageSpec::new("message-1", ZsWorkbenchMessageRole::Assistant)
                    .block(ZsWorkbenchContentBlock::paragraph("Ready")),
            )
            .scroll_y(42);
        let inspector = ZsInspectorPanelSpec::new("Inspector")
            .selected_tab("details")
            .tab(ZsWorkbenchActionSpec::new(
                "details",
                "Details",
                ZsWorkbenchIcon::Inspector,
            ));

        let flattened = ZsWorkbenchShellSpec::new("Workbench", sidebar, composer)
            .subtitle("Shared declaration")
            .toolbar_action(ZsWorkbenchActionSpec::new(
                "search",
                "Search",
                ZsWorkbenchIcon::Search,
            ))
            .timeline(timeline)
            .inspector(inspector)
            .into_workbench();

        assert_eq!(flattened.title, "Workbench");
        assert_eq!(flattened.subtitle.as_deref(), Some("Shared declaration"));
        assert_eq!(flattened.toolbar_actions.len(), 1);
        assert_eq!(flattened.messages.len(), 1);
        assert_eq!(flattened.composer.draft, "Hello");
        assert_eq!(flattened.message_scroll_y, 42);
        assert_eq!(
            flattened
                .inspector
                .as_ref()
                .and_then(|inspector| inspector.selected_tab_id.as_deref()),
            Some("details")
        );
    }

    #[test]
    fn layout_has_stable_sidebar_timeline_composer_and_inspector_regions() {
        let spec = sample_spec();
        let profile = workbench_profile();
        let plan = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            Dpi::standard(),
        );

        assert_eq!(
            plan.metrics.sidebar.width,
            scale_dp(profile.sidebar_width, Dpi::standard())
        );
        assert_eq!(
            plan.metrics.top_bar.height,
            scale_dp(profile.top_bar_height, Dpi::standard())
        );
        assert_eq!(
            plan.metrics.composer_band.height,
            scale_dp(profile.composer_height, Dpi::standard())
        );
        assert_eq!(
            plan.metrics.inspector.expect("inspector").width,
            scale_dp(profile.inspector_width, Dpi::standard())
        );
        assert_eq!(plan.messages.len(), 2);
        assert!(plan.regions.iter().any(|region| {
            region.kind == ZsWorkbenchRegionKind::Conversation && region.id == "thread-1"
        }));
    }

    #[test]
    fn layout_collapses_sidebar_and_hides_inspector_on_narrow_surfaces() {
        let mut spec = sample_spec();
        spec.sidebar.collapsed = true;
        let plan = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 760,
                height: 700,
            },
            Dpi::standard(),
        );

        assert_eq!(
            plan.metrics.sidebar.width,
            scale_dp(workbench_profile().collapsed_sidebar_width, Dpi::standard())
        );
        assert!(plan.metrics.inspector.is_none());
        assert!(plan.metrics.content.width > 0);
    }

    #[test]
    fn layout_scales_stable_shell_dimensions_for_high_dpi() {
        let spec = sample_spec();
        let dpi = Dpi::new(144.0);
        let plan = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1200,
            },
            dpi,
        );
        let expected_radius = scale_dp(workbench_style_tokens().radius.medium, dpi);
        let profile = workbench_profile();

        assert_eq!(plan.dpi, dpi);
        assert_eq!(
            plan.metrics.sidebar.width,
            scale_dp(profile.sidebar_width, dpi)
        );
        assert_eq!(
            plan.metrics.top_bar.height,
            scale_dp(profile.top_bar_height, dpi)
        );
        assert_eq!(
            plan.metrics.composer_band.height,
            scale_dp(profile.composer_height, dpi)
        );
        assert!(spec
            .native_draw_plan(plan.metrics.surface, plan.dpi)
            .commands
            .iter()
            .any(|command| matches!(
                command,
                NativeDrawCommand::RoundRect { radius, .. } if *radius == expected_radius
            )));
    }

    #[test]
    fn paint_plan_contains_navigation_messages_tools_and_composer() {
        let spec = sample_spec();
        let dpi = Dpi::standard();
        let draw = spec.native_draw_plan(
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            dpi,
        );
        let expected_radius = scale_dp(workbench_style_tokens().radius.medium, dpi);

        assert!(draw.command_count() >= 30);
        assert!(draw.text_count() >= 15);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect { radius, .. } if *radius == expected_radius
        )));
        assert!(draw
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::Icon(_))));
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(command) if command.icon == ZsWorkbenchIcon::Enter
        )));
        assert!(!draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command) if command.style.role == TextRole::Icon
        )));
    }

    #[test]
    fn runtime_routes_conversation_and_submit_events() {
        let mut runtime = ZsWorkbenchRuntime::new(
            sample_spec(),
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            Dpi::standard(),
        );
        let conversation = runtime
            .layout()
            .regions
            .iter()
            .find(|region| {
                region.kind == ZsWorkbenchRegionKind::Conversation && region.id == "thread-1"
            })
            .expect("conversation region")
            .bounds;
        let point = Point {
            x: conversation.x + 4,
            y: conversation.y + 4,
        };
        assert!(runtime.pointer_down(point));
        assert_eq!(
            runtime.pointer_up(point),
            Some(ZsWorkbenchInteractionEvent::SelectConversation {
                conversation_id: "thread-1".to_string(),
            })
        );

        let selected = runtime
            .spec()
            .sidebar
            .groups
            .iter()
            .flat_map(|group| group.conversations.iter())
            .find(|conversation| conversation.id == "thread-1")
            .expect("selected conversation");
        assert!(selected.selected);

        let submit = runtime
            .layout()
            .regions
            .iter()
            .find(|region| region.kind == ZsWorkbenchRegionKind::Submit)
            .expect("submit region")
            .bounds;
        let point = Point {
            x: submit.x + 4,
            y: submit.y + 4,
        };
        assert!(runtime.pointer_down(point));
        assert_eq!(
            runtime.pointer_up(point),
            Some(ZsWorkbenchInteractionEvent::Submit)
        );
    }

    #[test]
    fn runtime_clamps_message_scroll() {
        let mut spec = sample_spec();
        for index in 0..20 {
            spec.messages.push(
                ZsWorkbenchMessageSpec::new(
                    format!("extra-{index}"),
                    ZsWorkbenchMessageRole::Assistant,
                )
                .block(ZsWorkbenchContentBlock::paragraph(
                    "Additional content keeps the timeline scrollable.",
                )),
            );
        }
        let mut runtime = ZsWorkbenchRuntime::new(
            spec,
            Rect {
                x: 0,
                y: 0,
                width: 900,
                height: 600,
            },
            Dpi::standard(),
        );
        assert!(runtime.scroll_messages(100_000));
        assert_eq!(
            runtime.layout().message_scroll_y,
            runtime.layout().message_scroll_max
        );
        assert!(!runtime.scroll_messages(1));
    }

    #[test]
    fn overscan_messages_are_measured_but_screen_off_actions_are_not_interactive() {
        let mut spec = ZsWorkbenchSpec::new(
            "Workbench",
            ZsWorkbenchSidebarSpec::new("Workspace"),
            ZsWorkbenchComposerSpec::new("Message"),
        )
        .message_scroll_y(720);
        for index in 0..32 {
            spec.messages.push(
                ZsWorkbenchMessageSpec::new(
                    format!("message-{index}"),
                    ZsWorkbenchMessageRole::Assistant,
                )
                .block(ZsWorkbenchContentBlock::paragraph(format!(
                    "overscan paragraph {index} with enough text to remain unique"
                )))
                .action(ZsWorkbenchActionSpec::new(
                    format!("copy-{index}"),
                    "Copy",
                    ZsWorkbenchIcon::Copy,
                )),
            );
        }
        let plan = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 900,
                height: 520,
            },
            Dpi::standard(),
        );
        let timeline = plan.metrics.timeline;
        let offscreen = plan
            .messages
            .iter()
            .find(|message| !rects_intersect(message.bounds, timeline))
            .expect("overscan should materialize at least one screen-off message");
        let offscreen_text = match &spec.messages[offscreen.message_index].blocks[0] {
            ZsWorkbenchContentBlock::Paragraph { text } => text,
            _ => unreachable!(),
        };
        let draw = zs_workbench_native_draw_plan(&spec, &plan);
        assert!(draw.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command) if command.text == *offscreen_text
        )));
        assert!(plan
            .regions
            .iter()
            .filter(|region| region.kind == ZsWorkbenchRegionKind::MessageAction)
            .all(|region| {
                region.bounds.x >= timeline.x
                    && region.bounds.y >= timeline.y
                    && region.bounds.x + region.bounds.width <= timeline.x + timeline.width
                    && region.bounds.y + region.bounds.height <= timeline.y + timeline.height
            }));
    }

    #[test]
    fn narrow_user_message_uses_the_painted_bubble_width_for_measurement() {
        let text = "窄窗口中的用户消息需要使用气泡实际宽度进行系统文字测量。";
        let spec = ZsWorkbenchSpec::new(
            "Workbench",
            ZsWorkbenchSidebarSpec::new("Workspace"),
            ZsWorkbenchComposerSpec::new("Message"),
        )
        .message(
            ZsWorkbenchMessageSpec::new("user", ZsWorkbenchMessageRole::User)
                .block(ZsWorkbenchContentBlock::paragraph(text)),
        );
        let surface = Rect {
            x: 0,
            y: 0,
            width: 360,
            height: 640,
        };
        let dpi = Dpi::standard();
        let estimated = spec.layout(surface, dpi);
        let measured_width = estimated.messages[0].content_bounds.width;
        let mut measurements = crate::view::ViewTextMeasurements::default();
        measurements.insert(
            text,
            workbench_text_style(
                TextRole::Body,
                ColorRole::PrimaryText,
                TextWeight::Regular,
                HorizontalAlign::Start,
                TextWrap::Word,
            ),
            measured_width,
            crate::Size {
                width: measured_width,
                height: 73,
            },
        );
        let measured = spec.layout_with_text_measurements(surface, dpi, 1.0, &measurements);

        assert_eq!(measured.messages[0].content_bounds.width, measured_width);
        assert_eq!(measured.messages[0].blocks[0].bounds.height, 73);
    }

    #[test]
    fn sidebar_and_inspector_share_scrollbar_track_and_thumb_geometry() {
        let mut sidebar = ZsWorkbenchSidebarSpec::new("Workspace");
        let mut group = ZsWorkbenchConversationGroupSpec::new("all", "All");
        for index in 0..40 {
            group.conversations.push(ZsWorkbenchConversationSpec::new(
                format!("thread-{index}"),
                format!("Thread {index}"),
            ));
        }
        sidebar.groups.push(group);
        let inspector = ZsWorkbenchInspectorSpec::new("Inspector").body(
            (0..80)
                .map(|index| format!("Line {index}\n"))
                .collect::<String>(),
        );
        let spec = ZsWorkbenchSpec::new(
            "Workbench",
            sidebar,
            ZsWorkbenchComposerSpec::new("Message"),
        )
        .inspector(inspector);
        let plan = spec.layout(
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 420,
            },
            Dpi::standard(),
        );

        let sidebar_scrollbar = plan.sidebar_scrollbar.expect("sidebar scrollbar");
        assert_eq!(
            sidebar_scrollbar.track.height,
            plan.metrics
                .sidebar_list_viewport
                .expect("sidebar viewport")
                .height
        );
        assert!(sidebar_scrollbar.thumb.height > 0);
        assert!(sidebar_scrollbar.thumb.height < sidebar_scrollbar.track.height);

        let inspector_scrollbar = plan.inspector_scrollbar.expect("inspector scrollbar");
        assert_eq!(
            inspector_scrollbar.track.height,
            plan.inspector_body_viewport
                .expect("inspector viewport")
                .height
        );
        assert!(inspector_scrollbar.thumb.height > 0);
        assert!(inspector_scrollbar.thumb.height < inspector_scrollbar.track.height);
    }

    #[test]
    fn retained_text_measurements_keep_long_overscan_geometry_available() {
        let style = workbench_text_style(
            TextRole::Body,
            ColorRole::PrimaryText,
            TextWeight::Regular,
            HorizontalAlign::Start,
            TextWrap::Word,
        );
        let first_text = "first long message ".repeat(128);
        let second_text = "second long message ".repeat(128);
        let mut first_frame = crate::view::ViewTextMeasurements::default();
        first_frame.insert(
            &first_text,
            style,
            480,
            crate::Size {
                width: 480,
                height: 360,
            },
        );
        let mut next_frame = first_frame.clone();
        next_frame.insert(
            &second_text,
            style,
            480,
            crate::Size {
                width: 480,
                height: 420,
            },
        );

        assert_eq!(
            next_frame.measure(&first_text, style, 480),
            Some(crate::Size {
                width: 480,
                height: 360,
            })
        );
        assert_eq!(
            next_frame.measure(&second_text, style, 480),
            Some(crate::Size {
                width: 480,
                height: 420,
            })
        );
    }
}
