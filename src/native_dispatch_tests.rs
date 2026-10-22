use super::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
enum Msg {
    Start(usize),
    Done(usize),
}
fn runtime(chain: usize, received: Arc<Mutex<Vec<usize>>>) -> NativeViewInputRuntime {
    let builder = native_window("Dispatch test")
        .size(200, 100)
        .stateful_view_with_app_commands(
            (),
            |_| crate::spacer(),
            move |_, message, cx| {
                let queue = |index: usize, cx: &mut crate::AppCx| {
                    cx.try_show_dialog(
                        crate::NativeDialogSpec::message("Effect", index.to_string()),
                        move |_| Msg::Done(index),
                    )
                    .unwrap();
                };
                match message {
                    Msg::Start(count) => {
                        for index in 0..count {
                            queue(index, cx);
                        }
                    }
                    Msg::Done(index) => {
                        received.lock().unwrap().push(index);
                        if index + 1 < chain {
                            queue(index + 1, cx);
                        }
                    }
                }
            },
            |command| match command {
                Command::Custom { id, .. } => id.parse().ok().map(Msg::Start),
                _ => None,
            },
        );
    let mut runtime = builder.native_view_input_runtime();
    runtime.defer_app_command_execution();
    runtime.defer_ui_command_execution();
    runtime
}
fn complete(
    runtime: &mut NativeViewInputRuntime,
    request: crate::AppEffectRequest,
    fail: bool,
) -> NativeViewInputDispatchReport {
    assert!(request.try_begin());
    // Re-entering read/dispatch APIs proves that execution holds no live-view
    // mutex; the active dispatcher remains exclusive during a modal loop.
    let view = runtime.live_view.as_ref().unwrap().clone();
    view.interaction_plan();
    let outcome = crate::AppEffectOutcome::ShowDialog(if fail {
        Err(crate::ZsuiError::host("test", "provider failed"))
    } else {
        Ok(crate::DialogResponse::Ok)
    });
    runtime.dispatch_app_effect_outcome(request, outcome)
}

#[test]
fn deferred_dispatch_preserves_one_thousand_requests_and_one_hundred_chained_results() {
    for (batch, chain, total) in [(1000usize, 0, 1000usize), (1, 100, 100)] {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = runtime(chain, seen.clone());
        runtime.dispatch_app_command(Command::custom(batch.to_string()));
        let mut turns = 0;
        while runtime.has_pending_dispatch() {
            assert_eq!(runtime.transient_poll_interval_ms(), Some(1));
            let turn = runtime.begin_deferred_dispatch().unwrap();
            assert!(runtime.begin_deferred_dispatch().is_none());
            assert_eq!(runtime.transient_poll_interval_ms(), None);
            let mut budget = crate::dispatch_queue::DispatchTurnBudget::new();
            let mut count = 0;
            while budget.take() {
                let Some(request) = runtime.pop_pending_app_effect() else {
                    break;
                };
                let report = complete(&mut runtime, request, count == 5);
                assert_eq!(report.message_count, 1);
                count += 1;
            }
            assert!(count <= crate::DISPATCH_BATCH_SIZE);
            drop(turn);
            turns += 1;
            assert!(turns < 2000);
        }
        assert!(turns >= total.div_ceil(crate::DISPATCH_BATCH_SIZE));
        assert_eq!(*seen.lock().unwrap(), (0..total).collect::<Vec<_>>());
        assert_eq!(runtime.transient_poll_interval_ms(), None);
    }
}

#[test]
fn deferred_dispatch_cancel_duplicate_and_destroy_before_during_after_execution() {
    for stage in 0..4 {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = runtime(0, seen.clone());
        runtime.dispatch_app_command(Command::custom("2"));
        let request = runtime.pop_pending_app_effect().unwrap();
        match stage {
            0 => {
                assert!(request.cancellation().cancel());
                assert!(!request.try_begin());
            }
            1 => {
                runtime.shutdown_dispatch();
                assert!(!request.try_begin());
            }
            2 => {
                assert!(request.try_begin());
                runtime.shutdown_dispatch();
            }
            _ => {
                assert!(request.try_begin());
                assert!(request.cancellation().cancel());
            }
        }
        let report = runtime.dispatch_app_effect_outcome(
            request,
            crate::AppEffectOutcome::ShowDialog(Ok(crate::DialogResponse::Ok)),
        );
        assert_eq!(report.message_count, 0);
        runtime.shutdown_dispatch();
        assert!(!runtime.has_pending_dispatch());
        assert!(seen.lock().unwrap().is_empty());
    }
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = runtime(0, seen.clone());
    runtime.dispatch_app_command(Command::custom("1"));
    let request = runtime.pop_pending_app_effect().unwrap();
    let duplicate = request.clone();
    complete(&mut runtime, request, false);
    assert!(!duplicate.try_begin());
    assert_eq!(
        runtime
            .dispatch_app_effect_outcome(
                duplicate,
                crate::AppEffectOutcome::ShowDialog(Ok(crate::DialogResponse::Ok))
            )
            .message_count,
        0
    );
    runtime.shutdown_dispatch();
    assert_eq!(*seen.lock().unwrap(), vec![0]);
}

#[test]
fn deferred_dispatch_source_admission_uses_capacity_remaining_across_updates() {
    let rejected = Arc::new(Mutex::new(Vec::new()));
    let rejected_update = rejected.clone();
    let builder = native_window("Bounded actions").stateful_view_with_app_commands(
        (),
        |_| crate::spacer(),
        move |_, _: (), cx| {
            let mut accepted = 0;
            for _ in 0..crate::DEFAULT_DISPATCH_QUEUE_CAPACITY + 7 {
                if cx
                    .try_show_dialog(crate::NativeDialogSpec::message("x", "y"), |_| ())
                    .is_ok()
                {
                    accepted += 1;
                }
            }
            rejected_update.lock().unwrap().push(accepted);
        },
        |_| Some(()),
    );
    let mut runtime = builder.native_view_input_runtime();
    let first = runtime.dispatch_app_command(Command::custom("fill"));
    assert_eq!(
        runtime.pending_app_effects.len(),
        crate::DEFAULT_DISPATCH_QUEUE_CAPACITY
    );
    assert!(first.errors.iter().any(|error| error.contains("7 actions")));
    let second = runtime.dispatch_app_command(Command::custom("full"));
    assert_eq!(
        runtime.pending_app_effects.len(),
        crate::DEFAULT_DISPATCH_QUEUE_CAPACITY
    );
    assert!(second
        .errors
        .iter()
        .any(|error| error.contains("4103 actions")));
    runtime
        .pop_pending_app_effect()
        .unwrap()
        .cancellation()
        .cancel();
    runtime.dispatch_app_command(Command::custom("one slot"));
    assert_eq!(*rejected.lock().unwrap(), vec![4096, 0, 1]);
    runtime.shutdown_dispatch();
}

#[test]
fn deferred_dispatch_command_batches_retain_fifo_tail() {
    let mut runtime = NativeViewInputRuntime::default();
    runtime
        .pending_app_commands
        .extend((0..1000).map(|index| Command::custom(index.to_string())));
    let mut actual = Vec::new();
    while runtime.has_pending_dispatch() {
        let (_, batch) = runtime.take_pending_app_command_dispatch();
        assert!(batch.len() <= crate::DISPATCH_BATCH_SIZE);
        actual.extend(batch);
    }
    assert_eq!(
        actual,
        (0..1000)
            .map(|index| Command::custom(index.to_string()))
            .collect::<Vec<_>>()
    );
}
