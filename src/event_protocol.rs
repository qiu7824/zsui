use crate::{
    command_protocol::UiCommand,
    geometry::{Point, Size},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPhase {
    New,
    Mounted,
    Active,
    Suspended,
    Unmounted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    Mount,
    Resume,
    Suspend,
    Unmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleState {
    phase: ComponentPhase,
}

impl LifecycleState {
    pub const fn new() -> Self {
        Self {
            phase: ComponentPhase::New,
        }
    }

    pub const fn phase(self) -> ComponentPhase {
        self.phase
    }

    pub fn apply(&mut self, event: LifecycleEvent) -> bool {
        let next = match (self.phase, event) {
            (ComponentPhase::New, LifecycleEvent::Mount) => ComponentPhase::Mounted,
            (ComponentPhase::Mounted | ComponentPhase::Suspended, LifecycleEvent::Resume) => {
                ComponentPhase::Active
            }
            (ComponentPhase::Active, LifecycleEvent::Suspend) => ComponentPhase::Suspended,
            (
                ComponentPhase::Mounted | ComponentPhase::Active | ComponentPhase::Suspended,
                LifecycleEvent::Unmount,
            ) => ComponentPhase::Unmounted,
            _ => return false,
        };
        self.phase = next;
        true
    }
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Down,
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent<AppEvent = crate::AppEvent> {
    Application(AppEvent),
    Lifecycle(LifecycleEvent),
    PointerMove {
        position: Point,
    },
    PointerHover {
        position: Point,
    },
    PointerLeave,
    PointerCancel,
    PointerButton {
        position: Point,
        button: MouseButton,
        pressed: bool,
        click_count: u8,
    },
    MouseWheel {
        delta: i32,
    },
    Key {
        code: u32,
        state: KeyState,
        system: bool,
    },
    TextInput(String),
    Command(UiCommand),
    ControlCommand {
        control_id: u32,
        notification: u16,
    },
    ControlSelectionChanged {
        control_id: u32,
        index: usize,
    },
    GlobalHotkey {
        id: i32,
    },
    ClipboardChanged,
    Timer {
        id: u64,
    },
    WindowSize {
        size: Size,
        minimized: bool,
    },
    AppActivationChanged {
        active: bool,
    },
    SystemMetricsChanged,
    WindowMoved,
    WindowMoveCompleted,
    CloseRequested,
    ThemeChanged,
    DpiChanged {
        dpi: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_rejects_out_of_order_transitions() {
        let mut lifecycle = LifecycleState::new();
        assert!(!lifecycle.apply(LifecycleEvent::Resume));
        assert!(lifecycle.apply(LifecycleEvent::Mount));
        assert!(lifecycle.apply(LifecycleEvent::Resume));
        assert_eq!(lifecycle.phase(), ComponentPhase::Active);
        assert!(lifecycle.apply(LifecycleEvent::Suspend));
        assert!(lifecycle.apply(LifecycleEvent::Unmount));
        assert!(!lifecycle.apply(LifecycleEvent::Resume));
    }
}
