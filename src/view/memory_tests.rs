use super::*;

// Fixed representation for same-toolchain, same-feature allocation comparisons.
// The original native integration node retained tooltip/accessibility inline.
#[allow(dead_code)]
struct BaselineViewNode<Msg> {
    #[cfg(feature = "windows-shell")]
    file_drop_handler: Option<(
        crate::DragEffects,
        ViewMessageMapper<crate::FilesDropped, Msg>,
    )>,
    #[cfg(feature = "native-window-events")]
    native_window_handler: Option<ViewMessageMapper<crate::NativeWindowEvent, Msg>>,
    #[cfg(feature = "native-host")]
    pub(crate) native_host: Option<(crate::NativeHostId, bool)>,
    pub id: Option<WidgetId>,
    pub kind: ViewNodeKind<Msg>,
    pub style: ViewStyle,
    pub children: Vec<BaselineViewNode<Msg>>,
    #[cfg(feature = "label")]
    navigation_open_source: Option<bool>,
    #[cfg(all(feature = "label", feature = "resize"))]
    on_navigation_open_change: Option<ViewMessageMapper<bool, Msg>>,
    #[cfg(feature = "icon")]
    icon_role: Option<crate::ZsIconRole>,
    #[cfg(feature = "resize")]
    pane_resize_limits: Option<crate::ZsResizeLimits>,
    #[cfg(feature = "resize")]
    on_pane_resize: Option<ViewMessageMapper<Dp, Msg>>,
    #[cfg(feature = "table")]
    on_table_column_resize: Option<ViewMessageMapper<crate::ZsTableColumnResize, Msg>>,
    #[cfg(feature = "table")]
    on_table_columns_change: Option<ViewMessageMapper<Vec<crate::ZsTableColumn>, Msg>>,
    #[cfg(feature = "text-input-core")]
    editable_text: Option<ViewEditableTextDescriptor>,
    #[cfg(feature = "tooltip")]
    tooltip: Option<crate::ZsTooltipSpec>,
    #[cfg(feature = "accessibility")]
    accessibility: Option<crate::ZsAccessibilitySpec>,
    bounds: Option<Rect>,
    layout_dpi: Dpi,
    #[cfg(all(
        test,
        any(
            feature = "badge",
            feature = "split-view",
            feature = "button",
            feature = "icon",
            feature = "label",
            feature = "scroll",
            feature = "virtual-list"
        )
    ))]
    platform_style_override: Option<crate::ZsBaseControlPlatformStyle>,
    pub(crate) typography_scaled_height: bool,
    #[cfg(feature = "list")]
    pub(crate) list_item_horizontal_inset: Option<Dp>,
    #[cfg(feature = "combo")]
    combo_first_visible_option: Option<usize>,
    #[cfg(feature = "ui-viewer")]
    document_poll_interval_ms: Option<u64>,
    #[cfg(feature = "scroll")]
    adaptive_scroll_offset_y: Dp,
    #[cfg(feature = "scroll")]
    scroll_offset_controlled: bool,
    #[cfg(feature = "scroll")]
    resolved_scroll_content_height: Dp,

    #[cfg(feature = "shortcuts")]
    shortcuts: Vec<(crate::ZsAccelerator, Msg)>,
    #[cfg(feature = "window-chrome")]
    window_drag_region: bool,
    #[cfg(feature = "window-chrome")]
    window_state_message: Option<fn(crate::ZsWindowState) -> Msg>,
    message: PhantomData<fn() -> Msg>,
}
impl<Msg> BaselineViewNode<Msg> {
    fn from_node(mut node: ViewNode<Msg>) -> Self {
        // Preserve spare slots: representation comparisons must not shrink
        // only the baseline's child vectors during conversion.
        let mut children = Vec::with_capacity(node.children.capacity());
        children.extend(node.children.into_iter().map(Self::from_node));
        #[cfg(feature = "tooltip")]
        let tooltip = node
            .metadata
            .as_mut()
            .and_then(|metadata| metadata.tooltip.take());
        #[cfg(feature = "accessibility")]
        let accessibility = node
            .metadata
            .as_mut()
            .and_then(|metadata| metadata.accessibility.take());
        Self {
            #[cfg(feature = "windows-shell")]
            file_drop_handler: node.file_drop_handler,
            #[cfg(feature = "native-window-events")]
            native_window_handler: node.native_window_handler,
            #[cfg(feature = "native-host")]
            native_host: node.native_host,
            id: node.id,
            kind: node.kind,
            style: node.style,
            children,
            #[cfg(feature = "label")]
            navigation_open_source: node.navigation_open_source,
            #[cfg(all(feature = "label", feature = "resize"))]
            on_navigation_open_change: node.on_navigation_open_change,
            #[cfg(feature = "icon")]
            icon_role: node.icon_role,
            #[cfg(feature = "resize")]
            pane_resize_limits: node.pane_resize_limits,
            #[cfg(feature = "resize")]
            on_pane_resize: node.on_pane_resize,
            #[cfg(feature = "table")]
            on_table_column_resize: node.on_table_column_resize,
            #[cfg(feature = "table")]
            on_table_columns_change: node.on_table_columns_change,
            #[cfg(feature = "text-input-core")]
            editable_text: node.editable_text,
            #[cfg(feature = "tooltip")]
            tooltip: tooltip,
            #[cfg(feature = "accessibility")]
            accessibility: accessibility,
            bounds: node.bounds,
            layout_dpi: node.layout_dpi,
            #[cfg(all(
                test,
                any(
                    feature = "badge",
                    feature = "split-view",
                    feature = "button",
                    feature = "icon",
                    feature = "label",
                    feature = "scroll",
                    feature = "virtual-list"
                )
            ))]
            platform_style_override: node.platform_style_override,
            typography_scaled_height: node.typography_scaled_height,
            #[cfg(feature = "list")]
            list_item_horizontal_inset: node.list_item_horizontal_inset,
            #[cfg(feature = "combo")]
            combo_first_visible_option: node.combo_first_visible_option,
            #[cfg(feature = "ui-viewer")]
            document_poll_interval_ms: node.document_poll_interval_ms,
            #[cfg(feature = "scroll")]
            adaptive_scroll_offset_y: node.adaptive_scroll_offset_y,
            #[cfg(feature = "scroll")]
            scroll_offset_controlled: node.scroll_offset_controlled,
            #[cfg(feature = "scroll")]
            resolved_scroll_content_height: node.resolved_scroll_content_height,
            #[cfg(feature = "shortcuts")]
            shortcuts: node.shortcuts,
            #[cfg(feature = "window-chrome")]
            window_drag_region: node.window_drag_region,
            #[cfg(feature = "window-chrome")]
            window_state_message: node.window_state_message,
            message: node.message,
        }
    }
}

#[test]
fn ordinary_nodes_account_for_inline_and_indirect_allocations() {
    fn measure(name: &str, build: impl Fn() -> ViewNode<()>) -> serde_json::Value {
        let (current, current_allocation) =
            crate::rust_text_renderer::allocation_tests::measure_retained(&build);
        let (baseline, baseline_allocation) =
            crate::rust_text_renderer::allocation_tests::measure_retained(|| {
                BaselineViewNode::from_node(build())
            });
        assert!(
            current_allocation.retained_bytes < baseline_allocation.retained_bytes,
            "{name}: current={}, baseline={}",
            current_allocation.retained_bytes,
            baseline_allocation.retained_bytes
        );
        let report = serde_json::json!({
            "scene":name,"node_count":current.node_count(),
            "current_owned_bytes":current_allocation.retained_bytes,
            "baseline_owned_bytes":baseline_allocation.retained_bytes,
            "current_construction_allocations":current_allocation.allocations,
            "baseline_conversion_allocations":baseline_allocation.allocations
        });
        std::hint::black_box((current, baseline));
        report
    }
    assert!(std::mem::size_of::<ViewNode<()>>() < std::mem::size_of::<BaselineViewNode<()>>());
    #[allow(unused_mut)]
    let mut cases = vec![measure("1000 labels", || {
        column((0..1000).map(|index| crate::text::<()>(format!("Row {index} 中文"))))
    })];
    #[cfg(feature = "button")]
    cases.push(measure("1000 buttons", || {
        column((0..1000).map(|index| crate::button::<()>(format!("Action {index}"))))
    }));
    #[cfg(all(feature = "button", feature = "textbox"))]
    cases.push(measure("200 form rows with sparse metadata", || {
        column((0..200).map(|index| {
            let mut label = crate::text::<()>(format!("Field {index}"));
            if index % 10 == 0 {
                label = label
                    .tooltip("Field help")
                    .accessibility_label("Named field");
            }
            row([
                label,
                crate::textbox(format!("Value {index}")),
                crate::button("Apply"),
            ])
        }))
    }));
    let report = serde_json::json!({
        "schema":"zsui.node-memory/v1",
        "current_inline_bytes":std::mem::size_of::<ViewNode<()>>(),
        "baseline_inline_bytes":std::mem::size_of::<BaselineViewNode<()>>(),
        "includes_indirect_allocations":true,
        "measurement":"same-thread retained allocator delta; same payload types, strings, child vectors and optional metadata; conversion allocation counts are not historical construction counts",
        "cases":cases
    });
    if let Some(path) = std::env::var_os("ZSUI_LAYOUT_PROOF_DIR") {
        let directory = std::path::PathBuf::from(path);
        assert!(directory.is_absolute());
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("node-memory.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
