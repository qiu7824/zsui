#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use zsui::{
    Command, MenuItemSpec, MenuSpec, NativeWindowSmokeRunOptions, ZsAccelerator, ZsAcceleratorKey,
    ZsuiError, ZsuiResult, native_window,
};

use zsui_dataviewer_comparison::{
    SharedModel,
    data::load_and_preview,
    effects, lock_model,
    model::{
        AppModel, COMMAND_COPY, COMMAND_EXPORT, COMMAND_OPEN, COMMAND_RUN, message_for_app_command,
        update,
    },
    ui,
};

fn main() -> ZsuiResult<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let model = initial_model(&arguments)?;
    let shared: SharedModel = Arc::new(Mutex::new(model));
    let executor_model = shared.clone();
    let builder = native_window("DataViewer · ZSUI")
        .size(1180, 760)
        .min_size(800, 520)
        .release_view_when_hidden()
        .menu(application_menu())
        .stateful_view_with_app_commands(
            shared,
            |shared| ui::view(&lock_model(shared)),
            |shared, message, cx| update(&mut lock_model(shared), message, cx),
            message_for_app_command,
        )
        .app_command_executor(move |command| effects::execute(&executor_model, command));

    if arguments.iter().any(|argument| argument == "--smoke") {
        let screenshot = argument_value(&arguments, "--screenshot")
            .unwrap_or_else(|| "target/dataviewer-smoke/window.png".to_string());
        if let Some(parent) = std::path::Path::new(&screenshot).parent() {
            fs::create_dir_all(parent)
                .map_err(|error| ZsuiError::host("dataviewer.smoke.output", error.to_string()))?;
        }
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1_500)
                .screenshot_file(&screenshot)
                .require_screenshot(true),
        )?;
        if let Some(report_path) = argument_value(&arguments, "--report") {
            if let Some(parent) = std::path::Path::new(&report_path).parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    ZsuiError::host("dataviewer.smoke.report.output", error.to_string())
                })?;
            }
            let bytes = serde_json::to_vec_pretty(&report).map_err(|error| {
                ZsuiError::host("dataviewer.smoke.report.serialize", error.to_string())
            })?;
            fs::write(report_path, bytes).map_err(|error| {
                ZsuiError::host("dataviewer.smoke.report.write", error.to_string())
            })?;
        }
        if !report.visible_window_was_created() || !report.screenshot_captured {
            return Err(ZsuiError::host(
                "dataviewer.smoke",
                "native window or screenshot proof was not produced",
            ));
        }
    } else {
        builder.run()?;
    }
    Ok(())
}

fn initial_model(arguments: &[String]) -> ZsuiResult<AppModel> {
    let mut model = AppModel::default();
    let path = argument_value(arguments, "--fixture").map(PathBuf::from);
    let path = if arguments.iter().any(|argument| argument == "--sample") {
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sales.csv"))
    } else {
        path
    };
    if let Some(path) = path {
        let generation = model.begin_operation("正在加载启动数据…");
        let (data_set, sql, result) = load_and_preview(&path)
            .map_err(|error| ZsuiError::host("dataviewer.initial_fixture", error.to_string()))?;
        model.apply_loaded(generation, data_set, sql, result);
    }
    Ok(model)
}

fn application_menu() -> MenuSpec {
    let mut file = MenuSpec::new();
    file.items.push(
        MenuItemSpec::command("刷新当前查询", Command::custom(COMMAND_RUN))
            .accelerator(ZsAccelerator::primary_character('R')),
    );
    file.items.push(MenuItemSpec::Separator);
    file.items.push(
        MenuItemSpec::command("打开…", Command::custom(COMMAND_OPEN))
            .accelerator(ZsAccelerator::primary_character('O')),
    );
    file.items.push(
        MenuItemSpec::command("导出 CSV…", Command::custom(COMMAND_EXPORT))
            .accelerator(ZsAccelerator::primary_character('E')),
    );
    file.items.push(MenuItemSpec::Separator);
    file.items
        .push(MenuItemSpec::command("退出", Command::Quit));

    let mut query = MenuSpec::new();
    query.items.push(
        MenuItemSpec::command("执行查询", Command::custom(COMMAND_RUN))
            .accelerator(ZsAccelerator::primary(ZsAcceleratorKey::Enter)),
    );
    query.items.push(
        MenuItemSpec::command("复制所选行", Command::custom(COMMAND_COPY))
            .accelerator(ZsAccelerator::primary_character('C')),
    );

    MenuSpec::new().submenu("文件", file).submenu("查询", query)
}

fn argument_value(arguments: &[String], name: &str) -> Option<String> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}
