#[cfg(feature = "textbox")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsTextSelection {
    pub anchor: usize,
    pub caret: usize,
}

#[cfg(feature = "textbox")]
impl ZsTextSelection {
    pub const fn collapsed(caret: usize) -> Self {
        Self {
            anchor: caret,
            caret,
        }
    }

    pub const fn ordered(self) -> (usize, usize) {
        if self.anchor <= self.caret {
            (self.anchor, self.caret)
        } else {
            (self.caret, self.anchor)
        }
    }

    pub const fn is_collapsed(self) -> bool {
        self.anchor == self.caret
    }
}

#[cfg(feature = "textbox")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTextEditCommand {
    Undo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

#[cfg(feature = "textbox")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsTextEditCommandRequest {
    pub widget: Option<WidgetId>,
    pub command: ZsTextEditCommand,
}

#[cfg(feature = "textbox")]
impl ZsTextEditCommandRequest {
    pub const fn focused(command: ZsTextEditCommand) -> Self {
        Self {
            widget: None,
            command,
        }
    }

    pub const fn for_widget(widget: WidgetId, command: ZsTextEditCommand) -> Self {
        Self {
            widget: Some(widget),
            command,
        }
    }
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
    #[cfg(feature = "textbox")]
    TextEdited {
        widget: WidgetId,
        value: String,
        selection: ZsTextSelection,
    },
    #[cfg(feature = "textbox")]
    TextSelectionChanged {
        widget: WidgetId,
        selection: ZsTextSelection,
    },
    #[cfg(feature = "password-box")]
    PasswordChanged {
        widget: WidgetId,
        #[serde(skip, default)]
        value: crate::ZsPassword,
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
    #[cfg(feature = "number-box")]
    NumberBoxStep {
        widget: WidgetId,
        steps: i32,
        large: bool,
    },
    #[cfg(feature = "number-box")]
    NumberBoxCommit {
        widget: WidgetId,
    },
    #[cfg(feature = "number-box")]
    NumberBoxReset {
        widget: WidgetId,
    },
    #[cfg(feature = "radio")]
    RadioSelected {
        widget: WidgetId,
    },
    #[cfg(feature = "auto-suggest")]
    AutoSuggestExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "auto-suggest")]
    AutoSuggestHighlighted {
        widget: WidgetId,
        suggestion: crate::ZsAutoSuggestionId,
    },
    #[cfg(feature = "auto-suggest")]
    AutoSuggestCleared {
        widget: WidgetId,
    },
    #[cfg(feature = "auto-suggest")]
    AutoSuggestSubmitted {
        widget: WidgetId,
        suggestion: Option<crate::ZsAutoSuggestionId>,
    },
    #[cfg(feature = "tree")]
    TreeNodeExpandedChanged {
        widget: WidgetId,
        node: crate::ZsTreeNodeId,
        expanded: bool,
    },
    #[cfg(feature = "tree")]
    TreeNodeSelected {
        widget: WidgetId,
        node: crate::ZsTreeNodeId,
    },
    #[cfg(feature = "tree")]
    TreeNodeInvoked {
        widget: WidgetId,
        node: crate::ZsTreeNodeId,
    },
    #[cfg(feature = "grid-view")]
    GridViewItemSelected {
        widget: WidgetId,
        item: crate::ZsGridViewItemId,
    },
    #[cfg(feature = "grid-view")]
    GridViewItemInvoked {
        widget: WidgetId,
        item: crate::ZsGridViewItemId,
    },
    #[cfg(feature = "table")]
    TableRowSelected {
        widget: WidgetId,
        row: crate::ZsTableRowId,
    },
    #[cfg(feature = "table")]
    TableSorted {
        widget: WidgetId,
        column: crate::ZsTableColumnId,
    },
    #[cfg(feature = "table")]
    TableRowInvoked {
        widget: WidgetId,
        row: crate::ZsTableRowId,
    },
    #[cfg(feature = "dialog")]
    ContentDialogFocused {
        widget: WidgetId,
        button: crate::ZsContentDialogButton,
    },
    #[cfg(feature = "dialog")]
    ContentDialogResponded {
        widget: WidgetId,
        button: crate::ZsContentDialogButton,
    },
    #[cfg(feature = "command-palette")]
    CommandPaletteHighlighted {
        widget: WidgetId,
        item: crate::ZsCommandPaletteItemId,
    },
    #[cfg(feature = "command-palette")]
    CommandPaletteInvoked {
        widget: WidgetId,
        item: crate::ZsCommandPaletteItemId,
    },
    #[cfg(feature = "command-palette")]
    CommandPaletteOpenChanged {
        widget: WidgetId,
        open: bool,
    },
    #[cfg(feature = "toast")]
    ToastFocused {
        widget: WidgetId,
        toast: crate::ZsToastId,
        control: crate::ZsToastControl,
    },
    #[cfg(feature = "toast")]
    ToastResponded {
        widget: WidgetId,
        toast: crate::ZsToastId,
        response: crate::ZsToastResponse,
    },
    #[cfg(feature = "teaching-tip")]
    TeachingTipFocused {
        widget: WidgetId,
        control: crate::ZsTeachingTipControl,
    },
    #[cfg(feature = "teaching-tip")]
    TeachingTipResponded {
        widget: WidgetId,
        response: crate::ZsTeachingTipResponse,
    },
    #[cfg(feature = "info-bar")]
    InfoBarFocused {
        widget: WidgetId,
        control: crate::ZsInfoBarControl,
    },
    #[cfg(feature = "info-bar")]
    InfoBarInvoked {
        widget: WidgetId,
        event: crate::ZsInfoBarEvent,
    },
    #[cfg(feature = "breadcrumb")]
    BreadcrumbFocused {
        widget: WidgetId,
        target: crate::ZsBreadcrumbFocusTarget,
    },
    #[cfg(feature = "breadcrumb")]
    BreadcrumbExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "breadcrumb")]
    BreadcrumbSelected {
        widget: WidgetId,
        item: crate::ZsBreadcrumbId,
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
    #[cfg(feature = "color-picker")]
    ColorPickerExpandedChanged {
        widget: WidgetId,
        expanded: bool,
    },
    #[cfg(feature = "color-picker")]
    ColorPickerChannelChanged {
        widget: WidgetId,
        channel: ZsColorChannel,
    },
    #[cfg(feature = "color-picker")]
    ColorChanged {
        widget: WidgetId,
        color: crate::Color,
    },
    #[cfg(feature = "tabs")]
    TabSelected {
        widget: WidgetId,
        tab: ZsTabId,
    },
    #[cfg(any(
        feature = "auto-suggest",
        feature = "breadcrumb",
        feature = "color-picker",
        feature = "combo",
        feature = "date-picker",
        feature = "time-picker"
    ))]
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

#[cfg(feature = "tooltip")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewTooltipTarget {
    pub widget: WidgetId,
    pub bounds: Rect,
    pub spec: crate::ZsTooltipSpec,
}

#[cfg(feature = "tooltip")]
impl ViewTooltipTarget {
    pub fn contains(&self, point: Point) -> bool {
        self.bounds.contains(point)
    }
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
    #[cfg(feature = "toggle-button")]
    ToggleButton,
    Textbox,
    TextEditor,
    #[cfg(feature = "password-box")]
    PasswordBox,
    #[cfg(feature = "password-box")]
    PasswordBoxReveal,
    Checkbox,
    Toggle,
    #[cfg(feature = "slider")]
    Slider,
    #[cfg(feature = "number-box")]
    NumberBox,
    #[cfg(feature = "number-box")]
    NumberBoxDecrement,
    #[cfg(feature = "number-box")]
    NumberBoxIncrement,
    #[cfg(feature = "radio")]
    RadioButton,
    #[cfg(feature = "auto-suggest")]
    AutoSuggestBox,
    #[cfg(feature = "auto-suggest")]
    AutoSuggestSearch,
    #[cfg(feature = "auto-suggest")]
    AutoSuggestClear,
    #[cfg(feature = "auto-suggest")]
    AutoSuggestSuggestion {
        suggestion: crate::ZsAutoSuggestionId,
    },
    #[cfg(feature = "tree")]
    TreeView,
    #[cfg(feature = "tree")]
    TreeNode {
        node: crate::ZsTreeNodeId,
    },
    #[cfg(feature = "tree")]
    TreeNodeExpander {
        node: crate::ZsTreeNodeId,
    },
    #[cfg(feature = "grid-view")]
    GridView,
    #[cfg(feature = "grid-view")]
    GridViewItem {
        item: crate::ZsGridViewItemId,
    },
    #[cfg(feature = "table")]
    DataGrid,
    #[cfg(feature = "table")]
    TableHeader {
        column: crate::ZsTableColumnId,
    },
    #[cfg(feature = "table")]
    TableRow {
        row: crate::ZsTableRowId,
    },
    #[cfg(feature = "dialog")]
    ContentDialog,
    #[cfg(feature = "dialog")]
    ContentDialogScrim,
    #[cfg(feature = "dialog")]
    ContentDialogButton {
        button: crate::ZsContentDialogButton,
    },
    #[cfg(feature = "command-palette")]
    CommandPalette,
    #[cfg(feature = "command-palette")]
    CommandPaletteScrim,
    #[cfg(feature = "command-palette")]
    CommandPaletteClear,
    #[cfg(feature = "command-palette")]
    CommandPaletteItem {
        item: crate::ZsCommandPaletteItemId,
    },
    #[cfg(feature = "toast")]
    Toast,
    #[cfg(feature = "toast")]
    ToastAction,
    #[cfg(feature = "toast")]
    ToastClose,
    #[cfg(feature = "teaching-tip")]
    TeachingTip,
    #[cfg(feature = "teaching-tip")]
    TeachingTipAction,
    #[cfg(feature = "teaching-tip")]
    TeachingTipClose,
    #[cfg(feature = "info-bar")]
    InfoBar,
    #[cfg(feature = "info-bar")]
    InfoBarAction,
    #[cfg(feature = "info-bar")]
    InfoBarClose,
    #[cfg(feature = "breadcrumb")]
    BreadcrumbBar,
    #[cfg(feature = "breadcrumb")]
    BreadcrumbOverflow,
    #[cfg(feature = "breadcrumb")]
    BreadcrumbItem {
        item: crate::ZsBreadcrumbId,
    },
    #[cfg(feature = "breadcrumb")]
    BreadcrumbOverflowItem {
        item: crate::ZsBreadcrumbId,
    },
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
    #[cfg(feature = "color-picker")]
    ColorPicker,
    #[cfg(feature = "color-picker")]
    ColorPickerPopup,
    #[cfg(feature = "color-picker")]
    ColorPickerSpectrum,
    #[cfg(feature = "color-picker")]
    ColorPickerHue,
    #[cfg(feature = "color-picker")]
    ColorPickerChannel {
        channel: ZsColorChannel,
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
    #[cfg(feature = "tooltip")]
    #[serde(default)]
    pub tooltip_targets: Vec<ViewTooltipTarget>,
}

impl ViewInteractionPlan {
    pub fn new(hit_targets: impl IntoIterator<Item = ViewHitTarget>) -> Self {
        Self {
            hit_targets: hit_targets.into_iter().collect(),
            #[cfg(feature = "tooltip")]
            tooltip_targets: Vec::new(),
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
        #[cfg(feature = "command-palette")]
        if let Some(target) = self.hit_targets.iter().copied().find(|target| {
            target.widget == widget && target.kind == ViewHitTargetKind::CommandPalette
        }) {
            return Some(target);
        }
        self.hit_targets
            .iter()
            .copied()
            .find(|target| target.widget == widget)
    }

    #[cfg(feature = "tooltip")]
    pub fn tooltip_target_at(&self, point: Point) -> Option<ViewTooltipTarget> {
        self.tooltip_targets
            .iter()
            .rev()
            .find(|target| target.contains(point))
            .cloned()
    }

    #[cfg(feature = "tooltip")]
    pub fn tooltip_for_widget(&self, widget: WidgetId) -> Option<ViewTooltipTarget> {
        self.tooltip_targets
            .iter()
            .find(|target| target.widget == widget)
            .cloned()
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
            .find(|target| target.accepts_focus() && self.accepts_focus_scope(*target))
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
            .filter(|target| target.accepts_focus() && self.accepts_focus_scope(*target))
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
            (target.accepts_focus()
                && self.accepts_focus_scope(target)
                && accepts_tab_focus(target))
            .then_some(target)
        })
    }

    fn accepts_focus_scope(&self, _target: ViewHitTarget) -> bool {
        #[cfg(feature = "command-palette")]
        if let Some(palette) = self
            .hit_targets
            .iter()
            .rev()
            .find(|candidate| candidate.kind == ViewHitTargetKind::CommandPalette)
        {
            return _target.widget == palette.widget
                && _target.kind == ViewHitTargetKind::CommandPalette;
        }
        #[cfg(feature = "dialog")]
        if let Some(dialog) = self
            .hit_targets
            .iter()
            .rev()
            .find(|candidate| candidate.kind == ViewHitTargetKind::ContentDialog)
        {
            return _target.widget == dialog.widget
                && _target.kind == ViewHitTargetKind::ContentDialog;
        }
        true
    }
}

impl ViewHitTarget {
    fn accepts_focus(&self) -> bool {
        #[cfg(feature = "command-palette")]
        if matches!(
            self.kind,
            ViewHitTargetKind::CommandPaletteScrim
                | ViewHitTargetKind::CommandPaletteClear
                | ViewHitTargetKind::CommandPaletteItem { .. }
        ) {
            return false;
        }
        #[cfg(feature = "dialog")]
        if matches!(
            self.kind,
            ViewHitTargetKind::ContentDialogScrim | ViewHitTargetKind::ContentDialogButton { .. }
        ) {
            return false;
        }
        #[cfg(feature = "toast")]
        if matches!(
            self.kind,
            ViewHitTargetKind::ToastAction | ViewHitTargetKind::ToastClose
        ) {
            return false;
        }
        #[cfg(feature = "teaching-tip")]
        if matches!(
            self.kind,
            ViewHitTargetKind::TeachingTipAction | ViewHitTargetKind::TeachingTipClose
        ) {
            return false;
        }
        #[cfg(feature = "info-bar")]
        if matches!(
            self.kind,
            ViewHitTargetKind::InfoBarAction | ViewHitTargetKind::InfoBarClose
        ) {
            return false;
        }
        #[cfg(feature = "breadcrumb")]
        if matches!(
            self.kind,
            ViewHitTargetKind::BreadcrumbOverflow
                | ViewHitTargetKind::BreadcrumbItem { .. }
                | ViewHitTargetKind::BreadcrumbOverflowItem { .. }
        ) {
            return false;
        }
        #[cfg(feature = "grid-view")]
        if matches!(self.kind, ViewHitTargetKind::GridViewItem { .. }) {
            return false;
        }
        #[cfg(feature = "password-box")]
        if self.kind == ViewHitTargetKind::PasswordBoxReveal {
            return false;
        }
        #[cfg(feature = "number-box")]
        if matches!(
            self.kind,
            ViewHitTargetKind::NumberBoxDecrement | ViewHitTargetKind::NumberBoxIncrement
        ) {
            return false;
        }
        #[cfg(feature = "table")]
        if matches!(
            self.kind,
            ViewHitTargetKind::TableHeader { .. } | ViewHitTargetKind::TableRow { .. }
        ) {
            return false;
        }
        #[cfg(feature = "combo")]
        if matches!(self.kind, ViewHitTargetKind::ComboBoxOption { .. }) {
            return false;
        }
        #[cfg(feature = "auto-suggest")]
        if matches!(
            self.kind,
            ViewHitTargetKind::AutoSuggestSearch
                | ViewHitTargetKind::AutoSuggestClear
                | ViewHitTargetKind::AutoSuggestSuggestion { .. }
        ) {
            return false;
        }
        #[cfg(feature = "tree")]
        if matches!(
            self.kind,
            ViewHitTargetKind::TreeNode { .. } | ViewHitTargetKind::TreeNodeExpander { .. }
        ) {
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
        #[cfg(feature = "color-picker")]
        if matches!(
            self.kind,
            ViewHitTargetKind::ColorPickerPopup
                | ViewHitTargetKind::ColorPickerSpectrum
                | ViewHitTargetKind::ColorPickerHue
                | ViewHitTargetKind::ColorPickerChannel { .. }
        ) {
            return false;
        }
        true
    }
}

impl ViewHitTargetKind {
    pub(crate) fn accepts_text_input(self) -> bool {
        let accepts = matches!(self, Self::Textbox | Self::TextEditor);
        #[cfg(feature = "password-box")]
        let accepts = accepts || self == Self::PasswordBox;
        #[cfg(feature = "number-box")]
        let accepts = accepts || self == Self::NumberBox;
        #[cfg(feature = "auto-suggest")]
        let accepts = accepts || self == Self::AutoSuggestBox;
        #[cfg(feature = "command-palette")]
        let accepts = accepts || self == Self::CommandPalette;
        accepts
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
    #[cfg(feature = "textbox")]
    text_edit_commands: Vec<ZsTextEditCommandRequest>,
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

    #[cfg(feature = "textbox")]
    pub fn text_edit_command(&mut self, command: ZsTextEditCommand) {
        self.text_edit_commands
            .push(ZsTextEditCommandRequest::focused(command));
    }

    #[cfg(feature = "textbox")]
    pub fn text_edit_command_for(&mut self, widget: WidgetId, command: ZsTextEditCommand) {
        self.text_edit_commands
            .push(ZsTextEditCommandRequest::for_widget(widget, command));
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

    #[cfg(feature = "textbox")]
    pub fn text_edit_commands(&self) -> &[ZsTextEditCommandRequest] {
        &self.text_edit_commands
    }

    pub const fn quit_requested(&self) -> bool {
        self.quit_requested
    }
}
