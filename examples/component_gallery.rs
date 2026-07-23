#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{collections::BTreeSet, env, fs};

use zsui::*;

const NAV_INPUTS: WidgetId = WidgetId::new(10);
const NAV_COLLECTIONS: WidgetId = WidgetId::new(11);
const NAV_NAVIGATION: WidgetId = WidgetId::new(12);
const NAV_FEEDBACK: WidgetId = WidgetId::new(13);
const NAV_CATALOG: WidgetId = WidgetId::new(14);
const NAVIGATION_VIEW: WidgetId = WidgetId::new(15);
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
const FLYOUT: WidgetId = WidgetId::new(405);
const FLYOUT_TARGET: WidgetId = WidgetId::new(406);
const FLYOUT_ACTION: WidgetId = WidgetId::new(407);
const MENU_FLYOUT: WidgetId = WidgetId::new(408);
const MENU_FLYOUT_TARGET: WidgetId = WidgetId::new(409);
const CANVAS_SURFACE: WidgetId = WidgetId::new(501);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryPage {
    Inputs,
    Collections,
    Navigation,
    Feedback,
    Catalog,
}

impl GalleryPage {
    const ALL: [(Self, WidgetId); 5] = [
        (Self::Inputs, NAV_INPUTS),
        (Self::Collections, NAV_COLLECTIONS),
        (Self::Navigation, NAV_NAVIGATION),
        (Self::Feedback, NAV_FEEDBACK),
        (Self::Catalog, NAV_CATALOG),
    ];

    const fn title(self) -> &'static str {
        match self {
            Self::Inputs => "输入与操作 / Inputs",
            Self::Collections => "集合 / Collections",
            Self::Navigation => "导航 / Navigation",
            Self::Feedback => "反馈与浮层 / Feedback",
            Self::Catalog => "目录与布局 / Catalog",
        }
    }

    const fn icon(self) -> ZsIcon {
        match self {
            Self::Inputs => ZsIcon::Tool,
            Self::Collections => ZsIcon::Folder,
            Self::Navigation => ZsIcon::Sidebar,
            Self::Feedback => ZsIcon::Info,
            Self::Catalog => ZsIcon::Group,
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Inputs => "强类型值、焦点和输入状态 / Typed values, focus and input",
            Self::Collections => "列表、虚拟化、树、磁贴与表格 / Lists, trees, tiles and tables",
            Self::Navigation => "面包屑、标签页与键盘命令 / Breadcrumbs, tabs and commands",
            Self::Feedback => "行内状态、模态与非模态浮层 / Inline, modal and nonmodal feedback",
            Self::Catalog => "按特性启用的组件目录与布局 / Feature-gated catalog and layout",
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
            .map(|(page, _)| page)
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
    animations_frozen: bool,
    click_count: u32,
    canvas_activation_count: u32,
    canvas_pointer_count: u32,
    canvas_pointer: Option<ZsCanvasPointerEvent>,
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
    flyout_open: bool,
    menu_flyout_open: bool,
    image_preview: ZsImagePreviewState,
    status: String,
}

impl Default for GalleryState {
    fn default() -> Self {
        let mut image_preview = ZsImagePreviewState::default();
        if let Some(bytes) = ZsIcon::Image.png_24_bytes() {
            image_preview.set_png(ZsImageFrameId::new(1), std::sync::Arc::<[u8]>::from(bytes));
        }
        Self {
            page: GalleryPage::Inputs,
            dark: false,
            animations_frozen: false,
            click_count: 0,
            canvas_activation_count: 0,
            canvas_pointer_count: 0,
            canvas_pointer: None,
            text: "ZSUI 原生界面 / Native UI".to_string(),
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
            flyout_open: false,
            menu_flyout_open: false,
            image_preview,
            status: "就绪 / Ready".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    Navigate(GalleryPage),
    Dark(bool),
    PrimaryAction,
    CanvasActivated,
    CanvasPointer(ZsCanvasPointerEvent),
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
    OpenFlyout,
    FlyoutAction,
    FlyoutDismissed(ZsFlyoutDismissReason),
    OpenMenuFlyout,
    MenuFlyoutCommand(Command),
    MenuFlyoutOpen(bool),
    InfoBar(ZsInfoBarEvent),
}

fn role_text(value: impl Into<String>, role: TextRole) -> ViewNode<Msg> {
    styled_text(value, SemanticTextStyle::for_role(role))
}

fn body_strong(value: impl Into<String>) -> ViewNode<Msg> {
    let mut style = SemanticTextStyle::body();
    style.weight = TextWeight::Semibold;
    styled_text(value, style)
}

fn secondary_text(value: impl Into<String>, role: TextRole) -> ViewNode<Msg> {
    let mut style = SemanticTextStyle::for_role(role);
    style.color = ColorRole::SecondaryText;
    styled_text(value, style)
}

fn status_text(value: impl Into<String>) -> ViewNode<Msg> {
    let mut style = SemanticTextStyle::for_role(TextRole::Caption);
    style.color = ColorRole::SecondaryText;
    style.vertical_align = VerticalAlign::Start;
    style.wrap = TextWrap::Word;
    style.ellipsis = false;
    styled_text(value, style)
}

const fn canvas_phase_label(phase: ZsCanvasPointerPhase) -> &'static str {
    match phase {
        ZsCanvasPointerPhase::Pressed => "按下 / Pressed",
        ZsCanvasPointerPhase::Moved => "拖拽 / Moved",
        ZsCanvasPointerPhase::Released => "释放 / Released",
        ZsCanvasPointerPhase::Cancelled => "取消 / Cancelled",
    }
}

const fn pointer_button_label(button: ZsPointerButton) -> &'static str {
    match button {
        ZsPointerButton::Primary => "主键 / Primary",
        ZsPointerButton::Secondary => "右键 / Secondary",
        ZsPointerButton::Middle => "中键 / Middle",
        ZsPointerButton::Auxiliary(_) => "侧键 / Auxiliary",
    }
}

const fn category_label(category: ZsuiComponentCategory) -> &'static str {
    match category {
        ZsuiComponentCategory::Layout => "布局 / Layout",
        ZsuiComponentCategory::Navigation => "导航 / Navigation",
        ZsuiComponentCategory::Input => "输入 / Input",
        ZsuiComponentCategory::Collection => "集合 / Collection",
        ZsuiComponentCategory::Feedback => "反馈 / Feedback",
        ZsuiComponentCategory::Overlay => "浮层 / Overlay",
        ZsuiComponentCategory::Media => "媒体 / Media",
        ZsuiComponentCategory::Composite => "组合 / Composite",
    }
}

fn component_label(name: &str) -> String {
    let chinese = match name {
        "stack" => "栈布局",
        "grid" => "网格",
        "border" => "边框",
        "scroll" => "滚动视图",
        "split_view" => "拆分视图",
        "canvas" => "画布",
        "navigation" => "导航视图",
        "tabs" => "标签页",
        "breadcrumb" => "面包屑",
        "command_bar" => "命令栏",
        "text" => "文本",
        "button" => "按钮",
        "toggle_button" => "切换按钮",
        "checkbox" => "复选框",
        "toggle" => "切换开关",
        "textbox" => "文本框",
        "password_box" => "密码框",
        "combo_box" => "组合框",
        "radio_button" => "单选按钮",
        "slider" => "滑块",
        "number_box" => "数值框",
        "auto_suggest" => "自动建议框",
        "date_picker" => "日期选择器",
        "time_picker" => "时间选择器",
        "color_picker" => "颜色选择器",
        "list" => "列表",
        "grid_view" => "网格视图",
        "tree" => "树视图",
        "table" => "数据表格",
        "items_repeater" => "项目重复器",
        "badge" => "徽章",
        "progress_bar" => "进度条",
        "progress_ring" => "进度环",
        "toast" => "轻量通知",
        "info_bar" => "信息栏",
        "tooltip" => "工具提示",
        "content_dialog" => "内容对话框",
        "flyout" => "浮出层",
        "menu_flyout" => "菜单浮出层",
        "teaching_tip" => "教学提示",
        "command_palette" => "命令面板",
        "image" => "图像",
        "icon" => "图标",
        "settings_card" => "设置卡片",
        "workbench_shell" => "工作台外壳",
        "message_timeline" => "消息时间线",
        "composer" => "撰写器",
        "inspector_panel" => "检查器面板",
        _ => return name.to_string(),
    };
    format!("{chinese} / {name}")
}

fn card(title: impl Into<String>, children: Vec<ViewNode<Msg>>) -> ViewNode<Msg> {
    section(title, children).flex(1.0)
}

fn compact_card(title: impl Into<String>, children: Vec<ViewNode<Msg>>) -> ViewNode<Msg> {
    section(title, children).flex(0.0)
}

fn inputs_page(state: &GalleryState) -> ViewNode<Msg> {
    let actions = card(
        "按钮与选择 / Buttons and choices",
        vec![
            row([
                button(format!("保存 / Save ({})", state.click_count))
                    .id(PRIMARY_ACTION)
                    .tooltip("发送强类型应用消息 / Sends a typed message")
                    .on_click(Msg::PrimaryAction),
                toggle_button("固定 / Pinned", state.toggle_button).on_toggle(Msg::ToggleButton),
            ])
            .gap(Dp::new(12.0)),
            checkbox("自动更新 / Automatic updates", state.checkbox)
                .id(CHECKBOX_INPUT)
                .on_toggle(Msg::Checkbox),
            row([
                text("通知 / Notifications"),
                spacer(),
                toggle(state.toggle).id(TOGGLE_INPUT).on_toggle(Msg::Toggle),
            ])
            .gap(Dp::new(8.0)),
            row([
                text("深色模式 / Dark mode"),
                spacer(),
                toggle(state.dark).on_toggle(Msg::Dark),
            ])
            .gap(Dp::new(8.0)),
            row([
                radio_button("均衡 / Balanced", state.radio == 0).on_choose(Msg::Radio(0)),
                radio_button("性能 / Performance", state.radio == 1).on_choose(Msg::Radio(1)),
            ])
            .gap(Dp::new(12.0)),
            text(format!("滑块 / Slider: {:.0}", state.slider)),
            slider(state.slider, SliderRange::new(0.0, 100.0)).on_slide(Msg::Slider),
            progress_bar(state.slider, ProgressRange::new(0.0, 100.0)),
            row([
                progress_ring(if state.animations_frozen {
                    ZsProgressRingSpec::determinate(state.slider, ProgressRange::new(0.0, 100.0))
                } else {
                    ZsProgressRingSpec::indeterminate()
                }),
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
        "文本与选择 / Text and selection",
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
            .placeholder("搜索桌面平台 / Search platform")
            .expanded(!state.auto_query.is_empty())
            .on_auto_suggest_text_change(Msg::AutoQuery)
            .on_suggestion_chosen(Msg::AutoChosen)
            .on_query_submit(Msg::AutoSubmitted),
            combo_box(
                ["均衡 / Balanced", "快速 / Fast", "安静 / Quiet"],
                state.combo,
            )
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
    let tree = tree_view([ZsTreeNode::new(1, "工作区 / Workspace")
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
            [
                "按钮 / Button",
                "文本框 / TextBox",
                "切换开关 / ToggleSwitch",
                "树视图 / TreeView",
                "数据表格 / DataGrid",
            ],
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
        (0..6).map(|index| (index, format!("虚拟行 {index} / Virtual row {index}"))),
        |index, label| text(label).id(WidgetId::new(10_000 + index as u64)),
    )
    .id(VIRTUAL_LIST_VIEW)
    .height(Dp::new(144.0));

    let tiles = grid_view([
        ZsGridViewItem::new(1, "桌面 / Desktop")
            .subtitle("文件夹 / Folder")
            .icon(ZsIcon::Folder),
        ZsGridViewItem::new(2, "文档 / Documents")
            .subtitle("文件夹 / Folder")
            .icon(ZsIcon::Folder),
        ZsGridViewItem::new(3, "照片 / Photos")
            .subtitle("集合 / Collection")
            .icon(ZsIcon::Image),
        ZsGridViewItem::new(4, "说明 / README")
            .subtitle("文档 / Markdown")
            .icon(ZsIcon::Text),
    ])
    .id(GRID_VIEW)
    .height(Dp::new(246.0))
    .selected_grid_view_item(state.grid_selection)
    .on_grid_view_select(Msg::GridSelected)
    .on_grid_view_invoke(Msg::GridInvoked);

    let table = data_grid(
        [
            ZsTableColumn::new(1, "名称 / Name")
                .fill_width(1)
                .sortable(true),
            ZsTableColumn::new(2, "类型 / Type")
                .fill_width(1)
                .sortable(true),
            ZsTableColumn::new(3, "大小 / Size")
                .fill_width(1)
                .alignment(HorizontalAlign::End),
        ],
        [
            ZsTableRow::new(1, ["Cargo.toml", "清单 / TOML", "4 KB"]),
            ZsTableRow::new(2, ["src", "文件夹 / Dir", "—"]),
            ZsTableRow::new(3, ["README.md", "文档 / MD", "12 KB"]),
            ZsTableRow::new(4, ["examples", "文件夹 / Dir", "—"]),
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
        card(
            "列表与层级 / Lists and hierarchy",
            vec![tree, list, virtualized],
        ),
        card("磁贴与数据 / Tiles and data", vec![tiles, table]),
    ])
    .flex(1.0)
    .gap(Dp::new(16.0))
}

fn navigation_page(state: &GalleryState) -> ViewNode<Msg> {
    let breadcrumb = breadcrumb_bar([
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(1), "首页"),
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(2), "组件"),
        ZsBreadcrumbItem::new(ZsBreadcrumbId::new(3), "当前页面"),
    ])
    .id(BREADCRUMB)
    .expanded(state.breadcrumb_expanded)
    .on_expanded_change(Msg::BreadcrumbExpanded)
    .on_breadcrumb_select(Msg::Breadcrumb);

    let tabs = tab_view(
        [
            ZsTabItem::new(
                ZsTabId::new(1),
                "常规 / General",
                column([
                    text("共享状态持有所选页面 / Shared state owns the selected page"),
                    checkbox("显示导航标签 / Show navigation labels", true),
                ])
                .padding(Dp::new(16.0))
                .gap(Dp::new(12.0)),
            )
            .icon(ZsIcon::Settings),
            ZsTabItem::new(
                ZsTabId::new(2),
                "高级 / Advanced",
                column([
                    text("键盘和指针选择使用强类型消息 / Typed keyboard and pointer selection"),
                    toggle_button("紧凑模式 / Compact mode", false),
                ])
                .padding(Dp::new(16.0))
                .gap(Dp::new(12.0)),
            )
            .icon(ZsIcon::Tool),
            ZsTabItem::new(
                ZsTabId::new(3),
                "关于 / About",
                text("ZSUI v0.2 组件库 / Component gallery"),
            )
            .icon(ZsIcon::Info),
        ],
        Some(state.tab),
    )
    .id(TABS)
    .on_tab_select(Msg::Tab);

    let page = column([
        compact_card("面包屑 / BreadcrumbBar", vec![breadcrumb]),
        card("标签页 / TabView", vec![tabs]),
        row([
            button("打开命令面板 / Open palette").on_click(Msg::PaletteOpen(true)),
            text("键盘优先的 Ctrl+Shift+P 式界面 / Keyboard-first surface"),
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
            ZsCommandPaletteItem::new(1, "打开文件 / Open file")
                .icon(ZsIcon::File)
                .shortcut("Ctrl+O"),
            ZsCommandPaletteItem::new(2, "打开设置 / Open settings")
                .icon(ZsIcon::Settings)
                .shortcut("Ctrl+,"),
            ZsCommandPaletteItem::new(3, "切换主题 / Toggle theme")
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

fn gallery_menu_flyout_spec() -> MenuSpec {
    let mut menu = MenuSpec::new().id("gallery.feedback.actions");
    menu.items.push(
        MenuItemSpec::command("保存 / Save", Command::custom("gallery.save"))
            .accelerator(ZsAccelerator::primary_character('s')),
    );
    menu.items.push(MenuItemSpec::Separator);
    menu.items.push(
        MenuItemSpec::command("自动保存 / Auto save", Command::custom("gallery.autosave"))
            .checked(true),
    );
    menu.items.push(MenuItemSpec::Submenu {
        id: Some("gallery.feedback.more".to_string()),
        label: "更多 / More".to_string(),
        enabled: true,
        menu: MenuSpec::new()
            .submenu(
                "导出 / Export",
                MenuSpec::new().item(
                    "PDF 文档 / PDF document",
                    Command::custom("gallery.export.pdf"),
                ),
            )
            .item("复制 / Copy", Command::custom("gallery.copy"))
            .item("共享 / Share", Command::custom("gallery.share")),
    });
    menu
}

fn feedback_page(state: &GalleryState) -> ViewNode<Msg> {
    let page = column([
        info_bar(
            INFO_BAR,
            ZsInfoBarSpec::new("所有更改均保存在强类型状态中 / Changes stay in typed state")
                .title("原生反馈 / Native feedback")
                .severity(ZsInfoBarSeverity::Success)
                .action("详情 / Details"),
        )
        .on_info_bar_event(Msg::InfoBar),
        card(
            "浮层界面 / Overlay surfaces",
            vec![
                row([
                    button("内容对话框 / Dialog")
                        .on_click(Msg::OpenOverlay(GalleryOverlay::Dialog)),
                    button("轻量通知 / Toast").on_click(Msg::OpenOverlay(GalleryOverlay::Toast)),
                ])
                .gap(Dp::new(12.0)),
                row([
                    button("教学提示 / Tip")
                        .id(TEACHING_TARGET)
                        .on_click(Msg::OpenOverlay(GalleryOverlay::TeachingTip)),
                    button("弹出视图 / Flyout")
                        .id(FLYOUT_TARGET)
                        .on_click(Msg::OpenFlyout),
                    button("菜单 / Menu")
                        .id(MENU_FLYOUT_TARGET)
                        .on_click(Msg::OpenMenuFlyout),
                ])
                .gap(Dp::new(12.0)),
                column([
                    text("对话框与弹出视图拥有受限焦点域；所有浮层仍在共享视图树中"),
                    secondary_text(
                        "Dialog and Flyout constrain focus; every overlay stays in the shared View tree",
                        TextRole::Caption,
                    ),
                ])
                .gap(Dp::new(2.0)),
                button("悬停显示工具提示 / Hover for Tooltip").tooltip_spec(
                    ZsTooltipSpec::new("原生延时工具提示 / Native delayed tooltip")
                        .open_delay_ms(250),
                ),
            ],
        ),
        card(
            "状态 / Status",
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

    let page = flyout(
        FLYOUT,
        state.flyout_open,
        FLYOUT_TARGET,
        ZsFlyoutSpec::new(Dp::new(360.0), Dp::new(132.0)),
        column([
            body_strong("平台弹出视图 / Platform popover"),
            secondary_text(
                "同一段 View 内容使用各平台的独立外观参数",
                TextRole::Caption,
            ),
            secondary_text(
                "One View tree uses platform-specific composition",
                TextRole::Caption,
            ),
            primary_button("应用 / Apply")
                .id(FLYOUT_ACTION)
                .on_click(Msg::FlyoutAction),
        ])
        .gap(Dp::new(8.0)),
        page,
    )
    .on_flyout_dismiss(Msg::FlyoutDismissed);

    let page = menu_flyout(
        MENU_FLYOUT,
        state.menu_flyout_open,
        MENU_FLYOUT_TARGET,
        gallery_menu_flyout_spec(),
        page,
    )
    .on_menu_flyout_command(Msg::MenuFlyoutCommand)
    .on_menu_flyout_open_change(Msg::MenuFlyoutOpen);

    let page = teaching_tip(
        TEACHING_TIP,
        state.overlay == Some(GalleryOverlay::TeachingTip),
        TEACHING_TARGET,
        ZsTeachingTipSpec::new(
            "教学提示 / Teaching tip",
            "该界面跟随稳定目标，不使用 WebView / Tracks a stable target without WebView",
        )
        .action("知道了 / Got it"),
        page,
    )
    .on_teaching_tip_result(Msg::TeachingTipResult);
    let page = toast_presenter(
        TOAST,
        (state.overlay == Some(GalleryOverlay::Toast)).then(|| {
            ZsToastSpec::new(1, "设置已保存 / Settings saved")
                .action("撤销 / Undo")
                .duration(ZsToastDuration::Persistent)
        }),
        page,
    )
    .on_toast_result(Msg::ToastResult);
    content_dialog(
        DIALOG,
        state.overlay == Some(GalleryOverlay::Dialog),
        ZsContentDialogSpec::new("请选择后续操作 / Choose how to continue", "取消 / Cancel")
            .title("保存更改？ / Save changes?")
            .primary_button("保存 / Save")
            .secondary_button("放弃 / Discard")
            .default_button(ZsContentDialogButton::Primary),
        page,
    )
    .on_dialog_result(Msg::DialogResult)
}

fn catalog_page(state: &GalleryState) -> ViewNode<Msg> {
    let summary = zsui_component_catalog_summary();
    let contract_only = zsui_component_catalog()
        .iter()
        .filter(|component| component.status == ZsuiComponentStatus::ContractOnly)
        .map(|component| component_label(component.component_name))
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
            .map(|component| component_label(component.component_name))
            .collect::<Vec<_>>();
        categories.push(body_strong(category_label(category)).height(Dp::new(22.0)));
        categories.extend(
            names
                .chunks(2)
                .map(|names| text(names.join(", ")).height(Dp::new(24.0))),
        );
    }

    let inventory = scroll(column(categories).gap(Dp::new(6.0)))
        .height(Dp::new(324.0))
        .content_height(Dp::new(960.0));
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
                text("网格单元 A / Grid cell A")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            ),
            ZsGridCell::new(
                0,
                1,
                text("网格单元 B / Grid cell B")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            ),
            ZsGridCell::new(
                1,
                0,
                text("跨列布局 / Spanning layout")
                    .bg(ThemeColorToken::Control)
                    .padding(Dp::new(12.0)),
            )
            .column_span(ZsGridSpan::TWO),
        ],
    )
    .column_gap(Dp::new(8.0))
    .row_gap(Dp::new(8.0));
    let canvas_sample = ZsCanvasScene::new()
        .with(ZsCanvasPrimitive::round_fill(
            ZsCanvasRect::new(Dp::new(0.0), Dp::new(0.0), Dp::new(340.0), Dp::new(84.0)),
            NativeDrawFill::role(ColorRole::Control),
            Dp::new(8.0),
        ))
        .with(ZsCanvasPrimitive::round_fill(
            ZsCanvasRect::new(Dp::new(12.0), Dp::new(12.0), Dp::new(60.0), Dp::new(60.0)),
            NativeDrawFill::role(ColorRole::Accent),
            Dp::new(12.0),
        ))
        .with(ZsCanvasPrimitive::icon(
            ZsIcon::Image,
            ZsCanvasRect::new(Dp::new(30.0), Dp::new(30.0), Dp::new(24.0), Dp::new(24.0)),
            ColorRole::AccentText,
        ))
        .with(ZsCanvasPrimitive::text(
            "Dp + 语义色 / Semantic colors",
            ZsCanvasRect::new(Dp::new(84.0), Dp::new(26.0), Dp::new(244.0), Dp::new(34.0)),
            SemanticTextStyle::body(),
        ));

    column([
        info_bar(
            WidgetId::new(500),
            ZsInfoBarSpec::new(format!(
                "{} 个运行界面，{} 个仅契约组件 / {} runtime surfaces, {} contract-only",
                summary.runtime_surface_count,
                summary.contract_only_count,
                summary.runtime_surface_count,
                summary.contract_only_count
            ))
            .title(format!(
                "{} 个组件家族 / {} component families",
                summary.total_count, summary.total_count
            ))
            .severity(ZsInfoBarSeverity::Informational),
        ),
        row([
            card(
                "按特性启用的目录 / Feature-gated inventory",
                vec![inventory],
            ),
            card(
                "布局与绘制 / Layout and drawing",
                vec![
                    layout_sample,
                    body_strong("自绘画布 / Custom canvas"),
                    canvas(canvas_sample)
                        .id(CANVAS_SURFACE)
                        .height(Dp::new(84.0))
                        .on_click(Msg::CanvasActivated)
                        .on_canvas_pointer(Msg::CanvasPointer),
                    secondary_text(
                        format!(
                            "激活 {} · 指针 {} / Activated {} · Pointer {}",
                            state.canvas_activation_count,
                            state.canvas_pointer_count,
                            state.canvas_activation_count,
                            state.canvas_pointer_count,
                        ),
                        TextRole::Caption,
                    ),
                    secondary_text(
                        state.canvas_pointer.map_or_else(
                            || {
                                "等待按下、拖拽或右键 / Waiting for press, drag or secondary click"
                                    .to_string()
                            },
                            |event| {
                                format!(
                                    "{} · {} · {:.1}, {:.1} dp{}",
                                    canvas_phase_label(event.phase),
                                    pointer_button_label(event.button),
                                    event.position.x.0,
                                    event.position.y.0,
                                    if event.inside { "" } else { " · 外 / Out" },
                                )
                            },
                        ),
                        TextRole::Caption,
                    ),
                    body_strong("保留帧图片 / Retained image"),
                    image_preview(&state.image_preview.snapshot())
                        .height(Dp::new(96.0))
                        .image_fit(ZsImageFit::Contain),
                    body_strong("仅有契约 / Contract only"),
                    text(contract_only),
                    text("默认 / Default: window + button + label"),
                    text("组件库 / Gallery: component-gallery-demo"),
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
    let page = match state.page {
        GalleryPage::Inputs => inputs_page(state),
        GalleryPage::Collections => collections_page(state),
        GalleryPage::Navigation => navigation_page(state),
        GalleryPage::Feedback => feedback_page(state),
        GalleryPage::Catalog => catalog_page(state),
    };
    let spacing = ZsuiSpacingTokens::default();
    let content = column([
        role_text(state.page.title(), TextRole::WindowTitle),
        secondary_text(state.page.description(), TextRole::Body),
        page,
    ])
    .flex(1.0)
    .padding(spacing.page_padding)
    .gap(spacing.content_gap);
    let navigation_items = GalleryPage::ALL.into_iter().map(|(page, id)| {
        navigation_item(page.title(), page.icon(), page == state.page)
            .id(id)
            .on_click(Msg::Navigate(page))
    });
    navigation_view(
        ZsNavigationViewSpec::new(
            "ZSUI 组件库 / Gallery",
            format!(
                "{} 个运行界面 / {} runtime surfaces",
                summary.runtime_surface_count, summary.runtime_surface_count
            ),
        )
        .items(navigation_items)
        .footer_items([
            row([
                text(if state.dark {
                    "深色 / Dark"
                } else {
                    "浅色 / Light"
                }),
                spacer(),
                toggle(state.dark).on_toggle(Msg::Dark),
            ])
            .gap(Dp::new(8.0)),
            status_text(&state.status),
        ])
        .minimum_content_width(Dp::new(560.0))
        .content(NAVIGATION_VIEW, content),
    )
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
            state.flyout_open = false;
            state.menu_flyout_open = false;
            state.status = page.title().to_string();
        }
        Msg::Dark(value) => {
            state.dark = value;
            state.status = if value {
                "深色主题 / Dark theme"
            } else {
                "浅色主题 / Light theme"
            }
            .to_string();
        }
        Msg::PrimaryAction => {
            state.click_count = state.click_count.saturating_add(1);
            state.status = format!(
                "已保存 {} 次 / Saved {} time(s)",
                state.click_count, state.click_count
            );
        }
        Msg::CanvasActivated => {
            state.canvas_activation_count = state.canvas_activation_count.saturating_add(1);
            state.status = format!(
                "画布已激活 {} 次 / Canvas activated {} time(s)",
                state.canvas_activation_count, state.canvas_activation_count
            );
        }
        Msg::CanvasPointer(event) => {
            state.canvas_pointer_count = state.canvas_pointer_count.saturating_add(1);
            state.canvas_pointer = Some(event);
            state.status = format!(
                "画布 {} · {:.1}, {:.1} dp / Canvas",
                canvas_phase_label(event.phase),
                event.position.x.0,
                event.position.y.0,
            );
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
        Msg::AutoChosen(value) => {
            state.status = format!(
                "已选择建议 {} / Suggestion {} selected",
                value.get(),
                value.get()
            )
        }
        Msg::AutoSubmitted(value) => {
            state.status = format!(
                "已提交查询：{} / Query submitted: {}",
                value.query, value.query
            )
        }
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
        Msg::GridInvoked(value) => {
            state.status = format!(
                "已调用网格项 {} / Grid item {} invoked",
                value.get(),
                value.get()
            )
        }
        Msg::TreeSelected(value) => state.tree_selection = Some(value),
        Msg::TreeExpanded(change) => {
            if change.expanded {
                state.tree_expanded.insert(change.node);
            } else {
                state.tree_expanded.remove(&change.node);
            }
        }
        Msg::TreeInvoked(value) => {
            state.status = format!(
                "已调用树节点 {} / Tree node {} invoked",
                value.get(),
                value.get()
            )
        }
        Msg::TableSelected(value) => state.table_selection = Some(value),
        Msg::TableSorted(value) => state.table_sort = Some(value),
        Msg::TableInvoked(value) => {
            state.status = format!(
                "已调用表格行 {} / Table row {} invoked",
                value.get(),
                value.get()
            )
        }
        Msg::Tab(value) => state.tab = value,
        Msg::Breadcrumb(value) => {
            state.status = format!(
                "已选择面包屑 {} / Breadcrumb {} selected",
                value.get(),
                value.get()
            )
        }
        Msg::BreadcrumbExpanded(value) => state.breadcrumb_expanded = value,
        Msg::PaletteOpen(value) => state.palette_open = value,
        Msg::PaletteQuery(value) => state.palette_query = value,
        Msg::PaletteHighlight(value) => {
            state.status = format!(
                "已高亮命令 {} / Command {} highlighted",
                value.get(),
                value.get()
            )
        }
        Msg::PaletteInvoke(value) => {
            state.palette_open = false;
            state.status = format!(
                "已调用命令 {} / Command {} invoked",
                value.get(),
                value.get()
            );
        }
        Msg::OpenOverlay(value) => {
            state.overlay = Some(value);
            state.flyout_open = false;
            state.menu_flyout_open = false;
        }
        Msg::DialogResult(value) => {
            state.overlay = None;
            state.status = format!("对话框 / Dialog: {value:?}");
        }
        Msg::ToastResult(value) => {
            state.overlay = None;
            state.status = format!("轻量通知 / Toast: {:?}", value.response);
        }
        Msg::TeachingTipResult(value) => {
            state.overlay = None;
            state.status = format!("教学提示 / Teaching tip: {:?}", value.response);
        }
        Msg::OpenFlyout => {
            state.overlay = None;
            state.flyout_open = true;
            state.menu_flyout_open = false;
            state.status = "弹出视图已打开 / Flyout opened".to_string();
        }
        Msg::FlyoutAction => {
            state.status = "弹出视图操作已执行 / Flyout action invoked".to_string();
        }
        Msg::FlyoutDismissed(reason) => {
            state.flyout_open = false;
            state.status = format!("弹出视图已关闭 / Flyout dismissed: {reason:?}");
        }
        Msg::OpenMenuFlyout => {
            state.overlay = None;
            state.flyout_open = false;
            state.menu_flyout_open = true;
            state.status = "菜单已打开 / Menu opened".to_string();
        }
        Msg::MenuFlyoutCommand(command) => {
            let (zh, en) = match &command {
                Command::Custom { id, .. } if id == "gallery.save" => ("保存", "Save"),
                Command::Custom { id, .. } if id == "gallery.autosave" => ("自动保存", "Auto save"),
                Command::Custom { id, .. } if id == "gallery.copy" => ("复制", "Copy"),
                Command::Custom { id, .. } if id == "gallery.share" => ("共享", "Share"),
                Command::Custom { id, .. } if id == "gallery.export.pdf" => {
                    ("导出 PDF", "Export PDF")
                }
                _ => ("自定义命令", "Custom command"),
            };
            state.status = format!("菜单命令：{zh} / Menu command: {en}");
        }
        Msg::MenuFlyoutOpen(open) => state.menu_flyout_open = open,
        Msg::InfoBar(value) => state.status = format!("信息栏 / InfoBar: {value:?}"),
    }
}

fn main() -> ZsuiResult<()> {
    let args = env::args().collect::<Vec<_>>();
    let native_proof = args.iter().any(|argument| argument == "--native-proof");
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
    let dark = args
        .windows(2)
        .find(|pair| pair[0] == "--theme")
        .is_some_and(|pair| pair[1] == "dark");
    let mut state = GalleryState::default();
    state.page = initial_page;
    state.dark = dark;
    state.animations_frozen = args.iter().any(|argument| argument == "--benchmark-static");
    state.menu_flyout_open = native_proof && initial_page == GalleryPage::Feedback;
    let default_size = (1180, 780);
    let window_width = proof_dimension(&args, "--width", default_size.0);
    let window_height = proof_dimension(&args, "--height", default_size.1);
    let resize_target = match (
        optional_proof_dimension(&args, "--resize-width"),
        optional_proof_dimension(&args, "--resize-height"),
    ) {
        (Some(width), Some(height)) => Some(Size {
            width: width as i32,
            height: height as i32,
        }),
        (None, None) => None,
        _ => {
            return Err(ZsuiError::host(
                "gallery_native_resize_arguments",
                "--resize-width and --resize-height must be supplied together",
            ));
        }
    };
    let minimum_size = if native_proof {
        (
            window_width
                .min(resize_target.map_or(window_width, |size| size.width.max(1) as u32))
                .min(800),
            window_height
                .min(resize_target.map_or(window_height, |size| size.height.max(1) as u32))
                .min(520),
        )
    } else {
        (980, 680)
    };
    let builder = if args.iter().any(|argument| argument == "--benchmark-empty") {
        native_window("ZSUI 组件库 / Component Gallery")
            .size(window_width, window_height)
            .min_size(minimum_size.0, minimum_size.1)
            .release_view_when_hidden()
    } else {
        native_window("ZSUI 组件库 / Component Gallery")
            .size(window_width, window_height)
            .min_size(minimum_size.0, minimum_size.1)
            .release_view_when_hidden()
            .stateful_view(state, view, update)
    };
    if native_proof || args.iter().any(|argument| argument == "--smoke") {
        let theme = if dark { "dark" } else { "light" };
        let output = args
            .windows(2)
            .find(|pair| pair[0] == "--output")
            .map(|pair| std::path::PathBuf::from(&pair[1]));
        let screenshot = args
            .windows(2)
            .find(|pair| pair[0] == "--screenshot")
            .map(|pair| pair[1].clone())
            .or_else(|| {
                native_proof.then(|| {
                    output
                        .clone()
                        .unwrap_or_else(|| "target/native-proof".into())
                        .join(format!("gallery-{}-{theme}.png", initial_page.slug()))
                        .to_string_lossy()
                        .into_owned()
                })
            })
            .unwrap_or_else(|| {
                format!("target/zsui-component-gallery/{}.png", initial_page.slug())
            });
        let report_path = args
            .windows(2)
            .find(|pair| pair[0] == "--report")
            .map(|pair| pair[1].clone())
            .or_else(|| {
                native_proof.then(|| {
                    output
                        .unwrap_or_else(|| "target/native-proof".into())
                        .join(format!("gallery-{}-{theme}.json", initial_page.slug()))
                        .to_string_lossy()
                        .into_owned()
                })
            })
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
        let proof_duration_ms = std::env::var("ZSUI_NATIVE_PROOF_DURATION_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(1_500)
            .max(250);
        let mut options = NativeWindowSmokeRunOptions::new(proof_duration_ms)
            .screenshot_file(&screenshot)
            .require_screenshot(true);
        if let Some(size) = resize_target {
            options = options
                .native_window_resize(size)
                .require_native_window_resize(true);
        }
        if resize_target.is_none() && initial_page == GalleryPage::Inputs {
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
        } else if resize_target.is_none() && initial_page == GalleryPage::Catalog {
            let target = builder
                .native_view_interaction_plan()
                .and_then(|plan| plan.hit_target_for_widget(CANVAS_SURFACE))
                .ok_or_else(|| {
                    ZsuiError::host(
                        "gallery_interaction_target",
                        "missing Canvas gallery widget",
                    )
                })?;
            let point = Point {
                x: target.bounds.x + target.bounds.width / 2,
                y: target.bounds.y + target.bounds.height / 2,
            };
            let drag_end = Point {
                x: (point.x + 36).min(target.bounds.x + target.bounds.width - 4),
                y: (point.y + 12).min(target.bounds.y + target.bounds.height - 4),
            };
            options = options.native_view_click(point).native_view_pointer_drag(
                point,
                drag_end,
                ZsPointerButton::Secondary,
                ZsPointerModifiers::default(),
            );
        } else if resize_target.is_none() && initial_page == GalleryPage::Navigation {
            let target = builder
                .native_view_interaction_plan()
                .and_then(|plan| {
                    plan.hit_targets.iter().find(|target| {
                        matches!(
                            target.kind,
                            ViewHitTargetKind::Tab {
                                tab_view: TABS,
                                tab,
                                ..
                            } if tab == ZsTabId::new(2)
                        )
                    })
                })
                .ok_or_else(|| {
                    ZsuiError::host(
                        "gallery_interaction_target",
                        "missing Advanced gallery tab header",
                    )
                })?;
            let point = Point {
                x: target.bounds.x + target.bounds.width / 2,
                y: target.bounds.y + target.bounds.height / 2,
            };
            options = options
                .native_view_click(point)
                .native_view_key_down(NativeViewKey::Right);
        } else if resize_target.is_none() && initial_page == GalleryPage::Feedback {
            let interaction = builder.native_view_interaction_plan().ok_or_else(|| {
                ZsuiError::host("gallery_interaction_plan", "missing View input plan")
            })?;
            let center = |target: ViewHitTarget| Point {
                x: target.bounds.x + target.bounds.width / 2,
                y: target.bounds.y + target.bounds.height / 2,
            };
            let reopen = interaction
                .hit_target_for_widget(MENU_FLYOUT_TARGET)
                .map(center)
                .ok_or_else(|| {
                    ZsuiError::host(
                        "gallery_interaction_target",
                        "missing MenuFlyout gallery target",
                    )
                })?;
            options = options
                .native_view_key_downs([
                    NativeViewKey::Down,
                    NativeViewKey::Down,
                    NativeViewKey::Right,
                    NativeViewKey::Right,
                    NativeViewKey::Enter,
                ])
                .native_view_click(reopen)
                .native_view_key_downs([
                    NativeViewKey::Down,
                    NativeViewKey::Down,
                    NativeViewKey::Right,
                    NativeViewKey::Right,
                ]);
        }
        let live_view = builder.native_live_view_runtime().cloned();
        let initial_widgets = builder
            .native_view_interaction_plan()
            .map(|plan| plan.hit_targets.clone())
            .unwrap_or_default();
        let report = builder.run_smoke(options)?;
        let widgets = live_view
            .map(|runtime| runtime.interaction_plan().hit_targets)
            .unwrap_or(initial_widgets);
        let document = if native_proof {
            let messages = if resize_target.is_some() {
                Vec::new()
            } else {
                match initial_page {
                    GalleryPage::Inputs => {
                        vec!["PrimaryAction", "CheckboxChanged", "ToggleChanged"]
                    }
                    GalleryPage::Catalog => vec!["CanvasActivated", "CanvasPointer"],
                    GalleryPage::Navigation => vec!["TabSelected"],
                    GalleryPage::Feedback => {
                        vec![
                            "MenuFlyoutCommand",
                            "MenuFlyoutOpenChanged",
                            "OpenMenuFlyout",
                        ]
                    }
                    GalleryPage::Collections => Vec::new(),
                }
            };
            serde_json::to_value(
                NativeProofDocument::new(
                    "component_gallery",
                    args.windows(2)
                        .find(|pair| pair[0] == "--scenario")
                        .map(|pair| pair[1].clone())
                        .unwrap_or_else(|| format!("gallery-{}-{theme}", initial_page.slug())),
                    theme,
                    window_width,
                    window_height,
                    widgets,
                    report.clone(),
                )
                .messages(messages),
            )
            .map_err(|error| ZsuiError::host("serialize_gallery_native_proof", error.to_string()))?
        } else {
            serde_json::to_value(&report)
                .map_err(|error| ZsuiError::host("serialize_gallery_report", error.to_string()))?
        };
        fs::write(
            report_path,
            serde_json::to_vec_pretty(&document)
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

fn proof_dimension(args: &[String], flag: &str, default: u32) -> u32 {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .and_then(|pair| pair[1].parse::<u32>().ok())
        .filter(|value| (320..=4096).contains(value))
        .unwrap_or(default)
}

fn optional_proof_dimension(args: &[String], flag: &str) -> Option<u32> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .and_then(|pair| pair[1].parse::<u32>().ok())
        .filter(|value| (320..=4096).contains(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_chinese_and_english(value: &str) -> bool {
        value.contains(" / ")
            && value
                .chars()
                .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
            && value
                .chars()
                .any(|character| character.is_ascii_alphabetic())
    }

    #[test]
    fn gallery_navigation_and_catalog_labels_are_bilingual() {
        for (page, _) in GalleryPage::ALL {
            assert!(has_chinese_and_english(page.title()), "{}", page.title());
            assert!(
                has_chinese_and_english(page.description()),
                "{}",
                page.description()
            );
        }
        for component in zsui_component_catalog() {
            let label = component_label(component.component_name);
            assert!(has_chinese_and_english(&label), "{label}");
        }
    }

    #[test]
    fn gallery_declares_every_catalog_family_and_keeps_contracts_explicit() {
        let summary = zsui_component_catalog_summary();
        assert_eq!(summary.total_count, 48);
        assert_eq!(summary.runtime_surface_count, 48);
        assert_eq!(summary.contract_only_count, 0);
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
        let metrics = ZsBaseControlMetrics::current();
        assert_eq!(save.bounds.height, metrics.button_height.0 as i32);
        assert_eq!(text.bounds.height, metrics.text_input_height.0 as i32);
        assert!(save.bounds.width >= metrics.button_minimum_width.0 as i32);
        assert!(text.bounds.width > save.bounds.width);
    }

    #[test]
    fn gallery_consumes_the_framework_adaptive_navigation_shell() {
        let mut narrow = view(&GalleryState::default());
        View::layout(
            &mut narrow,
            &mut ViewLayoutCx::new(
                Rect {
                    x: 0,
                    y: 0,
                    width: 620,
                    height: 520,
                },
                Dpi::standard(),
            ),
        );
        let narrow_interaction = narrow.interaction_plan();
        assert_eq!(
            narrow_interaction
                .hit_target_for_widget(NAVIGATION_VIEW)
                .map(|target| target.kind),
            Some(ViewHitTargetKind::NavigationViewToggle)
        );
        assert!(narrow_interaction
            .hit_target_for_widget(NAV_INPUTS)
            .is_none());
        assert!(narrow_interaction
            .hit_target_for_widget(PRIMARY_ACTION)
            .is_some());

        let mut wide = view(&GalleryState::default());
        View::layout(
            &mut wide,
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
        let wide_interaction = wide.interaction_plan();
        assert!(wide_interaction
            .hit_target_for_widget(NAVIGATION_VIEW)
            .is_none());
        assert!(wide_interaction.hit_target_for_widget(NAV_INPUTS).is_some());
    }

    #[test]
    fn gallery_navigation_page_exposes_typed_tab_headers_for_native_proof() {
        let mut state = GalleryState::default();
        state.page = GalleryPage::Navigation;
        let mut gallery = view(&state);
        View::layout(
            &mut gallery,
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
        let interaction = gallery.interaction_plan();
        let tab_ids = interaction
            .hit_targets
            .iter()
            .filter_map(|target| match target.kind {
                ViewHitTargetKind::Tab {
                    tab_view: TABS,
                    tab,
                    ..
                } => Some(tab),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            tab_ids,
            vec![ZsTabId::new(1), ZsTabId::new(2), ZsTabId::new(3)]
        );
    }
}
