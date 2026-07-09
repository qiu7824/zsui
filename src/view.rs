use std::marker::PhantomData;

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

    #[cfg(feature = "checkbox")]
    pub fn on_toggle(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::Checkbox { on_toggle, .. } = &mut self.kind {
            *on_toggle = Some(message);
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
    column(items.into_iter().map(|item| render(item)))
}

pub fn spacer<Msg>() -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Spacer)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewEvent {
    Click { widget: WidgetId },
    TextChanged { widget: WidgetId, value: String },
    Toggled { widget: WidgetId, checked: bool },
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

    pub fn target_kind_at(&self, point: Point) -> Option<ViewHitTargetKind> {
        self.hit_target_at(point).map(|target| target.kind)
    }

    pub fn click_event_at(&self, point: Point) -> Option<ViewEvent> {
        self.target_at(point)
            .map(|widget| ViewEvent::Click { widget })
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

        let child_bounds = split_child_bounds(cx.bounds, &self.kind, self.children.len());
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
            (None, _) => false,
        }
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        let mut hit_targets = Vec::new();
        self.collect_hit_targets(&mut hit_targets);
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
        }

        self.children
            .iter()
            .find_map(|child| child.widget_checked_value(widget))
    }

    fn collect_hit_targets(&self, hit_targets: &mut Vec<ViewHitTarget>) {
        if let (Some(widget), Some(bounds)) = (self.id, self.bounds) {
            hit_targets.push(ViewHitTarget::with_kind(
                widget,
                bounds,
                self.hit_target_kind(),
            ));
        }

        for child in &self.children {
            child.collect_hit_targets(hit_targets);
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
            _ => ViewHitTargetKind::Unknown,
        }
    }
}

fn split_child_bounds<Msg>(
    bounds: Rect,
    kind: &ViewNodeKind<Msg>,
    child_count: usize,
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
        } => {
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
        _ => vec![bounds; child_count],
    }
}

#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "textbox",
    feature = "checkbox"
))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "button", feature = "textbox", feature = "checkbox"))]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Msg {
        SaveClicked,
        #[cfg(feature = "textbox")]
        NameChanged(String),
        #[cfg(feature = "checkbox")]
        DarkModeChanged(bool),
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
            plan.click_event_at(Point { x: 150, y: 90 }),
            Some(ViewEvent::Click { widget: save_id })
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
    fn app_context_keeps_commands_explicit() {
        let mut cx = AppCx::new();

        cx.command(Command::OpenSettings);
        cx.ui_command(crate::UiCommand::app(crate::CommandId("view.save")));
        cx.quit();

        assert_eq!(cx.commands(), &[Command::OpenSettings]);
        assert_eq!(cx.ui_commands()[0].id, crate::CommandId("view.save"));
        assert!(cx.quit_requested());
    }
}
