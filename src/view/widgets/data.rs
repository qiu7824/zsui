
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
pub fn platform_section<Msg>(
    title: impl Into<String>,
    children: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    platform_section_for_style(
        crate::ZsBaseControlPlatformStyle::current(),
        title,
        children,
    )
}

/// Deterministic variant used by proof fixtures and framework tests that need
/// to inspect more than the host platform.
#[cfg(feature = "label")]
pub fn platform_section_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    title: impl Into<String>,
    children: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    let heading = styled_text(
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
    match platform {
        crate::ZsBaseControlPlatformStyle::Windows => column([
            heading,
            column(children)
                .padding(Dp::new(16.0))
                .gap(Dp::new(10.0))
                .radius(Dp::new(8.0))
                .bg(crate::ThemeColorToken::SurfaceRaised),
        ])
        .gap(Dp::new(8.0)),
        crate::ZsBaseControlPlatformStyle::Macos => {
            column([heading, column(children).gap(Dp::new(8.0))]).gap(Dp::new(8.0))
        }
        crate::ZsBaseControlPlatformStyle::Gtk => {
            // GNOME's boxed-list pattern uses padded rows separated by thin
            // dividers. Padding is part of the row's outer geometry; keeping
            // it out of the minimum height makes the content box collapse
            // and clips controls (especially switches and progress rings).
            const ROW_PADDING: f32 = 12.0;
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
                    .unwrap_or(Dp::new(crate::TextRole::Body.line_height()));
                rows.push(
                    child
                        .padding(Dp::new(ROW_PADDING))
                        .min_height(Dp::new(intrinsic_height.0 + ROW_PADDING * 2.0)),
                );
            }
            column([
                heading,
                column(rows)
                    .gap(Dp::new(0.0))
                    .radius(Dp::new(12.0))
                    .bg(crate::ThemeColorToken::SurfaceRaised),
            ])
            .gap(Dp::new(8.0))
        }
    }
}

/// Builds the navigation surface for the target desktop family.
///
/// Navigation is a platform contract, not a colored `Column`: Windows uses a
/// Fluent NavigationView pane, macOS uses an unboxed source list, and GTK
/// uses a grouped Adwaita sidebar list. The caller supplies semantic rows;
/// the framework owns their information architecture and chrome.
#[cfg(feature = "label")]
pub fn platform_navigation<Msg>(
    title: impl Into<String>,
    subtitle: impl Into<String>,
    items: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    platform_navigation_for_style(
        crate::ZsBaseControlPlatformStyle::current(),
        title,
        subtitle,
        items,
    )
}

/// Deterministic navigation composition used by platform proof fixtures.
#[cfg(feature = "label")]
pub fn platform_navigation_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    title: impl Into<String>,
    subtitle: impl Into<String>,
    items: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
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
    let heading = styled_text(title, title_style);
    let subtitle = styled_text(
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
    let (open_pane_width, horizontal_inset) = match platform {
        crate::ZsBaseControlPlatformStyle::Windows => (Dp::new(320.0), 32.0),
        crate::ZsBaseControlPlatformStyle::Macos => (Dp::new(240.0), 24.0),
        crate::ZsBaseControlPlatformStyle::Gtk => (Dp::new(280.0), 32.0),
    };
    let item_width = Dp::new(open_pane_width.0 - horizontal_inset);
    let items = items
        .into_iter()
        .map(|item| item.width(item_width))
        .collect::<Vec<_>>();
    let navigation = match platform {
        crate::ZsBaseControlPlatformStyle::Windows => column([
            heading,
            subtitle,
            column(items).gap(Dp::new(4.0)),
        ])
        .padding(Dp::new(16.0))
        .gap(Dp::new(8.0))
        .bg(crate::ThemeColorToken::SurfaceRaised),
        crate::ZsBaseControlPlatformStyle::Macos => column([
            heading,
            subtitle,
            column(items).gap(Dp::new(2.0)),
        ])
        .padding(Dp::new(12.0))
        .gap(Dp::new(6.0))
        .bg(crate::ThemeColorToken::SurfaceRaised),
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
            column([
                heading,
                subtitle,
                column(rows)
                    .gap(Dp::new(0.0))
                    .padding(Dp::new(4.0))
                    .radius(Dp::new(12.0))
                    .bg(crate::ThemeColorToken::SurfaceRaised),
            ])
            .padding(Dp::new(12.0))
            .gap(Dp::new(8.0))
            .bg(crate::ThemeColorToken::Surface)
        }
    };
    navigation.width(open_pane_width)
}

/// Builds a document command bar using the target desktop's action density.
///
/// `leading` and `trailing` are the small cross-platform action set.
/// `expanded_leading` and `expanded_trailing` are compacted into icon actions
/// on Windows and remain in the target-native menu on AppKit and GTK. The
/// application supplies typed buttons; the framework owns grouping, spacing,
/// compact presentation and platform visibility.
#[cfg(feature = "button")]
pub fn platform_document_command_bar_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    leading: impl IntoIterator<Item = ViewNode<Msg>>,
    trailing: impl IntoIterator<Item = ViewNode<Msg>>,
    expanded_leading: impl IntoIterator<Item = ViewNode<Msg>>,
    expanded_trailing: impl IntoIterator<Item = ViewNode<Msg>>,
) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(platform);
    let mut children = leading.into_iter().collect::<Vec<_>>();
    if platform == crate::ZsBaseControlPlatformStyle::Windows {
        children.extend(
            expanded_leading
                .into_iter()
                .map(|item| compact_toolbar_item(item, metrics.button_height)),
        );
    }
    children.push(spacer());
    children.extend(trailing);
    if platform == crate::ZsBaseControlPlatformStyle::Windows {
        children.extend(
            expanded_trailing
                .into_iter()
                .map(|item| compact_toolbar_item(item, metrics.button_height)),
        );
    }
    row(children)
        .height(metrics.button_height)
        .gap(Dp::new(match platform {
            crate::ZsBaseControlPlatformStyle::Windows => 8.0,
            crate::ZsBaseControlPlatformStyle::Macos
            | crate::ZsBaseControlPlatformStyle::Gtk => 6.0,
        }))
        .bg(crate::ThemeColorToken::Surface)
}

#[cfg(feature = "button")]
fn compact_toolbar_item<Msg>(mut item: ViewNode<Msg>, square: Dp) -> ViewNode<Msg> {
    if let ViewNodeKind::Button {
        presentation:
            crate::ZsButtonPresentation::Toolbar {
                show_label,
                ..
            },
        ..
    } = &mut item.kind
    {
        *show_label = false;
        item.style.width = Some(square);
        item.style.min_width = Some(square);
    }
    item
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

    #[test]
    fn gtk_boxed_rows_reserve_their_outer_padding() {
        let child = row([spacer::<()>().height(Dp::new(34.0))]);
        let section = platform_section_for_style(
            crate::ZsBaseControlPlatformStyle::Gtk,
            "Settings",
            [child],
        );
        let rows = &section.children[1];
        let first_row = &rows.children[0];
        assert_eq!(first_row.style.padding, Some(Dp::new(12.0)));
        assert_eq!(first_row.style.min_height, Some(Dp::new(58.0)));
    }
}
