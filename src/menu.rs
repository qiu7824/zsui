use std::fmt;

use crate::core::{Command, ZsuiError, ZsuiResult};
use serde::{Deserialize, Serialize};

/// A platform-neutral key that can participate in a native menu accelerator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsAcceleratorKey {
    Character(char),
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Function(u8),
}

impl ZsAcceleratorKey {
    pub fn validate(self) -> ZsuiResult<()> {
        match self {
            Self::Character(key) if key.is_ascii_alphanumeric() => Ok(()),
            Self::Character(key) => Err(ZsuiError::invalid_spec(
                "menu.accelerator.key",
                format!("character accelerator `{key}` must be ASCII alphanumeric"),
            )),
            Self::Function(number) if (1..=24).contains(&number) => Ok(()),
            Self::Function(number) => Err(ZsuiError::invalid_spec(
                "menu.accelerator.key",
                format!("function key F{number} is outside F1-F24"),
            )),
            _ => Ok(()),
        }
    }

    pub(crate) fn label(self) -> String {
        match self {
            Self::Character(key) => key.to_ascii_uppercase().to_string(),
            Self::Enter => "Enter".to_string(),
            Self::Escape => "Escape".to_string(),
            Self::Tab => "Tab".to_string(),
            Self::Space => "Space".to_string(),
            Self::Backspace => "Backspace".to_string(),
            Self::Delete => "Delete".to_string(),
            Self::Up => "Up".to_string(),
            Self::Down => "Down".to_string(),
            Self::Left => "Left".to_string(),
            Self::Right => "Right".to_string(),
            Self::Home => "Home".to_string(),
            Self::End => "End".to_string(),
            Self::PageUp => "PageUp".to_string(),
            Self::PageDown => "PageDown".to_string(),
            Self::Function(number) => format!("F{number}"),
        }
    }
}

/// A typed native menu accelerator.
///
/// `primary` maps to Control on Windows and Linux and Command on macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsAccelerator {
    key: ZsAcceleratorKey,
    primary: bool,
    shift: bool,
    alt: bool,
    super_key: bool,
}

impl ZsAccelerator {
    pub const fn new(key: ZsAcceleratorKey) -> Self {
        Self {
            key,
            primary: false,
            shift: false,
            alt: false,
            super_key: false,
        }
    }

    pub const fn primary(key: ZsAcceleratorKey) -> Self {
        Self {
            primary: true,
            ..Self::new(key)
        }
    }

    pub fn primary_character(key: char) -> Self {
        Self::primary(ZsAcceleratorKey::Character(key.to_ascii_uppercase()))
    }

    pub const fn shifted(mut self) -> Self {
        self.shift = true;
        self
    }

    pub const fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub const fn with_super(mut self) -> Self {
        self.super_key = true;
        self
    }

    pub const fn key(self) -> ZsAcceleratorKey {
        self.key
    }

    pub const fn uses_primary(self) -> bool {
        self.primary
    }

    pub const fn uses_shift(self) -> bool {
        self.shift
    }

    pub const fn uses_alt(self) -> bool {
        self.alt
    }

    pub const fn uses_super(self) -> bool {
        self.super_key
    }

    pub fn validate(self) -> ZsuiResult<()> {
        self.key.validate()
    }
}

impl fmt::Display for ZsAccelerator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.primary {
            formatter.write_str("Primary+")?;
        }
        if self.super_key {
            formatter.write_str("Super+")?;
        }
        if self.alt {
            formatter.write_str("Alt+")?;
        }
        if self.shift {
            formatter.write_str("Shift+")?;
        }
        formatter.write_str(&self.key.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuSpec {
    pub id: Option<String>,
    pub title: Option<String>,
    pub items: Vec<MenuItemSpec>,
}

impl MenuSpec {
    pub fn new() -> Self {
        Self {
            id: None,
            title: None,
            items: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn item(mut self, label: impl Into<String>, command: Command) -> Self {
        self.items.push(MenuItemSpec::command(label, command));
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(MenuItemSpec::Separator);
        self
    }

    pub fn submenu(mut self, label: impl Into<String>, menu: MenuSpec) -> Self {
        self.items.push(MenuItemSpec::Submenu {
            id: None,
            label: label.into(),
            enabled: true,
            menu,
        });
        self
    }
}

impl Default for MenuSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuItemSpec {
    Command {
        id: Option<String>,
        label: String,
        command: Command,
        enabled: bool,
        checked: bool,
        accelerator: Option<ZsAccelerator>,
    },
    Separator,
    Submenu {
        id: Option<String>,
        label: String,
        enabled: bool,
        menu: MenuSpec,
    },
}

impl MenuItemSpec {
    pub fn command(label: impl Into<String>, command: Command) -> Self {
        Self::Command {
            id: None,
            label: label.into(),
            command,
            enabled: true,
            checked: false,
            accelerator: None,
        }
    }

    pub fn disabled(mut self) -> Self {
        if let Self::Command { enabled, .. } | Self::Submenu { enabled, .. } = &mut self {
            *enabled = false;
        }
        self
    }

    pub fn checked(mut self, checked_value: bool) -> Self {
        if let Self::Command { checked, .. } = &mut self {
            *checked = checked_value;
        }
        self
    }

    pub fn accelerator(mut self, value: ZsAccelerator) -> Self {
        if let Self::Command { accelerator, .. } = &mut self {
            *accelerator = Some(value);
        }
        self
    }

    pub fn id(mut self, value: impl Into<String>) -> Self {
        match &mut self {
            Self::Command { id, .. } | Self::Submenu { id, .. } => *id = Some(value.into()),
            Self::Separator => {}
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_accelerator_has_stable_semantic_form() {
        let accelerator = ZsAccelerator::primary_character('o').with_alt().shifted();

        assert_eq!(accelerator.to_string(), "Primary+Alt+Shift+O");
        assert!(accelerator.validate().is_ok());
    }

    #[test]
    fn typed_accelerator_rejects_nonportable_keys() {
        assert!(ZsAccelerator::new(ZsAcceleratorKey::Character('+'))
            .validate()
            .is_err());
        assert!(ZsAccelerator::new(ZsAcceleratorKey::Function(25))
            .validate()
            .is_err());
    }
}
