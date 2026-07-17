#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    styled_text(text, crate::SemanticTextStyle::body())
}

/// Creates a label using a semantic type-ramp role instead of a raw font size.
/// The line box follows the role's platform-independent typography metric.
#[cfg(feature = "label")]
pub fn styled_text<Msg>(
    text: impl Into<String>,
    style: crate::SemanticTextStyle,
) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Text {
        text: text.into(),
        style,
    })
    .height(crate::Dp::new(style.role.line_height()))
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
    .height(metrics.button_height)
    .flex(0.0)
}

/// Creates an icon-and-label action for a platform toolbar or header bar.
///
/// The framework keeps this presentation flat at rest and maps the semantic
/// icon through WinUI, SF Symbols or the GTK symbolic icon theme. Use
/// [`platform_document_command_bar_for_style`] to let the framework own
/// platform action density and grouping.
#[cfg(feature = "button")]
pub fn toolbar_button<Msg>(label: impl Into<String>, icon: crate::ZsIcon) -> ViewNode<Msg> {
    toolbar_button_for_style(
        crate::ZsBaseControlPlatformStyle::current(),
        label,
        icon,
    )
}

/// Deterministic toolbar-button variant for target proof fixtures and tests.
#[cfg(feature = "button")]
pub fn toolbar_button_for_style<Msg>(
    platform: crate::ZsBaseControlPlatformStyle,
    label: impl Into<String>,
    icon: crate::ZsIcon,
) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(platform);
    let label = label.into();
    let icon_size = Dp::new(match platform {
        // WinUI AppBarButton primary-command icons are 20×20 epx.
        crate::ZsBaseControlPlatformStyle::Windows => 20.0,
        crate::ZsBaseControlPlatformStyle::Macos
        | crate::ZsBaseControlPlatformStyle::Gtk => 16.0,
    });
    let content_gap = Dp::new(match platform {
        crate::ZsBaseControlPlatformStyle::Windows => 8.0,
        crate::ZsBaseControlPlatformStyle::Macos
        | crate::ZsBaseControlPlatformStyle::Gtk => 6.0,
    });
    let minimum_width = Dp::new(
        metrics
            .estimated_text_width_with_shaping_reserve(&label)
            .0
            + metrics.button_padding_left.0
            + icon_size.0
            + content_gap.0
            + metrics.button_padding_right.0,
    );
    ViewNode::new(ViewNodeKind::Button {
        label,
        presentation: ZsButtonPresentation::Toolbar {
            icon,
            show_label: true,
            platform,
        },
        on_click: None,
    })
    .min_width(minimum_width)
    .height(metrics.button_height)
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
    .height(metrics.item_height)
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
    .height(metrics.minimum_height)
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
    .height(metrics.check_height)
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
    .height(metrics.radio_height)
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
