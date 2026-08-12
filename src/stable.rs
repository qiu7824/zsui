//! Stable application-authoring API for the ZSUI 0.2 release line.
//!
//! Items in this module follow the compatibility policy documented in
//! `docs/api-stability.md`: patch releases in the 0.2 line may add APIs, but
//! do not remove or change existing signatures or serialized meanings.
//!
//! ```no_run
//! # #[cfg(all(feature = "button", feature = "label"))]
//! # fn main() -> Result<(), zsui::stable::Error> {
//! use zsui::stable::{button, column, text, window, Dp, Element, UpdateContext};
//!
//! #[derive(Clone)]
//! enum Message {
//!     Increment,
//! }
//!
//! struct State {
//!     count: u32,
//! }
//!
//! fn view(state: &State) -> Element<Message> {
//!     column([
//!         text(format!("Count: {}", state.count)),
//!         button("Increment").on_click(Message::Increment),
//!     ])
//!     .gap(Dp::new(12.0))
//!     .padding(Dp::new(20.0))
//! }
//!
//! fn update(state: &mut State, message: Message, _cx: &mut UpdateContext<'_>) {
//!     match message {
//!         Message::Increment => state.count += 1,
//!     }
//! }
//!
//! window("Stable ZSUI")
//!     .size(480, 320)
//!     .stateful(State { count: 0 }, view, update)
//!     .run()?;
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "button", feature = "label")))]
//! # fn main() {}
//! ```

#![deny(missing_docs)]

use std::fmt;

/// A platform-independent logical length measured at 96 DPI.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Dp(f32);

impl Dp {
    /// Creates a logical length from its scalar value.
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    /// Returns the scalar logical value.
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl From<Dp> for crate::Dp {
    fn from(value: Dp) -> Self {
        Self::new(value.0)
    }
}

/// A deterministic application-level widget identity.
///
/// IDs are optional because ZSUI assigns structural IDs to interactive nodes.
/// Supply one when state, automation or another component must address the
/// same widget across tree insertion or reordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Creates an explicit widget identity.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric identity chosen by the application.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<WidgetId> for crate::WidgetId {
    fn from(value: WidgetId) -> Self {
        Self::new(value.0)
    }
}

/// An opaque retained UI element that carries messages of type `Message`.
///
/// The platform-specific render, input and accessibility objects remain
/// private. Applications compose elements and return them from one shared
/// `view` function on every desktop platform.
#[derive(Debug, Clone)]
pub struct Element<Message> {
    inner: crate::ViewNode<Message>,
}

impl<Message> Element<Message> {
    fn from_raw(inner: crate::ViewNode<Message>) -> Self {
        Self { inner }
    }

    fn into_raw(self) -> crate::ViewNode<Message> {
        self.inner
    }

    /// Assigns an explicit identity to the element.
    pub fn id(mut self, id: WidgetId) -> Self {
        self.inner = self.inner.id(id.into());
        self
    }

    /// Sets uniform inner padding.
    pub fn padding(mut self, padding: Dp) -> Self {
        self.inner = self.inner.padding(padding.into());
        self
    }

    /// Sets the spacing between children in a row or column.
    pub fn gap(mut self, gap: Dp) -> Self {
        self.inner = self.inner.gap(gap.into());
        self
    }

    /// Sets a fixed logical width.
    pub fn width(mut self, width: Dp) -> Self {
        self.inner = self.inner.width(width.into());
        self
    }

    /// Sets a fixed logical height.
    pub fn height(mut self, height: Dp) -> Self {
        self.inner = self.inner.height(height.into());
        self
    }

    /// Sets the minimum logical width used by intrinsic layout.
    pub fn min_width(mut self, width: Dp) -> Self {
        self.inner = self.inner.min_width(width.into());
        self
    }

    /// Sets the minimum logical height used by intrinsic layout.
    pub fn min_height(mut self, height: Dp) -> Self {
        self.inner = self.inner.min_height(height.into());
        self
    }

    /// Sets the non-negative share of remaining space assigned by its parent.
    pub fn flex(mut self, factor: f32) -> Self {
        self.inner = self.inner.flex(factor);
        self
    }

    /// Enables vertical scrolling when width-aware intrinsic content exceeds
    /// this element's viewport.
    #[cfg(feature = "scroll")]
    pub fn auto_scroll_y(mut self) -> Self {
        self.inner = self.inner.auto_scroll_y();
        self
    }

    /// Sets a semantic accessibility label without exposing platform APIs.
    #[cfg(feature = "accessibility")]
    pub fn accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.accessibility_label(label);
        self
    }

    /// Attaches a native-style tooltip to the element.
    #[cfg(feature = "tooltip")]
    pub fn tooltip(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.tooltip(label);
        self
    }
}

impl<Message: Clone> Element<Message> {
    /// Emits `message` when a button or clickable canvas is activated.
    #[cfg(any(feature = "button", feature = "canvas"))]
    pub fn on_click(mut self, message: Message) -> Self {
        self.inner = self.inner.on_click(message);
        self
    }

    /// Emits a mapped message when editable text changes.
    #[cfg(feature = "textbox")]
    pub fn on_change(mut self, map: fn(String) -> Message) -> Self {
        self.inner = self.inner.on_change(map);
        self
    }

    /// Emits a mapped message when a toggle changes value.
    #[cfg(feature = "toggle")]
    pub fn on_toggle(mut self, map: fn(bool) -> Message) -> Self {
        self.inner = self.inner.on_toggle(map);
        self
    }

    /// Controls whether a button can receive focus and activation.
    #[cfg(feature = "button")]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.enabled(enabled);
        self
    }
}

/// Creates a vertical stack from child elements.
pub fn column<Message>(children: impl IntoIterator<Item = Element<Message>>) -> Element<Message> {
    Element::from_raw(crate::column(children.into_iter().map(Element::into_raw)))
}

/// Creates a horizontal stack from child elements.
pub fn row<Message>(children: impl IntoIterator<Item = Element<Message>>) -> Element<Message> {
    Element::from_raw(crate::row(children.into_iter().map(Element::into_raw)))
}

/// Creates a flexible empty layout element.
pub fn spacer<Message>() -> Element<Message> {
    Element::from_raw(crate::spacer())
}

/// Creates body text resolved through the current platform's native type ramp.
#[cfg(feature = "label")]
pub fn text<Message>(value: impl Into<String>) -> Element<Message> {
    Element::from_raw(crate::text(value))
}

/// Creates a native-style push button.
#[cfg(feature = "button")]
pub fn button<Message>(label: impl Into<String>) -> Element<Message> {
    Element::from_raw(crate::button(label))
}

/// Creates a single-line editable text field.
#[cfg(feature = "textbox")]
pub fn textbox<Message>(value: impl Into<String>) -> Element<Message> {
    Element::from_raw(crate::textbox(value))
}

/// Creates a platform-native toggle surface.
#[cfg(feature = "toggle")]
pub fn toggle<Message>(checked: bool) -> Element<Message> {
    Element::from_raw(crate::toggle(checked))
}

/// Mutable commands available while reducing a typed application message.
pub struct UpdateContext<'a> {
    inner: &'a mut crate::AppCx,
}

impl UpdateContext<'_> {
    /// Requests orderly shutdown after the current update completes.
    pub fn quit(&mut self) {
        self.inner.quit();
    }
}

/// A native window declaration that does not yet contain a view.
#[derive(Debug, Clone)]
pub struct WindowBuilder {
    inner: crate::NativeWindowBuilder,
}

/// Creates a native window declaration.
pub fn window(title: impl Into<String>) -> WindowBuilder {
    WindowBuilder {
        inner: crate::NativeWindowBuilder::new(title),
    }
}

impl WindowBuilder {
    /// Sets the application identity used by desktop services.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.inner = self.inner.app_name(name);
        self
    }

    /// Sets the initial client size in logical pixels.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.inner = self.inner.size(width, height);
        self
    }

    /// Sets the minimum client size in logical pixels.
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.inner = self.inner.min_size(width, height);
        self
    }

    /// Controls whether the user can resize the native window.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.inner = self.inner.resizable(resizable);
        self
    }

    /// Controls whether the platform draws standard window decorations.
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.inner = self.inner.decorations(decorations);
        self
    }

    /// Keeps the native window above ordinary windows when supported.
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.inner = self.inner.always_on_top(always_on_top);
        self
    }

    /// Uses a transparent host surface when supported by the platform.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.inner = self.inner.transparent(transparent);
        self
    }

    /// Loads the platform window icon from an application-owned path.
    pub fn icon_path(mut self, path: impl Into<String>) -> Self {
        self.inner = self.inner.icon_path(path);
        self
    }

    /// Releases the retained view and transient caches while hidden, keeping
    /// application state and command routing alive.
    pub fn release_view_when_hidden(mut self) -> Self {
        self.inner = self.inner.release_view_when_hidden();
        self
    }

    /// Installs a fixed view and makes the window runnable.
    pub fn view<Message: Clone>(mut self, view: Element<Message>) -> RunnableWindow {
        self.inner = self.inner.view(view.into_raw());
        RunnableWindow { inner: self.inner }
    }

    /// Installs the shared typed `State`/`Message`/`view`/`update` loop.
    pub fn stateful<State, Message, ViewFn, UpdateFn>(
        mut self,
        state: State,
        view: ViewFn,
        update: UpdateFn,
    ) -> RunnableWindow
    where
        State: Send + 'static,
        Message: Clone + Send + 'static,
        ViewFn: Fn(&State) -> Element<Message> + Send + 'static,
        UpdateFn: for<'a> Fn(&mut State, Message, &mut UpdateContext<'a>) + Send + 'static,
    {
        self.inner = self.inner.stateful_view(
            state,
            move |state| view(state).into_raw(),
            move |state, message, context| {
                update(state, message, &mut UpdateContext { inner: context });
            },
        );
        RunnableWindow { inner: self.inner }
    }
}

/// A native window with content that can enter the platform event loop.
#[derive(Debug, Clone)]
pub struct RunnableWindow {
    inner: crate::NativeWindowBuilder,
}

impl RunnableWindow {
    /// Creates the native host and runs its platform event loop until exit.
    pub fn run(self) -> Result<(), Error> {
        self.inner.run().map(|_| ()).map_err(Error)
    }
}

/// Error returned by the stable application-authoring API.
#[derive(Debug)]
pub struct Error(crate::ZsuiError);

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(all(test, feature = "button", feature = "label"))]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum Message {
        Increment,
    }

    struct State(u32);

    #[test]
    fn stable_stateful_window_uses_the_shared_live_runtime() {
        let runnable = window("Stable API").size(360, 240).stateful(
            State(0),
            |state| {
                column([
                    text(format!("Count: {}", state.0)),
                    button("Increment").on_click(Message::Increment),
                ])
            },
            |state, message, _context| match message {
                Message::Increment => state.0 += 1,
            },
        );

        assert!(runnable.inner.native_live_view_runtime().is_some());
    }
}
