use serde::{Deserialize, Serialize};

#[cfg(any(feature = "button", feature = "checkbox", feature = "toggle"))]
use crate::core::Command;

#[cfg(feature = "tabs")]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ZsTabId(pub u64);

#[cfg(feature = "tabs")]
impl ZsTabId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the framework-owned identity of this tab's header surface.
    ///
    /// The parent identity is part of the mapping, so two TabViews may reuse
    /// the same strongly typed tab IDs without colliding with each other or
    /// with application-assigned widget IDs.
    pub(crate) const fn header_widget_id(self, tab_view: crate::WidgetId) -> crate::WidgetId {
        crate::WidgetId::synthetic_child(tab_view, self.0)
    }
}

#[cfg(feature = "tabs")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ZsTabSpec {
    pub id: ZsTabId,
    pub label: String,
    #[serde(default)]
    pub icon: Option<crate::ZsIcon>,
}

#[cfg(feature = "tabs")]
impl ZsTabSpec {
    pub fn new(id: ZsTabId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
        }
    }

    pub const fn icon(mut self, icon: crate::ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStackDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiNodeKind {
    #[cfg(feature = "label")]
    Text {
        text: String,
    },
    #[cfg(feature = "button")]
    Button {
        label: String,
        command: Command,
    },
    #[cfg(feature = "textbox")]
    TextInput {
        label: String,
        value: String,
    },
    #[cfg(feature = "checkbox")]
    Checkbox {
        label: String,
        checked: bool,
        command: Option<Command>,
    },
    #[cfg(feature = "toggle")]
    Toggle {
        checked: bool,
        command: Option<Command>,
    },
    Stack {
        direction: UiStackDirection,
        gap: u16,
    },
    Spacer {
        size: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNode {
    pub id: String,
    pub kind: UiNodeKind,
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn new(id: impl Into<String>, kind: UiNodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            children: Vec::new(),
        }
    }

    #[cfg(feature = "label")]
    pub fn text(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(id, UiNodeKind::Text { text: text.into() })
    }

    #[cfg(feature = "button")]
    pub fn button(id: impl Into<String>, label: impl Into<String>, command: Command) -> Self {
        Self::new(
            id,
            UiNodeKind::Button {
                label: label.into(),
                command,
            },
        )
    }

    #[cfg(feature = "textbox")]
    pub fn text_input(
        id: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            UiNodeKind::TextInput {
                label: label.into(),
                value: value.into(),
            },
        )
    }

    #[cfg(feature = "checkbox")]
    pub fn checkbox(
        id: impl Into<String>,
        label: impl Into<String>,
        checked: bool,
        command: Option<Command>,
    ) -> Self {
        Self::new(
            id,
            UiNodeKind::Checkbox {
                label: label.into(),
                checked,
                command,
            },
        )
    }

    #[cfg(feature = "toggle")]
    pub fn toggle(id: impl Into<String>, checked: bool, command: Option<Command>) -> Self {
        Self::new(id, UiNodeKind::Toggle { checked, command })
    }

    pub fn stack(id: impl Into<String>, direction: UiStackDirection) -> Self {
        Self::new(id, UiNodeKind::Stack { direction, gap: 0 })
    }

    pub fn row(id: impl Into<String>) -> Self {
        Self::stack(id, UiStackDirection::Row)
    }

    pub fn column(id: impl Into<String>) -> Self {
        Self::stack(id, UiStackDirection::Column)
    }

    pub fn spacer(id: impl Into<String>, size: u16) -> Self {
        Self::new(id, UiNodeKind::Spacer { size })
    }

    pub fn gap(mut self, gap: u16) -> Self {
        if let UiNodeKind::Stack { gap: current, .. } = &mut self.kind {
            *current = gap;
        }
        self
    }

    pub fn child(mut self, child: UiNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = UiNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(UiNode::node_count).sum::<usize>()
    }

    pub fn contains_node_id(&self, id: &str) -> bool {
        self.id == id || self.children.iter().any(|child| child.contains_node_id(id))
    }
}

#[cfg(feature = "label")]
use crate::{
    command_protocol::CommandQueue,
    component_protocol::Component,
    event_protocol::{LifecycleEvent, LifecycleState, UiEvent},
    geometry::{ComponentId, LayoutInput, LayoutNode, LayoutOutput, LayoutProtocol, Rect},
    render_protocol::{Renderer, TextLayout, TextStyle},
};

#[cfg(feature = "label")]
pub struct Label {
    id: ComponentId,
    lifecycle: LifecycleState,
    text: String,
    style: TextStyle,
    bounds: Rect,
}

#[cfg(feature = "label")]
impl Label {
    pub fn new(id: ComponentId, text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            id,
            lifecycle: LifecycleState::new(),
            text: text.into(),
            style,
            bounds: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }

    pub const fn lifecycle_state(&self) -> LifecycleState {
        self.lifecycle
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.style = style;
    }
}

#[cfg(feature = "label")]
impl LayoutProtocol for Label {
    fn layout(&mut self, input: LayoutInput) -> LayoutOutput {
        self.bounds = input.bounds;
        LayoutOutput {
            bounds: self.bounds,
            children: vec![LayoutNode {
                component: self.id,
                bounds: self.bounds,
            }],
        }
    }
}

#[cfg(feature = "label")]
impl Component for Label {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn lifecycle(&mut self, event: LifecycleEvent) {
        self.lifecycle.apply(event);
    }

    fn update(&mut self, _event: &UiEvent, _commands: &mut CommandQueue) {}

    fn layout(&mut self, input: LayoutInput) -> LayoutOutput {
        LayoutProtocol::layout(self, input)
    }

    fn render(&self, renderer: &mut dyn Renderer, text: &dyn TextLayout) {
        for run in text.layout_runs(&self.text, &self.style, self.bounds) {
            renderer.draw_text(&run, &self.style);
        }
    }
}

#[cfg(all(test, feature = "label"))]
mod label_tests {
    use super::*;
    use crate::{
        render_protocol::{Color, HorizontalAlign, TextRun, TextWeight, TextWrap, VerticalAlign},
        ComponentPhase, Size,
    };

    #[derive(Default)]
    struct RecordingRenderer {
        text: Vec<String>,
    }

    impl Renderer for RecordingRenderer {
        fn fill_rect(&mut self, _rect: Rect, _color: Color) {}
        fn stroke_rect(&mut self, _rect: Rect, _color: Color, _width: i32) {}
        fn stroke_arc(
            &mut self,
            _rect: Rect,
            _color: Color,
            _width: i32,
            _start_degrees: i16,
            _sweep_degrees: i16,
        ) {
        }
        fn draw_text(&mut self, run: &TextRun, _style: &TextStyle) {
            self.text.push(run.text.clone());
        }
        fn push_clip(&mut self, _rect: Rect) {}
        fn pop_clip(&mut self) {}
    }

    struct SingleRunTextLayout;

    impl TextLayout for SingleRunTextLayout {
        fn measure(&self, text: &str, _style: &TextStyle, _max_width: i32) -> Size {
            Size {
                width: text.len() as i32,
                height: 1,
            }
        }

        fn layout_runs(&self, text: &str, _style: &TextStyle, bounds: Rect) -> Vec<TextRun> {
            vec![TextRun {
                text: text.to_string(),
                bounds,
            }]
        }
    }

    fn style() -> TextStyle {
        TextStyle {
            font_family: "Test".to_string(),
            size: 14.0,
            line_height: 20.0,
            semantic_role: Some(crate::TextRole::Body),
            weight: TextWeight::Regular,
            color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            horizontal_align: HorizontalAlign::Start,
            vertical_align: VerticalAlign::Center,
            wrap: TextWrap::NoWrap,
            ellipsis: true,
        }
    }

    #[test]
    fn label_uses_layout_and_renderer_protocols() {
        let mut label = Label::new(ComponentId(1), "hello", style());
        label.lifecycle(LifecycleEvent::Mount);
        label.lifecycle(LifecycleEvent::Resume);
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 30,
        };
        let output = Component::layout(&mut label, LayoutInput { bounds, scale: 1.0 });
        let mut renderer = RecordingRenderer::default();
        label.render(&mut renderer, &SingleRunTextLayout);

        assert_eq!(output.bounds, bounds);
        assert_eq!(output.children[0].component, ComponentId(1));
        assert_eq!(renderer.text, vec!["hello"]);
        assert_eq!(label.lifecycle_state().phase(), ComponentPhase::Active);
    }
}

#[cfg(all(test, feature = "label", feature = "button"))]
mod ui_node_tests {
    use super::*;

    #[test]
    fn ui_node_builders_create_serializable_component_tree() {
        let tree = UiNode::column("root")
            .gap(8)
            .child(UiNode::text("title", "Hello"))
            .child(UiNode::row("actions").child(UiNode::button(
                "save",
                "Save",
                Command::custom("demo.save"),
            )));

        let json = serde_json::to_string(&tree).expect("ui node tree should serialize");

        assert_eq!(tree.node_count(), 4);
        assert!(tree.contains_node_id("save"));
        assert!(json.contains("demo.save"));
    }
}
