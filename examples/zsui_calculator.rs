#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf};

use zsui::{
    calculator_view, native_window, AppCx, NativeDrawCommand, NativeWindowSmokeRunOptions, Point,
    TextRole, ViewInteractionPlan, ViewNode, ZsCalculatorAction, ZsCalculatorBinaryOperator,
    ZsCalculatorEngine, ZsCalculatorShellSpec, ZsCalculatorViewIds, ZsuiError, ZsuiResult,
};

const CALCULATOR_IDS: ZsCalculatorViewIds = ZsCalculatorViewIds::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Msg {
    Action(ZsCalculatorAction),
}

#[derive(Debug, Default)]
struct CalculatorState {
    engine: ZsCalculatorEngine,
    history_visible: bool,
}

fn view(state: &CalculatorState) -> ViewNode<Msg> {
    let spec =
        ZsCalculatorShellSpec::from_engine(&state.engine).history_visible(state.history_visible);
    calculator_view(&spec, CALCULATOR_IDS, Msg::Action)
}

fn update(state: &mut CalculatorState, message: Msg, _cx: &mut AppCx) {
    let Msg::Action(action) = message;
    match action {
        ZsCalculatorAction::ToggleHistory => {
            state.history_visible = !state.history_visible;
        }
        ZsCalculatorAction::ClearHistory => state.engine.apply(action),
        _ => {
            state.history_visible = false;
            state.engine.apply(action);
        }
    }
}

fn main() -> ZsuiResult<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let smoke = arguments.iter().any(|argument| argument == "--smoke");
    let benchmark_duration_ms = benchmark_duration_ms(&arguments);
    let builder = native_window("ZSUI Calculator")
        .size(420, 680)
        .min_size(360, 560)
        .stateful_view(CalculatorState::default(), view, update);

    if smoke {
        return run_smoke(builder);
    }
    if let Some(duration_ms) = benchmark_duration_ms {
        builder.run_smoke(NativeWindowSmokeRunOptions::new(duration_ms))?;
        return Ok(());
    }

    builder.run()?;
    Ok(())
}

fn run_smoke(builder: zsui::NativeWindowBuilder) -> ZsuiResult<()> {
    let artifact_dir = PathBuf::from("target/zsui-calculator");
    fs::create_dir_all(&artifact_dir)
        .map_err(|error| ZsuiError::host("create_calculator_smoke_dir", error.to_string()))?;
    let interaction = builder
        .native_view_interaction_plan()
        .ok_or_else(|| ZsuiError::host("calculator_smoke", "missing interaction plan"))?;
    let actions = [
        ZsCalculatorAction::Digit(1),
        ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
        ZsCalculatorAction::Digit(2),
        ZsCalculatorAction::Equals,
    ];
    let mut options = NativeWindowSmokeRunOptions::new(1_800)
        .screenshot_file(artifact_dir.join("window.png").to_string_lossy())
        .require_screenshot(true);
    for action in actions {
        options = options.native_view_click(action_center(interaction, action)?);
    }
    let live_runtime = builder
        .native_live_view_runtime()
        .cloned()
        .ok_or_else(|| ZsuiError::host("calculator_smoke", "missing live View runtime"))?;
    let report = builder.run_smoke(options)?;
    let final_display_is_three = live_runtime.draw_plan().commands.iter().any(|command| {
        matches!(
            command,
            NativeDrawCommand::Text(text)
                if text.text == "3"
                    && matches!(text.style.role, TextRole::Display | TextRole::Title)
        )
    });
    if !report.visible_window_was_created()
        || !report.screenshot_captured
        || report.native_view_pointer_down_count < actions.len()
        || report.native_view_pointer_up_count < actions.len()
        || report.native_view_message_count < actions.len()
        || report.native_view_live_revision < actions.len() as u64
        || !final_display_is_three
    {
        return Err(ZsuiError::host(
            "calculator_smoke",
            format!(
                "verification failed: visible={}, screenshot={}, pointer_down={}, pointer_up={}, messages={}, revision={}, final_display_three={}",
                report.visible_window_was_created(),
                report.screenshot_captured,
                report.native_view_pointer_down_count,
                report.native_view_pointer_up_count,
                report.native_view_message_count,
                report.native_view_live_revision,
                final_display_is_three
            ),
        ));
    }
    fs::write(
        artifact_dir.join("report.json"),
        serde_json::to_vec_pretty(&report)
            .map_err(|error| ZsuiError::host("serialize_calculator_smoke", error.to_string()))?,
    )
    .map_err(|error| ZsuiError::host("write_calculator_smoke", error.to_string()))?;
    Ok(())
}

fn action_center(
    interaction: &ViewInteractionPlan,
    action: ZsCalculatorAction,
) -> ZsuiResult<Point> {
    let target = interaction
        .hit_target_for_widget(CALCULATOR_IDS.for_action(action))
        .ok_or_else(|| {
            ZsuiError::host(
                "calculator_smoke",
                format!("missing hit target for {action:?}"),
            )
        })?;
    Ok(Point {
        x: target.bounds.x + target.bounds.width / 2,
        y: target.bounds.y + target.bounds.height / 2,
    })
}

fn benchmark_duration_ms(arguments: &[String]) -> Option<u64> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--benchmark-seconds")
        .and_then(|pair| pair[1].parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1_000).max(250))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_view_and_update_complete_a_typed_calculation() {
        let mut state = CalculatorState::default();
        let mut cx = AppCx::new();
        for action in [
            ZsCalculatorAction::Digit(1),
            ZsCalculatorAction::Binary(ZsCalculatorBinaryOperator::Add),
            ZsCalculatorAction::Digit(2),
            ZsCalculatorAction::Equals,
        ] {
            update(&mut state, Msg::Action(action), &mut cx);
        }
        assert_eq!(state.engine.display(), "3");

        let mut view = view(&state);
        let mut layout = zsui::ViewLayoutCx::new(
            zsui::Rect {
                x: 0,
                y: 0,
                width: 420,
                height: 680,
            },
            zsui::Dpi::standard(),
        );
        zsui::View::layout(&mut view, &mut layout);
        assert_eq!(view.interaction_plan().hit_target_count(), 28);
    }
}
