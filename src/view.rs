use std::{
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

#[cfg(feature = "button")]
use crate::render_protocol::TextRole;
#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "textbox",
    feature = "checkbox"
))]
use crate::render_protocol::{NativeDrawTextCommand, SemanticTextStyle};
use crate::{
    geometry::{ComponentId, Dp, Dpi, LayoutNode, LayoutOutput, Point, Rect},
    render_protocol::{ColorRole, NativeDrawCommand, NativeDrawFill, NativeDrawPlan},
    style::ThemeColorToken,
    Command, UiCommand,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<WidgetId> for ComponentId {
    fn from(value: WidgetId) -> Self {
        Self(value.0)
    }
}

impl From<ComponentId> for WidgetId {
    fn from(value: ComponentId) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewStackDirection {
    Row,
    Column,
}

#[derive(Debug, Clone)]
pub enum ViewNodeKind<Msg> {
    #[doc(hidden)]
    __Message(PhantomData<fn() -> Msg>),
    #[cfg(feature = "label")]
    Text {
        text: String,
    },
    #[cfg(feature = "button")]
    Button {
        label: String,
        on_click: Option<Msg>,
    },
    #[cfg(feature = "textbox")]
    Textbox {
        value: String,
        on_change: Option<fn(String) -> Msg>,
    },
    #[cfg(feature = "checkbox")]
    Checkbox {
        label: String,
        checked: bool,
        on_toggle: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "toggle")]
    Toggle {
        checked: bool,
        on_toggle: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "list")]
    List {
        selected_index: Option<usize>,
        on_select: Option<fn(usize) -> Msg>,
    },
    #[cfg(feature = "scroll")]
    Scroll {
        offset_y: Dp,
        content_height: Option<Dp>,
        on_scroll: Option<fn(Dp) -> Msg>,
    },
    Stack {
        direction: ViewStackDirection,
    },
    Spacer,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewStyle {
    pub padding: Option<Dp>,
    pub radius: Option<Dp>,
    pub background: Option<ThemeColorToken>,
}

#[derive(Debug, Clone)]
pub struct ViewNode<Msg> {
    pub id: Option<WidgetId>,
    pub kind: ViewNodeKind<Msg>,
    pub style: ViewStyle,
    pub children: Vec<ViewNode<Msg>>,
    bounds: Option<Rect>,
    message: PhantomData<fn() -> Msg>,
}

impl<Msg> ViewNode<Msg> {
    pub fn new(kind: ViewNodeKind<Msg>) -> Self {
        Self {
            id: None,
            kind,
            style: ViewStyle::default(),
            children: Vec::new(),
            bounds: None,
            message: PhantomData,
        }
    }

    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn padding(mut self, padding: Dp) -> Self {
        self.style.padding = Some(padding);
        self
    }

    pub fn radius(mut self, radius: Dp) -> Self {
        self.style.radius = Some(radius);
        self
    }

    pub fn bg(mut self, token: ThemeColorToken) -> Self {
        self.style.background = Some(token);
        self
    }

    pub fn child(mut self, child: ViewNode<Msg>) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }
}

impl<Msg: Clone> ViewNode<Msg> {
    #[cfg(feature = "button")]
    pub fn on_click(mut self, message: Msg) -> Self {
        if let ViewNodeKind::Button { on_click, .. } = &mut self.kind {
            *on_click = Some(message);
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn on_change(mut self, message: fn(String) -> Msg) -> Self {
        if let ViewNodeKind::Textbox { on_change, .. } = &mut self.kind {
            *on_change = Some(message);
        }
        self
    }

    #[cfg(any(feature = "checkbox", feature = "toggle"))]
    pub fn on_toggle(mut self, message: fn(bool) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { on_toggle, .. } => *on_toggle = Some(message),
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { on_toggle, .. } => *on_toggle = Some(message),
            _ => {}
        }
        self
    }

    #[cfg(feature = "list")]
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        if let ViewNodeKind::List { selected_index, .. } = &mut self.kind {
            *selected_index = index;
        }
        self
    }

    #[cfg(feature = "list")]
    pub fn on_select(mut self, message: fn(usize) -> Msg) -> Self {
        if let ViewNodeKind::List { on_select, .. } = &mut self.kind {
            *on_select = Some(message);
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn scroll_y(mut self, offset_y: Dp) -> Self {
        if let ViewNodeKind::Scroll {
            offset_y: current, ..
        } = &mut self.kind
        {
            *current = offset_y;
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn content_height(mut self, height: Dp) -> Self {
        if let ViewNodeKind::Scroll { content_height, .. } = &mut self.kind {
            *content_height = Some(height);
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn on_scroll(mut self, message: fn(Dp) -> Msg) -> Self {
        if let ViewNodeKind::Scroll { on_scroll, .. } = &mut self.kind {
            *on_scroll = Some(message);
        }
        self
    }
}

#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Text { text: text.into() })
}

#[cfg(feature = "button")]
pub fn button<Msg>(label: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Button {
        label: label.into(),
        on_click: None,
    })
}

#[cfg(feature = "textbox")]
pub fn textbox<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Textbox {
        value: value.into(),
        on_change: None,
    })
}

#[cfg(feature = "checkbox")]
pub fn checkbox<Msg>(label: impl Into<String>, checked: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Checkbox {
        label: label.into(),
        checked,
        on_toggle: None,
    })
}

#[cfg(feature = "toggle")]
pub fn toggle<Msg>(checked: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Toggle {
        checked,
        on_toggle: None,
    })
}

pub fn row<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Row,
    })
    .children(children)
}

pub fn column<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Column,
    })
    .children(children)
}

#[cfg(feature = "list")]
pub fn list<T, Msg>(
    items: impl IntoIterator<Item = T>,
    mut render: impl FnMut(T) -> ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::List {
        selected_index: None,
        on_select: None,
    })
    .children(items.into_iter().map(|item| render(item)))
}

#[cfg(feature = "scroll")]
pub fn scroll<Msg>(child: ViewNode<Msg>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Scroll {
        offset_y: Dp::new(0.0),
        content_height: None,
        on_scroll: None,
    })
    .child(child)
}

pub fn spacer<Msg>() -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Spacer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewEvent {
    Click {
        widget: WidgetId,
    },
    TextChanged {
        widget: WidgetId,
        value: String,
    },
    Toggled {
        widget: WidgetId,
        checked: bool,
    },
    #[cfg(feature = "scroll")]
    ScrollBy {
        widget: WidgetId,
        delta_y: Dp,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewHitTarget {
    pub widget: WidgetId,
    pub bounds: Rect,
    pub kind: ViewHitTargetKind,
}

impl ViewHitTarget {
    pub const fn new(widget: WidgetId, bounds: Rect) -> Self {
        Self {
            widget,
            bounds,
            kind: ViewHitTargetKind::Unknown,
        }
    }

    pub const fn with_kind(widget: WidgetId, bounds: Rect, kind: ViewHitTargetKind) -> Self {
        Self {
            widget,
            bounds,
            kind,
        }
    }

    pub const fn contains(self, point: Point) -> bool {
        self.bounds.contains(point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewHitTargetKind {
    Unknown,
    Button,
    Textbox,
    Checkbox,
    Toggle,
    #[cfg(feature = "scroll")]
    Scroll,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewInteractionPlan {
    pub hit_targets: Vec<ViewHitTarget>,
}

impl ViewInteractionPlan {
    pub fn new(hit_targets: impl IntoIterator<Item = ViewHitTarget>) -> Self {
        Self {
            hit_targets: hit_targets.into_iter().collect(),
        }
    }

    pub fn from_layout_output(layout: &LayoutOutput) -> Self {
        Self::new(
            layout
                .children
                .iter()
                .map(|node| ViewHitTarget::new(node.component.into(), node.bounds)),
        )
    }

    pub fn hit_target_count(&self) -> usize {
        self.hit_targets.len()
    }

    pub fn target_at(&self, point: Point) -> Option<WidgetId> {
        self.hit_target_at(point).map(|target| target.widget)
    }

    pub fn hit_target_at(&self, point: Point) -> Option<ViewHitTarget> {
        self.hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.contains(point))
    }

    pub fn hit_target_for_widget(&self, widget: WidgetId) -> Option<ViewHitTarget> {
        self.hit_targets
            .iter()
            .copied()
            .find(|target| target.widget == widget)
    }

    pub fn target_kind_at(&self, point: Point) -> Option<ViewHitTargetKind> {
        self.hit_target_at(point).map(|target| target.kind)
    }

    pub fn click_event_at(&self, point: Point) -> Option<ViewEvent> {
        self.target_at(point)
            .map(|widget| ViewEvent::Click { widget })
    }

    pub fn first_focus_target(&self) -> Option<ViewHitTarget> {
        self.hit_targets.first().copied()
    }

    pub fn next_focus_target(
        &self,
        current: Option<WidgetId>,
        offset: isize,
    ) -> Option<ViewHitTarget> {
        let len = self.hit_targets.len();
        if len == 0 {
            return None;
        }

        let current_index = current.and_then(|widget| {
            self.hit_targets
                .iter()
                .position(|target| target.widget == widget)
        });
        let next_index = match current_index {
            Some(index) => (index as isize + offset).rem_euclid(len as isize) as usize,
            None if offset < 0 => len - 1,
            None => 0,
        };
        self.hit_targets.get(next_index).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewLayoutCx {
    pub bounds: Rect,
    pub dpi: Dpi,
}

impl ViewLayoutCx {
    pub const fn new(bounds: Rect, dpi: Dpi) -> Self {
        Self { bounds, dpi }
    }
}

#[derive(Debug, Clone)]
pub struct ViewEventCx<Msg> {
    messages: Vec<Msg>,
}

impl<Msg> ViewEventCx<Msg> {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn emit(&mut self, message: Msg) {
        self.messages.push(message);
    }

    pub fn messages(&self) -> &[Msg] {
        &self.messages
    }

    pub fn into_messages(self) -> Vec<Msg> {
        self.messages
    }
}

impl<Msg> Default for ViewEventCx<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppCx {
    commands: Vec<Command>,
    ui_commands: Vec<UiCommand>,
    quit_requested: bool,
}

impl AppCx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn ui_command(&mut self, command: UiCommand) {
        self.ui_commands.push(command);
    }

    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn ui_commands(&self) -> &[UiCommand] {
        &self.ui_commands
    }

    pub const fn quit_requested(&self) -> bool {
        self.quit_requested
    }
}

#[derive(Debug, Clone, Default)]
pub struct LiveViewUpdate {
    pub redraw: bool,
    pub message_count: usize,
    pub commands: Vec<Command>,
    pub ui_commands: Vec<UiCommand>,
    pub quit_requested: bool,
    pub revision: u64,
}

trait LiveViewDriver: Send {
    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool;
    fn draw_plan(&self) -> NativeDrawPlan;
    fn interaction_plan(&self) -> ViewInteractionPlan;
    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate;
    fn widget_text_value(&self, widget: WidgetId) -> Option<String>;
    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool>;
    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)>;
    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize>;
    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId>;
    fn revision(&self) -> u64;
}

#[derive(Clone)]
pub struct SharedLiveViewRuntime {
    inner: Arc<Mutex<Box<dyn LiveViewDriver>>>,
}

impl SharedLiveViewRuntime {
    pub fn set_surface(&self, bounds: Rect, dpi: Dpi) -> bool {
        self.lock().set_surface(bounds, dpi)
    }

    pub fn draw_plan(&self) -> NativeDrawPlan {
        self.lock().draw_plan()
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        self.lock().interaction_plan()
    }

    pub fn dispatch_event(&self, event: &ViewEvent) -> LiveViewUpdate {
        self.lock().dispatch_event(event)
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.lock().widget_text_value(widget)
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.lock().widget_checked_value(widget)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.lock().widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.lock().widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.lock().widget_scroll_target(widget)
    }

    pub fn revision(&self) -> u64 {
        self.lock().revision()
    }

    fn lock(&self) -> MutexGuard<'_, Box<dyn LiveViewDriver>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for SharedLiveViewRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedLiveViewRuntime")
            .field("revision", &self.revision())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SharedLiveViewRuntime {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

struct TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    view: ViewNode<Msg>,
    bounds: Rect,
    dpi: Dpi,
    revision: u64,
}

impl<State, Msg, ViewFn, UpdateFn> TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    fn new(state: State, view_fn: ViewFn, update_fn: UpdateFn, bounds: Rect, dpi: Dpi) -> Self {
        let view = view_fn(&state);
        let mut driver = Self {
            state,
            view_fn,
            update_fn,
            view,
            bounds,
            dpi,
            revision: 0,
        };
        driver.layout();
        driver
    }

    fn layout(&mut self) {
        self.view = (self.view_fn)(&self.state);
        let mut cx = ViewLayoutCx::new(self.bounds, self.dpi);
        self.view.layout(&mut cx);
    }
}

impl<State, Msg, ViewFn, UpdateFn> LiveViewDriver
    for TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool {
        if self.bounds == bounds && self.dpi == dpi {
            return false;
        }
        self.bounds = bounds;
        self.dpi = dpi;
        self.layout();
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn draw_plan(&self) -> NativeDrawPlan {
        let mut cx = ViewPaintCx::new(self.dpi);
        self.view.paint(&mut cx);
        cx.into_plan()
    }

    fn interaction_plan(&self) -> ViewInteractionPlan {
        self.view.interaction_plan()
    }

    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate {
        let mut event_cx = ViewEventCx::new();
        self.view.event(&mut event_cx, event);
        let messages = event_cx.into_messages();
        if messages.is_empty() {
            return LiveViewUpdate {
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }

        let message_count = messages.len();
        let mut app_cx = AppCx::new();
        for message in messages {
            (self.update_fn)(&mut self.state, message, &mut app_cx);
        }
        self.layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: true,
            message_count,
            commands: app_cx.commands().to_vec(),
            ui_commands: app_cx.ui_commands().to_vec(),
            quit_requested: app_cx.quit_requested(),
            revision: self.revision,
        }
    }

    fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.view.widget_text_value(widget).map(str::to_string)
    }

    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.view.widget_checked_value(widget)
    }

    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.view.widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.view.widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.view.widget_scroll_target(widget)
    }

    fn revision(&self) -> u64 {
        self.revision
    }
}

pub fn live_view_runtime<State, Msg, ViewFn, UpdateFn>(
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    bounds: Rect,
    dpi: Dpi,
) -> SharedLiveViewRuntime
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    SharedLiveViewRuntime {
        inner: Arc::new(Mutex::new(Box::new(TypedLiveViewDriver::new(
            state, view_fn, update_fn, bounds, dpi,
        )))),
    }
}

#[derive(Debug, Clone)]
pub struct ViewPaintCx {
    pub dpi: Dpi,
    plan: NativeDrawPlan,
}

impl ViewPaintCx {
    pub fn new(dpi: Dpi) -> Self {
        Self {
            dpi,
            plan: NativeDrawPlan::default(),
        }
    }

    pub fn draw(&mut self, command: NativeDrawCommand) {
        self.plan.push(command);
    }

    pub fn plan(&self) -> &NativeDrawPlan {
        &self.plan
    }

    pub fn into_plan(self) -> NativeDrawPlan {
        self.plan
    }
}

pub trait View<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput;
    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent);
    fn paint(&self, cx: &mut ViewPaintCx);
}

impl<Msg: Clone> View<Msg> for ViewNode<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }

        let child_bounds = split_child_bounds(cx.bounds, &self.kind, self.children.len(), cx.dpi);
        for (child, bounds) in self.children.iter_mut().zip(child_bounds) {
            let mut child_cx = ViewLayoutCx {
                bounds,
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }

        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }

    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent) {
        #[cfg(feature = "list")]
        if let (
            ViewNodeKind::List {
                selected_index,
                on_select,
            },
            ViewEvent::Click { widget },
        ) = (&mut self.kind, event)
        {
            if let Some(index) = self
                .children
                .iter()
                .position(|child| child.contains_widget(*widget))
            {
                *selected_index = Some(index);
                if let Some(message) = on_select {
                    cx.emit(message(index));
                }
            }
        }

        if self.event_targets_self(event) {
            match (&mut self.kind, event) {
                #[cfg(feature = "button")]
                (ViewNodeKind::Button { on_click, .. }, ViewEvent::Click { .. }) => {
                    if let Some(message) = on_click.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "textbox")]
                (
                    ViewNodeKind::Textbox { value, on_change },
                    ViewEvent::TextChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = next_value.clone();
                    if let Some(message) = on_change {
                        cx.emit(message(next_value.clone()));
                    }
                }
                #[cfg(feature = "checkbox")]
                (
                    ViewNodeKind::Checkbox {
                        checked, on_toggle, ..
                    },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "toggle")]
                (
                    ViewNodeKind::Toggle { checked, on_toggle },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "scroll")]
                (
                    ViewNodeKind::Scroll {
                        offset_y,
                        content_height,
                        on_scroll,
                    },
                    ViewEvent::ScrollBy { delta_y, .. },
                ) => {
                    let max_offset = scroll_max_offset_y(self.bounds, *content_height);
                    let next = Dp::new((offset_y.0 + delta_y.0).clamp(0.0, max_offset.0));
                    *offset_y = next;
                    if let Some(message) = on_scroll {
                        cx.emit(message(next));
                    }
                }
                _ => {}
            }
        }

        for child in &mut self.children {
            child.event(cx, event);
        }
    }

    fn paint(&self, cx: &mut ViewPaintCx) {
        let Some(bounds) = self.bounds else {
            return;
        };

        if let Some(background) = self.style.background {
            cx.draw(NativeDrawCommand::RoundFill {
                rect: bounds,
                fill: NativeDrawFill::Role(color_role_for_token(background)),
                radius: radius_px(self.style.radius, cx.dpi),
            });
        }

        match &self.kind {
            #[cfg(feature = "label")]
            ViewNodeKind::Text { text } => {
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    text,
                    padded_bounds(bounds, self.style.padding, cx.dpi),
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "button")]
            ViewNodeKind::Button { label, .. } => {
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Control),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    radius: radius_px(self.style.radius.or(Some(Dp::new(6.0))), cx.dpi),
                });
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    padded_bounds(bounds, self.style.padding.or(Some(Dp::new(8.0))), cx.dpi),
                    SemanticTextStyle {
                        role: TextRole::Button,
                        ..SemanticTextStyle::body()
                    },
                )));
            }
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { value, .. } => {
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Surface),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Control)),
                    radius: radius_px(self.style.radius.or(Some(Dp::new(6.0))), cx.dpi),
                });
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    value,
                    padded_bounds(bounds, self.style.padding.or(Some(Dp::new(8.0))), cx.dpi),
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { label, checked, .. } => {
                let check_bounds = Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.height.min(20),
                    height: bounds.height.min(20),
                };
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: check_bounds,
                    fill: NativeDrawFill::Role(if *checked {
                        ColorRole::Accent
                    } else {
                        ColorRole::Control
                    }),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    radius: 4,
                });
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    Rect {
                        x: bounds.x + check_bounds.width + 8,
                        y: bounds.y,
                        width: (bounds.width - check_bounds.width - 8).max(0),
                        height: bounds.height,
                    },
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { checked, .. } => {
                let plan = crate::zs_toggle_render_plan(bounds, false, *checked, cx.dpi);
                for command in crate::zs_toggle_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "list")]
            ViewNodeKind::List { selected_index, .. } => {
                if let Some(bounds) = selected_index
                    .and_then(|index| self.children.get(index))
                    .and_then(ViewNode::bounds)
                {
                    cx.draw(NativeDrawCommand::RoundFill {
                        rect: bounds,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Accent,
                            alpha: 36,
                        },
                        radius: radius_px(self.style.radius.or(Some(Dp::new(4.0))), cx.dpi),
                    });
                }
            }
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => {
                cx.draw(NativeDrawCommand::PushClip { rect: bounds });
                for child in &self.children {
                    child.paint(cx);
                }
                cx.draw(NativeDrawCommand::PopClip);
                return;
            }
            ViewNodeKind::Stack { .. } | ViewNodeKind::Spacer | ViewNodeKind::__Message(_) => {}
        }

        for child in &self.children {
            child.paint(cx);
        }
    }
}

impl<Msg> ViewNode<Msg> {
    fn event_targets_self(&self, event: &ViewEvent) -> bool {
        match (self.id, event) {
            (Some(id), ViewEvent::Click { widget })
            | (Some(id), ViewEvent::TextChanged { widget, .. })
            | (Some(id), ViewEvent::Toggled { widget, .. }) => id == *widget,
            #[cfg(feature = "scroll")]
            (Some(id), ViewEvent::ScrollBy { widget, .. }) => id == *widget,
            (None, _) => false,
        }
    }

    #[cfg(any(feature = "list", feature = "scroll"))]
    fn contains_widget(&self, widget: WidgetId) -> bool {
        self.id == Some(widget)
            || self
                .children
                .iter()
                .any(|child| child.contains_widget(widget))
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        let mut hit_targets = Vec::new();
        self.collect_hit_targets(&mut hit_targets, None);
        ViewInteractionPlan { hit_targets }
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<&str> {
        if self.id == Some(widget) {
            #[cfg(feature = "textbox")]
            if let ViewNodeKind::Textbox { value, .. } = &self.kind {
                return Some(value);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_text_value(widget))
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        if self.id == Some(widget) {
            #[cfg(feature = "checkbox")]
            if let ViewNodeKind::Checkbox { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "toggle")]
            if let ViewNodeKind::Toggle { checked, .. } = &self.kind {
                return Some(*checked);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_checked_value(widget))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            return self
                .children
                .iter()
                .position(|child| child.contains_widget(widget));
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_index(widget))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            let current = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            let next = current
                .saturating_add_signed(offset)
                .min(self.children.len().saturating_sub(1));
            if next == current {
                return None;
            }
            return self.children[next]
                .first_widget_id()
                .map(|widget| (widget, next));
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_relative_widget(widget, offset))
    }

    #[cfg(feature = "list")]
    fn first_widget_id(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id))
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        if matches!(self.kind, ViewNodeKind::Scroll { .. }) && self.contains_widget(widget) {
            return self.id.or_else(|| self.first_widget_id_any());
        }

        self.children
            .iter()
            .find_map(|child| child.widget_scroll_target(widget))
    }

    #[cfg(feature = "scroll")]
    fn first_widget_id_any(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id_any))
    }

    fn collect_hit_targets(&self, hit_targets: &mut Vec<ViewHitTarget>, clip: Option<Rect>) {
        if let (Some(widget), Some(bounds)) = (self.id, self.bounds) {
            if let Some(bounds) = clipped_rect(bounds, clip) {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    self.hit_target_kind(),
                ));
            }
        }

        #[cfg(feature = "scroll")]
        let child_clip = if matches!(self.kind, ViewNodeKind::Scroll { .. }) {
            self.bounds.and_then(|bounds| clipped_rect(bounds, clip))
        } else {
            clip
        };
        #[cfg(not(feature = "scroll"))]
        let child_clip = clip;

        for child in &self.children {
            child.collect_hit_targets(hit_targets, child_clip);
        }
    }

    fn hit_target_kind(&self) -> ViewHitTargetKind {
        match &self.kind {
            #[cfg(feature = "button")]
            ViewNodeKind::Button { .. } => ViewHitTargetKind::Button,
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { .. } => ViewHitTargetKind::Textbox,
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { .. } => ViewHitTargetKind::Checkbox,
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { .. } => ViewHitTargetKind::Toggle,
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => ViewHitTargetKind::Scroll,
            _ => ViewHitTargetKind::Unknown,
        }
    }
}

fn split_child_bounds<Msg>(
    bounds: Rect,
    kind: &ViewNodeKind<Msg>,
    child_count: usize,
    _dpi: Dpi,
) -> Vec<Rect> {
    if child_count == 0 {
        return Vec::new();
    }

    match kind {
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => {
            let width = bounds.width / child_count as i32;
            (0..child_count)
                .map(|index| Rect {
                    x: bounds.x + width * index as i32,
                    y: bounds.y,
                    width,
                    height: bounds.height,
                })
                .collect()
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => split_column_child_bounds(bounds, child_count),
        #[cfg(feature = "list")]
        ViewNodeKind::List { .. } => split_column_child_bounds(bounds, child_count),
        #[cfg(feature = "scroll")]
        ViewNodeKind::Scroll {
            offset_y,
            content_height,
            ..
        } => {
            let offset_y = offset_y.to_px(_dpi).round_i32().max(0);
            let height = content_height
                .map(|height| height.to_px(_dpi).round_i32())
                .unwrap_or(bounds.height)
                .max(bounds.height);
            vec![
                Rect {
                    x: bounds.x,
                    y: bounds.y - offset_y,
                    width: bounds.width,
                    height,
                };
                child_count
            ]
        }
        _ => vec![bounds; child_count],
    }
}

fn split_column_child_bounds(bounds: Rect, child_count: usize) -> Vec<Rect> {
    let height = bounds.height / child_count as i32;
    (0..child_count)
        .map(|index| Rect {
            x: bounds.x,
            y: bounds.y + height * index as i32,
            width: bounds.width,
            height,
        })
        .collect()
}

#[cfg(any(feature = "label", feature = "button", feature = "textbox"))]
fn padded_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    let padding = padding
        .map(|value| value.to_px(dpi).round_i32())
        .unwrap_or(0);
    Rect {
        x: bounds.x + padding,
        y: bounds.y + padding,
        width: (bounds.width - padding * 2).max(0),
        height: (bounds.height - padding * 2).max(0),
    }
}

fn radius_px(radius: Option<Dp>, dpi: Dpi) -> i32 {
    radius
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0)
}

#[cfg(feature = "scroll")]
fn scroll_max_offset_y(bounds: Option<Rect>, content_height: Option<Dp>) -> Dp {
    let viewport_height = bounds
        .map(|bounds| bounds.height.max(0) as f32)
        .unwrap_or(0.0);
    let content_height = content_height
        .map(|height| height.0.max(0.0))
        .unwrap_or(viewport_height);
    Dp::new((content_height - viewport_height).max(0.0))
}

fn color_role_for_token(token: ThemeColorToken) -> ColorRole {
    match token {
        ThemeColorToken::Surface | ThemeColorToken::SurfaceRaised => ColorRole::Surface,
        ThemeColorToken::TextPrimary => ColorRole::PrimaryText,
        ThemeColorToken::TextSecondary => ColorRole::SecondaryText,
        ThemeColorToken::Accent => ColorRole::Accent,
        ThemeColorToken::Control => ColorRole::Control,
        ThemeColorToken::Danger => ColorRole::Danger,
    }
}

fn clipped_rect(rect: Rect, clip: Option<Rect>) -> Option<Rect> {
    let Some(clip) = clip else {
        return Some(rect);
    };
    let left = rect.x.max(clip.x);
    let top = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    let width = right - left;
    let height = bottom - top;
    (width > 0 && height > 0).then_some(Rect {
        x: left,
        y: top,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        feature = "button",
        feature = "textbox",
        feature = "checkbox",
        feature = "toggle",
        feature = "list"
    ))]
    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        #[cfg(feature = "button")]
        SaveClicked,
        #[cfg(feature = "textbox")]
        NameChanged(String),
        #[cfg(any(feature = "checkbox", feature = "toggle"))]
        DarkModeChanged(bool),
        #[cfg(feature = "list")]
        RowSelected(usize),
        #[cfg(feature = "scroll")]
        ScrollChanged(Dp),
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_uses_typed_messages_without_string_events() {
        let save_id = WidgetId::new(1);
        let mut view = column(vec![
            text("Clipboard history"),
            button("Save")
                .id(save_id)
                .padding(Dp::new(12.0))
                .radius(Dp::new(8.0))
                .on_click(Msg::SaveClicked),
        ]);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: save_id });

        assert_eq!(events.into_messages(), vec![Msg::SaveClicked]);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_layout_and_paint_emit_native_draw_plan() {
        let mut view: ViewNode<Msg> =
            column(vec![text("Title"), button("Copy").radius(Dp::new(8.0))]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert_eq!(output.bounds.width, 240);
        assert_eq!(paint.plan().text_count(), 2);
        assert!(paint.plan().command_count() >= 3);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_interaction_plan_maps_points_to_typed_click_events() {
        let save_id = WidgetId::new(42);
        let mut view: ViewNode<Msg> = column(vec![
            text("Title"),
            button("Save").id(save_id).on_click(Msg::SaveClicked),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 120,
            },
            Dpi::standard(),
        );
        let _output = view.layout(&mut layout);
        let plan = view.interaction_plan();

        assert_eq!(plan.hit_target_count(), 1);
        assert_eq!(
            plan.target_kind_at(Point { x: 150, y: 90 }),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.hit_target_for_widget(save_id)
                .map(|target| target.kind),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.click_event_at(Point { x: 150, y: 90 }),
            Some(ViewEvent::Click { widget: save_id })
        );
        assert_eq!(
            plan.first_focus_target().map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(
            plan.next_focus_target(None, 1).map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(plan.click_event_at(Point { x: 150, y: 20 }), None);
    }

    #[test]
    #[cfg(all(feature = "textbox", feature = "checkbox"))]
    fn input_views_map_runtime_values_into_typed_messages() {
        let name_id = WidgetId::new(2);
        let dark_id = WidgetId::new(3);
        let mut view = column(vec![
            textbox("").id(name_id).on_change(Msg::NameChanged),
            checkbox("Dark mode", false)
                .id(dark_id)
                .on_toggle(Msg::DarkModeChanged),
        ]);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: name_id,
                value: "ZSUI".to_string(),
            },
        );
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: dark_id,
                checked: true,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::NameChanged("ZSUI".to_string()),
                Msg::DarkModeChanged(true)
            ]
        );
    }

    #[test]
    #[cfg(feature = "toggle")]
    fn toggle_routes_typed_state_and_paints_shared_geometry() {
        let toggle_id = WidgetId::new(4);
        let mut view = toggle(false).id(toggle_id).on_toggle(Msg::DarkModeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: toggle_id,
                checked: true,
            },
        );

        assert_eq!(view.widget_checked_value(toggle_id), Some(true));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::Toggle);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(events.into_messages(), vec![Msg::DarkModeChanged(true)]);
    }

    #[test]
    #[cfg(all(feature = "list", feature = "label"))]
    fn list_view_routes_child_clicks_to_typed_selection_messages() {
        let first = WidgetId::new(10);
        let second = WidgetId::new(11);
        let mut view = list([(first, "One"), (second, "Two")], |(id, label)| {
            text(label).id(id)
        })
        .selected_index(Some(0))
        .on_select(Msg::RowSelected);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: second });

        assert_eq!(events.into_messages(), vec![Msg::RowSelected(1)]);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::RoundFill { .. })));
        assert_eq!(view.widget_list_index(second), Some(1));
        assert_eq!(
            view.widget_list_relative_widget(second, -1),
            Some((first, 0))
        );
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn scroll_view_offsets_children_and_clips_hit_targets() {
        let top = WidgetId::new(20);
        let bottom = WidgetId::new(21);
        let scroll_id = WidgetId::new(22);
        let mut view: ViewNode<Msg> = scroll(column([
            text("Top row").id(top),
            text("Bottom row").id(bottom),
        ]))
        .id(scroll_id)
        .content_height(Dp::new(120.0))
        .scroll_y(Dp::new(60.0))
        .on_scroll(Msg::ScrollChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 60,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let plan = view.interaction_plan();
        let mut events = ViewEventCx::new();
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: scroll_id,
                delta_y: Dp::new(-20.0),
            },
        );
        view.paint(&mut paint);

        assert_eq!(
            events.into_messages(),
            vec![Msg::ScrollChanged(Dp::new(40.0))]
        );
        assert_eq!(plan.target_at(Point { x: 20, y: 20 }), Some(bottom));
        assert_eq!(plan.hit_target_for_widget(top), None);
        assert_eq!(view.widget_scroll_target(bottom), Some(scroll_id));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PushClip { .. })));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PopClip)));
    }

    #[test]
    fn app_context_keeps_commands_explicit() {
        let mut cx = AppCx::new();

        cx.command(Command::OpenSettings);
        cx.ui_command(crate::UiCommand::app(crate::CommandId("view.save")));
        cx.quit();

        assert_eq!(cx.commands(), &[Command::OpenSettings]);
        assert_eq!(cx.ui_commands()[0].id, crate::CommandId("view.save"));
        assert!(cx.quit_requested());
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn live_view_runtime_rebuilds_from_state_after_typed_message() {
        #[derive(Clone)]
        enum CounterMsg {
            Increment,
        }

        struct CounterState {
            value: u32,
        }

        let button_id = WidgetId::new(90);
        let runtime = live_view_runtime(
            CounterState { value: 0 },
            move |state| {
                column([
                    text(format!("Count: {}", state.value)),
                    button("Increment")
                        .id(button_id)
                        .on_click(CounterMsg::Increment),
                ])
            },
            |state, message, cx| match message {
                CounterMsg::Increment => {
                    state.value += 1;
                    cx.ui_command(UiCommand::app(crate::CommandId("counter.incremented")));
                }
            },
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 160,
            },
            Dpi::standard(),
        );

        let before = runtime.draw_plan();
        assert!(before.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 0"
        )));

        let update = runtime.dispatch_event(&ViewEvent::Click { widget: button_id });

        assert!(update.redraw);
        assert_eq!(update.message_count, 1);
        assert_eq!(update.revision, 1);
        assert_eq!(
            update.ui_commands[0].id,
            crate::CommandId("counter.incremented")
        );
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 1"
        )));
    }
}
