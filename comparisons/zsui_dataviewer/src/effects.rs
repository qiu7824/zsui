use std::{fs::File, path::PathBuf, thread};

use zsui::{
    AppEvent, ClipboardData, ClipboardService, Command, FileDialogService, FileDialogSpec,
    NativeClipboardService, NativeFileDialogService, SaveFileDialogSpec, ZsuiError, ZsuiResult,
};

use crate::{
    SharedModel,
    data::{load_and_preview, run_query},
    lock_model,
    model::{COMMAND_COPY, COMMAND_EXPORT, COMMAND_OPEN, COMMAND_RUN},
};

pub fn execute(model: &SharedModel, command: Command) -> ZsuiResult<Vec<AppEvent>> {
    match command {
        Command::Custom { id, .. } if id == COMMAND_OPEN => open_file(model)?,
        Command::Custom { id, .. } if id == COMMAND_RUN => run_current_query(model),
        Command::Custom { id, .. } if id == COMMAND_COPY => copy_selected_row(model)?,
        Command::Custom { id, .. } if id == COMMAND_EXPORT => export_result(model)?,
        _ => {}
    }
    Ok(Vec::new())
}

fn open_file(model: &SharedModel) -> ZsuiResult<()> {
    let mut dialogs = NativeFileDialogService::new();
    let selection = dialogs.open_file_dialog(&FileDialogSpec::new("打开数据文件").filter(
        "数据文件",
        [
            "*.csv",
            "*.tsv",
            "*.parquet",
            "*.json",
            "*.jsonl",
            "*.ndjson",
        ],
    ))?;
    let Some(path) = selection.and_then(|paths| paths.into_iter().next()) else {
        lock_model(model).status = "已取消打开文件。".to_string();
        return Ok(());
    };
    let generation = lock_model(model).begin_operation(format!(
        "正在读取 {}…",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("数据文件")
    ));
    spawn_load(model.clone(), generation, path);
    Ok(())
}

pub fn spawn_load(model: SharedModel, generation: u64, path: PathBuf) {
    thread::spawn(move || match load_and_preview(&path) {
        Ok((data_set, sql, result)) => {
            lock_model(&model).apply_loaded(generation, data_set, sql, result)
        }
        Err(error) => lock_model(&model).apply_error(generation, error.to_string()),
    });
}

fn run_current_query(model: &SharedModel) {
    let (generation, path, sql) = {
        let model = lock_model(model);
        let Some(path) = model.data_path() else {
            return;
        };
        (model.operation_generation, path, model.sql.clone())
    };
    let model = model.clone();
    thread::spawn(move || match run_query(path, &sql) {
        Ok(result) => lock_model(&model).apply_query(generation, result),
        Err(error) => lock_model(&model).apply_error(generation, error.to_string()),
    });
}

fn copy_selected_row(model: &SharedModel) -> ZsuiResult<()> {
    let text = {
        let model = lock_model(model);
        model.selected_row_cells().map(|cells| cells.join("\t"))
    };
    let Some(text) = text else {
        lock_model(model).error = Some("请先选择一行结果。".to_string());
        return Ok(());
    };
    NativeClipboardService::new().write_clipboard(&ClipboardData::text(text))?;
    lock_model(model).status = "已复制所选结果行。".to_string();
    Ok(())
}

fn export_result(model: &SharedModel) -> ZsuiResult<()> {
    let (columns, rows) = {
        let state = lock_model(model);
        let Some(result) = &state.result else {
            drop(state);
            lock_model(model).error = Some("当前没有可导出的查询结果。".to_string());
            return Ok(());
        };
        let rows = result
            .filtered_rows(&state.result_filter)
            .into_iter()
            .map(|row| row.cells.clone())
            .collect::<Vec<_>>();
        (result.columns.clone(), rows)
    };
    let mut dialogs = NativeFileDialogService::new();
    let Some(path) = dialogs.save_file_dialog(
        &SaveFileDialogSpec::new("导出查询结果")
            .suggested_name("query-result.csv")
            .filter("CSV", ["*.csv"]),
    )?
    else {
        lock_model(model).status = "已取消导出。".to_string();
        return Ok(());
    };
    write_csv(&path, &columns, &rows)?;
    lock_model(model).status = format!("已导出 {} 行到 {}。", rows.len(), path.display());
    Ok(())
}

fn write_csv(path: &PathBuf, columns: &[String], rows: &[Vec<String>]) -> ZsuiResult<()> {
    let file = File::create(path)
        .map_err(|error| ZsuiError::host("dataviewer.export.create", error.to_string()))?;
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(columns)
        .map_err(|error| ZsuiError::host("dataviewer.export.header", error.to_string()))?;
    for row in rows {
        writer
            .write_record(row)
            .map_err(|error| ZsuiError::host("dataviewer.export.row", error.to_string()))?;
    }
    writer
        .flush()
        .map_err(|error| ZsuiError::host("dataviewer.export.flush", error.to_string()))
}
