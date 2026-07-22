#[cfg(feature = "textbox")]
pub fn textbox<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::Textbox {
        value: value.into(),
        multiline: false,
        wrap: crate::TextWrap::NoWrap,
        on_change: None,
        on_selection_change: None,
    })
    .min_width(metrics.text_input_minimum_width)
    .native_typography_height(metrics.text_input_height)
}

#[cfg(feature = "textbox")]
pub fn text_editor<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Textbox {
        value: value.into(),
        multiline: true,
        wrap: crate::TextWrap::Word,
        on_change: None,
        on_selection_change: None,
    })
}

#[cfg(feature = "password-box")]
pub fn password_box<Msg>(value: impl Into<crate::ZsPassword>) -> ViewNode<Msg> {
    let metrics = crate::ZsPasswordBoxMetrics::for_platform(
        crate::ZsPasswordBoxPlatformStyle::current(),
    );
    let base = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::PasswordBox {
        value: value.into(),
        reveal_mode: crate::ZsPasswordRevealMode::platform_default(),
        on_change: None,
    })
    .min_width(base.text_input_minimum_width)
    .native_typography_height(metrics.minimum_height)
}

#[cfg(feature = "slider")]
pub fn slider<Msg>(value: f32, range: impl Into<SliderRange>) -> ViewNode<Msg> {
    let range = range.into();
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::Slider {
        value: range.snap(value),
        range,
        on_slide: None,
    })
    .min_width(metrics.slider_minimum_width)
    .height(metrics.slider_height)
}

#[cfg(feature = "number-box")]
pub fn number_box<Msg>(
    value: impl Into<Option<f64>>,
    range: impl Into<ZsNumberRange>,
) -> ViewNode<Msg> {
    let range = range.into();
    let value = value
        .into()
        .filter(|value| value.is_finite())
        .map(|value| range.clamp(value));
    let format = ZsNumberFormat::default();
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::NumberBox {
        value,
        draft: format.format(value),
        range,
        format,
        wraps: false,
        on_change: None,
    })
    .min_width(metrics.button_minimum_width)
    .native_typography_height(metrics.text_input_height)
}

#[cfg(feature = "auto-suggest")]
pub fn auto_suggest_box<T, Msg>(
    query: impl Into<String>,
    suggestions: impl IntoIterator<Item = T>,
) -> ViewNode<Msg>
where
    T: Into<crate::ZsAutoSuggestion>,
{
    let metrics =
        crate::ZsAutoSuggestMetrics::for_platform(crate::ZsAutoSuggestPlatformStyle::current());
    ViewNode::new(ViewNodeKind::AutoSuggestBox {
        query: query.into(),
        suggestions: suggestions.into_iter().map(Into::into).collect(),
        highlighted: None,
        expanded: false,
        placeholder: None,
        no_results_text: None,
        query_icon: true,
        on_text_change: None,
        on_suggestion_chosen: None,
        on_query_submit: None,
        on_expanded_change: None,
    })
    .native_typography_height(metrics.control_height)
}

#[cfg(feature = "date-picker")]
pub fn date_picker<Msg>(value: ZsDate) -> ViewNode<Msg> {
    let minimum = ZsDate::new(ZsDate::MIN_YEAR, 1, 1).expect("minimum date is valid");
    let maximum = ZsDate::new(ZsDate::MAX_YEAR, 12, 31).expect("maximum date is valid");
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::DatePicker {
        value,
        minimum,
        maximum,
        visible_month: value.first_day_of_month(),
        today: ZsDate::today_local().ok(),
        expanded: false,
        on_date_change: None,
        on_month_change: None,
        on_expanded_change: None,
    })
    .min_width(metrics.selection_minimum_width)
    .native_typography_height(metrics.selection_height)
}

#[cfg(feature = "time-picker")]
pub fn time_picker<Msg>(value: ZsTime) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::TimePicker {
        value,
        minute_increment: ZsMinuteIncrement::ONE,
        clock: ZsTimePickerPlatformStyle::current().default_clock(),
        expanded: false,
        on_time_change: None,
        on_expanded_change: None,
    })
    .min_width(metrics.time_picker_minimum_width)
    .native_typography_height(metrics.selection_height)
}

#[cfg(feature = "color-picker")]
pub fn color_picker<Msg>(state: ZsColorPickerState) -> ViewNode<Msg> {
    let metrics = crate::ZsBaseControlMetrics::for_platform(
        crate::ZsBaseControlPlatformStyle::current(),
    );
    ViewNode::new(ViewNodeKind::ColorPicker {
        state: state.normalized(),
        on_color_change: None,
        on_expanded_change: None,
        on_channel_change: None,
    })
    .min_width(metrics.selection_minimum_width)
    .native_typography_height(metrics.selection_height)
}
