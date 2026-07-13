use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ZsIcon;

/// Stable application-owned identity for one node in a [`ZsTreeNode`] hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZsTreeNodeId(u64);

impl ZsTreeNodeId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZsTreeNodeId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Application-owned hierarchical data displayed by TreeView.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeNode {
    id: ZsTreeNodeId,
    label: String,
    icon: Option<ZsIcon>,
    children: Vec<Self>,
    has_unrealized_children: bool,
}

impl ZsTreeNode {
    pub fn new(id: impl Into<ZsTreeNodeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            has_unrealized_children: false,
        }
    }

    pub fn icon(mut self, icon: ZsIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Keeps the disclosure glyph visible while an application loads children lazily.
    pub fn unrealized_children(mut self, has_unrealized_children: bool) -> Self {
        self.has_unrealized_children = has_unrealized_children;
        self
    }

    pub const fn id(&self) -> ZsTreeNodeId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn node_icon(&self) -> Option<ZsIcon> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeExpansionChange {
    pub node: ZsTreeNodeId,
    pub expanded: bool,
}

impl ZsTreeExpansionChange {
    pub const fn new(node: ZsTreeNodeId, expanded: bool) -> Self {
        Self { node, expanded }
    }
}

/// One currently visible row in a TreeView state snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeRowState {
    pub node: ZsTreeNodeId,
    pub parent: Option<ZsTreeNodeId>,
    pub depth: usize,
    pub expandable: bool,
    pub expanded: bool,
}

/// Read-only explicit state used by native input routing and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTreeViewState {
    pub selected: Option<ZsTreeNodeId>,
    pub rows: Vec<ZsTreeRowState>,
}

impl ZsTreeViewState {
    pub fn row(&self, node: ZsTreeNodeId) -> Option<ZsTreeRowState> {
        self.rows.iter().copied().find(|row| row.node == node)
    }

    pub fn first_visible(&self) -> Option<ZsTreeNodeId> {
        self.rows.first().map(|row| row.node)
    }

    pub fn last_visible(&self) -> Option<ZsTreeNodeId> {
        self.rows.last().map(|row| row.node)
    }

    pub fn relative_visible(&self, offset: isize) -> Option<ZsTreeNodeId> {
        let current = self
            .selected
            .and_then(|selected| self.rows.iter().position(|row| row.node == selected));
        let index = match current {
            Some(index) => index
                .saturating_add_signed(offset)
                .min(self.rows.len().saturating_sub(1)),
            None if offset < 0 => self.rows.len().checked_sub(1)?,
            None => 0,
        };
        self.rows.get(index).map(|row| row.node)
    }

    pub fn parent_of(&self, node: ZsTreeNodeId) -> Option<ZsTreeNodeId> {
        self.row(node).and_then(|row| row.parent)
    }

    pub fn first_visible_child(&self, node: ZsTreeNodeId) -> Option<ZsTreeNodeId> {
        self.rows
            .iter()
            .find(|row| row.parent == Some(node))
            .map(|row| row.node)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZsTreeVisibleNode<'a> {
    pub node: &'a ZsTreeNode,
    pub parent: Option<ZsTreeNodeId>,
    pub depth: usize,
    pub expanded: bool,
}

pub(crate) fn visible_tree_nodes<'a>(
    roots: &'a [ZsTreeNode],
    expanded: &BTreeSet<ZsTreeNodeId>,
) -> Vec<ZsTreeVisibleNode<'a>> {
    fn append<'a>(
        nodes: &'a [ZsTreeNode],
        parent: Option<ZsTreeNodeId>,
        depth: usize,
        expanded: &BTreeSet<ZsTreeNodeId>,
        seen: &mut BTreeSet<ZsTreeNodeId>,
        rows: &mut Vec<ZsTreeVisibleNode<'a>>,
    ) {
        for node in nodes {
            if !seen.insert(node.id()) {
                continue;
            }
            let is_expanded = node.is_expandable() && expanded.contains(&node.id());
            rows.push(ZsTreeVisibleNode {
                node,
                parent,
                depth,
                expanded: is_expanded,
            });
            if is_expanded {
                append(
                    node.child_nodes(),
                    Some(node.id()),
                    depth.saturating_add(1),
                    expanded,
                    seen,
                    rows,
                );
            }
        }
    }

    let mut rows = Vec::new();
    append(roots, None, 0, expanded, &mut BTreeSet::new(), &mut rows);
    rows
}

pub(crate) fn tree_view_state(
    roots: &[ZsTreeNode],
    expanded: &BTreeSet<ZsTreeNodeId>,
    selected: Option<ZsTreeNodeId>,
) -> ZsTreeViewState {
    let rows = visible_tree_nodes(roots, expanded)
        .into_iter()
        .map(|row| ZsTreeRowState {
            node: row.node.id(),
            parent: row.parent,
            depth: row.depth,
            expandable: row.node.is_expandable(),
            expanded: row.expanded,
        })
        .collect::<Vec<_>>();
    ZsTreeViewState { selected, rows }
}

pub(crate) fn find_tree_node(roots: &[ZsTreeNode], id: ZsTreeNodeId) -> Option<&ZsTreeNode> {
    for node in roots {
        if node.id() == id {
            return Some(node);
        }
        if let Some(found) = find_tree_node(node.child_nodes(), id) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> Vec<ZsTreeNode> {
        vec![ZsTreeNode::new(1, "Root").children([
            ZsTreeNode::new(2, "Child").children([ZsTreeNode::new(3, "Leaf")]),
            ZsTreeNode::new(4, "Sibling"),
        ])]
    }

    #[test]
    fn explicit_expansion_flattens_only_visible_rows_and_preserves_parents() {
        let state = tree_view_state(
            &roots(),
            &BTreeSet::from([ZsTreeNodeId::new(1), ZsTreeNodeId::new(2)]),
            Some(ZsTreeNodeId::new(3)),
        );

        assert_eq!(
            state
                .rows
                .iter()
                .map(|row| row.node.get())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            state.parent_of(ZsTreeNodeId::new(3)),
            Some(ZsTreeNodeId::new(2))
        );
        assert_eq!(
            state.first_visible_child(ZsTreeNodeId::new(1)),
            Some(ZsTreeNodeId::new(2))
        );
    }

    #[test]
    fn hidden_selection_is_preserved_and_duplicate_ids_are_not_realized_twice() {
        let mut roots = roots();
        roots.push(ZsTreeNode::new(2, "Duplicate"));
        let state = tree_view_state(
            &roots,
            &BTreeSet::from([ZsTreeNodeId::new(1)]),
            Some(ZsTreeNodeId::new(3)),
        );

        assert_eq!(state.selected, Some(ZsTreeNodeId::new(3)));
        assert_eq!(
            state
                .rows
                .iter()
                .map(|row| row.node.get())
                .collect::<Vec<_>>(),
            vec![1, 2, 4]
        );
    }
}
