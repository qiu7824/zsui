use std::{collections::BTreeSet, env, fs};

use zsui::*;

const NAV_INPUTS: WidgetId = WidgetId::new(10);
const NAV_COLLECTIONS: WidgetId = WidgetId::new(11);
const NAV_NAVIGATION: WidgetId = WidgetId::new(12);
const NAV_FEEDBACK: WidgetId = WidgetId::new(13);
const NAV_CATALOG: WidgetId = WidgetId::new(14);
const PRIMARY_ACTION: WidgetId = WidgetId::new(100);
const TEXT_INPUT: WidgetId = WidgetId::new(101);
const PASSWORD_INPUT: WidgetId = WidgetId::new(102);
const AUTO_SUGGEST: WidgetId = WidgetId::new(103);
const COMBO_INPUT: WidgetId = WidgetId::new(104);
const DATE_INPUT: WidgetId = WidgetId::new(105);
const TIME_INPUT: WidgetId = WidgetId::new(106);
const COLOR_INPUT: WidgetId = WidgetId::new(107);
const CHECKBOX_INPUT: WidgetId = WidgetId::new(108);
const TOGGLE_INPUT: WidgetId = WidgetId::new(109);
const LIST_VIEW: WidgetId = WidgetId::new(200);
const VIRTUAL_LIST_VIEW: WidgetId = WidgetId::new(201);
const TREE_VIEW: WidgetId = WidgetId::new(202);
const GRID_VIEW: WidgetId = WidgetId::new(203);
const TABLE_VIEW: WidgetId = WidgetId::new(204);
const BREADCRUMB: WidgetId = WidgetId::new(300);
const TABS: WidgetId = WidgetId::new(301);
const COMMAND_PALETTE: WidgetId = WidgetId::new(302);
const INFO_BAR: WidgetId = WidgetId::new(400);
const DIALOG: WidgetId = WidgetId::new(401);
const TOAST: WidgetId = WidgetId::new(402);
const TEACHING_TIP: WidgetId = WidgetId::new(403);
const TEACHING_TARGET: WidgetId = WidgetId::new(404);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryPage {
    Inputs,
    Collections,
    Navigation,
    Feedback,
    Catalog,
}

impl GalleryPage {
    const ALL: [(Self, &'static str, WidgetId); 5] = [
        (Self::Inputs, "Inputs and actions", NAV_INPUTS),
        (Self::Collections, "Collections", NAV_COLLECTIONS),
        (Self::Navigation, "Navigation", NAV_NAVIGATION),
        (Self::Feedback, "Feedback and overlays", NAV_FEEDBACK),
        (Self::Catalog, "Catalog and layout", NAV_CATALOG),
    ];

    const fn title(self) -> &'static str {
        match self {
            Self::Inputs => "Inputs and actions",
            Self::Collections => "Collections",
            Self::Navigation => "Navigation",
            Self::Feedback => "Feedback and overlays",
            Self::Catalog => "Catalog and layout",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Inputs => "Typed values, focus, keyboard and pointer states",
            Self::Collections => "Lists, virtualization, trees, tiles and tabular data",
            Self::Navigation => "Breadcrumbs, tabs and keyboard-first commands",
            Self::Feedback => "Inline status, modal and nonmodal overlay surfaces",
            Self::Catalog => "Feature-gated framework inventory and layout primitives",
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Inputs => "inputs",
            Self::Collections => "collections",
            Self::Navigation => "navigation",
            Self::Feedback => "feedback",
            Self::Catalog => "catalog",
        }
    }

    fn from_slug(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .map(|(page, _, _)| page)
            .find(|page| page.slug() == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryOverlay {
    Dialog,
    Toast,
    TeachingTip,
}

#[derive(Debug)]
struct GalleryState {
    page: GalleryPage,
    dark: bool,
    click_count: u32,
    text: String,
    password: ZsPassword,
    checkbox: bool,
    toggle: bool,
    toggle_button: bool,
    radio: usize,
    slider: f32,
    number: Option<f64>,
    auto_query: String,
    combo: Option<usize>,
    combo_expanded: bool,
    date: ZsDate,
    date_expanded: bool,
    time: ZsTime,
    time_expanded: bool,
    color: ZsColorPickerState,
    list_selection: Option<usize>,
    grid_selection: Option<ZsGridViewItemId>,
    tree_selection: Option<ZsTreeNodeId>,
    tree_expanded: BTreeSet<ZsTreeNodeId>,
    table_selection: Option<ZsTableRowId>,
    table_sort: Option<ZsTableSort>,
    tab: ZsTabId,
    breadcrumb_expanded: bool,
    palette_open: bool,
    palette_query: String,
    overlay: Option<GalleryOverlay>,
    status: String,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            page: GalleryPage::Inputs,
            dark: false,
            click_count: 0,
            text: "ZSUI Native".to_string(),
            password: ZsPassword::from("desktop"),
            checkbox: true,
            toggle: true,
            toggle_button: false,
            radio: 0,
            slider: 42.0,
            number: Some(12.5),
            auto_query: String::new(),
            combo: Some(0),
            combo_expanded: false,
            date: ZsDate::new(2026, 7, 15).expect("gallery date is valid"),
            date_expanded: false,
            time: ZsTime::new(9, 30).expect("gallery time is valid"),
            time_expanded: false,
            color: ZsColorPickerState::new(Color::rgba(0, 120, 212, 255)),
            list_selection: Some(0),
            grid_selection: Some(ZsGridViewItemId::new(1)),
            tree_selection: Some(ZsTreeNodeId::new(2)),
            tree_expanded: BTreeSet::from([ZsTreeNodeId::new(1)]),
            table_selection: Some(ZsTableRowId::new(2)),
            table_sort: None,
            tab: ZsTabId::new(1),
            breadcrumb_expanded: false,
            palette_open: false,
            palette_query: String::new(),
            overlay: None,
            status: "Ready".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    Navigate(GalleryPage),
    Dark(bool),
    PrimaryAction,
    Text(String),
    Password(ZsPassword),
    Checkbox(bool),
    Toggle(bool),
    ToggleButton(bool),
    Radio(usize),
    Slider(f32),
    Number(Option<f64>),
    AutoQuery(ZsAutoSuggestTextChange),
    AutoChosen(ZsAutoSuggestionId),
    AutoSubmitted(ZsAutoSuggestSubmission),
    Combo(usize),
    ComboExpanded(bool),
    Date(ZsDate),
    DateExpanded(bool),
    Time(ZsTime),
    TimeExpanded(bool),
    Color(Color),
    ColorChannel(ZsColorChannel),
    ColorExpanded(bool),
    List(usize),
    GridSelected(ZsGridViewItemId),
    GridInvoked(ZsGridViewItemId),
    TreeSelected(ZsTreeNodeId),
    TreeExpanded(ZsTreeExpansionChange),
    TreeInvoked(ZsTreeNodeId),
    TableSelected(ZsTableRowId),
    TableSorted(ZsTableSort),
    TableInvoked(ZsTableRowId),
    Tab(ZsTabId),
    Breadcrumb(ZsBreadcrumbId),
    BreadcrumbExpanded(bool),
    PaletteOpen(bool),
    PaletteQuery(String),
    PaletteHighlight(ZsCommandPaletteItemId),
    PaletteInvoke(ZsCommandPaletteItemId),
    OpenOverlay(GalleryOverlay),
    DialogResult(ZsContentDialogResult),
    ToastResult(ZsToastResult),
    TeachingTipResult(ZsTeachingTipResult),
    InfoBar(ZsInfoBarEvent),
}

fn card(title: impl Into<String>, children: Vec<ViewNode<Msg>>) -> ViewNode<Msg> {
    let mut nodes = Vec::with_capacity(children.len() + 1);
    nodes.push(text(title.into()).height(Dp::new(24.0)));
    nodes.extend(children);
    column(nodes)
        .flex(1.0)
        .padding(Dp::new(16.0))
        .gap(Dp::new(10.0))
        .radius(Dp::new(8.0))
        .bg(ThemeColorToken::SurfaceRaised)
}

fn inputs_page(state: &GalleryState) -> ViewNode<Msg> {
    let actions = card(
        "Buttons and choices",
        vec![
            row([
                button(format!("Save ({})", state.click_count))
                    .id(PRIMARY_ACTION)
                    .tooltip("Invokes a typed application message")
                    .on_click(Msg::PrimaryAction),
                toggle_button("Pinned", state.toggle_button).on_toggle(Msg::ToggleButton),
            ])
            .gap(Dp::new(12.0)),
            checkbox("Automatic updates", state.checkbox)
                .id(CHECKBOX_INPUT)
                .on_toggle(Msg::Checkbox),
            row([
                text("Notifications"),
                spacer(),
                toggle(state.toggle).id(TOGGLE_INPUT).on_toggle(Msg::Toggle),
            ])
            .gap(Dp::new(8.0)),
            row([
                text("Dark mode"),
                spacer(),
                toggle(state.dark).on_toggle(Msg::Dark),
            ])
            .gap(Dp::new(8.0)),
            row([
                radio_button("Balanced", state.radio == 0).on_choose(Msg::Radio(0)),
                radio_button("Performance", state.radio == 1).on_choose(Msg::Radio(1)),
            ])
            .gap(Dp::new(12.0)),
            text(format!("Slider: {:.0}", state.slider)),
            slider(state.slider, SliderRange::new(0.0, 100.0)).on_slide(Msg::Slider),
            progress_bar(state.slider, ProgressRange::new(0.0, 100.0)),
            row([
                progress_ring(ZsProgressRingSpec::indeterminate()),
                progress_ring(ZsProgressRingSpec::determinate(
                    state.slider,
                    ProgressRange::new(0.0, 100.0),
                )),
                spacer(),
            ])
            .height(Dp::new(40.0))
            .gap(Dp::new(12.0)),
        ],
    );

    let editors = card(
        "Text and selection",
        vec![
            textbox(&state.text).id(TEXT_INPUT).on_change(Msg::Text),
            password_box(&state.password)
                .id(PASSWORD_INPUT)
                .reveal_mode(ZsPasswordRevealMode::Peek)
                .on_password_change(Msg::Password),
            number_box(state.number, ZsNumberRange::new(0.0, 100.0).step(0.5))
                .fraction_digits(1)
                .on_number_change(Msg::Number),
            auto_suggest_box(
                state.auto_query.clone(),
                [
                    ZsAutoSuggestion::new(1_u64, "Windows"),
                    ZsAutoSuggestion::new(2_u64, "macOS"),
                    ZsAutoSuggestion::new(3_u64, "Linux"),
                ],
            )
            .id(AUTO_SUGGEST)
            .placeholder("Search a desktop platform")
            .expanded(!state.auto_query.is_empty())
            .on_auto_suggest_text_change(Msg::AutoQuery)
            .on_suggestion_chosen(Msg::AutoChosen)
            .on_query_submit(Msg::AutoSubmitted),
            combo_box(["Balanced", "Fast", "Quiet"], state.combo)
                .id(COMBO_INPUT)
                .expanded(state.combo_expanded)
                .on_select(Msg::Combo)
                .on_expanded_change(Msg::ComboExpanded),
            date_picker(state.date)
                .id(DATE_INPUT)
                .expanded(state.date_expanded)
                .on_date_change(Msg::Date)
                .on_expanded_change(Msg::DateExpanded),
            time_picker(state.time)
                .id(TIME_INPUT)
                .minute_increment(ZsMinuteIncrement::FIFTEEN)
                .clock_format(ZsClockFormat::TwentyFourHour)
                .expanded(state.time_expanded)
                .on_time_change(Msg::Time)
                .on_expanded_change(Msg::TimeExpanded),
            color_picker(state.color)
                .id(COLOR_INPUT)
                .on_color_change(Msg::Color)
                .on_color_channel_change(Msg::ColorChannel)
                .on_expanded_change(Msg::ColorExpanded),
        ],
    );

    row([actions, editors]).flex(1.0).gap(Dp::new(16.0))
}

fn collections_page(state: &GalleryState) -> ViewNode<Msg> {
    let tree = tree_view([ZsTreeNode::new(1, "Workspace")
        .icon(ZsIcon::Folder)
        .children([
            ZsTreeNode::new(2, "src").icon(ZsIcon::Folder).children([
                ZsTreeNode::new(3, "lib.rs").icon(ZsIcon::File),
                ZsTreeNode::new(4, "view").icon(ZsIcon::Folder),
            ]),
            ZsTreeNode::new(5, "examples").icon(ZsIcon::Folder),
            ZsTreeNode::new(6, "Cargo.toml").icon(ZsIcon::File),
        ])])
    .id(TREE_VIEW)
    .height(Dp::new(210.0))
    .selected_tree_node(state.tree_selection)
    .expanded_tree_nodes(state.tree_expanded.iter().copied())
    .on_tree_select(Msg::TreeSelected)
    .on_tree_expansion_change(Msg::TreeExpanded)
    .on_tree_invoke(Msg::TreeInvoked);

    let list = scroll(
        list(
            ["Button", "TextBox", "ToggleSwitch", "TreeView", "DataGrid"],
            |label| text(label).height(Dp::new(32.0)),
        )
        .id(LIST_VIEW)
        .selected_index(state.list_selection)
        .on_select(Msg::List),
    )
    .height(Dp::new(138.0))
    .content_height(Dp::new(168.0));

    let virtualized = virtual_list(
        10_000,
        (240..246).map(|index| (index, format!("Virtual row {index}"))),
        |index, label| text(label).id(WidgetId::new(10_000 + index as u64)),
    )
    .id(VIRTUAL_LIST_VIEW)
    .height(Dp::new(144.0));

    let tiles = grid_view([
        ZsGridViewItem::new(1, "Desktop")
            .subtitle("Folder")
            .icon(ZsIcon::Folder),
        ZsGridViewItem::new(2, "Documents")
            .subtitle("Folder")
            .icon(ZsIcon::Folder),
        ZsGridViewItem::new(3, "Photos")
            .subtitle("Collection")
            .icon(ZsIcon::Image),
        ZsGridViewItem::new(4, "README")
            .subtitle("Markdown")
            .icon(ZsIcon::Text),
    ])
    .id(GRID_VIEW)
    .height(Dp::new(246.0))
    .selected_grid_view_item(state.grid_selection)
    .on_grid_view_select(Msg::GridSelected)
    .on_grid_view_invoke(Msg::GridInvoked);

    let table = data_grid(
        [
            ZsTableColumn::new(1, "Name").fill_width(2).sortable(true),
            ZsTableColumn::new(2, "Type").fill_width(1).sortable(true),
            ZsTableColumn::new(3, "Size")
                .fixed_width(Dp::new(88.0))
                .alignment(HorizontalAlign::End),
        ],
        [
            ZsTableRow::new(1, ["Cargo.toml", "Manifest", "4 KB"]),
            ZsTableRow::new(2, ["src", "Folder", "—"]),
            ZsTableRow::new(3, ["README.md", "Markdown", "12 KB"]),
            ZsTableRow::new(4, ["examples", "Folder", "—"]),
        ],
    )
    .id(TABLE_VIEW)
    .height(Dp::new(250.0))
    .selected_table_row(state.table_selection)
    .table_sort(state.table_sort)
    .on_table_select(Msg::TableSelected)
    .on_table_sort(Msg::TableSorted)
    .on_table_invoke(Msg::TableInvoked);

    row([
        card("Lists and hierarchy", vec![tree, list, virtualized]),
        card("Tiles and data", vec![tiles, table]),
    ])
    .flex(1.0)
    .gap(Dp::new(16.0))
}

fn navigation_page(state: &GalleryState) -> ViewNode<Msg> {
    let breadcrumb = breadcrumb_bar([
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(1), "Home"),
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(2), "Components"),
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(3), "Navigation"),
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(4), "Current page"),
    ])
    .id(BREADCRUMB)
    .expanded(state.breadcrumb_expanded)
    .on_expanded_change(Msg::BreadcrumbExpanded)
    .on_breadcrumb_select(Msg::Breadcrumb);

    let tabs = tab_view(
        [
            ZsTabItem::new(
                ZsTabId::new(1),
                "General",
                column([
                    text("Shared state owns the selected page."),
                    checkbox("Show navigation labels", true),
                ])
                .padding(Dp::new(16.0))
                .gap(Dp::new(12.0)),
            ),
            ZsTabItem::new(
                ZsTabId::new(2),
                "Advanced",
                column([
                    text("Keyboard and pointer selection use typed messages."),
                    toggle_button("Compact mode", false),
                ])
                .padding(Dp::new(16.0))
                .gap(Dp::new(12.0)),
            ),
            ZsTabItem::new(
                ZsTabId::new(3),
                "About",
                text("ZSUI v0.2 component gallery"),
            ),
        ],
        Some(state.tab),
    )
    .id(TABS)
    .height(Dp::new(320.0))
    .on_tab_select(Msg::Tab);

    let page = column([
        card("BreadcrumbBar", vec![breadcrumb]),
        card("TabView", vec![tabs]),
        row([
            button("Open command palette").on_click(Msg::PaletteOpen(true)),
            text("Ctrl+Shift+P style keyboard-first surface"),
        ])
        .gap(Dp::new(12.0)),
    ])
    .flex(1.0)
    .gap(Dp::new(16.0));

    command_palette(
        COMMAND_PALETTE,
        state.palette_open,
        state.palette_query.clone(),
        [
            ZsCommandPaletteItem::new(1, "Open file")
                .icon(ZsIcon::File)
                .shortcut("Ctrl+O"),
            ZsCommandPaletteItem::new(2, "Open settings")
                .icon(ZsIcon::Settings)
                .shortcut("Ctrl+,"),
            ZsCommandPaletteItem::new(3, "Toggle theme")
                .icon(ZsIcon::App)
                .shortcut("Ctrl+T"),
        ],
        page,
    )
    .on_command_palette_query_change(Msg::PaletteQuery)
    .on_command_palette_highlight_change(Msg::PaletteHighlight)
    .on_command_palette_invoke(Msg::PaletteInvoke)
    .on_command_palette_open_change(Msg::PaletteOpen)
}

fn feedback_page(state: &GalleryState) -> ViewNode<Msg> {
    let page = column([
        info_bar(
            INFO_BAR,
            ZsInfoBarSpec::new("All changes are stored in the typed gallery state.")
                .title("Native feedback")
                .severity(ZsInfoBarSeverity::Success)
                .action("Details"),
        )
        .on_info_bar_event(Msg::InfoBar),
        card(
            "Overlay surfaces",
            vec![
                row([
                    button("Content dialog")
                        .on_click(Msg::OpenOverlay(GalleryOverlay::Dialog)),
                    button("Toast").on_click(Msg::OpenOverlay(GalleryOverlay::Toast)),
                    button("Teaching tip")
                        .id(TEACHING_TARGET)
                        .on_click(Msg::OpenOverlay(GalleryOverlay::TeachingTip)),
                ])
                .gap(Dp::new(12.0)),
                text("Dialog owns a modal focus scope. Toast and teaching tip remain in the shared View tree."),
                button("Hover for Tooltip")
                    .tooltip_spec(ZsTooltipSpec::new("Native delayed tooltip").open_delay_ms(250)),
            ],
        ),
        card(
            "Status",
            vec![
                text(&state.status),
                row([
                    progress_bar(68.0, ProgressRange::new(0.0, 100.0)).flex(1.0),
                    progress_ring(ZsProgressRingSpec::determinate(
                        68.0,
                        ProgressRange::new(0.0, 100.0),
                    )),
                ])
                .height(Dp::new(40.0))
                .gap(Dp::new(12.0)),
            ],
        ),
    ])
    .flex(1.0)
    .gap(Dp::new(16.0));

    let page = teaching_tip(
        TEACHING_TIP,
        state.overlay == Some(GalleryOverlay::TeachingTip),
        TEACHING_TARGET,
        ZsTeachingTipSpec::new(
            "Teaching tip",
            "This surface tracks a stable target without a WebView.",
        )
        .action("Got it"),
        page,
    )
    .on_teaching_tip_result(Msg::TeachingTipResult);
    let page = toast_presenter(
        TOAST,
        (state.overlay == Some(GalleryOverlay::Toast)).then(|| {
            ZsToastSpec::new(1, "Settings saved")
                .action("Undo")
                .duration(ZsToastDuration::Persistent)
        }),
        page,
    )
    .on_toast_result(Msg::ToastResult);
    content_dialog(
        DIALOG,
        state.overlay == Some(GalleryOverlay::Dialog),
        ZsContentDialogSpec::new("Choose how to continue.", "Cancel")
            .title("Save changes?")
            .primary_button("Save")
            .secondary_button("Discard")
            .default_button(ZsContentDialogButton::Primary),
        page,
    )
    .on_dialog_result(Msg::DialogResult)
}

fn catalog_page() -> ViewNode<Msg> {
    let summary = zsui_component_catalog_summary();
    let contract_only = zsui_component_catalog()
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::ContractOnly)
        .map(|component| component.component_name)
        .collect::<Vec<_>>()
        .join(", ");
    let mut categories = Vec::new();
    for category in [
        ZsuiComponentCategory::Layout,
        ZsuiComponentCategory::Navigation,
        ZsuiComponentCategory::Input,
        ZsuiComponentCategory::Collection,
        ZsuiComponentCategory::Feedback,
        ZsuiComponentCategory::Overlay,
        ZsuiComponentCategory::Media,
        ZsuiComponentCategory::Composite,
    ] {
        let names = zsui_component_catalog()
            .iter()
            .filter(|component| component.category == category)
            .map(|component| component.component_name)
            .collect::<Vec<_>>();
        categories.push(text(format!("{}", category.category_name())).height(Dp::new(22.0)));
        categories.extend(
            names
                .chunks(4)
                .map(|names| text(names.join(", ")).height(Dp::new(24.0))),
        );
    }

    let inventory = scroll(column(categories).gap(Dp::new(6.0)))
        .height(Dp::new(340.0))
        .content_height(Dp::new(620.0));
    let layout_sample = grid(
        [ZsGridTrack::FLEX, ZsGridTrack::FLEX],
        [
            ZsGridTrack::fixed(Dp::new(56.0)),
            ZsGridTrack::fixed(Dp::new(56.0)),
        ],
        [
            ZsGridCell::new(
                0,
                0,
                text("Grid cell A")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            ),
            ZsGridCell::new(
                0,
                1,
                text("Grid cell B")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            ),
            ZsGridCell::new(
                1,
                0,
                text("Spanning layout")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            )
            .column_span(ZsGridSpan::TWO),
        ],
    )
    .column_gap(Dp::new(8.0))
    .row_gap(Dp::new(8.0));

    column([
        info_bar(
            WidgetId::new(500),
            ZsInfoBarSpec::new(format!(
                "{} runtime surfaces; {} contract-only families.",
                summary.runtime_surface_count, summary.contract_only_count
            ))
            .title(format!("{} catalog families", summary.total_count))
            .severity(ZsInfoBarSeverity::Informational),
        ),
        row([
            card("Feature-gated inventory", vec![inventory]),
            card(
                "Layout and contracts",
                vec![
                    layout_sample,
                    text(format!("Contract only: {contract_only}")),
                    text("Default build: window, button and label"),
                    text("Gallery build: explicit component-gallery-demo feature"),
                ],
            ),
        ])
        .flex(1.0)
        .gap(Dp::new(16.0)),
    ])
    .flex(1.0)
    .gap(Dp::new(16.0))
}

fn view(state: &GalleryState) -> ViewNode<Msg> {
    let summary = zsui_component_catalog_summary();
    let mut navigation = vec![
        text("ZSUI Gallery").height(Dp::new(32.0)),
        text(format!(
            "{} runtime surfaces",
            summary.runtime_surface_count
        ))
        .height(Dp::new(28.0)),
    ];
    navigation.extend(GalleryPage::ALL.into_iter().map(|(page, label, id)| {
        button(if page == state.page {
            format!("• {label}")
        } else {
            label.to_string()
        })
        .id(id)
        .width(Dp::new(184.0))
        .height(Dp::new(40.0))
        .on_click(Msg::Navigate(page))
    }));
    navigation.push(spacer());
    navigation.push(
        row([
            text(if state.dark { "Dark" } else { "Light" }),
            spacer(),
            toggle(state.dark).on_toggle(Msg::Dark),
        ])
        .gap(Dp::new(8.0)),
    );
    navigation.push(text(&state.status).height(Dp::new(42.0)));

    let navigation = column(navigation)
        .width(Dp::new(216.0))
        .padding(Dp::new(16.0))
        .gap(Dp::new(8.0))
        .bg(ThemeColorToken::SurfaceRaised);
    let page = match state.page {
        GalleryPage::Inputs => inputs_page(state),
        GalleryPage::Collections => collections_page(state),
        GalleryPage::Navigation => navigation_page(state),
        GalleryPage::Feedback => feedback_page(state),
        GalleryPage::Catalog => catalog_page(),
    };
    let content = column([
        text(state.page.title()).height(Dp::new(30.0)),
        text(state.page.description()).height(Dp::new(24.0)),
        page,
    ])
    .flex(1.0)
    .padding(Dp::new(24.0))
    .gap(Dp::new(10.0));

    row([navigation, content])
        .gap(Dp::new(0.0))
        .bg(ThemeColorToken::Surface)
        .theme_mode(if state.dark {
            ZsuiThemeMode::Dark
        } else {
            ZsuiThemeMode::Light
        })
}

fn update(state: &mut GalleryState, message: Msg, _cx: &mut AppCx) {
    match message {
        Msg::Navigate(page) => {
            state.page = page;
            state.palette_open = false;
            state.overlay = None;
            state.status = page.title().to_string();
        }
        Msg::Dark(value) => {
            state.dark = value;
            state.status = if value { "Dark theme" } else { "Light theme" }.to_string();
        }
        Msg::PrimaryAction => {
            state.click_count = state.click_count.saturating_add(1);
            state.status = format!("Saved {} time(s)", state.click_count);
        }
        Msg::Text(value) => state.text = value,
        Msg::Password(value) => state.password = value,
        Msg::Checkbox(value) => state.checkbox = value,
        Msg::Toggle(value) => state.toggle = value,
        Msg::ToggleButton(value) => state.toggle_button = value,
        Msg::Radio(value) => state.radio = value,
        Msg::Slider(value) => state.slider = value,
        Msg::Number(value) => state.number = value,
        Msg::AutoQuery(value) => state.auto_query = value.text,
        Msg::AutoChosen(value) => state.status = format!("Suggestion {} selected", value.get()),
        Msg::AutoSubmitted(value) => state.status = format!("Query submitted: {}", value.query),
        Msg::Combo(value) => state.combo = Some(value),
        Msg::ComboExpanded(value) => state.combo_expanded = value,
        Msg::Date(value) => state.date = value,
        Msg::DateExpanded(value) => state.date_expanded = value,
        Msg::Time(value) => state.time = value,
        Msg::TimeExpanded(value) => state.time_expanded = value,
        Msg::Color(value) => state.color.color = value,
        Msg::ColorChannel(value) => state.color.active_channel = value,
        Msg::ColorExpanded(value) => state.color.expanded = value,
        Msg::List(value) => state.list_selection = Some(value),
        Msg::GridSelected(value) => state.grid_selection = Some(value),
        Msg::GridInvoked(value) => state.status = format!("Grid item {} invoked", value.get()),
        Msg::TreeSelected(value) => state.tree_selection = Some(value),
        Msg::TreeExpanded(change) => {
            if change.expanded {
                state.tree_expanded.insert(change.node);
            } else {
                state.tree_expanded.remove(&change.node);
            }
        }
        Msg::TreeInvoked(value) => state.status = format!("Tree node {} invoked", value.get()),
        Msg::TableSelected(value) => state.table_selection = Some(value),
        Msg::TableSorted(value) => state.table_sort = Some(value),
        Msg::TableInvoked(value) => state.status = format!("Table row {} invoked", value.get()),
        Msg::Tab(value) => state.tab = value,
        Msg::Breadcrumb(value) => state.status = format!("Breadcrumb {} selected", value.get()),
        Msg::BreadcrumbExpanded(value) => state.breadcrumb_expanded = value,
        Msg::PaletteOpen(value) => state.palette_open = value,
        Msg::PaletteQuery(value) => state.palette_query = value,
        Msg::PaletteHighlight(value) => {
            state.status = format!("Command {} highlighted", value.get())
        }
        Msg::PaletteInvoke(value) => {
            state.palette_open = false;
            state.status = format!("Command {} invoked", value.get());
        }
        Msg::OpenOverlay(value) => state.overlay = Some(value),
        Msg::DialogResult(value) => {
            state.overlay = None;
            state.status = format!("Dialog: {value:?}");
        }
        Msg::ToastResult(value) => {
            state.overlay = None;
            state.status = format!("Toast: {:?}", value.response);
        }
        Msg::TeachingTipResult(value) => {
            state.overlay = None;
            state.status = format!("Teaching tip: {:?}", value.response);
        }
        Msg::InfoBar(value) => state.status = format!("InfoBar: {value:?}"),
    }
}

fn main() -> ZsuiResult<()> {
    let args = env::args().collect::<Vec<_>>();
    if let Some(path) = args
        .windows(2)
        .find(|pair| pair[0] == "--catalog-json")
        .map(|pair| pair[1].clone())
    {
        let document = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "summary": zsui_component_catalog_summary(),
            "components": zsui_component_catalog(),
        });
        fs::write(
            path,
            serde_json::to_vec_pretty(&document).map_err(|error| {
                ZsuiError::host("serialize_component_catalog", error.to_string())
            })?,
        )
        .map_err(|error| ZsuiError::host("write_component_catalog", error.to_string()))?;
        return Ok(());
    }
    let initial_page = args
        .windows(2)
        .find(|pair| pair[0] == "--page")
        .and_then(|pair| GalleryPage::from_slug(&pair[1]))
        .unwrap_or(GalleryPage::Inputs);
    let mut state = GalleryState::default();
    state.page = initial_page;
    let builder = native_window("ZSUI v0.2 Component Gallery")
        .size(1180, 780)
        .min_size(980, 680)
        .stateful_view(state, view, update);
    if args.iter().any(|argument| argument == "--smoke") {
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone())
            .unwrap_or_else(|| {
                format!("target/zsui-component-gallery/{}.png", initial_page.slug())
            });
        let report_path = args
            .windows(2)
            .find(|pair| pair[0] == "--report")
            .map(|pair| pair[1].clone())
            .unwrap_or_else(|| {
                format!(
                    "target/zsui-component-gallery/{}-report.json",
                    initial_page.slug()
                )
            });
        if let Some(parent) = std::path::Path::new(&screenshot).parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ZsuiError::host("create_gallery_artifact_dir", error.to_string())
            })?;
        }
        let mut options = NativeWindowSmokeRunOptions::new(1_500)
            .screenshot_file(&screenshot)
            .require_screenshot(cfg!(windows));
        if initial_page == GalleryPage::Inputs {
            let click_points = {
                let interaction = builder.native_view_interaction_plan().ok_or_else(|| {
                    ZsuiError::host("gallery_interaction_plan", "missing View input plan")
                })?;
                let center = |widget| {
                    interaction
                        .hit_target_for_widget(widget)
                        .map(|target| Point {
                            x: target.bounds.x + target.bounds.width / 2,
                            y: target.bounds.y + target.bounds.height / 2,
                        })
                        .ok_or_else(|| {
                            ZsuiError::host(
                                "gallery_interaction_target",
                                format!("missing gallery widget {}", widget.0),
                            )
                        })
                };
                [
                    center(PRIMARY_ACTION)?,
                    center(CHECKBOX_INPUT)?,
                    center(TOGGLE_INPUT)?,
                ]
            };
            for point in click_points {
                options = options.native_view_click(point);
            }
        }
        let report = builder.run_smoke(options)?;
        fs::write(
            report_path,
            serde_json::to_vec_pretty(&report)
                .map_err(|error| ZsuiError::host("serialize_gallery_report", error.to_string()))?,
        )
        .map_err(|error| ZsuiError::host("write_gallery_report", error.to_string()))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        return Ok(());
    }
    builder.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_declares_every_catalog_family_and_keeps_contracts_explicit() {
        let summary = zsui_component_catalog_summary();
        assert_eq!(summary.total_count, 48);
        assert_eq!(summary.runtime_surface_count, 45);
        assert_eq!(summary.contract_only_count, 3);
    }

    #[test]
    fn gallery_default_page_has_compact_interactive_bounds() {
        let mut page = view(&GalleryState::default());
        let output = View::layout(
            &mut page,
            &mut ViewLayoutCx::new(
                Rect {
                    x: 0,
                    y: 0,
                    width: 1180,
                    height: 780,
                },
                Dpi::standard(),
            ),
        );
        let save = output
            .children
            .iter()
            .find(|node| node.component == PRIMARY_ACTION.into())
            .expect("save button should be present");
        let text = output
            .children
            .iter()
            .find(|node| node.component == TEXT_INPUT.into())
            .expect("text input should be present");
        assert_eq!(save.bounds.height, 32);
        assert_eq!(text.bounds.height, 32);
        assert!(save.bounds.width >= 120);
        assert!(text.bounds.width > save.bounds.width);
    }
}
