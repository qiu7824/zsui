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
use crate::{Command, DesktopEvent, MenuItemSpec, MenuSpec, WindowId, ZsAccelerator};
#[cfg(any(
    test,
    target_os = "macos",
    all(target_os = "linux", not(target_env = "ohos"))
))]
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
        accelerator: Option<ZsAccelerator>,
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
                            .map(|accelerator| {
                                accelerator.validate()?;
                                Ok(accelerator)
                            })
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
    fn invalid_typed_accelerators_fail_before_reaching_a_backend() {
        let menu = MenuSpec {
            id: None,
            title: None,
            items: vec![MenuItemSpec::command("Invalid", Command::Quit)
                .accelerator(ZsAccelerator::new(crate::ZsAcceleratorKey::Function(25)))],
        };

        assert!(NativeMenuModel::lower(&menu, 1).is_err());
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
