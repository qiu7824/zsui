use std::{env, fs, path::PathBuf, process::ExitCode};

use serde_json::json;
#[cfg(feature = "auto-suggest")]
use zsui::auto_suggest_box;
#[cfg(feature = "button")]
use zsui::button;
#[cfg(all(feature = "button", feature = "label", feature = "checkbox"))]
use zsui::checkbox;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "toggle-button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "number-box", feature = "label"),
    all(feature = "password-box", feature = "label"),
    all(feature = "tooltip", feature = "button", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "progress", feature = "label"),
    all(feature = "progress-ring", feature = "label"),
    all(feature = "auto-suggest", feature = "label"),
    all(feature = "grid-view", feature = "label"),
    all(feature = "tree", feature = "label"),
    all(feature = "table", feature = "label"),
    all(feature = "dialog", feature = "label"),
    all(feature = "toast", feature = "label"),
    all(feature = "info-bar", feature = "label"),
    all(feature = "teaching-tip", feature = "button", feature = "label"),
    all(feature = "breadcrumb", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::column;
#[cfg(feature = "combo")]
use zsui::combo_box;
#[cfg(all(feature = "button", feature = "label", feature = "list"))]
use zsui::list;
#[cfg(all(feature = "progress", feature = "label"))]
use zsui::progress_bar;
#[cfg(feature = "radio")]
use zsui::radio_button;
#[cfg(all(feature = "button", feature = "label", feature = "scroll"))]
use zsui::scroll;
#[cfg(feature = "label")]
use zsui::text;
#[cfg(all(feature = "button", feature = "label", feature = "textbox"))]
use zsui::textbox;
#[cfg(feature = "toggle-button")]
use zsui::toggle_button;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "toggle-button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "number-box", feature = "label"),
    all(feature = "password-box", feature = "label"),
    all(feature = "tooltip", feature = "button", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "auto-suggest", feature = "label"),
    all(feature = "grid-view", feature = "label"),
    all(feature = "tree", feature = "label"),
    all(feature = "table", feature = "label"),
    all(feature = "dialog", feature = "label"),
    all(feature = "toast", feature = "label"),
    all(feature = "info-bar", feature = "label"),
    all(feature = "teaching-tip", feature = "button", feature = "label"),
    all(feature = "breadcrumb", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::CommandId;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "toggle-button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "number-box", feature = "label"),
    all(feature = "tooltip", feature = "button", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "auto-suggest", feature = "label"),
    all(feature = "grid-view", feature = "label"),
    all(feature = "tree", feature = "label"),
    all(feature = "table", feature = "label"),
    all(feature = "dialog", feature = "label"),
    all(feature = "toast", feature = "label"),
    all(feature = "info-bar", feature = "label"),
    all(feature = "teaching-tip", feature = "button", feature = "label"),
    all(feature = "breadcrumb", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label")
))]
use zsui::NativeViewKey;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "toggle-button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "number-box", feature = "label"),
    all(feature = "password-box", feature = "label"),
    all(feature = "tooltip", feature = "button", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "auto-suggest", feature = "label"),
    all(feature = "tree", feature = "label"),
    all(feature = "table", feature = "label"),
    all(feature = "dialog", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::Point;
#[cfg(any(
    all(feature = "progress", feature = "label"),
    all(feature = "progress-ring", feature = "label")
))]
use zsui::ProgressRange;
#[cfg(all(feature = "breadcrumb", feature = "label"))]
use zsui::{breadcrumb_bar, ZsBreadcrumbId, ZsBreadcrumbItem};
#[cfg(all(feature = "dialog", feature = "label"))]
use zsui::{content_dialog, ZsContentDialogButton, ZsContentDialogResult, ZsContentDialogSpec};
#[cfg(all(feature = "table", feature = "label"))]
use zsui::{
    data_grid, HorizontalAlign, ZsTableColumn, ZsTableColumnId, ZsTableRow, ZsTableRowId,
    ZsTableSort, ZsTableSortDirection,
};
#[cfg(feature = "date-picker")]
use zsui::{date_picker, ZsDate, ZsuiThemeMode};
#[cfg(all(feature = "grid", feature = "button", feature = "label"))]
use zsui::{grid, ZsGridCell, ZsGridFraction, ZsGridSpan, ZsGridTrack};
#[cfg(all(feature = "grid-view", feature = "label"))]
use zsui::{grid_view, ZsGridViewItem, ZsGridViewItemId};
#[cfg(all(feature = "info-bar", feature = "label"))]
use zsui::{info_bar, ZsInfoBarEvent, ZsInfoBarSeverity, ZsInfoBarSpec};
use zsui::{
    native_ui_platform_for_current_target, native_window,
    write_native_host_smoke_artifacts_with_interaction_to, Command, MenuItemSpec, MenuSpec,
    NativeHostSmokeInteractionReport, NativeUiPlatform, NativeWindowBuilder,
    NativeWindowRuntimeDriver, NativeWindowSmokeRunOptions, TraySpec,
};
#[cfg(feature = "number-box")]
use zsui::{number_box, ZsNumberRange};
#[cfg(feature = "password-box")]
use zsui::{password_box, ZsPassword, ZsPasswordRevealMode};
#[cfg(all(feature = "progress-ring", feature = "label"))]
use zsui::{progress_ring, ZsProgressRingSpec};
#[cfg(feature = "slider")]
use zsui::{slider, SliderRange};
#[cfg(feature = "tabs")]
use zsui::{tab_view, ZsTabId, ZsTabItem};
#[cfg(all(feature = "teaching-tip", feature = "button", feature = "label"))]
use zsui::{teaching_tip, ZsTeachingTipResult, ZsTeachingTipSpec};
#[cfg(feature = "time-picker")]
use zsui::{time_picker, ZsClockFormat, ZsMinuteIncrement, ZsTime};
#[cfg(all(feature = "toast", feature = "label"))]
use zsui::{toast_presenter, ZsToastDuration, ZsToastResult, ZsToastSpec};
#[cfg(all(feature = "tree", feature = "label"))]
use zsui::{tree_view, ZsTreeExpansionChange, ZsTreeNode, ZsTreeNodeId};
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "toggle-button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "number-box", feature = "label"),
    all(feature = "password-box", feature = "label"),
    all(feature = "tooltip", feature = "button", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "auto-suggest", feature = "label"),
    all(feature = "grid-view", feature = "label"),
    all(feature = "tree", feature = "label"),
    all(feature = "table", feature = "label"),
    all(feature = "dialog", feature = "label"),
    all(feature = "toast", feature = "label"),
    all(feature = "info-bar", feature = "label"),
    all(feature = "teaching-tip", feature = "button", feature = "label"),
    all(feature = "breadcrumb", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::{UiCommand, WidgetId};
#[cfg(all(feature = "auto-suggest", feature = "label"))]
use zsui::{
    ZsAutoSuggestSubmission, ZsAutoSuggestTextChange, ZsAutoSuggestTextChangeReason,
    ZsAutoSuggestion, ZsAutoSuggestionId,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let date_picker_high_contrast = args.iter().any(|arg| arg == "--date-picker-high-contrast");
    let non_flag_args = args
        .iter()
        .filter(|arg| !arg.starts_with("--"))
        .collect::<Vec<_>>();
    match run_smoke(
        non_flag_args.first().map(|arg| arg.as_str()),
        non_flag_args.get(1).map(|arg| arg.as_str()),
        args.iter()
            .any(|arg| arg == "--tray" || arg == "--status-item"),
        args.iter()
            .any(|arg| arg == "--menu" || arg == "--window-menu"),
        args.iter()
            .any(|arg| arg == "--grid-view" || arg == "--grid"),
        args.iter()
            .any(|arg| arg == "--toggle-button-view" || arg == "--toggle-button"),
        args.iter().any(|arg| arg == "--view"),
        args.iter()
            .any(|arg| arg == "--scroll-view" || arg == "--scroll"),
        args.iter()
            .any(|arg| arg == "--slider-view" || arg == "--slider"),
        args.iter()
            .any(|arg| arg == "--number-box-view" || arg == "--number-box"),
        args.iter()
            .any(|arg| arg == "--password-box-view" || arg == "--password-box"),
        args.iter()
            .any(|arg| arg == "--tooltip-view" || arg == "--tooltip"),
        args.iter()
            .any(|arg| arg == "--radio-view" || arg == "--radio"),
        args.iter()
            .any(|arg| arg == "--progress-view" || arg == "--progress"),
        args.iter()
            .any(|arg| arg == "--progress-ring-view" || arg == "--progress-ring"),
        args.iter()
            .any(|arg| arg == "--auto-suggest-view" || arg == "--auto-suggest"),
        args.iter()
            .any(|arg| arg == "--tree-view" || arg == "--tree"),
        args.iter()
            .any(|arg| arg == "--gallery-view" || arg == "--gallery"),
        args.iter()
            .any(|arg| arg == "--table-view" || arg == "--table" || arg == "--data-grid"),
        args.iter()
            .any(|arg| arg == "--content-dialog" || arg == "--dialog"),
        args.iter()
            .any(|arg| arg == "--toast-view" || arg == "--toast"),
        args.iter()
            .any(|arg| arg == "--info-bar-view" || arg == "--info-bar"),
        args.iter()
            .any(|arg| arg == "--teaching-tip-view" || arg == "--teaching-tip"),
        args.iter()
            .any(|arg| arg == "--breadcrumb-view" || arg == "--breadcrumb"),
        args.iter()
            .any(|arg| arg == "--combo-view" || arg == "--combo"),
        args.iter()
            .any(|arg| arg == "--date-picker-view" || arg == "--date-picker")
            || date_picker_high_contrast,
        date_picker_high_contrast,
        args.iter()
            .any(|arg| arg == "--time-picker-view" || arg == "--time-picker"),
        args.iter()
            .any(|arg| arg == "--tabs-view" || arg == "--tabs"),
    ) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(2)
        }
    }
}

fn run_smoke(
    platform: Option<&str>,
    artifact_root: Option<&str>,
    include_status_item: bool,
    include_window_menu: bool,
    include_grid_view: bool,
    include_toggle_button_view: bool,
    include_typed_view: bool,
    include_scroll_view: bool,
    include_slider_view: bool,
    include_number_box_view: bool,
    include_password_box_view: bool,
    include_tooltip_view: bool,
    include_radio_view: bool,
    include_progress_view: bool,
    include_progress_ring_view: bool,
    include_auto_suggest_view: bool,
    include_tree_view: bool,
    include_gallery_view: bool,
    include_table_view: bool,
    include_dialog_view: bool,
    include_toast_view: bool,
    include_info_bar_view: bool,
    include_teaching_tip_view: bool,
    include_breadcrumb_view: bool,
    include_combo_view: bool,
    include_date_picker_view: bool,
    date_picker_high_contrast: bool,
    include_time_picker_view: bool,
    include_tabs_view: bool,
) -> Result<String, String> {
    let platform = parse_platform(platform.unwrap_or("current"))?;
    let current = native_ui_platform_for_current_target()
        .ok_or_else(|| "current target is not a supported ZSUI platform".to_string())?;
    if platform != current {
        return Err(format!(
            "cannot run `{}` native smoke on current `{}` target",
            platform.platform_name(),
            current.platform_name()
        ));
    }
    #[cfg(not(all(feature = "toggle-button", feature = "label")))]
    if include_toggle_button_view {
        return Err(
            "--toggle-button-view requires the toggle-button and label features".to_string(),
        );
    }
    #[cfg(not(all(feature = "slider", feature = "label")))]
    if include_slider_view {
        return Err("--slider-view requires the slider and label features".to_string());
    }
    #[cfg(not(all(feature = "number-box", feature = "label")))]
    if include_number_box_view {
        return Err("--number-box-view requires the number-box and label features".to_string());
    }
    #[cfg(not(all(feature = "password-box", feature = "label")))]
    if include_password_box_view {
        return Err("--password-box-view requires the password-box and label features".to_string());
    }
    #[cfg(not(all(feature = "tooltip", feature = "button", feature = "label")))]
    if include_tooltip_view {
        return Err("--tooltip-view requires the tooltip, button and label features".to_string());
    }
    #[cfg(not(all(feature = "radio", feature = "label")))]
    if include_radio_view {
        return Err("--radio-view requires the radio and label features".to_string());
    }
    #[cfg(not(all(feature = "progress", feature = "label")))]
    if include_progress_view {
        return Err("--progress-view requires the progress and label features".to_string());
    }
    #[cfg(not(all(feature = "progress-ring", feature = "label")))]
    if include_progress_ring_view {
        return Err(
            "--progress-ring-view requires the progress-ring and label features".to_string(),
        );
    }
    #[cfg(not(all(feature = "auto-suggest", feature = "label")))]
    if include_auto_suggest_view {
        return Err("--auto-suggest-view requires the auto-suggest and label features".to_string());
    }
    #[cfg(not(all(feature = "tree", feature = "label")))]
    if include_tree_view {
        return Err("--tree-view requires the tree and label features".to_string());
    }
    #[cfg(not(all(feature = "grid-view", feature = "label")))]
    if include_gallery_view {
        return Err("--gallery-view requires the grid-view and label features".to_string());
    }
    #[cfg(not(all(feature = "table", feature = "label")))]
    if include_table_view {
        return Err("--table-view requires the table and label features".to_string());
    }
    #[cfg(not(all(feature = "dialog", feature = "label")))]
    if include_dialog_view {
        return Err("--content-dialog requires the dialog and label features".to_string());
    }
    #[cfg(not(all(feature = "toast", feature = "label")))]
    if include_toast_view {
        return Err("--toast-view requires the toast and label features".to_string());
    }
    #[cfg(not(all(feature = "info-bar", feature = "label")))]
    if include_info_bar_view {
        return Err("--info-bar-view requires the info-bar and label features".to_string());
    }
    #[cfg(not(all(feature = "teaching-tip", feature = "button", feature = "label")))]
    if include_teaching_tip_view {
        return Err(
            "--teaching-tip-view requires the teaching-tip, button and label features".to_string(),
        );
    }
    #[cfg(not(all(feature = "breadcrumb", feature = "label")))]
    if include_breadcrumb_view {
        return Err("--breadcrumb-view requires the breadcrumb and label features".to_string());
    }
    #[cfg(not(all(feature = "combo", feature = "label")))]
    if include_combo_view {
        return Err("--combo-view requires the combo and label features".to_string());
    }
    #[cfg(not(all(feature = "date-picker", feature = "label")))]
    if include_date_picker_view {
        return Err("--date-picker-view requires the date-picker and label features".to_string());
    }
    #[cfg(not(all(feature = "time-picker", feature = "label")))]
    if include_time_picker_view {
        return Err("--time-picker-view requires the time-picker and label features".to_string());
    }
    #[cfg(not(all(feature = "tabs", feature = "label")))]
    if include_tabs_view {
        return Err("--tabs-view requires the tabs and label features".to_string());
    }
    #[cfg(not(all(feature = "grid", feature = "button", feature = "label")))]
    if include_grid_view {
        return Err("--grid-view requires the grid, button and label features".to_string());
    }
    let artifact_root = artifact_root.unwrap_or("target/native-host-smoke");
    let artifact_dir = PathBuf::from(artifact_root).join(platform.platform_name());
    fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;
    let screenshot_file = artifact_dir
        .join("window.png")
        .to_string_lossy()
        .replace('\\', "/");
    let mut smoke_options = NativeWindowSmokeRunOptions::quick()
        .screenshot_file(screenshot_file)
        .require_screenshot(platform == NativeUiPlatform::Windows);
    if include_status_item {
        smoke_options = smoke_options
            .status_item(
                TraySpec::new()
                    .tooltip("ZSUI Smoke")
                    .item("Open", Command::ShowMainWindow)
                    .separator()
                    .item("Quit", Command::Quit),
            )
            .require_status_item(platform == NativeUiPlatform::Windows);
    }
    #[cfg(all(feature = "button", feature = "label"))]
    if include_typed_view && !include_scroll_view {
        smoke_options = smoke_options.native_view_key_down(NativeViewKey::Tab);
        #[cfg(feature = "textbox")]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 120 })
                .native_view_drag(Point { x: 16, y: 120 }, Point { x: 32, y: 120 })
                .native_view_text_input("ZSUI");
        }
        #[cfg(all(feature = "list", feature = "textbox", feature = "checkbox"))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 176 })
                .native_view_key_down(NativeViewKey::Up);
        }
        #[cfg(all(feature = "list", feature = "textbox", not(feature = "checkbox")))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 220 })
                .native_view_key_down(NativeViewKey::Up);
        }
        #[cfg(all(feature = "list", not(feature = "textbox"), feature = "checkbox"))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 140 })
                .native_view_key_down(NativeViewKey::Up);
        }
        #[cfg(all(feature = "list", not(feature = "textbox"), not(feature = "checkbox")))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 180 })
                .native_view_key_down(NativeViewKey::Up);
        }
        #[cfg(feature = "checkbox")]
        {
            smoke_options = smoke_options.native_view_click(Point { x: 260, y: 200 });
        }
        #[cfg(any(feature = "textbox", feature = "checkbox"))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 280 })
                .native_view_key_down(NativeViewKey::Enter);
        }
        #[cfg(not(any(feature = "textbox", feature = "checkbox")))]
        {
            smoke_options = smoke_options
                .native_view_click(Point { x: 260, y: 240 })
                .native_view_key_down(NativeViewKey::Enter);
        }
    }
    #[cfg(all(feature = "button", feature = "label", feature = "scroll"))]
    if include_scroll_view {
        smoke_options = smoke_options.native_view_scroll(Point { x: 260, y: 220 }, 48);
    }
    #[cfg(all(feature = "toggle-button", feature = "label"))]
    if include_toggle_button_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 100, y: 84 })
            .native_view_key_down(NativeViewKey::Space)
            .native_view_click(Point { x: 100, y: 84 });
    }
    #[cfg(all(feature = "slider", feature = "label"))]
    if include_slider_view {
        smoke_options = smoke_options
            .native_view_drag(Point { x: 100, y: 84 }, Point { x: 400, y: 84 })
            .native_view_key_down(NativeViewKey::Left);
    }
    #[cfg(all(feature = "number-box", feature = "label"))]
    if include_number_box_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 482, y: 72 })
            .native_view_key_down(NativeViewKey::Up)
            .native_view_key_down(NativeViewKey::PageUp)
            .native_view_text_input("\u{8}\u{8}\u{8}\u{8}")
            .native_view_text_input("42.5")
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "password-box", feature = "label"))]
    if include_password_box_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 420, y: 84 })
            .native_view_text_input("ZSUI")
            .native_view_click(Point { x: 480, y: 84 });
    }
    #[cfg(all(feature = "tooltip", feature = "button", feature = "label"))]
    if include_tooltip_view {
        smoke_options = smoke_options.native_view_key_down(NativeViewKey::Tab);
    }
    #[cfg(all(feature = "radio", feature = "label"))]
    if include_radio_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 100, y: 124 })
            .native_view_key_down(NativeViewKey::Space)
            .native_view_key_down(NativeViewKey::Up)
            .native_view_key_down(NativeViewKey::Tab);
    }
    #[cfg(all(feature = "combo", feature = "label"))]
    if include_combo_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 100, y: 158 })
            .native_view_key_down(NativeViewKey::Space)
            .native_view_key_down(NativeViewKey::Down)
            .native_view_text_input("B")
            .native_view_key_down(NativeViewKey::Space)
            .native_view_scroll(Point { x: 100, y: 158 }, 48);
    }
    #[cfg(all(feature = "auto-suggest", feature = "label"))]
    if include_auto_suggest_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 100, y: 154 })
            .native_view_text_input("x")
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Enter)
            .native_view_click(Point { x: 480, y: 80 })
            .native_view_text_input("B");
    }
    #[cfg(all(feature = "tree", feature = "label"))]
    if include_tree_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 62, y: 112 })
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Enter)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_click(Point { x: 180, y: 144 });
    }
    #[cfg(all(feature = "grid-view", feature = "label"))]
    if include_gallery_view {
        smoke_options = smoke_options
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "table", feature = "label"))]
    if include_table_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 180, y: 80 })
            .native_view_click(Point { x: 180, y: 80 })
            .native_view_click(Point { x: 180, y: 148 })
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Enter)
            .native_view_key_down(NativeViewKey::Home);
    }
    #[cfg(all(feature = "dialog", feature = "label"))]
    if include_dialog_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 4, y: 4 })
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "toast", feature = "label"))]
    if include_toast_view {
        smoke_options = smoke_options
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "info-bar", feature = "label"))]
    if include_info_bar_view {
        smoke_options = smoke_options
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "teaching-tip", feature = "button", feature = "label"))]
    if include_teaching_tip_view {
        smoke_options = smoke_options
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "breadcrumb", feature = "label"))]
    if include_breadcrumb_view {
        smoke_options = smoke_options
            .native_view_key_down(NativeViewKey::Tab)
            .native_view_key_down(NativeViewKey::Home)
            .native_view_key_down(NativeViewKey::Enter)
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "date-picker", feature = "label"))]
    if include_date_picker_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 130, y: 284 })
            .native_view_click(Point { x: 100, y: 80 });
    }
    #[cfg(all(feature = "time-picker", feature = "label"))]
    if include_time_picker_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 164, y: 241 })
            .native_view_key_down(NativeViewKey::Escape)
            .native_view_key_down(NativeViewKey::Down)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_click(Point { x: 100, y: 80 });
    }
    #[cfg(all(feature = "tabs", feature = "label"))]
    if include_tabs_view {
        smoke_options = smoke_options
            .native_view_click(Point { x: 170, y: 80 })
            .native_view_key_down(NativeViewKey::Left)
            .native_view_key_down(NativeViewKey::Space)
            .native_view_key_down(NativeViewKey::Right)
            .native_view_key_down(NativeViewKey::Enter);
    }
    #[cfg(all(feature = "grid", feature = "button", feature = "label"))]
    if include_grid_view {
        smoke_options = smoke_options.native_view_click(Point { x: 390, y: 312 });
    }

    let builder = native_window("ZSUI Smoke").size(
        520,
        if include_date_picker_view {
            480
        } else if include_gallery_view {
            464
        } else if include_time_picker_view {
            360
        } else if include_grid_view {
            360
        } else {
            320
        },
    );
    let builder = if include_window_menu {
        builder.menu(smoke_window_menu())
    } else {
        builder
    }
    .ui_command_executor(NativeWindowRuntimeDriver::new());
    let builder = if include_grid_view {
        attach_grid_view(builder)
    } else if include_toggle_button_view {
        attach_toggle_button_view(builder)
    } else if include_number_box_view {
        attach_number_box_view(builder)
    } else if include_password_box_view {
        attach_password_box_view(builder)
    } else if include_tooltip_view {
        attach_tooltip_view(builder)
    } else if include_slider_view {
        attach_slider_view(builder)
    } else if include_radio_view {
        attach_radio_view(builder)
    } else if include_progress_view {
        attach_progress_view(builder)
    } else if include_progress_ring_view {
        attach_progress_ring_view(builder)
    } else if include_auto_suggest_view {
        attach_auto_suggest_view(builder)
    } else if include_tree_view {
        attach_tree_view(builder)
    } else if include_gallery_view {
        attach_gallery_view(builder)
    } else if include_table_view {
        attach_table_view(builder)
    } else if include_dialog_view {
        attach_content_dialog_view(builder)
    } else if include_toast_view {
        attach_toast_view(builder)
    } else if include_info_bar_view {
        attach_info_bar_view(builder)
    } else if include_teaching_tip_view {
        attach_teaching_tip_view(builder)
    } else if include_breadcrumb_view {
        attach_breadcrumb_view(builder)
    } else if include_combo_view {
        attach_combo_view(builder)
    } else if include_date_picker_view {
        attach_date_picker_view(builder, date_picker_high_contrast)
    } else if include_time_picker_view {
        attach_time_picker_view(builder)
    } else if include_tabs_view {
        attach_tabs_view(builder)
    } else if include_scroll_view {
        attach_scroll_view(builder)
    } else if include_typed_view {
        attach_typed_view(builder)
    } else {
        builder
    };
    let run_report = builder
        .run_smoke(smoke_options)
        .map_err(|err| err.to_string())?;
    let interaction = NativeHostSmokeInteractionReport::from_native_window_smoke(
        platform.platform_name(),
        "real_native_host",
        &run_report,
    );
    let write_report =
        write_native_host_smoke_artifacts_with_interaction_to(platform, artifact_root, interaction)
            .map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&json!({
        "run": run_report,
        "artifacts": write_report,
    }))
    .map_err(|err| err.to_string())
}

#[cfg(all(feature = "tooltip", feature = "button", feature = "label"))]
fn attach_tooltip_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.ui_command_view(
        column([
            text::<UiCommand>("ZSUI ToolTip Smoke").height(zsui::Dp::new(28.0)),
            button("Save document")
                .id(WidgetId::new(77))
                .height(zsui::Dp::new(40.0))
                .tooltip_spec(
                    zsui::ZsTooltipSpec::new("Save the current document").open_delay_ms(100),
                )
                .on_click(UiCommand::app(CommandId("zsui.native_smoke.tooltip_owner"))),
        ])
        .padding(zsui::Dp::new(24.0))
        .gap(zsui::Dp::new(12.0)),
    )
}

#[cfg(not(all(feature = "tooltip", feature = "button", feature = "label")))]
fn attach_tooltip_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "grid", feature = "button", feature = "label"))]
fn attach_grid_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.ui_command_view(
        grid(
            [
                ZsGridTrack::fixed(zsui::Dp::new(128.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fraction(ZsGridFraction::TWO),
            ],
            [
                ZsGridTrack::fixed(zsui::Dp::new(44.0)),
                ZsGridTrack::FLEX,
                ZsGridTrack::fixed(zsui::Dp::new(48.0)),
            ],
            [
                ZsGridCell::new(
                    0,
                    0,
                    text::<UiCommand>("ZSUI typed Grid smoke")
                        .padding(zsui::Dp::new(10.0))
                        .bg(zsui::ThemeColorToken::SurfaceRaised),
                )
                .column_span(ZsGridSpan::THREE),
                ZsGridCell::new(
                    1,
                    0,
                    text("Navigation")
                        .padding(zsui::Dp::new(12.0))
                        .bg(zsui::ThemeColorToken::Control),
                ),
                ZsGridCell::new(
                    1,
                    1,
                    text("Flexible content spans two columns")
                        .padding(zsui::Dp::new(12.0))
                        .bg(zsui::ThemeColorToken::SurfaceRaised),
                )
                .column_span(ZsGridSpan::TWO),
                ZsGridCell::new(2, 0, text("Status").padding(zsui::Dp::new(12.0)))
                    .column_span(ZsGridSpan::TWO),
                ZsGridCell::new(
                    2,
                    2,
                    button("Apply")
                        .id(WidgetId::new(17))
                        .on_click(UiCommand::app(CommandId("zsui.native_smoke.grid_apply"))),
                ),
            ],
        )
        .padding(zsui::Dp::new(24.0))
        .column_gap(zsui::Dp::new(12.0))
        .row_gap(zsui::Dp::new(10.0)),
    )
}

#[cfg(not(all(feature = "grid", feature = "button", feature = "label")))]
fn attach_grid_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "toggle-button", feature = "label"))]
#[derive(Clone)]
enum ToggleButtonSmokeMsg {
    Changed(bool),
}

#[cfg(all(feature = "toggle-button", feature = "label"))]
fn attach_toggle_button_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        false,
        |checked| {
            column([
                text::<ToggleButtonSmokeMsg>("ZSUI ToggleButton Smoke").height(zsui::Dp::new(28.0)),
                zsui::row([
                    toggle_button("Pin panel", *checked)
                        .id(WidgetId::new(19))
                        .width(zsui::Dp::new(160.0))
                        .on_toggle(ToggleButtonSmokeMsg::Changed),
                    zsui::spacer(),
                ])
                .height(zsui::Dp::new(36.0)),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
            .bg(zsui::ThemeColorToken::Surface)
        },
        |checked, message, cx| match message {
            ToggleButtonSmokeMsg::Changed(next) => {
                *checked = next;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.toggle_button_changed",
                )));
            }
        },
    )
}

#[cfg(not(all(feature = "toggle-button", feature = "label")))]
fn attach_toggle_button_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "slider", feature = "label"))]
fn attach_slider_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.ui_command_view(
        column([
            text::<UiCommand>("ZSUI Slider Smoke").height(zsui::Dp::new(28.0)),
            slider(25.0, SliderRange::new(0.0, 100.0).step(5.0))
                .id(WidgetId::new(10))
                .height(zsui::Dp::new(40.0))
                .on_slide(native_smoke_slider_changed),
        ])
        .padding(zsui::Dp::new(24.0))
        .gap(zsui::Dp::new(12.0)),
    )
}

#[cfg(not(all(feature = "slider", feature = "label")))]
fn attach_slider_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "slider", feature = "label"))]
fn native_smoke_slider_changed(_: f32) -> UiCommand {
    UiCommand::app(CommandId("zsui.native_smoke.slider_changed"))
}

#[cfg(all(feature = "number-box", feature = "label"))]
#[derive(Clone)]
enum NumberBoxSmokeMsg {
    Changed(Option<f64>),
}

#[cfg(all(feature = "number-box", feature = "label"))]
fn attach_number_box_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let range = ZsNumberRange::new(-100.0, 100.0).step(0.5).large_step(10.0);
    builder.stateful_view(
        Some(12.5_f64),
        move |value| {
            column([
                text::<NumberBoxSmokeMsg>("ZSUI NumberBox Smoke").height(zsui::Dp::new(28.0)),
                number_box(*value, range)
                    .id(WidgetId::new(18))
                    .height(zsui::Dp::new(40.0))
                    .fraction_digits(1)
                    .on_number_change(NumberBoxSmokeMsg::Changed),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
            .bg(zsui::ThemeColorToken::Surface)
        },
        |value, message, cx| match message {
            NumberBoxSmokeMsg::Changed(next) => {
                *value = next;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.number_box_changed",
                )));
            }
        },
    )
}

#[cfg(not(all(feature = "number-box", feature = "label")))]
fn attach_number_box_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "password-box", feature = "label"))]
#[derive(Clone)]
enum PasswordBoxSmokeMsg {
    Changed(ZsPassword),
}

#[cfg(all(feature = "password-box", feature = "label"))]
fn attach_password_box_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        ZsPassword::from("A🙂"),
        |value| {
            column([
                text::<PasswordBoxSmokeMsg>("ZSUI PasswordBox Smoke").height(zsui::Dp::new(28.0)),
                password_box(value)
                    .id(WidgetId::new(20))
                    .height(zsui::Dp::new(36.0))
                    .reveal_mode(ZsPasswordRevealMode::Peek)
                    .on_password_change(PasswordBoxSmokeMsg::Changed),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
            .bg(zsui::ThemeColorToken::Surface)
        },
        |value, message, cx| match message {
            PasswordBoxSmokeMsg::Changed(next) => {
                *value = next;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.password_box_changed",
                )));
            }
        },
    )
}

#[cfg(not(all(feature = "password-box", feature = "label")))]
fn attach_password_box_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "radio", feature = "label"))]
#[derive(Clone)]
enum RadioSmokeMsg {
    Choose(usize),
}

#[cfg(all(feature = "radio", feature = "label"))]
fn attach_radio_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        0usize,
        |selected| {
            column([
                text::<RadioSmokeMsg>("ZSUI RadioButton Smoke").height(zsui::Dp::new(28.0)),
                radio_button("Balanced", *selected == 0)
                    .id(WidgetId::new(11))
                    .height(zsui::Dp::new(32.0))
                    .on_choose(RadioSmokeMsg::Choose(0)),
                radio_button("Performance", *selected == 1)
                    .id(WidgetId::new(12))
                    .height(zsui::Dp::new(32.0))
                    .on_choose(RadioSmokeMsg::Choose(1)),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |selected, message, cx| match message {
            RadioSmokeMsg::Choose(index) => {
                *selected = index;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.radio_selected",
                )));
            }
        },
    )
}

#[cfg(not(all(feature = "radio", feature = "label")))]
fn attach_radio_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "progress", feature = "label"))]
fn attach_progress_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.view(
        column([
            text::<()>("ZSUI ProgressBar Smoke").height(zsui::Dp::new(28.0)),
            progress_bar::<()>(65.0, ProgressRange::new(0.0, 100.0)).height(zsui::Dp::new(32.0)),
        ])
        .padding(zsui::Dp::new(24.0))
        .gap(zsui::Dp::new(12.0)),
    )
}

#[cfg(all(feature = "progress-ring", feature = "label"))]
fn attach_progress_ring_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        (),
        |_| {
            column([
                text::<()>("ZSUI ProgressRing Smoke").height(zsui::Dp::new(28.0)),
                zsui::row([
                    progress_ring::<()>(ZsProgressRingSpec::indeterminate()),
                    progress_ring::<()>(ZsProgressRingSpec::determinate(
                        65.0,
                        ProgressRange::new(0.0, 100.0),
                    )),
                    zsui::spacer(),
                ])
                .height(zsui::Dp::new(48.0))
                .gap(zsui::Dp::new(16.0)),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |_, _, _| {},
    )
}

#[cfg(all(feature = "auto-suggest", feature = "label"))]
#[derive(Clone)]
enum AutoSuggestSmokeMsg {
    TextChanged(ZsAutoSuggestTextChange),
    SuggestionChosen(ZsAutoSuggestionId),
    Submitted(ZsAutoSuggestSubmission),
    Expanded(bool),
}

#[cfg(all(feature = "auto-suggest", feature = "label"))]
struct AutoSuggestSmokeState {
    query: String,
    highlighted: Option<ZsAutoSuggestionId>,
    expanded: bool,
}

#[cfg(all(feature = "auto-suggest", feature = "label"))]
fn attach_auto_suggest_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        AutoSuggestSmokeState {
            query: "B".into(),
            highlighted: None,
            expanded: true,
        },
        |state| {
            let highlighted = state.expanded.then_some(state.highlighted).flatten();
            column([
                text::<AutoSuggestSmokeMsg>("ZSUI AutoSuggestBox Smoke")
                    .height(zsui::Dp::new(28.0)),
                auto_suggest_box(
                    state.query.clone(),
                    [
                        ZsAutoSuggestion::new(1_u64, "Alpha"),
                        ZsAutoSuggestion::new(2_u64, "Beta"),
                        ZsAutoSuggestion::new(3_u64, "Bravo"),
                        ZsAutoSuggestion::new(4_u64, "Build"),
                        ZsAutoSuggestion::new(5_u64, "Bundle"),
                        ZsAutoSuggestion::new(6_u64, "Button"),
                        ZsAutoSuggestion::new(7_u64, "Browser"),
                        ZsAutoSuggestion::new(8_u64, "Branch"),
                        ZsAutoSuggestion::new(9_u64, "Baseline"),
                        ZsAutoSuggestion::new(10_u64, "Backend"),
                    ],
                )
                .id(WidgetId::new(23))
                .placeholder("Search components")
                .expanded(state.expanded)
                .highlighted_suggestion(highlighted)
                .no_results_text("No matching components")
                .on_auto_suggest_text_change(AutoSuggestSmokeMsg::TextChanged)
                .on_suggestion_chosen(AutoSuggestSmokeMsg::SuggestionChosen)
                .on_query_submit(AutoSuggestSmokeMsg::Submitted)
                .on_expanded_change(AutoSuggestSmokeMsg::Expanded),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            AutoSuggestSmokeMsg::TextChanged(change) => {
                state.query = change.text;
                if change.reason == ZsAutoSuggestTextChangeReason::UserInput {
                    state.highlighted = None;
                }
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.auto_suggest_text_changed",
                )));
            }
            AutoSuggestSmokeMsg::SuggestionChosen(suggestion) => {
                state.highlighted = Some(suggestion);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.auto_suggest_chosen",
                )));
            }
            AutoSuggestSmokeMsg::Submitted(_submission) => {
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.auto_suggest_submitted",
                )));
            }
            AutoSuggestSmokeMsg::Expanded(expanded) => {
                state.expanded = expanded;
                if !expanded {
                    state.highlighted = None;
                }
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.auto_suggest_expanded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "tree", feature = "label"))]
#[derive(Clone)]
enum TreeSmokeMsg {
    Selected(ZsTreeNodeId),
    Expanded(ZsTreeExpansionChange),
    Invoked(ZsTreeNodeId),
}

#[cfg(all(feature = "tree", feature = "label"))]
struct TreeSmokeState {
    selected: Option<ZsTreeNodeId>,
    expanded: std::collections::BTreeSet<ZsTreeNodeId>,
}

#[cfg(all(feature = "tree", feature = "label"))]
fn attach_tree_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let root = ZsTreeNodeId::new(1);
    let src = ZsTreeNodeId::new(2);
    builder.stateful_view(
        TreeSmokeState {
            selected: Some(src),
            expanded: std::collections::BTreeSet::from([root]),
        },
        move |state| {
            column([
                text::<TreeSmokeMsg>("ZSUI TreeView Smoke").height(zsui::Dp::new(28.0)),
                tree_view([ZsTreeNode::new(root, "Workspace")
                    .icon(zsui::ZsIcon::Folder)
                    .children([
                        ZsTreeNode::new(src, "src")
                            .icon(zsui::ZsIcon::Folder)
                            .children([
                                ZsTreeNode::new(3, "lib.rs").icon(zsui::ZsIcon::File),
                                ZsTreeNode::new(4, "widget_render.rs").icon(zsui::ZsIcon::File),
                            ]),
                        ZsTreeNode::new(5, "examples")
                            .icon(zsui::ZsIcon::Folder)
                            .unrealized_children(true),
                        ZsTreeNode::new(6, "Cargo.toml").icon(zsui::ZsIcon::File),
                    ])])
                .id(WidgetId::new(24))
                .expanded_tree_nodes(state.expanded.iter().copied())
                .selected_tree_node(state.selected)
                .on_tree_select(TreeSmokeMsg::Selected)
                .on_tree_expansion_change(TreeSmokeMsg::Expanded)
                .on_tree_invoke(TreeSmokeMsg::Invoked),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            TreeSmokeMsg::Selected(node) => {
                state.selected = Some(node);
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.tree_selected")));
            }
            TreeSmokeMsg::Expanded(change) => {
                if change.expanded {
                    state.expanded.insert(change.node);
                } else {
                    state.expanded.remove(&change.node);
                }
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.tree_expanded")));
            }
            TreeSmokeMsg::Invoked(_node) => {
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.tree_invoked")));
            }
        },
    )
}

#[cfg(all(feature = "grid-view", feature = "label"))]
#[derive(Clone)]
enum GallerySmokeMsg {
    Selected(ZsGridViewItemId),
    Invoked(ZsGridViewItemId),
}

#[cfg(all(feature = "grid-view", feature = "label"))]
struct GallerySmokeState {
    selected: Option<ZsGridViewItemId>,
    invoked: Option<ZsGridViewItemId>,
}

#[cfg(all(feature = "grid-view", feature = "label"))]
fn attach_gallery_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        GallerySmokeState {
            selected: Some(ZsGridViewItemId::new(1)),
            invoked: None,
        },
        |state| {
            column([
                text::<GallerySmokeMsg>(match state.invoked {
                    Some(item) => format!("ZSUI GridView Smoke · opened {}", item.get()),
                    None => "ZSUI GridView Smoke".to_string(),
                })
                .height(zsui::Dp::new(28.0)),
                grid_view([
                    ZsGridViewItem::new(1, "Desktop")
                        .subtitle("Folder")
                        .icon(zsui::ZsIcon::Folder),
                    ZsGridViewItem::new(2, "Documents")
                        .subtitle("Folder")
                        .icon(zsui::ZsIcon::Folder),
                    ZsGridViewItem::new(3, "Photos")
                        .subtitle("Collection")
                        .icon(zsui::ZsIcon::Image),
                    ZsGridViewItem::new(4, "README")
                        .subtitle("Markdown")
                        .icon(zsui::ZsIcon::Text),
                    ZsGridViewItem::new(5, "src")
                        .subtitle("Folder")
                        .icon(zsui::ZsIcon::Folder),
                    ZsGridViewItem::new(6, "Cargo.toml")
                        .subtitle("Manifest")
                        .icon(zsui::ZsIcon::File),
                    ZsGridViewItem::new(7, "Assets")
                        .subtitle("Resources")
                        .icon(zsui::ZsIcon::Image),
                ])
                .id(WidgetId::new(38))
                .height(zsui::Dp::new(352.0))
                .selected_grid_view_item(state.selected)
                .on_grid_view_select(GallerySmokeMsg::Selected)
                .on_grid_view_invoke(GallerySmokeMsg::Invoked),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            GallerySmokeMsg::Selected(item) => state.selected = Some(item),
            GallerySmokeMsg::Invoked(item) => {
                state.invoked = Some(item);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.grid_view_invoked",
                )));
            }
        },
    )
}

#[cfg(all(feature = "table", feature = "label"))]
#[derive(Clone)]
enum TableSmokeMsg {
    Selected(ZsTableRowId),
    Sorted(ZsTableSort),
    Invoked(ZsTableRowId),
}

#[cfg(all(feature = "table", feature = "label"))]
struct TableSmokeState {
    selected: Option<ZsTableRowId>,
    sort: Option<ZsTableSort>,
}

#[cfg(all(feature = "table", feature = "label"))]
fn attach_table_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let id_column = ZsTableColumnId::new(1);
    let name_column = ZsTableColumnId::new(2);
    let size_column = ZsTableColumnId::new(4);
    builder.stateful_view(
        TableSmokeState {
            selected: Some(ZsTableRowId::new(2)),
            sort: None,
        },
        move |state| {
            let mut rows = vec![
                ZsTableRow::new(1, ["001", "Cargo.toml", "Manifest", "4 KB"]),
                ZsTableRow::new(2, ["002", "src", "Folder", "—"]),
                ZsTableRow::new(3, ["003", "examples", "Folder", "—"]),
                ZsTableRow::new(4, ["004", "README.md", "Markdown", "12 KB"]),
            ];
            if let Some(sort) = state.sort {
                let index = if sort.column == id_column {
                    0
                } else if sort.column == name_column {
                    1
                } else if sort.column == size_column {
                    3
                } else {
                    2
                };
                rows.sort_by(|left, right| left.cell(index).cmp(right.cell(index)));
                if sort.direction == ZsTableSortDirection::Descending {
                    rows.reverse();
                }
            }
            column([
                text::<TableSmokeMsg>("ZSUI DataGrid Smoke").height(zsui::Dp::new(28.0)),
                data_grid(
                    [
                        ZsTableColumn::new(id_column, "ID")
                            .fixed_width(zsui::Dp::new(64.0))
                            .alignment(HorizontalAlign::End)
                            .sortable(true),
                        ZsTableColumn::new(name_column, "Name")
                            .fill_width(2)
                            .sortable(true),
                        ZsTableColumn::new(3, "Type").fill_width(1),
                        ZsTableColumn::new(size_column, "Size")
                            .fixed_width(zsui::Dp::new(88.0))
                            .alignment(HorizontalAlign::End)
                            .sortable(true),
                    ],
                    rows,
                )
                .id(WidgetId::new(25))
                .selected_table_row(state.selected)
                .table_sort(state.sort)
                .on_table_select(TableSmokeMsg::Selected)
                .on_table_sort(TableSmokeMsg::Sorted)
                .on_table_invoke(TableSmokeMsg::Invoked),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            TableSmokeMsg::Selected(row) => {
                state.selected = Some(row);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.table_selected",
                )));
            }
            TableSmokeMsg::Sorted(sort) => {
                state.sort = Some(sort);
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.table_sorted")));
            }
            TableSmokeMsg::Invoked(_row) => {
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.table_invoked")));
            }
        },
    )
}

#[cfg(all(feature = "dialog", feature = "label"))]
#[derive(Clone)]
enum ContentDialogSmokeMsg {
    Responded(ZsContentDialogResult),
}

#[cfg(all(feature = "dialog", feature = "label"))]
struct ContentDialogSmokeState {
    open: bool,
    last_result: Option<ZsContentDialogResult>,
}

#[cfg(all(feature = "dialog", feature = "label"))]
fn attach_content_dialog_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        ContentDialogSmokeState {
            open: true,
            last_result: None,
        },
        |state| {
            let page = column([
                text::<ContentDialogSmokeMsg>("ZSUI ContentDialog Smoke")
                    .height(zsui::Dp::new(28.0)),
                text(format!(
                    "Last typed response: {}",
                    state
                        .last_result
                        .map(|result| format!("{result:?}"))
                        .unwrap_or_else(|| "none".to_string())
                )),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0));
            content_dialog(
                WidgetId::new(26),
                state.open,
                ZsContentDialogSpec::new(
                    "The framework owns the modal focus scope while the application owns whether the dialog is open.",
                    "Cancel",
                )
                .title("Save changes?")
                .primary_button("Save")
                .secondary_button("Discard")
                .default_button(ZsContentDialogButton::Primary)
                .destructive_button(ZsContentDialogButton::Secondary),
                page,
            )
            .on_dialog_result(ContentDialogSmokeMsg::Responded)
        },
        |state, message, cx| match message {
            ContentDialogSmokeMsg::Responded(result) => {
                state.last_result = Some(result);
                // Reopen after the synthetic response so window.png still proves the modal layer.
                state.open = true;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.content_dialog_responded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "toast", feature = "label"))]
#[derive(Clone)]
enum ToastSmokeMsg {
    Responded(ZsToastResult),
}

#[cfg(all(feature = "toast", feature = "label"))]
struct ToastSmokeState {
    toast_id: u64,
    last_result: Option<ZsToastResult>,
}

#[cfg(all(feature = "toast", feature = "label"))]
fn attach_toast_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        ToastSmokeState {
            toast_id: 1,
            last_result: None,
        },
        |state| {
            let page = column([
                text::<ToastSmokeMsg>("ZSUI Toast Smoke").height(zsui::Dp::new(28.0)),
                text(format!(
                    "Last typed response: {}",
                    state
                        .last_result
                        .map(|result| format!("{:?}", result.response))
                        .unwrap_or_else(|| "none".to_string())
                ))
                .height(zsui::Dp::new(28.0)),
                text("Foreground feedback stays inside the shared self-drawn View tree.")
                    .height(zsui::Dp::new(28.0)),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0));
            toast_presenter(
                WidgetId::new(27),
                Some(
                    ZsToastSpec::new(state.toast_id, "File deleted")
                        .action("Undo")
                        .duration(ZsToastDuration::Persistent),
                ),
                page,
            )
            .on_toast_result(ToastSmokeMsg::Responded)
        },
        |state, message, cx| match message {
            ToastSmokeMsg::Responded(result) => {
                state.last_result = Some(result);
                state.toast_id = state.toast_id.saturating_add(1);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.toast_responded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "info-bar", feature = "label"))]
#[derive(Clone)]
enum InfoBarSmokeMsg {
    Invoked(ZsInfoBarEvent),
}

#[cfg(all(feature = "info-bar", feature = "label"))]
struct InfoBarSmokeState {
    last_event: Option<ZsInfoBarEvent>,
}

#[cfg(all(feature = "info-bar", feature = "label"))]
fn attach_info_bar_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        InfoBarSmokeState { last_event: None },
        |state| {
            column([
                text::<InfoBarSmokeMsg>("ZSUI InfoBar Smoke").height(zsui::Dp::new(28.0)),
                text(format!(
                    "Last typed event: {}",
                    state
                        .last_event
                        .map(|event| format!("{event:?}"))
                        .unwrap_or_else(|| "none".to_string())
                ))
                .height(zsui::Dp::new(28.0)),
                info_bar(
                    WidgetId::new(28),
                    ZsInfoBarSpec::new("Renew to keep all functionality.")
                        .title("Subscription expires soon")
                        .severity(ZsInfoBarSeverity::Warning)
                        .action("Renew"),
                )
                .on_info_bar_event(InfoBarSmokeMsg::Invoked),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            InfoBarSmokeMsg::Invoked(event) => {
                state.last_event = Some(event);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.info_bar_invoked",
                )));
            }
        },
    )
}

#[cfg(all(feature = "teaching-tip", feature = "button", feature = "label"))]
#[derive(Clone)]
enum TeachingTipSmokeMsg {
    TargetClicked,
    Responded(ZsTeachingTipResult),
}

#[cfg(all(feature = "teaching-tip", feature = "button", feature = "label"))]
struct TeachingTipSmokeState {
    open: bool,
    last_result: Option<ZsTeachingTipResult>,
}

#[cfg(all(feature = "teaching-tip", feature = "button", feature = "label"))]
fn attach_teaching_tip_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let target = WidgetId::new(29);
    builder.stateful_view(
        TeachingTipSmokeState {
            open: true,
            last_result: None,
        },
        move |state| {
            let page = column([
                text::<TeachingTipSmokeMsg>("ZSUI TeachingTip Smoke").height(zsui::Dp::new(28.0)),
                text(format!(
                    "Last typed response: {}",
                    state
                        .last_result
                        .map(|result| format!("{:?}", result.response))
                        .unwrap_or_else(|| "none".to_string())
                ))
                .height(zsui::Dp::new(28.0)),
                button("Save")
                    .id(target)
                    .height(zsui::Dp::new(36.0))
                    .on_click(TeachingTipSmokeMsg::TargetClicked),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0));
            teaching_tip(
                WidgetId::new(30),
                state.open,
                target,
                ZsTeachingTipSpec::new("Save automatically", "Your changes are saved as you work.")
                    .action("Review settings"),
                page,
            )
            .on_teaching_tip_result(TeachingTipSmokeMsg::Responded)
        },
        |state, message, cx| match message {
            TeachingTipSmokeMsg::TargetClicked => {
                state.open = true;
            }
            TeachingTipSmokeMsg::Responded(result) => {
                state.last_result = Some(result);
                // Reopen so window.png retains the targeted surface after the synthetic action.
                state.open = true;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.teaching_tip_responded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "breadcrumb", feature = "label"))]
#[derive(Clone)]
enum BreadcrumbSmokeMsg {
    Expanded(bool),
    Selected(ZsBreadcrumbId),
}

#[cfg(all(feature = "breadcrumb", feature = "label"))]
struct BreadcrumbSmokeState {
    expanded: bool,
    last_selected: Option<ZsBreadcrumbId>,
}

#[cfg(all(feature = "breadcrumb", feature = "label"))]
fn attach_breadcrumb_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        BreadcrumbSmokeState {
            expanded: false,
            last_selected: None,
        },
        |state| {
            let items = [
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(1), "Home"),
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(2), "Projects"),
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(3), "ZSUI Framework"),
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(4), "Documentation"),
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(5), "Design Assets"),
                ZsBreadcrumbItem::new(ZsBreadcrumbId::new(6), "BreadcrumbBar"),
            ];
            column([
                text::<BreadcrumbSmokeMsg>("ZSUI BreadcrumbBar Smoke").height(zsui::Dp::new(28.0)),
                text(format!(
                    "Last typed selection: {}",
                    state
                        .last_selected
                        .map(|item| item.get().to_string())
                        .unwrap_or_else(|| "none".to_string())
                ))
                .height(zsui::Dp::new(28.0)),
                breadcrumb_bar(items)
                    .id(WidgetId::new(31))
                    .width(zsui::Dp::new(320.0))
                    .expanded(state.expanded)
                    .on_expanded_change(BreadcrumbSmokeMsg::Expanded)
                    .on_breadcrumb_select(BreadcrumbSmokeMsg::Selected),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            BreadcrumbSmokeMsg::Expanded(expanded) => {
                state.expanded = expanded;
            }
            BreadcrumbSmokeMsg::Selected(item) => {
                state.last_selected = Some(item);
                // Keep the overflow visible in window.png after the synthetic selection.
                state.expanded = true;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.breadcrumb_selected",
                )));
            }
        },
    )
}

#[cfg(all(feature = "combo", feature = "label"))]
#[derive(Clone)]
enum ComboSmokeMsg {
    Selected(usize),
    Expanded(bool),
}

#[cfg(all(feature = "combo", feature = "label"))]
struct ComboSmokeState {
    selected: Option<usize>,
    expanded: bool,
}

#[cfg(all(feature = "combo", feature = "label"))]
fn attach_combo_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        ComboSmokeState {
            selected: Some(0),
            expanded: true,
        },
        |state| {
            column([
                text::<ComboSmokeMsg>("ZSUI ComboBox Smoke").height(zsui::Dp::new(28.0)),
                combo_box(
                    [
                        "Balanced",
                        "Fast",
                        "Quiet",
                        "Efficient",
                        "Compact",
                        "Focused",
                        "Silent",
                        "Adaptive",
                        "Performance",
                        "Eco",
                        "Standard",
                        "Dynamic",
                        "Gaming",
                        "Studio",
                        "Travel",
                        "Presentation",
                        "Reading",
                        "Custom",
                        "Legacy",
                        "Experimental",
                    ],
                    state.selected,
                )
                .id(WidgetId::new(13))
                .height(zsui::Dp::new(36.0))
                .expanded(state.expanded)
                .on_select(ComboSmokeMsg::Selected)
                .on_expanded_change(ComboSmokeMsg::Expanded),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            ComboSmokeMsg::Selected(index) => {
                state.selected = Some(index);
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.combo_selected",
                )));
            }
            ComboSmokeMsg::Expanded(expanded) => {
                state.expanded = expanded;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.combo_expanded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "date-picker", feature = "label"))]
#[derive(Clone)]
enum DatePickerSmokeMsg {
    Changed(ZsDate),
    Expanded(bool),
}

#[cfg(all(feature = "date-picker", feature = "label"))]
struct DatePickerSmokeState {
    value: ZsDate,
    expanded: bool,
}

#[cfg(all(feature = "date-picker", feature = "label"))]
fn attach_date_picker_view(
    builder: NativeWindowBuilder,
    high_contrast: bool,
) -> NativeWindowBuilder {
    let theme_mode = if high_contrast {
        ZsuiThemeMode::HighContrast
    } else {
        ZsuiThemeMode::System
    };
    builder.stateful_view(
        DatePickerSmokeState {
            value: ZsDate::new(2026, 7, 13).expect("smoke date should be valid"),
            expanded: true,
        },
        move |state| {
            column([
                text::<DatePickerSmokeMsg>("ZSUI DatePicker smoke").height(zsui::Dp::new(28.0)),
                date_picker(state.value)
                    .id(WidgetId::new(14))
                    .height(zsui::Dp::new(32.0))
                    .expanded(state.expanded)
                    .on_date_change(DatePickerSmokeMsg::Changed)
                    .on_expanded_change(DatePickerSmokeMsg::Expanded),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
            .theme_mode(theme_mode)
        },
        |state, message, cx| match message {
            DatePickerSmokeMsg::Changed(next) => {
                state.value = next;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.date_picker_changed",
                )));
            }
            DatePickerSmokeMsg::Expanded(expanded) => state.expanded = expanded,
        },
    )
}

#[cfg(all(feature = "time-picker", feature = "label"))]
#[derive(Clone)]
enum TimePickerSmokeMsg {
    Changed(ZsTime),
    Expanded(bool),
}

#[cfg(all(feature = "time-picker", feature = "label"))]
struct TimePickerSmokeState {
    value: ZsTime,
    expanded: bool,
}

#[cfg(all(feature = "time-picker", feature = "label"))]
fn attach_time_picker_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.stateful_view(
        TimePickerSmokeState {
            value: ZsTime::new(9, 30).expect("smoke time should be valid"),
            expanded: true,
        },
        |state| {
            column([
                text::<TimePickerSmokeMsg>("ZSUI TimePicker smoke").height(zsui::Dp::new(28.0)),
                time_picker(state.value)
                    .id(WidgetId::new(16))
                    .height(zsui::Dp::new(32.0))
                    .minute_increment(ZsMinuteIncrement::FIFTEEN)
                    .clock_format(ZsClockFormat::TwentyFourHour)
                    .expanded(state.expanded)
                    .on_time_change(TimePickerSmokeMsg::Changed)
                    .on_expanded_change(TimePickerSmokeMsg::Expanded),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |state, message, cx| match message {
            TimePickerSmokeMsg::Changed(next) => {
                state.value = next;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.time_picker_changed",
                )));
            }
            TimePickerSmokeMsg::Expanded(expanded) => {
                state.expanded = expanded;
                cx.ui_command(UiCommand::app(CommandId(
                    "zsui.native_smoke.time_picker_expanded",
                )));
            }
        },
    )
}

#[cfg(all(feature = "tabs", feature = "label"))]
#[derive(Clone)]
enum TabsSmokeMsg {
    Selected(ZsTabId),
}

#[cfg(all(feature = "tabs", feature = "label"))]
fn attach_tabs_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let general = ZsTabId::new(151);
    let advanced = ZsTabId::new(152);
    let about = ZsTabId::new(153);
    builder.stateful_view(
        general,
        move |selected| {
            column([
                text::<TabsSmokeMsg>("ZSUI Tabs smoke").height(zsui::Dp::new(28.0)),
                tab_view(
                    [
                        ZsTabItem::new(
                            general,
                            "General",
                            column([
                                text("General settings"),
                                text("Shared Rust state owns the active page."),
                            ])
                            .padding(zsui::Dp::new(16.0))
                            .gap(zsui::Dp::new(8.0)),
                        ),
                        ZsTabItem::new(
                            advanced,
                            "Advanced",
                            column([
                                text("Advanced settings"),
                                text("Pointer and keyboard selection use typed messages."),
                            ])
                            .padding(zsui::Dp::new(16.0))
                            .gap(zsui::Dp::new(8.0)),
                        ),
                        ZsTabItem::new(
                            about,
                            "About",
                            column([
                                text("ZSUI v0.2"),
                                text("Self-drawn with platform-specific tab behavior."),
                            ])
                            .padding(zsui::Dp::new(16.0))
                            .gap(zsui::Dp::new(8.0)),
                        ),
                    ],
                    Some(*selected),
                )
                .id(WidgetId::new(15))
                .on_tab_select(TabsSmokeMsg::Selected),
            ])
            .padding(zsui::Dp::new(24.0))
            .gap(zsui::Dp::new(12.0))
        },
        |selected, message, cx| match message {
            TabsSmokeMsg::Selected(tab) => {
                *selected = tab;
                cx.ui_command(UiCommand::app(CommandId("zsui.native_smoke.tabs_selected")));
            }
        },
    )
}

#[cfg(not(all(feature = "tabs", feature = "label")))]
fn attach_tabs_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "time-picker", feature = "label")))]
fn attach_time_picker_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "date-picker", feature = "label")))]
fn attach_date_picker_view(
    builder: NativeWindowBuilder,
    _high_contrast: bool,
) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "combo", feature = "label")))]
fn attach_combo_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "auto-suggest", feature = "label")))]
fn attach_auto_suggest_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "tree", feature = "label")))]
fn attach_tree_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "grid-view", feature = "label")))]
fn attach_gallery_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "table", feature = "label")))]
fn attach_table_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "dialog", feature = "label")))]
fn attach_content_dialog_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "toast", feature = "label")))]
fn attach_toast_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "info-bar", feature = "label")))]
fn attach_info_bar_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "teaching-tip", feature = "button", feature = "label")))]
fn attach_teaching_tip_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "breadcrumb", feature = "label")))]
fn attach_breadcrumb_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "progress", feature = "label")))]
fn attach_progress_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(not(all(feature = "progress-ring", feature = "label")))]
fn attach_progress_ring_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

fn smoke_window_menu() -> MenuSpec {
    let mut file = MenuSpec::new();
    file.items.push(
        MenuItemSpec::command("Open", Command::custom("zsui.native_smoke.open"))
            .accelerator("Primary+O"),
    );
    file.items.push(
        MenuItemSpec::command("Save", Command::custom("zsui.native_smoke.save"))
            .accelerator("Primary+S"),
    );
    file.items.push(MenuItemSpec::Separator);
    file.items.push(
        MenuItemSpec::command("Disabled", Command::custom("zsui.native_smoke.disabled")).disabled(),
    );
    MenuSpec::new().title("ZSUI Smoke").submenu("File", file)
}

#[cfg(all(feature = "button", feature = "label"))]
fn attach_typed_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    let mut children = vec![text::<UiCommand>("ZSUI Native View Smoke")];
    #[cfg(feature = "textbox")]
    {
        children.push(
            textbox("A中文Z")
                .id(WidgetId::new(2))
                .on_change(native_smoke_text_changed),
        );
    }
    #[cfg(feature = "list")]
    {
        children.push(
            list(
                [
                    (WidgetId::new(4), "Recent item"),
                    (WidgetId::new(5), "Pinned item"),
                ],
                |(id, label)| text(label).id(id),
            )
            .on_select(native_smoke_list_selected),
        );
    }
    #[cfg(feature = "checkbox")]
    {
        children.push(
            checkbox("Dark mode", false)
                .id(WidgetId::new(3))
                .on_toggle(native_smoke_toggle_changed),
        );
    }
    children.push(
        button("Save")
            .id(WidgetId::new(1))
            .on_click(UiCommand::app(CommandId("zsui.native_smoke.save"))),
    );
    builder.ui_command_view(column(children))
}

#[cfg(all(feature = "button", feature = "label", feature = "textbox"))]
fn native_smoke_text_changed(_: String) -> UiCommand {
    UiCommand::app(CommandId("zsui.native_smoke.text_changed"))
}

#[cfg(all(feature = "button", feature = "label", feature = "checkbox"))]
fn native_smoke_toggle_changed(_: bool) -> UiCommand {
    UiCommand::app(CommandId("zsui.native_smoke.toggle_changed"))
}

#[cfg(all(feature = "button", feature = "label", feature = "list"))]
fn native_smoke_list_selected(_: usize) -> UiCommand {
    UiCommand::app(CommandId("zsui.native_smoke.list_selected"))
}

#[cfg(all(feature = "button", feature = "label", feature = "scroll"))]
fn attach_scroll_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder.ui_command_view(column([
        text::<UiCommand>("ZSUI Scroll Smoke"),
        scroll(
            column([
                text("Pinned row").id(WidgetId::new(7)),
                text("Recent row").id(WidgetId::new(8)),
                text("Archive row").id(WidgetId::new(9)),
            ])
            .content_height(zsui::Dp::new(240.0)),
        )
        .id(WidgetId::new(6))
        .content_height(zsui::Dp::new(240.0))
        .on_scroll(native_smoke_scrolled),
    ]))
}

#[cfg(not(all(feature = "button", feature = "label", feature = "scroll")))]
fn attach_scroll_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

#[cfg(all(feature = "button", feature = "label", feature = "scroll"))]
fn native_smoke_scrolled(_: zsui::Dp) -> UiCommand {
    UiCommand::app(CommandId("zsui.native_smoke.scrolled"))
}

#[cfg(not(all(feature = "button", feature = "label")))]
fn attach_typed_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
    builder
}

fn parse_platform(platform: &str) -> Result<NativeUiPlatform, String> {
    if platform == "current" {
        return native_ui_platform_for_current_target()
            .ok_or_else(|| "current target is not a supported ZSUI platform".to_string());
    }

    match platform {
        "windows" => Ok(NativeUiPlatform::Windows),
        "macos" => Ok(NativeUiPlatform::Macos),
        "linux" => Ok(NativeUiPlatform::Linux),
        "android" => Ok(NativeUiPlatform::Android),
        "harmony" => Ok(NativeUiPlatform::Harmony),
        _ => Err(format!("unknown ZSUI platform `{platform}`")),
    }
}
