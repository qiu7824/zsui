//! Control-specific event builders. Configure behavior before converting to
//! the common layout element with `into()`.
use super::Element;

/// A push button that can emit a message when activated.
#[cfg(feature = "button")]
pub struct Button<M>(Element<M>);

/// Creates a typed push button.
#[cfg(feature = "button")]
pub fn button<M>(label: impl Into<String>) -> Button<M> {
    Button(super::button(label))
}

#[cfg(feature = "button")]
impl<M: Clone> Button<M> {
    /// Sets the activation message.
    pub fn on_click(self, message: M) -> Self {
        Self(self.0.on_click(message))
    }
    /// Enables or disables focus and activation.
    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }
}

/// A read-only label.
///
/// ```compile_fail,E0599
/// use zsui::stable::typed;
/// let _ = typed::text::<()>("Save").on_click(());
/// ```
#[cfg(feature = "label")]
pub struct Label<M>(Element<M>);

/// Creates a typed read-only label.
#[cfg(feature = "label")]
pub fn text<M>(value: impl Into<String>) -> Label<M> {
    Label(super::text(value))
}

/// An editor that emits its new string value.
#[cfg(feature = "textbox")]
pub struct TextBox<M>(Element<M>);

/// Creates a typed single-line editor.
#[cfg(feature = "textbox")]
pub fn textbox<M>(value: impl Into<String>) -> TextBox<M> {
    TextBox(super::textbox(value))
}

#[cfg(feature = "textbox")]
impl<M: Clone> TextBox<M> {
    /// Maps a new value with an allocation-free function pointer.
    pub fn on_change(self, map: fn(String) -> M) -> Self {
        Self(self.0.on_change(map))
    }
    /// Maps a new value with captured application data.
    pub fn on_change_with(self, map: impl Fn(String) -> M + Send + Sync + 'static) -> Self {
        Self(Element::from_raw(self.0.into_raw().on_change_with(map)))
    }
}

/// A switch that emits its new Boolean value.
#[cfg(feature = "toggle")]
pub struct Toggle<M>(Element<M>);

/// Creates a typed toggle.
#[cfg(feature = "toggle")]
pub fn toggle<M>(checked: bool) -> Toggle<M> {
    Toggle(super::toggle(checked))
}

#[cfg(feature = "toggle")]
impl<M: Clone> Toggle<M> {
    /// Maps the new checked state with an allocation-free function pointer.
    pub fn on_toggle(self, map: fn(bool) -> M) -> Self {
        Self(self.0.on_toggle(map))
    }
    /// Maps the new checked state with captured application data.
    pub fn on_toggle_with(self, map: impl Fn(bool) -> M + Send + Sync + 'static) -> Self {
        Self(Element::from_raw(self.0.into_raw().on_toggle_with(map)))
    }
}

macro_rules! into_element {
    ($name:ident) => {
        impl<M> From<$name<M>> for Element<M> {
            fn from(control: $name<M>) -> Self {
                control.0
            }
        }
    };
}
#[cfg(feature = "button")]
into_element!(Button);
#[cfg(feature = "label")]
into_element!(Label);
#[cfg(feature = "textbox")]
into_element!(TextBox);
#[cfg(feature = "toggle")]
into_element!(Toggle);

#[cfg(all(test, feature = "button", feature = "textbox", feature = "toggle"))]
mod tests {
    use super::*;
    use crate::{View, ViewEvent, ViewEventCx, WidgetId};

    #[test]
    fn typed_controls_deliver_only_their_bound_messages() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Save,
            Name(u32, String),
            Active(bool),
        }
        let record = 7;
        let save: Element<_> = button("Save").on_click(Msg::Save).into();
        let name: Element<_> = textbox("")
            .on_change_with(move |s| Msg::Name(record, s))
            .into();
        let active: Element<_> = toggle(false).on_toggle(Msg::Active).into();
        let mut form = crate::column([
            save.into_raw().id(WidgetId(10)),
            name.into_raw().id(WidgetId(11)),
            active.into_raw().id(WidgetId(12)),
        ]);
        let mut events = ViewEventCx::new();
        for event in [
            ViewEvent::Click {
                widget: WidgetId(10),
            },
            ViewEvent::TextChanged {
                widget: WidgetId(11),
                value: "Ada".into(),
            },
            ViewEvent::Toggled {
                widget: WidgetId(12),
                checked: true,
            },
        ] {
            form.event(&mut events, &event);
        }
        assert_eq!(
            events.into_messages(),
            [
                Msg::Save,
                Msg::Name(record, "Ada".into()),
                Msg::Active(true)
            ]
        );
    }
}
