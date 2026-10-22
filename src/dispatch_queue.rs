//! Bounded admission for application actions. Accepted actions retain FIFO order.
use std::{collections::VecDeque, fmt};

/// Default number of outstanding actions in each window queue.
pub const DEFAULT_DISPATCH_QUEUE_CAPACITY: usize = 4096;
/// Maximum actions of one kind dispatched in one event-loop turn.
pub const DISPATCH_BATCH_SIZE: usize = 32;

/// Admission failed; the action was not accepted and may be retried later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueFull {
    /// Queue that exhausted its remaining capacity.
    pub queue: &'static str,
    /// Remaining admission allowance for this update or queue.
    pub capacity: usize,
}
impl fmt::Display for QueueFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} queue is full (allowance {})",
            self.queue, self.capacity
        )
    }
}
impl std::error::Error for QueueFull {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DispatchAllowance {
    pub commands: usize,
    pub ui_commands: usize,
    pub effects: usize,
}
impl Default for DispatchAllowance {
    fn default() -> Self {
        Self {
            commands: DEFAULT_DISPATCH_QUEUE_CAPACITY,
            ui_commands: DEFAULT_DISPATCH_QUEUE_CAPACITY,
            effects: DEFAULT_DISPATCH_QUEUE_CAPACITY,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DeferredQueue<T> {
    pending: VecDeque<T>,
    rejected: usize,
}
impl<T> Default for DeferredQueue<T> {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            rejected: 0,
        }
    }
}
impl<T> DeferredQueue<T> {
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn remaining(&self) -> usize {
        DEFAULT_DISPATCH_QUEUE_CAPACITY.saturating_sub(self.len())
    }
    pub fn push(&mut self, value: T) {
        if self.remaining() == 0 {
            self.rejected = self.rejected.saturating_add(1);
        } else {
            self.pending.push_back(value);
        }
    }
    pub fn extend(&mut self, values: impl IntoIterator<Item = T>) {
        for value in values {
            self.push(value);
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        self.pending.pop_front()
    }
    pub fn take_batch(&mut self) -> Vec<T> {
        self.pending
            .drain(..self.len().min(DISPATCH_BATCH_SIZE))
            .collect()
    }
    pub fn take_rejected(&mut self) -> usize {
        std::mem::take(&mut self.rejected)
    }
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

/// A turn checks its deadline before removing the next request. Modal calls
/// already in progress cannot be preempted; the remainder stays in the queue.
#[allow(dead_code)] // Used by the optional native desktop hosts.
pub(crate) struct DispatchTurnBudget {
    start: std::time::Instant,
    count: usize,
}
#[allow(dead_code)]
impl DispatchTurnBudget {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            count: 0,
        }
    }
    pub fn take(&mut self) -> bool {
        if self.count >= DISPATCH_BATCH_SIZE
            || self.start.elapsed() >= std::time::Duration::from_millis(4)
        {
            return false;
        }
        self.count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deferred_dispatch_deadline_expires_without_popping_a_request() {
        let mut queue = DeferredQueue::default();
        queue.push(1);
        let mut budget = DispatchTurnBudget {
            start: std::time::Instant::now() - std::time::Duration::from_millis(10),
            count: 0,
        };
        assert!(!budget.take());
        assert_eq!(queue.pop(), Some(1));
    }
    #[test]
    fn bounded_command_queue_reports_admission_failure_without_changing_fifo() {
        let mut queue = crate::CommandQueue::with_capacity(2);
        let command = |id| crate::UiCommand::window(crate::CommandId(id));
        assert!(queue.try_push(command("first")).is_ok());
        queue.push(command("second"));
        assert!(queue.try_push(command("third")).is_err());
        assert_eq!(queue.rejected_count(), 1);
        assert_eq!(queue.pop().unwrap().id, crate::CommandId("first"));
        assert_eq!(queue.pop().unwrap().id, crate::CommandId("second"));
        assert!(queue.pop().is_none());
    }
}
