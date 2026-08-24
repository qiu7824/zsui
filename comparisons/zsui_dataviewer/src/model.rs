use std::path::PathBuf;

use zsui::{
    AppCx, Command, Dp, ZsInfoBarEvent, ZsTableRowId, ZsTableSort, ZsTableSortDirection,
    ZsuiThemeMode,
};

use crate::data::{LoadedDataSet, QueryResult};

pub const COMMAND_OPEN: &str = "dataviewer.file.open";
pub const COMMAND_RUN: &str = "dataviewer.query.run";
pub const COMMAND_COPY: &str = "dataviewer.result.copy";
pub const COMMAND_EXPORT: &str = "dataviewer.result.export";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Query,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const fn index(self) -> usize {
        match self {
            Self::System => 0,
            Self::Light => 1,
            Self::Dark => 2,
        }
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Light,
            2 => Self::Dark,
            _ => Self::System,
        }
    }

    pub const fn zsui_mode(self) -> ZsuiThemeMode {
        match self {
            Self::System => ZsuiThemeMode::System,
            Self::Light => ZsuiThemeMode::Light,
            Self::Dark => ZsuiThemeMode::Dark,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppModel {
    pub page: AppPage,
    pub data_set: Option<LoadedDataSet>,
    pub sql: String,
    pub result: Option<QueryResult>,
    pub result_filter: String,
    pub selected_row: Option<ZsTableRowId>,
    pub table_sort: Option<ZsTableSort>,
    pub result_scroll: Dp,
    pub theme: ThemeChoice,
    pub theme_expanded: bool,
    pub busy: bool,
    pub operation_generation: u64,
    pub error: Option<String>,
    pub status: String,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            page: AppPage::Query,
            data_set: None,
            sql: String::new(),
            result: None,
            result_filter: String::new(),
            selected_row: None,
            table_sort: None,
            result_scroll: Dp::new(0.0),
            theme: ThemeChoice::System,
            theme_expanded: false,
            busy: false,
            operation_generation: 0,
            error: None,
            status: "请选择 CSV、TSV、Parquet 或 JSON 数据文件。".to_string(),
        }
    }
}

impl AppModel {
    pub fn begin_operation(&mut self, status: impl Into<String>) -> u64 {
        self.operation_generation = self.operation_generation.saturating_add(1);
        self.busy = true;
        self.error = None;
        self.status = status.into();
        self.operation_generation
    }

    pub fn apply_loaded(
        &mut self,
        generation: u64,
        data_set: LoadedDataSet,
        sql: String,
        result: QueryResult,
    ) {
        if generation != self.operation_generation {
            return;
        }
        let row_count = result.rows.len();
        let elapsed_micros = result.elapsed_micros;
        self.data_set = Some(data_set);
        self.sql = sql;
        self.result = Some(result);
        self.result_filter.clear();
        self.selected_row = None;
        self.table_sort = None;
        self.result_scroll = Dp::new(0.0);
        self.busy = false;
        self.error = None;
        self.status = format!(
            "已读取 {row_count} 行，查询耗时 {:.2} ms。",
            elapsed_micros as f64 / 1_000.0
        );
    }

    pub fn apply_query(&mut self, generation: u64, result: QueryResult) {
        if generation != self.operation_generation {
            return;
        }
        let row_count = result.rows.len();
        let elapsed_micros = result.elapsed_micros;
        self.result = Some(result);
        self.selected_row = None;
        self.table_sort = None;
        self.result_scroll = Dp::new(0.0);
        self.busy = false;
        self.error = None;
        self.status = format!(
            "查询完成：{row_count} 行，{:.2} ms。",
            elapsed_micros as f64 / 1_000.0
        );
    }

    pub fn apply_error(&mut self, generation: u64, error: impl Into<String>) {
        if generation != self.operation_generation {
            return;
        }
        let error = error.into();
        self.busy = false;
        self.status = "操作失败。".to_string();
        self.error = Some(error);
    }

    pub fn selected_row_cells(&self) -> Option<&[String]> {
        let id = self.selected_row?.get();
        self.result
            .as_ref()?
            .row(id)
            .map(|row| row.cells.as_slice())
    }

    pub fn data_path(&self) -> Option<PathBuf> {
        self.data_set.as_ref().map(|data_set| data_set.path.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Navigate(AppPage),
    OpenFile,
    SqlChanged(String),
    RunQuery,
    ResetQuery,
    FilterChanged(String),
    TableSelected(ZsTableRowId),
    TableSorted(ZsTableSort),
    TableInvoked(ZsTableRowId),
    ResultScrolled(Dp),
    CopySelection,
    ExportResult,
    ThemeSelected(usize),
    ThemeExpanded(bool),
    InfoBar(ZsInfoBarEvent),
}

pub fn update(model: &mut AppModel, message: Msg, cx: &mut AppCx) {
    match message {
        Msg::Navigate(page) => model.page = page,
        Msg::OpenFile => {
            model.status = "正在选择数据文件…".to_string();
            cx.command(Command::custom(COMMAND_OPEN));
        }
        Msg::SqlChanged(sql) => model.sql = sql,
        Msg::RunQuery => {
            if model.data_set.is_none() {
                model.error = Some("请先打开数据文件。".to_string());
            } else if model.sql.trim().is_empty() {
                model.error = Some("请输入 SQL 查询。".to_string());
            } else if !model.busy {
                model.begin_operation("正在执行 SQL 查询…");
                cx.command(Command::custom(COMMAND_RUN));
            }
        }
        Msg::ResetQuery => {
            if let Some(data_set) = &model.data_set {
                model.sql = data_set.default_sql();
                model.begin_operation("正在重置并执行默认查询…");
                cx.command(Command::custom(COMMAND_RUN));
            }
        }
        Msg::FilterChanged(filter) => {
            model.result_filter = filter;
            model.selected_row = None;
            model.result_scroll = Dp::new(0.0);
        }
        Msg::TableSelected(row) => model.selected_row = Some(row),
        Msg::TableSorted(sort) => {
            if let Some(result) = &mut model.result {
                let column = sort.column.get().saturating_sub(1) as usize;
                result.sort_by_column(column, sort.direction == ZsTableSortDirection::Descending);
                model.table_sort = Some(sort);
                model.selected_row = None;
            }
        }
        Msg::TableInvoked(row) => {
            model.selected_row = Some(row);
            model.status = format!("已打开结果行 {}。", row.get());
        }
        Msg::ResultScrolled(offset) => model.result_scroll = offset,
        Msg::CopySelection => cx.command(Command::custom(COMMAND_COPY)),
        Msg::ExportResult => cx.command(Command::custom(COMMAND_EXPORT)),
        Msg::ThemeSelected(index) => {
            model.theme = ThemeChoice::from_index(index);
            model.theme_expanded = false;
        }
        Msg::ThemeExpanded(expanded) => model.theme_expanded = expanded,
        Msg::InfoBar(ZsInfoBarEvent::Close) => model.error = None,
        Msg::InfoBar(ZsInfoBarEvent::Action) => {}
    }
}

pub fn message_for_app_command(command: &Command) -> Option<Msg> {
    match command {
        Command::Custom { id, .. } if id == COMMAND_OPEN => Some(Msg::OpenFile),
        Command::Custom { id, .. } if id == COMMAND_RUN => Some(Msg::RunQuery),
        Command::Custom { id, .. } if id == COMMAND_COPY => Some(Msg::CopySelection),
        Command::Custom { id, .. } if id == COMMAND_EXPORT => Some(Msg::ExportResult),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_choice_maps_to_platform_neutral_zsui_modes() {
        assert_eq!(
            ThemeChoice::from_index(0).zsui_mode(),
            ZsuiThemeMode::System
        );
        assert_eq!(ThemeChoice::from_index(2).zsui_mode(), ZsuiThemeMode::Dark);
    }

    #[test]
    fn running_without_a_dataset_reports_a_user_error() {
        let mut model = AppModel::default();
        update(&mut model, Msg::RunQuery, &mut AppCx::new());
        assert_eq!(model.error.as_deref(), Some("请先打开数据文件。"));
        assert!(!model.busy);
    }
}
