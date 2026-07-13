use std::{env, fs, path::PathBuf, process::ExitCode};

use serde_json::json;
#[cfg(feature = "button")]
use zsui::button;
#[cfg(all(feature = "button", feature = "label", feature = "checkbox"))]
use zsui::checkbox;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "progress", feature = "label"),
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
#[cfg(feature = "radio")]
use zsui::radio_button;
#[cfg(all(feature = "button", feature = "label", feature = "scroll"))]
use zsui::scroll;
#[cfg(feature = "label")]
use zsui::text;
#[cfg(all(feature = "button", feature = "label", feature = "textbox"))]
use zsui::textbox;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::CommandId;
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label")
))]
use zsui::NativeViewKey;
#[cfg(feature = "date-picker")]
use zsui::{date_picker, ZsDate, ZsuiThemeMode};
#[cfg(all(feature = "grid", feature = "button", feature = "label"))]
use zsui::{grid, ZsGridCell, ZsGridFraction, ZsGridSpan, ZsGridTrack};
use zsui::{
    native_ui_platform_for_current_target, native_window,
    write_native_host_smoke_artifacts_with_interaction_to, Command, MenuItemSpec, MenuSpec,
    NativeHostSmokeInteractionReport, NativeUiPlatform, NativeWindowBuilder,
    NativeWindowRuntimeDriver, NativeWindowSmokeRunOptions, TraySpec,
};
#[cfg(feature = "progress")]
use zsui::{progress_bar, ProgressRange};
#[cfg(feature = "slider")]
use zsui::{slider, SliderRange};
#[cfg(feature = "tabs")]
use zsui::{tab_view, ZsTabId, ZsTabItem};
#[cfg(feature = "time-picker")]
use zsui::{time_picker, ZsClockFormat, ZsMinuteIncrement, ZsTime};
#[cfg(any(
    all(feature = "button", feature = "label"),
    all(feature = "slider", feature = "label"),
    all(feature = "radio", feature = "label"),
    all(feature = "combo", feature = "label"),
    all(feature = "date-picker", feature = "label"),
    all(feature = "time-picker", feature = "label"),
    all(feature = "tabs", feature = "label"),
    all(feature = "grid", feature = "button", feature = "label")
))]
use zsui::{Point, UiCommand, WidgetId};

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
        args.iter().any(|arg| arg == "--view"),
        args.iter()
            .any(|arg| arg == "--scroll-view" || arg == "--scroll"),
        args.iter()
            .any(|arg| arg == "--slider-view" || arg == "--slider"),
        args.iter()
            .any(|arg| arg == "--radio-view" || arg == "--radio"),
        args.iter()
            .any(|arg| arg == "--progress-view" || arg == "--progress"),
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
    include_typed_view: bool,
    include_scroll_view: bool,
    include_slider_view: bool,
    include_radio_view: bool,
    include_progress_view: bool,
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
    #[cfg(not(all(feature = "slider", feature = "label")))]
    if include_slider_view {
        return Err("--slider-view requires the slider and label features".to_string());
    }
    #[cfg(not(all(feature = "radio", feature = "label")))]
    if include_radio_view {
        return Err("--radio-view requires the radio and label features".to_string());
    }
    #[cfg(not(all(feature = "progress", feature = "label")))]
    if include_progress_view {
        return Err("--progress-view requires the progress and label features".to_string());
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
    #[cfg(all(feature = "slider", feature = "label"))]
    if include_slider_view {
        smoke_options = smoke_options
            .native_view_drag(Point { x: 100, y: 84 }, Point { x: 400, y: 84 })
            .native_view_key_down(NativeViewKey::Left);
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
    } else if include_slider_view {
        attach_slider_view(builder)
    } else if include_radio_view {
        attach_radio_view(builder)
    } else if include_progress_view {
        attach_progress_view(builder)
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

#[cfg(not(all(feature = "progress", feature = "label")))]
fn attach_progress_view(builder: NativeWindowBuilder) -> NativeWindowBuilder {
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
