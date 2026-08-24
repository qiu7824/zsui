use zsui::{
    Dp, SemanticTextStyle, TextRole, ThemeColorToken, ViewNode, WidgetId, ZsIcon,
    ZsInfoBarSeverity, ZsInfoBarSpec, ZsNavigationViewSpec, ZsProgressRingSpec, ZsTableColumn,
    ZsTableMetrics, ZsTablePlatformStyle, ZsTableRow, button, column, combo_box, data_grid,
    info_bar, navigation_item, navigation_view, primary_button, progress_ring, row, scroll,
    section, spacer, styled_text, text, text_editor, textbox,
};

use crate::model::{AppModel, AppPage, Msg};

const NAVIGATION: WidgetId = WidgetId::new(1);
const NAV_QUERY: WidgetId = WidgetId::new(2);
const NAV_SETTINGS: WidgetId = WidgetId::new(3);
const INFO_BAR: WidgetId = WidgetId::new(10);
const SQL_EDITOR: WidgetId = WidgetId::new(20);
const RESULT_FILTER: WidgetId = WidgetId::new(21);
const RESULT_TABLE: WidgetId = WidgetId::new(22);
const RESULT_SCROLL: WidgetId = WidgetId::new(23);
const THEME_COMBO: WidgetId = WidgetId::new(30);

pub fn view(model: &AppModel) -> ViewNode<Msg> {
    let content = match model.page {
        AppPage::Query => query_page(model),
        AppPage::Settings => settings_page(model),
    };
    let navigation = ZsNavigationViewSpec::new("DataViewer", "本地数据读取与 SQL 查询")
        .items([
            navigation_item("数据查询", ZsIcon::Search, model.page == AppPage::Query)
                .id(NAV_QUERY)
                .on_click(Msg::Navigate(AppPage::Query)),
        ])
        .footer_items([
            navigation_item("设置", ZsIcon::Settings, model.page == AppPage::Settings)
                .id(NAV_SETTINGS)
                .on_click(Msg::Navigate(AppPage::Settings)),
        ])
        .pane_width(Dp::new(240.0))
        .minimum_content_width(Dp::new(640.0))
        .content(NAVIGATION, content);

    navigation_view(navigation)
        .bg(ThemeColorToken::Surface)
        .theme_mode(model.theme.zsui_mode())
}

fn query_page(model: &AppModel) -> ViewNode<Msg> {
    let content = row([data_panel(model), workspace(model)])
        .flex(1.0)
        .gap(Dp::new(16.0));
    column([
        styled_text(
            "数据查询",
            SemanticTextStyle::for_role(TextRole::WindowTitle),
        )
        .flex(0.0),
        text("使用 DuckDB 直接查询本地 CSV、TSV、Parquet 与 JSON 文件。").flex(0.0),
        content,
    ])
    .padding(Dp::new(20.0))
    .gap(Dp::new(12.0))
    .bg(ThemeColorToken::Surface)
}

fn data_panel(model: &AppModel) -> ViewNode<Msg> {
    let data_set_summary = if let Some(data_set) = &model.data_set {
        column([
            styled_text(
                data_set.file_name.clone(),
                SemanticTextStyle::for_role(TextRole::Body),
            ),
            text(format!("格式：{}", data_set.format.label())),
            text(format!("SQL 别名：{}", data_set.alias)),
            text(format!("列数：{}", data_set.columns.len())),
            text(format!("显示上限：{} 行", data_set.preview_limit)),
        ])
        .gap(Dp::new(4.0))
    } else {
        column([text("尚未加载文件"), text("请选择一个本地数据文件开始。")]).gap(Dp::new(4.0))
    };
    let mut children = vec![
        primary_button("打开文件")
            .enabled(!model.busy)
            .on_click(Msg::OpenFile),
        data_set_summary,
        column([
            text("支持格式"),
            text(".csv · 自动识别分隔与类型"),
            text(".tsv · 制表符分隔"),
            text(".parquet · 列式数据"),
            text(".json / .jsonl / .ndjson"),
        ])
        .gap(Dp::new(3.0)),
    ];
    if model.busy {
        children.push(
            row([
                progress_ring(ZsProgressRingSpec::indeterminate()),
                text("正在处理…"),
                spacer(),
            ])
            .height(Dp::new(36.0))
            .gap(Dp::new(8.0)),
        );
    }
    section("数据集", children).width(Dp::new(240.0)).flex(0.0)
}

fn workspace(model: &AppModel) -> ViewNode<Msg> {
    let mut children = Vec::new();
    if let Some(error) = &model.error {
        children.push(
            info_bar(
                INFO_BAR,
                ZsInfoBarSpec::new(error.clone())
                    .title("操作失败")
                    .severity(ZsInfoBarSeverity::Error),
            )
            .on_info_bar_event(Msg::InfoBar),
        );
    }
    children.push(sql_panel(model));
    children.push(result_panel(model));
    column(children).flex(1.0).gap(Dp::new(12.0))
}

fn sql_panel(model: &AppModel) -> ViewNode<Msg> {
    let actions = row([
        column([
            text("查询当前数据集；Primary+Enter 可从原生菜单执行。"),
            text("结果视图最多保留 200 行。"),
        ])
        .gap(Dp::new(2.0))
        .flex(1.0),
        button("重置")
            .enabled(!model.busy && model.data_set.is_some())
            .on_click(Msg::ResetQuery),
        primary_button("执行查询")
            .enabled(!model.busy && model.data_set.is_some() && !model.sql.trim().is_empty())
            .on_click(Msg::RunQuery),
    ])
    .gap(Dp::new(8.0))
    .flex(0.0);
    section(
        "SQL 查询",
        [
            actions,
            text_editor(model.sql.clone())
                .id(SQL_EDITOR)
                .height(Dp::new(132.0))
                .on_change(Msg::SqlChanged),
        ],
    )
    .flex(0.0)
}

fn result_panel(model: &AppModel) -> ViewNode<Msg> {
    let (summary, table) = if let Some(result) = &model.result {
        let filtered = result.filtered_rows(&model.result_filter);
        let summary = format!(
            "{} 行，{} 列{} · {:.2} ms",
            filtered.len(),
            result.columns.len(),
            if result.truncated {
                "（已截断）"
            } else {
                ""
            },
            result.elapsed_micros as f64 / 1_000.0
        );
        let columns = result
            .columns
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let mut column = ZsTableColumn::new(index as u64 + 1, name.clone()).sortable(true);
                if result.columns.len() <= 6 {
                    column = column.fill_width(1);
                } else {
                    column = column.fixed_width(Dp::new(156.0));
                }
                column
            })
            .collect::<Vec<_>>();
        let rows = filtered
            .iter()
            .map(|row| ZsTableRow::new(row.id, row.cells.clone()))
            .collect::<Vec<_>>();
        let metrics = ZsTableMetrics::for_platform(ZsTablePlatformStyle::current());
        let content_height =
            Dp::new(metrics.header_height.0 + metrics.row_height.0 * rows.len() as f32);
        let grid = data_grid(columns, rows)
            .id(RESULT_TABLE)
            .height(content_height)
            .selected_table_row(model.selected_row)
            .table_sort(model.table_sort)
            .on_table_select(Msg::TableSelected)
            .on_table_sort(Msg::TableSorted)
            .on_table_invoke(Msg::TableInvoked);
        let table = scroll(grid)
            .id(RESULT_SCROLL)
            .content_height(content_height)
            .scroll_y(model.result_scroll)
            .on_scroll(Msg::ResultScrolled)
            .flex(1.0)
            .min_height(Dp::new(180.0));
        (summary, table)
    } else {
        (
            "加载数据或执行 SQL 后显示结果。".to_string(),
            column([spacer(), text("暂无查询结果"), spacer()]).flex(1.0),
        )
    };
    let controls = row([
        column([text("查询结果"), text(summary)])
            .gap(Dp::new(2.0))
            .flex(1.0),
        textbox(model.result_filter.clone())
            .id(RESULT_FILTER)
            .placeholder("筛选当前结果")
            .width(Dp::new(180.0))
            .on_change(Msg::FilterChanged),
        button("复制行")
            .enabled(model.selected_row.is_some())
            .on_click(Msg::CopySelection),
        button("导出 CSV")
            .enabled(model.result.is_some())
            .on_click(Msg::ExportResult),
    ])
    .gap(Dp::new(8.0))
    .flex(0.0);
    section("结果", [controls, table, text(model.status.clone())]).flex(1.0)
}

fn settings_page(model: &AppModel) -> ViewNode<Msg> {
    let theme = row([
        column([text("应用主题"), text("跟随系统、浅色或深色")])
            .gap(Dp::new(2.0))
            .flex(1.0),
        combo_box(["跟随系统", "浅色", "深色"], Some(model.theme.index()))
            .id(THEME_COMBO)
            .expanded(model.theme_expanded)
            .on_select(Msg::ThemeSelected)
            .on_expanded_change(Msg::ThemeExpanded),
    ])
    .gap(Dp::new(12.0));
    let editor = column([
        text("编辑器：ZSUI 原生多行文本控件"),
        text("输入：Unicode、中文输入法、选择、撤销与横纵滚动"),
        text("快捷键：Primary+Enter 执行查询"),
    ])
    .gap(Dp::new(6.0));
    let platforms = column([
        text("Windows · Win32 缓冲绘制与原生服务"),
        text("macOS · AppKit 主机与系统面板"),
        text("Linux · 原生窗口、Cairo/Pango 与桌面门户"),
    ])
    .gap(Dp::new(6.0));
    column([
        styled_text("设置", SemanticTextStyle::for_role(TextRole::WindowTitle)),
        text("调整外观并查看当前原生能力边界。"),
        section("外观", [theme]),
        section("SQL 编辑器", [editor]),
        section("桌面平台", [platforms]),
        spacer(),
    ])
    .padding(Dp::new(24.0))
    .gap(Dp::new(16.0))
    .bg(ThemeColorToken::Surface)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_builds_one_adaptive_navigation_tree() {
        let view = view(&AppModel::default());
        assert_eq!(view.id, Some(NAVIGATION));
        assert_eq!(view.children.len(), 3);
    }

    #[test]
    fn settings_and_query_share_the_same_root_contract() {
        let mut model = AppModel::default();
        let query = view(&model);
        model.page = AppPage::Settings;
        let settings = view(&model);
        assert_eq!(query.id, settings.id);
        assert_eq!(query.style.theme_mode, settings.style.theme_mode);
    }
}
