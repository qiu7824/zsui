#[cfg(any(feature = "resize", feature = "label"))]
use super::Dp;
#[cfg(any(
    feature = "resize",
    feature = "label",
    feature = "icon",
    feature = "button"
))]
use super::Element;
#[cfg(any(feature = "split-view", feature = "label"))]
use super::WidgetId;

/// Platform-neutral semantic icon identity.
///
/// Native glyph names, font code points and resource paths stay in the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Icon {
    /// Application identity.
    App,
    /// Calculator action or content.
    Calculator,
    /// History action or content.
    History,
    /// Delete the preceding character.
    Backspace,
    /// Add action or content.
    Add,
    /// Search action or content.
    Search,
    /// Settings action or content.
    Settings,
    /// Sidebar action or content.
    Sidebar,
    /// Inspector action or content.
    Inspector,
    /// More action or content.
    More,
    /// Attach action or content.
    Attach,
    /// Confirm or insert a line break.
    Enter,
    /// Send the current content.
    Send,
    /// Stop action or content.
    Stop,
    /// Refresh action or content.
    Refresh,
    /// Retry a failed operation.
    Retry,
    /// Code action or content.
    Code,
    /// Tool action or content.
    Tool,
    /// Check action or content.
    Check,
    /// Info action or content.
    Info,
    /// Success action or content.
    Success,
    /// Warning action or content.
    Warning,
    /// Error action or content.
    Error,
    /// Minimize a window.
    Minimize,
    /// Close a surface.
    Close,
    /// Text action or content.
    Text,
    /// Image action or content.
    Image,
    /// File action or content.
    File,
    /// Folder action or content.
    Folder,
    /// Save action or content.
    Save,
    /// Undo action or content.
    Undo,
    /// Cut action or content.
    Cut,
    /// Pin action or content.
    Pin,
    /// Delete action or content.
    Delete,
    /// Copy action or content.
    Copy,
    /// Paste action or content.
    Paste,
    /// Edit action or content.
    Edit,
    /// Group action or content.
    Group,
    /// Phrase action or content.
    Phrase,
    /// Upward disclosure.
    ChevronUp,
    /// Downward disclosure.
    ChevronDown,
    /// Calendar action or content.
    Calendar,
    /// Backward disclosure.
    ChevronLeft,
    /// Forward disclosure.
    ChevronRight,
    /// Reveal protected text while explicitly requested.
    PasswordReveal,
}

impl Icon {
    /// All stable semantic icons, in catalog order.
    pub const ALL: [Self; 45] = [
        Self::App,
        Self::Calculator,
        Self::History,
        Self::Backspace,
        Self::Add,
        Self::Search,
        Self::Settings,
        Self::Sidebar,
        Self::Inspector,
        Self::More,
        Self::Attach,
        Self::Enter,
        Self::Send,
        Self::Stop,
        Self::Refresh,
        Self::Retry,
        Self::Code,
        Self::Tool,
        Self::Check,
        Self::Info,
        Self::Success,
        Self::Warning,
        Self::Error,
        Self::Minimize,
        Self::Close,
        Self::Text,
        Self::Image,
        Self::File,
        Self::Folder,
        Self::Save,
        Self::Undo,
        Self::Cut,
        Self::Pin,
        Self::Delete,
        Self::Copy,
        Self::Paste,
        Self::Edit,
        Self::Group,
        Self::Phrase,
        Self::ChevronUp,
        Self::ChevronDown,
        Self::Calendar,
        Self::ChevronLeft,
        Self::ChevronRight,
        Self::PasswordReveal,
    ];
    #[cfg(any(feature = "icon", feature = "button"))]
    const fn into_raw(self) -> crate::ZsIcon {
        match self {
            Self::App => crate::ZsIcon::App,
            Self::Calculator => crate::ZsIcon::Calculator,
            Self::History => crate::ZsIcon::History,
            Self::Backspace => crate::ZsIcon::Backspace,
            Self::Add => crate::ZsIcon::Add,
            Self::Search => crate::ZsIcon::Search,
            Self::Settings => crate::ZsIcon::Settings,
            Self::Sidebar => crate::ZsIcon::Sidebar,
            Self::Inspector => crate::ZsIcon::Inspector,
            Self::More => crate::ZsIcon::More,
            Self::Attach => crate::ZsIcon::Attach,
            Self::Enter => crate::ZsIcon::Enter,
            Self::Send => crate::ZsIcon::Send,
            Self::Stop => crate::ZsIcon::Stop,
            Self::Refresh => crate::ZsIcon::Refresh,
            Self::Retry => crate::ZsIcon::Retry,
            Self::Code => crate::ZsIcon::Code,
            Self::Tool => crate::ZsIcon::Tool,
            Self::Check => crate::ZsIcon::Check,
            Self::Info => crate::ZsIcon::Info,
            Self::Success => crate::ZsIcon::Success,
            Self::Warning => crate::ZsIcon::Warning,
            Self::Error => crate::ZsIcon::Error,
            Self::Minimize => crate::ZsIcon::Minimize,
            Self::Close => crate::ZsIcon::Close,
            Self::Text => crate::ZsIcon::Text,
            Self::Image => crate::ZsIcon::Image,
            Self::File => crate::ZsIcon::File,
            Self::Folder => crate::ZsIcon::Folder,
            Self::Save => crate::ZsIcon::Save,
            Self::Undo => crate::ZsIcon::Undo,
            Self::Cut => crate::ZsIcon::Cut,
            Self::Pin => crate::ZsIcon::Pin,
            Self::Delete => crate::ZsIcon::Delete,
            Self::Copy => crate::ZsIcon::Copy,
            Self::Paste => crate::ZsIcon::Paste,
            Self::Edit => crate::ZsIcon::Edit,
            Self::Group => crate::ZsIcon::Group,
            Self::Phrase => crate::ZsIcon::Phrase,
            Self::ChevronUp => crate::ZsIcon::ChevronUp,
            Self::ChevronDown => crate::ZsIcon::ChevronDown,
            Self::Calendar => crate::ZsIcon::Calendar,
            Self::ChevronLeft => crate::ZsIcon::ChevronLeft,
            Self::ChevronRight => crate::ZsIcon::ChevronRight,
            Self::PasswordReveal => crate::ZsIcon::PasswordReveal,
        }
    }
}

/// Theme-resolved semantic foreground color.
pub use crate::ColorRole;
/// Platform-owned glyph sizing for navigation, toolbars and other placements.
pub use crate::ZsIconRole as IconRole;
/// Platform-owned small, standard and large icon sizing.
#[cfg(feature = "icon")]
pub use crate::ZsIconSize as IconSize;
/// Axis used by a resizable panel.
#[cfg(feature = "resize")]
pub use crate::ZsResizeAxis as ResizeAxis;

/// Validated logical bounds for a width or height changed by the user.
#[cfg(feature = "resize")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResizeLimits(crate::ZsResizeLimits);

#[cfg(feature = "resize")]
impl ResizeLimits {
    /// Rejects nonfinite, negative or inverted limits.
    pub fn new(minimum: Dp, maximum: Dp) -> Result<Self, super::Error> {
        crate::ZsResizeLimits::new(minimum.into(), maximum.into())
            .map(Self)
            .map_err(super::Error)
    }
}

/// A controlled two-pane composition with no platform-specific types.
#[cfg(feature = "split-view")]
#[derive(Debug, Clone, Copy)]
pub struct PanelSpec(crate::ZsSplitViewSpec);

#[cfg(feature = "split-view")]
impl PanelSpec {
    /// Creates an open inline panel with its extent owned by application state.
    pub fn new(extent: Dp) -> Self {
        Self(
            crate::ZsSplitViewSpec::new(true)
                .display_mode(crate::ZsSplitViewDisplayMode::Inline)
                .pane_extent(extent.into()),
        )
    }
    /// Selects side-by-side panes or vertically stacked panels.
    pub fn axis(mut self, axis: ResizeAxis) -> Self {
        self.0 = self.0.axis(axis);
        self
    }
    /// Places the panel after the content on the selected axis.
    pub fn trailing(mut self, trailing: bool) -> Self {
        self.0 = self.0.pane_placement(if trailing {
            crate::ZsSplitViewPanePlacement::Trailing
        } else {
            crate::ZsSplitViewPanePlacement::Leading
        });
        self
    }
    /// Preserves this minimum extent for the content beside the panel.
    pub fn minimum_content_extent(mut self, extent: Dp) -> Self {
        self.0 = self.0.minimum_content_extent(extent.into());
        self
    }
    /// Switches to an overlay when both panes no longer fit their constraints.
    pub fn adaptive(mut self) -> Self {
        self.0 = self.0.display_mode(crate::ZsSplitViewDisplayMode::Adaptive);
        self
    }
}

/// Composes a resizable side panel or stacked panel around arbitrary elements.
#[cfg(feature = "split-view")]
pub fn split_panel<Message>(
    id: WidgetId,
    spec: PanelSpec,
    pane: Element<Message>,
    content: Element<Message>,
) -> Element<Message> {
    Element::from_raw(crate::split_view(
        id.into(),
        spec.0,
        pane.into_raw(),
        content.into_raw(),
    ))
}

/// Creates a theme-aware semantic icon with native platform metrics.
#[cfg(feature = "icon")]
pub fn icon<Message>(icon: Icon) -> Element<Message> {
    Element::from_raw(crate::icon(icon.into_raw()))
}

#[cfg(feature = "icon")]
impl<Message> Element<Message> {
    /// Selects the platform's semantic standalone icon size.
    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.inner = self.inner.icon_size(size);
        self
    }
    /// Selects a semantic glyph role independently of its enclosing layout slot.
    pub fn icon_role(mut self, role: IconRole) -> Self {
        self.inner = self.inner.icon_role(role);
        self
    }
    /// Selects a semantic theme color without embedding a local palette.
    pub fn icon_color(mut self, color: ColorRole) -> Self {
        self.inner = self.inner.icon_color(color);
        self
    }
}

#[cfg(feature = "resize")]
impl<Message: 'static> Element<Message> {
    /// Maps compact or overlay sidebar open-state changes into application state.
    #[cfg(feature = "label")]
    pub fn on_navigation_open_change(mut self, map: fn(bool) -> Message) -> Self {
        self.inner = self.inner.on_navigation_open_change(map);
        self
    }
    /// Constrains a pane's user-adjustable logical width or height.
    pub fn pane_resize_limits(mut self, limits: ResizeLimits) -> Self {
        self.inner = self.inner.pane_resize_limits(limits.0);
        self
    }
    /// Enables pane dragging and maps its next extent into application state.
    pub fn on_resize(mut self, map: fn(Dp) -> Message) -> Self {
        self.inner = self
            .inner
            .on_pane_resize_with(move |extent| map(Dp::new(extent.0)));
        self
    }
}

/// Explicit navigation items and one content region.
#[cfg(feature = "label")]
pub struct NavigationSpec<Message>(crate::ZsNavigationViewSpec<Message>);

#[cfg(feature = "label")]
impl<Message> NavigationSpec<Message> {
    /// Creates navigation with application-owned title and subtitle text.
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self(crate::ZsNavigationViewSpec::new(title, subtitle))
    }
    /// Adds one semantic navigation item.
    pub fn item(mut self, item: Element<Message>) -> Self {
        self.0 = self.0.item(item.into_raw());
        self
    }
    /// Adds an item to the bottom navigation group.
    pub fn footer_item(mut self, item: Element<Message>) -> Self {
        self.0 = self.0.footer_item(item.into_raw());
        self
    }
    /// Supplies the application-owned sidebar width.
    pub fn pane_width(mut self, width: Dp) -> Self {
        self.0 = self.0.pane_width(width.into());
        self
    }
    /// Supplies controlled open state for compact and overlay sidebars.
    pub fn pane_open(mut self, open: bool) -> Self {
        self.0 = self.0.pane_open(open);
        self
    }
    /// Sets the content width reserved while the sidebar is resized.
    pub fn minimum_content_width(mut self, width: Dp) -> Self {
        self.0 = self.0.minimum_content_width(width.into());
        self
    }
    /// Attaches content and a stable navigation identity.
    pub fn content(mut self, id: WidgetId, content: Element<Message>) -> Self {
        self.0 = self.0.content(id.into(), content.into_raw());
        self
    }
}

/// Creates navigation using the current desktop's native composition profile.
#[cfg(feature = "label")]
pub fn navigation_view<Message>(spec: NavigationSpec<Message>) -> Element<Message> {
    Element::from_raw(crate::navigation_view(spec.0))
}

/// Creates a semantic item whose selection remains explicit application state.
#[cfg(feature = "button")]
pub fn navigation_item<Message>(
    label: impl Into<String>,
    icon: Icon,
    selected: bool,
) -> Element<Message> {
    Element::from_raw(crate::navigation_item(label, icon.into_raw(), selected))
}

/// Stable application-owned table column identity.
#[cfg(feature = "table")]
pub use crate::ZsTableColumnId as ColumnId;
/// Read-only application-owned cell values.
#[cfg(feature = "table")]
pub use crate::ZsTableRow as TableRow;
/// Stable application-owned table row identity.
#[cfg(feature = "table")]
pub use crate::ZsTableRowId as RowId;
/// Typed sort request; the application owns data ordering.
#[cfg(feature = "table")]
pub use crate::ZsTableSort as TableSort;

/// Column presentation and resize constraints.
#[cfg(feature = "table")]
#[derive(Debug, Clone)]
pub struct TableColumn(crate::ZsTableColumn);

#[cfg(feature = "table")]
impl TableColumn {
    /// Creates a column with a stable identity and application-owned header.
    pub fn new(id: ColumnId, header: impl Into<String>) -> Self {
        Self(crate::ZsTableColumn::new(id, header))
    }
    /// Sets a fixed logical width.
    pub fn width(mut self, width: Dp) -> Self {
        self.0 = self.0.fixed_width(width.into());
        self
    }
    /// Shares remaining width by a nonzero weight.
    pub fn fill(mut self, weight: u16) -> Self {
        self.0 = self.0.fill_width(weight);
        self
    }
    /// Enables typed sort requests from this column header.
    pub fn sortable(mut self, enabled: bool) -> Self {
        self.0 = self.0.sortable(enabled);
        self
    }
    /// Controls whether this column exposes a resize handle.
    pub fn resizable(mut self, enabled: bool) -> Self {
        self.0 = self.0.resizable(enabled);
        self
    }
    /// Sets the column's resize bounds.
    pub fn resize_limits(mut self, limits: ResizeLimits) -> Self {
        self.0 = self.0.resize_limits(limits.0);
        self
    }
    /// Returns the stable identity, independent of display position.
    pub fn id(&self) -> ColumnId {
        self.0.id()
    }
    /// Applies a typed resize result to application-owned column state.
    pub fn resize_to(&mut self, width: Dp) {
        self.0.resize_to(width.into());
    }
}

/// One column-width update emitted by a divider.
#[cfg(feature = "table")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnResize {
    /// Stable column identity.
    pub column: ColumnId,
    /// Next logical width after applying layout and resize constraints.
    pub width: Dp,
}

/// Creates a table from explicit columns and read-only rows.
#[cfg(feature = "table")]
pub fn data_grid<Message>(
    columns: impl IntoIterator<Item = TableColumn>,
    rows: impl IntoIterator<Item = TableRow>,
) -> Element<Message> {
    Element::from_raw(crate::data_grid(
        columns.into_iter().map(|column| column.0),
        rows,
    ))
}

#[cfg(feature = "table")]
impl<Message: Clone + 'static> Element<Message> {
    /// Enables column resizing and emits a stable column ID plus logical width.
    pub fn on_column_resize(mut self, map: fn(ColumnResize) -> Message) -> Self {
        self.inner = self.inner.on_table_column_resize_with(move |change| {
            map(ColumnResize {
                column: change.column,
                width: Dp::new(change.width.0),
            })
        });
        self
    }
    /// Emits selection without changing the application's data source.
    pub fn on_row_select(mut self, map: fn(RowId) -> Message) -> Self {
        self.inner = self.inner.on_table_select(map);
        self
    }
    /// Emits a sort request without sorting application rows.
    pub fn on_sort(mut self, map: fn(TableSort) -> Message) -> Self {
        self.inner = self.inner.on_table_sort(map);
        self
    }
    /// Supplies the selected row from application state.
    pub fn selected_row(mut self, selected: Option<RowId>) -> Self {
        self.inner = self.inner.selected_table_row(selected);
        self
    }
}

/// Creates an icon-only button with a required accessible action label.
#[cfg(feature = "button")]
pub fn icon_button<Message>(label: impl Into<String>, icon: Icon) -> Element<Message> {
    Element::from_raw(crate::icon_button(label, icon.into_raw()))
}

/// Creates a toolbar action with the platform's icon and label metrics.
#[cfg(feature = "button")]
pub fn toolbar_button<Message>(label: impl Into<String>, icon: Icon) -> Element<Message> {
    Element::from_raw(crate::toolbar_button(label, icon.into_raw()))
}
