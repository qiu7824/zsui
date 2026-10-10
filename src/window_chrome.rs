//! Application-drawn window chrome.
//!
//! A window built with `custom_title_bar(true)` hands its caption area to the
//! View. Nodes marked with `ViewNode::window_drag_region` move the window
//! where no interactive target covers them, and caption buttons raise
//! [`ZsWindowCommand`]s through `AppCx::window_command`.

use serde::{Deserialize, Serialize};

/// A window-management request raised from a View update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsWindowCommand {
    /// Minimizes the window to the taskbar or dock.
    Minimize,
    /// Maximizes a restored window and restores a maximized one.
    ToggleMaximize,
    /// Requests closing the window through the normal close path, so close
    /// requests can still be vetoed by the application.
    Close,
}
