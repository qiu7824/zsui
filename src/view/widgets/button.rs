#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    styled_text(text, crate::SemanticTextStyle::body())
}

/// Creates a label using a semantic type-ramp role instead of a raw font size.
/// The line box follows the active desktop's native typography metric.
#[cfg(feature = "label")]
pub fn styled_text<Msg>(
    text: impl Into<String>,
    style: crate::SemanticTextStyle,
) -> ViewNode<Msg> {
    styled_text_for_platform(
        crate::ZsTypographyPlatformStyle::current(),
        text,
        style,
    )
}

/// Deterministic semantic-label variant for framework platform compositions.
#[cfg(feature = "label")]
pub(crate) fn styled_text_for_platform<Msg>(
    platform: crate::ZsTypographyPlatformStyle,
    text: impl Into<String>,
    style: crate::SemanticTextStyle,
) -> ViewNode<Msg> {
    let line_height = style.role.metrics_for(platform).line_height;
    let text = text.into();
    let explicit_line_count = text.lines().count().max(1) as f32;
    let node = ViewNode::new(ViewNodeKind::Text {
        text,
        style,
    });
    if style.wrap == crate::TextWrap::Word {
        // A wrapping label must not be frozen to a one-line box. The native
        // backend owns final shaping and wrapping; the shared tree only
        // reserves the explicit lines and lets the label consume available
        // vertical space.
        node.native_typography_min_height(crate::Dp::new(line_height * explicit_line_count))
    } else {
        node.native_typography_height(crate::Dp::new(line_height))
    }
}

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ZsToolbarMetrics {
    pub bar_height: Dp,
    pub button_height: Dp,
    pub icon_size: Dp,
    pub content_gap: Dp,
    pub item_gap: Dp,
    pub label_role: crate::TextRole,
}

#[cfg(feature = "button")]
impl ZsToolbarMetrics {
    pub(crate) const fn for_platform(platform: crate::ZsBaseControlPlatformStyle) -> Self {
        match platform {
            crate::ZsBaseControlPlatformStyle::Windows => Self {
                // WinUI CommandBar with labels on the right keeps the closed
                // compact 48 epx height, 20 epx primary icon, 8 epx label gap
                // and the AppBarButton 12 epx label style.
                bar_height: Dp::new(48.0),
                button_height: Dp::new(48.0),
                icon_size: Dp::new(20.0),
                content_gap: Dp::new(8.0),
                item_gap: Dp::new(8.0),
                label_role: crate::TextRole::Caption,
            },
            crate::ZsBaseControlPlatformStyle::Macos => Self {
                bar_height: Dp::new(28.0),
                button_height: Dp::new(28.0),
                icon_size: Dp::new(16.0),
                content_gap: Dp::new(6.0),
                item_gap: Dp::new(6.0),
                label_role: crate::TextRole::Button,
            },
            crate::ZsBaseControlPlatformStyle::Gtk => Self {
                // Libadwaita's toolbar class specifies 6 px spacing and
                // margins; control height and font remain GTK semantic
                // fallbacks until the backend resolves the active theme.
                bar_height: Dp::new(34.0),
                button_height: Dp::new(34.0),
                icon_size: Dp::new(16.0),
                content_gap: Dp::new(6.0),
                item_gap: Dp::new(6.0),
                label_role: crate::TextRole::Button,
            },
        }
    }
}

#[cfg(feature = "button")]
pub fn button<Msg>(label: impl Into<String>) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    let label = label.into();
    let minimum_width = metrics.button_minimum_width_for_label(&label);
    ViewNode::new(ViewNodeKind::Button {
        label,
        presentation: ZsButtonPresentation::Standard,
        on_click: None,
    })
    .min_width(minimum_width)
    .native_typography_height(metrics.button_height)
    .flex(0.0)
}

/// Creates an icon-and-label action for a platform toolbar or header bar.
///
/// The framework keeps this presentation flat at rest and maps the semantic
/// icon through WinUI, SF Symbols or the GTK symbolic icon theme. Use it in a
/// [`ZsCommandBarSpec`](crate::ZsCommandBarSpec) so the framework owns platform
/// action density and grouping.
#[cfg(feature = "button")]
pub fn toolbar_button<Msg>(label: impl Into<String>, icon: crate::ZsIcon) -> ViewNode<Msg> {
    toolbar_button_impl(crate::ZsBaseControlPlatformStyle::current(), label, icon)
}

/// Deterministic toolbar-button variant for target proof fixtures and tests.
#[cfg(all(test, feature = "button"))]
pub(crate) fn toolbar_button_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    label: impl Into<String>,
    icon: crate::ZsIcon,
) -> ViewNode<Msg> {
    toolbar_button_impl(platform, label, icon).with_platform_style_override(platform)
}

#[cfg(feature = "button")]
fn toolbar_button_impl<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    label: impl Into<String>,
    icon: crate::ZsIcon,
) -> ViewNode<Msg> {
    let base = crate::ZsBaseControlMetrics::for_platform(platform);
    let metrics = ZsToolbarMetrics::for_platform(platform);
    let label = label.into();
    let minimum_width = Dp::new(
        base
            .estimated_text_width_with_shaping_reserve(&label)
            .0
            + base.button_padding_left.0
            + metrics.icon_size.0
            + metrics.content_gap.0
            + base.button_padding_right.0,
    );
    ViewNode::new(ViewNodeKind::Button {
        label,
        presentation: ZsButtonPresentation::Toolbar {
            icon,
            show_label: true,
        },
        on_click: None,
    })
    .min_width(minimum_width)
    .native_typography_height(metrics.button_height)
    .flex(0.0)
}

/// Creates a self-drawn navigation row with a semantic icon and explicit
/// selected state. It uses the same typed activation path as a Button while
/// retaining NavigationView item geometry instead of Button chrome.
#[cfg(feature = "button")]
pub fn navigation_item<Msg>(
    label: impl Into<String>,
    icon: crate::ZsIcon,
    selected: bool,
) -> ViewNode<Msg> {
    let metrics = crate::ZsNavigationItemMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::Button {
        label: label.into(),
        presentation: ZsButtonPresentation::NavigationItem { icon, selected },
        on_click: None,
    })
    .native_typography_height(metrics.item_height)
    .flex(0.0)
}

#[cfg(feature = "toggle-button")]
pub fn toggle_button<Msg>(label: impl Into<String>, checked: bool) -> ViewNode<Msg> {
    let metrics = crate::ZsToggleButtonMetrics::for_platform(
        crate::ZsToggleButtonPlatformStyle::current(),
    );
    let base = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    let label = label.into();
    let minimum_width = base.button_minimum_width_for_label(&label);
    ViewNode::new(ViewNodeKind::ToggleButton {
        label,
        checked,
        on_toggle: None,
    })
    .min_width(minimum_width)
    .native_typography_height(metrics.minimum_height)
    .flex(0.0)
}

#[cfg(feature = "checkbox")]
pub fn checkbox<Msg>(label: impl Into<String>, checked: bool) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    let label = label.into();
    let minimum_width = metrics.check_minimum_width_for_label(&label);
    ViewNode::new(ViewNodeKind::Checkbox {
        label,
        checked,
        on_toggle: None,
    })
    .min_width(minimum_width)
    .native_typography_height(metrics.check_height)
    .flex(0.0)
}

#[cfg(feature = "toggle")]
pub fn toggle<Msg>(checked: bool) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::Toggle {
        checked,
        on_toggle: None,
    })
    .width(metrics.toggle_width)
    .height(metrics.toggle_height)
    .flex(0.0)
}

#[cfg(feature = "radio")]
pub fn radio_button<Msg>(label: impl Into<String>, selected: bool) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    let label = label.into();
    let minimum_width = metrics.radio_minimum_width_for_label(&label);
    ViewNode::new(ViewNodeKind::RadioButton {
        label,
        selected,
        on_choose: None,
    })
    .min_width(minimum_width)
    .native_typography_height(metrics.radio_height)
    .flex(0.0)
}

#[cfg(feature = "progress")]
pub fn progress_bar<Msg>(value: f32, range: impl Into<crate::ProgressRange>) -> ViewNode<Msg> {
    let range = range.into();
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::ProgressBar {
        value: range.clamp(value),
        range,
    })
    .height(metrics.progress_slot_height)
}

#[cfg(feature = "progress-ring")]
pub fn progress_ring<Msg>(spec: crate::ZsProgressRingSpec) -> ViewNode<Msg> {
    let metrics = crate::zs_progress_ring_metrics(
        crate::ZsProgressRingPlatformStyle::current(),
        spec.size_value(),
    );
    ViewNode::new(ViewNodeKind::ProgressRing { spec })
        .width(metrics.diameter)
        .height(metrics.diameter)
}
