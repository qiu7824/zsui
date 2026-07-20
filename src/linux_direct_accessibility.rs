use std::collections::HashMap;
use std::sync::{
    mpsc::{self, Receiver},
    Arc, RwLock,
};

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Affine, DeactivationHandler, Node,
    NodeId, Rect as AccessRect, Role, Toggled, Tree, TreeId, TreeUpdate,
};
use accesskit_winit::Adapter;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::linux_direct_menu::{
    LinuxMenuAccessibilityRole, LinuxMenuAccessibilitySnapshot, LinuxMenuAccessibilityTarget,
};
use crate::{
    NativeDrawCommand, NativeDrawPlan, Rect, ViewHitTarget, ViewHitTargetKind, ViewInteractionPlan,
};

const ROOT_NODE_ID: NodeId = NodeId(0);

#[derive(Debug, Clone)]
pub(crate) struct LinuxAccessibilityAction {
    pub request: ActionRequest,
    pub target: LinuxAccessibilityTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinuxAccessibilityTarget {
    View(ViewHitTarget),
    Menu(LinuxMenuAccessibilityTarget),
}

struct TreeActivationHandler {
    tree: Arc<RwLock<TreeUpdate>>,
}

impl ActivationHandler for TreeActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.tree.read().ok().map(|tree| tree.clone())
    }
}

struct ChannelActionHandler {
    sender: mpsc::Sender<ActionRequest>,
}

impl ActionHandler for ChannelActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        let _ = self.sender.send(request);
    }
}

struct NoopDeactivationHandler;

impl DeactivationHandler for NoopDeactivationHandler {
    fn deactivate_accessibility(&mut self) {}
}

pub(crate) struct LinuxDirectAccessibility {
    adapter: Adapter,
    action_receiver: Receiver<ActionRequest>,
    tree: Arc<RwLock<TreeUpdate>>,
    targets: HashMap<NodeId, LinuxAccessibilityTarget>,
    node_count: usize,
    action_count: usize,
}

impl LinuxDirectAccessibility {
    pub(crate) fn new(
        event_loop: &ActiveEventLoop,
        window: &Window,
        title: &str,
        logical_bounds: Rect,
        scale_factor: f64,
        content_offset_y: i32,
        menu: Option<LinuxMenuAccessibilitySnapshot>,
        plan: &NativeDrawPlan,
        interaction: Option<ViewInteractionPlan>,
        focused_widget: Option<crate::WidgetId>,
    ) -> Self {
        let (update, targets) = build_tree_update(
            title,
            logical_bounds,
            scale_factor,
            content_offset_y,
            menu,
            plan,
            interaction,
            focused_widget,
        );
        let node_count = update.nodes.len();
        let tree = Arc::new(RwLock::new(update));
        let (action_sender, action_receiver) = mpsc::channel();
        let adapter = Adapter::with_direct_handlers(
            event_loop,
            window,
            TreeActivationHandler {
                tree: Arc::clone(&tree),
            },
            ChannelActionHandler {
                sender: action_sender,
            },
            NoopDeactivationHandler,
        );
        Self {
            adapter,
            action_receiver,
            tree,
            targets,
            node_count,
            action_count: 0,
        }
    }

    pub(crate) fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub(crate) fn update(
        &mut self,
        title: &str,
        logical_bounds: Rect,
        scale_factor: f64,
        content_offset_y: i32,
        menu: Option<LinuxMenuAccessibilitySnapshot>,
        plan: &NativeDrawPlan,
        interaction: Option<ViewInteractionPlan>,
        focused_widget: Option<crate::WidgetId>,
    ) {
        let (update, targets) = build_tree_update(
            title,
            logical_bounds,
            scale_factor,
            content_offset_y,
            menu,
            plan,
            interaction,
            focused_widget,
        );
        self.node_count = update.nodes.len();
        self.targets = targets;
        if let Ok(mut tree) = self.tree.write() {
            *tree = update.clone();
        }
        self.adapter.update_if_active(|| update);
    }

    pub(crate) fn take_actions(&mut self) -> Vec<LinuxAccessibilityAction> {
        let mut actions = Vec::new();
        while let Ok(request) = self.action_receiver.try_recv() {
            self.action_count = self.action_count.saturating_add(1);
            if let Some(target) = self.targets.get(&request.target_node).copied() {
                actions.push(LinuxAccessibilityAction { request, target });
            }
        }
        actions
    }

    pub(crate) const fn node_count(&self) -> usize {
        self.node_count
    }

    pub(crate) const fn action_count(&self) -> usize {
        self.action_count
    }
}

fn build_tree_update(
    title: &str,
    logical_bounds: Rect,
    scale_factor: f64,
    content_offset_y: i32,
    menu: Option<LinuxMenuAccessibilitySnapshot>,
    plan: &NativeDrawPlan,
    interaction: Option<ViewInteractionPlan>,
    focused_widget: Option<crate::WidgetId>,
) -> (TreeUpdate, HashMap<NodeId, LinuxAccessibilityTarget>) {
    let targets = interaction
        .map(|interaction| interaction.hit_targets)
        .unwrap_or_default();
    let mut node_targets = HashMap::with_capacity(targets.len());
    let mut nodes = Vec::with_capacity(targets.len().saturating_add(1));
    let mut child_ids = Vec::with_capacity(targets.len());
    let mut focused_node = ROOT_NODE_ID;

    for (index, target) in targets.into_iter().enumerate() {
        let node_id = NodeId(index as u64 + 1);
        let mut node = Node::new(accesskit_role(target.kind));
        node.set_bounds(accesskit_rect(Rect {
            y: target.bounds.y.saturating_add(content_offset_y),
            ..target.bounds
        }));
        node.set_author_id(format!("zsui-widget-{}", target.widget.0));
        node.set_label(accessible_label(plan, target));
        if target.kind.accepts_text_input() {
            node.add_action(Action::SetValue);
            node.add_action(Action::ReplaceSelectedText);
        }
        if accesskit_role(target.kind) != Role::GenericContainer {
            node.add_action(Action::Focus);
            node.add_action(Action::Click);
        }
        if focused_widget == Some(target.widget) {
            focused_node = node_id;
        }
        child_ids.push(node_id);
        node_targets.insert(node_id, LinuxAccessibilityTarget::View(target));
        nodes.push((node_id, node));
    }

    if let Some(menu) = menu {
        let first_menu_node = nodes.len() as u64 + 1;
        let menu_bar_id = NodeId(first_menu_node);
        let root_ids = (0..menu.roots.len())
            .map(|index| NodeId(first_menu_node + 1 + index as u64))
            .collect::<Vec<_>>();
        let first_row_node = first_menu_node + 1 + root_ids.len() as u64;
        let row_ids = (0..menu.rows.len())
            .map(|index| NodeId(first_row_node + index as u64))
            .collect::<Vec<_>>();

        let mut menu_bar = Node::new(Role::MenuBar);
        menu_bar.set_author_id("zsui-menu-bar");
        menu_bar.set_label("应用菜单 / Application menu");
        menu_bar.set_bounds(accesskit_rect(menu.bar_bounds));
        menu_bar.set_children(root_ids.clone());
        child_ids.insert(0, menu_bar_id);
        nodes.push((menu_bar_id, menu_bar));

        for (root_index, (node_id, item)) in
            root_ids.iter().copied().zip(menu.roots.iter()).enumerate()
        {
            let mut node = menu_accessibility_node(item);
            if menu.open_root == Some(root_index) {
                node.set_children(row_ids.clone());
            }
            if item.focused {
                focused_node = node_id;
            }
            if let Some(target) = item.target {
                node_targets.insert(node_id, LinuxAccessibilityTarget::Menu(target));
            }
            nodes.push((node_id, node));
        }

        for (node_id, item) in row_ids.iter().copied().zip(menu.rows.iter()) {
            let node = menu_accessibility_node(item);
            if item.focused {
                focused_node = node_id;
            }
            if let Some(target) = item.target {
                node_targets.insert(node_id, LinuxAccessibilityTarget::Menu(target));
            }
            nodes.push((node_id, node));
        }
    }

    let mut root = Node::new(Role::Window);
    root.set_label(title);
    root.set_bounds(accesskit_rect(logical_bounds));
    root.set_transform(Affine::scale(scale_factor.max(0.1)));
    root.set_children(child_ids);
    nodes.insert(0, (ROOT_NODE_ID, root));

    (
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(ROOT_NODE_ID)),
            tree_id: TreeId::ROOT,
            focus: focused_node,
        },
        node_targets,
    )
}

fn menu_accessibility_node(item: &crate::linux_direct_menu::LinuxMenuAccessibilityItem) -> Node {
    let role = match item.role {
        LinuxMenuAccessibilityRole::Menu => Role::Menu,
        LinuxMenuAccessibilityRole::MenuItem => Role::MenuItem,
        LinuxMenuAccessibilityRole::CheckedMenuItem => Role::MenuItemCheckBox,
        LinuxMenuAccessibilityRole::Separator => Role::GenericContainer,
    };
    let mut node = Node::new(role);
    node.set_author_id(item.author_id.clone());
    if !item.label.is_empty() {
        node.set_label(item.label.clone());
    }
    node.set_bounds(accesskit_rect(item.bounds));
    if !item.enabled {
        node.set_disabled();
    }
    if let Some(expanded) = item.expanded {
        node.set_expanded(expanded);
    }
    if let Some(checked) = item.checked {
        node.set_toggled(Toggled::from(checked));
    }
    if item.target.is_some() && item.enabled {
        node.add_action(Action::Focus);
        node.add_action(Action::Click);
    }
    node
}

fn accesskit_rect(rect: Rect) -> AccessRect {
    AccessRect {
        x0: f64::from(rect.x),
        y0: f64::from(rect.y),
        x1: f64::from(rect.x.saturating_add(rect.width.max(0))),
        y1: f64::from(rect.y.saturating_add(rect.height.max(0))),
    }
}

fn accessible_label(plan: &NativeDrawPlan, target: ViewHitTarget) -> String {
    let mut best: Option<(i64, &str)> = None;
    for command in &plan.commands {
        let NativeDrawCommand::Text(text) = command else {
            continue;
        };
        let value = text.text.trim();
        if value.is_empty() {
            continue;
        }
        let overlap = overlap_area(target.bounds, text.bounds);
        if overlap <= 0 {
            continue;
        }
        if best.is_none_or(|(score, _)| overlap > score) {
            best = Some((overlap, value));
        }
    }
    best.map(|(_, value)| value.to_string())
        .unwrap_or_else(|| format!("{} {}", role_label(target.kind), target.widget.0))
}

fn overlap_area(left: Rect, right: Rect) -> i64 {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = left
        .x
        .saturating_add(left.width.max(0))
        .min(right.x.saturating_add(right.width.max(0)));
    let y1 = left
        .y
        .saturating_add(left.height.max(0))
        .min(right.y.saturating_add(right.height.max(0)));
    i64::from((x1 - x0).max(0)) * i64::from((y1 - y0).max(0))
}

fn role_label(kind: ViewHitTargetKind) -> &'static str {
    match accesskit_role(kind) {
        Role::Button => "Button",
        Role::CheckBox => "Checkbox",
        Role::RadioButton => "Radio button",
        Role::Switch => "Switch",
        Role::Slider => "Slider",
        Role::TextInput => "Text input",
        Role::MultilineTextInput => "Text area",
        Role::PasswordInput => "Password input",
        Role::ComboBox => "Combo box",
        Role::Tab => "Tab",
        Role::Tree => "Tree",
        Role::TreeItem => "Tree item",
        Role::Table | Role::Grid => "Table",
        Role::Row => "Row",
        Role::ColumnHeader => "Column header",
        Role::Dialog => "Dialog",
        Role::Canvas => "Canvas",
        Role::ScrollView => "Scroll view",
        _ => "Control",
    }
}

fn accesskit_role(kind: ViewHitTargetKind) -> Role {
    match kind {
        #[cfg(feature = "canvas")]
        ViewHitTargetKind::Canvas => Role::Canvas,
        ViewHitTargetKind::Button => Role::Button,
        #[cfg(feature = "label")]
        ViewHitTargetKind::NavigationViewToggle => Role::Button,
        #[cfg(feature = "label")]
        ViewHitTargetKind::NavigationViewScrim => Role::GenericContainer,
        #[cfg(feature = "toggle-button")]
        ViewHitTargetKind::ToggleButton => Role::Button,
        ViewHitTargetKind::Textbox => Role::TextInput,
        ViewHitTargetKind::TextEditor => Role::MultilineTextInput,
        #[cfg(feature = "password-box")]
        ViewHitTargetKind::PasswordBox => Role::PasswordInput,
        #[cfg(feature = "password-box")]
        ViewHitTargetKind::PasswordBoxReveal => Role::Button,
        ViewHitTargetKind::Checkbox => Role::CheckBox,
        ViewHitTargetKind::Toggle => Role::Switch,
        #[cfg(feature = "slider")]
        ViewHitTargetKind::Slider => Role::Slider,
        #[cfg(feature = "number-box")]
        ViewHitTargetKind::NumberBox => Role::NumberInput,
        #[cfg(feature = "number-box")]
        ViewHitTargetKind::NumberBoxDecrement | ViewHitTargetKind::NumberBoxIncrement => {
            Role::Button
        }
        #[cfg(feature = "radio")]
        ViewHitTargetKind::RadioButton => Role::RadioButton,
        #[cfg(feature = "auto-suggest")]
        ViewHitTargetKind::AutoSuggestBox => Role::EditableComboBox,
        #[cfg(feature = "auto-suggest")]
        ViewHitTargetKind::AutoSuggestSearch | ViewHitTargetKind::AutoSuggestClear => Role::Button,
        #[cfg(feature = "auto-suggest")]
        ViewHitTargetKind::AutoSuggestSuggestion { .. } => Role::ListBoxOption,
        #[cfg(feature = "tree")]
        ViewHitTargetKind::TreeView => Role::Tree,
        #[cfg(feature = "tree")]
        ViewHitTargetKind::TreeNode { .. } => Role::TreeItem,
        #[cfg(feature = "tree")]
        ViewHitTargetKind::TreeNodeExpander { .. } => Role::DisclosureTriangle,
        #[cfg(feature = "grid-view")]
        ViewHitTargetKind::GridView => Role::Grid,
        #[cfg(feature = "grid-view")]
        ViewHitTargetKind::GridViewItem { .. } => Role::GridCell,
        #[cfg(feature = "table")]
        ViewHitTargetKind::DataGrid => Role::Table,
        #[cfg(feature = "table")]
        ViewHitTargetKind::TableHeader { .. } => Role::ColumnHeader,
        #[cfg(feature = "table")]
        ViewHitTargetKind::TableRow { .. } => Role::Row,
        #[cfg(feature = "dialog")]
        ViewHitTargetKind::ContentDialog => Role::Dialog,
        #[cfg(feature = "dialog")]
        ViewHitTargetKind::ContentDialogScrim => Role::GenericContainer,
        #[cfg(feature = "dialog")]
        ViewHitTargetKind::ContentDialogButton { .. } => Role::Button,
        #[cfg(feature = "flyout")]
        ViewHitTargetKind::Flyout => Role::Dialog,
        #[cfg(feature = "flyout")]
        ViewHitTargetKind::FlyoutScrim => Role::GenericContainer,
        #[cfg(feature = "command-palette")]
        ViewHitTargetKind::CommandPalette => Role::SearchInput,
        #[cfg(feature = "command-palette")]
        ViewHitTargetKind::CommandPaletteScrim => Role::GenericContainer,
        #[cfg(feature = "command-palette")]
        ViewHitTargetKind::CommandPaletteClear => Role::Button,
        #[cfg(feature = "command-palette")]
        ViewHitTargetKind::CommandPaletteItem { .. } => Role::ListBoxOption,
        #[cfg(feature = "toast")]
        ViewHitTargetKind::Toast => Role::Status,
        #[cfg(feature = "toast")]
        ViewHitTargetKind::ToastAction | ViewHitTargetKind::ToastClose => Role::Button,
        #[cfg(feature = "teaching-tip")]
        ViewHitTargetKind::TeachingTip => Role::Tooltip,
        #[cfg(feature = "teaching-tip")]
        ViewHitTargetKind::TeachingTipAction | ViewHitTargetKind::TeachingTipClose => Role::Button,
        #[cfg(feature = "info-bar")]
        ViewHitTargetKind::InfoBar => Role::Alert,
        #[cfg(feature = "info-bar")]
        ViewHitTargetKind::InfoBarAction | ViewHitTargetKind::InfoBarClose => Role::Button,
        #[cfg(feature = "breadcrumb")]
        ViewHitTargetKind::BreadcrumbBar => Role::Navigation,
        #[cfg(feature = "breadcrumb")]
        ViewHitTargetKind::BreadcrumbOverflow => Role::Button,
        #[cfg(feature = "breadcrumb")]
        ViewHitTargetKind::BreadcrumbItem { .. }
        | ViewHitTargetKind::BreadcrumbOverflowItem { .. } => Role::Link,
        #[cfg(feature = "combo")]
        ViewHitTargetKind::ComboBox => Role::ComboBox,
        #[cfg(feature = "combo")]
        ViewHitTargetKind::ComboBoxOption { .. } => Role::ListBoxOption,
        #[cfg(feature = "date-picker")]
        ViewHitTargetKind::DatePicker => Role::DateInput,
        #[cfg(feature = "date-picker")]
        ViewHitTargetKind::DatePickerDay { .. } => Role::Button,
        #[cfg(feature = "date-picker")]
        ViewHitTargetKind::DatePickerPreviousMonth | ViewHitTargetKind::DatePickerNextMonth => {
            Role::Button
        }
        #[cfg(feature = "time-picker")]
        ViewHitTargetKind::TimePicker => Role::TimeInput,
        #[cfg(feature = "time-picker")]
        ViewHitTargetKind::TimePickerChoice { .. } => Role::ListBoxOption,
        #[cfg(feature = "color-picker")]
        ViewHitTargetKind::ColorPicker => Role::ColorWell,
        #[cfg(feature = "color-picker")]
        ViewHitTargetKind::ColorPickerPopup => Role::Dialog,
        #[cfg(feature = "color-picker")]
        ViewHitTargetKind::ColorPickerSpectrum
        | ViewHitTargetKind::ColorPickerHue
        | ViewHitTargetKind::ColorPickerChannel { .. } => Role::Slider,
        #[cfg(feature = "tabs")]
        ViewHitTargetKind::Tab { .. } => Role::Tab,
        #[cfg(feature = "scroll")]
        ViewHitTargetKind::Scroll => Role::ScrollView,
        ViewHitTargetKind::Unknown => Role::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NativeDrawTextCommand, SemanticTextStyle, ZsuiThemeMode};

    #[test]
    fn visible_text_names_the_accessible_control() {
        let target = ViewHitTarget::with_kind(
            crate::WidgetId(7),
            Rect {
                x: 10,
                y: 20,
                width: 120,
                height: 32,
            },
            ViewHitTargetKind::Button,
        );
        let plan = NativeDrawPlan::new([NativeDrawCommand::Text(NativeDrawTextCommand {
            text: "保存 / Save".to_string(),
            bounds: target.bounds,
            style: SemanticTextStyle::body(),
        })])
        .theme_mode(ZsuiThemeMode::Light);
        assert_eq!(accessible_label(&plan, target), "保存 / Save");
        assert_eq!(accesskit_role(target.kind), Role::Button);
    }

    #[cfg(all(feature = "canvas", feature = "flyout"))]
    #[test]
    fn overlay_and_canvas_targets_keep_platform_semantics() {
        assert_eq!(accesskit_role(ViewHitTargetKind::Canvas), Role::Canvas);
        assert_eq!(accesskit_role(ViewHitTargetKind::Flyout), Role::Dialog);
        assert_eq!(
            accesskit_role(ViewHitTargetKind::FlyoutScrim),
            Role::GenericContainer
        );
    }
}
