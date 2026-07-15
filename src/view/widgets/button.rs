#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Text { text: text.into() })
}

#[cfg(feature = "button")]
pub fn button<Msg>(label: impl Into<String>) -> ViewNode<Msg> {
    let button = ViewNode::new(ViewNodeKind::Button {
        label: label.into(),
        on_click: None,
    });
    #[cfg(windows)]
    {
        button
            .min_width(Dp::new(crate::style::ZSUI_WINUI_BUTTON_MIN_WIDTH as f32))
            .min_height(Dp::new(
                crate::style::ZSUI_FLUENT_STANDARD_CONTROL_HEIGHT as f32,
            ))
            .flex(0.0)
    }
    #[cfg(not(windows))]
    {
        button
    }
}

#[cfg(feature = "toggle-button")]
pub fn toggle_button<Msg>(label: impl Into<String>, checked: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::ToggleButton {
        label: label.into(),
        checked,
        on_toggle: None,
    })
}

#[cfg(feature = "checkbox")]
pub fn checkbox<Msg>(label: impl Into<String>, checked: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Checkbox {
        label: label.into(),
        checked,
        on_toggle: None,
    })
}

#[cfg(feature = "toggle")]
pub fn toggle<Msg>(checked: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Toggle {
        checked,
        on_toggle: None,
    })
}

#[cfg(feature = "radio")]
pub fn radio_button<Msg>(label: impl Into<String>, selected: bool) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::RadioButton {
        label: label.into(),
        selected,
        on_choose: None,
    })
}

#[cfg(feature = "progress")]
pub fn progress_bar<Msg>(value: f32, range: impl Into<crate::ProgressRange>) -> ViewNode<Msg> {
    let range = range.into();
    ViewNode::new(ViewNodeKind::ProgressBar {
        value: range.clamp(value),
        range,
    })
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
