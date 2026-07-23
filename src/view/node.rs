#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// An owned, typed value-to-message callback used by controls that emit values.
///
/// Function-pointer handlers remain allocation-free. Capturing callbacks are
/// stored only when an application opts into a `*_with` builder.
#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "command-palette",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tree",
    feature = "grid-view",
    feature = "textbox",
    feature = "password-box",
    feature = "checkbox",
    feature = "toggle",
    feature = "toggle-button",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "list",
    feature = "tabs",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "scroll"
))]
#[doc(hidden)]
pub struct ViewMessageMapper<Input, Msg> {
    mapper: ViewMessageMapperKind<Input, Msg>,
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "command-palette",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tree",
    feature = "grid-view",
    feature = "textbox",
    feature = "password-box",
    feature = "checkbox",
    feature = "toggle",
    feature = "toggle-button",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "list",
    feature = "tabs",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "scroll"
))]
enum ViewMessageMapperKind<Input, Msg> {
    Function(fn(Input) -> Msg),
    Shared(Arc<dyn Fn(Input) -> Msg + Send + Sync + 'static>),
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "command-palette",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tree",
    feature = "grid-view",
    feature = "textbox",
    feature = "password-box",
    feature = "checkbox",
    feature = "toggle",
    feature = "toggle-button",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "list",
    feature = "tabs",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "scroll"
))]
impl<Input, Msg> ViewMessageMapper<Input, Msg> {
    fn from_function(mapper: fn(Input) -> Msg) -> Self {
        Self {
            mapper: ViewMessageMapperKind::Function(mapper),
        }
    }

    fn from_shared(
        mapper: impl Fn(Input) -> Msg + Send + Sync + 'static,
    ) -> Self {
        Self {
            mapper: ViewMessageMapperKind::Shared(Arc::new(mapper)),
        }
    }

    pub(crate) fn map(&self, input: Input) -> Msg {
        match &self.mapper {
            ViewMessageMapperKind::Function(mapper) => mapper(input),
            ViewMessageMapperKind::Shared(mapper) => mapper(input),
        }
    }
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "command-palette",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tree",
    feature = "grid-view",
    feature = "textbox",
    feature = "password-box",
    feature = "checkbox",
    feature = "toggle",
    feature = "toggle-button",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "list",
    feature = "tabs",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "scroll"
))]
impl<Input, Msg> Clone for ViewMessageMapper<Input, Msg> {
    fn clone(&self) -> Self {
        Self {
            mapper: match &self.mapper {
                ViewMessageMapperKind::Function(mapper) => {
                    ViewMessageMapperKind::Function(*mapper)
                }
                ViewMessageMapperKind::Shared(mapper) => {
                    ViewMessageMapperKind::Shared(Arc::clone(mapper))
                }
            },
        }
    }
}

#[cfg(any(
    feature = "auto-suggest",
    feature = "breadcrumb",
    feature = "command-palette",
    feature = "flyout",
    feature = "menu-flyout",
    feature = "tree",
    feature = "grid-view",
    feature = "textbox",
    feature = "password-box",
    feature = "checkbox",
    feature = "toggle",
    feature = "toggle-button",
    feature = "slider",
    feature = "number-box",
    feature = "combo",
    feature = "date-picker",
    feature = "time-picker",
    feature = "color-picker",
    feature = "list",
    feature = "tabs",
    feature = "dialog",
    feature = "toast",
    feature = "info-bar",
    feature = "teaching-tip",
    feature = "scroll"
))]
impl<Input, Msg> fmt::Debug for ViewMessageMapper<Input, Msg> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ViewMessageMapper(..)")
    }
}

const FRAMEWORK_WIDGET_ID_PAYLOAD_MASK: u64 = (1 << 62) - 1;
const AUTOMATIC_WIDGET_ID_NAMESPACE: u64 = 2 << 62;
#[cfg(feature = "tabs")]
const SYNTHETIC_WIDGET_ID_NAMESPACE: u64 = 3 << 62;
const AUTOMATIC_WIDGET_ID_PROBE: u64 = 0x1e37_79b9_7f4a_7c15;

impl WidgetId {
    /// Builds a deterministic identity for an interactive surface owned by a
    /// composite widget rather than by an application View node.
    #[cfg(feature = "tabs")]
    pub(crate) const fn synthetic_child(parent: Self, local: u64) -> Self {
        let mut hash = parent.0 ^ 0x9e37_79b9_7f4a_7c15;
        hash = hash
            .rotate_left(27)
            .wrapping_mul(0x94d0_49bb_1331_11eb)
            ^ local;
        hash ^= hash >> 31;
        hash = hash.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        hash ^= hash >> 29;
        Self(SYNTHETIC_WIDGET_ID_NAMESPACE | (hash & FRAMEWORK_WIDGET_ID_PAYLOAD_MASK))
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

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZsNumberRange {
    min: f64,
    max: f64,
    step: f64,
    large_step: f64,
}

#[cfg(feature = "number-box")]
impl ZsNumberRange {
    pub fn new(min: f64, max: f64) -> Self {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 100.0 };
        let (min, mut max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f64::EPSILON {
            max = min + 1.0;
        }
        let span = max - min;
        Self {
            min,
            max,
            step: (span / 100.0).max(f64::EPSILON),
            large_step: (span / 10.0).max(f64::EPSILON),
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step.min(self.max - self.min);
        }
        self
    }

    pub fn large_step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.large_step = step.min(self.max - self.min);
        }
        self
    }

    pub const fn min(self) -> f64 {
        self.min
    }

    pub const fn max(self) -> f64 {
        self.max
    }

    pub const fn step_size(self) -> f64 {
        self.step
    }

    pub const fn large_step_size(self) -> f64 {
        self.large_step
    }

    pub fn contains(self, value: f64) -> bool {
        value.is_finite() && value >= self.min && value <= self.max
    }

    pub fn clamp(self, value: f64) -> f64 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    pub fn offset(self, value: f64, steps: i32, large: bool, wraps: bool) -> f64 {
        let increment = if large { self.large_step } else { self.step };
        let requested = self.clamp(value) + increment * f64::from(steps);
        if wraps {
            if requested > self.max {
                return self.min;
            }
            if requested < self.min {
                return self.max;
            }
        }
        self.clamp(requested)
    }
}

#[cfg(feature = "number-box")]
impl From<RangeInclusive<f64>> for ZsNumberRange {
    fn from(range: RangeInclusive<f64>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsNumberFormat {
    fraction_digits: u8,
}

#[cfg(feature = "number-box")]
impl ZsNumberFormat {
    pub const fn new(fraction_digits: u8) -> Self {
        Self {
            fraction_digits: if fraction_digits > 12 {
                12
            } else {
                fraction_digits
            },
        }
    }

    pub const fn fraction_digits(self) -> u8 {
        self.fraction_digits
    }

    pub fn format(self, value: Option<f64>) -> String {
        value
            .filter(|value| value.is_finite())
            .map_or_else(String::new, |value| {
                format!("{:.*}", usize::from(self.fraction_digits), value)
            })
    }

    pub fn parse(self, text: &str) -> Option<f64> {
        let _ = self;
        text.trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    }
}

#[cfg(feature = "number-box")]
impl Default for ZsNumberFormat {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(feature = "number-box")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsNumberBoxState {
    pub value: Option<f64>,
    pub draft: String,
    pub valid: bool,
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

    pub fn icon(mut self, icon: crate::ZsIcon) -> Self {
        self.spec = self.spec.icon(icon);
        self
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

#[cfg(feature = "button")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZsButtonPresentation {
    Standard,
    Primary,
    Icon {
        icon: crate::ZsIcon,
    },
    Toolbar {
        icon: crate::ZsIcon,
        show_label: bool,
    },
    NavigationItem {
        icon: crate::ZsIcon,
        selected: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ViewNodeKind<Msg> {
    #[doc(hidden)]
    __Message(PhantomData<fn() -> Msg>),
    #[cfg(feature = "label")]
    Text {
        text: String,
        style: SemanticTextStyle,
    },
    #[cfg(feature = "label")]
    #[doc(hidden)]
    NavigationView {
        title: String,
        subtitle: String,
        item_count: usize,
        footer_count: usize,
        pane_open: bool,
        pane_width: Option<Dp>,
        minimum_content_width: Dp,
    },
    #[cfg(feature = "image-preview")]
    ImagePreview {
        snapshot: ZsImagePreviewSnapshot,
        fit: ZsImageFit,
        interpolation: NativeImageInterpolation,
    },
    #[cfg(feature = "canvas")]
    Canvas {
        scene: crate::ZsCanvasScene,
        on_click: Option<Msg>,
        on_pointer: Option<fn(crate::ZsCanvasPointerEvent) -> Msg>,
    },
    #[cfg(feature = "button")]
    Button {
        label: String,
        presentation: ZsButtonPresentation,
        enabled: bool,
        on_click: Option<Msg>,
    },
    #[cfg(feature = "breadcrumb")]
    BreadcrumbBar {
        items: Vec<crate::ZsBreadcrumbItem>,
        overflow_open: bool,
        focused: Option<crate::ZsBreadcrumbFocusTarget>,
        on_select: Option<ViewMessageMapper<crate::ZsBreadcrumbId, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "toggle-button")]
    ToggleButton {
        label: String,
        checked: bool,
        on_toggle: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "textbox")]
    Textbox {
        value: String,
        multiline: bool,
        wrap: crate::TextWrap,
        on_change: Option<ViewMessageMapper<String, Msg>>,
        on_selection_change: Option<fn(ZsTextSelection) -> Msg>,
    },
    #[cfg(feature = "password-box")]
    PasswordBox {
        value: crate::ZsPassword,
        reveal_mode: crate::ZsPasswordRevealMode,
        on_change: Option<ViewMessageMapper<crate::ZsPassword, Msg>>,
    },
    #[cfg(feature = "checkbox")]
    Checkbox {
        label: String,
        checked: bool,
        on_toggle: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "toggle")]
    Toggle {
        checked: bool,
        on_toggle: Option<ViewMessageMapper<bool, Msg>>,
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
        on_slide: Option<ViewMessageMapper<f32, Msg>>,
    },
    #[cfg(feature = "number-box")]
    NumberBox {
        value: Option<f64>,
        draft: String,
        range: ZsNumberRange,
        format: ZsNumberFormat,
        wraps: bool,
        on_change: Option<ViewMessageMapper<Option<f64>, Msg>>,
    },
    #[cfg(feature = "progress")]
    ProgressBar {
        value: f32,
        range: crate::ProgressRange,
    },
    #[cfg(feature = "progress-ring")]
    ProgressRing {
        spec: crate::ZsProgressRingSpec,
    },
    #[cfg(feature = "auto-suggest")]
    AutoSuggestBox {
        query: String,
        suggestions: Vec<crate::ZsAutoSuggestion>,
        highlighted: Option<crate::ZsAutoSuggestionId>,
        expanded: bool,
        placeholder: Option<String>,
        no_results_text: Option<String>,
        query_icon: bool,
        on_text_change: Option<ViewMessageMapper<crate::ZsAutoSuggestTextChange, Msg>>,
        on_suggestion_chosen: Option<ViewMessageMapper<crate::ZsAutoSuggestionId, Msg>>,
        on_query_submit: Option<ViewMessageMapper<crate::ZsAutoSuggestSubmission, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "tree")]
    TreeView {
        roots: Vec<crate::ZsTreeNode>,
        expanded: BTreeSet<crate::ZsTreeNodeId>,
        selected: Option<crate::ZsTreeNodeId>,
        on_select: Option<ViewMessageMapper<crate::ZsTreeNodeId, Msg>>,
        on_expansion_change: Option<ViewMessageMapper<crate::ZsTreeExpansionChange, Msg>>,
        on_invoke: Option<ViewMessageMapper<crate::ZsTreeNodeId, Msg>>,
    },
    #[cfg(feature = "grid-view")]
    GridView {
        items: Vec<crate::ZsGridViewItem>,
        selected: Option<crate::ZsGridViewItemId>,
        on_select: Option<ViewMessageMapper<crate::ZsGridViewItemId, Msg>>,
        on_invoke: Option<ViewMessageMapper<crate::ZsGridViewItemId, Msg>>,
    },
    #[cfg(feature = "table")]
    DataGrid {
        columns: Vec<crate::ZsTableColumn>,
        rows: Vec<crate::ZsTableRow>,
        selected: Option<crate::ZsTableRowId>,
        sort: Option<crate::ZsTableSort>,
        on_select: Option<fn(crate::ZsTableRowId) -> Msg>,
        on_sort: Option<fn(crate::ZsTableSort) -> Msg>,
        on_invoke: Option<fn(crate::ZsTableRowId) -> Msg>,
    },
    #[cfg(feature = "dialog")]
    ContentDialog {
        spec: crate::ZsContentDialogSpec,
        open: bool,
        focused_button: crate::ZsContentDialogButton,
        on_result: Option<ViewMessageMapper<crate::ZsContentDialogResult, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "flyout")]
    Flyout {
        spec: crate::ZsFlyoutSpec,
        open: bool,
        target: WidgetId,
        on_dismiss: Option<ViewMessageMapper<crate::ZsFlyoutDismissReason, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "menu-flyout")]
    MenuFlyout {
        menu: crate::MenuSpec,
        open: bool,
        target: WidgetId,
        highlighted: Option<crate::ZsMenuFlyoutPath>,
        open_submenus: Vec<crate::ZsMenuFlyoutPath>,
        on_command: Option<ViewMessageMapper<crate::Command, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "command-palette")]
    CommandPalette {
        items: Vec<crate::ZsCommandPaletteItem>,
        query: String,
        highlighted: Option<crate::ZsCommandPaletteItemId>,
        open: bool,
        placeholder: String,
        no_results_text: String,
        on_query_change: Option<ViewMessageMapper<String, Msg>>,
        on_highlight_change: Option<ViewMessageMapper<crate::ZsCommandPaletteItemId, Msg>>,
        on_invoke: Option<ViewMessageMapper<crate::ZsCommandPaletteItemId, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "toast")]
    ToastPresenter {
        toast: Option<crate::ZsToastSpec>,
        focused_control: crate::ZsToastControl,
        on_result: Option<ViewMessageMapper<crate::ZsToastResult, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "teaching-tip")]
    TeachingTip {
        spec: crate::ZsTeachingTipSpec,
        open: bool,
        target: WidgetId,
        focused_control: crate::ZsTeachingTipControl,
        on_result: Option<ViewMessageMapper<crate::ZsTeachingTipResult, Msg>>,
        on_open_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "info-bar")]
    InfoBar {
        spec: crate::ZsInfoBarSpec,
        focused_control: Option<crate::ZsInfoBarControl>,
        on_event: Option<ViewMessageMapper<crate::ZsInfoBarEvent, Msg>>,
    },
    #[cfg(feature = "combo")]
    ComboBox {
        options: Vec<String>,
        selected_index: Option<usize>,
        expanded: bool,
        placeholder: Option<String>,
        on_select: Option<ViewMessageMapper<usize, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "date-picker")]
    DatePicker {
        value: ZsDate,
        minimum: ZsDate,
        maximum: ZsDate,
        visible_month: ZsDate,
        today: Option<ZsDate>,
        expanded: bool,
        on_date_change: Option<ViewMessageMapper<ZsDate, Msg>>,
        on_month_change: Option<ViewMessageMapper<ZsDate, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "time-picker")]
    TimePicker {
        value: ZsTime,
        minute_increment: ZsMinuteIncrement,
        clock: ZsClockFormat,
        expanded: bool,
        on_time_change: Option<ViewMessageMapper<ZsTime, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
    },
    #[cfg(feature = "color-picker")]
    ColorPicker {
        state: ZsColorPickerState,
        on_color_change: Option<ViewMessageMapper<crate::Color, Msg>>,
        on_expanded_change: Option<ViewMessageMapper<bool, Msg>>,
        on_channel_change: Option<ViewMessageMapper<ZsColorChannel, Msg>>,
    },
    #[cfg(feature = "tabs")]
    Tabs {
        tabs: Vec<ZsTabSpec>,
        selected: Option<ZsTabId>,
        on_select: Option<ViewMessageMapper<ZsTabId, Msg>>,
    },
    #[cfg(feature = "list")]
    List {
        selected_index: Option<usize>,
        on_select: Option<ViewMessageMapper<usize, Msg>>,
    },
    #[cfg(feature = "scroll")]
    Scroll {
        offset_y: Dp,
        content_height: Option<Dp>,
        on_scroll: Option<ViewMessageMapper<Dp, Msg>>,
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
    pub min_width: Option<Dp>,
    pub min_height: Option<Dp>,
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
            min_width: None,
            min_height: None,
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
    #[cfg(feature = "tooltip")]
    tooltip: Option<crate::ZsTooltipSpec>,
    bounds: Option<Rect>,
    layout_dpi: Dpi,
    #[cfg(all(test, any(feature = "button", feature = "label")))]
    platform_style_override: Option<crate::ZsBaseControlPlatformStyle>,
    pub(crate) typography_scaled_height: bool,
    #[cfg(feature = "list")]
    pub(crate) list_item_horizontal_inset: Option<Dp>,
    #[cfg(feature = "combo")]
    combo_first_visible_option: Option<usize>,
    #[cfg(feature = "ui-viewer")]
    document_poll_interval_ms: Option<u64>,
    message: PhantomData<fn() -> Msg>,
}

impl<Msg> ViewNode<Msg> {
    pub fn new(kind: ViewNodeKind<Msg>) -> Self {
        Self {
            id: None,
            kind,
            style: ViewStyle::default(),
            children: Vec::new(),
            #[cfg(feature = "tooltip")]
            tooltip: None,
            bounds: None,
            layout_dpi: Dpi::standard(),
            #[cfg(all(test, any(feature = "button", feature = "label")))]
            platform_style_override: None,
            typography_scaled_height: false,
            #[cfg(feature = "list")]
            list_item_horizontal_inset: None,
            #[cfg(feature = "combo")]
            combo_first_visible_option: None,
            #[cfg(feature = "ui-viewer")]
            document_poll_interval_ms: None,
            message: PhantomData,
        }
    }

    pub fn id(mut self, id: WidgetId) -> Self {
        self.id = Some(id);
        self
    }

    /// Assigns deterministic IDs to event-capable nodes that do not have an
    /// explicit [`WidgetId`]. Normal layout calls this automatically.
    ///
    /// Automatic IDs are stable while the View tree keeps the same structure.
    /// Keep using [`ViewNode::id`] when application state, another component,
    /// or a test needs to refer to a widget across insertion or reordering.
    pub fn assign_automatic_ids(&mut self) {
        let mut used = BTreeSet::new();
        self.collect_explicit_widget_ids(&mut used);
        let mut path = Vec::new();
        self.assign_automatic_ids_at_path(&mut path, false, &mut used);
    }

    fn collect_explicit_widget_ids(&self, used: &mut BTreeSet<u64>) {
        if let Some(id) = self.id {
            used.insert(id.0);
        }
        for child in &self.children {
            child.collect_explicit_widget_ids(used);
        }
    }

    fn assign_automatic_ids_at_path(
        &mut self,
        path: &mut Vec<usize>,
        list_item: bool,
        used: &mut BTreeSet<u64>,
    ) {
        if self.id.is_none() && (list_item || self.accepts_automatic_widget_id()) {
            let mut candidate = automatic_widget_id_for_path(path);
            while used.contains(&candidate) {
                candidate = AUTOMATIC_WIDGET_ID_NAMESPACE
                    | (candidate
                        .wrapping_add(AUTOMATIC_WIDGET_ID_PROBE)
                        & FRAMEWORK_WIDGET_ID_PAYLOAD_MASK);
            }
            self.id = Some(WidgetId(candidate));
            used.insert(candidate);
        }

        let children_are_list_items = false;
        #[cfg(feature = "list")]
        let children_are_list_items =
            children_are_list_items || matches!(&self.kind, ViewNodeKind::List { .. });
        #[cfg(feature = "virtual-list")]
        let children_are_list_items =
            children_are_list_items || matches!(&self.kind, ViewNodeKind::VirtualList { .. });

        for (index, child) in self.children.iter_mut().enumerate() {
            path.push(index);
            child.assign_automatic_ids_at_path(path, children_are_list_items, used);
            path.pop();
        }
    }

    fn accepts_automatic_widget_id(&self) -> bool {
        #[cfg(feature = "tooltip")]
        if self.tooltip.is_some() {
            return true;
        }
        match &self.kind {
            #[cfg(feature = "label")]
            ViewNodeKind::NavigationView { .. } => true,
            #[cfg(feature = "canvas")]
            ViewNodeKind::Canvas { .. } => true,
            #[cfg(feature = "button")]
            ViewNodeKind::Button { .. } => true,
            #[cfg(feature = "breadcrumb")]
            ViewNodeKind::BreadcrumbBar { .. } => true,
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { .. } => true,
            #[cfg(feature = "textbox")]
            ViewNodeKind::Textbox { .. } => true,
            #[cfg(feature = "password-box")]
            ViewNodeKind::PasswordBox { .. } => true,
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { .. } => true,
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { .. } => true,
            #[cfg(feature = "radio")]
            ViewNodeKind::RadioButton { .. } => true,
            #[cfg(feature = "slider")]
            ViewNodeKind::Slider { .. } => true,
            #[cfg(feature = "number-box")]
            ViewNodeKind::NumberBox { .. } => true,
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox { .. } => true,
            #[cfg(feature = "tree")]
            ViewNodeKind::TreeView { .. } => true,
            #[cfg(feature = "grid-view")]
            ViewNodeKind::GridView { .. } => true,
            #[cfg(feature = "table")]
            ViewNodeKind::DataGrid { .. } => true,
            #[cfg(feature = "dialog")]
            ViewNodeKind::ContentDialog { .. } => true,
            #[cfg(feature = "flyout")]
            ViewNodeKind::Flyout { .. } => true,
            #[cfg(feature = "menu-flyout")]
            ViewNodeKind::MenuFlyout { .. } => true,
            #[cfg(feature = "command-palette")]
            ViewNodeKind::CommandPalette { .. } => true,
            #[cfg(feature = "toast")]
            ViewNodeKind::ToastPresenter { .. } => true,
            #[cfg(feature = "teaching-tip")]
            ViewNodeKind::TeachingTip { .. } => true,
            #[cfg(feature = "info-bar")]
            ViewNodeKind::InfoBar { .. } => true,
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { .. } => true,
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker { .. } => true,
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker { .. } => true,
            #[cfg(feature = "color-picker")]
            ViewNodeKind::ColorPicker { .. } => true,
            #[cfg(feature = "tabs")]
            ViewNodeKind::Tabs { .. } => true,
            #[cfg(feature = "scroll")]
            ViewNodeKind::Scroll { .. } => true,
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { .. } => true,
            _ => false,
        }
    }

    #[cfg(all(test, any(feature = "button", feature = "label")))]
    pub(crate) fn with_platform_style_override(
        mut self,
        platform: crate::ZsBaseControlPlatformStyle,
    ) -> Self {
        self.platform_style_override = Some(platform);
        self
    }

    #[cfg(any(feature = "button", feature = "label"))]
    pub(crate) fn resolved_platform_style(&self) -> crate::ZsBaseControlPlatformStyle {
        #[cfg(test)]
        if let Some(platform) = self.platform_style_override {
            return platform;
        }
        crate::ZsBaseControlPlatformStyle::current()
    }

    #[cfg(feature = "tooltip")]
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(crate::ZsTooltipSpec::new(text));
        self
    }

    #[cfg(feature = "tooltip")]
    pub fn tooltip_spec(mut self, spec: impl Into<crate::ZsTooltipSpec>) -> Self {
        self.tooltip = Some(spec.into());
        self
    }

    #[cfg(feature = "tooltip")]
    pub fn without_tooltip(mut self) -> Self {
        self.tooltip = None;
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

    pub fn min_width(mut self, min_width: Dp) -> Self {
        self.style.min_width = Some(min_width);
        self
    }

    pub fn min_height(mut self, min_height: Dp) -> Self {
        self.style.min_height = Some(min_height);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn native_typography_height(mut self, height: Dp) -> Self {
        self.style.height = Some(height);
        self.typography_scaled_height = true;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn native_typography_min_height(mut self, min_height: Dp) -> Self {
        self.style.min_height = Some(min_height);
        self.typography_scaled_height = true;
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
        #[cfg(feature = "ui-viewer")]
        if let Some(interval_ms) = self.document_poll_interval_ms {
            return Some(interval_ms);
        }
        #[cfg(feature = "progress-ring")]
        if matches!(
            self.kind,
            ViewNodeKind::ProgressRing { spec } if spec.is_animating()
        ) {
            return Some(16);
        }
        #[cfg(feature = "virtual-list")]
        if matches!(self.kind, ViewNodeKind::VirtualList { loading: true, .. }) {
            return Some(33);
        }
        #[cfg(feature = "image-preview")]
        if matches!(
            self.kind,
            ViewNodeKind::ImagePreview {
                snapshot: ZsImagePreviewSnapshot { loading: true, .. },
                ..
            }
        ) {
            return Some(16);
        }
        self.children
            .iter()
            .filter_map(ViewNode::background_poll_interval_ms)
            .min()
    }

    #[cfg(feature = "ui-viewer")]
    pub(crate) fn with_document_poll_interval_ms(mut self, interval_ms: u64) -> Self {
        self.document_poll_interval_ms = Some(interval_ms.max(16));
        self
    }

    #[cfg(any(
        feature = "flyout",
        feature = "label",
        feature = "menu-flyout",
        feature = "tabs",
        feature = "virtual-list"
    ))]
    fn clear_layout_bounds(&mut self) {
        self.bounds = None;
        for child in &mut self.children {
            child.clear_layout_bounds();
        }
    }
}

fn automatic_widget_id_for_path(path: &[usize]) -> u64 {
    // FNV-1a keeps the mapping deterministic across processes and Rust versions.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    hash ^= path.len() as u64;
    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    for &index in path {
        for byte in (index as u64).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    AUTOMATIC_WIDGET_ID_NAMESPACE | (hash & FRAMEWORK_WIDGET_ID_PAYLOAD_MASK)
}

impl<Msg: Clone> ViewNode<Msg> {
    #[cfg(any(feature = "button", feature = "canvas"))]
    pub fn on_click(mut self, message: Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "button")]
            ViewNodeKind::Button { on_click, .. } => *on_click = Some(message),
            #[cfg(feature = "canvas")]
            ViewNodeKind::Canvas { on_click, .. } => *on_click = Some(message),
            _ => {}
        }
        self
    }

    #[cfg(feature = "button")]
    /// Controls whether a button participates in focus, hit testing and activation.
    pub fn enabled(mut self, enabled: bool) -> Self {
        if let ViewNodeKind::Button {
            enabled: button_enabled,
            ..
        } = &mut self.kind
        {
            *button_enabled = enabled;
        }
        self
    }

    #[cfg(feature = "breadcrumb")]
    pub fn on_breadcrumb_select(mut self, message: fn(crate::ZsBreadcrumbId) -> Msg) -> Self {
        if let ViewNodeKind::BreadcrumbBar { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "breadcrumb")]
    pub fn on_breadcrumb_select_with(
        mut self,
        message: impl Fn(crate::ZsBreadcrumbId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::BreadcrumbBar { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn on_change(mut self, message: fn(String) -> Msg) -> Self {
        if let ViewNodeKind::Textbox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn on_change_with(
        mut self,
        message: impl Fn(String) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Textbox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn on_text_selection_change(mut self, message: fn(ZsTextSelection) -> Msg) -> Self {
        if let ViewNodeKind::Textbox {
            on_selection_change,
            ..
        } = &mut self.kind
        {
            *on_selection_change = Some(message);
        }
        self
    }

    #[cfg(feature = "textbox")]
    pub fn text_wrap(mut self, wrap: crate::TextWrap) -> Self {
        if let ViewNodeKind::Textbox {
            multiline,
            wrap: current,
            ..
        } = &mut self.kind
        {
            if *multiline {
                *current = wrap;
            }
        }
        self
    }

    #[cfg(feature = "password-box")]
    pub fn on_password_change(mut self, message: fn(crate::ZsPassword) -> Msg) -> Self {
        if let ViewNodeKind::PasswordBox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Registers an application-owned secure value callback.
    ///
    /// The password stays as [`ZsPassword`](crate::ZsPassword) and is never
    /// lowered through the JSON action channel used by ordinary UI document
    /// values.
    #[cfg(feature = "password-box")]
    pub fn on_password_change_with(
        mut self,
        message: impl Fn(crate::ZsPassword) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::PasswordBox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "password-box")]
    pub fn reveal_mode(mut self, mode: crate::ZsPasswordRevealMode) -> Self {
        if let ViewNodeKind::PasswordBox { reveal_mode, .. } = &mut self.kind {
            *reveal_mode = mode;
        }
        self
    }

    #[cfg(any(feature = "checkbox", feature = "toggle", feature = "toggle-button"))]
    pub fn on_toggle(mut self, message: fn(bool) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { on_toggle, .. } => {
                *on_toggle = Some(ViewMessageMapper::from_function(message));
            }
            #[cfg(feature = "checkbox")]
            ViewNodeKind::Checkbox { on_toggle, .. } => {
                *on_toggle = Some(ViewMessageMapper::from_function(message));
            }
            #[cfg(feature = "toggle")]
            ViewNodeKind::Toggle { on_toggle, .. } => {
                *on_toggle = Some(ViewMessageMapper::from_function(message));
            }
            _ => {}
        }
        self
    }

    #[cfg(any(feature = "checkbox", feature = "toggle", feature = "toggle-button"))]
    pub fn on_toggle_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        let message = ViewMessageMapper::from_shared(message);
        match &mut self.kind {
            #[cfg(feature = "toggle-button")]
            ViewNodeKind::ToggleButton { on_toggle, .. } => *on_toggle = Some(message),
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
            *on_slide = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "slider")]
    pub fn on_slide_with(
        mut self,
        message: impl Fn(f32) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Slider { on_slide, .. } = &mut self.kind {
            *on_slide = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "number-box")]
    pub fn on_number_change(mut self, message: fn(Option<f64>) -> Msg) -> Self {
        if let ViewNodeKind::NumberBox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "number-box")]
    pub fn on_number_change_with(
        mut self,
        message: impl Fn(Option<f64>) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::NumberBox { on_change, .. } = &mut self.kind {
            *on_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "number-box")]
    pub fn fraction_digits(mut self, digits: u8) -> Self {
        if let ViewNodeKind::NumberBox {
            value,
            draft,
            format,
            ..
        } = &mut self.kind
        {
            *format = ZsNumberFormat::new(digits);
            *draft = format.format(*value);
        }
        self
    }

    #[cfg(feature = "number-box")]
    pub fn wraps(mut self, should_wrap: bool) -> Self {
        if let ViewNodeKind::NumberBox { wraps, .. } = &mut self.kind {
            *wraps = should_wrap;
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
            ViewNodeKind::List { on_select, .. } => {
                *on_select = Some(ViewMessageMapper::from_function(message))
            }
            #[cfg(feature = "virtual-list")]
            ViewNodeKind::VirtualList { on_select, .. } => *on_select = Some(message),
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { on_select, .. } => {
                *on_select = Some(ViewMessageMapper::from_function(message))
            }
            _ => {}
        }
        self
    }

    #[cfg(feature = "list")]
    pub fn on_list_select_with(
        mut self,
        message: impl Fn(usize) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::List { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "combo")]
    pub fn on_combo_select_with(
        mut self,
        message: impl Fn(usize) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ComboBox { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn selected_tree_node(mut self, selected: Option<crate::ZsTreeNodeId>) -> Self {
        if let ViewNodeKind::TreeView {
            selected: current, ..
        } = &mut self.kind
        {
            *current = selected;
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn expanded_tree_nodes(
        mut self,
        expanded: impl IntoIterator<Item = crate::ZsTreeNodeId>,
    ) -> Self {
        if let ViewNodeKind::TreeView {
            expanded: current, ..
        } = &mut self.kind
        {
            *current = expanded.into_iter().collect();
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_select(mut self, message: fn(crate::ZsTreeNodeId) -> Msg) -> Self {
        if let ViewNodeKind::TreeView { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_select_with(
        mut self,
        message: impl Fn(crate::ZsTreeNodeId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TreeView { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_expansion_change(
        mut self,
        message: fn(crate::ZsTreeExpansionChange) -> Msg,
    ) -> Self {
        if let ViewNodeKind::TreeView {
            on_expansion_change,
            ..
        } = &mut self.kind
        {
            *on_expansion_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_expansion_change_with(
        mut self,
        message: impl Fn(crate::ZsTreeExpansionChange) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TreeView {
            on_expansion_change,
            ..
        } = &mut self.kind
        {
            *on_expansion_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_invoke(mut self, message: fn(crate::ZsTreeNodeId) -> Msg) -> Self {
        if let ViewNodeKind::TreeView { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "tree")]
    pub fn on_tree_invoke_with(
        mut self,
        message: impl Fn(crate::ZsTreeNodeId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TreeView { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "grid-view")]
    pub fn selected_grid_view_item(mut self, selected: Option<crate::ZsGridViewItemId>) -> Self {
        if let ViewNodeKind::GridView {
            selected: current, ..
        } = &mut self.kind
        {
            *current = selected;
        }
        self
    }

    #[cfg(feature = "grid-view")]
    pub fn on_grid_view_select(mut self, message: fn(crate::ZsGridViewItemId) -> Msg) -> Self {
        if let ViewNodeKind::GridView { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "grid-view")]
    pub fn on_grid_view_select_with(
        mut self,
        message: impl Fn(crate::ZsGridViewItemId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::GridView { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "grid-view")]
    pub fn on_grid_view_invoke(mut self, message: fn(crate::ZsGridViewItemId) -> Msg) -> Self {
        if let ViewNodeKind::GridView { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "grid-view")]
    pub fn on_grid_view_invoke_with(
        mut self,
        message: impl Fn(crate::ZsGridViewItemId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::GridView { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "table")]
    pub fn selected_table_row(mut self, selected: Option<crate::ZsTableRowId>) -> Self {
        if let ViewNodeKind::DataGrid {
            selected: current, ..
        } = &mut self.kind
        {
            *current = selected;
        }
        self
    }

    #[cfg(feature = "table")]
    pub fn table_sort(mut self, sort: Option<crate::ZsTableSort>) -> Self {
        if let ViewNodeKind::DataGrid { sort: current, .. } = &mut self.kind {
            *current = sort;
        }
        self
    }

    #[cfg(feature = "table")]
    pub fn on_table_select(mut self, message: fn(crate::ZsTableRowId) -> Msg) -> Self {
        if let ViewNodeKind::DataGrid { on_select, .. } = &mut self.kind {
            *on_select = Some(message);
        }
        self
    }

    #[cfg(feature = "table")]
    pub fn on_table_sort(mut self, message: fn(crate::ZsTableSort) -> Msg) -> Self {
        if let ViewNodeKind::DataGrid { on_sort, .. } = &mut self.kind {
            *on_sort = Some(message);
        }
        self
    }

    #[cfg(feature = "table")]
    pub fn on_table_invoke(mut self, message: fn(crate::ZsTableRowId) -> Msg) -> Self {
        if let ViewNodeKind::DataGrid { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(message);
        }
        self
    }

    #[cfg(feature = "dialog")]
    pub fn on_dialog_result(mut self, message: fn(crate::ZsContentDialogResult) -> Msg) -> Self {
        if let ViewNodeKind::ContentDialog { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Registers an owned callback for a content-dialog response.
    ///
    /// This is the closure-capable counterpart to [`Self::on_dialog_result`].
    /// It is used by document-backed views to retain the stable node and
    /// binding identity without a global callback registry.
    #[cfg(feature = "dialog")]
    pub fn on_dialog_result_with(
        mut self,
        message: impl Fn(crate::ZsContentDialogResult) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ContentDialog { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    /// Registers a controlled-state callback when a dialog response closes
    /// the modal surface.
    #[cfg(feature = "dialog")]
    pub fn on_dialog_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::ContentDialog { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Capturing-callback variant of [`Self::on_dialog_open_change`].
    #[cfg(feature = "dialog")]
    pub fn on_dialog_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ContentDialog { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "canvas")]
    /// Maps Canvas pointer press, move, release and cancellation to an app message.
    pub fn on_canvas_pointer(
        mut self,
        message: fn(crate::ZsCanvasPointerEvent) -> Msg,
    ) -> Self {
        if let ViewNodeKind::Canvas { on_pointer, .. } = &mut self.kind {
            *on_pointer = Some(message);
        }
        self
    }

    #[cfg(feature = "flyout")]
    pub fn on_flyout_dismiss(
        mut self,
        message: fn(crate::ZsFlyoutDismissReason) -> Msg,
    ) -> Self {
        if let ViewNodeKind::Flyout { on_dismiss, .. } = &mut self.kind {
            *on_dismiss = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "flyout")]
    pub fn on_flyout_dismiss_with(
        mut self,
        message: impl Fn(crate::ZsFlyoutDismissReason) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Flyout { on_dismiss, .. } = &mut self.kind {
            *on_dismiss = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "flyout")]
    pub fn on_flyout_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::Flyout { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "flyout")]
    pub fn on_flyout_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Flyout { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "menu-flyout")]
    pub fn on_menu_flyout_command(mut self, message: fn(crate::Command) -> Msg) -> Self {
        if let ViewNodeKind::MenuFlyout { on_command, .. } = &mut self.kind {
            *on_command = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "menu-flyout")]
    pub fn on_menu_flyout_command_with(
        mut self,
        message: impl Fn(crate::Command) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::MenuFlyout { on_command, .. } = &mut self.kind {
            *on_command = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "menu-flyout")]
    pub fn on_menu_flyout_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::MenuFlyout { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "menu-flyout")]
    pub fn on_menu_flyout_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::MenuFlyout { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn highlighted_command(
        mut self,
        highlighted_item: Option<crate::ZsCommandPaletteItemId>,
    ) -> Self {
        if let ViewNodeKind::CommandPalette {
            items,
            query,
            highlighted,
            ..
        } = &mut self.kind
        {
            let state =
                crate::command_palette::command_palette_state(true, query, items, highlighted_item);
            *highlighted = state.highlighted;
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        if let ViewNodeKind::CommandPalette {
            placeholder: current,
            ..
        } = &mut self.kind
        {
            *current = placeholder.into();
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette_no_results_text(mut self, text: impl Into<String>) -> Self {
        if let ViewNodeKind::CommandPalette {
            no_results_text, ..
        } = &mut self.kind
        {
            *no_results_text = text.into();
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_query_change(mut self, message: fn(String) -> Msg) -> Self {
        if let ViewNodeKind::CommandPalette {
            on_query_change, ..
        } = &mut self.kind
        {
            *on_query_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_query_change_with(
        mut self,
        message: impl Fn(String) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::CommandPalette {
            on_query_change, ..
        } = &mut self.kind
        {
            *on_query_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_highlight_change(
        mut self,
        message: fn(crate::ZsCommandPaletteItemId) -> Msg,
    ) -> Self {
        if let ViewNodeKind::CommandPalette {
            on_highlight_change,
            ..
        } = &mut self.kind
        {
            *on_highlight_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_highlight_change_with(
        mut self,
        message: impl Fn(crate::ZsCommandPaletteItemId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::CommandPalette {
            on_highlight_change,
            ..
        } = &mut self.kind
        {
            *on_highlight_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_invoke(
        mut self,
        message: fn(crate::ZsCommandPaletteItemId) -> Msg,
    ) -> Self {
        if let ViewNodeKind::CommandPalette { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_invoke_with(
        mut self,
        message: impl Fn(crate::ZsCommandPaletteItemId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::CommandPalette { on_invoke, .. } = &mut self.kind {
            *on_invoke = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::CommandPalette { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn on_command_palette_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::CommandPalette { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "toast")]
    pub fn on_toast_result(mut self, message: fn(crate::ZsToastResult) -> Msg) -> Self {
        if let ViewNodeKind::ToastPresenter { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Registers an owned callback for a toast response.
    ///
    /// Document-backed Views use this closure-capable form to retain the
    /// stable node and typed binding identity without a global event registry.
    #[cfg(feature = "toast")]
    pub fn on_toast_result_with(
        mut self,
        message: impl Fn(crate::ZsToastResult) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ToastPresenter { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    /// Registers an owned callback for the toast's controlled open state.
    ///
    /// A toast emits `false` after any response, including timeout, so a
    /// document-backed open binding cannot resurrect a dismissed toast on the
    /// next Viewer rebuild.
    #[cfg(feature = "toast")]
    pub fn on_toast_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ToastPresenter { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "teaching-tip")]
    pub fn on_teaching_tip_result(
        mut self,
        message: fn(crate::ZsTeachingTipResult) -> Msg,
    ) -> Self {
        if let ViewNodeKind::TeachingTip { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Registers an owned callback for a teaching-tip response.
    ///
    /// Document-backed Views use this form to retain the stable node and
    /// binding identity without a global event registry.
    #[cfg(feature = "teaching-tip")]
    pub fn on_teaching_tip_result_with(
        mut self,
        message: impl Fn(crate::ZsTeachingTipResult) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TeachingTip { on_result, .. } = &mut self.kind {
            *on_result = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    /// Reports the controlled teaching-tip open transition after a response.
    #[cfg(feature = "teaching-tip")]
    pub fn on_teaching_tip_open_change(mut self, message: fn(bool) -> Msg) -> Self {
        if let ViewNodeKind::TeachingTip { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Capturing-callback variant of [`Self::on_teaching_tip_open_change`].
    #[cfg(feature = "teaching-tip")]
    pub fn on_teaching_tip_open_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TeachingTip { on_open_change, .. } = &mut self.kind {
            *on_open_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "info-bar")]
    pub fn on_info_bar_event(mut self, message: fn(crate::ZsInfoBarEvent) -> Msg) -> Self {
        if let ViewNodeKind::InfoBar { on_event, .. } = &mut self.kind {
            *on_event = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    /// Registers an owned callback for an InfoBar action or close event.
    ///
    /// Document-backed Views use this closure-capable form to retain the
    /// stable node and typed binding identity without a global event registry.
    #[cfg(feature = "info-bar")]
    pub fn on_info_bar_event_with(
        mut self,
        message: impl Fn(crate::ZsInfoBarEvent) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::InfoBar { on_event, .. } = &mut self.kind {
            *on_event = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker"
    ))]
    pub fn expanded(mut self, is_expanded: bool) -> Self {
        match &mut self.kind {
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "breadcrumb")]
            ViewNodeKind::BreadcrumbBar { overflow_open, .. } => *overflow_open = is_expanded,
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker { expanded, .. } => *expanded = is_expanded,
            #[cfg(feature = "color-picker")]
            ViewNodeKind::ColorPicker { state, .. } => state.expanded = is_expanded,
            _ => {}
        }
        self
    }

    #[cfg(any(feature = "auto-suggest", feature = "combo"))]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        match &mut self.kind {
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox { placeholder, .. } => *placeholder = Some(text),
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox { placeholder, .. } => *placeholder = Some(text),
            _ => {}
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn highlighted_suggestion(mut self, suggestion: Option<crate::ZsAutoSuggestionId>) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            suggestions,
            highlighted,
            ..
        } = &mut self.kind
        {
            *highlighted =
                suggestion.filter(|id| suggestions.iter().any(|candidate| candidate.id() == *id));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn no_results_text(mut self, text: impl Into<String>) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            no_results_text, ..
        } = &mut self.kind
        {
            *no_results_text = Some(text.into());
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn query_icon(mut self, visible: bool) -> Self {
        if let ViewNodeKind::AutoSuggestBox { query_icon, .. } = &mut self.kind {
            *query_icon = visible;
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_auto_suggest_text_change(
        mut self,
        message: fn(crate::ZsAutoSuggestTextChange) -> Msg,
    ) -> Self {
        if let ViewNodeKind::AutoSuggestBox { on_text_change, .. } = &mut self.kind {
            *on_text_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_auto_suggest_text_change_with(
        mut self,
        message: impl Fn(crate::ZsAutoSuggestTextChange) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::AutoSuggestBox { on_text_change, .. } = &mut self.kind {
            *on_text_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_suggestion_chosen(mut self, message: fn(crate::ZsAutoSuggestionId) -> Msg) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            on_suggestion_chosen,
            ..
        } = &mut self.kind
        {
            *on_suggestion_chosen = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_suggestion_chosen_with(
        mut self,
        message: impl Fn(crate::ZsAutoSuggestionId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            on_suggestion_chosen,
            ..
        } = &mut self.kind
        {
            *on_suggestion_chosen = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_query_submit(mut self, message: fn(crate::ZsAutoSuggestSubmission) -> Msg) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            on_query_submit, ..
        } = &mut self.kind
        {
            *on_query_submit = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_query_submit_with(
        mut self,
        message: impl Fn(crate::ZsAutoSuggestSubmission) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            on_query_submit, ..
        } = &mut self.kind
        {
            *on_query_submit = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker"
    ))]
    pub fn on_expanded_change(mut self, message: fn(bool) -> Msg) -> Self {
        match &mut self.kind {
            #[cfg(feature = "auto-suggest")]
            ViewNodeKind::AutoSuggestBox {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
            #[cfg(feature = "breadcrumb")]
            ViewNodeKind::BreadcrumbBar {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
            #[cfg(feature = "combo")]
            ViewNodeKind::ComboBox {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
            #[cfg(feature = "date-picker")]
            ViewNodeKind::DatePicker {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
            #[cfg(feature = "time-picker")]
            ViewNodeKind::TimePicker {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
            #[cfg(feature = "color-picker")]
            ViewNodeKind::ColorPicker {
                on_expanded_change, ..
            } => *on_expanded_change = Some(ViewMessageMapper::from_function(message)),
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
            *on_date_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn on_date_change_with(
        mut self,
        message: impl Fn(ZsDate) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::DatePicker { on_date_change, .. } = &mut self.kind {
            *on_date_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    /// Sets the first visible calendar month independently from the selected
    /// value so controlled document views can retain month navigation.
    #[cfg(feature = "date-picker")]
    pub fn visible_month(mut self, month: ZsDate) -> Self {
        if let ViewNodeKind::DatePicker {
            minimum,
            maximum,
            visible_month,
            ..
        } = &mut self.kind
        {
            *visible_month = clamp_visible_month(month, *minimum, *maximum);
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn on_date_picker_month_change(mut self, message: fn(ZsDate) -> Msg) -> Self {
        if let ViewNodeKind::DatePicker {
            on_month_change, ..
        } = &mut self.kind
        {
            *on_month_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn on_date_picker_month_change_with(
        mut self,
        message: impl Fn(ZsDate) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::DatePicker {
            on_month_change, ..
        } = &mut self.kind
        {
            *on_month_change = Some(ViewMessageMapper::from_shared(message));
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
            *on_time_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "time-picker")]
    pub fn on_time_change_with(
        mut self,
        message: impl Fn(ZsTime) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TimePicker { on_time_change, .. } = &mut self.kind {
            *on_time_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "color-picker")]
    pub fn on_color_change(mut self, message: fn(crate::Color) -> Msg) -> Self {
        if let ViewNodeKind::ColorPicker {
            on_color_change, ..
        } = &mut self.kind
        {
            *on_color_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "color-picker")]
    pub fn on_color_change_with(
        mut self,
        message: impl Fn(crate::Color) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ColorPicker {
            on_color_change, ..
        } = &mut self.kind
        {
            *on_color_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "color-picker")]
    pub fn on_color_channel_change(mut self, message: fn(ZsColorChannel) -> Msg) -> Self {
        if let ViewNodeKind::ColorPicker {
            on_channel_change, ..
        } = &mut self.kind
        {
            *on_channel_change = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "color-picker")]
    pub fn on_color_channel_change_with(
        mut self,
        message: impl Fn(ZsColorChannel) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ColorPicker {
            on_channel_change, ..
        } = &mut self.kind
        {
            *on_channel_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "tabs")]
    pub fn on_tab_select(mut self, message: fn(ZsTabId) -> Msg) -> Self {
        if let ViewNodeKind::Tabs { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "tabs")]
    pub fn on_tab_select_with(
        mut self,
        message: impl Fn(ZsTabId) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Tabs { on_select, .. } = &mut self.kind {
            *on_select = Some(ViewMessageMapper::from_shared(message));
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
            *on_scroll = Some(ViewMessageMapper::from_function(message));
        }
        self
    }

    #[cfg(feature = "combo")]
    pub fn on_combo_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ComboBox {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "auto-suggest")]
    pub fn on_auto_suggest_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::AutoSuggestBox {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "breadcrumb")]
    pub fn on_breadcrumb_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::BreadcrumbBar {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "date-picker")]
    pub fn on_date_picker_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::DatePicker {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "time-picker")]
    pub fn on_time_picker_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::TimePicker {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "color-picker")]
    pub fn on_color_picker_expanded_change_with(
        mut self,
        message: impl Fn(bool) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::ColorPicker {
            on_expanded_change, ..
        } = &mut self.kind
        {
            *on_expanded_change = Some(ViewMessageMapper::from_shared(message));
        }
        self
    }

    #[cfg(feature = "scroll")]
    pub fn on_scroll_with(
        mut self,
        message: impl Fn(Dp) -> Msg + Send + Sync + 'static,
    ) -> Self {
        if let ViewNodeKind::Scroll { on_scroll, .. } = &mut self.kind {
            *on_scroll = Some(ViewMessageMapper::from_shared(message));
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

    #[cfg(feature = "image-preview")]
    pub fn image_fit(mut self, fit: ZsImageFit) -> Self {
        if let ViewNodeKind::ImagePreview { fit: current, .. } = &mut self.kind {
            *current = fit;
        }
        self
    }

    #[cfg(feature = "image-preview")]
    pub fn image_interpolation(mut self, interpolation: NativeImageInterpolation) -> Self {
        if let ViewNodeKind::ImagePreview {
            interpolation: current,
            ..
        } = &mut self.kind
        {
            *current = interpolation;
        }
        self
    }
}
