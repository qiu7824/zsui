use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
};

use serde::Serialize;

use crate::{geometry::ComponentId, AppEvent, ZsuiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandScope {
    App,
    Window,
    Component(ComponentId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPayload {
    None,
    ControlId(i64),
    Text(String),
    ItemId(i64),
    Paths(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCommand {
    pub id: CommandId,
    pub scope: CommandScope,
    pub payload: CommandPayload,
}

impl UiCommand {
    pub const fn app(id: CommandId) -> Self {
        Self {
            id,
            scope: CommandScope::App,
            payload: CommandPayload::None,
        }
    }

    pub const fn window(id: CommandId) -> Self {
        Self {
            id,
            scope: CommandScope::Window,
            payload: CommandPayload::None,
        }
    }

    pub const fn window_with_payload(id: CommandId, payload: CommandPayload) -> Self {
        Self {
            id,
            scope: CommandScope::Window,
            payload,
        }
    }
}

pub trait UiCommandExecutor: Send {
    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>>;
}

impl<F> UiCommandExecutor for F
where
    F: FnMut(UiCommand) -> ZsuiResult<Vec<AppEvent>> + Send,
{
    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        self(command)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UiCommandDispatchReport {
    pub submitted_count: usize,
    pub executed_count: usize,
    pub failed_count: usize,
    pub emitted_event_count: usize,
    pub command_ids: Vec<&'static str>,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct SharedUiCommandExecutor {
    inner: Arc<SharedUiCommandExecutorInner>,
}

struct SharedUiCommandExecutorInner {
    executor: Mutex<Box<dyn UiCommandExecutor>>,
    report: Mutex<UiCommandDispatchReport>,
}

impl SharedUiCommandExecutor {
    pub fn new(executor: impl UiCommandExecutor + 'static) -> Self {
        Self {
            inner: Arc::new(SharedUiCommandExecutorInner {
                executor: Mutex::new(Box::new(executor)),
                report: Mutex::new(UiCommandDispatchReport::default()),
            }),
        }
    }

    pub fn dispatch(&self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        let command_id = command.id.0;
        let result = self.executor().execute_ui_command(command);
        let mut report = self.report_mut();
        report.submitted_count += 1;
        report.command_ids.push(command_id);
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
        commands: impl IntoIterator<Item = UiCommand>,
    ) -> Vec<ZsuiResult<Vec<AppEvent>>> {
        commands
            .into_iter()
            .map(|command| self.dispatch(command))
            .collect()
    }

    pub fn report(&self) -> UiCommandDispatchReport {
        self.report_mut().clone()
    }

    fn executor(&self) -> MutexGuard<'_, Box<dyn UiCommandExecutor>> {
        self.inner
            .executor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn report_mut(&self) -> MutexGuard<'_, UiCommandDispatchReport> {
        self.inner
            .report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for SharedUiCommandExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedUiCommandExecutor")
            .field("report", &self.report())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SharedUiCommandExecutor {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandQueue {
    pending: VecDeque<UiCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, command: UiCommand) {
        self.pending.push_back(command);
    }

    pub fn pop(&mut self) -> Option<UiCommand> {
        self.pending.pop_front()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_queue_preserves_order() {
        let mut queue = CommandQueue::default();
        queue.push(UiCommand::window(CommandId("first")));
        queue.push(UiCommand::window(CommandId("second")));

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop().unwrap().id, CommandId("first"));
        assert_eq!(queue.pop().unwrap().id, CommandId("second"));
        assert!(queue.is_empty());
    }

    #[test]
    fn shared_ui_executor_records_product_handoff() {
        let executor = SharedUiCommandExecutor::new(|command: UiCommand| {
            Ok(vec![AppEvent::Custom {
                id: command.id.0.to_string(),
                payload: None,
            }])
        });

        let events = executor
            .dispatch(UiCommand::app(CommandId("settings.save")))
            .unwrap();
        let report = executor.report();

        assert_eq!(events.len(), 1);
        assert_eq!(report.executed_count, 1);
        assert_eq!(report.emitted_event_count, 1);
        assert_eq!(report.command_ids, vec!["settings.save"]);
    }
}
