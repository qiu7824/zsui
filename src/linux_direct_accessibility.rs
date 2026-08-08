use std::collections::HashMap;
use std::sync::{
    mpsc::{self, Receiver},
    Arc, RwLock,
};

#[cfg(feature = "menu-flyout")]
use accesskit::HasPopup;
use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Affine, DeactivationHandler, Live,
    Node, NodeId, Rect as AccessRect, Role, Toggled, Tree, TreeId, TreeUpdate,
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

#[cfg(feature = "tabs")]
type LinuxTabAccessibilitySnapshots =
    Vec<crate::native_tab_accessibility::NativeTabAccessibilitySnapshot>;
#[cfg(not(feature = "tabs"))]
type LinuxTabAccessibilitySnapshots = ();

#[cfg(feature = "menu-flyout")]
#[derive(Default)]
struct MenuFlyoutAccessibilityHierarchy {
    root_by_widget: HashMap<crate::WidgetId, NodeId>,
    item_by_path: HashMap<(crate::WidgetId, crate::ZsMenuFlyoutPath), NodeId>,
    parent_by_child: HashMap<NodeId, NodeId>,
    children_by_parent: HashMap<NodeId, Vec<NodeId>>,
}

#[cfg(feature = "menu-flyout")]
fn menu_flyout_accessibility_hierarchy(
    targets: &[(NodeId, ViewHitTarget)],
) -> MenuFlyoutAccessibilityHierarchy {
    let mut hierarchy = MenuFlyoutAccessibilityHierarchy::default();
    for (node_id, target) in targets {
        match target.kind {
            ViewHitTargetKind::MenuFlyout => {
                hierarchy
                    .root_by_widget
                    .entry(target.widget)
                    .or_insert(*node_id);
            }
            ViewHitTargetKind::MenuFlyoutItem { path, .. } => {
                hierarchy
                    .item_by_path
                    .insert((target.widget, path), *node_id);
            }
            _ => {}
        }
    }

    for (node_id, target) in targets {
        let ViewHitTargetKind::MenuFlyoutItem { path, .. } = target.kind else {
            continue;
        };
        let parent = match path.parent() {
            Some(parent) => hierarchy
                .item_by_path
                .get(&(target.widget, parent))
                .copied(),
            None => hierarchy.root_by_widget.get(&target.widget).copied(),
        };
        if let Some(parent) = parent {
            hierarchy.parent_by_child.insert(*node_id, parent);
            hierarchy
                .children_by_parent
                .entry(parent)
                .or_default()
                .push(*node_id);
        }
    }
    hierarchy
}

#[cfg(feature = "menu-flyout")]
fn menu_flyout_accessibility_author_id(
    widget: crate::WidgetId,
    path: crate::ZsMenuFlyoutPath,
) -> String {
    let mut indices = Vec::with_capacity(path.level().saturating_add(1));
    let mut current = Some(path);
    while let Some(path) = current {
        indices.push(path.item().to_string());
        current = path.parent();
    }
    indices.reverse();
    format!("zsui-menu-flyout-{}-{}", widget.0, indices.join("-"))
}

#[derive(Debug, Clone)]
pub(crate) struct LinuxAccessibilityAction {
    pub request: ActionRequest,
    pub target: LinuxAccessibilityTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinuxAccessibilityTarget {
    View(ViewHitTarget),
    Semantic(crate::WidgetId),
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
        tab_snapshots: LinuxTabAccessibilitySnapshots,
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
            tab_snapshots,
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
        tab_snapshots: LinuxTabAccessibilitySnapshots,
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
            tab_snapshots,
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
    tab_snapshots: LinuxTabAccessibilitySnapshots,
) -> (TreeUpdate, HashMap<NodeId, LinuxAccessibilityTarget>) {
    #[cfg(not(feature = "tabs"))]
    let _ = &tab_snapshots;
    let (targets, semantic_nodes) = interaction
        .map(|interaction| (interaction.hit_targets, interaction.accessibility_nodes))
        .unwrap_or_default();
    let targets = targets
        .into_iter()
        .enumerate()
        .map(|(index, target)| (NodeId(index as u64 + 1), target))
        .collect::<Vec<_>>();
    let mut next_synthetic_node_id = targets.len() as u64 + 1;
    #[cfg(feature = "tabs")]
    let tab_node_ids = targets
        .iter()
        .filter_map(|(node_id, target)| {
            matches!(target.kind, ViewHitTargetKind::Tab { .. })
                .then_some((target.widget, *node_id))
        })
        .collect::<HashMap<_, _>>();
    #[cfg(feature = "menu-flyout")]
    let menu_flyout_hierarchy = menu_flyout_accessibility_hierarchy(&targets);
    let mut node_targets = HashMap::with_capacity(targets.len());
    let mut nodes = Vec::with_capacity(targets.len().saturating_add(1));
    let mut child_ids = Vec::with_capacity(targets.len());
    let mut focused_node = ROOT_NODE_ID;
    let mut widget_node_ids = HashMap::new();
    let mut target_bounds = HashMap::new();

    for (node_id, target) in targets {
        #[cfg(feature = "scroll")]
        if target.kind == ViewHitTargetKind::ScrollbarThumb {
            continue;
        }
        #[cfg(feature = "virtual-list")]
        if target.kind == ViewHitTargetKind::ItemsRepeaterScrollbarThumb {
            continue;
        }
        #[cfg(feature = "menu-flyout")]
        match target.kind {
            ViewHitTargetKind::MenuFlyoutScrim => continue,
            ViewHitTargetKind::MenuFlyout
                if menu_flyout_hierarchy.root_by_widget.get(&target.widget) != Some(&node_id) =>
            {
                continue;
            }
            _ => {}
        }
        let mut node = Node::new(accesskit_role(target.kind));
        node.set_bounds(accesskit_rect(Rect {
            y: target.bounds.y.saturating_add(content_offset_y),
            ..target.bounds
        }));
        #[cfg(feature = "menu-flyout")]
        match target.kind {
            ViewHitTargetKind::MenuFlyout => {
                node.set_author_id(format!("zsui-menu-flyout-{}", target.widget.0));
            }
            ViewHitTargetKind::MenuFlyoutItem { path, .. } => {
                node.set_author_id(menu_flyout_accessibility_author_id(target.widget, path));
            }
            _ => node.set_author_id(format!("zsui-widget-{}", target.widget.0)),
        }
        #[cfg(not(feature = "menu-flyout"))]
        node.set_author_id(format!("zsui-widget-{}", target.widget.0));
        #[cfg(feature = "menu-flyout")]
        let is_menu_flyout_surface = target.kind == ViewHitTargetKind::MenuFlyout;
        #[cfg(not(feature = "menu-flyout"))]
        let is_menu_flyout_surface = false;
        if !is_menu_flyout_surface {
            node.set_label(accessible_label(plan, target));
        }
        apply_view_accessibility_state(&mut node, target.kind);
        #[cfg(feature = "tabs")]
        if let ViewHitTargetKind::Tab { tab_view, tab, .. } = target.kind {
            if let Some(item) = tab_snapshots
                .iter()
                .find(|snapshot| snapshot.tab_view == tab_view)
                .and_then(|snapshot| snapshot.items.iter().find(|item| item.tab() == tab))
            {
                node.set_selected(item.selected);
                node.set_position_in_set(item.position);
                node.set_size_of_set(item.count);
            }
        }
        #[cfg(feature = "menu-flyout")]
        if let Some(children) = menu_flyout_hierarchy.children_by_parent.get(&node_id) {
            node.set_children(children.clone());
        }
        if target.kind.accepts_text_input() {
            node.add_action(Action::SetValue);
            node.add_action(Action::ReplaceSelectedText);
        }
        if accesskit_role(target.kind) != Role::GenericContainer && !is_menu_flyout_surface {
            node.add_action(Action::Focus);
            node.add_action(Action::Click);
        }
        #[cfg(feature = "menu-flyout")]
        let menu_item_highlighted = matches!(
            target.kind,
            ViewHitTargetKind::MenuFlyoutItem {
                highlighted: true,
                ..
            }
        );
        #[cfg(not(feature = "menu-flyout"))]
        let menu_item_highlighted = false;
        #[cfg(feature = "menu-flyout")]
        let is_menu_item = matches!(target.kind, ViewHitTargetKind::MenuFlyoutItem { .. });
        #[cfg(not(feature = "menu-flyout"))]
        let is_menu_item = false;
        if menu_item_highlighted || (focused_widget == Some(target.widget) && !is_menu_item) {
            focused_node = node_id;
        }
        #[cfg(feature = "menu-flyout")]
        let is_nested_menu_flyout_item =
            menu_flyout_hierarchy.parent_by_child.contains_key(&node_id);
        #[cfg(not(feature = "menu-flyout"))]
        let is_nested_menu_flyout_item = false;
        if !is_nested_menu_flyout_item {
            child_ids.push(node_id);
        }
        widget_node_ids.entry(target.widget).or_insert(node_id);
        target_bounds.insert(node_id, target.bounds);
        node_targets.insert(node_id, LinuxAccessibilityTarget::View(target));
        nodes.push((node_id, node));
    }

    #[cfg(feature = "tabs")]
    for snapshot in tab_snapshots {
        if semantic_nodes.iter().any(|semantic| {
            semantic.widget == snapshot.tab_view
                && semantic.role == crate::ZsAccessibilityRole::TabList
        }) {
            continue;
        }
        let list_id = NodeId(next_synthetic_node_id);
        next_synthetic_node_id = next_synthetic_node_id.saturating_add(1);
        let panel_id = NodeId(next_synthetic_node_id);
        next_synthetic_node_id = next_synthetic_node_id.saturating_add(1);
        let item_ids = snapshot
            .items
            .iter()
            .filter_map(|item| tab_node_ids.get(&item.target.widget).copied())
            .collect::<Vec<_>>();
        if item_ids.is_empty() {
            continue;
        }
        for item_id in item_ids.iter().copied() {
            if let Some((_, node)) = nodes.iter_mut().find(|(node_id, _)| *node_id == item_id) {
                node.set_controls(vec![panel_id]);
            }
            child_ids.retain(|candidate| *candidate != item_id);
        }

        let mut list = Node::new(Role::TabList);
        list.set_author_id(format!("zsui-tab-list-{}", snapshot.tab_view.0));
        list.set_label("Tabs");
        list.set_bounds(accesskit_rect(Rect {
            y: snapshot.list_bounds.y.saturating_add(content_offset_y),
            ..snapshot.list_bounds
        }));
        list.set_children(item_ids);

        let mut panel = Node::new(Role::TabPanel);
        panel.set_author_id(format!("zsui-tab-panel-{}", snapshot.tab_view.0));
        panel.set_bounds(accesskit_rect(Rect {
            y: snapshot.panel_bounds.y.saturating_add(content_offset_y),
            ..snapshot.panel_bounds
        }));
        if let Some(selected) = snapshot.selected_item() {
            if let Some(selected_id) = tab_node_ids.get(&selected.target.widget).copied() {
                panel.set_labelled_by(vec![selected_id]);
                panel.set_label(selected.label.clone());
            }
        }
        child_ids.push(list_id);
        child_ids.push(panel_id);
        nodes.push((list_id, list));
        nodes.push((panel_id, panel));
    }

    let mut semantic_entries = Vec::with_capacity(semantic_nodes.len());
    for semantic in semantic_nodes {
        let node_id = if let Some(node_id) = widget_node_ids.get(&semantic.widget).copied() {
            if let Some((_, node)) = nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == node_id)
            {
                apply_semantic_accessibility_node(node, &semantic, content_offset_y);
            }
            node_id
        } else {
            let node_id = NodeId(next_synthetic_node_id);
            next_synthetic_node_id = next_synthetic_node_id.saturating_add(1);
            let mut node = Node::new(accesskit_semantic_role(semantic.role));
            node.set_author_id(format!("zsui-semantic-{}", semantic.widget.0));
            apply_semantic_accessibility_node(&mut node, &semantic, content_offset_y);
            nodes.push((node_id, node));
            widget_node_ids.insert(semantic.widget, node_id);
            node_id
        };
        if semantic.action_target.is_some() && semantic.enabled {
            if let Some((_, node)) = nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == node_id)
            {
                node.add_action(Action::Focus);
                node.add_action(Action::Click);
            }
            node_targets.insert(node_id, LinuxAccessibilityTarget::Semantic(semantic.widget));
        }
        if focused_widget == Some(semantic.widget) {
            focused_node = node_id;
        }
        semantic_entries.push((node_id, semantic));
    }

    for (node_id, semantic) in &semantic_entries {
        if let Some(parent_id) = semantic
            .parent
            .and_then(|parent| widget_node_ids.get(&parent).copied())
        {
            child_ids.retain(|candidate| candidate != node_id);
            push_accessibility_child(&mut nodes, parent_id, *node_id);
        } else if !child_ids.contains(node_id) {
            child_ids.push(*node_id);
        }
    }

    let mut semantic_parents = semantic_entries
        .iter()
        .filter(|(_, semantic)| semantic_role_can_contain_children(semantic.role))
        .collect::<Vec<_>>();
    semantic_parents.sort_by_key(|(_, semantic)| {
        i64::from(semantic.bounds.width.max(0)) * i64::from(semantic.bounds.height.max(0))
    });
    for (parent_id, semantic) in semantic_parents {
        let adopt = child_ids
            .iter()
            .copied()
            .filter(|candidate| candidate != parent_id)
            .filter(|candidate| {
                target_bounds
                    .get(candidate)
                    .is_some_and(|bounds| rect_contains(semantic.bounds, *bounds))
            })
            .collect::<Vec<_>>();
        for child in adopt {
            child_ids.retain(|candidate| *candidate != child);
            push_accessibility_child(&mut nodes, *parent_id, child);
        }
    }

    if let Some(menu) = menu {
        let first_menu_node = next_synthetic_node_id;
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

fn accesskit_semantic_role(role: crate::ZsAccessibilityRole) -> Role {
    match role {
        crate::ZsAccessibilityRole::Application => Role::Application,
        crate::ZsAccessibilityRole::Article => Role::Article,
        crate::ZsAccessibilityRole::Button => Role::Button,
        crate::ZsAccessibilityRole::Canvas => Role::Canvas,
        crate::ZsAccessibilityRole::ColorWell => Role::ColorWell,
        crate::ZsAccessibilityRole::ComboBox => Role::ComboBox,
        crate::ZsAccessibilityRole::Complementary => Role::Complementary,
        crate::ZsAccessibilityRole::DatePicker => Role::DateInput,
        crate::ZsAccessibilityRole::Dialog => Role::Dialog,
        crate::ZsAccessibilityRole::Form => Role::Form,
        crate::ZsAccessibilityRole::Grid => Role::Grid,
        crate::ZsAccessibilityRole::Group => Role::Group,
        crate::ZsAccessibilityRole::Heading => Role::Heading,
        crate::ZsAccessibilityRole::Image => Role::Image,
        crate::ZsAccessibilityRole::List => Role::List,
        crate::ZsAccessibilityRole::ListItem => Role::ListItem,
        crate::ZsAccessibilityRole::Log => Role::Log,
        crate::ZsAccessibilityRole::Main => Role::Main,
        crate::ZsAccessibilityRole::Navigation => Role::Navigation,
        crate::ZsAccessibilityRole::ProgressBar => Role::ProgressIndicator,
        crate::ZsAccessibilityRole::Region => Role::Region,
        crate::ZsAccessibilityRole::Slider => Role::Slider,
        crate::ZsAccessibilityRole::SpinButton => Role::SpinButton,
        crate::ZsAccessibilityRole::Status => Role::Status,
        crate::ZsAccessibilityRole::Tab => Role::Tab,
        crate::ZsAccessibilityRole::TabList => Role::TabList,
        crate::ZsAccessibilityRole::TabPanel => Role::TabPanel,
        crate::ZsAccessibilityRole::Text => Role::Label,
        crate::ZsAccessibilityRole::TextBox => Role::MultilineTextInput,
        crate::ZsAccessibilityRole::TimePicker => Role::TimeInput,
        crate::ZsAccessibilityRole::Tree => Role::Tree,
    }
}

fn apply_semantic_accessibility_node(
    node: &mut Node,
    semantic: &crate::ZsAccessibilityNode,
    content_offset_y: i32,
) {
    node.set_role(accesskit_semantic_role(semantic.role));
    node.set_author_id(format!("zsui-semantic-{}", semantic.widget.0));
    node.set_bounds(accesskit_rect(Rect {
        y: semantic.bounds.y.saturating_add(content_offset_y),
        ..semantic.bounds
    }));
    if let Some(label) = &semantic.label {
        node.set_label(label.clone());
    }
    if let Some(description) = &semantic.description {
        node.set_description(description.clone());
    }
    if let Some(live_region) = semantic.live_region {
        node.set_live(match live_region {
            crate::ZsAccessibilityLiveRegion::Polite => Live::Polite,
            crate::ZsAccessibilityLiveRegion::Assertive => Live::Assertive,
        });
    }
    if !semantic.enabled {
        node.set_disabled();
    }
    if let Some(selected) = semantic.selected {
        node.set_selected(selected);
    }
    if let Some(checked) = semantic.checked {
        node.set_toggled(Toggled::from(checked));
    }
    if let Some(range) = semantic.range_value {
        node.set_numeric_value(range.value);
        node.set_min_numeric_value(range.minimum);
        node.set_max_numeric_value(range.maximum);
        if semantic.enabled && !range.interaction.is_read_only() {
            node.add_action(Action::SetValue);
            node.add_action(Action::Increment);
            node.add_action(Action::Decrement);
        }
    }
}

fn push_accessibility_child(nodes: &mut [(NodeId, Node)], parent: NodeId, child: NodeId) {
    if let Some((_, node)) = nodes.iter_mut().find(|(candidate, _)| *candidate == parent) {
        let mut children = node.children().to_vec();
        if !children.contains(&child) {
            children.push(child);
            node.set_children(children);
        }
    }
}

fn semantic_role_can_contain_children(role: crate::ZsAccessibilityRole) -> bool {
    matches!(
        role,
        crate::ZsAccessibilityRole::Application
            | crate::ZsAccessibilityRole::Article
            | crate::ZsAccessibilityRole::Canvas
            | crate::ZsAccessibilityRole::Complementary
            | crate::ZsAccessibilityRole::Form
            | crate::ZsAccessibilityRole::Group
            | crate::ZsAccessibilityRole::List
            | crate::ZsAccessibilityRole::ListItem
            | crate::ZsAccessibilityRole::Log
            | crate::ZsAccessibilityRole::Main
            | crate::ZsAccessibilityRole::Navigation
            | crate::ZsAccessibilityRole::Region
            | crate::ZsAccessibilityRole::TabList
            | crate::ZsAccessibilityRole::TabPanel
    )
}

fn rect_contains(parent: Rect, child: Rect) -> bool {
    let parent_right = parent.x.saturating_add(parent.width.max(0));
    let parent_bottom = parent.y.saturating_add(parent.height.max(0));
    let child_right = child.x.saturating_add(child.width.max(0));
    let child_bottom = child.y.saturating_add(child.height.max(0));
    child.x >= parent.x
        && child.y >= parent.y
        && child_right <= parent_right
        && child_bottom <= parent_bottom
}

#[cfg(feature = "menu-flyout")]
fn apply_view_accessibility_state(node: &mut Node, kind: ViewHitTargetKind) {
    #[cfg(feature = "menu-flyout")]
    if let ViewHitTargetKind::MenuFlyoutItem {
        row_kind,
        expanded,
        highlighted,
        ..
    } = kind
    {
        match row_kind {
            crate::ZsMenuFlyoutRowKind::Command { checked: true } => {
                node.set_toggled(Toggled::True);
            }
            crate::ZsMenuFlyoutRowKind::Submenu => {
                node.set_expanded(expanded);
                node.set_has_popup(HasPopup::Menu);
            }
            crate::ZsMenuFlyoutRowKind::Command { checked: false }
            | crate::ZsMenuFlyoutRowKind::Separator => {}
        }
        node.set_selected(highlighted);
    }
}

#[cfg(not(feature = "menu-flyout"))]
fn apply_view_accessibility_state(_node: &mut Node, _kind: ViewHitTargetKind) {}

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
        Role::Menu => "Menu",
        Role::MenuItem | Role::MenuItemCheckBox => "Menu item",
        Role::Canvas => "Canvas",
        Role::ScrollBar => "Scroll bar",
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
        #[cfg(feature = "split-view")]
        ViewHitTargetKind::SplitViewScrim => Role::GenericContainer,
        #[cfg(feature = "virtual-list")]
        ViewHitTargetKind::ItemsRepeaterScrollbarTrack => Role::ScrollBar,
        #[cfg(feature = "virtual-list")]
        ViewHitTargetKind::ItemsRepeaterScrollbarThumb => Role::GenericContainer,
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
        #[cfg(feature = "menu-flyout")]
        ViewHitTargetKind::MenuFlyout => Role::Menu,
        #[cfg(feature = "menu-flyout")]
        ViewHitTargetKind::MenuFlyoutScrim => Role::GenericContainer,
        #[cfg(feature = "menu-flyout")]
        ViewHitTargetKind::MenuFlyoutItem { row_kind, .. } => match row_kind {
            crate::ZsMenuFlyoutRowKind::Command { checked: true } => Role::MenuItemCheckBox,
            crate::ZsMenuFlyoutRowKind::Submenu => Role::Menu,
            crate::ZsMenuFlyoutRowKind::Command { checked: false }
            | crate::ZsMenuFlyoutRowKind::Separator => Role::MenuItem,
        },
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
        #[cfg(feature = "scroll")]
        ViewHitTargetKind::ScrollbarTrack => Role::ScrollBar,
        #[cfg(feature = "scroll")]
        ViewHitTargetKind::ScrollbarThumb => Role::GenericContainer,
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

    #[cfg(feature = "split-view")]
    #[test]
    fn split_view_scrim_is_an_accessible_generic_container() {
        assert_eq!(
            accesskit_role(ViewHitTargetKind::SplitViewScrim),
            Role::GenericContainer
        );
    }

    #[cfg(feature = "virtual-list")]
    #[test]
    fn items_repeater_scrollbar_targets_expose_scrollbar_semantics() {
        assert_eq!(
            accesskit_role(ViewHitTargetKind::ItemsRepeaterScrollbarTrack),
            Role::ScrollBar
        );
        assert_eq!(
            accesskit_role(ViewHitTargetKind::ItemsRepeaterScrollbarThumb),
            Role::GenericContainer
        );
    }

    #[cfg(feature = "scroll")]
    #[test]
    fn scroll_targets_expose_one_scrollbar_semantic_surface() {
        assert_eq!(
            accesskit_role(ViewHitTargetKind::ScrollbarTrack),
            Role::ScrollBar
        );
        assert_eq!(
            accesskit_role(ViewHitTargetKind::ScrollbarThumb),
            Role::GenericContainer
        );
    }

    #[test]
    fn semantic_groups_adopt_controls_and_preserve_explicit_children() {
        let group_widget = crate::WidgetId(10);
        let image_widget = crate::WidgetId(11);
        let button_widget = crate::WidgetId(12);
        let group_bounds = Rect {
            x: 10,
            y: 10,
            width: 240,
            height: 180,
        };
        let interaction = ViewInteractionPlan {
            hit_targets: vec![ViewHitTarget::with_kind(
                button_widget,
                Rect {
                    x: 30,
                    y: 120,
                    width: 100,
                    height: 32,
                },
                ViewHitTargetKind::Button,
            )],
            accessibility_nodes: vec![
                crate::ZsAccessibilityNode {
                    widget: group_widget,
                    parent: None,
                    bounds: group_bounds,
                    role: crate::ZsAccessibilityRole::Group,
                    label: Some("Appearance".to_owned()),
                    description: None,
                    live_region: None,
                    enabled: true,
                    selected: None,
                    checked: None,
                    range_value: None,
                    action_target: None,
                },
                crate::ZsAccessibilityNode {
                    widget: image_widget,
                    parent: Some(group_widget),
                    bounds: Rect {
                        x: 30,
                        y: 35,
                        width: 64,
                        height: 64,
                    },
                    role: crate::ZsAccessibilityRole::Image,
                    label: Some("Theme preview".to_owned()),
                    description: None,
                    live_region: None,
                    enabled: true,
                    selected: None,
                    checked: None,
                    range_value: None,
                    action_target: None,
                },
            ],
            #[cfg(feature = "tooltip")]
            tooltip_targets: Vec::new(),
        };
        #[cfg(feature = "tabs")]
        let tabs = Vec::new();
        #[cfg(not(feature = "tabs"))]
        let tabs = ();
        let (tree, _) = build_tree_update(
            "Settings",
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 240,
            },
            1.0,
            0,
            None,
            &NativeDrawPlan::new([]),
            Some(interaction),
            None,
            tabs,
        );
        let (_, root) = &tree.nodes[0];
        assert_eq!(root.children().len(), 1);
        let (group_id, group) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Group)
            .expect("semantic group");
        assert_eq!(group.label(), Some("Appearance"));
        assert_eq!(root.children(), &[*group_id]);
        let child_roles = group
            .children()
            .iter()
            .filter_map(|child| {
                tree.nodes
                    .iter()
                    .find(|(node_id, _)| node_id == child)
                    .map(|(_, node)| node.role())
            })
            .collect::<Vec<_>>();
        assert!(child_roles.contains(&Role::Image));
        assert!(child_roles.contains(&Role::Button));
    }

    #[test]
    fn progress_semantics_project_numeric_range_into_accesskit() {
        let mut node = Node::new(Role::ProgressIndicator);
        apply_semantic_accessibility_node(
            &mut node,
            &crate::ZsAccessibilityNode {
                widget: crate::WidgetId(22),
                parent: None,
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 8,
                },
                role: crate::ZsAccessibilityRole::ProgressBar,
                label: Some("Download".to_owned()),
                description: None,
                live_region: None,
                enabled: true,
                selected: None,
                checked: None,
                range_value: Some(crate::ZsAccessibilityRangeValue::new(42.0, 0.0, 100.0)),
                action_target: None,
            },
            0,
        );

        assert_eq!(node.role(), Role::ProgressIndicator);
        assert_eq!(node.numeric_value(), Some(42.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert!(!node.supports_action(Action::SetValue));
    }

    #[test]
    fn adjustable_range_semantics_enable_accesskit_numeric_actions() {
        let mut node = Node::new(Role::Slider);
        apply_semantic_accessibility_node(
            &mut node,
            &crate::ZsAccessibilityNode {
                widget: crate::WidgetId(23),
                parent: None,
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 32,
                },
                role: crate::ZsAccessibilityRole::Slider,
                label: Some("Volume".to_owned()),
                description: None,
                live_region: None,
                enabled: true,
                selected: None,
                checked: None,
                range_value: Some(
                    crate::ZsAccessibilityRangeValue::new(25.0, 0.0, 100.0).adjustable(5.0, 50.0),
                ),
                action_target: None,
            },
            0,
        );

        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.numeric_value(), Some(25.0));
        assert!(node.supports_action(Action::SetValue));
        assert!(node.supports_action(Action::Increment));
        assert!(node.supports_action(Action::Decrement));
    }

    #[test]
    fn checked_button_semantics_project_accesskit_toggled_state() {
        let mut node = Node::new(Role::Button);
        apply_semantic_accessibility_node(
            &mut node,
            &crate::ZsAccessibilityNode {
                widget: crate::WidgetId(24),
                parent: None,
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 32,
                },
                role: crate::ZsAccessibilityRole::Button,
                label: Some("Pin panel".to_owned()),
                description: None,
                live_region: None,
                enabled: true,
                selected: None,
                checked: Some(true),
                range_value: None,
                action_target: None,
            },
            0,
        );

        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    #[cfg(feature = "dialog")]
    #[test]
    fn content_dialog_semantic_buttons_are_focusable_action_targets() {
        let dialog = crate::WidgetId(30);
        let button = crate::content_dialog::zs_content_dialog_button_accessibility_id(
            dialog,
            crate::ZsContentDialogButton::Secondary,
        );
        let interaction = ViewInteractionPlan {
            hit_targets: Vec::new(),
            accessibility_nodes: vec![
                crate::ZsAccessibilityNode {
                    widget: dialog,
                    parent: None,
                    bounds: Rect {
                        x: 20,
                        y: 20,
                        width: 260,
                        height: 180,
                    },
                    role: crate::ZsAccessibilityRole::Dialog,
                    label: Some("Save changes?".to_owned()),
                    description: None,
                    live_region: None,
                    enabled: true,
                    selected: None,
                    checked: None,
                    range_value: None,
                    action_target: None,
                },
                crate::ZsAccessibilityNode {
                    widget: button,
                    parent: Some(dialog),
                    bounds: Rect {
                        x: 140,
                        y: 150,
                        width: 100,
                        height: 32,
                    },
                    role: crate::ZsAccessibilityRole::Button,
                    label: Some("Review".to_owned()),
                    description: None,
                    live_region: None,
                    enabled: true,
                    selected: None,
                    checked: None,
                    range_value: None,
                    action_target: Some(
                        crate::ZsAccessibilityActionTarget::ContentDialogSecondary { dialog },
                    ),
                },
            ],
            #[cfg(feature = "tooltip")]
            tooltip_targets: Vec::new(),
        };
        #[cfg(feature = "tabs")]
        let tabs = Vec::new();
        #[cfg(not(feature = "tabs"))]
        let tabs = ();
        let (tree, targets) = build_tree_update(
            "Editor",
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 240,
            },
            1.0,
            0,
            None,
            &NativeDrawPlan::new([]),
            Some(interaction),
            Some(button),
            tabs,
        );
        let focused = tree
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == tree.focus)
            .expect("focused semantic dialog button");
        assert_eq!(focused.1.role(), Role::Button);
        assert_eq!(focused.1.label(), Some("Review"));
        assert!(focused.1.supports_action(Action::Focus));
        assert!(focused.1.supports_action(Action::Click));
        assert_eq!(
            targets.get(&focused.0),
            Some(&LinuxAccessibilityTarget::Semantic(button))
        );
    }

    #[cfg(feature = "menu-flyout")]
    #[test]
    fn menu_flyout_items_expose_checked_submenu_and_highlight_state() {
        let checked_kind = ViewHitTargetKind::MenuFlyoutItem {
            path: crate::ZsMenuFlyoutPath::root(1),
            row_kind: crate::ZsMenuFlyoutRowKind::Command { checked: true },
            expanded: false,
            highlighted: true,
        };
        let mut checked = Node::new(accesskit_role(checked_kind));
        apply_view_accessibility_state(&mut checked, checked_kind);
        assert_eq!(checked.role(), Role::MenuItemCheckBox);
        assert_eq!(checked.toggled(), Some(Toggled::True));
        assert_eq!(checked.is_selected(), Some(true));

        let submenu_kind = ViewHitTargetKind::MenuFlyoutItem {
            path: crate::ZsMenuFlyoutPath::root(2),
            row_kind: crate::ZsMenuFlyoutRowKind::Submenu,
            expanded: true,
            highlighted: false,
        };
        let mut submenu = Node::new(accesskit_role(submenu_kind));
        apply_view_accessibility_state(&mut submenu, submenu_kind);
        assert_eq!(submenu.role(), Role::Menu);
        assert_eq!(submenu.is_expanded(), Some(true));
        assert_eq!(submenu.has_popup(), Some(HasPopup::Menu));
        assert_eq!(submenu.is_selected(), Some(false));
    }

    #[cfg(feature = "menu-flyout")]
    #[test]
    fn menu_flyout_items_form_a_recursive_accessibility_tree() {
        let widget = crate::WidgetId(42);
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 24,
        };
        let submenu = crate::ZsMenuFlyoutPath::root(3);
        let nested = submenu.descendant(1).expect("nested submenu path");
        let leaf = nested.descendant(2).expect("nested command path");
        let targets = [
            (
                NodeId(1),
                ViewHitTarget::with_kind(widget, bounds, ViewHitTargetKind::MenuFlyout),
            ),
            (
                NodeId(2),
                ViewHitTarget::with_kind(widget, bounds, ViewHitTargetKind::MenuFlyout),
            ),
            (
                NodeId(3),
                ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::MenuFlyoutItem {
                        path: crate::ZsMenuFlyoutPath::root(0),
                        row_kind: crate::ZsMenuFlyoutRowKind::Command { checked: false },
                        expanded: false,
                        highlighted: false,
                    },
                ),
            ),
            (
                NodeId(4),
                ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::MenuFlyoutItem {
                        path: submenu,
                        row_kind: crate::ZsMenuFlyoutRowKind::Submenu,
                        expanded: true,
                        highlighted: false,
                    },
                ),
            ),
            (
                NodeId(5),
                ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::MenuFlyoutItem {
                        path: nested,
                        row_kind: crate::ZsMenuFlyoutRowKind::Submenu,
                        expanded: true,
                        highlighted: true,
                    },
                ),
            ),
            (
                NodeId(6),
                ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::MenuFlyoutItem {
                        path: leaf,
                        row_kind: crate::ZsMenuFlyoutRowKind::Command { checked: false },
                        expanded: false,
                        highlighted: false,
                    },
                ),
            ),
        ];

        let hierarchy = menu_flyout_accessibility_hierarchy(&targets);
        assert_eq!(hierarchy.root_by_widget.get(&widget), Some(&NodeId(1)));
        assert_eq!(
            hierarchy.children_by_parent.get(&NodeId(1)),
            Some(&vec![NodeId(3), NodeId(4)])
        );
        assert_eq!(
            hierarchy.children_by_parent.get(&NodeId(4)),
            Some(&vec![NodeId(5)])
        );
        assert_eq!(
            hierarchy.children_by_parent.get(&NodeId(5)),
            Some(&vec![NodeId(6)])
        );
        assert_eq!(hierarchy.parent_by_child.get(&NodeId(6)), Some(&NodeId(5)));
        assert_eq!(
            menu_flyout_accessibility_author_id(widget, submenu),
            "zsui-menu-flyout-42-3"
        );
        assert_eq!(
            menu_flyout_accessibility_author_id(widget, nested),
            "zsui-menu-flyout-42-3-1"
        );
        assert_eq!(
            menu_flyout_accessibility_author_id(widget, leaf),
            "zsui-menu-flyout-42-3-1-2"
        );
    }
}
