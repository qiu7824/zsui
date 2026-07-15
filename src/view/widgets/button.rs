#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::Text { text: text.into() }).height(metrics.body_line_height)
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
