use std::collections::VecDeque;

use crate::geometry::ComponentId;

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
}
