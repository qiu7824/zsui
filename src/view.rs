#[cfg(any(feature = "slider", feature = "progress"))]
use std::ops::RangeInclusive;
use std::{
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

#[cfg(feature = "button")]
use crate::render_protocol::TextRole;
#[cfg(any(
    feature = "label",
    feature = "button",
    feature = "textbox",
    feature = "checkbox",
    feature = "radio"
))]
use crate::render_protocol::{NativeDrawTextCommand, SemanticTextStyle};
#[cfg(feature = "date-picker")]
use crate::ZsDate;
use crate::{
    geometry::{ComponentId, Dp, Dpi, LayoutNode, LayoutOutput, Point, Rect},
    render_protocol::{ColorRole, NativeDrawCommand, NativeDrawFill, NativeDrawPlan},
    style::{ThemeColorToken, ZsuiThemeMode},
    Command, UiCommand,
};
#[cfg(feature = "time-picker")]
use crate::{ZsClockFormat, ZsMinuteIncrement, ZsTime, ZsTimePickerPlatformStyle};
#[cfg(feature = "tabs")]
use crate::{ZsTabId, ZsTabSpec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl From<WidgetId> for ComponentId {
    fn from(value: WidgetId) -> Self {
        Self(value.0)
    }
}

impl From<ComponentId> for WidgetId {
    fn from(value: ComponentId) -> Self {
        Self(value.0)
    }
}

#[cfg(feature = "slider")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SliderRange {
    min: f32,
    max: f32,
    step: f32,
}

#[cfg(feature = "slider")]
impl SliderRange {
    pub fn new(min: f32, max: f32) -> Self {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 100.0 };
        let (min, mut max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f32::EPSILON {
            max = min + 1.0;
        }
        let step = ((max - min) / 100.0).max(f32::EPSILON);
        Self { min, max, step }
    }

    pub fn step(mut self, step: f32) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step.min(self.max - self.min);
        }
        self
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub const fn step_size(self) -> f32 {
        self.step
    }

    pub fn clamp(self, value: f32) -> f32 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    pub fn snap(self, value: f32) -> f32 {
        let value = self.clamp(value);
        if value <= self.min {
            return self.min;
        }
        if value >= self.max {
            return self.max;
        }
        let steps = ((value - self.min) / self.step).round();
        self.clamp(self.min + steps * self.step)
    }

    pub fn fraction(self, value: f32) -> f32 {
        ((self.clamp(value) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    pub fn value_at_fraction(self, fraction: f32) -> f32 {
        let fraction = if fraction.is_finite() {
            fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.snap(self.min + (self.max - self.min) * fraction)
    }

    pub fn offset_steps(self, value: f32, steps: i32) -> f32 {
        self.snap(self.clamp(value) + self.step * steps as f32)
    }
}

#[cfg(feature = "slider")]
impl From<RangeInclusive<f32>> for SliderRange {
    fn from(range: RangeInclusive<f32>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}

#[cfg(feature = "progress")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProgressRange {
    min: f32,
    max: f32,
}

#[cfg(feature = "progress")]
impl ProgressRange {
    pub fn new(min: f32, max: f32) -> Self {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 100.0 };
        let (min, mut max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f32::EPSILON {
            max = min + 1.0;
        }
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub fn clamp(self, value: f32) -> f32 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    pub fn fraction(self, value: f32) -> f32 {
        ((self.clamp(value) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

#[cfg(feature = "progress")]
impl From<RangeInclusive<f32>> for ProgressRange {
    fn from(range: RangeInclusive<f32>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}

#[cfg(feature = "virtual-list")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualListRange {
    pub start: usize,
    pub end: usize,
}

#[cfg(feature = "virtual-list")]
impl VirtualListRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: if end < start { start } else { end },
        }
    }

    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub const fn is_empty(self) -> bool {
        self.start >= self.end
    }

    pub const fn contains(self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    pub const fn clamp(self, total_count: usize) -> Self {
        let start = if self.start > total_count {
            total_count
        } else {
            self.start
        };
        let end = if self.end > total_count {
            total_count
        } else {
            self.end
        };
        Self::new(start, end)
    }
}

#[cfg(feature = "virtual-list")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualListScrollDirection {
    Backward,
    Stationary,
    Forward,
}

#[cfg(feature = "virtual-list")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VirtualListViewport {
    pub offset_y: Dp,
    pub row_height: Dp,
    pub visible_range: VirtualListRange,
    pub materialized_range: VirtualListRange,
    pub direction: VirtualListScrollDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewStackDirection {
    Row,
    Column,
}

#[cfg(feature = "grid")]
/// A validated positive weight for a fractional Grid track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZsGridFraction(u16);

#[cfg(feature = "grid")]
impl ZsGridFraction {
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);
    pub const THREE: Self = Self(3);
    pub const FOUR: Self = Self(4);

    pub fn new(weight: u16) -> crate::ZsuiResult<Self> {
        if weight == 0 {
            return Err(crate::ZsuiError::invalid_spec(
                "grid.fraction",
                "fractional track weight must be greater than zero",
            ));
        }
        Ok(Self(weight))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[cfg(feature = "grid")]
/// A validated nonzero number of Grid tracks covered by a child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZsGridSpan(u16);

#[cfg(feature = "grid")]
impl ZsGridSpan {
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);
    pub const THREE: Self = Self(3);
    pub const FOUR: Self = Self(4);

    pub fn new(track_count: u16) -> crate::ZsuiResult<Self> {
        if track_count == 0 {
            return Err(crate::ZsuiError::invalid_spec(
                "grid.span",
                "grid span must cover at least one track",
            ));
        }
        Ok(Self(track_count))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[cfg(feature = "grid")]
/// The zero-based row/column position and typed span of a Grid child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZsGridPlacement {
    pub row: usize,
    pub column: usize,
    pub row_span: ZsGridSpan,
    pub column_span: ZsGridSpan,
}

#[cfg(feature = "grid")]
impl ZsGridPlacement {
    pub const fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            row_span: ZsGridSpan::ONE,
            column_span: ZsGridSpan::ONE,
        }
    }

    pub const fn with_spans(mut self, row_span: ZsGridSpan, column_span: ZsGridSpan) -> Self {
        self.row_span = row_span;
        self.column_span = column_span;
        self
    }

    pub const fn with_row_span(mut self, row_span: ZsGridSpan) -> Self {
        self.row_span = row_span;
        self
    }

    pub const fn with_column_span(mut self, column_span: ZsGridSpan) -> Self {
        self.column_span = column_span;
        self
    }
}

#[cfg(feature = "grid")]
/// One fixed-DP or weighted fractional Grid track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZsGridTrack {
    Fixed(Dp),
    Fraction(ZsGridFraction),
}

#[cfg(feature = "grid")]
impl ZsGridTrack {
    pub const FLEX: Self = Self::Fraction(ZsGridFraction::ONE);

    pub const fn fixed(size: Dp) -> Self {
        Self::Fixed(size)
    }

    pub const fn fraction(weight: ZsGridFraction) -> Self {
        Self::Fraction(weight)
    }
}

#[cfg(feature = "grid")]
impl Default for ZsGridTrack {
    fn default() -> Self {
        Self::FLEX
    }
}

#[cfg(feature = "grid")]
/// One explicitly placed child cell in a [`grid`] declaration.
#[derive(Debug, Clone)]
pub struct ZsGridCell<Msg> {
    pub placement: ZsGridPlacement,
    pub content: ViewNode<Msg>,
}

#[cfg(feature = "grid")]
impl<Msg> ZsGridCell<Msg> {
    pub fn new(row: usize, column: usize, content: ViewNode<Msg>) -> Self {
        Self {
            placement: ZsGridPlacement::new(row, column),
            content,
        }
    }

    pub fn placement(mut self, placement: ZsGridPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn row_span(mut self, span: ZsGridSpan) -> Self {
        self.placement.row_span = span;
        self
    }

    pub fn column_span(mut self, span: ZsGridSpan) -> Self {
        self.placement.column_span = span;
        self
    }
}

#[cfg(feature = "date-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDatePickerState {
    pub value: ZsDate,
    pub minimum: ZsDate,
    pub maximum: ZsDate,
    pub visible_month: ZsDate,
    pub expanded: bool,
}

#[cfg(feature = "time-picker")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTimePickerState {
    pub value: ZsTime,
    pub minute_increment: ZsMinuteIncrement,
    pub clock: ZsClockFormat,
    pub expanded: bool,
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone)]
pub struct ZsTabItem<Msg> {
    pub spec: ZsTabSpec,
    pub content: ViewNode<Msg>,
}

#[cfg(feature = "tabs")]
impl<Msg> ZsTabItem<Msg> {
    pub fn new(id: ZsTabId, label: impl Into<String>, content: ViewNode<Msg>) -> Self {
        Self {
            spec: ZsTabSpec::new(id, label),
            content,
        }
    }
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTabViewState {
    pub selected: Option<ZsTabId>,
    pub tab_count: usize,
}

#[cfg(feature = "tabs")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ZsTabHeaderState {
    pub tab_view: WidgetId,
    pub tab: ZsTabId,
    pub selected: bool,
    pub previous: Option<WidgetId>,
    pub next: Option<WidgetId>,
    pub first: WidgetId,
    pub last: WidgetId,
}

#[derive(Debug, Clone)]
pub enum ViewNodeKind<Msg> {
    #[doc(hidden)]
    __Message(PhantomData<fn() -> Msg>),
    #[cfg(feature = "label")]
    Text {
        text: String,
    },
    #[cfg(feature = "button")]
    Button {
        label: String,
        on_click: Option<Msg>,
    },
    #[cfg(feature = "textbox")]
    Textbox {
        value: String,
        multiline: bool,
        on_change: Option<fn(String) -> Msg>,
    },
    #[cfg(feature = "checkbox")]
    Checkbox {
        label: String,
        checked: bool,
        on_toggle: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "toggle")]
    Toggle {
        checked: bool,
        on_toggle: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "radio")]
    RadioButton {
        label: String,
        selected: bool,
        on_choose: Option<Msg>,
    },
    #[cfg(feature = "slider")]
    Slider {
        value: f32,
        range: SliderRange,
        on_slide: Option<fn(f32) -> Msg>,
    },
    #[cfg(feature = "progress")]
    ProgressBar {
        value: f32,
        range: ProgressRange,
    },
    #[cfg(feature = "combo")]
    ComboBox {
        options: Vec<String>,
        selected_index: Option<usize>,
        expanded: bool,
        placeholder: Option<String>,
        on_select: Option<fn(usize) -> Msg>,
        on_expanded_change: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "date-picker")]
    DatePicker {
        value: ZsDate,
        minimum: ZsDate,
        maximum: ZsDate,
        visible_month: ZsDate,
        today: Option<ZsDate>,
        expanded: bool,
        on_date_change: Option<fn(ZsDate) -> Msg>,
        on_expanded_change: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "time-picker")]
    TimePicker {
        value: ZsTime,
        minute_increment: ZsMinuteIncrement,
        clock: ZsClockFormat,
        expanded: bool,
        on_time_change: Option<fn(ZsTime) -> Msg>,
        on_expanded_change: Option<fn(bool) -> Msg>,
    },
    #[cfg(feature = "tabs")]
    Tabs {
        tabs: Vec<ZsTabSpec>,
        selected: Option<ZsTabId>,
        on_select: Option<fn(ZsTabId) -> Msg>,
    },
    #[cfg(feature = "list")]
    List {
        selected_index: Option<usize>,
        on_select: Option<fn(usize) -> Msg>,
    },
    #[cfg(feature = "scroll")]
    Scroll {
        offset_y: Dp,
        content_height: Option<Dp>,
        on_scroll: Option<fn(Dp) -> Msg>,
    },
    #[cfg(feature = "virtual-list")]
    VirtualList {
        total_count: usize,
        row_height: Dp,
        overscan_rows: usize,
        row_indices: Vec<usize>,
        selected_index: Option<usize>,
        offset_y: Dp,
        visible_range: VirtualListRange,
        materialized_range: VirtualListRange,
        on_select: Option<fn(usize) -> Msg>,
        on_viewport_changed: Option<fn(VirtualListViewport) -> Msg>,
        loading: bool,
        show_placeholders: bool,
    },
    #[cfg(feature = "grid")]
    Grid {
        columns: Vec<ZsGridTrack>,
        rows: Vec<ZsGridTrack>,
        placements: Vec<ZsGridPlacement>,
        column_gap: Option<Dp>,
        row_gap: Option<Dp>,
    },
    Stack {
        direction: ViewStackDirection,
    },
    Spacer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewStyle {
    pub padding: Option<Dp>,
    pub radius: Option<Dp>,
    pub background: Option<ThemeColorToken>,
    pub width: Option<Dp>,
    pub height: Option<Dp>,
    pub flex: f32,
    pub gap: Option<Dp>,
    pub theme_mode: Option<ZsuiThemeMode>,
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self {
            padding: None,
            radius: None,
            background: None,
            width: None,
            height: None,
            flex: 1.0,
            gap: None,
            theme_mode: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ViewNode<Msg> {
    pub id: Option<WidgetId>,
    pub kind: ViewNodeKind<Msg>,
    pub style: ViewStyle,
    pub children: Vec<ViewNode<Msg>>,
    bounds: Option<Rect>,
    layout_dpi: Dpi,
    #[cfg(feature = "combo")]
    combo_first_visible_option: Option<usize>,
    message: PhantomData<fn() -> Msg>,
}

impl<Msg> ViewNode<Msg> {
    pub fn new(kind: ViewNodeKind<Msg>) -> Self {
        Self {
            id: None,
            kind,
            style: ViewStyle::default(),
            children: Vec::new(),
            bounds: None,
            layout_dpi: Dpi::standard(),
            #[cfg(feature = "combo")]
            combo_first_visible_option: None,
            message: PhantomData,
        }
    }

    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn padding(mut self, padding: Dp) -> Self {
        self.style.padding = Some(padding);
        self
    }

    pub fn radius(mut self, radius: Dp) -> Self {
        self.style.radius = Some(radius);
        self
    }

    pub fn bg(mut self, token: ThemeColorToken) -> Self {
        self.style.background = Some(token);
        self
    }

    pub fn width(mut self, width: Dp) -> Self {
        self.style.width = Some(width);
        self
    }

    pub fn height(mut self, height: Dp) -> Self {
        self.style.height = Some(height);
        self
    }

    pub fn flex(mut self, flex: f32) -> Self {
        self.style.flex = flex.max(0.0);
        self
    }

    pub fn gap(mut self, gap: Dp) -> Self {
        self.style.gap = Some(gap);
        self
    }

    #[cfg(feature = "grid")]
    /// Overrides the Grid's horizontal spacing; `.gap(...)` remains the fallback.
    pub fn column_gap(mut self, gap: Dp) -> Self {
        if let ViewNodeKind::Grid { column_gap, .. } = &mut self.kind {
            *column_gap = Some(gap);
        }
        self
    }

    #[cfg(feature = "grid")]
    /// Overrides the Grid's vertical spacing; `.gap(...)` remains the fallback.
    pub fn row_gap(mut self, gap: Dp) -> Self {
        if let ViewNodeKind::Grid { row_gap, .. } = &mut self.kind {
            *row_gap = Some(gap);
        }
        self
    }

    pub fn theme_mode(mut self, theme_mode: ZsuiThemeMode) -> Self {
        self.style.theme_mode = Some(theme_mode);
        self
    }

    pub fn child(mut self, child: ViewNode<Msg>) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = ViewNode<Msg>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    pub fn background_poll_interval_ms(&self) -> Option<u64> {
        #[cfg(feature = "virtual-list")]
        if matches!(self.kind, ViewNodeKind::VirtualList { loading: true, .. }) {
            return Some(33);
        }
        self.children
            .iter()
            .filter_map(ViewNode::background_poll_interval_ms)
            .min()
    }

    #[cfg(any(feature = "tabs", feature = "virtual-list"))]
    fn clear_layout_bounds(&mut self) {
        self.bounds = None;
        for child in &mut self.children {
            child.clear_layout_bounds();
        }
    }
}

impl<Msg: Clone> ViewNode<Msg> {
    #[cfg(feature = "button")]
    pub fn on_click(mut self, message: Msg) -> Self {
        if let ViewNodeKind::Button { on_click, .. } = &mut self.kind {
            *on_click = Some(message);
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn on_change(mut self, message: fn(String) -> Msg) -> Self {
        if let ViewNodeKind::Textbox { on_change, .. } = &mut self.kind {
            *on_change = Some(message);
        }
        self
    }

    #[cfg(any(feature = "checkbox", feature = "toggle"))]
    pub fn on_toggle(mut self, message: fn(bool) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { on_toggle, .. } => *on_toggle = Some(message),
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { on_toggle, .. } => *on_toggle = Some(message),
            _ => {}
        }
        self
    }

    #[cfg(feature = "slider")]
    pub fn on_slide(mut self, message: fn(f32) -> Msg) -> Self {
        if let ViewNodeKind::Slider { on_slide, .. } = &mut self.kind {
            *on_slide = Some(message);
        }
        self
    }

    #[cfg(feature = "radio")]
    pub fn on_choose(mut self, message: Msg) -> Self {
        if let ViewNodeKind::RadioButton { on_choose, .. } = &mut self.kind {
            *on_choose = Some(message);
        }
        self
    }

    #[cfg(feature = "list")]
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        match &mut self.kind {
            ViewNodeKind::List { selected_index, .. } => *selected_index = index,
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { selected_index, .. } => *selected_index = index,
            _ => {}
        }
        self
    }

    #[cfg(any(feature = "list", feature = "combo"))]
    pub fn on_select(mut self, message: fn(usize) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "list")]
            ViewNodeKind::List { on_select, .. } => *on_select = Some(message),
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { on_select, .. } => *on_select = Some(message),
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { on_select, .. } => *on_select = Some(message),
            _ => {}
        }
        self
    }

    #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
    pub fn expanded(mut self, is_expanded: bool) -> Self {
        match &mut self.kind {
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker { expanded, .. } => *expanded = is_expanded,
            _ => {}
        }
        self
    }

    #[cfg(feature = "combo")]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        if let ViewNodeKind::ComboBox { placeholder, .. } = &mut self.kind {
            *placeholder = Some(text.into());
        }
        self
    }

    #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
    pub fn on_expanded_change(mut self, message: fn(bool) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox {
                on_expanded_change, ..
            } => *on_expanded_change = Some(message),
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker {
                on_expanded_change, ..
            } => *on_expanded_change = Some(message),
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker {
                on_expanded_change, ..
            } => *on_expanded_change = Some(message),
            _ => {}
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn date_range(mut self, minimum: ZsDate, maximum: ZsDate) -> Self {
        if let ViewNodeKind::DatePicker {
            value,
            minimum: current_minimum,
            maximum: current_maximum,
            visible_month,
            ..
        } = &mut self.kind
        {
            let (minimum, maximum) = if minimum <= maximum {
                (minimum, maximum)
            } else {
                (maximum, minimum)
            };
            *current_minimum = minimum;
            *current_maximum = maximum;
            *value = (*value).clamp(minimum, maximum);
            *visible_month = clamp_visible_month(*visible_month, minimum, maximum);
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn on_date_change(mut self, message: fn(ZsDate) -> Msg) -> Self {
        if let ViewNodeKind::DatePicker { on_date_change, .. } = &mut self.kind {
            *on_date_change = Some(message);
        }
        self
    }

    /// Overrides the local-clock date used for the CalendarView today marker.
    #[cfg(feature = "date-picker")]
    pub fn today(mut self, today: ZsDate) -> Self {
        if let ViewNodeKind::DatePicker { today: current, .. } = &mut self.kind {
            *current = Some(today);
        }
        self
    }

    #[cfg(feature = "time-picker")]
    pub fn minute_increment(mut self, increment: ZsMinuteIncrement) -> Self {
        if let ViewNodeKind::TimePicker {
            value,
            minute_increment,
            ..
        } = &mut self.kind
        {
            *minute_increment = increment;
            *value = value.snap(increment);
        }
        self
    }

    #[cfg(feature = "time-picker")]
    pub fn clock_format(mut self, clock: ZsClockFormat) -> Self {
        if let ViewNodeKind::TimePicker { clock: current, .. } = &mut self.kind {
            *current = clock;
        }
        self
    }

    #[cfg(feature = "time-picker")]
    pub fn on_time_change(mut self, message: fn(ZsTime) -> Msg) -> Self {
        if let ViewNodeKind::TimePicker { on_time_change, .. } = &mut self.kind {
            *on_time_change = Some(message);
        }
        self
    }

    #[cfg(feature = "tabs")]
    pub fn on_tab_select(mut self, message: fn(ZsTabId) -> Msg) -> Self {
        if let ViewNodeKind::Tabs { on_select, .. } = &mut self.kind {
            *on_select = Some(message);
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn scroll_y(mut self, offset_y: Dp) -> Self {
        match &mut self.kind {
            ViewNodeKind::Scroll {
                offset_y: current, ..
            } => *current = offset_y,
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList {
                offset_y: current, ..
            } => *current = offset_y,
            _ => {}
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn content_height(mut self, height: Dp) -> Self {
        if let ViewNodeKind::Scroll { content_height, .. } = &mut self.kind {
            *content_height = Some(height);
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn on_scroll(mut self, message: fn(Dp) -> Msg) -> Self {
        if let ViewNodeKind::Scroll { on_scroll, .. } = &mut self.kind {
            *on_scroll = Some(message);
        }
        self
    }

    #[cfg(feature = "virtual-list")]
    pub fn item_height(mut self, row_height: Dp) -> Self {
        if let ViewNodeKind::VirtualList {
            row_height: current,
            ..
        } = &mut self.kind
        {
            *current = Dp::new(row_height.0.max(1.0));
        }
        self
    }

    #[cfg(feature = "virtual-list")]
    pub fn overscan_rows(mut self, rows: usize) -> Self {
        if let ViewNodeKind::VirtualList { overscan_rows, .. } = &mut self.kind {
            *overscan_rows = rows;
        }
        self
    }

    #[cfg(feature = "virtual-list")]
    pub fn on_viewport_changed(mut self, message: fn(VirtualListViewport) -> Msg) -> Self {
        if let ViewNodeKind::VirtualList {
            on_viewport_changed,
            ..
        } = &mut self.kind
        {
            *on_viewport_changed = Some(message);
        }
        self
    }

    #[cfg(feature = "virtual-list")]
    pub fn loading(mut self, is_loading: bool) -> Self {
        if let ViewNodeKind::VirtualList { loading, .. } = &mut self.kind {
            *loading = is_loading;
        }
        self
    }

    #[cfg(feature = "virtual-list")]
    pub fn placeholders(mut self, visible: bool) -> Self {
        if let ViewNodeKind::VirtualList {
            show_placeholders, ..
        } = &mut self.kind
        {
            *show_placeholders = visible;
        }
        self
    }
}

#[cfg(feature = "label")]
pub fn text<Msg>(text: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Text { text: text.into() })
}

#[cfg(feature = "button")]
pub fn button<Msg>(label: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Button {
        label: label.into(),
        on_click: None,
    })
}

#[cfg(feature = "textbox")]
pub fn textbox<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Textbox {
        value: value.into(),
        multiline: false,
        on_change: None,
    })
}

#[cfg(feature = "textbox")]
pub fn text_editor<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::Textbox {
        value: value.into(),
        multiline: true,
        on_change: None,
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

#[cfg(feature = "slider")]
pub fn slider<Msg>(value: f32, range: impl Into<SliderRange>) -> ViewNode<Msg> {
    let range = range.into();
    ViewNode::new(ViewNodeKind::Slider {
        value: range.snap(value),
        range,
        on_slide: None,
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
pub fn progress_bar<Msg>(value: f32, range: impl Into<ProgressRange>) -> ViewNode<Msg> {
    let range = range.into();
    ViewNode::new(ViewNodeKind::ProgressBar {
        value: range.clamp(value),
        range,
    })
}

#[cfg(feature = "combo")]
pub fn combo_box<T, Msg>(
    options: impl IntoIterator<Item = T>,
    selected_index: Option<usize>,
) -> ViewNode<Msg>
where
    T: Into<String>,
{
    let options = options.into_iter().map(Into::into).collect::<Vec<_>>();
    let selected_index = selected_index.filter(|index| *index < options.len());
    ViewNode::new(ViewNodeKind::ComboBox {
        options,
        selected_index,
        expanded: false,
        placeholder: None,
        on_select: None,
        on_expanded_change: None,
    })
}

#[cfg(feature = "date-picker")]
pub fn date_picker<Msg>(value: ZsDate) -> ViewNode<Msg> {
    let minimum = ZsDate::new(ZsDate::MIN_YEAR, 1, 1).expect("minimum date is valid");
    let maximum = ZsDate::new(ZsDate::MAX_YEAR, 12, 31).expect("maximum date is valid");
    ViewNode::new(ViewNodeKind::DatePicker {
        value,
        minimum,
        maximum,
        visible_month: value.first_day_of_month(),
        today: ZsDate::today_local().ok(),
        expanded: false,
        on_date_change: None,
        on_expanded_change: None,
    })
}

#[cfg(feature = "time-picker")]
pub fn time_picker<Msg>(value: ZsTime) -> ViewNode<Msg> {
    ViewNode::new(ViewNodeKind::TimePicker {
        value,
        minute_increment: ZsMinuteIncrement::ONE,
        clock: ZsTimePickerPlatformStyle::current().default_clock(),
        expanded: false,
        on_time_change: None,
        on_expanded_change: None,
    })
}

#[cfg(feature = "tabs")]
pub fn tab_view<Msg>(
    items: impl IntoIterator<Item = ZsTabItem<Msg>>,
    selected: Option<ZsTabId>,
) -> ViewNode<Msg> {
    let items = items.into_iter().collect::<Vec<_>>();
    let tabs = items
        .iter()
        .map(|item| item.spec.clone())
        .collect::<Vec<_>>();
    let selected = selected
        .filter(|selected| tabs.iter().any(|tab| tab.id == *selected))
        .or_else(|| tabs.first().map(|tab| tab.id));
    ViewNode::new(ViewNodeKind::Tabs {
        tabs,
        selected,
        on_select: None,
    })
    .children(items.into_iter().map(|item| item.content))
}

pub fn row<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Row,
    })
    .children(children)
}

pub fn column<Msg>(children: impl IntoIterator<Item = ViewNode<Msg>>) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Stack {
        direction: ViewStackDirection::Column,
    })
    .children(children)
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

#[cfg(feature = "list")]
pub fn list<T, Msg>(
    items: impl IntoIterator<Item = T>,
    render: impl FnMut(T) -> ViewNode<Msg>,
) -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::List {
        selected_index: None,
        on_select: None,
    })
    .children(items.into_iter().map(render))
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

pub fn spacer<Msg>() -> ViewNode<Msg> {
    ViewNode::<Msg>::new(ViewNodeKind::Spacer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewEvent {
    Click {
        widget: WidgetId,
    },
    TextChanged {
        widget: WidgetId,
        value: String,
    },
    Toggled {
        widget: WidgetId,
        checked: bool,
    },
    #[cfg(feature = "slider")]
    SliderChanged {
        widget: WidgetId,
        value: f32,
    },
    #[cfg(feature = "radio")]
    RadioSelected {
        widget: WidgetId,
    },
    #[cfg(feature = "combo")]
    ComboBoxExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "combo")]
    ComboBoxSelected {
        widget: WidgetId,
        index: usize,
    },
    #[cfg(feature = "combo")]
    ComboBoxScrolled {
        widget: WidgetId,
        first_visible_index: usize,
    },
    #[cfg(feature = "date-picker")]
    DatePickerExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "date-picker")]
    DatePickerMonthChanged {
        widget: WidgetId,
        month: ZsDate,
    },
    #[cfg(feature = "date-picker")]
    DateChanged {
        widget: WidgetId,
        value: ZsDate,
    },
    #[cfg(feature = "time-picker")]
    TimePickerExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "time-picker")]
    TimeChanged {
        widget: WidgetId,
        value: ZsTime,
    },
    #[cfg(feature = "tabs")]
    TabSelected {
        widget: WidgetId,
        tab: ZsTabId,
    },
    #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
    DismissPopupOverlays {
        except: Option<WidgetId>,
    },
    #[cfg(feature = "scroll")]
    ScrollBy {
        widget: WidgetId,
        delta_y: Dp,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewHitTarget {
    pub widget: WidgetId,
    pub bounds: Rect,
    pub kind: ViewHitTargetKind,
}

impl ViewHitTarget {
    pub const fn new(widget: WidgetId, bounds: Rect) -> Self {
        Self {
            widget,
            bounds,
            kind: ViewHitTargetKind::Unknown,
        }
    }

    pub const fn with_kind(widget: WidgetId, bounds: Rect, kind: ViewHitTargetKind) -> Self {
        Self {
            widget,
            bounds,
            kind,
        }
    }

    pub const fn contains(self, point: Point) -> bool {
        self.bounds.contains(point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewHitTargetKind {
    Unknown,
    Button,
    Textbox,
    TextEditor,
    Checkbox,
    Toggle,
    #[cfg(feature = "slider")]
    Slider,
    #[cfg(feature = "radio")]
    RadioButton,
    #[cfg(feature = "combo")]
    ComboBox,
    #[cfg(feature = "combo")]
    ComboBoxOption {
        index: usize,
    },
    #[cfg(feature = "date-picker")]
    DatePicker,
    #[cfg(feature = "date-picker")]
    DatePickerDay {
        date: ZsDate,
    },
    #[cfg(feature = "date-picker")]
    DatePickerPreviousMonth,
    #[cfg(feature = "date-picker")]
    DatePickerNextMonth,
    #[cfg(feature = "time-picker")]
    TimePicker,
    #[cfg(feature = "time-picker")]
    TimePickerChoice {
        value: ZsTime,
    },
    #[cfg(feature = "tabs")]
    Tab {
        tab_view: WidgetId,
        tab: ZsTabId,
        index: usize,
    },
    #[cfg(feature = "scroll")]
    Scroll,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewInteractionPlan {
    pub hit_targets: Vec<ViewHitTarget>,
}

impl ViewInteractionPlan {
    pub fn new(hit_targets: impl IntoIterator<Item = ViewHitTarget>) -> Self {
        Self {
            hit_targets: hit_targets.into_iter().collect(),
        }
    }

    pub fn from_layout_output(layout: &LayoutOutput) -> Self {
        Self::new(
            layout
                .children
                .iter()
                .map(|node| ViewHitTarget::new(node.component.into(), node.bounds)),
        )
    }

    pub fn hit_target_count(&self) -> usize {
        self.hit_targets.len()
    }

    pub fn target_at(&self, point: Point) -> Option<WidgetId> {
        self.hit_target_at(point).map(|target| target.widget)
    }

    pub fn hit_target_at(&self, point: Point) -> Option<ViewHitTarget> {
        self.hit_targets
            .iter()
            .rev()
            .copied()
            .find(|target| target.contains(point))
    }

    pub fn hit_target_for_widget(&self, widget: WidgetId) -> Option<ViewHitTarget> {
        self.hit_targets
            .iter()
            .copied()
            .find(|target| target.widget == widget)
    }

    pub fn target_kind_at(&self, point: Point) -> Option<ViewHitTargetKind> {
        self.hit_target_at(point).map(|target| target.kind)
    }

    #[cfg(feature = "combo")]
    pub(crate) fn combo_visible_option_range(
        &self,
        widget: WidgetId,
    ) -> Option<std::ops::Range<usize>> {
        let mut indices = self.hit_targets.iter().filter_map(|target| {
            if target.widget != widget {
                return None;
            }
            match target.kind {
                ViewHitTargetKind::ComboBoxOption { index } => Some(index),
                _ => None,
            }
        });
        let first = indices.next()?;
        let (minimum, maximum) = indices.fold((first, first), |(minimum, maximum), index| {
            (minimum.min(index), maximum.max(index))
        });
        Some(minimum..maximum.saturating_add(1))
    }

    pub fn click_event_at(&self, point: Point) -> Option<ViewEvent> {
        self.target_at(point)
            .map(|widget| ViewEvent::Click { widget })
    }

    pub fn first_focus_target(&self) -> Option<ViewHitTarget> {
        self.hit_targets
            .iter()
            .copied()
            .find(|target| target.accepts_focus())
    }

    pub fn next_focus_target(
        &self,
        current: Option<WidgetId>,
        offset: isize,
    ) -> Option<ViewHitTarget> {
        let focus_targets = self
            .hit_targets
            .iter()
            .copied()
            .filter(ViewHitTarget::accepts_focus)
            .collect::<Vec<_>>();
        let len = focus_targets.len();
        if len == 0 {
            return None;
        }

        let current_index = current.and_then(|widget| {
            focus_targets
                .iter()
                .position(|target| target.widget == widget)
        });
        let next_index = match current_index {
            Some(index) => (index as isize + offset).rem_euclid(len as isize) as usize,
            None if offset < 0 => len - 1,
            None => 0,
        };
        focus_targets.get(next_index).copied()
    }

    pub(crate) fn next_focus_target_where(
        &self,
        current: Option<WidgetId>,
        offset: isize,
        mut accepts_tab_focus: impl FnMut(ViewHitTarget) -> bool,
    ) -> Option<ViewHitTarget> {
        let len = self.hit_targets.len();
        if len == 0 || offset == 0 {
            return None;
        }
        let step = offset.signum();
        let start = current
            .and_then(|widget| {
                self.hit_targets
                    .iter()
                    .position(|target| target.widget == widget)
            })
            .map(|index| index as isize)
            .unwrap_or(if step < 0 { 0 } else { -1 });
        (1..=len).find_map(|distance| {
            let index = (start + step * distance as isize).rem_euclid(len as isize) as usize;
            let target = self.hit_targets[index];
            (target.accepts_focus() && accepts_tab_focus(target)).then_some(target)
        })
    }
}

impl ViewHitTarget {
    fn accepts_focus(&self) -> bool {
        #[cfg(feature = "combo")]
        if matches!(self.kind, ViewHitTargetKind::ComboBoxOption { .. }) {
            return false;
        }
        #[cfg(feature = "date-picker")]
        if matches!(
            self.kind,
            ViewHitTargetKind::DatePickerDay { .. }
                | ViewHitTargetKind::DatePickerPreviousMonth
                | ViewHitTargetKind::DatePickerNextMonth
        ) {
            return false;
        }
        #[cfg(feature = "time-picker")]
        if matches!(self.kind, ViewHitTargetKind::TimePickerChoice { .. }) {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewLayoutCx {
    pub bounds: Rect,
    pub dpi: Dpi,
}

impl ViewLayoutCx {
    pub const fn new(bounds: Rect, dpi: Dpi) -> Self {
        Self { bounds, dpi }
    }
}

#[derive(Debug, Clone)]
pub struct ViewEventCx<Msg> {
    messages: Vec<Msg>,
}

impl<Msg> ViewEventCx<Msg> {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn emit(&mut self, message: Msg) {
        self.messages.push(message);
    }

    pub fn messages(&self) -> &[Msg] {
        &self.messages
    }

    pub fn into_messages(self) -> Vec<Msg> {
        self.messages
    }
}

impl<Msg> Default for ViewEventCx<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppCx {
    commands: Vec<Command>,
    ui_commands: Vec<UiCommand>,
    quit_requested: bool,
}

impl AppCx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn ui_command(&mut self, command: UiCommand) {
        self.ui_commands.push(command);
    }

    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn ui_commands(&self) -> &[UiCommand] {
        &self.ui_commands
    }

    pub const fn quit_requested(&self) -> bool {
        self.quit_requested
    }
}

#[derive(Debug, Clone, Default)]
pub struct LiveViewUpdate {
    pub redraw: bool,
    pub message_count: usize,
    pub commands: Vec<Command>,
    pub ui_commands: Vec<UiCommand>,
    pub quit_requested: bool,
    pub revision: u64,
}

trait LiveViewDriver: Send {
    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool;
    fn refresh(&mut self) -> LiveViewUpdate;
    fn background_poll_interval_ms(&self) -> Option<u64>;
    fn draw_plan(&self) -> NativeDrawPlan;
    fn interaction_plan(&self) -> ViewInteractionPlan;
    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate;
    fn widget_text_value(&self, widget: WidgetId) -> Option<String>;
    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool>;
    #[cfg(feature = "radio")]
    fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool>;
    #[cfg(feature = "radio")]
    fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId>;
    #[cfg(feature = "slider")]
    fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)>;
    #[cfg(feature = "combo")]
    fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)>;
    #[cfg(feature = "combo")]
    fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize>;
    #[cfg(feature = "date-picker")]
    fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState>;
    #[cfg(feature = "time-picker")]
    fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState>;
    #[cfg(feature = "tabs")]
    fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState>;
    #[cfg(feature = "tabs")]
    fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)>;
    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)>;
    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize>;
    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId>;
    fn revision(&self) -> u64;
}

#[derive(Clone)]
pub struct SharedLiveViewRuntime {
    inner: Arc<Mutex<Box<dyn LiveViewDriver>>>,
}

impl SharedLiveViewRuntime {
    pub fn set_surface(&self, bounds: Rect, dpi: Dpi) -> bool {
        self.lock().set_surface(bounds, dpi)
    }

    pub fn draw_plan(&self) -> NativeDrawPlan {
        self.lock().draw_plan()
    }

    pub fn refresh(&self) -> LiveViewUpdate {
        self.lock().refresh()
    }

    pub fn background_poll_interval_ms(&self) -> Option<u64> {
        self.lock().background_poll_interval_ms()
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        self.lock().interaction_plan()
    }

    pub fn dispatch_event(&self, event: &ViewEvent) -> LiveViewUpdate {
        self.lock().dispatch_event(event)
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.lock().widget_text_value(widget)
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.lock().widget_checked_value(widget)
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        self.lock().widget_radio_is_tab_stop(widget)
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        self.lock()
            .widget_radio_relative_widget(widget, navigation, offset)
    }

    #[cfg(feature = "slider")]
    pub fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        self.lock().widget_slider_state(widget)
    }

    #[cfg(feature = "combo")]
    pub fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        self.lock().widget_combo_state(widget)
    }

    #[cfg(feature = "combo")]
    pub(crate) fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        self.lock()
            .widget_combo_type_ahead_match(widget, query, start_after)
    }

    #[cfg(feature = "date-picker")]
    pub fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        self.lock().widget_date_picker_state(widget)
    }

    #[cfg(feature = "time-picker")]
    pub fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        self.lock().widget_time_picker_state(widget)
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        self.lock().widget_tab_header_state(widget)
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        self.lock().widget_tab_cycle_target(focused, offset)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.lock().widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.lock().widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.lock().widget_scroll_target(widget)
    }

    pub fn revision(&self) -> u64 {
        self.lock().revision()
    }

    fn lock(&self) -> MutexGuard<'_, Box<dyn LiveViewDriver>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for SharedLiveViewRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedLiveViewRuntime")
            .field("revision", &self.revision())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SharedLiveViewRuntime {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

struct TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    view: ViewNode<Msg>,
    bounds: Rect,
    dpi: Dpi,
    revision: u64,
}

impl<State, Msg, ViewFn, UpdateFn> TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    Msg: Clone,
    ViewFn: Fn(&State) -> ViewNode<Msg>,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx),
{
    fn new(state: State, view_fn: ViewFn, update_fn: UpdateFn, bounds: Rect, dpi: Dpi) -> Self {
        let view = view_fn(&state);
        let mut driver = Self {
            state,
            view_fn,
            update_fn,
            view,
            bounds,
            dpi,
            revision: 0,
        };
        driver.layout();
        driver
    }

    fn layout(&mut self) {
        self.view = (self.view_fn)(&self.state);
        let mut cx = ViewLayoutCx::new(self.bounds, self.dpi);
        self.view.layout(&mut cx);
    }
}

impl<State, Msg, ViewFn, UpdateFn> LiveViewDriver
    for TypedLiveViewDriver<State, Msg, ViewFn, UpdateFn>
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool {
        if self.bounds == bounds && self.dpi == dpi {
            return false;
        }
        self.bounds = bounds;
        self.dpi = dpi;
        self.layout();
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn refresh(&mut self) -> LiveViewUpdate {
        self.layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: true,
            revision: self.revision,
            ..LiveViewUpdate::default()
        }
    }

    fn background_poll_interval_ms(&self) -> Option<u64> {
        self.view.background_poll_interval_ms()
    }

    fn draw_plan(&self) -> NativeDrawPlan {
        let mut cx = ViewPaintCx::new(self.dpi);
        self.view.paint(&mut cx);
        cx.into_plan()
    }

    fn interaction_plan(&self) -> ViewInteractionPlan {
        self.view.interaction_plan()
    }

    fn dispatch_event(&mut self, event: &ViewEvent) -> LiveViewUpdate {
        let mut event_cx = ViewEventCx::new();
        self.view.event(&mut event_cx, event);
        let messages = event_cx.into_messages();
        if messages.is_empty() {
            self.revision = self.revision.saturating_add(1);
            return LiveViewUpdate {
                redraw: true,
                revision: self.revision,
                ..LiveViewUpdate::default()
            };
        }

        let message_count = messages.len();
        let mut app_cx = AppCx::new();
        for message in messages {
            (self.update_fn)(&mut self.state, message, &mut app_cx);
        }
        self.layout();
        self.revision = self.revision.saturating_add(1);
        LiveViewUpdate {
            redraw: true,
            message_count,
            commands: app_cx.commands().to_vec(),
            ui_commands: app_cx.ui_commands().to_vec(),
            quit_requested: app_cx.quit_requested(),
            revision: self.revision,
        }
    }

    fn widget_text_value(&self, widget: WidgetId) -> Option<String> {
        self.view.widget_text_value(widget).map(str::to_string)
    }

    fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        self.view.widget_checked_value(widget)
    }

    #[cfg(feature = "radio")]
    fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        self.view.widget_radio_is_tab_stop(widget)
    }

    #[cfg(feature = "radio")]
    fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        self.view
            .widget_radio_relative_widget(widget, navigation, offset)
    }

    #[cfg(feature = "slider")]
    fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        self.view.widget_slider_state(widget)
    }

    #[cfg(feature = "combo")]
    fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        self.view.widget_combo_state(widget)
    }

    #[cfg(feature = "combo")]
    fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        self.view
            .widget_combo_type_ahead_match(widget, query, start_after)
    }

    #[cfg(feature = "date-picker")]
    fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        self.view.widget_date_picker_state(widget)
    }

    #[cfg(feature = "time-picker")]
    fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        self.view.widget_time_picker_state(widget)
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        self.view.widget_tab_header_state(widget)
    }

    #[cfg(feature = "tabs")]
    fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        self.view.widget_tab_cycle_target(focused, offset)
    }

    #[cfg(feature = "list")]
    fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        self.view.widget_list_relative_widget(widget, offset)
    }

    #[cfg(feature = "list")]
    fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        self.view.widget_list_index(widget)
    }

    #[cfg(feature = "scroll")]
    fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        self.view.widget_scroll_target(widget)
    }

    fn revision(&self) -> u64 {
        self.revision
    }
}

pub fn live_view_runtime<State, Msg, ViewFn, UpdateFn>(
    state: State,
    view_fn: ViewFn,
    update_fn: UpdateFn,
    bounds: Rect,
    dpi: Dpi,
) -> SharedLiveViewRuntime
where
    State: Send + 'static,
    Msg: Clone + Send + 'static,
    ViewFn: Fn(&State) -> ViewNode<Msg> + Send + 'static,
    UpdateFn: Fn(&mut State, Msg, &mut AppCx) + Send + 'static,
{
    SharedLiveViewRuntime {
        inner: Arc::new(Mutex::new(Box::new(TypedLiveViewDriver::new(
            state, view_fn, update_fn, bounds, dpi,
        )))),
    }
}

#[derive(Debug, Clone)]
pub struct ViewPaintCx {
    pub dpi: Dpi,
    plan: NativeDrawPlan,
    paint_depth: usize,
}

impl ViewPaintCx {
    pub fn new(dpi: Dpi) -> Self {
        Self {
            dpi,
            plan: NativeDrawPlan::default(),
            paint_depth: 0,
        }
    }

    pub fn draw(&mut self, command: NativeDrawCommand) {
        self.plan.push(command);
    }

    pub fn plan(&self) -> &NativeDrawPlan {
        &self.plan
    }

    pub fn into_plan(self) -> NativeDrawPlan {
        self.plan
    }

    pub fn set_theme_mode(&mut self, theme_mode: ZsuiThemeMode) {
        self.plan.theme_mode = theme_mode;
    }

    fn finish_node<Msg>(&mut self, _root: &ViewNode<Msg>) {
        self.paint_depth = self.paint_depth.saturating_sub(1);
        #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
        if self.paint_depth == 0 {
            _root.paint_overlays(self, None);
        }
    }
}

pub trait View<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput;
    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent);
    fn paint(&self, cx: &mut ViewPaintCx);
}

#[cfg(feature = "tabs")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_tabs(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let (tabs, selected) = match &self.kind {
            ViewNodeKind::Tabs { tabs, selected, .. } => (tabs, *selected),
            _ => unreachable!("tab layout requires a tab view node"),
        };
        let labels = tabs.iter().map(|tab| tab.label.clone()).collect::<Vec<_>>();
        let selected_index = selected
            .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
        let plan = crate::zs_tab_view_render_plan(
            cx.bounds,
            &labels,
            selected_index,
            crate::ZsTabPlatformStyle::current(),
            cx.dpi,
        );
        for child in &mut self.children {
            child.clear_layout_bounds();
        }
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }
        if let Some(child) = selected_index.and_then(|index| self.children.get_mut(index)) {
            let mut child_cx = ViewLayoutCx {
                bounds: plan.content_bounds,
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

#[cfg(feature = "virtual-list")]
impl<Msg: Clone> ViewNode<Msg> {
    fn layout_virtual_list(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let (total_count, row_height, overscan_rows, row_indices, current_offset) = match &self.kind
        {
            ViewNodeKind::VirtualList {
                total_count,
                row_height,
                overscan_rows,
                row_indices,
                offset_y,
                ..
            } => (
                *total_count,
                *row_height,
                *overscan_rows,
                row_indices.clone(),
                *offset_y,
            ),
            _ => unreachable!("virtual list layout requires a virtual list node"),
        };
        let content_bounds = inset_bounds(cx.bounds, self.style.padding, cx.dpi);
        let viewport_height =
            Dp::new(content_bounds.height.max(0) as f32 / cx.dpi.scale_factor().max(f32::EPSILON));
        let viewport = virtual_list_viewport(
            total_count,
            row_height,
            current_offset,
            viewport_height,
            overscan_rows,
            VirtualListScrollDirection::Stationary,
        );
        if let ViewNodeKind::VirtualList {
            offset_y,
            visible_range,
            materialized_range,
            ..
        } = &mut self.kind
        {
            *offset_y = viewport.offset_y;
            *visible_range = viewport.visible_range;
            *materialized_range = viewport.materialized_range;
        }

        for child in &mut self.children {
            child.clear_layout_bounds();
        }
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }
        let row_height_px = row_height.to_px(cx.dpi).round_i32().max(1);
        let offset_px = viewport.offset_y.to_px(cx.dpi).round_i32().max(0);
        for (index, child) in row_indices.into_iter().zip(self.children.iter_mut()) {
            if !viewport.materialized_range.contains(index) {
                continue;
            }
            let row_top = (index as i64)
                .saturating_mul(row_height_px as i64)
                .saturating_sub(offset_px as i64)
                .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            let mut child_cx = ViewLayoutCx {
                bounds: Rect {
                    x: content_bounds.x,
                    y: content_bounds.y.saturating_add(row_top),
                    width: content_bounds.width,
                    height: row_height_px,
                },
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }
        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }
}

impl<Msg: Clone> View<Msg> for ViewNode<Msg> {
    fn layout(&mut self, cx: &mut ViewLayoutCx) -> LayoutOutput {
        #[cfg(feature = "tabs")]
        if matches!(self.kind, ViewNodeKind::Tabs { .. }) {
            return self.layout_tabs(cx);
        }
        #[cfg(feature = "virtual-list")]
        if matches!(self.kind, ViewNodeKind::VirtualList { .. }) {
            return self.layout_virtual_list(cx);
        }

        self.bounds = Some(cx.bounds);
        self.layout_dpi = cx.dpi;
        let mut children = Vec::new();
        if let Some(id) = self.id {
            children.push(LayoutNode {
                component: id.into(),
                bounds: cx.bounds,
            });
        }

        let child_bounds = split_child_bounds(
            inset_bounds(cx.bounds, self.style.padding, cx.dpi),
            &self.kind,
            &self.children,
            self.style.gap,
            cx.dpi,
        );
        for (child, bounds) in self.children.iter_mut().zip(child_bounds) {
            let mut child_cx = ViewLayoutCx {
                bounds,
                dpi: cx.dpi,
            };
            children.extend(child.layout(&mut child_cx).children);
        }

        LayoutOutput {
            bounds: cx.bounds,
            children,
        }
    }

    fn event(&mut self, cx: &mut ViewEventCx<Msg>, event: &ViewEvent) {
        #[cfg(feature = "tabs")]
        if let ViewNodeKind::Tabs {
            tabs,
            selected,
            on_select,
        } = &mut self.kind
        {
            if let ViewEvent::TabSelected { widget, tab } = event {
                if self.id == Some(*widget)
                    && tabs.iter().any(|candidate| candidate.id == *tab)
                    && *selected != Some(*tab)
                {
                    *selected = Some(*tab);
                    if let Some(message) = on_select {
                        cx.emit(message(*tab));
                    }
                }
            }
            let selected_index = (*selected)
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            if let Some(child) = selected_index.and_then(|index| self.children.get_mut(index)) {
                child.event(cx, event);
            }
            return;
        }

        #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
        if let ViewEvent::DismissPopupOverlays { except } = event {
            let should_dismiss = self.id.is_some() && self.id != *except;
            #[cfg(feature = "combo")]
            if should_dismiss {
                if let ViewNodeKind::ComboBox {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        self.combo_first_visible_option = None;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "time-picker")]
            if should_dismiss {
                if let ViewNodeKind::TimePicker {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
            #[cfg(feature = "date-picker")]
            if should_dismiss {
                if let ViewNodeKind::DatePicker {
                    expanded,
                    on_expanded_change,
                    ..
                } = &mut self.kind
                {
                    if *expanded {
                        *expanded = false;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(false));
                        }
                    }
                }
            }
        }

        #[cfg(feature = "list")]
        if let (
            ViewNodeKind::List {
                selected_index,
                on_select,
            },
            ViewEvent::Click { widget },
        ) = (&mut self.kind, event)
        {
            if let Some(index) = self
                .children
                .iter()
                .position(|child| child.contains_widget(*widget))
            {
                *selected_index = Some(index);
                if let Some(message) = on_select {
                    cx.emit(message(index));
                }
            }
        }

        #[cfg(feature = "radio")]
        if let ViewEvent::RadioSelected { widget } = event {
            let contains_target = self.children.iter().any(|child| {
                child.id == Some(*widget) && matches!(&child.kind, ViewNodeKind::RadioButton { .. })
            });
            if contains_target && matches!(&self.kind, ViewNodeKind::Stack { .. }) {
                for child in &mut self.children {
                    if let ViewNodeKind::RadioButton { selected, .. } = &mut child.kind {
                        *selected = child.id == Some(*widget);
                    }
                }
            }
        }

        #[cfg(feature = "virtual-list")]
        if let (
            ViewNodeKind::VirtualList {
                row_indices,
                selected_index,
                on_select,
                ..
            },
            ViewEvent::Click { widget },
        ) = (&mut self.kind, event)
        {
            if let Some(position) = self
                .children
                .iter()
                .position(|child| child.contains_widget(*widget))
            {
                if let Some(index) = row_indices.get(position).copied() {
                    *selected_index = Some(index);
                    if let Some(message) = on_select {
                        cx.emit(message(index));
                    }
                }
            }
        }

        #[cfg(feature = "combo")]
        if let (
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded,
                on_select,
                on_expanded_change,
                ..
            },
            ViewEvent::ComboBoxSelected { index, .. },
        ) = (&mut self.kind, event)
        {
            if *index < options.len() {
                *selected_index = Some(*index);
                let was_expanded = *expanded;
                *expanded = false;
                self.combo_first_visible_option = None;
                if let Some(message) = on_select {
                    cx.emit(message(*index));
                }
                if was_expanded {
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(false));
                    }
                }
            }
        }

        #[cfg(feature = "combo")]
        if let (
            ViewNodeKind::ComboBox {
                options,
                expanded: true,
                ..
            },
            ViewEvent::ComboBoxScrolled {
                first_visible_index,
                ..
            },
        ) = (&mut self.kind, event)
        {
            self.combo_first_visible_option =
                Some((*first_visible_index).min(options.len().saturating_sub(1)));
        }

        #[cfg(feature = "date-picker")]
        if let (
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                expanded,
                on_date_change,
                on_expanded_change,
                ..
            },
            ViewEvent::DateChanged {
                value: next_value, ..
            },
        ) = (&mut self.kind, event)
        {
            let next_value = (*next_value).clamp(*minimum, *maximum);
            let changed = *value != next_value;
            let was_expanded = *expanded;
            *value = next_value;
            *visible_month = next_value.first_day_of_month();
            *expanded = false;
            if changed {
                if let Some(message) = on_date_change {
                    cx.emit(message(next_value));
                }
            }
            if was_expanded {
                if let Some(message) = on_expanded_change {
                    cx.emit(message(false));
                }
            }
        }

        #[cfg(feature = "time-picker")]
        if let (
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                on_time_change,
                ..
            },
            ViewEvent::TimeChanged {
                widget,
                value: next_value,
            },
        ) = (&mut self.kind, event)
        {
            if self.id == Some(*widget) {
                let next_value = next_value.snap(*minute_increment);
                if *value != next_value {
                    *value = next_value;
                    if let Some(message) = on_time_change {
                        cx.emit(message(next_value));
                    }
                }
            }
        }

        if self.event_targets_self(event) {
            #[cfg(feature = "virtual-list")]
            let list_bounds = self
                .bounds
                .map(|bounds| inset_bounds(bounds, self.style.padding, self.layout_dpi));
            match (&mut self.kind, event) {
                #[cfg(feature = "button")]
                (ViewNodeKind::Button { on_click, .. }, ViewEvent::Click { .. }) => {
                    if let Some(message) = on_click.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "textbox")]
                (
                    ViewNodeKind::Textbox {
                        value, on_change, ..
                    },
                    ViewEvent::TextChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = next_value.clone();
                    if let Some(message) = on_change {
                        cx.emit(message(next_value.clone()));
                    }
                }
                #[cfg(feature = "checkbox")]
                (
                    ViewNodeKind::Checkbox {
                        checked, on_toggle, ..
                    },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "toggle")]
                (
                    ViewNodeKind::Toggle { checked, on_toggle },
                    ViewEvent::Toggled {
                        checked: next_checked,
                        ..
                    },
                ) => {
                    *checked = *next_checked;
                    if let Some(message) = on_toggle {
                        cx.emit(message(*next_checked));
                    }
                }
                #[cfg(feature = "slider")]
                (
                    ViewNodeKind::Slider {
                        value,
                        range,
                        on_slide,
                    },
                    ViewEvent::SliderChanged {
                        value: next_value, ..
                    },
                ) => {
                    *value = range.snap(*next_value);
                    if let Some(message) = on_slide {
                        cx.emit(message(*value));
                    }
                }
                #[cfg(feature = "radio")]
                (
                    ViewNodeKind::RadioButton {
                        selected,
                        on_choose,
                        ..
                    },
                    ViewEvent::RadioSelected { .. },
                ) => {
                    *selected = true;
                    if let Some(message) = on_choose.clone() {
                        cx.emit(message);
                    }
                }
                #[cfg(feature = "combo")]
                (
                    ViewNodeKind::ComboBox {
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::ComboBoxExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    *expanded = *next_expanded;
                    self.combo_first_visible_option = None;
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        value,
                        visible_month,
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::DatePickerExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    *expanded = *next_expanded;
                    if *next_expanded {
                        *visible_month = value.first_day_of_month();
                    }
                    if let Some(message) = on_expanded_change {
                        cx.emit(message(*next_expanded));
                    }
                }
                #[cfg(feature = "date-picker")]
                (
                    ViewNodeKind::DatePicker {
                        minimum,
                        maximum,
                        visible_month,
                        ..
                    },
                    ViewEvent::DatePickerMonthChanged { month, .. },
                ) => {
                    *visible_month = clamp_visible_month(*month, *minimum, *maximum);
                }
                #[cfg(feature = "time-picker")]
                (
                    ViewNodeKind::TimePicker {
                        expanded,
                        on_expanded_change,
                        ..
                    },
                    ViewEvent::TimePickerExpandedChanged {
                        expanded: next_expanded,
                        ..
                    },
                ) => {
                    if *expanded != *next_expanded {
                        *expanded = *next_expanded;
                        if let Some(message) = on_expanded_change {
                            cx.emit(message(*next_expanded));
                        }
                    }
                }
                #[cfg(feature = "scroll")]
                (
                    ViewNodeKind::Scroll {
                        offset_y,
                        content_height,
                        on_scroll,
                    },
                    ViewEvent::ScrollBy { delta_y, .. },
                ) => {
                    let max_offset =
                        scroll_max_offset_y(self.bounds, *content_height, self.layout_dpi);
                    let next = Dp::new((offset_y.0 + delta_y.0).clamp(0.0, max_offset.0));
                    *offset_y = next;
                    if let Some(message) = on_scroll {
                        cx.emit(message(next));
                    }
                }
                #[cfg(feature = "virtual-list")]
                (
                    ViewNodeKind::VirtualList {
                        total_count,
                        row_height,
                        overscan_rows,
                        offset_y,
                        visible_range,
                        materialized_range,
                        on_viewport_changed,
                        ..
                    },
                    ViewEvent::ScrollBy { delta_y, .. },
                ) => {
                    let viewport_height = list_bounds
                        .map(|bounds| {
                            Dp::new(
                                bounds.height.max(0) as f32
                                    / self.layout_dpi.scale_factor().max(f32::EPSILON),
                            )
                        })
                        .unwrap_or(Dp::new(0.0));
                    let requested = Dp::new(offset_y.0 + delta_y.0);
                    let direction = if requested.0 > offset_y.0 {
                        VirtualListScrollDirection::Forward
                    } else if requested.0 < offset_y.0 {
                        VirtualListScrollDirection::Backward
                    } else {
                        VirtualListScrollDirection::Stationary
                    };
                    let viewport = virtual_list_viewport(
                        *total_count,
                        *row_height,
                        requested,
                        viewport_height,
                        *overscan_rows,
                        direction,
                    );
                    *offset_y = viewport.offset_y;
                    *visible_range = viewport.visible_range;
                    *materialized_range = viewport.materialized_range;
                    if let Some(message) = on_viewport_changed {
                        cx.emit(message(viewport));
                    }
                }
                _ => {}
            }
        }

        for child in &mut self.children {
            child.event(cx, event);
        }
    }

    fn paint(&self, cx: &mut ViewPaintCx) {
        let Some(bounds) = self.bounds else {
            return;
        };
        cx.paint_depth = cx.paint_depth.saturating_add(1);

        if let Some(theme_mode) = self.style.theme_mode {
            cx.set_theme_mode(theme_mode);
        }

        if let Some(background) = self.style.background {
            let fill = NativeDrawFill::Role(color_role_for_token(background));
            let radius = radius_px(self.style.radius, cx.dpi);
            if radius == 0 {
                cx.draw(NativeDrawCommand::FillRect { rect: bounds, fill });
            } else {
                cx.draw(NativeDrawCommand::RoundFill {
                    rect: bounds,
                    fill,
                    radius,
                });
            }
        }

        match &self.kind {
            #[cfg(feature = "label")]
            ViewNodeKind::Text { text } => {
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    text,
                    padded_bounds(bounds, self.style.padding, cx.dpi),
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "button")]
            ViewNodeKind::Button { label, .. } => {
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Control),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    radius: radius_px(self.style.radius.or(Some(Dp::new(6.0))), cx.dpi),
                });
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    padded_bounds(bounds, self.style.padding.or(Some(Dp::new(8.0))), cx.dpi),
                    SemanticTextStyle {
                        role: TextRole::Button,
                        ..SemanticTextStyle::body()
                    },
                )));
            }
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox {
                value, multiline, ..
            } => {
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: bounds,
                    fill: NativeDrawFill::Role(ColorRole::Surface),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Control)),
                    radius: radius_px(self.style.radius.or(Some(Dp::new(6.0))), cx.dpi),
                });
                let mut text_style = SemanticTextStyle::body();
                if *multiline {
                    text_style.vertical_align = crate::VerticalAlign::Start;
                    text_style.wrap = crate::TextWrap::Word;
                    text_style.ellipsis = false;
                }
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    value,
                    padded_bounds(bounds, self.style.padding.or(Some(Dp::new(8.0))), cx.dpi),
                    text_style,
                )));
            }
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { label, checked, .. } => {
                let check_bounds = Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.height.min(20),
                    height: bounds.height.min(20),
                };
                cx.draw(NativeDrawCommand::RoundRect {
                    rect: check_bounds,
                    fill: NativeDrawFill::Role(if *checked {
                        ColorRole::Accent
                    } else {
                        ColorRole::Control
                    }),
                    stroke: Some(NativeDrawFill::Role(ColorRole::Accent)),
                    radius: 4,
                });
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    Rect {
                        x: bounds.x + check_bounds.width + 8,
                        y: bounds.y,
                        width: (bounds.width - check_bounds.width - 8).max(0),
                        height: bounds.height,
                    },
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { checked, .. } => {
                let plan = crate::zs_toggle_render_plan(bounds, false, *checked, cx.dpi);
                for command in crate::zs_toggle_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                ..
            } => {
                let plan = crate::zs_time_picker_render_plan(
                    bounds,
                    *value,
                    *minute_increment,
                    *clock,
                    false,
                    ZsTimePickerPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_time_picker_header_native_draw_plan(&plan, *value).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                ..
            } => {
                let plan = crate::zs_date_picker_render_plan_with_today(
                    bounds,
                    *value,
                    *visible_month,
                    *minimum,
                    *maximum,
                    *today,
                    false,
                    cx.dpi,
                );
                for command in crate::zs_date_picker_header_native_draw_plan(&plan, *value).commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "slider")]
            ViewNodeKind::Slider { value, range, .. } => {
                let plan = crate::zs_slider_render_plan(bounds, range.fraction(*value), cx.dpi);
                for command in crate::zs_slider_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "radio")]
            ViewNodeKind::RadioButton {
                label, selected, ..
            } => {
                let plan = crate::zs_radio_render_plan(bounds, *selected, cx.dpi);
                for command in crate::zs_radio_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
                let gap = Dp::new(8.0).to_px(cx.dpi).round_i32().max(0);
                let label_x = plan
                    .indicator
                    .x
                    .saturating_add(plan.indicator.width)
                    .saturating_add(gap);
                cx.draw(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    Rect {
                        x: label_x,
                        y: bounds.y,
                        width: bounds
                            .x
                            .saturating_add(bounds.width)
                            .saturating_sub(label_x)
                            .max(0),
                        height: bounds.height,
                    },
                    SemanticTextStyle::body(),
                )));
            }
            #[cfg(feature = "progress")]
            ViewNodeKind::ProgressBar { value, range } => {
                let plan =
                    crate::zs_progress_bar_render_plan(bounds, range.fraction(*value), cx.dpi);
                for command in crate::zs_progress_bar_native_draw_plan(&plan).commands {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                placeholder,
                ..
            } => {
                let plan = crate::zs_combo_box_render_plan(bounds, options.len(), false, cx.dpi);
                let selected_text = selected_index
                    .and_then(|index| options.get(index))
                    .map(String::as_str);
                for command in crate::zs_combo_box_header_native_draw_plan(
                    &plan,
                    selected_text,
                    placeholder.as_deref(),
                )
                .commands
                {
                    cx.draw(command);
                }
            }
            #[cfg(feature = "tabs")]
            ViewNodeKind::Tabs { tabs, selected, .. } => {
                let labels = tabs.iter().map(|tab| tab.label.clone()).collect::<Vec<_>>();
                let selected_index = selected.and_then(|selected| {
                    tabs.iter().position(|candidate| candidate.id == selected)
                });
                let plan = crate::zs_tab_view_render_plan(
                    bounds,
                    &labels,
                    selected_index,
                    crate::ZsTabPlatformStyle::current(),
                    cx.dpi,
                );
                for command in crate::zs_tab_view_native_draw_plan(&plan, &labels).commands {
                    cx.draw(command);
                }
                if let Some(child) = selected_index.and_then(|index| self.children.get(index)) {
                    cx.draw(NativeDrawCommand::PushClip {
                        rect: plan.content_bounds,
                    });
                    child.paint(cx);
                    cx.draw(NativeDrawCommand::PopClip);
                }
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "list")]
            ViewNodeKind::List { selected_index, .. } => {
                if let Some(bounds) = selected_index
                    .and_then(|index| self.children.get(index))
                    .and_then(ViewNode::bounds)
                {
                    cx.draw(NativeDrawCommand::RoundFill {
                        rect: bounds,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Accent,
                            alpha: 36,
                        },
                        radius: radius_px(self.style.radius.or(Some(Dp::new(4.0))), cx.dpi),
                    });
                }
            }
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList {
                row_height,
                row_indices,
                selected_index,
                offset_y,
                visible_range,
                show_placeholders,
                ..
            } => {
                cx.draw(NativeDrawCommand::PushClip { rect: bounds });
                if let Some(selected_bounds) = selected_index
                    .and_then(|index| row_indices.binary_search(&index).ok())
                    .and_then(|position| self.children.get(position))
                    .and_then(ViewNode::bounds)
                {
                    cx.draw(NativeDrawCommand::RoundFill {
                        rect: selected_bounds,
                        fill: NativeDrawFill::RoleWithAlpha {
                            role: ColorRole::Accent,
                            alpha: 36,
                        },
                        radius: radius_px(self.style.radius.or(Some(Dp::new(4.0))), cx.dpi),
                    });
                }
                if *show_placeholders {
                    let content_bounds = inset_bounds(bounds, self.style.padding, cx.dpi);
                    for index in visible_range.start..visible_range.end {
                        if row_indices.binary_search(&index).is_ok() {
                            continue;
                        }
                        let row_bounds = virtual_list_row_bounds(
                            content_bounds,
                            index,
                            *row_height,
                            *offset_y,
                            cx.dpi,
                        );
                        let inset_x = 8.min(row_bounds.width / 4).max(0);
                        let inset_y = 6.min(row_bounds.height / 4).max(0);
                        let placeholder = Rect {
                            x: row_bounds.x + inset_x,
                            y: row_bounds.y + inset_y,
                            width: (row_bounds.width - inset_x * 2).max(0),
                            height: (row_bounds.height - inset_y * 2).max(0),
                        };
                        if placeholder.width > 0 && placeholder.height > 0 {
                            cx.draw(NativeDrawCommand::RoundFill {
                                rect: placeholder,
                                fill: NativeDrawFill::RoleWithAlpha {
                                    role: ColorRole::Control,
                                    alpha: 96,
                                },
                                radius: 4,
                            });
                        }
                    }
                }
                for child in &self.children {
                    child.paint(cx);
                }
                cx.draw(NativeDrawCommand::PopClip);
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => {
                cx.draw(NativeDrawCommand::PushClip { rect: bounds });
                for child in &self.children {
                    child.paint(cx);
                }
                cx.draw(NativeDrawCommand::PopClip);
                cx.finish_node(self);
                return;
            }
            #[cfg(feature = "grid")]
            ViewNodeKind::Grid { .. } => {}
            ViewNodeKind::Stack { .. } | ViewNodeKind::Spacer | ViewNodeKind::__Message(_) => {}
        }

        for child in &self.children {
            child.paint(cx);
        }
        cx.finish_node(self);
    }
}

impl<Msg> ViewNode<Msg> {
    fn event_targets_self(&self, event: &ViewEvent) -> bool {
        match (self.id, event) {
            (Some(id), ViewEvent::Click { widget })
            | (Some(id), ViewEvent::TextChanged { widget, .. })
            | (Some(id), ViewEvent::Toggled { widget, .. }) => id == *widget,
            #[cfg(feature = "slider")]
            (Some(id), ViewEvent::SliderChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "radio")]
            (Some(id), ViewEvent::RadioSelected { widget }) => id == *widget,
            #[cfg(feature = "combo")]
            (Some(id), ViewEvent::ComboBoxExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::ComboBoxSelected { widget, .. })
            | (Some(id), ViewEvent::ComboBoxScrolled { widget, .. }) => id == *widget,
            #[cfg(feature = "date-picker")]
            (Some(id), ViewEvent::DatePickerExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::DatePickerMonthChanged { widget, .. })
            | (Some(id), ViewEvent::DateChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "time-picker")]
            (Some(id), ViewEvent::TimePickerExpandedChanged { widget, .. })
            | (Some(id), ViewEvent::TimeChanged { widget, .. }) => id == *widget,
            #[cfg(feature = "tabs")]
            (Some(id), ViewEvent::TabSelected { widget, .. }) => id == *widget,
            #[cfg(feature = "scroll")]
            (Some(id), ViewEvent::ScrollBy { widget, .. }) => id == *widget,
            #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
            (Some(_), ViewEvent::DismissPopupOverlays { .. }) => false,
            (None, _) => false,
        }
    }

    #[cfg(any(feature = "list", feature = "scroll", feature = "tabs"))]
    fn contains_widget(&self, widget: WidgetId) -> bool {
        self.id == Some(widget)
            || self
                .children
                .iter()
                .any(|child| child.contains_widget(widget))
    }

    pub fn interaction_plan(&self) -> ViewInteractionPlan {
        let mut hit_targets = Vec::new();
        self.collect_hit_targets(&mut hit_targets, None);
        #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
        self.collect_overlay_hit_targets(&mut hit_targets, None);
        ViewInteractionPlan { hit_targets }
    }

    pub fn widget_text_value(&self, widget: WidgetId) -> Option<&str> {
        if self.id == Some(widget) {
            #[cfg(feature = "textbox")]
            if let ViewNodeKind::Textbox { value, .. } = &self.kind {
                return Some(value);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_text_value(widget))
    }

    pub fn widget_checked_value(&self, widget: WidgetId) -> Option<bool> {
        if self.id == Some(widget) {
            #[cfg(feature = "checkbox")]
            if let ViewNodeKind::Checkbox { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "toggle")]
            if let ViewNodeKind::Toggle { checked, .. } = &self.kind {
                return Some(*checked);
            }
            #[cfg(feature = "radio")]
            if let ViewNodeKind::RadioButton { selected, .. } = &self.kind {
                return Some(*selected);
            }
        }

        self.children
            .iter()
            .find_map(|child| child.widget_checked_value(widget))
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_is_tab_stop(&self, widget: WidgetId) -> Option<bool> {
        if matches!(&self.kind, ViewNodeKind::Stack { .. }) {
            let mut radio_widgets = self.children.iter().filter_map(|child| {
                if let ViewNodeKind::RadioButton { selected, .. } = &child.kind {
                    child.id.map(|id| (id, *selected))
                } else {
                    None
                }
            });
            if let Some(first) = radio_widgets.next() {
                let mut group = vec![first];
                group.extend(radio_widgets);
                if group.iter().any(|(candidate, _)| *candidate == widget) {
                    let tab_stop = group
                        .iter()
                        .find_map(|(candidate, selected)| selected.then_some(*candidate))
                        .unwrap_or(first.0);
                    return Some(widget == tab_stop);
                }
            }
        }
        if self.id == Some(widget) && matches!(&self.kind, ViewNodeKind::RadioButton { .. }) {
            return Some(true);
        }
        self.children
            .iter()
            .find_map(|child| child.widget_radio_is_tab_stop(widget))
    }

    #[cfg(feature = "radio")]
    pub(crate) fn widget_radio_relative_widget(
        &self,
        widget: WidgetId,
        navigation: ViewStackDirection,
        offset: isize,
    ) -> Option<WidgetId> {
        if let ViewNodeKind::Stack { direction } = &self.kind {
            let navigation_supported = match direction {
                ViewStackDirection::Column => navigation == ViewStackDirection::Column,
                ViewStackDirection::Row => true,
            };
            let radio_widgets = self
                .children
                .iter()
                .filter_map(|child| {
                    matches!(&child.kind, ViewNodeKind::RadioButton { .. })
                        .then_some(child.id)
                        .flatten()
                })
                .collect::<Vec<_>>();
            if let Some(index) = radio_widgets
                .iter()
                .position(|candidate| *candidate == widget)
            {
                if navigation_supported {
                    let next_index = index as isize + offset;
                    return Some(
                        usize::try_from(next_index)
                            .ok()
                            .and_then(|index| radio_widgets.get(index).copied())
                            .unwrap_or(widget),
                    );
                }
                return Some(widget);
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_radio_relative_widget(widget, navigation, offset))
    }

    #[cfg(feature = "slider")]
    pub fn widget_slider_state(&self, widget: WidgetId) -> Option<(f32, SliderRange)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Slider { value, range, .. } = &self.kind {
                return Some((*value, *range));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_slider_state(widget))
    }

    #[cfg(feature = "combo")]
    pub fn widget_combo_state(&self, widget: WidgetId) -> Option<(Option<usize>, usize, bool)> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded,
                ..
            } = &self.kind
            {
                return Some((*selected_index, options.len(), *expanded));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_combo_state(widget))
    }

    #[cfg(feature = "combo")]
    pub(crate) fn widget_combo_type_ahead_match(
        &self,
        widget: WidgetId,
        query: &str,
        start_after: Option<usize>,
    ) -> Option<usize> {
        if self.id == Some(widget) {
            if let ViewNodeKind::ComboBox { options, .. } = &self.kind {
                if query.is_empty() || options.is_empty() {
                    return None;
                }
                let query = query.to_lowercase();
                let start = start_after
                    .filter(|index| *index < options.len())
                    .map_or(0, |index| (index + 1) % options.len());
                return (0..options.len())
                    .map(|offset| (start + offset) % options.len())
                    .find(|index| options[*index].to_lowercase().starts_with(&query));
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_combo_type_ahead_match(widget, query, start_after))
    }

    #[cfg(feature = "date-picker")]
    pub fn widget_date_picker_state(&self, widget: WidgetId) -> Option<ZsDatePickerState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                expanded,
                ..
            } = &self.kind
            {
                return Some(ZsDatePickerState {
                    value: *value,
                    minimum: *minimum,
                    maximum: *maximum,
                    visible_month: *visible_month,
                    expanded: *expanded,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_date_picker_state(widget))
    }

    #[cfg(feature = "time-picker")]
    pub fn widget_time_picker_state(&self, widget: WidgetId) -> Option<ZsTimePickerState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded,
                ..
            } = &self.kind
            {
                return Some(ZsTimePickerState {
                    value: *value,
                    minute_increment: *minute_increment,
                    clock: *clock,
                    expanded: *expanded,
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_time_picker_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub fn widget_tab_view_state(&self, widget: WidgetId) -> Option<ZsTabViewState> {
        if self.id == Some(widget) {
            if let ViewNodeKind::Tabs { tabs, selected, .. } = &self.kind {
                return Some(ZsTabViewState {
                    selected: *selected,
                    tab_count: tabs.len(),
                });
            }
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_view_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_header_state(&self, widget: WidgetId) -> Option<ZsTabHeaderState> {
        if let (Some(tab_view), ViewNodeKind::Tabs { tabs, selected, .. }) = (self.id, &self.kind) {
            if let Some((index, tab)) = tabs
                .iter()
                .enumerate()
                .find(|(_, tab)| WidgetId(tab.id.0) == widget)
            {
                return Some(ZsTabHeaderState {
                    tab_view,
                    tab: tab.id,
                    selected: *selected == Some(tab.id),
                    previous: index
                        .checked_sub(1)
                        .and_then(|index| tabs.get(index))
                        .map(|tab| WidgetId(tab.id.0)),
                    next: tabs
                        .get(index.saturating_add(1))
                        .map(|tab| WidgetId(tab.id.0)),
                    first: WidgetId(tabs.first().expect("matched tab list is non-empty").id.0),
                    last: WidgetId(tabs.last().expect("matched tab list is non-empty").id.0),
                });
            }
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            return selected_index
                .and_then(|index| self.children.get(index))
                .and_then(|child| child.widget_tab_header_state(widget));
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_header_state(widget))
    }

    #[cfg(feature = "tabs")]
    pub(crate) fn widget_tab_cycle_target(
        &self,
        focused: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, ZsTabId)> {
        if let (Some(tab_view), ViewNodeKind::Tabs { tabs, selected, .. }) = (self.id, &self.kind) {
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            let focused_header = tabs.iter().any(|tab| WidgetId(tab.id.0) == focused);
            let focused_content = selected_index
                .and_then(|index| self.children.get(index))
                .is_some_and(|child| child.contains_widget(focused));
            if (focused_header || focused_content) && !tabs.is_empty() {
                let current = selected_index.unwrap_or(0) as isize;
                let next = (current + offset).rem_euclid(tabs.len() as isize) as usize;
                return Some((tab_view, tabs[next].id));
            }
            return selected_index
                .and_then(|index| self.children.get(index))
                .and_then(|child| child.widget_tab_cycle_target(focused, offset));
        }
        self.children
            .iter()
            .find_map(|child| child.widget_tab_cycle_target(focused, offset))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_index(&self, widget: WidgetId) -> Option<usize> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            return self
                .children
                .iter()
                .position(|child| child.contains_widget(widget));
        }
        #[cfg(feature = "virtual-list")]
        if let ViewNodeKind::VirtualList { row_indices, .. } = &self.kind {
            let position = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            return row_indices.get(position).copied();
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_index(widget))
    }

    #[cfg(feature = "list")]
    pub fn widget_list_relative_widget(
        &self,
        widget: WidgetId,
        offset: isize,
    ) -> Option<(WidgetId, usize)> {
        if matches!(self.kind, ViewNodeKind::List { .. }) {
            let current = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            let next = current
                .saturating_add_signed(offset)
                .min(self.children.len().saturating_sub(1));
            if next == current {
                return None;
            }
            return self.children[next]
                .first_widget_id()
                .map(|widget| (widget, next));
        }
        #[cfg(feature = "virtual-list")]
        if let ViewNodeKind::VirtualList { row_indices, .. } = &self.kind {
            let current = self
                .children
                .iter()
                .position(|child| child.contains_widget(widget))?;
            let next = current
                .saturating_add_signed(offset)
                .min(self.children.len().saturating_sub(1));
            if next == current {
                return None;
            }
            let index = *row_indices.get(next)?;
            return self.children[next]
                .first_widget_id()
                .map(|widget| (widget, index));
        }

        self.children
            .iter()
            .find_map(|child| child.widget_list_relative_widget(widget, offset))
    }

    #[cfg(feature = "list")]
    fn first_widget_id(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id))
    }

    #[cfg(feature = "scroll")]
    pub fn widget_scroll_target(&self, widget: WidgetId) -> Option<WidgetId> {
        let is_scroll_target = matches!(self.kind, ViewNodeKind::Scroll { .. });
        #[cfg(feature = "virtual-list")]
        let is_scroll_target =
            is_scroll_target || matches!(self.kind, ViewNodeKind::VirtualList { .. });
        if is_scroll_target && self.contains_widget(widget) {
            return self.id.or_else(|| self.first_widget_id_any());
        }

        self.children
            .iter()
            .find_map(|child| child.widget_scroll_target(widget))
    }

    #[cfg(feature = "scroll")]
    fn first_widget_id_any(&self) -> Option<WidgetId> {
        self.id
            .or_else(|| self.children.iter().find_map(ViewNode::first_widget_id_any))
    }

    fn collect_hit_targets(&self, hit_targets: &mut Vec<ViewHitTarget>, clip: Option<Rect>) {
        #[cfg(feature = "tabs")]
        if let (Some(tab_view), Some(bounds), ViewNodeKind::Tabs { tabs, selected, .. }) =
            (self.id, self.bounds, &self.kind)
        {
            let labels = tabs.iter().map(|tab| tab.label.clone()).collect::<Vec<_>>();
            let selected_index = selected
                .and_then(|selected| tabs.iter().position(|candidate| candidate.id == selected));
            let plan = crate::zs_tab_view_render_plan(
                bounds,
                &labels,
                selected_index,
                crate::ZsTabPlatformStyle::current(),
                self.layout_dpi,
            );
            hit_targets.extend(plan.headers.iter().zip(tabs).filter_map(|(header, tab)| {
                clipped_rect(header.bounds, clip).map(|bounds| {
                    ViewHitTarget::with_kind(
                        WidgetId(tab.id.0),
                        bounds,
                        ViewHitTargetKind::Tab {
                            tab_view,
                            tab: tab.id,
                            index: tabs
                                .iter()
                                .position(|candidate| candidate.id == tab.id)
                                .unwrap_or(0),
                        },
                    )
                })
            }));
            if let Some(child) = selected_index.and_then(|index| self.children.get(index)) {
                child.collect_hit_targets(hit_targets, clipped_rect(plan.content_bounds, clip));
            }
            return;
        }

        #[cfg(feature = "progress")]
        let accepts_input = !matches!(self.kind, ViewNodeKind::ProgressBar { .. });
        #[cfg(not(feature = "progress"))]
        let accepts_input = true;
        if accepts_input {
            if let (Some(widget), Some(bounds)) = (self.id, self.bounds) {
                if let Some(bounds) = clipped_rect(bounds, clip) {
                    hit_targets.push(ViewHitTarget::with_kind(
                        widget,
                        bounds,
                        self.hit_target_kind(),
                    ));
                }
            }
        }

        #[cfg(feature = "scroll")]
        let clips_children = matches!(self.kind, ViewNodeKind::Scroll { .. });
        #[cfg(all(feature = "scroll", feature = "virtual-list"))]
        let clips_children =
            clips_children || matches!(self.kind, ViewNodeKind::VirtualList { .. });
        #[cfg(feature = "scroll")]
        let child_clip = if clips_children {
            self.bounds.and_then(|bounds| clipped_rect(bounds, clip))
        } else {
            clip
        };
        #[cfg(not(feature = "scroll"))]
        let child_clip = clip;

        for child in &self.children {
            child.collect_hit_targets(hit_targets, child_clip);
        }
    }

    #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
    fn collect_overlay_hit_targets(
        &self,
        hit_targets: &mut Vec<ViewHitTarget>,
        viewport: Option<Rect>,
    ) {
        #[cfg(feature = "combo")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_combo_box_render_plan_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_combo_box_render_plan_in_viewport_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            hit_targets.extend(
                plan.option_rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, bounds)| {
                        ViewHitTarget::with_kind(
                            widget,
                            bounds,
                            ViewHitTargetKind::ComboBoxOption {
                                index: plan.first_visible_option.saturating_add(index),
                            },
                        )
                    }),
            );
        }
        #[cfg(feature = "date-picker")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_date_picker_render_plan_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_date_picker_render_plan_in_viewport_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            if let Some(bounds) = plan.previous_button {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::DatePickerPreviousMonth,
                ));
            }
            if let Some(bounds) = plan.next_button {
                hit_targets.push(ViewHitTarget::with_kind(
                    widget,
                    bounds,
                    ViewHitTargetKind::DatePickerNextMonth,
                ));
            }
            hit_targets.extend(plan.day_cells.into_iter().filter(|cell| cell.enabled).map(
                |cell| {
                    ViewHitTarget::with_kind(
                        widget,
                        cell.bounds,
                        ViewHitTargetKind::DatePickerDay { date: cell.date },
                    )
                },
            ));
        }
        #[cfg(feature = "time-picker")]
        if let (
            Some(widget),
            Some(bounds),
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded: true,
                ..
            },
        ) = (self.id, self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_time_picker_render_plan(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        self.layout_dpi,
                    )
                },
                |viewport| {
                    crate::zs_time_picker_render_plan_in_viewport(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        self.layout_dpi,
                        viewport,
                    )
                },
            );
            hit_targets.extend(plan.choices.into_iter().map(|choice| {
                ViewHitTarget::with_kind(
                    widget,
                    choice.bounds,
                    ViewHitTargetKind::TimePickerChoice {
                        value: choice.value,
                    },
                )
            }));
        }
        let child_viewport = viewport.or(self.bounds);
        for child in &self.children {
            child.collect_overlay_hit_targets(hit_targets, child_viewport);
        }
    }

    #[cfg(any(feature = "combo", feature = "date-picker", feature = "time-picker"))]
    fn paint_overlays(&self, cx: &mut ViewPaintCx, viewport: Option<Rect>) {
        #[cfg(feature = "combo")]
        if let (
            Some(bounds),
            ViewNodeKind::ComboBox {
                options,
                selected_index,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_combo_box_render_plan_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_combo_box_render_plan_in_viewport_with_scroll(
                        bounds,
                        options.len(),
                        *selected_index,
                        self.combo_first_visible_option,
                        true,
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in
                crate::zs_combo_box_popup_native_draw_plan(&plan, options, *selected_index, cx.dpi)
                    .commands
            {
                cx.draw(command);
            }
        }
        #[cfg(feature = "date-picker")]
        if let (
            Some(bounds),
            ViewNodeKind::DatePicker {
                value,
                minimum,
                maximum,
                visible_month,
                today,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_date_picker_render_plan_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_date_picker_render_plan_in_viewport_with_today(
                        bounds,
                        *value,
                        *visible_month,
                        *minimum,
                        *maximum,
                        *today,
                        true,
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in
                crate::zs_date_picker_popup_native_draw_plan(&plan, *visible_month, cx.dpi).commands
            {
                cx.draw(command);
            }
        }
        #[cfg(feature = "time-picker")]
        if let (
            Some(bounds),
            ViewNodeKind::TimePicker {
                value,
                minute_increment,
                clock,
                expanded: true,
                ..
            },
        ) = (self.bounds, &self.kind)
        {
            let plan = viewport.map_or_else(
                || {
                    crate::zs_time_picker_render_plan(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        cx.dpi,
                    )
                },
                |viewport| {
                    crate::zs_time_picker_render_plan_in_viewport(
                        bounds,
                        *value,
                        *minute_increment,
                        *clock,
                        true,
                        ZsTimePickerPlatformStyle::current(),
                        cx.dpi,
                        viewport,
                    )
                },
            );
            for command in crate::zs_time_picker_popup_native_draw_plan(&plan).commands {
                cx.draw(command);
            }
        }
        let child_viewport = viewport.or(self.bounds);
        for child in &self.children {
            child.paint_overlays(cx, child_viewport);
        }
    }

    fn hit_target_kind(&self) -> ViewHitTargetKind {
        match &self.kind {
            #[cfg(feature = "button")]
            ViewNodeKind::Button { .. } => ViewHitTargetKind::Button,
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { multiline, .. } => {
                if *multiline {
                    ViewHitTargetKind::TextEditor
                } else {
                    ViewHitTargetKind::Textbox
                }
            }
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { .. } => ViewHitTargetKind::Checkbox,
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { .. } => ViewHitTargetKind::Toggle,
            #[cfg(feature = "slider")]
            ViewNodeKind::Slider { .. } => ViewHitTargetKind::Slider,
            #[cfg(feature = "radio")]
            ViewNodeKind::RadioButton { .. } => ViewHitTargetKind::RadioButton,
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { .. } => ViewHitTargetKind::ComboBox,
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker { .. } => ViewHitTargetKind::DatePicker,
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker { .. } => ViewHitTargetKind::TimePicker,
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => ViewHitTargetKind::Scroll,
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { .. } => ViewHitTargetKind::Scroll,
            _ => ViewHitTargetKind::Unknown,
        }
    }
}

#[cfg(feature = "virtual-list")]
pub fn virtual_list_viewport(
    total_count: usize,
    row_height: Dp,
    offset_y: Dp,
    viewport_height: Dp,
    overscan_rows: usize,
    direction: VirtualListScrollDirection,
) -> VirtualListViewport {
    let row_height = if row_height.0.is_finite() {
        row_height.0.max(1.0)
    } else {
        1.0
    };
    let viewport_height = if viewport_height.0.is_finite() {
        viewport_height.0.max(0.0)
    } else {
        0.0
    };
    let requested_offset = if offset_y.0.is_finite() {
        offset_y.0.max(0.0)
    } else {
        0.0
    };
    let content_height = total_count as f64 * row_height as f64;
    let max_offset = (content_height - viewport_height as f64).max(0.0) as f32;
    let offset_y = requested_offset.min(max_offset);
    if total_count == 0 || viewport_height <= 0.0 {
        return VirtualListViewport {
            offset_y: Dp::new(offset_y),
            row_height: Dp::new(row_height),
            visible_range: VirtualListRange::new(0, 0),
            materialized_range: VirtualListRange::new(0, 0),
            direction,
        };
    }

    let start = ((offset_y / row_height).floor() as usize).min(total_count);
    let end = (((offset_y + viewport_height) / row_height).ceil() as usize)
        .max(start.saturating_add(1))
        .min(total_count);
    let visible_range = VirtualListRange::new(start, end);
    let materialized_range = VirtualListRange::new(
        start.saturating_sub(overscan_rows),
        end.saturating_add(overscan_rows).min(total_count),
    );
    VirtualListViewport {
        offset_y: Dp::new(offset_y),
        row_height: Dp::new(row_height),
        visible_range,
        materialized_range,
        direction,
    }
}

#[cfg(feature = "virtual-list")]
fn virtual_list_row_bounds(
    bounds: Rect,
    index: usize,
    row_height: Dp,
    offset_y: Dp,
    dpi: Dpi,
) -> Rect {
    let row_height_px = row_height.to_px(dpi).round_i32().max(1);
    let offset_px = offset_y.to_px(dpi).round_i32().max(0);
    let row_top = (index as i64)
        .saturating_mul(row_height_px as i64)
        .saturating_sub(offset_px as i64)
        .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Rect {
        x: bounds.x,
        y: bounds.y.saturating_add(row_top),
        width: bounds.width,
        height: row_height_px,
    }
}

fn split_child_bounds<Msg>(
    bounds: Rect,
    kind: &ViewNodeKind<Msg>,
    children: &[ViewNode<Msg>],
    gap: Option<Dp>,
    dpi: Dpi,
) -> Vec<Rect> {
    let child_count = children.len();
    if child_count == 0 {
        return Vec::new();
    }
    let gap = gap
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);

    match kind {
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Row,
        } => {
            let widths =
                allocate_axis_lengths(bounds.width, gap, children, |style| style.width, dpi);
            let mut x = bounds.x;
            widths
                .into_iter()
                .map(|width| {
                    let rect = Rect {
                        x,
                        y: bounds.y,
                        width,
                        height: bounds.height,
                    };
                    x += width + gap;
                    rect
                })
                .collect()
        }
        ViewNodeKind::Stack {
            direction: ViewStackDirection::Column,
        } => split_column_child_bounds(bounds, children, gap, dpi),
        #[cfg(feature = "list")]
        ViewNodeKind::List { .. } => split_column_child_bounds(bounds, children, gap, dpi),
        #[cfg(feature = "scroll")]
        ViewNodeKind::Scroll {
            offset_y,
            content_height,
            ..
        } => {
            let offset_y = offset_y.to_px(dpi).round_i32().max(0);
            let height = content_height
                .map(|height| height.to_px(dpi).round_i32())
                .unwrap_or(bounds.height)
                .max(bounds.height);
            vec![
                Rect {
                    x: bounds.x,
                    y: bounds.y - offset_y,
                    width: bounds.width,
                    height,
                };
                child_count
            ]
        }
        #[cfg(feature = "grid")]
        ViewNodeKind::Grid {
            columns,
            rows,
            placements,
            column_gap,
            row_gap,
        } => split_grid_child_bounds(
            bounds,
            columns,
            rows,
            placements,
            children.len(),
            column_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            row_gap
                .map(|value| value.to_px(dpi).round_i32().max(0))
                .unwrap_or(gap),
            dpi,
        ),
        _ => vec![bounds; child_count],
    }
}

#[cfg(feature = "grid")]
fn split_grid_child_bounds(
    bounds: Rect,
    columns: &[ZsGridTrack],
    rows: &[ZsGridTrack],
    placements: &[ZsGridPlacement],
    child_count: usize,
    column_gap: i32,
    row_gap: i32,
    dpi: Dpi,
) -> Vec<Rect> {
    let column_lengths = allocate_grid_track_lengths(bounds.width, column_gap, columns, dpi);
    let row_lengths = allocate_grid_track_lengths(bounds.height, row_gap, rows, dpi);
    (0..child_count)
        .map(|index| {
            placements
                .get(index)
                .and_then(|placement| {
                    grid_placement_bounds(
                        bounds,
                        &column_lengths,
                        &row_lengths,
                        column_gap,
                        row_gap,
                        *placement,
                    )
                })
                .unwrap_or(Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: 0,
                    height: 0,
                })
        })
        .collect()
}

#[cfg(feature = "grid")]
fn allocate_grid_track_lengths(total: i32, gap: i32, tracks: &[ZsGridTrack], dpi: Dpi) -> Vec<i32> {
    if tracks.is_empty() {
        return Vec::new();
    }
    let total = total.max(0);
    let total_gap = gap
        .max(0)
        .saturating_mul(tracks.len().saturating_sub(1) as i32)
        .min(total);
    let available = total.saturating_sub(total_gap);
    let requested = tracks
        .iter()
        .map(|track| match track {
            ZsGridTrack::Fixed(size) => Some(size.to_px(dpi).round_i32().max(0)),
            ZsGridTrack::Fraction(_) => None,
        })
        .collect::<Vec<_>>();
    let fixed_total = requested
        .iter()
        .flatten()
        .fold(0i32, |total, value| total.saturating_add(*value));
    let mut lengths = vec![0; tracks.len()];

    if fixed_total >= available && fixed_total > 0 {
        let fixed_indices = requested
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.map(|value| (index, value)))
            .collect::<Vec<_>>();
        let mut assigned = 0;
        for (position, (index, value)) in fixed_indices.iter().enumerate() {
            let length = if position + 1 == fixed_indices.len() {
                available.saturating_sub(assigned)
            } else {
                ((i64::from(*value) * i64::from(available)) / i64::from(fixed_total))
                    .clamp(0, i64::from(i32::MAX)) as i32
            };
            lengths[*index] = length;
            assigned = assigned.saturating_add(length);
        }
        return lengths;
    }

    for (index, value) in requested.iter().enumerate() {
        if let Some(value) = value {
            lengths[index] = *value;
        }
    }

    let fractional_indices = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| match track {
            ZsGridTrack::Fraction(weight) => Some((index, weight.get())),
            ZsGridTrack::Fixed(_) => None,
        })
        .collect::<Vec<_>>();
    if fractional_indices.is_empty() {
        return lengths;
    }
    let remaining = available.saturating_sub(fixed_total).max(0);
    let total_weight = fractional_indices.iter().fold(0u64, |total, (_, weight)| {
        total.saturating_add(u64::from(*weight))
    });
    let mut assigned = 0;
    for (position, (index, weight)) in fractional_indices.iter().enumerate() {
        let length = if position + 1 == fractional_indices.len() {
            remaining.saturating_sub(assigned)
        } else {
            ((remaining as u128 * u128::from(*weight)) / u128::from(total_weight))
                .min(i32::MAX as u128) as i32
        };
        lengths[*index] = length;
        assigned = assigned.saturating_add(length);
    }
    lengths
}

#[cfg(feature = "grid")]
fn grid_placement_bounds(
    bounds: Rect,
    column_lengths: &[i32],
    row_lengths: &[i32],
    column_gap: i32,
    row_gap: i32,
    placement: ZsGridPlacement,
) -> Option<Rect> {
    let (x, width) = grid_axis_span(
        bounds.x,
        column_lengths,
        column_gap,
        placement.column,
        placement.column_span,
    )?;
    let (y, height) = grid_axis_span(
        bounds.y,
        row_lengths,
        row_gap,
        placement.row,
        placement.row_span,
    )?;
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

#[cfg(feature = "grid")]
fn grid_axis_span(
    origin: i32,
    lengths: &[i32],
    gap: i32,
    start: usize,
    span: ZsGridSpan,
) -> Option<(i32, i32)> {
    if start >= lengths.len() {
        return None;
    }
    let gap = gap.max(0);
    let before = lengths[..start]
        .iter()
        .fold(0i32, |total, length| total.saturating_add(*length))
        .saturating_add(gap.saturating_mul(start as i32));
    let end = start
        .saturating_add(usize::from(span.get()))
        .min(lengths.len());
    let covered_tracks = end.saturating_sub(start);
    let length = lengths[start..end]
        .iter()
        .fold(0i32, |total, length| total.saturating_add(*length))
        .saturating_add(gap.saturating_mul(covered_tracks.saturating_sub(1) as i32));
    Some((origin.saturating_add(before), length.max(0)))
}

fn split_column_child_bounds<Msg>(
    bounds: Rect,
    children: &[ViewNode<Msg>],
    gap: i32,
    dpi: Dpi,
) -> Vec<Rect> {
    let heights = allocate_axis_lengths(bounds.height, gap, children, |style| style.height, dpi);
    let mut y = bounds.y;
    heights
        .into_iter()
        .map(|height| {
            let rect = Rect {
                x: bounds.x,
                y,
                width: bounds.width,
                height,
            };
            y += height + gap;
            rect
        })
        .collect()
}

fn allocate_axis_lengths<Msg>(
    total: i32,
    gap: i32,
    children: &[ViewNode<Msg>],
    fixed: impl Fn(&ViewStyle) -> Option<Dp>,
    dpi: Dpi,
) -> Vec<i32> {
    let total = total.max(0);
    let total_gap = gap
        .saturating_mul(children.len().saturating_sub(1) as i32)
        .min(total);
    let available = total - total_gap;
    let requested = children
        .iter()
        .map(|child| fixed(&child.style).map(|value| value.to_px(dpi).round_i32().max(0)))
        .collect::<Vec<_>>();
    let fixed_total: i32 = requested.iter().flatten().copied().sum();
    let mut lengths = vec![0; children.len()];

    if fixed_total >= available && fixed_total > 0 {
        let scale = available as f32 / fixed_total as f32;
        let fixed_indices = requested
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.map(|value| (index, value)))
            .collect::<Vec<_>>();
        let mut assigned = 0;
        for (position, (index, value)) in fixed_indices.iter().enumerate() {
            let length = if position + 1 == fixed_indices.len() {
                available - assigned
            } else {
                ((*value as f32) * scale).floor() as i32
            }
            .max(0);
            lengths[*index] = length;
            assigned += length;
        }
        return lengths;
    }

    for (index, value) in requested.iter().enumerate() {
        if let Some(value) = value {
            lengths[index] = *value;
        }
    }

    let remaining = (available - fixed_total).max(0);
    let flexible = requested
        .iter()
        .enumerate()
        .filter(|(_, value)| value.is_none())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if flexible.is_empty() {
        return lengths;
    }

    let flex_total: f32 = flexible
        .iter()
        .map(|index| children[*index].style.flex.max(0.0))
        .sum();
    let equal_flex = flex_total <= f32::EPSILON;
    let denominator = if equal_flex {
        flexible.len() as f32
    } else {
        flex_total
    };
    let mut assigned = 0;
    for (position, index) in flexible.iter().enumerate() {
        let length = if position + 1 == flexible.len() {
            remaining - assigned
        } else {
            let weight = if equal_flex {
                1.0
            } else {
                children[*index].style.flex.max(0.0)
            };
            ((remaining as f32) * weight / denominator).floor() as i32
        }
        .max(0);
        lengths[*index] = length;
        assigned += length;
    }
    lengths
}

#[cfg(feature = "date-picker")]
fn clamp_visible_month(month: ZsDate, minimum: ZsDate, maximum: ZsDate) -> ZsDate {
    let (minimum, maximum) = if minimum <= maximum {
        (minimum, maximum)
    } else {
        (maximum, minimum)
    };
    month
        .first_day_of_month()
        .max(minimum.first_day_of_month())
        .min(maximum.first_day_of_month())
}

fn inset_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    let padding = padding
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0);
    Rect {
        x: bounds.x + padding,
        y: bounds.y + padding,
        width: (bounds.width - padding * 2).max(0),
        height: (bounds.height - padding * 2).max(0),
    }
}

#[cfg(any(feature = "label", feature = "button", feature = "textbox"))]
fn padded_bounds(bounds: Rect, padding: Option<Dp>, dpi: Dpi) -> Rect {
    inset_bounds(bounds, padding, dpi)
}

fn radius_px(radius: Option<Dp>, dpi: Dpi) -> i32 {
    radius
        .map(|value| value.to_px(dpi).round_i32().max(0))
        .unwrap_or(0)
}

#[cfg(feature = "scroll")]
fn scroll_max_offset_y(bounds: Option<Rect>, content_height: Option<Dp>, dpi: Dpi) -> Dp {
    let viewport_px = bounds
        .map(|bounds| bounds.height.max(0) as f32)
        .unwrap_or(0.0);
    let content_px = content_height
        .map(|height| height.to_px(dpi).0.max(0.0))
        .unwrap_or(viewport_px);
    let scale = (dpi.0 / Dpi::standard().0).max(f32::EPSILON);
    Dp::new(((content_px - viewport_px) / scale).max(0.0))
}

fn color_role_for_token(token: ThemeColorToken) -> ColorRole {
    match token {
        ThemeColorToken::Surface => ColorRole::Surface,
        ThemeColorToken::SurfaceRaised => ColorRole::SurfaceRaised,
        ThemeColorToken::TextPrimary => ColorRole::PrimaryText,
        ThemeColorToken::TextSecondary => ColorRole::SecondaryText,
        ThemeColorToken::Accent => ColorRole::Accent,
        ThemeColorToken::AccentText => ColorRole::AccentText,
        ThemeColorToken::Control => ColorRole::Control,
        ThemeColorToken::Border => ColorRole::Border,
        ThemeColorToken::Success => ColorRole::Success,
        ThemeColorToken::Warning => ColorRole::Warning,
        ThemeColorToken::Danger => ColorRole::Danger,
    }
}

fn clipped_rect(rect: Rect, clip: Option<Rect>) -> Option<Rect> {
    let Some(clip) = clip else {
        return Some(rect);
    };
    let left = rect.x.max(clip.x);
    let top = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    let width = right - left;
    let height = bottom - top;
    (width > 0 && height > 0).then_some(Rect {
        x: left,
        y: top,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        feature = "button",
        feature = "textbox",
        feature = "checkbox",
        feature = "toggle",
        feature = "slider",
        feature = "radio",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker",
        feature = "tabs",
        feature = "list"
    ))]
    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        #[cfg(feature = "button")]
        SaveClicked,
        #[cfg(feature = "textbox")]
        NameChanged(String),
        #[cfg(any(feature = "checkbox", feature = "toggle"))]
        DarkModeChanged(bool),
        #[cfg(feature = "slider")]
        VolumeChanged(f32),
        #[cfg(feature = "radio")]
        ChoiceSelected(&'static str),
        #[cfg(feature = "combo")]
        ComboSelected(usize),
        #[cfg(feature = "combo")]
        ComboExpanded(bool),
        #[cfg(feature = "date-picker")]
        DateChanged(ZsDate),
        #[cfg(feature = "date-picker")]
        DateExpanded(bool),
        #[cfg(feature = "time-picker")]
        TimeChanged(ZsTime),
        #[cfg(feature = "time-picker")]
        TimeExpanded(bool),
        #[cfg(feature = "tabs")]
        TabSelected(ZsTabId),
        #[cfg(feature = "list")]
        RowSelected(usize),
        #[cfg(feature = "scroll")]
        ScrollChanged(Dp),
        #[cfg(feature = "virtual-list")]
        ViewportChanged(VirtualListViewport),
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_uses_typed_messages_without_string_events() {
        let save_id = WidgetId::new(1);
        let mut view = column(vec![
            text("Clipboard history"),
            button("Save")
                .id(save_id)
                .padding(Dp::new(12.0))
                .radius(Dp::new(8.0))
                .on_click(Msg::SaveClicked),
        ]);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: save_id });

        assert_eq!(events.into_messages(), vec![Msg::SaveClicked]);
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_node_layout_and_paint_emit_native_draw_plan() {
        let mut view: ViewNode<Msg> =
            column(vec![text("Title"), button("Copy").radius(Dp::new(8.0))]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert_eq!(output.bounds.width, 240);
        assert_eq!(paint.plan().text_count(), 2);
        assert!(paint.plan().command_count() >= 3);
    }

    #[test]
    fn stack_layout_honors_fixed_size_flex_and_gap() {
        let navigation = WidgetId::new(70);
        let content = WidgetId::new(71);
        let mut view: ViewNode<()> = row(vec![
            spacer().id(navigation).width(Dp::new(240.0)),
            spacer().id(content).flex(1.0),
        ])
        .gap(Dp::new(12.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 960,
                height: 640,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let navigation_bounds = output
            .children
            .iter()
            .find(|node| node.component == navigation.into())
            .unwrap()
            .bounds;
        let content_bounds = output
            .children
            .iter()
            .find(|node| node.component == content.into())
            .unwrap()
            .bounds;

        assert_eq!(navigation_bounds.width, 240);
        assert_eq!(content_bounds.x, 252);
        assert_eq!(content_bounds.width, 708);
    }

    #[test]
    #[cfg(feature = "grid")]
    fn grid_layout_honors_typed_tracks_spans_and_independent_gaps() {
        let header = WidgetId::new(72);
        let content = WidgetId::new(73);
        let action = WidgetId::new(74);
        let mut view: ViewNode<()> = grid(
            [
                ZsGridTrack::fixed(Dp::new(120.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fraction(ZsGridFraction::TWO),
            ],
            [
                ZsGridTrack::fixed(Dp::new(40.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fixed(Dp::new(60.0)),
            ],
            [
                ZsGridCell::new(0, 0, spacer().id(header)).column_span(ZsGridSpan::THREE),
                ZsGridCell::new(1, 1, spacer().id(content)).column_span(ZsGridSpan::TWO),
                ZsGridCell::new(2, 2, spacer().id(action)),
            ],
        )
        .column_gap(Dp::new(10.0))
        .row_gap(Dp::new(8.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 600,
                height: 300,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let bounds_for = |widget: WidgetId| {
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("grid child should be laid out")
                .bounds
        };

        assert_eq!(
            bounds_for(header),
            Rect {
                x: 0,
                y: 0,
                width: 600,
                height: 40,
            }
        );
        assert_eq!(
            bounds_for(content),
            Rect {
                x: 130,
                y: 48,
                width: 470,
                height: 184,
            }
        );
        assert_eq!(
            bounds_for(action),
            Rect {
                x: 293,
                y: 240,
                width: 307,
                height: 60,
            }
        );
    }

    #[test]
    #[cfg(feature = "grid")]
    fn grid_explicit_placement_scales_fixed_tracks_with_dpi_and_bounds_invalid_cells() {
        let first = WidgetId::new(75);
        let second = WidgetId::new(76);
        let explicit = WidgetId::new(77);
        let invalid = WidgetId::new(79);
        let mut view: ViewNode<()> = grid(
            [ZsGridTrack::fixed(Dp::new(40.0)), ZsGridTrack::FLEX],
            [ZsGridTrack::FLEX, ZsGridTrack::FLEX],
            [
                ZsGridCell::new(1, 0, spacer().id(first)),
                ZsGridCell::new(1, 1, spacer().id(second)),
                ZsGridCell::new(0, 0, spacer().id(explicit)).column_span(ZsGridSpan::TWO),
                ZsGridCell::new(4, 4, spacer().id(invalid)),
            ],
        )
        .gap(Dp::new(8.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 180,
            },
            Dpi::new(144.0),
        );

        let output = view.layout(&mut layout);
        let bounds_for = |widget: WidgetId| {
            output
                .children
                .iter()
                .find(|node| node.component == widget.into())
                .expect("grid child should be laid out")
                .bounds
        };

        assert_eq!(
            bounds_for(explicit),
            Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 84,
            }
        );
        assert_eq!(
            bounds_for(first),
            Rect {
                x: 10,
                y: 116,
                width: 60,
                height: 84,
            }
        );
        assert_eq!(bounds_for(second).width, 228);
        assert_eq!(bounds_for(second).x, 82);
        assert_eq!(bounds_for(invalid).width, 0);
        assert_eq!(bounds_for(invalid).height, 0);
        assert!(ZsGridFraction::new(0).is_err());
        assert!(ZsGridSpan::new(0).is_err());
    }

    #[test]
    #[cfg(all(feature = "grid", feature = "button"))]
    fn grid_layout_drives_shared_paint_and_typed_hit_geometry() {
        let behind = WidgetId::new(80);
        let action = WidgetId::new(78);
        let mut view: ViewNode<Msg> = grid(
            [
                ZsGridTrack::FLEX,
                ZsGridTrack::fraction(ZsGridFraction::TWO),
            ],
            [ZsGridTrack::FLEX, ZsGridTrack::FLEX],
            [
                ZsGridCell::new(0, 0, spacer()),
                ZsGridCell::new(1, 1, button("Behind").id(behind).on_click(Msg::SaveClicked)),
                ZsGridCell::new(1, 1, button("Apply").id(action).on_click(Msg::SaveClicked)),
            ],
        )
        .gap(Dp::new(6.0));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 306,
                height: 206,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let interaction = view.interaction_plan();
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert_eq!(
            interaction.click_event_at(Point { x: 205, y: 155 }),
            Some(ViewEvent::Click { widget: action })
        );
        assert_eq!(interaction.hit_target_count(), 2);
        assert!(paint.plan().text_count() >= 2);
    }

    #[test]
    fn square_background_uses_full_rect_fill() {
        let mut view: ViewNode<()> = spacer().bg(ThemeColorToken::Surface);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 80,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.paint(&mut paint);

        assert!(matches!(
            paint.plan().commands.first(),
            Some(NativeDrawCommand::FillRect { .. })
        ));
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn view_interaction_plan_maps_points_to_typed_click_events() {
        let save_id = WidgetId::new(42);
        let mut view: ViewNode<Msg> = column(vec![
            text("Title"),
            button("Save").id(save_id).on_click(Msg::SaveClicked),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 120,
            },
            Dpi::standard(),
        );
        let _output = view.layout(&mut layout);
        let plan = view.interaction_plan();

        assert_eq!(plan.hit_target_count(), 1);
        assert_eq!(
            plan.target_kind_at(Point { x: 150, y: 90 }),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.hit_target_for_widget(save_id)
                .map(|target| target.kind),
            Some(ViewHitTargetKind::Button)
        );
        assert_eq!(
            plan.click_event_at(Point { x: 150, y: 90 }),
            Some(ViewEvent::Click { widget: save_id })
        );
        assert_eq!(
            plan.first_focus_target().map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(
            plan.next_focus_target(None, 1).map(|target| target.widget),
            Some(save_id)
        );
        assert_eq!(plan.click_event_at(Point { x: 150, y: 20 }), None);
    }

    #[test]
    #[cfg(all(feature = "textbox", feature = "checkbox"))]
    fn input_views_map_runtime_values_into_typed_messages() {
        let name_id = WidgetId::new(2);
        let dark_id = WidgetId::new(3);
        let mut view = column(vec![
            textbox("").id(name_id).on_change(Msg::NameChanged),
            checkbox("Dark mode", false)
                .id(dark_id)
                .on_toggle(Msg::DarkModeChanged),
        ]);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: name_id,
                value: "ZSUI".to_string(),
            },
        );
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: dark_id,
                checked: true,
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![
                Msg::NameChanged("ZSUI".to_string()),
                Msg::DarkModeChanged(true)
            ]
        );
    }

    #[test]
    #[cfg(all(feature = "textbox", not(feature = "checkbox")))]
    fn textbox_maps_runtime_value_without_other_input_features() {
        let name_id = WidgetId::new(2);
        let mut view = textbox("").id(name_id).on_change(Msg::NameChanged);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::TextChanged {
                widget: name_id,
                value: "ZSUI".to_string(),
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![Msg::NameChanged("ZSUI".to_string())]
        );
    }

    #[test]
    #[cfg(feature = "textbox")]
    fn text_editor_is_a_multiline_focus_target_with_wrapped_text() {
        let editor_id = WidgetId::new(5);
        let mut view = text_editor::<Msg>("first\nsecond").id(editor_id);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 180,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::TextEditor);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text)
                if text.style.wrap == crate::TextWrap::Word
                    && text.style.vertical_align == crate::VerticalAlign::Start
                    && !text.style.ellipsis
        )));
    }

    #[test]
    #[cfg(feature = "toggle")]
    fn toggle_routes_typed_state_and_paints_shared_geometry() {
        let toggle_id = WidgetId::new(4);
        let mut view = toggle(false).id(toggle_id).on_toggle(Msg::DarkModeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 48,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::Toggled {
                widget: toggle_id,
                checked: true,
            },
        );

        assert_eq!(view.widget_checked_value(toggle_id), Some(true));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::Toggle);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(events.into_messages(), vec![Msg::DarkModeChanged(true)]);
    }

    #[test]
    #[cfg(feature = "slider")]
    fn slider_clamps_snaps_routes_typed_value_and_paints_shared_geometry() {
        let slider_id = WidgetId::new(6);
        let range = SliderRange::new(0.0, 10.0).step(0.5);
        let mut view = slider(12.0, range)
            .id(slider_id)
            .on_slide(Msg::VolumeChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::SliderChanged {
                widget: slider_id,
                value: 4.74,
            },
        );

        assert_eq!(view.widget_slider_state(slider_id), Some((4.5, range)));
        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::Slider);
        assert_eq!(paint.plan().command_count(), 3);
        assert_eq!(events.into_messages(), vec![Msg::VolumeChanged(4.5)]);
        assert_eq!(range.value_at_fraction(0.26), 2.5);
        assert_eq!(range.offset_steps(4.5, 1), 5.0);

        let uneven = SliderRange::new(0.0, 1.0).step(0.3);
        assert_eq!(uneven.value_at_fraction(1.0), 1.0);
        assert_eq!(uneven.offset_steps(0.9, 1), 1.0);
    }

    #[test]
    #[cfg(feature = "radio")]
    fn radio_button_routes_typed_choice_and_paints_selected_state() {
        let radio_id = WidgetId::new(7);
        let mut view = radio_button("Balanced", false)
            .id(radio_id)
            .on_choose(Msg::ChoiceSelected("balanced"));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();
        view.event(&mut events, &ViewEvent::RadioSelected { widget: radio_id });
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(view.hit_target_kind(), ViewHitTargetKind::RadioButton);
        assert_eq!(paint.plan().command_count(), 3);
        assert_eq!(
            events.into_messages(),
            vec![Msg::ChoiceSelected("balanced")]
        );
        assert!(matches!(
            view.kind,
            ViewNodeKind::RadioButton { selected: true, .. }
        ));
    }

    #[test]
    #[cfg(feature = "radio")]
    fn radio_groups_enforce_single_selection_and_non_wrapping_directional_navigation() {
        let first = WidgetId::new(71);
        let second = WidgetId::new(72);
        let third = WidgetId::new(73);
        let mut vertical = column([
            radio_button("Balanced", true)
                .id(first)
                .on_choose(Msg::ChoiceSelected("balanced")),
            radio_button("Performance", false)
                .id(second)
                .on_choose(Msg::ChoiceSelected("performance")),
            radio_button("Quiet", false)
                .id(third)
                .on_choose(Msg::ChoiceSelected("quiet")),
        ]);

        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Column, -1),
            Some(first)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Column, 1),
            Some(second)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(first, ViewStackDirection::Row, 1),
            Some(first)
        );
        assert_eq!(
            vertical.widget_radio_relative_widget(third, ViewStackDirection::Column, 1),
            Some(third)
        );
        assert_eq!(vertical.widget_radio_is_tab_stop(first), Some(true));
        assert_eq!(vertical.widget_radio_is_tab_stop(second), Some(false));

        let mut events = ViewEventCx::new();
        vertical.event(&mut events, &ViewEvent::RadioSelected { widget: second });
        assert_eq!(vertical.widget_checked_value(first), Some(false));
        assert_eq!(vertical.widget_checked_value(second), Some(true));
        assert_eq!(vertical.widget_checked_value(third), Some(false));
        assert_eq!(vertical.widget_radio_is_tab_stop(first), Some(false));
        assert_eq!(vertical.widget_radio_is_tab_stop(second), Some(true));
        assert_eq!(
            events.into_messages(),
            vec![Msg::ChoiceSelected("performance")]
        );

        let horizontal = row([
            radio_button::<()>("One", true).id(first),
            radio_button::<()>("Two", false).id(second),
            radio_button::<()>("Three", false).id(third),
        ]);
        assert_eq!(
            horizontal.widget_radio_relative_widget(second, ViewStackDirection::Row, -1),
            Some(first)
        );
        assert_eq!(
            horizontal.widget_radio_relative_widget(second, ViewStackDirection::Column, 1),
            Some(third)
        );
    }

    #[test]
    #[cfg(feature = "progress")]
    fn progress_bar_normalizes_range_clamps_state_and_paints_fraction() {
        let range = ProgressRange::new(100.0, 0.0);
        let mut view = progress_bar::<()>(125.0, range).id(WidgetId::new(8));
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 32,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(range.min(), 0.0);
        assert_eq!(range.max(), 100.0);
        assert_eq!(range.fraction(25.0), 0.25);
        assert_eq!(paint.plan().command_count(), 2);
        assert_eq!(view.interaction_plan().hit_target_count(), 0);
        assert!(matches!(
            view.kind,
            ViewNodeKind::ProgressBar { value: 100.0, .. }
        ));
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_routes_overlay_selection_and_paints_above_later_siblings() {
        let combo_id = WidgetId::new(9);
        let mut view = column([
            combo_box(["Balanced", "Fast", "Quiet"], Some(0))
                .id(combo_id)
                .height(Dp::new(36.0))
                .expanded(true)
                .on_select(Msg::ComboSelected)
                .on_expanded_change(Msg::ComboExpanded),
            spacer().bg(ThemeColorToken::Control),
        ]);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 160,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let interaction = view.interaction_plan();
        let option = interaction
            .hit_targets
            .iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .copied()
            .expect("expanded option should be in the overlay hit plan");
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::ComboBox)
        );
        assert_eq!(
            interaction.target_kind_at(Point {
                x: option.bounds.x + 8,
                y: option.bounds.y + option.bounds.height / 2,
            }),
            Some(ViewHitTargetKind::ComboBoxOption { index: 1 })
        );

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ComboBoxSelected {
                widget: combo_id,
                index: 1,
            },
        );
        assert_eq!(
            events.into_messages(),
            vec![Msg::ComboSelected(1), Msg::ComboExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo_id), Some((Some(1), 3, false)));

        let mut expanded = combo_box::<_, ()>(["One", "Two"], Some(0))
            .id(combo_id)
            .expanded(true);
        expanded.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 36,
            },
            Dpi::standard(),
        ));
        let mut paint = ViewPaintCx::new(Dpi::standard());
        expanded.paint(&mut paint);
        assert!(matches!(
            paint.plan().commands.last(),
            Some(NativeDrawCommand::Text(text)) if text.text == "Two"
        ));
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_rejects_out_of_range_initial_selection() {
        let view = combo_box::<_, ()>(["One"], Some(7)).id(WidgetId::new(10));
        assert_eq!(
            view.widget_combo_state(WidgetId::new(10)),
            Some((None, 1, false))
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_scrolls_a_bounded_popup_with_global_option_indices() {
        let combo_id = WidgetId::new(91);
        let options = (0..30)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let mut view = column([
            combo_box::<_, ()>(options, Some(0))
                .id(combo_id)
                .height(Dp::new(36.0))
                .expanded(true),
            spacer(),
        ]);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 200,
            },
            Dpi::standard(),
        ));

        let initial_plan = view.interaction_plan();
        let initial_range = initial_plan
            .combo_visible_option_range(combo_id)
            .expect("expanded long combo should expose visible options");
        assert_eq!(initial_range.start, 0);
        assert!(initial_range.len() < 30);

        let mut events = ViewEventCx::new();
        view.event(
            &mut events,
            &ViewEvent::ComboBoxScrolled {
                widget: combo_id,
                first_visible_index: 1,
            },
        );
        assert!(events.into_messages().is_empty());

        let scrolled_plan = view.interaction_plan();
        let scrolled_range = scrolled_plan
            .combo_visible_option_range(combo_id)
            .expect("scrolled combo should retain visible options");
        assert_eq!(scrolled_range.start, 1);
        assert_eq!(scrolled_range.len(), initial_range.len());
        let first_row = scrolled_plan
            .hit_targets
            .iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .expect("first scrolled row should keep its global option index");
        assert_eq!(
            scrolled_plan.target_kind_at(Point {
                x: first_row.bounds.x + 8,
                y: first_row.bounds.y + first_row.bounds.height / 2,
            }),
            Some(ViewHitTargetKind::ComboBoxOption { index: 1 })
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_type_ahead_match_wraps_after_selection() {
        let widget = WidgetId::new(12);
        let view = combo_box::<_, ()>(["Quartz", "Quiet", "Balanced"], Some(2)).id(widget);

        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "Q", Some(2)),
            Some(0)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "qu", Some(2)),
            Some(0)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "qui", Some(2)),
            Some(1)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "b", Some(1)),
            Some(2)
        );
        assert_eq!(
            view.widget_combo_type_ahead_match(widget, "missing", Some(2)),
            None
        );
    }

    #[test]
    #[cfg(feature = "combo")]
    fn combo_box_overlay_paint_and_hits_share_viewport_flipped_geometry() {
        let widget = WidgetId::new(11);
        let mut view = column([
            spacer(),
            combo_box::<_, ()>(["One", "Two", "Three"], None)
                .id(widget)
                .height(Dp::new(32.0))
                .expanded(true),
        ])
        .gap(Dp::new(4.0));
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 300,
                height: 240,
            },
            Dpi::standard(),
        ));

        let option = view
            .interaction_plan()
            .hit_targets
            .into_iter()
            .find(|target| target.kind == ViewHitTargetKind::ComboBoxOption { index: 1 })
            .expect("second option should be hittable in the flipped popup");
        assert_eq!(option.bounds.y, 140);

        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundRect { rect, .. }
                if *rect == Rect { x: 0, y: 108, width: 300, height: 96 }
        )));
    }

    #[test]
    #[cfg(feature = "date-picker")]
    fn date_picker_routes_typed_range_month_and_overlay_selection() {
        let widget = WidgetId::new(12);
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let minimum = ZsDate::new(2026, 7, 10).unwrap();
        let maximum = ZsDate::new(2026, 8, 20).unwrap();
        let mut view = date_picker(value)
            .id(widget)
            .height(Dp::new(32.0))
            .date_range(minimum, maximum)
            .today(ZsDate::new(2026, 7, 14).unwrap())
            .expanded(true)
            .on_date_change(Msg::DateChanged)
            .on_expanded_change(Msg::DateExpanded);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 24,
                y: 64,
                width: 472,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        let next_day = ZsDate::new(2026, 7, 14).unwrap();
        assert!(interaction
            .hit_targets
            .iter()
            .any(|target| { target.kind == ViewHitTargetKind::DatePickerDay { date: next_day } }));
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::DatePicker)
        );
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert_eq!(
            paint
                .plan()
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    NativeDrawCommand::RoundRect {
                        fill: NativeDrawFill::Role(ColorRole::Accent),
                        ..
                    }
                ))
                .count(),
            2
        );

        let mut month_events = ViewEventCx::new();
        view.event(
            &mut month_events,
            &ViewEvent::DatePickerMonthChanged {
                widget,
                month: ZsDate::new(2026, 8, 1).unwrap(),
            },
        );
        assert!(month_events.messages().is_empty());
        assert_eq!(
            view.widget_date_picker_state(widget)
                .expect("date picker state")
                .visible_month,
            ZsDate::new(2026, 8, 1).unwrap()
        );

        let mut selection_events = ViewEventCx::new();
        view.event(
            &mut selection_events,
            &ViewEvent::DateChanged {
                widget,
                value: ZsDate::new(2026, 8, 31).unwrap(),
            },
        );
        assert_eq!(
            selection_events.into_messages(),
            vec![Msg::DateChanged(maximum), Msg::DateExpanded(false)]
        );
        assert_eq!(
            view.widget_date_picker_state(widget),
            Some(ZsDatePickerState {
                value: maximum,
                minimum,
                maximum,
                visible_month: maximum.first_day_of_month(),
                expanded: false,
            })
        );
    }

    #[test]
    #[cfg(feature = "time-picker")]
    fn time_picker_routes_typed_increment_popup_and_selection() {
        let widget = WidgetId::new(13);
        let initial = ZsTime::new(18, 15).unwrap();
        let selected = ZsTime::new(18, 30).unwrap();
        let mut view = time_picker(initial)
            .id(widget)
            .height(Dp::new(32.0))
            .minute_increment(ZsMinuteIncrement::FIFTEEN)
            .clock_format(ZsClockFormat::TwentyFourHour)
            .expanded(true)
            .on_time_change(Msg::TimeChanged)
            .on_expanded_change(Msg::TimeExpanded);
        view.layout(&mut ViewLayoutCx::new(
            Rect {
                x: 24,
                y: 180,
                width: 240,
                height: 32,
            },
            Dpi::standard(),
        ));

        let interaction = view.interaction_plan();
        assert!(interaction.hit_targets.iter().any(|target| {
            target.kind == ViewHitTargetKind::TimePickerChoice { value: selected }
        }));
        assert_eq!(
            interaction.first_focus_target().map(|target| target.kind),
            Some(ViewHitTargetKind::TimePicker)
        );
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Icon(crate::NativeDrawIconCommand {
                icon: crate::ZsIcon::ChevronDown,
                ..
            })
        )));

        let mut selection_events = ViewEventCx::new();
        view.event(
            &mut selection_events,
            &ViewEvent::TimeChanged {
                widget,
                value: selected,
            },
        );
        assert_eq!(
            selection_events.into_messages(),
            vec![Msg::TimeChanged(selected)]
        );
        assert_eq!(
            view.widget_time_picker_state(widget),
            Some(ZsTimePickerState {
                value: selected,
                minute_increment: ZsMinuteIncrement::FIFTEEN,
                clock: ZsClockFormat::TwentyFourHour,
                expanded: true,
            })
        );

        let mut expanded_events = ViewEventCx::new();
        view.event(
            &mut expanded_events,
            &ViewEvent::TimePickerExpandedChanged {
                widget,
                expanded: false,
            },
        );
        assert_eq!(
            expanded_events.into_messages(),
            vec![Msg::TimeExpanded(false)]
        );
        assert_eq!(
            view.widget_time_picker_state(widget)
                .map(|state| state.expanded),
            Some(false)
        );
    }

    #[test]
    #[cfg(all(feature = "combo", feature = "date-picker"))]
    fn dismiss_popup_overlays_closes_every_expanded_control_except_the_owner() {
        let combo = WidgetId::new(90);
        let date = WidgetId::new(91);
        let value = ZsDate::new(2026, 7, 13).unwrap();
        let mut view = column([
            combo_box(["One", "Two"], Some(0))
                .id(combo)
                .expanded(true)
                .on_expanded_change(Msg::ComboExpanded),
            date_picker(value)
                .id(date)
                .expanded(true)
                .on_expanded_change(Msg::DateExpanded),
        ]);

        let mut date_dismissed = ViewEventCx::new();
        view.event(
            &mut date_dismissed,
            &ViewEvent::DismissPopupOverlays {
                except: Some(combo),
            },
        );
        assert_eq!(
            date_dismissed.into_messages(),
            vec![Msg::DateExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo), Some((Some(0), 2, true)));
        assert_eq!(
            view.widget_date_picker_state(date)
                .map(|state| state.expanded),
            Some(false)
        );

        let mut all_dismissed = ViewEventCx::new();
        view.event(
            &mut all_dismissed,
            &ViewEvent::DismissPopupOverlays { except: None },
        );
        assert_eq!(
            all_dismissed.into_messages(),
            vec![Msg::ComboExpanded(false)]
        );
        assert_eq!(view.widget_combo_state(combo), Some((Some(0), 2, false)));
    }

    #[test]
    #[cfg(all(feature = "list", feature = "label"))]
    fn list_view_routes_child_clicks_to_typed_selection_messages() {
        let first = WidgetId::new(10);
        let second = WidgetId::new(11);
        let mut view = list([(first, "One"), (second, "Two")], |(id, label)| {
            text(label).id(id)
        })
        .selected_index(Some(0))
        .on_select(Msg::RowSelected);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: second });

        assert_eq!(events.into_messages(), vec![Msg::RowSelected(1)]);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::RoundFill { .. })));
        assert_eq!(view.widget_list_index(second), Some(1));
        assert_eq!(
            view.widget_list_relative_widget(second, -1),
            Some((first, 0))
        );
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn scroll_view_offsets_children_and_clips_hit_targets() {
        let top = WidgetId::new(20);
        let bottom = WidgetId::new(21);
        let scroll_id = WidgetId::new(22);
        let mut view: ViewNode<Msg> = scroll(column([
            text("Top row").id(top),
            text("Bottom row").id(bottom),
        ]))
        .id(scroll_id)
        .content_height(Dp::new(120.0))
        .scroll_y(Dp::new(60.0))
        .on_scroll(Msg::ScrollChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 60,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let plan = view.interaction_plan();
        let mut events = ViewEventCx::new();
        let mut paint = ViewPaintCx::new(Dpi::standard());

        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: scroll_id,
                delta_y: Dp::new(-20.0),
            },
        );
        view.paint(&mut paint);

        assert_eq!(
            events.into_messages(),
            vec![Msg::ScrollChanged(Dp::new(40.0))]
        );
        assert_eq!(plan.target_at(Point { x: 20, y: 20 }), Some(bottom));
        assert_eq!(plan.hit_target_for_widget(top), None);
        assert_eq!(view.widget_scroll_target(bottom), Some(scroll_id));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PushClip { .. })));
        assert!(paint
            .plan()
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PopClip)));
    }

    #[test]
    #[cfg(all(feature = "scroll", feature = "label"))]
    fn scroll_boundary_converts_viewport_pixels_at_high_dpi() {
        let scroll_id = WidgetId::new(23);
        let mut view: ViewNode<Msg> = scroll(text("High DPI content"))
            .id(scroll_id)
            .content_height(Dp::new(240.0))
            .scroll_y(Dp::new(170.0))
            .on_scroll(Msg::ScrollChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 120,
            },
            Dpi::new(192.0),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: scroll_id,
                delta_y: Dp::new(20.0),
            },
        );

        assert_eq!(
            events.into_messages(),
            vec![Msg::ScrollChanged(Dp::new(180.0))]
        );
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn virtual_list_layout_and_paint_only_touch_the_materialized_window() {
        let list_id = WidgetId::new(600);
        let mut view = virtual_list(
            100_000,
            (490..520).map(|index| (index, format!("Row {index}"))),
            |index, label| text(label).id(WidgetId::new(1_000 + index as u64)),
        )
        .id(list_id)
        .height(Dp::new(100.0))
        .item_height(Dp::new(20.0))
        .overscan_rows(2)
        .scroll_y(Dp::new(10_000.0))
        .on_viewport_changed(Msg::ViewportChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 100,
            },
            Dpi::standard(),
        );

        let output = view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);

        assert_eq!(output.children.len(), 10);
        assert_eq!(view.interaction_plan().hit_target_count(), 6);
        assert_eq!(
            paint
                .plan()
                .commands
                .iter()
                .filter(|command| matches!(command, NativeDrawCommand::Text(_)))
                .count(),
            9
        );
        assert!(view.children[0].bounds().is_none());
        assert!(view.children[8].bounds().is_some());
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn virtual_list_scroll_emits_global_range_and_global_selection() {
        let list_id = WidgetId::new(700);
        let row_id = WidgetId::new(711);
        let mut view = virtual_list(100, [(11, "Eleven"), (12, "Twelve")], |index, label| {
            text(label).id(if index == 11 {
                row_id
            } else {
                WidgetId::new(712)
            })
        })
        .id(list_id)
        .item_height(Dp::new(20.0))
        .overscan_rows(1)
        .scroll_y(Dp::new(200.0))
        .on_select(Msg::RowSelected)
        .on_viewport_changed(Msg::ViewportChanged);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 60,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);
        let mut events = ViewEventCx::new();

        view.event(&mut events, &ViewEvent::Click { widget: row_id });
        view.event(
            &mut events,
            &ViewEvent::ScrollBy {
                widget: list_id,
                delta_y: Dp::new(20.0),
            },
        );

        assert_eq!(events.messages()[0], Msg::RowSelected(11));
        assert!(matches!(
            events.messages()[1],
            Msg::ViewportChanged(VirtualListViewport {
                visible_range: VirtualListRange { start: 11, end: 14 },
                materialized_range: VirtualListRange { start: 10, end: 15 },
                direction: VirtualListScrollDirection::Forward,
                ..
            })
        ));
        assert_eq!(view.widget_list_index(row_id), Some(11));
    }

    #[test]
    #[cfg(feature = "virtual-list")]
    fn virtual_list_viewport_clamps_large_offsets_without_iterating_items() {
        let viewport = virtual_list_viewport(
            100_000,
            Dp::new(24.0),
            Dp::new(f32::MAX),
            Dp::new(240.0),
            4,
            VirtualListScrollDirection::Forward,
        );

        assert_eq!(
            viewport.visible_range,
            VirtualListRange::new(99_990, 100_000)
        );
        assert_eq!(
            viewport.materialized_range,
            VirtualListRange::new(99_986, 100_000)
        );
        assert_eq!(viewport.offset_y, Dp::new(2_399_760.0));
    }

    #[test]
    #[cfg(all(feature = "virtual-list", feature = "label"))]
    fn live_view_background_poll_stops_after_loaded_state_is_refreshed() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };

        let loading = Arc::new(AtomicBool::new(true));
        let view_loading = Arc::clone(&loading);
        let runtime = live_view_runtime(
            (),
            move |_| {
                virtual_list(1, [(0, "Loaded")], |_, value| text(value))
                    .loading(view_loading.load(Ordering::SeqCst))
            },
            |_, _: (), _| {},
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 80,
            },
            Dpi::standard(),
        );

        assert_eq!(runtime.background_poll_interval_ms(), Some(33));
        loading.store(false, Ordering::SeqCst);
        let update = runtime.refresh();
        assert!(update.redraw);
        assert_eq!(update.revision, 1);
        assert_eq!(runtime.background_poll_interval_ms(), None);
    }

    #[test]
    #[cfg(all(feature = "tabs", feature = "label"))]
    fn tab_view_keeps_one_active_page_and_routes_strongly_typed_selection() {
        let tab_view_id = WidgetId::new(200);
        let general = ZsTabId::new(201);
        let advanced = ZsTabId::new(202);
        let about = ZsTabId::new(203);
        let general_content = WidgetId::new(211);
        let advanced_content = WidgetId::new(212);
        let mut view = tab_view(
            [
                ZsTabItem::new(
                    general,
                    "General",
                    text("General content").id(general_content),
                ),
                ZsTabItem::new(
                    advanced,
                    "Advanced",
                    text("Advanced content").id(advanced_content),
                ),
                ZsTabItem::new(about, "About", text("About content")),
            ],
            Some(general),
        )
        .id(tab_view_id)
        .on_tab_select(Msg::TabSelected);
        let mut layout = ViewLayoutCx::new(
            Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 260,
            },
            Dpi::standard(),
        );
        view.layout(&mut layout);

        let interactions = view.interaction_plan();
        assert_eq!(
            interactions
                .hit_targets
                .iter()
                .filter(|target| matches!(target.kind, ViewHitTargetKind::Tab { .. }))
                .count(),
            3
        );
        assert!(interactions
            .hit_target_for_widget(general_content)
            .is_some());
        assert!(interactions
            .hit_target_for_widget(advanced_content)
            .is_none());
        assert!(view
            .widget_tab_header_state(WidgetId(general.0))
            .is_some_and(|state| state.selected));
        assert!(view
            .widget_tab_header_state(WidgetId(advanced.0))
            .is_some_and(|state| !state.selected));
        assert_eq!(
            view.widget_tab_cycle_target(general_content, 1),
            Some((tab_view_id, advanced))
        );

        let mut event_cx = ViewEventCx::new();
        view.event(
            &mut event_cx,
            &ViewEvent::TabSelected {
                widget: tab_view_id,
                tab: advanced,
            },
        );
        assert_eq!(event_cx.messages(), &[Msg::TabSelected(advanced)]);
        assert_eq!(
            view.widget_tab_view_state(tab_view_id),
            Some(ZsTabViewState {
                selected: Some(advanced),
                tab_count: 3,
            })
        );

        view.layout(&mut layout);
        let mut paint = ViewPaintCx::new(Dpi::standard());
        view.paint(&mut paint);
        assert!(paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Advanced content"
        )));
        assert!(!paint.plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "General content"
        )));
    }

    #[test]
    fn app_context_keeps_commands_explicit() {
        let mut cx = AppCx::new();

        cx.command(Command::OpenSettings);
        cx.ui_command(crate::UiCommand::app(crate::CommandId("view.save")));
        cx.quit();

        assert_eq!(cx.commands(), &[Command::OpenSettings]);
        assert_eq!(cx.ui_commands()[0].id, crate::CommandId("view.save"));
        assert!(cx.quit_requested());
    }

    #[test]
    #[cfg(all(feature = "button", feature = "label"))]
    fn live_view_runtime_rebuilds_from_state_after_typed_message() {
        #[derive(Clone)]
        enum CounterMsg {
            Increment,
        }

        struct CounterState {
            value: u32,
        }

        let button_id = WidgetId::new(90);
        let runtime = live_view_runtime(
            CounterState { value: 0 },
            move |state| {
                column([
                    text(format!("Count: {}", state.value)),
                    button("Increment")
                        .id(button_id)
                        .on_click(CounterMsg::Increment),
                ])
            },
            |state, message, cx| match message {
                CounterMsg::Increment => {
                    state.value += 1;
                    cx.ui_command(UiCommand::app(crate::CommandId("counter.incremented")));
                }
            },
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 160,
            },
            Dpi::standard(),
        );

        let before = runtime.draw_plan();
        assert!(before.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 0"
        )));

        let update = runtime.dispatch_event(&ViewEvent::Click { widget: button_id });

        assert!(update.redraw);
        assert_eq!(update.message_count, 1);
        assert_eq!(update.revision, 1);
        assert_eq!(
            update.ui_commands[0].id,
            crate::CommandId("counter.incremented")
        );
        assert!(runtime.draw_plan().commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(text) if text.text == "Count: 1"
        )));
    }
}
