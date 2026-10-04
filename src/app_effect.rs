use std::{any::Any, fmt, path::PathBuf, sync::Arc};

use crate::{
    DialogResponse, DirectoryDialogSpec, FileDialogService, FileDialogSpec,
    NativeDesktopDialogService, NativeDialogService, NativeDialogSpec, NativeFileDialogService,
    SaveFileDialogSpec, ZsuiResult,
};

/// A platform-neutral native effect requested from `update` through
/// [`AppCx`](crate::AppCx).
///
/// Desktop hosts always execute effects outside the live-view and route locks,
/// so a synchronous modal dialog cannot re-enter a held update. Each queued
/// effect produces exactly one [`AppEffectOutcome`] that is delivered back to
/// the typed update at most once; outcomes are dropped when the window is
/// destroyed before the dialog finishes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AppEffect {
    /// Opens the native file-open dialog. `Ok(None)` means the user cancelled.
    OpenFileDialog(FileDialogSpec),
    /// Opens the native save dialog. `Ok(None)` means the user cancelled.
    SaveFileDialog(SaveFileDialogSpec),
    /// Opens a native directory picker. `Ok(None)` means the user cancelled.
    PickDirectory(DirectoryDialogSpec),
    /// Shows a native message/confirmation dialog and reports the typed
    /// [`DialogResponse`].
    ShowDialog(NativeDialogSpec),
}

/// The typed result of one executed [`AppEffect`].
///
/// The outcome variant always matches the request variant. Cancellation is
/// `Ok(None)` for pickers and the corresponding [`DialogResponse`] for message
/// dialogs; backend failures surface as `Err(ZsuiError)` instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AppEffectOutcome {
    /// Result of [`AppEffect::OpenFileDialog`].
    OpenFileDialog(ZsuiResult<Option<Vec<PathBuf>>>),
    /// Result of [`AppEffect::SaveFileDialog`].
    SaveFileDialog(ZsuiResult<Option<PathBuf>>),
    /// Result of [`AppEffect::PickDirectory`].
    PickDirectory(ZsuiResult<Option<PathBuf>>),
    /// Result of [`AppEffect::ShowDialog`].
    ShowDialog(ZsuiResult<DialogResponse>),
}

/// Turns an [`AppEffectOutcome`] into a boxed application message. Returning
/// `None` drops the outcome (for example when the outcome kind does not match
/// the request, which hosts never produce for well-formed dispatches).
type AppEffectResponder =
    Arc<dyn Fn(AppEffectOutcome) -> Option<Box<dyn Any + Send>> + Send + Sync>;

/// One queued [`AppEffect`] plus the responder that maps its outcome back into
/// the application's typed message type.
///
/// Requests are created by the [`AppCx`](crate::AppCx) effect helpers and are
/// consumed exactly once by the desktop host: it executes
/// [`AppEffectRequest::effect`] outside the live-view lock and then delivers
/// the outcome through the stored responder.
#[derive(Clone)]
pub struct AppEffectRequest {
    effect: AppEffect,
    responder: Option<AppEffectResponder>,
}

impl AppEffectRequest {
    /// Queues `effect` without a responder; its outcome is discarded after the
    /// dialog finishes.
    pub fn new(effect: AppEffect) -> Self {
        Self {
            effect,
            responder: None,
        }
    }

    /// The platform-neutral effect the host executes outside the update lock.
    pub fn effect(&self) -> &AppEffect {
        &self.effect
    }

    pub(crate) fn typed<Msg, Respond>(effect: AppEffect, respond: Respond) -> Self
    where
        Msg: Send + 'static,
        Respond: Fn(AppEffectOutcome) -> Option<Msg> + Send + Sync + 'static,
    {
        Self::with_responder(
            effect,
            Arc::new(move |outcome| {
                respond(outcome).map(|message| Box::new(message) as Box<dyn Any + Send>)
            }),
        )
    }

    pub(crate) fn with_responder(effect: AppEffect, responder: AppEffectResponder) -> Self {
        Self {
            effect,
            responder: Some(responder),
        }
    }

    /// Produces the boxed application message for `outcome`, or `None` when the
    /// request has no responder or the outcome kind does not match.
    pub(crate) fn respond(self, outcome: AppEffectOutcome) -> Option<Box<dyn Any + Send>> {
        self.responder.and_then(|responder| responder(outcome))
    }
}

impl fmt::Debug for AppEffectRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppEffectRequest")
            .field("effect", &self.effect)
            .field("has_responder", &self.responder.is_some())
            .finish()
    }
}

/// Executes one queued effect against the selected desktop backend.
///
/// Desktop hosts call this only after releasing the live-view/route lock, so
/// the synchronous modal dialog runs on the UI thread without re-entering a
/// held update. The returned outcome is then delivered through
/// `dispatch_app_effect_outcome` exactly once.
#[allow(dead_code)] // Called by the per-host pending-effect drain patches.
pub(crate) fn execute_native_app_effect(effect: &AppEffect) -> AppEffectOutcome {
    match effect {
        AppEffect::OpenFileDialog(spec) => {
            AppEffectOutcome::OpenFileDialog(NativeFileDialogService::new().open_file_dialog(spec))
        }
        AppEffect::SaveFileDialog(spec) => {
            AppEffectOutcome::SaveFileDialog(NativeFileDialogService::new().save_file_dialog(spec))
        }
        AppEffect::PickDirectory(spec) => AppEffectOutcome::PickDirectory(
            NativeFileDialogService::new().pick_directory_dialog(spec),
        ),
        AppEffect::ShowDialog(spec) => {
            AppEffectOutcome::ShowDialog(NativeDesktopDialogService::new().show_native_dialog(spec))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{live_view_runtime, AppCx, Dpi, Rect};

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Opened(ZsuiResult<Option<Vec<PathBuf>>>),
        Saved(ZsuiResult<Option<PathBuf>>),
        Confirmed(ZsuiResult<DialogResponse>),
    }

    fn recording_runtime(seen: Arc<std::sync::Mutex<Vec<Msg>>>) -> crate::SharedLiveViewRuntime {
        live_view_runtime(
            (),
            |_: &()| crate::spacer(),
            move |_: &mut (), message: Msg, _cx: &mut AppCx| {
                seen.lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(message);
            },
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
            Dpi::standard(),
        )
    }

    fn take_effects(cx: &AppCx) -> Vec<AppEffectRequest> {
        cx.effects().to_vec()
    }

    #[test]
    fn app_cx_queues_native_effects_in_request_order() {
        let mut cx = AppCx::new();
        cx.open_file_dialog(FileDialogSpec::new("Open"), Msg::Opened);
        cx.save_file_dialog(SaveFileDialogSpec::new("Save"), Msg::Saved);
        cx.pick_directory(DirectoryDialogSpec::new("Folder"), |result| {
            Msg::Saved(result)
        });
        cx.show_dialog(
            NativeDialogSpec::message("Confirm", "Continue?"),
            Msg::Confirmed,
        );
        cx.effect(AppEffect::ShowDialog(NativeDialogSpec::message(
            "Note", "Done",
        )));

        let effects = take_effects(&cx);
        assert_eq!(effects.len(), 5);
        assert!(matches!(
            effects[0].effect(),
            AppEffect::OpenFileDialog(spec) if spec.title == "Open"
        ));
        assert!(matches!(
            effects[1].effect(),
            AppEffect::SaveFileDialog(spec) if spec.title == "Save"
        ));
        assert!(matches!(
            effects[2].effect(),
            AppEffect::PickDirectory(spec) if spec.title == "Folder"
        ));
        assert!(matches!(
            effects[3].effect(),
            AppEffect::ShowDialog(spec) if spec.title == "Confirm"
        ));
        assert!(matches!(effects[4].effect(), AppEffect::ShowDialog(_)));
    }

    #[test]
    fn app_effect_outcome_returns_to_typed_update_exactly_once() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let runtime = recording_runtime(seen.clone());
        let mut cx = AppCx::new();
        cx.open_file_dialog(FileDialogSpec::new("Open"), Msg::Opened);
        let request = take_effects(&cx).into_iter().next().unwrap();

        let update = runtime.dispatch_app_effect(
            request,
            AppEffectOutcome::OpenFileDialog(Ok(Some(vec![PathBuf::from("picked.txt")]))),
        );

        assert_eq!(update.message_count, 1);
        assert!(update.redraw);
        assert_eq!(
            seen.lock().unwrap().as_slice(),
            &[Msg::Opened(Ok(Some(vec![PathBuf::from("picked.txt")])))]
        );
    }

    #[test]
    fn app_effect_without_responder_discards_the_outcome() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let runtime = recording_runtime(seen.clone());
        let request =
            AppEffectRequest::new(AppEffect::PickDirectory(DirectoryDialogSpec::new("Folder")));

        let update = runtime.dispatch_app_effect(
            request,
            AppEffectOutcome::PickDirectory(Ok(Some(PathBuf::from("dir")))),
        );

        assert_eq!(update.message_count, 0);
        assert!(seen.lock().unwrap().is_empty());
    }

    #[test]
    fn app_effect_outcome_kind_mismatch_is_dropped() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let runtime = recording_runtime(seen.clone());
        let responder: AppEffectResponder = Arc::new(|outcome| match outcome {
            AppEffectOutcome::ShowDialog(result) => {
                Some(Box::new(Msg::Confirmed(result)) as Box<dyn Any + Send>)
            }
            _ => None,
        });
        let request = AppEffectRequest::with_responder(
            AppEffect::ShowDialog(NativeDialogSpec::message("Confirm", "Continue?")),
            responder,
        );

        let update = runtime.dispatch_app_effect(
            request,
            AppEffectOutcome::PickDirectory(Ok(Some(PathBuf::from("dir")))),
        );

        assert_eq!(update.message_count, 0);
        assert!(seen.lock().unwrap().is_empty());
    }

    #[test]
    fn update_queued_effects_are_deferred_into_the_live_view_update() {
        let runtime = live_view_runtime(
            (),
            |_: &()| crate::spacer(),
            |_: &mut (), message: Msg, cx: &mut AppCx| {
                if let Msg::Confirmed(_) = message {
                    cx.open_file_dialog(FileDialogSpec::new("Next"), Msg::Opened);
                }
            },
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
            Dpi::standard(),
        );
        let mut cx = AppCx::new();
        cx.show_dialog(
            NativeDialogSpec::message("Confirm", "Continue?"),
            Msg::Confirmed,
        );
        let request = take_effects(&cx).into_iter().next().unwrap();

        let update = runtime.dispatch_app_effect(
            request,
            AppEffectOutcome::ShowDialog(Ok(DialogResponse::Ok)),
        );

        assert_eq!(update.message_count, 1);
        assert_eq!(update.effects.len(), 1);
        assert!(matches!(
            update.effects[0].effect(),
            AppEffect::OpenFileDialog(spec) if spec.title == "Next"
        ));
    }

    #[test]
    fn suspended_view_still_applies_the_effect_outcome_to_state() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let runtime = recording_runtime(seen.clone());
        runtime.suspend();
        let mut cx = AppCx::new();
        cx.show_dialog(
            NativeDialogSpec::message("Confirm", "Continue?"),
            Msg::Confirmed,
        );
        let request = take_effects(&cx).into_iter().next().unwrap();

        let update = runtime.dispatch_app_effect(
            request,
            AppEffectOutcome::ShowDialog(Ok(DialogResponse::Yes)),
        );

        assert_eq!(update.message_count, 1);
        assert!(!update.redraw);
        assert_eq!(
            seen.lock().unwrap().as_slice(),
            &[Msg::Confirmed(Ok(DialogResponse::Yes))]
        );
    }

    #[test]
    fn directory_picker_default_service_reports_unsupported() {
        struct NoopDialogs;

        impl FileDialogService for NoopDialogs {
            fn open_file_dialog(
                &mut self,
                _spec: &FileDialogSpec,
            ) -> ZsuiResult<Option<Vec<PathBuf>>> {
                Ok(None)
            }

            fn save_file_dialog(
                &mut self,
                _spec: &SaveFileDialogSpec,
            ) -> ZsuiResult<Option<PathBuf>> {
                Ok(None)
            }
        }

        let result = NoopDialogs.pick_directory_dialog(&DirectoryDialogSpec::new("Folder"));
        assert!(matches!(
            result,
            Err(crate::ZsuiError::Unsupported { capability, .. }) if capability == "pick_directory_dialog"
        ));
    }
}
