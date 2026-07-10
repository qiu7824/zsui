use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard},
};

use serde::Serialize;

use crate::{AppEvent, Command, ZsuiResult};

pub trait AppCommandExecutor: Send {
    fn execute_app_command(&mut self, command: Command) -> ZsuiResult<Vec<AppEvent>>;
}

impl<F> AppCommandExecutor for F
where
    F: FnMut(Command) -> ZsuiResult<Vec<AppEvent>> + Send,
{
    fn execute_app_command(&mut self, command: Command) -> ZsuiResult<Vec<AppEvent>> {
        self(command)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AppCommandDispatchReport {
    pub submitted_count: usize,
    pub executed_count: usize,
    pub failed_count: usize,
    pub emitted_event_count: usize,
    pub command_names: Vec<&'static str>,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct SharedAppCommandExecutor {
    inner: Arc<SharedAppCommandExecutorInner>,
}

struct SharedAppCommandExecutorInner {
    executor: Mutex<Box<dyn AppCommandExecutor>>,
    report: Mutex<AppCommandDispatchReport>,
}

impl SharedAppCommandExecutor {
    pub fn new(executor: impl AppCommandExecutor + 'static) -> Self {
        Self {
            inner: Arc::new(SharedAppCommandExecutorInner {
                executor: Mutex::new(Box::new(executor)),
                report: Mutex::new(AppCommandDispatchReport::default()),
            }),
        }
    }

    pub fn dispatch(&self, command: Command) -> ZsuiResult<Vec<AppEvent>> {
        let command_name = app_command_name(&command);
        let result = self.executor().execute_app_command(command);
        let mut report = self.report_mut();
        report.submitted_count += 1;
        report.command_names.push(command_name);
        match &result {
            Ok(events) => {
                report.executed_count += 1;
                report.emitted_event_count += events.len();
            }
            Err(err) => {
                report.failed_count += 1;
                report.errors.push(err.to_string());
            }
        }
        result
    }

    pub fn dispatch_all(
        &self,
        commands: impl IntoIterator<Item = Command>,
    ) -> Vec<ZsuiResult<Vec<AppEvent>>> {
        commands
            .into_iter()
            .map(|command| self.dispatch(command))
            .collect()
    }

    pub fn report(&self) -> AppCommandDispatchReport {
        self.report_mut().clone()
    }

    fn executor(&self) -> MutexGuard<'_, Box<dyn AppCommandExecutor>> {
        self.inner
            .executor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn report_mut(&self) -> MutexGuard<'_, AppCommandDispatchReport> {
        self.inner
            .report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for SharedAppCommandExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedAppCommandExecutor")
            .field("report", &self.report())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SharedAppCommandExecutor {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

pub const fn app_command_name(command: &Command) -> &'static str {
    match command {
        Command::ShowMainWindow => "show_main_window",
        Command::HideMainWindow => "hide_main_window",
        Command::ToggleMainWindow => "toggle_main_window",
        Command::OpenQuickPanel => "open_quick_panel",
        Command::OpenSettings => "open_settings",
        Command::CopySelection => "copy_selection",
        Command::PasteSelection => "paste_selection",
        Command::ReadClipboard => "read_clipboard",
        Command::WriteClipboard => "write_clipboard",
        Command::Quit => "quit",
        Command::Custom { .. } => "custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZsuiError;

    #[test]
    fn shared_executor_records_successful_command_handoff() {
        let executor =
            SharedAppCommandExecutor::new(|command| Ok(vec![AppEvent::MenuCommand { command }]));

        let events = executor.dispatch(Command::OpenSettings).unwrap();
        let report = executor.report();

        assert_eq!(events.len(), 1);
        assert_eq!(report.submitted_count, 1);
        assert_eq!(report.executed_count, 1);
        assert_eq!(report.emitted_event_count, 1);
        assert_eq!(report.command_names, vec!["open_settings"]);
    }

    #[test]
    fn shared_executor_records_errors_without_panicking() {
        let executor = SharedAppCommandExecutor::new(|_| {
            Err(ZsuiError::host("app_command", "rejected by product"))
        });

        assert!(executor.dispatch(Command::CopySelection).is_err());
        let report = executor.report();
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.errors.len(), 1);
    }
}
