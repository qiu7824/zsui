//! Layout and identity contracts for embedded native child surfaces.

use serde::{Deserialize, Serialize};

/// A safe, declarative native child provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeChildSpec {
    /// A native edit control.
    Edit { text: String, multiline: bool },
    /// A Windows Shell folder browser, behind `windows-shell`.
    Explorer { folder: std::path::PathBuf },
    /// A Windows Shell namespace tree, behind `windows-shell`.
    NamespaceTree { folder: std::path::PathBuf },
}

/// Window-local native provider declarations; no global registry is involved.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeHosts {
    pub(crate) providers: std::collections::BTreeMap<NativeHostId, NativeChildSpec>,
}

impl NativeHosts {
    /// Starts an empty provider set.
    pub fn new() -> Self {
        Self::default()
    }
    /// Registers an explicit provider, rejecting duplicate identities.
    pub fn register(mut self, id: NativeHostId, spec: NativeChildSpec) -> crate::ZsuiResult<Self> {
        if self.providers.contains_key(&id) {
            return Err(crate::ZsuiError::invalid_spec(
                "native_host.id",
                "duplicate native child ID",
            ));
        }
        self.providers.insert(id, spec);
        Ok(self)
    }
    /// Registers a system text-edit control.
    pub fn edit(
        self,
        id: NativeHostId,
        text: impl Into<String>,
        multiline: bool,
    ) -> crate::ZsuiResult<Self> {
        self.register(
            id,
            NativeChildSpec::Edit {
                text: text.into(),
                multiline,
            },
        )
    }
    /// Registers a native Windows folder browser.
    #[cfg(feature = "windows-shell")]
    pub fn explorer(
        self,
        id: NativeHostId,
        folder: impl Into<std::path::PathBuf>,
    ) -> crate::ZsuiResult<Self> {
        self.register(
            id,
            NativeChildSpec::Explorer {
                folder: folder.into(),
            },
        )
    }
    /// Registers a native Windows folder tree.
    #[cfg(feature = "windows-shell")]
    pub fn namespace_tree(
        self,
        id: NativeHostId,
        folder: impl Into<std::path::PathBuf>,
    ) -> crate::ZsuiResult<Self> {
        self.register(
            id,
            NativeChildSpec::NamespaceTree {
                folder: folder.into(),
            },
        )
    }
}

/// Stable identity of one native child, independent of its current layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NativeHostId(u64);

impl NativeHostId {
    /// Creates an application-owned identity. IDs must be unique within a window.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    /// Returns the identity's scalar representation.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// An embedded child's physical placement after layout and ancestor clipping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeHostPlacement {
    /// The child provider registered on the window.
    pub id: NativeHostId,
    /// The View's stable focus identity.
    pub widget: crate::WidgetId,
    /// Full layout bounds, in client-area physical pixels.
    pub bounds: crate::Rect,
    /// Visible part of the bounds after ancestor clipping.
    pub clip: crate::Rect,
    /// Whether the child participates in keyboard traversal.
    pub focusable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeSurfaceOcclusion {
    pub bounds: crate::Rect,
    pub hosts_before: usize,
    pub drops_before: usize,
}

/// Declares an embedded native surface. Register the matching provider on the window.
///
/// Use ordinary View width, height, flex and minimum-size modifiers for layout.
/// Native surfaces preserve their identity across retained View rebuilds.
pub fn native_host<Msg>(id: NativeHostId) -> crate::ViewNode<Msg> {
    let mut view = crate::ViewNode::new(crate::ViewNodeKind::Spacer).id(crate::WidgetId(id.get()));
    view.native_host = Some((id, true));
    view
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{View, ViewLayoutCx, ViewPaintCx};
    #[test]
    fn native_slot_keeps_identity_and_final_pixel_bounds() {
        let id = NativeHostId::new(701);
        let mut node = native_host::<()>(id);
        let bounds = crate::Rect {
            x: 12,
            y: 18,
            width: 260,
            height: 120,
        };
        let mut layout = ViewLayoutCx::new(bounds, crate::Dpi::new(144.0));
        node.layout(&mut layout);
        let mut paint = ViewPaintCx::new(crate::Dpi::new(144.0));
        node.paint(&mut paint);
        let plan = paint.into_plan();
        assert_eq!(plan.native_hosts.len(), 1);
        assert_eq!(plan.native_hosts[0].bounds, bounds);
        assert_eq!(plan.native_hosts[0].id, id);
        assert_eq!(
            node.interaction_plan().hit_targets[0].widget,
            crate::WidgetId(701)
        );
        let node = node.native_focusable(false);
        assert!(node.interaction_plan().hit_targets.is_empty());
    }

    #[cfg(feature = "windows-shell")]
    #[test]
    fn file_drop_containers_do_not_add_a_tab_stop_before_native_children() {
        let child = NativeHostId::new(810);
        let mut root = crate::column([native_host::<()>(child).height(crate::Dp::new(40.0))])
            .id(crate::WidgetId(800))
            .on_drop_files(|_| ());
        root.layout(&mut ViewLayoutCx::new(
            crate::Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 200,
            },
            crate::Dpi::standard(),
        ));
        assert_eq!(
            root.interaction_plan()
                .next_focus_target(None, 1)
                .unwrap()
                .widget,
            crate::WidgetId(810)
        );
    }
}
