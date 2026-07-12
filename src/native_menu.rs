#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
use std::collections::BTreeMap;

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
use crate::{Command, DesktopEvent, MenuItemSpec, MenuSpec, WindowId};
use crate::{ZsuiError, ZsuiResult};

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeMenuModel {
    pub title: Option<String>,
    pub items: Vec<NativeMenuNode>,
    commands: BTreeMap<u32, NativeMenuCommandBinding>,
    next_native_id: u32,
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
impl NativeMenuModel {
    pub fn lower(spec: &MenuSpec, first_native_id: u32) -> ZsuiResult<Self> {
        if first_native_id == 0 {
            return Err(ZsuiError::invalid_spec(
                "menu.native_id",
                "native menu command ids must start above zero",
            ));
        }

        let mut lowerer = NativeMenuLowerer {
            next_native_id: first_native_id,
            commands: BTreeMap::new(),
        };
        let items = lowerer.lower_items(&spec.items, true)?;
        Ok(Self {
            title: spec.title.clone(),
            items,
            commands: lowerer.commands,
            next_native_id: lowerer.next_native_id,
        })
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn next_native_id(&self) -> u32 {
        self.next_native_id
    }

    pub fn command_for_native_id(&self, native_id: u32) -> Option<Command> {
        self.commands
            .get(&native_id)
            .filter(|binding| binding.enabled)
            .map(|binding| binding.command.clone())
    }
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
pub(crate) fn native_menu_command_event(
    model: Option<&NativeMenuModel>,
    window: Option<WindowId>,
    native_id: u32,
) -> Option<DesktopEvent> {
    Some(DesktopEvent::MenuCommand {
        window: window?,
        command: model?.command_for_native_id(native_id)?,
    })
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NativeMenuNode {
    Command {
        native_id: u32,
        label: String,
        enabled: bool,
        checked: bool,
        accelerator: Option<NativeMenuAccelerator>,
    },
    Separator,
    Submenu {
        label: String,
        enabled: bool,
        items: Vec<NativeMenuNode>,
    },
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeMenuCommandBinding {
    command: Command,
    enabled: bool,
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
struct NativeMenuLowerer {
    next_native_id: u32,
    commands: BTreeMap<u32, NativeMenuCommandBinding>,
}

#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
impl NativeMenuLowerer {
    fn lower_items(
        &mut self,
        items: &[MenuItemSpec],
        ancestors_enabled: bool,
    ) -> ZsuiResult<Vec<NativeMenuNode>> {
        items
            .iter()
            .map(|item| match item {
                MenuItemSpec::Command {
                    label,
                    command,
                    enabled,
                    checked,
                    accelerator,
                    ..
                } => {
                    let enabled = ancestors_enabled && *enabled;
                    let native_id = self.allocate_native_id()?;
                    self.commands.insert(
                        native_id,
                        NativeMenuCommandBinding {
                            command: command.clone(),
                            enabled,
                        },
                    );
                    Ok(NativeMenuNode::Command {
                        native_id,
                        label: label.clone(),
                        enabled,
                        checked: *checked,
                        accelerator: accelerator
                            .as_deref()
                            .map(NativeMenuAccelerator::parse)
                            .transpose()?,
                    })
                }
                MenuItemSpec::Separator => Ok(NativeMenuNode::Separator),
                MenuItemSpec::Submenu {
                    label,
                    enabled,
                    menu,
                    ..
                } => {
                    let enabled = ancestors_enabled && *enabled;
                    Ok(NativeMenuNode::Submenu {
                        label: label.clone(),
                        enabled,
                        items: self.lower_items(&menu.items, enabled)?,
                    })
                }
            })
            .collect()
    }

    fn allocate_native_id(&mut self) -> ZsuiResult<u32> {
        let native_id = self.next_native_id;
        self.next_native_id = self.next_native_id.checked_add(1).ok_or_else(|| {
            ZsuiError::invalid_spec(
                "menu.native_id",
                "native menu command id range is exhausted",
            )
        })?;
        Ok(native_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeMenuAccelerator {
    pub key: String,
    pub primary: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl NativeMenuAccelerator {
    pub fn parse(value: &str) -> ZsuiResult<Self> {
        let mut accelerator = Self {
            key: String::new(),
            primary: false,
            shift: false,
            alt: false,
            super_key: false,
        };

        for part in value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match part.to_ascii_lowercase().as_str() {
                "primary" | "ctrl" | "control" => accelerator.primary = true,
                "shift" => accelerator.shift = true,
                "alt" | "option" => accelerator.alt = true,
                "cmd" | "command" | "meta" | "super" | "win" => accelerator.super_key = true,
                _ if accelerator.key.is_empty() => {
                    accelerator.key = normalize_accelerator_key(part)?;
                }
                _ => {
                    return Err(ZsuiError::invalid_spec(
                        "menu.accelerator",
                        format!("accelerator `{value}` contains more than one key"),
                    ));
                }
            }
        }

        if accelerator.key.is_empty() {
            return Err(ZsuiError::invalid_spec(
                "menu.accelerator",
                format!("accelerator `{value}` does not contain a key"),
            ));
        }
        Ok(accelerator)
    }

    #[cfg(any(test, all(target_os = "linux", not(target_env = "ohos"))))]
    pub fn gtk_accelerator(&self) -> String {
        let mut value = String::new();
        if self.primary {
            value.push_str("<Primary>");
        }
        if self.super_key {
            value.push_str("<Super>");
        }
        if self.alt {
            value.push_str("<Alt>");
        }
        if self.shift {
            value.push_str("<Shift>");
        }
        value.push_str(match self.key.as_str() {
            "Enter" => "Return",
            "Space" => "space",
            "Backspace" => "BackSpace",
            "PageUp" => "Page_Up",
            "PageDown" => "Page_Down",
            key => key,
        });
        value
    }

    #[cfg(any(test, target_os = "macos"))]
    pub fn appkit_key_equivalent(&self) -> Option<String> {
        let value = match self.key.as_str() {
            "Enter" | "Return" => "\r".to_string(),
            "Tab" => "\t".to_string(),
            "Escape" => "\u{1b}".to_string(),
            "Space" => " ".to_string(),
            "Backspace" => "\u{8}".to_string(),
            "Delete" => "\u{7f}".to_string(),
            "Up" => "\u{f700}".to_string(),
            "Down" => "\u{f701}".to_string(),
            "Left" => "\u{f702}".to_string(),
            "Right" => "\u{f703}".to_string(),
            "Home" => "\u{f729}".to_string(),
            "End" => "\u{f72b}".to_string(),
            "PageUp" => "\u{f72c}".to_string(),
            "PageDown" => "\u{f72d}".to_string(),
            key if key.len() == 1 => key.to_ascii_lowercase(),
            key if key.starts_with('F') => {
                let number = key[1..].parse::<u32>().ok()?;
                char::from_u32(0xf703 + number)?.to_string()
            }
            _ => return None,
        };
        Some(value)
    }
}

fn normalize_accelerator_key(key: &str) -> ZsuiResult<String> {
    if key.chars().count() == 1 {
        return Ok(key.to_ascii_uppercase());
    }

    let normalized = match key.to_ascii_lowercase().as_str() {
        "enter" => "Enter",
        "return" => "Return",
        "tab" => "Tab",
        "esc" | "escape" => "Escape",
        "space" => "Space",
        "backspace" => "Backspace",
        "delete" | "del" => "Delete",
        "up" => "Up",
        "down" => "Down",
        "left" => "Left",
        "right" => "Right",
        "home" => "Home",
        "end" => "End",
        "pageup" | "page_up" => "PageUp",
        "pagedown" | "page_down" => "PageDown",
        other if other.len() >= 2 && other.starts_with('f') => {
            let number = other[1..].parse::<u8>().map_err(|_| {
                ZsuiError::invalid_spec(
                    "menu.accelerator",
                    format!("unsupported accelerator key `{key}`"),
                )
            })?;
            if !(1..=24).contains(&number) {
                return Err(ZsuiError::invalid_spec(
                    "menu.accelerator",
                    format!("function key `{key}` is outside F1-F24"),
                ));
            }
            return Ok(format!("F{number}"));
        }
        _ => {
            return Err(ZsuiError::invalid_spec(
                "menu.accelerator",
                format!("unsupported accelerator key `{key}`"),
            ));
        }
    };
    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowering_preserves_nested_state_and_routes_only_enabled_commands() {
        let mut file_menu = MenuSpec::new()
            .item("Open", Command::custom("file.open"))
            .separator()
            .submenu(
                "Recent",
                MenuSpec::new().item("One", Command::custom("file.recent.one")),
            );
        file_menu
            .items
            .push(MenuItemSpec::command("Disabled", Command::custom("file.disabled")).disabled());
        let menu = MenuSpec::new()
            .title("Application")
            .submenu("File", file_menu);

        let model = NativeMenuModel::lower(&menu, 41).expect("menu should lower");

        assert_eq!(model.title.as_deref(), Some("Application"));
        assert_eq!(model.command_count(), 3);
        assert_eq!(model.next_native_id(), 44);
        assert_eq!(
            model.command_for_native_id(41),
            Some(Command::custom("file.open"))
        );
        assert_eq!(
            model.command_for_native_id(42),
            Some(Command::custom("file.recent.one"))
        );
        assert_eq!(model.command_for_native_id(43), None);
        assert!(matches!(
            &model.items[0],
            NativeMenuNode::Submenu { items, .. }
                if matches!(items[1], NativeMenuNode::Separator)
        ));
    }

    #[test]
    fn accelerator_is_normalized_for_gtk_and_appkit() {
        let accelerator =
            NativeMenuAccelerator::parse("Ctrl+Alt+Shift+O").expect("valid accelerator");

        assert_eq!(accelerator.gtk_accelerator(), "<Primary><Alt><Shift>O");
        assert_eq!(accelerator.appkit_key_equivalent().as_deref(), Some("o"));
        assert!(accelerator.primary);
        assert!(accelerator.alt);
        assert!(accelerator.shift);

        let f12 = NativeMenuAccelerator::parse("Cmd+F12").expect("function key");
        assert_eq!(f12.appkit_key_equivalent(), Some("\u{f70f}".to_string()));
    }

    #[test]
    fn invalid_accelerators_fail_before_reaching_a_backend() {
        assert!(NativeMenuAccelerator::parse("Ctrl+Shift").is_err());
        assert!(NativeMenuAccelerator::parse("Ctrl+O+P").is_err());
        assert!(NativeMenuAccelerator::parse("Ctrl+F25").is_err());
        assert!(NativeMenuModel::lower(&MenuSpec::new(), 0).is_err());
    }

    #[test]
    fn command_event_requires_current_window_model_and_enabled_binding() {
        let mut menu = MenuSpec::new().item("Open", Command::custom("file.open"));
        menu.items
            .push(MenuItemSpec::command("Disabled", Command::Quit).disabled());
        let model = NativeMenuModel::lower(&menu, 90).expect("menu model");

        assert_eq!(
            native_menu_command_event(Some(&model), Some(WindowId(7)), 90),
            Some(DesktopEvent::MenuCommand {
                window: WindowId(7),
                command: Command::custom("file.open"),
            })
        );
        assert_eq!(
            native_menu_command_event(Some(&model), Some(WindowId(7)), 91),
            None
        );
        assert_eq!(native_menu_command_event(Some(&model), None, 90), None);
        assert_eq!(native_menu_command_event(None, Some(WindowId(7)), 90), None);
    }
}
