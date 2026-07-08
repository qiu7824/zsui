use crate::{Point, UiRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsComponentKind {
    Label,
    TextInput,
    Toggle,
    Dropdown,
    Button,
    AccentButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeControlFamily {
    StaticText,
    TextInput,
    Action,
}

impl SettingsComponentKind {
    pub const fn family(self) -> NativeControlFamily {
        match self {
            SettingsComponentKind::Label => NativeControlFamily::StaticText,
            SettingsComponentKind::TextInput => NativeControlFamily::TextInput,
            SettingsComponentKind::Toggle
            | SettingsComponentKind::Dropdown
            | SettingsComponentKind::Button
            | SettingsComponentKind::AccentButton => NativeControlFamily::Action,
        }
    }

    pub const fn is_action(self) -> bool {
        matches!(self.family(), NativeControlFamily::Action)
    }
}

pub trait NativeControlMapper {
    type ClassName: Copy + Eq;

    fn class_name(&self, kind: SettingsComponentKind) -> Self::ClassName;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeControlMapperOperation {
    ClassName,
}

impl NativeControlMapperOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::ClassName => "class_name",
        }
    }
}

pub const REQUIRED_NATIVE_CONTROL_MAPPER_OPERATIONS: [NativeControlMapperOperation; 1] =
    [NativeControlMapperOperation::ClassName];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsControlSpec {
    pub id: Option<i64>,
    pub text: String,
    pub kind: SettingsComponentKind,
    pub bounds: UiRect,
}

impl SettingsControlSpec {
    pub fn new(
        kind: SettingsComponentKind,
        id: Option<i64>,
        text: impl Into<String>,
        bounds: UiRect,
    ) -> Self {
        Self {
            id,
            text: text.into(),
            kind,
            bounds,
        }
    }

    pub fn action(
        kind: SettingsComponentKind,
        id: i64,
        text: impl Into<String>,
        bounds: UiRect,
    ) -> Self {
        debug_assert!(kind.is_action());
        Self::new(kind, Some(id), text, bounds)
    }

    pub fn label(text: impl Into<String>, bounds: UiRect) -> Self {
        Self::new(SettingsComponentKind::Label, None, text, bounds)
    }

    pub fn text_input(id: i64, text: impl Into<String>, bounds: UiRect) -> Self {
        Self::new(SettingsComponentKind::TextInput, Some(id), text, bounds)
    }

    pub const fn width(&self) -> i32 {
        self.bounds.right - self.bounds.left
    }

    pub const fn height(&self) -> i32 {
        self.bounds.bottom - self.bounds.top
    }
}

pub trait NativeSettingsControlHost {
    type Handle: Copy + Eq;

    fn create_control(&mut self, spec: &SettingsControlSpec) -> Self::Handle;
    fn destroy_control(&mut self, handle: Self::Handle);
    fn control_exists(&self, handle: Self::Handle) -> bool;
    fn set_control_visible(&mut self, handle: Self::Handle, visible: bool);
    fn set_control_enabled(&mut self, handle: Self::Handle, enabled: bool);
    fn set_control_bounds(&mut self, handle: Self::Handle, bounds: UiRect);
    fn control_at_point(&self, point: Point) -> Option<Self::Handle>;
    fn control_screen_bounds(&self, handle: Self::Handle) -> Option<UiRect>;
    fn control_text(&self, handle: Self::Handle) -> String;
    fn set_control_text(&mut self, handle: Self::Handle, text: &str);
    fn request_control_repaint(&mut self, handle: Self::Handle) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControlHostOperation {
    CreateControl,
    DestroyControl,
    ControlExists,
    SetControlVisible,
    SetControlEnabled,
    SetControlBounds,
    ControlAtPoint,
    ControlScreenBounds,
    ControlText,
    SetControlText,
    RequestControlRepaint,
}

impl SettingsControlHostOperation {
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::CreateControl => "create_control",
            Self::DestroyControl => "destroy_control",
            Self::ControlExists => "control_exists",
            Self::SetControlVisible => "set_control_visible",
            Self::SetControlEnabled => "set_control_enabled",
            Self::SetControlBounds => "set_control_bounds",
            Self::ControlAtPoint => "control_at_point",
            Self::ControlScreenBounds => "control_screen_bounds",
            Self::ControlText => "control_text",
            Self::SetControlText => "set_control_text",
            Self::RequestControlRepaint => "request_control_repaint",
        }
    }
}

pub const REQUIRED_SETTINGS_CONTROL_HOST_OPERATIONS: [SettingsControlHostOperation; 11] = [
    SettingsControlHostOperation::CreateControl,
    SettingsControlHostOperation::DestroyControl,
    SettingsControlHostOperation::ControlExists,
    SettingsControlHostOperation::SetControlVisible,
    SettingsControlHostOperation::SetControlEnabled,
    SettingsControlHostOperation::SetControlBounds,
    SettingsControlHostOperation::ControlAtPoint,
    SettingsControlHostOperation::ControlScreenBounds,
    SettingsControlHostOperation::ControlText,
    SettingsControlHostOperation::SetControlText,
    SettingsControlHostOperation::RequestControlRepaint,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_control_spec_keeps_geometry_and_intent() {
        let spec = SettingsControlSpec::action(
            SettingsComponentKind::AccentButton,
            7,
            "Save",
            UiRect::new(10, 20, 90, 52),
        );

        assert_eq!(spec.kind.family(), NativeControlFamily::Action);
        assert_eq!(spec.width(), 80);
        assert_eq!(spec.height(), 32);
        assert_eq!(
            REQUIRED_SETTINGS_CONTROL_HOST_OPERATIONS[0].operation_name(),
            "create_control"
        );
    }
}
