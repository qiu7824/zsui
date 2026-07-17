
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
    title: String,
    subtitle: String,
    items: Vec<ViewNode<Msg>>,
    footer_items: Vec<ViewNode<Msg>>,
    pane_width: Option<Dp>,
}

#[cfg(feature = "label")]
impl<Msg> ZsNavigationViewSpec<Msg> {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            items: Vec::new(),
            footer_items: Vec::new(),
            pane_width: None,
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
}

/// Builds the navigation surface for the target desktop family.
///
/// Navigation is a platform contract, not a colored `Column`: Windows uses a
/// Fluent NavigationView pane, macOS uses an unboxed source list, and GTK
/// uses a grouped Adwaita sidebar list. The caller supplies semantic rows;
/// the framework owns their information architecture and chrome.
#[cfg(feature = "label")]
pub fn navigation_view<Msg>(spec: ZsNavigationViewSpec<Msg>) -> ViewNode<Msg> {
    navigation_view_for_style(
        crate::ZsBaseControlPlatformStyle::current(),
        spec,
    )
}

/// Deterministic navigation composition used by platform proof fixtures.
#[cfg(feature = "label")]
pub(crate) fn navigation_view_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    spec: ZsNavigationViewSpec<Msg>,
) -> ViewNode<Msg> {
    let ZsNavigationViewSpec {
        title,
        subtitle,
        items,
        footer_items,
        pane_width,
    } = spec;
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
