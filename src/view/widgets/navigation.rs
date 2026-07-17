/// Creates a self-drawn root-to-current breadcrumb path.
///
/// Item IDs remain application-owned. The overflow flyout is controlled with
/// `.expanded(...)` and `.on_expanded_change(...)` like other shared popups.
#[cfg(feature = "breadcrumb")]
pub fn breadcrumb_bar<T, Msg>(items: impl IntoIterator<Item = T>) -> ViewNode<Msg>
where
    T: Into<crate::ZsBreadcrumbItem>,
{
    let metrics =
        crate::ZsBreadcrumbMetrics::for_platform(crate::ZsBreadcrumbPlatformStyle::current());
    ViewNode::new(ViewNodeKind::BreadcrumbBar {
        items: items.into_iter().map(Into::into).collect(),
        overflow_open: false,
        focused: None,
        on_select: None,
        on_expanded_change: None,
    })
    .native_typography_height(metrics.control_height)
}

/// Wraps one page in a modal, self-drawn content-dialog layer.
///
/// The application owns `open` and rebuilds the same node from its state. The
/// framework temporarily closes the live node after a response and emits one
/// typed result through [`ViewNode::on_dialog_result`].
#[cfg(feature = "dialog")]
pub fn content_dialog<Msg>(
    widget: WidgetId,
    open: bool,
    spec: crate::ZsContentDialogSpec,
    page: ViewNode<Msg>,
) -> ViewNode<Msg> {
    let focused_button = spec.initial_focus();
    ViewNode::<Msg>::new(ViewNodeKind::ContentDialog {
        spec,
        open,
        focused_button,
        on_result: None,
    })
    .id(widget)
    .child(page)
}

/// Wraps one page in a keyboard-first, self-drawn command-palette layer.
///
/// ZSUI filters application-owned display metadata and emits strong IDs. It
/// does not execute commands or install a global shortcut.
#[cfg(feature = "command-palette")]
pub fn command_palette<T, Msg>(
    widget: WidgetId,
    open: bool,
    query: impl Into<String>,
    items: impl IntoIterator<Item = T>,
    page: ViewNode<Msg>,
) -> ViewNode<Msg>
where
    T: Into<crate::ZsCommandPaletteItem>,
{
    let query = query.into();
    let items = items.into_iter().map(Into::into).collect::<Vec<_>>();
    let highlighted =
        crate::command_palette::command_palette_state(true, &query, &items, None).first_enabled();
    ViewNode::<Msg>::new(ViewNodeKind::CommandPalette {
        items,
        query,
        highlighted,
        open,
        placeholder: "Type a command".into(),
        no_results_text: "No matching commands".into(),
        on_query_change: None,
        on_highlight_change: None,
        on_invoke: None,
        on_open_change: None,
    })
    .id(widget)
    .child(page)
}

/// Wraps one page in a nonmodal, self-drawn in-app toast layer.
///
/// The application owns the optional toast and removes or replaces it after a
/// typed result. ZSUI owns platform ordering, pointer/keyboard interaction and
/// the timeout deadline while the toast is visible.
#[cfg(feature = "toast")]
pub fn toast_presenter<Msg>(
    widget: WidgetId,
    toast: Option<crate::ZsToastSpec>,
    page: ViewNode<Msg>,
) -> ViewNode<Msg> {
    let toast = toast.filter(|toast| !toast.is_empty());
    let focused_control = toast
        .as_ref()
        .map(crate::ZsToastSpec::initial_control)
        .unwrap_or(crate::ZsToastControl::Close);
    ViewNode::<Msg>::new(ViewNodeKind::ToastPresenter {
        toast,
        focused_control,
        on_result: None,
    })
    .id(widget)
    .child(page)
}

/// Wraps one page in a targeted, self-drawn teaching-tip layer.
///
/// The application owns `open` and identifies the target with a stable
/// [`WidgetId`]. ZSUI owns viewport-aware tail placement and typed responses.
#[cfg(feature = "teaching-tip")]
pub fn teaching_tip<Msg>(
    widget: WidgetId,
    open: bool,
    target: WidgetId,
    spec: crate::ZsTeachingTipSpec,
    page: ViewNode<Msg>,
) -> ViewNode<Msg> {
    let open = open && !spec.is_empty();
    let focused_control = spec.initial_control();
    ViewNode::<Msg>::new(ViewNodeKind::TeachingTip {
        spec,
        open,
        target,
        focused_control,
        on_result: None,
    })
    .id(widget)
    .child(page)
}

/// Creates a persistent inline status message that participates in page layout.
///
/// The application owns whether the node is present. ZSUI emits action and
/// close intent without hiding the node behind application state.
#[cfg(feature = "info-bar")]
pub fn info_bar<Msg>(widget: WidgetId, spec: crate::ZsInfoBarSpec) -> ViewNode<Msg> {
    let metrics = crate::ZsInfoBarMetrics::for_platform(crate::ZsInfoBarPlatformStyle::current());
    let desired_height = metrics.desired_height(&spec);
    let focused_control = spec.initial_control();
    ViewNode::<Msg>::new(ViewNodeKind::InfoBar {
        spec,
        focused_control,
        on_event: None,
    })
    .id(widget)
    .height(desired_height)
}
