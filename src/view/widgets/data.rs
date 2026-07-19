
pub fn row<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    let children = children.into_iter().collect::<Vec<_>>();
    let intrinsic_height = children
        .iter()
        .filter_map(|child| child.style.height.or(child.style.min_height))
        .map(|height| height.0)
        .fold(0.0_f32, f32::max);
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Row,
    })
    .children(children)
    .min_height(Dp::new(intrinsic_height))
    .flex(0.0)
}

pub fn column<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Column,
    })
    .children(children)
}

/// Groups related content using the target desktop's information architecture.
///
/// This is a semantic composition primitive, not a Windows card with a few
/// spacing values changed: Windows gets a raised Fluent group, macOS gets an
/// unboxed form section, and GTK gets an Adwaita-style boxed group with row
/// separators. Applications keep one typed view tree while the framework owns
/// the platform composition decision.
#[cfg(feature = "label")]
pub fn section<Msg>(
    title: impl Into<String>,
    children: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    section_for_style(
        crate::ZsBaseControlPlatformStyle::current(),
        title,
        children,
    )
}

/// Deterministic variant used by proof fixtures and framework tests that need
/// to inspect more than the host platform.
#[cfg(feature = "label")]
pub(crate) fn section_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    title: impl Into<String>,
    children: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    let spacing = crate::ZsuiSpacingTokens::for_platform(platform);
    let radius = crate::ZsuiRadiusTokens::for_platform(platform);
    let heading = styled_text_for_platform(
        platform.typography(),
        title,
        crate::SemanticTextStyle {
            role: crate::TextRole::Body,
            color: match platform {
                crate::ZsBaseControlPlatformStyle::Macos => crate::ColorRole::SecondaryText,
                crate::ZsBaseControlPlatformStyle::Windows
                | crate::ZsBaseControlPlatformStyle::Gtk => crate::ColorRole::PrimaryText,
            },
            weight: crate::TextWeight::Semibold,
            horizontal_align: crate::HorizontalAlign::Start,
            vertical_align: crate::VerticalAlign::Center,
            wrap: crate::TextWrap::NoWrap,
            ellipsis: false,
        },
    );
    let children = children.into_iter().collect::<Vec<_>>();
    let title_height = crate::TextRole::Body
        .metrics_for(platform.typography())
        .line_height;
    let fallback_child_height = title_height;
    let child_count = children.len();
    let child_content_height = children
        .iter()
        .map(|child| {
            child
                .style
                .height
                .or(child.style.min_height)
                .unwrap_or(Dp::new(fallback_child_height))
                .0
        })
        .sum::<f32>();
    match platform {
        crate::ZsBaseControlPlatformStyle::Windows => column([
            heading,
            column(children)
                .padding(spacing.lg)
                .gap(spacing.content_gap)
                .radius(radius.medium)
                .bg(crate::ThemeColorToken::SurfaceRaised),
        ])
        .gap(spacing.sm)
        .native_typography_min_height(Dp::new(
            title_height
                + spacing.sm.0
                + spacing.lg.0 * 2.0
                + child_content_height
                + spacing.content_gap.0 * child_count.saturating_sub(1) as f32,
        )),
        crate::ZsBaseControlPlatformStyle::Macos => {
            column([heading, column(children).gap(spacing.md)])
                .gap(spacing.md)
                .native_typography_min_height(Dp::new(
                    title_height
                        + spacing.md.0
                        + child_content_height
                        + spacing.md.0 * child_count.saturating_sub(1) as f32,
                ))
        }
        crate::ZsBaseControlPlatformStyle::Gtk => {
            // GNOME's boxed-list pattern uses padded rows separated by thin
            // dividers. Padding is part of the row's outer geometry; keeping
            // it out of the minimum height makes the content box collapse
            // and clips controls (especially switches and progress rings).
            let row_padding = spacing.md;
            let mut rows = Vec::with_capacity(children.len().saturating_mul(2));
            for (index, child) in children.into_iter().enumerate() {
                if index > 0 {
                    rows.push(
                        spacer()
                            .height(Dp::new(1.0))
                            .flex(0.0)
                            .bg(crate::ThemeColorToken::Border),
                    );
                }
                let intrinsic_height = child
                    .style
                    .height
                    .or(child.style.min_height)
                    .unwrap_or(Dp::new(
                        crate::TextRole::Body
                            .metrics_for(platform.typography())
                            .line_height,
                    ));
                rows.push(
                    child
                        .padding(row_padding)
                        .min_height(Dp::new(intrinsic_height.0 + row_padding.0 * 2.0)),
                );
            }
            column([
                heading,
                column(rows)
                    .gap(Dp::new(0.0))
                    .radius(radius.medium)
                    .bg(crate::ThemeColorToken::SurfaceRaised),
            ])
            .gap(spacing.sm)
            .native_typography_min_height(Dp::new(
                title_height
                    + spacing.sm.0
                    + child_content_height
                    + row_padding.0 * 2.0 * child_count as f32
                    + child_count.saturating_sub(1) as f32,
            ))
        }
    }
}

/// Platform-neutral declaration for a desktop navigation view.
///
/// Applications provide semantic items and optional footer items once. ZSUI
/// selects the target-native pane geometry and grouping internally. A pane
/// width override is intentionally platform-neutral: it changes the requested
/// application parameter without exposing a platform enum or backend handle.
#[cfg(feature = "label")]
pub struct ZsNavigationViewSpec<Msg> {
    id: Option<WidgetId>,
    title: String,
    subtitle: String,
    items: Vec<ViewNode<Msg>>,
    footer_items: Vec<ViewNode<Msg>>,
    pane_width: Option<Dp>,
    minimum_content_width: Dp,
    content: Option<Box<ViewNode<Msg>>>,
}

#[cfg(feature = "label")]
impl<Msg> ZsNavigationViewSpec<Msg> {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            id: None,
            title: title.into(),
            subtitle: subtitle.into(),
            items: Vec::new(),
            footer_items: Vec::new(),
            pane_width: None,
            minimum_content_width: Dp::new(0.0),
            content: None,
        }
    }

    pub fn item(mut self, item: ViewNode<Msg>) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn footer_item(mut self, item: ViewNode<Msg>) -> Self {
        self.footer_items.push(item);
        self
    }

    pub fn footer_items(mut self, items: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.footer_items.extend(items);
        self
    }

    pub fn pane_width(mut self, width: Dp) -> Self {
        self.pane_width = Some(Dp::new(width.0.max(0.0)));
        self
    }

    /// Declares the application's platform-neutral content width constraint.
    ///
    /// AppKit uses the same constraint-driven collapse rule as a split-view
    /// sidebar. GTK combines it with its adaptive breakpoint. Windows keeps
    /// the documented NavigationView Auto thresholds.
    pub fn minimum_content_width(mut self, width: Dp) -> Self {
        self.minimum_content_width = Dp::new(width.0.max(0.0));
        self
    }

    /// Supplies the content pane and the stable identity used by the
    /// framework-owned adaptive pane toggle.
    ///
    /// Requiring the identity here prevents an adaptive shell from rendering
    /// an inert compact/minimal toggle. Static pane-only navigation does not
    /// need an identity.
    pub fn content(mut self, id: WidgetId, content: ViewNode<Msg>) -> Self {
        self.id = Some(id);
        self.content = Some(Box::new(content));
        self
    }
}

/// Builds the navigation surface for the target desktop family.
///
/// Navigation is a platform contract, not a colored `Column`: Windows uses a
/// Fluent NavigationView pane, macOS uses an unboxed source list, and GTK
/// uses a grouped Adwaita sidebar list. The caller supplies semantic rows;
/// the framework owns their information architecture and chrome.
#[cfg(feature = "label")]
pub fn navigation_view<Msg>(spec: ZsNavigationViewSpec<Msg>) -> ViewNode<Msg> {
    navigation_view_impl(crate::ZsBaseControlPlatformStyle::current(), spec)
}

/// Deterministic navigation composition used by platform proof fixtures.
#[cfg(all(test, feature = "label"))]
pub(crate) fn navigation_view_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    spec: ZsNavigationViewSpec<Msg>,
) -> ViewNode<Msg> {
    navigation_view_impl(platform, spec).with_platform_style_override(platform)
}

#[cfg(feature = "label")]
fn navigation_view_impl<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    spec: ZsNavigationViewSpec<Msg>,
) -> ViewNode<Msg> {
    let ZsNavigationViewSpec {
        id,
        title,
        subtitle,
        items,
        footer_items,
        pane_width,
        minimum_content_width,
        content,
    } = spec;
    if let Some(content) = content {
        let item_count = items.len();
        let footer_count = footer_items.len();
        let children = items
            .into_iter()
            .chain(footer_items)
            .chain([*content])
            .collect::<Vec<_>>();
        let mut navigation = ViewNode::new(ViewNodeKind::NavigationView {
            title,
            subtitle,
            item_count,
            footer_count,
            pane_open: false,
            pane_width,
            minimum_content_width,
        })
        .children(children);
        navigation.id = id;
        return navigation;
    }
    let spacing = crate::ZsuiSpacingTokens::for_platform(platform);
    let radius = crate::ZsuiRadiusTokens::for_platform(platform);
    let title_style = crate::SemanticTextStyle {
        role: match platform {
            crate::ZsBaseControlPlatformStyle::Windows => crate::TextRole::Subtitle,
            crate::ZsBaseControlPlatformStyle::Macos
            | crate::ZsBaseControlPlatformStyle::Gtk => crate::TextRole::Body,
        },
        color: crate::ColorRole::PrimaryText,
        weight: crate::TextWeight::Semibold,
        horizontal_align: crate::HorizontalAlign::Start,
        vertical_align: crate::VerticalAlign::Center,
        wrap: crate::TextWrap::NoWrap,
        ellipsis: false,
    };
    let heading = styled_text_for_platform(platform.typography(), title, title_style);
    let subtitle = styled_text_for_platform(
        platform.typography(),
        subtitle,
        crate::SemanticTextStyle {
            role: crate::TextRole::Caption,
            color: crate::ColorRole::SecondaryText,
            weight: crate::TextWeight::Regular,
            horizontal_align: crate::HorizontalAlign::Start,
            vertical_align: crate::VerticalAlign::Center,
            wrap: crate::TextWrap::NoWrap,
            ellipsis: true,
        },
    );
    // Keep the navigation composition usable with `label` alone. The
    // interactive navigation row renderer is optional (`button`), but the
    // shell primitive still needs stable pane geometry for static views.
    let (default_pane_width, horizontal_inset) = match platform {
        crate::ZsBaseControlPlatformStyle::Windows => (Dp::new(320.0), 32.0),
        crate::ZsBaseControlPlatformStyle::Macos => (Dp::new(240.0), 24.0),
        crate::ZsBaseControlPlatformStyle::Gtk => (Dp::new(280.0), 32.0),
    };
    let open_pane_width = pane_width.unwrap_or(default_pane_width);
    let item_width = Dp::new(open_pane_width.0 - horizontal_inset);
    let items = items
        .into_iter()
        .map(|item| item.width(item_width))
        .collect::<Vec<_>>();
    let footer_items = footer_items
        .into_iter()
        .map(|item| item.width(item_width))
        .collect::<Vec<_>>();
    let navigation = match platform {
        crate::ZsBaseControlPlatformStyle::Windows => {
            let mut children = vec![heading, subtitle, column(items).gap(spacing.xs)];
            if !footer_items.is_empty() {
                children.push(spacer());
                children.push(column(footer_items).gap(spacing.xs));
            }
            column(children)
                .padding(spacing.lg)
                .gap(spacing.sm)
                .bg(crate::ThemeColorToken::SurfaceRaised)
        }
        crate::ZsBaseControlPlatformStyle::Macos => {
            let mut children = vec![heading, subtitle, column(items).gap(spacing.xs)];
            if !footer_items.is_empty() {
                children.push(spacer());
                children.push(column(footer_items).gap(spacing.xs));
            }
            column(children)
                .padding(spacing.lg)
                .gap(spacing.sm)
                .bg(crate::ThemeColorToken::SurfaceRaised)
        }
        crate::ZsBaseControlPlatformStyle::Gtk => {
            let mut rows = Vec::with_capacity(items.len().saturating_mul(2));
            for (index, item) in items.into_iter().enumerate() {
                if index > 0 {
                    rows.push(
                        spacer()
                            .height(Dp::new(1.0))
                            .flex(0.0)
                            .bg(crate::ThemeColorToken::Border),
                    );
                }
                rows.push(item);
            }
            let mut children = vec![
                heading,
                subtitle,
                column(rows)
                    .gap(Dp::new(0.0))
                    .padding(Dp::new(4.0))
                    .radius(radius.medium)
                    .bg(crate::ThemeColorToken::SurfaceRaised),
            ];
            if !footer_items.is_empty() {
                children.push(spacer());
                children.push(
                    column(footer_items)
                        .gap(spacing.xs)
                        .padding(spacing.xs)
                        .radius(radius.medium)
                        .bg(crate::ThemeColorToken::SurfaceRaised),
                );
            }
            column(children)
                .padding(spacing.md)
                .gap(spacing.sm)
                .bg(crate::ThemeColorToken::Surface)
        }
    };
    navigation.width(open_pane_width)
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZsNavigationViewLayoutMode {
    Expanded,
    Compact,
    Collapsed,
}

#[cfg(feature = "label")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ZsNavigationViewLayout {
    pub mode: ZsNavigationViewLayoutMode,
    pub pane_bounds: Option<Rect>,
    pub content_bounds: Rect,
    pub header_bounds: Option<Rect>,
    pub toggle_bounds: Option<Rect>,
    pub title_bounds: Option<Rect>,
    pub subtitle_bounds: Option<Rect>,
    pub item_bounds: Rect,
    pub footer_bounds: Rect,
    pub scrim_bounds: Option<Rect>,
    pub overlay_open: bool,
}

#[cfg(feature = "label")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn zs_navigation_view_layout(
    bounds: Rect,
    platform: crate::ZsBaseControlPlatformStyle,
    pane_width: Option<Dp>,
    minimum_content_width: Dp,
    pane_open: bool,
    dpi: Dpi,
    typography_scale: f32,
) -> ZsNavigationViewLayout {
    let scale = dpi.scale_factor().max(f32::EPSILON);
    let logical_width = bounds.width.max(0) as f32 / scale;
    let spacing = crate::ZsuiSpacingTokens::for_platform(platform);
    let padding = match platform {
        crate::ZsBaseControlPlatformStyle::Gtk => spacing.md,
        crate::ZsBaseControlPlatformStyle::Windows
        | crate::ZsBaseControlPlatformStyle::Macos => spacing.lg,
    }
    .to_px(dpi)
    .round_i32()
    .max(0);
    let gap = spacing.sm.to_px(dpi).round_i32().max(0);
    let title_line = Dp::new(
        crate::TextRole::Subtitle
            .metrics_for(platform.typography())
            .line_height
            * typography_scale,
    )
    .to_px(dpi)
    .round_i32()
    .max(1);
    let subtitle_line = Dp::new(
        crate::TextRole::Caption
            .metrics_for(platform.typography())
            .line_height
            * typography_scale,
    )
    .to_px(dpi)
    .round_i32()
    .max(1);

    let open_pane_dp = pane_width.map_or_else(
        || match platform {
            // NavigationView.OpenPaneLength defaults to 320 epx.
            crate::ZsBaseControlPlatformStyle::Windows => 320.0,
            // AppKit's standard sidebar initializer owns its min/max values;
            // 240 remains ZSUI's preferred thickness until the backend can
            // feed those runtime values into this profile.
            crate::ZsBaseControlPlatformStyle::Macos => 240.0,
            // AdwNavigationSplitView defaults to 25%, constrained to 180sp
            // through 280sp.
            crate::ZsBaseControlPlatformStyle::Gtk => {
                (logical_width * 0.25).clamp(180.0, 280.0)
            }
        },
        |width| width.0.max(0.0),
    );
    let open_pane_width = Dp::new(open_pane_dp)
        .to_px(dpi)
        .round_i32()
        .clamp(0, bounds.width.max(0));
    let minimum_content_dp = minimum_content_width.0.max(0.0);
    let mode = match platform {
        crate::ZsBaseControlPlatformStyle::Windows if logical_width >= 1008.0 => {
            ZsNavigationViewLayoutMode::Expanded
        }
        crate::ZsBaseControlPlatformStyle::Windows if logical_width > 640.0 => {
            ZsNavigationViewLayoutMode::Compact
        }
        crate::ZsBaseControlPlatformStyle::Windows => ZsNavigationViewLayoutMode::Collapsed,
        crate::ZsBaseControlPlatformStyle::Macos
            if minimum_content_dp > 0.0
                && logical_width < open_pane_dp + minimum_content_dp =>
        {
            ZsNavigationViewLayoutMode::Collapsed
        }
        crate::ZsBaseControlPlatformStyle::Macos => ZsNavigationViewLayoutMode::Expanded,
        crate::ZsBaseControlPlatformStyle::Gtk => {
            let constraint_breakpoint = if minimum_content_dp > 0.0 {
                180.0 + minimum_content_dp
            } else {
                0.0
            };
            if logical_width <= 400.0_f32.max(constraint_breakpoint) {
                ZsNavigationViewLayoutMode::Collapsed
            } else {
                ZsNavigationViewLayoutMode::Expanded
            }
        }
    };
    let compact_width = Dp::new(48.0)
        .to_px(dpi)
        .round_i32()
        .clamp(0, bounds.width.max(0));
    let base_control = crate::ZsBaseControlMetrics::for_platform(platform);
    let collapsed_header_height = match platform {
        // The NavigationView content header is 52 epx in Minimal mode.
        crate::ZsBaseControlPlatformStyle::Windows => Dp::new(52.0),
        crate::ZsBaseControlPlatformStyle::Macos
        | crate::ZsBaseControlPlatformStyle::Gtk => Dp::new(
            base_control.button_height.0 + spacing.sm.0 * 2.0,
        ),
    }
    .to_px(dpi)
    .round_i32()
    .max(1)
    .min(bounds.height.max(0));
    let overlay_open = mode != ZsNavigationViewLayoutMode::Expanded && pane_open;
    let inline_pane_width = match mode {
        ZsNavigationViewLayoutMode::Expanded => open_pane_width,
        ZsNavigationViewLayoutMode::Compact => compact_width,
        ZsNavigationViewLayoutMode::Collapsed => 0,
    };
    let pane_bounds = if overlay_open {
        Some(Rect {
            x: bounds.x,
            y: bounds.y,
            width: open_pane_width,
            height: bounds.height,
        })
    } else if inline_pane_width > 0 {
        Some(Rect {
            x: bounds.x,
            y: bounds.y,
            width: inline_pane_width,
            height: bounds.height,
        })
    } else {
        None
    };
    let content_x = bounds.x.saturating_add(inline_pane_width);
    let content_y = if mode == ZsNavigationViewLayoutMode::Collapsed {
        bounds.y.saturating_add(collapsed_header_height)
    } else {
        bounds.y
    };
    let content_bounds = Rect {
        x: content_x,
        y: content_y,
        width: bounds
            .width
            .saturating_sub(inline_pane_width)
            .max(0),
        height: bounds
            .y
            .saturating_add(bounds.height)
            .saturating_sub(content_y)
            .max(0),
    };
    let header_bounds = (mode == ZsNavigationViewLayoutMode::Collapsed).then_some(Rect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: collapsed_header_height,
    });
    let toggle_size = match mode {
        ZsNavigationViewLayoutMode::Expanded => 0,
        ZsNavigationViewLayoutMode::Compact => compact_width.min(collapsed_header_height.max(1)),
        ZsNavigationViewLayoutMode::Collapsed => collapsed_header_height,
    };
    let toggle_bounds = (toggle_size > 0).then_some(Rect {
        x: bounds.x,
        y: bounds.y,
        width: toggle_size,
        height: toggle_size,
    });
    let expanded_header_height = padding
        .saturating_add(title_line)
        .saturating_add(gap)
        .saturating_add(subtitle_line)
        .saturating_add(gap);
    let pane = pane_bounds.unwrap_or(Rect {
        x: bounds.x,
        y: bounds.y,
        width: 0,
        height: bounds.height,
    });
    let shows_expanded_pane = mode == ZsNavigationViewLayoutMode::Expanded || overlay_open;
    let title_x = if overlay_open {
        pane.x.saturating_add(toggle_size).saturating_add(gap)
    } else {
        pane.x.saturating_add(padding)
    };
    let title_bounds = shows_expanded_pane.then_some(Rect {
        x: title_x,
        y: pane.y.saturating_add(padding),
        width: pane
            .x
            .saturating_add(pane.width)
            .saturating_sub(padding)
            .saturating_sub(title_x)
            .max(0),
        height: title_line,
    });
    let subtitle_bounds = shows_expanded_pane.then_some(Rect {
        x: pane.x.saturating_add(padding),
        y: pane
            .y
            .saturating_add(padding)
            .saturating_add(title_line)
            .saturating_add(gap),
        width: pane.width.saturating_sub(padding.saturating_mul(2)).max(0),
        height: subtitle_line,
    });
    let item_top = if shows_expanded_pane {
        pane.y.saturating_add(expanded_header_height)
    } else {
        pane.y.saturating_add(compact_width)
    };
    let item_inset = if shows_expanded_pane { padding } else { 0 };
    let item_bounds = Rect {
        x: pane.x.saturating_add(item_inset),
        y: item_top,
        width: pane.width.saturating_sub(item_inset.saturating_mul(2)).max(0),
        height: pane
            .y
            .saturating_add(pane.height)
            .saturating_sub(item_top)
            .saturating_sub(padding)
            .max(0),
    };
    let footer_bounds = item_bounds;
    let scrim_bounds = overlay_open.then_some(Rect {
        x: bounds.x.saturating_add(open_pane_width),
        y: bounds.y,
        width: bounds.width.saturating_sub(open_pane_width).max(0),
        height: bounds.height,
    });

    ZsNavigationViewLayout {
        mode,
        pane_bounds,
        content_bounds,
        header_bounds,
        toggle_bounds,
        title_bounds,
        subtitle_bounds,
        item_bounds,
        footer_bounds,
        scrim_bounds,
        overlay_open,
    }
}

/// Platform-neutral declaration for a command bar.
///
/// Every declared action remains visible. Overflow/menu projection is a
/// separate capability so the framework never hides an action merely because
/// a particular desktop usually mirrors it in a native application menu.
#[cfg(feature = "button")]
pub struct ZsCommandBarSpec<Msg> {
    leading: Vec<ViewNode<Msg>>,
    trailing: Vec<ViewNode<Msg>>,
    gap: Option<Dp>,
}

#[cfg(feature = "button")]
impl<Msg> ZsCommandBarSpec<Msg> {
    pub fn new() -> Self {
        Self {
            leading: Vec::new(),
            trailing: Vec::new(),
            gap: None,
        }
    }

    pub fn leading(mut self, items: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.leading.extend(items);
        self
    }

    pub fn trailing(mut self, items: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.trailing.extend(items);
        self
    }

    pub fn gap(mut self, gap: Dp) -> Self {
        self.gap = Some(Dp::new(gap.0.max(0.0)));
        self
    }
}

#[cfg(feature = "button")]
impl<Msg> Default for ZsCommandBarSpec<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "button")]
pub fn command_bar<Msg>(spec: ZsCommandBarSpec<Msg>) -> ViewNode<Msg> {
    command_bar_for_style(crate::ZsBaseControlPlatformStyle::current(), spec)
}

/// Builds a command bar using the target desktop's action density.
///
/// This deterministic entry is kept inside the framework for proof fixtures;
/// application code uses [`command_bar`] and never selects a platform.
#[cfg(feature = "button")]
pub(crate) fn command_bar_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    spec: ZsCommandBarSpec<Msg>,
) -> ViewNode<Msg> {
    let ZsCommandBarSpec {
        leading,
        trailing,
        gap,
    } = spec;
    let metrics = ZsToolbarMetrics::for_platform(platform);
    let mut children = leading;
    children.push(spacer());
    children.extend(trailing);
    row(children)
        .native_typography_height(metrics.bar_height)
        .gap(gap.unwrap_or(metrics.item_gap))
        .bg(crate::ThemeColorToken::Surface)
}

#[cfg(feature = "grid")]
/// Creates a two-dimensional Grid using shared DPI-aware layout geometry.
///
/// Every [`ZsGridCell`] carries an explicit zero-based placement, matching the
/// row/column attachment model shared by WinUI Grid, AppKit Grid View and
/// GTK4 Grid. Explicit overlaps retain declaration order for painting and hit
/// testing.
pub fn grid<Msg>(
    columns: impl IntoIterator<Item = ZsGridTrack>,
    rows: impl IntoIterator<Item = ZsGridTrack>,
    items: impl IntoIterator<Item = ZsGridCell<Msg>>,
) -> ViewNode<Msg> {
    let items = items.into_iter().collect::<Vec<_>>();
    let placements = items.iter().map(|item| item.placement).collect();
    ViewNode::<Msg>::new(ViewNodeKind::Grid {
        columns: columns.into_iter().collect(),
        rows: rows.into_iter().collect(),
        placements,
        column_gap: None,
        row_gap: None,
    })
    .children(items.into_iter().map(|item| item.content))
}

#[cfg(feature = "virtual-list")]
pub fn virtual_list<T, Msg>(
    total_count: usize,
    rows: impl IntoIterator<Item = (usize, T)>,
    mut render: impl FnMut(usize, T) -> ViewNode<Msg>,
) -> ViewNode<Msg> {
    let mut rows = rows
        .into_iter()
        .filter(|(index, _)| *index < total_count)
        .collect::<Vec<_>>();
    rows.sort_by_key(|(index, _)| *index);
    rows.dedup_by_key(|(index, _)| *index);
    let mut row_indices = Vec::with_capacity(rows.len());
    let mut children = Vec::with_capacity(rows.len());
    for (index, item) in rows {
        row_indices.push(index);
        children.push(render(index, item));
    }
    ViewNode::<Msg>::new(ViewNodeKind::VirtualList {
        total_count,
        row_height: Dp::new(40.0),
        overscan_rows: 4,
        row_indices,
        selected_index: None,
        offset_y: Dp::new(0.0),
        visible_range: VirtualListRange::new(0, 0),
        materialized_range: VirtualListRange::new(0, 0),
        on_select: None,
        on_viewport_changed: None,
        loading: false,
        show_placeholders: true,
    })
    .children(children)
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

#[cfg(feature = "image-preview")]
pub fn image_preview<Msg>(snapshot: &ZsImagePreviewSnapshot) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::ImagePreview {
        snapshot: snapshot.clone(),
        fit: ZsImageFit::Contain,
        interpolation: NativeImageInterpolation::Smooth,
    })
    .min_width(Dp::new(48.0))
    .min_height(Dp::new(48.0))
}

pub fn spacer<Msg>() -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Spacer)
}

#[cfg(all(test, feature = "label"))]
mod data_tests {
    use super::*;

    fn contains_widget<Msg>(node: &ViewNode<Msg>, widget: crate::WidgetId) -> bool {
        node.id == Some(widget)
            || node
                .children
                .iter()
                .any(|child| contains_widget(child, widget))
    }

    #[test]
    fn gtk_boxed_rows_reserve_their_outer_padding() {
        let child = row([spacer::<()>().height(Dp::new(34.0))]);
        let section = section_for_style(
            crate::ZsBaseControlPlatformStyle::Gtk,
            "Settings",
            [child],
        );
        let rows = &section.children[1];
        let first_row = &rows.children[0];
        assert_eq!(first_row.style.padding, Some(Dp::new(12.0)));
        assert_eq!(first_row.style.min_height, Some(Dp::new(58.0)));
    }

    #[test]
    fn section_intrinsic_height_prevents_non_flex_content_from_collapsing() {
        let height = |platform| {
            section_for_style::<()>(
                platform,
                "Breadcrumb",
                [spacer().height(Dp::new(32.0))],
            )
            .style
            .min_height
            .expect("section must expose its intrinsic minimum height")
        };

        assert_eq!(
            height(crate::ZsBaseControlPlatformStyle::Windows),
            Dp::new(92.0)
        );
        assert_eq!(
            height(crate::ZsBaseControlPlatformStyle::Macos),
            Dp::new(56.0)
        );
        assert_eq!(
            height(crate::ZsBaseControlPlatformStyle::Gtk),
            Dp::new(84.0)
        );
    }

    #[test]
    fn navigation_view_owns_platform_selection_and_keeps_common_overrides() {
        let item = crate::WidgetId::new(501);
        let footer = crate::WidgetId::new(502);
        let build = |platform| {
            navigation_view_for_style(
                platform,
                ZsNavigationViewSpec::<()>::new("Library", "12 items")
                    .item(spacer().id(item))
                    .footer_item(spacer().id(footer))
                    .pane_width(Dp::new(260.0)),
            )
        };

        for platform in [
            crate::ZsBaseControlPlatformStyle::Windows,
            crate::ZsBaseControlPlatformStyle::Macos,
            crate::ZsBaseControlPlatformStyle::Gtk,
        ] {
            let navigation = build(platform);
            assert_eq!(navigation.style.width, Some(Dp::new(260.0)));
            assert!(contains_widget(&navigation, item));
            assert!(contains_widget(&navigation, footer));
        }
    }

    #[test]
    fn navigation_view_content_creates_one_framework_owned_adaptive_shell() {
        let shell = crate::WidgetId::new(503);
        let item = crate::WidgetId::new(504);
        let content = crate::WidgetId::new(505);
        let mut navigation = navigation_view_for_style(
            crate::ZsBaseControlPlatformStyle::Macos,
            ZsNavigationViewSpec::<()>::new("Library", "12 items")
                .item(spacer().id(item))
                .minimum_content_width(Dp::new(420.0))
                .content(shell, spacer().id(content)),
        );

        assert_eq!(navigation.id, Some(shell));
        assert!(contains_widget(&navigation, item));
        assert!(contains_widget(&navigation, content));
        assert!(matches!(
            &navigation.kind,
            ViewNodeKind::NavigationView {
                item_count: 1,
                footer_count: 0,
                minimum_content_width: Dp(420.0),
                ..
            }
        ));
        assert_eq!(
            navigation.resolved_platform_style(),
            crate::ZsBaseControlPlatformStyle::Macos
        );
        navigation.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 640,
            },
            Dpi::standard(),
        ));
        assert_eq!(
            navigation.children[1]
                .bounds()
                .expect("content should receive AppKit split-view bounds")
                .x,
            240
        );
    }

    #[test]
    fn public_navigation_view_keeps_platform_selection_out_of_its_payload() {
        let shell = crate::WidgetId::new(506);
        let navigation = navigation_view(
            ZsNavigationViewSpec::<()>::new("Library", "12 items")
                .content(shell, spacer()),
        );

        assert_eq!(navigation.platform_style_override, None);
        assert!(matches!(
            navigation.kind,
            ViewNodeKind::NavigationView { .. }
        ));
    }

    #[test]
    fn navigation_view_uses_platform_adaptive_width_contracts() {
        let bounds = |width| Rect {
            x: 0,
            y: 0,
            width,
            height: 640,
        };
        let layout = |width, platform, pane_width, minimum_content_width| {
            zs_navigation_view_layout(
                bounds(width),
                platform,
                pane_width,
                minimum_content_width,
                false,
                Dpi::standard(),
                1.0,
            )
        };

        let windows_expanded = layout(
            1008,
            crate::ZsBaseControlPlatformStyle::Windows,
            None,
            Dp::new(560.0),
        );
        assert_eq!(
            windows_expanded.mode,
            ZsNavigationViewLayoutMode::Expanded
        );
        assert_eq!(windows_expanded.pane_bounds.unwrap().width, 320);
        assert_eq!(windows_expanded.content_bounds.x, 320);

        let windows_compact = layout(
            1007,
            crate::ZsBaseControlPlatformStyle::Windows,
            None,
            Dp::new(560.0),
        );
        assert_eq!(
            windows_compact.mode,
            ZsNavigationViewLayoutMode::Compact
        );
        assert_eq!(windows_compact.pane_bounds.unwrap().width, 48);
        assert_eq!(windows_compact.content_bounds.x, 48);

        let windows_minimal = layout(
            640,
            crate::ZsBaseControlPlatformStyle::Windows,
            None,
            Dp::new(560.0),
        );
        assert_eq!(
            windows_minimal.mode,
            ZsNavigationViewLayoutMode::Collapsed
        );
        assert_eq!(windows_minimal.pane_bounds, None);
        assert_eq!(windows_minimal.header_bounds.unwrap().height, 52);
        assert_eq!(windows_minimal.content_bounds.y, 52);

        let macos_collapsed = layout(
            799,
            crate::ZsBaseControlPlatformStyle::Macos,
            None,
            Dp::new(560.0),
        );
        assert_eq!(
            macos_collapsed.mode,
            ZsNavigationViewLayoutMode::Collapsed
        );
        let macos_expanded = layout(
            800,
            crate::ZsBaseControlPlatformStyle::Macos,
            None,
            Dp::new(560.0),
        );
        assert_eq!(macos_expanded.mode, ZsNavigationViewLayoutMode::Expanded);
        assert_eq!(macos_expanded.pane_bounds.unwrap().width, 240);

        let gtk_collapsed = layout(
            400,
            crate::ZsBaseControlPlatformStyle::Gtk,
            None,
            Dp::new(0.0),
        );
        assert_eq!(gtk_collapsed.mode, ZsNavigationViewLayoutMode::Collapsed);
        let gtk_narrow_expanded = layout(
            401,
            crate::ZsBaseControlPlatformStyle::Gtk,
            None,
            Dp::new(0.0),
        );
        assert_eq!(
            gtk_narrow_expanded.mode,
            ZsNavigationViewLayoutMode::Expanded
        );
        assert_eq!(gtk_narrow_expanded.pane_bounds.unwrap().width, 180);
        let gtk_wide = layout(
            1200,
            crate::ZsBaseControlPlatformStyle::Gtk,
            None,
            Dp::new(0.0),
        );
        assert_eq!(gtk_wide.pane_bounds.unwrap().width, 280);
    }

    #[test]
    fn navigation_view_overlay_keeps_text_line_boxes_inside_the_pane() {
        let scale = 1.5;
        let layout = zs_navigation_view_layout(
            Rect {
                x: 0,
                y: 0,
                width: 620,
                height: 480,
            },
            crate::ZsBaseControlPlatformStyle::Macos,
            None,
            Dp::new(560.0),
            true,
            Dpi::standard(),
            scale,
        );
        let pane = layout.pane_bounds.expect("open overlay must own a pane");
        let title = layout.title_bounds.expect("open overlay must show title");
        let subtitle = layout
            .subtitle_bounds
            .expect("open overlay must show subtitle");

        assert!(layout.overlay_open);
        assert!(title.width > 0 && title.height > 0);
        assert!(subtitle.width > 0 && subtitle.height > 0);
        assert!(
            title.x >= pane.x
                && title.x.saturating_add(title.width) <= pane.x.saturating_add(pane.width)
        );
        assert!(
            subtitle.x >= pane.x
                && subtitle.x.saturating_add(subtitle.width) <= pane.x.saturating_add(pane.width)
        );
        assert!(title.y.saturating_add(title.height) <= subtitle.y);
        assert!(subtitle.y.saturating_add(subtitle.height) <= layout.item_bounds.y);
    }

    #[test]
    #[cfg(feature = "button")]
    fn command_bar_keeps_one_semantic_spec_and_platform_owned_density() {
        let build = |platform| {
            command_bar_for_style(
                platform,
                ZsCommandBarSpec::<()>::new()
                    .leading([spacer()])
                    .trailing([spacer()])
                    .gap(Dp::new(9.0)),
            )
        };

        let windows = build(crate::ZsBaseControlPlatformStyle::Windows);
        let macos = build(crate::ZsBaseControlPlatformStyle::Macos);
        let gtk = build(crate::ZsBaseControlPlatformStyle::Gtk);

        assert_eq!(windows.children.len(), 3);
        assert_eq!(macos.children.len(), 3);
        assert_eq!(gtk.children.len(), 3);
        assert_eq!(windows.style.gap, Some(Dp::new(9.0)));
        assert_eq!(macos.style.gap, Some(Dp::new(9.0)));
        assert_eq!(gtk.style.gap, Some(Dp::new(9.0)));
        assert_eq!(windows.style.height, Some(Dp::new(48.0)));
        assert_eq!(macos.style.height, Some(Dp::new(28.0)));
        assert_eq!(gtk.style.height, Some(Dp::new(34.0)));
    }
}
