use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;

use crate::native_menu::{native_menu_command_event, NativeMenuModel, NativeMenuNode};
use crate::{DesktopEvent, MenuService, MenuSpec, WindowId, ZsuiError, ZsuiResult};

pub struct LinuxGtkMenuService {
    application: gtk::Application,
    window: Option<WindowId>,
    model: Option<NativeMenuModel>,
    menu: Option<gio::Menu>,
    pending_commands: Rc<RefCell<VecDeque<u32>>>,
    command_handler: Rc<RefCell<Option<Rc<dyn Fn(u32)>>>>,
    action_names: Vec<String>,
    next_native_id: u32,
}

impl fmt::Debug for LinuxGtkMenuService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinuxGtkMenuService")
            .field("window", &self.window)
            .field(
                "command_count",
                &self
                    .model
                    .as_ref()
                    .map(NativeMenuModel::command_count)
                    .unwrap_or(0),
            )
            .finish_non_exhaustive()
    }
}

impl LinuxGtkMenuService {
    pub fn for_current_application() -> ZsuiResult<Self> {
        crate::linux_gtk_services::ensure_gtk_main_thread("linux_gtk_menu")?;
        let application = gio::Application::default()
            .and_then(|application| application.downcast::<gtk::Application>().ok())
            .ok_or_else(|| {
                ZsuiError::host(
                    "linux_gtk_menu",
                    "a running GTK Application is required before installing a native menu",
                )
            })?;
        Ok(Self::from_application(application))
    }

    pub(crate) fn from_application(application: gtk::Application) -> Self {
        Self {
            application,
            window: None,
            model: None,
            menu: None,
            pending_commands: Rc::new(RefCell::new(VecDeque::new())),
            command_handler: Rc::new(RefCell::new(None)),
            action_names: Vec::new(),
            next_native_id: 1,
        }
    }

    pub fn poll_menu_command(&mut self) -> Option<DesktopEvent> {
        loop {
            let native_id = self.pending_commands.borrow_mut().pop_front()?;
            if let Some(event) =
                native_menu_command_event(self.model.as_ref(), self.window, native_id)
            {
                return Some(event);
            }
        }
    }

    pub fn native_command_count(&self) -> usize {
        self.model
            .as_ref()
            .map(NativeMenuModel::command_count)
            .unwrap_or(0)
    }

    pub(crate) fn set_event_handler(&mut self, handler: impl Fn(DesktopEvent) + 'static) {
        let model = self.model.clone();
        let window = self.window;
        *self.command_handler.borrow_mut() = Some(Rc::new(move |native_id| {
            if let Some(event) = native_menu_command_event(model.as_ref(), window, native_id) {
                handler(event);
            }
        }));
    }

    pub(crate) fn invoke_first_enabled_command_for_proof(&self) -> bool {
        self.action_names.iter().any(|action_name| {
            self.application
                .lookup_action(action_name)
                .filter(|action| action.is_enabled())
                .is_some_and(|action| {
                    action.activate(None);
                    true
                })
        })
    }

    fn remove_owned_menu(&mut self) {
        if let Some(owned_menu) = self.menu.as_ref() {
            let owns_current_menu = self
                .application
                .menubar()
                .is_some_and(|current| current == *owned_menu);
            if owns_current_menu {
                self.application.set_menubar(None::<&gio::Menu>);
            }
        }
        for action_name in self.action_names.drain(..) {
            let detailed_action = format!("app.{action_name}");
            self.application
                .set_accels_for_action(&detailed_action, &[]);
            self.application.remove_action(&action_name);
        }
        self.menu = None;
    }
}

impl MenuService for LinuxGtkMenuService {
    fn set_window_menu(&mut self, window: WindowId, menu: Option<&MenuSpec>) -> ZsuiResult<()> {
        crate::linux_gtk_services::ensure_gtk_main_thread("linux_gtk_menu")?;
        self.command_handler.borrow_mut().take();
        let Some(menu_spec) = menu else {
            self.remove_owned_menu();
            self.window = None;
            self.model = None;
            return Ok(());
        };

        let model = NativeMenuModel::lower(menu_spec, self.next_native_id)?;
        self.remove_owned_menu();
        let native_menu = build_gtk_menu(
            &self.application,
            &model.items,
            &self.pending_commands,
            &self.command_handler,
            &mut self.action_names,
        );
        self.application.set_menubar(Some(&native_menu));
        self.next_native_id = model.next_native_id();
        self.window = Some(window);
        self.model = Some(model);
        self.menu = Some(native_menu);
        Ok(())
    }
}

impl Drop for LinuxGtkMenuService {
    fn drop(&mut self) {
        if gtk::is_initialized_main_thread() {
            self.remove_owned_menu();
        }
    }
}

fn build_gtk_menu(
    application: &gtk::Application,
    nodes: &[NativeMenuNode],
    pending_commands: &Rc<RefCell<VecDeque<u32>>>,
    command_handler: &Rc<RefCell<Option<Rc<dyn Fn(u32)>>>>,
    action_names: &mut Vec<String>,
) -> gio::Menu {
    let menu = gio::Menu::new();
    let mut section = gio::Menu::new();
    for node in nodes {
        match node {
            NativeMenuNode::Separator => {
                append_nonempty_gtk_section(&menu, &section);
                section = gio::Menu::new();
            }
            NativeMenuNode::Command {
                native_id,
                label,
                enabled,
                checked,
                accelerator,
            } => {
                let action_name = format!("zsui_menu_{native_id}");
                let detailed_action = format!("app.{action_name}");
                let action = if *checked {
                    gio::SimpleAction::new_stateful(&action_name, None, &true.to_variant())
                } else {
                    gio::SimpleAction::new(&action_name, None)
                };
                action.set_enabled(*enabled);
                let queue = Rc::clone(pending_commands);
                let handler = Rc::clone(command_handler);
                let native_id = *native_id;
                action.connect_activate(move |_, _| {
                    if let Some(handler) = handler.borrow().as_ref().cloned() {
                        handler(native_id);
                    } else {
                        queue.borrow_mut().push_back(native_id);
                    }
                });
                application.add_action(&action);
                if let Some(accelerator) = accelerator {
                    let accelerator = accelerator.gtk_accelerator();
                    application.set_accels_for_action(&detailed_action, &[&accelerator]);
                }
                action_names.push(action_name);
                section.append_item(&gio::MenuItem::new(Some(label), Some(&detailed_action)));
            }
            NativeMenuNode::Submenu {
                label,
                enabled,
                items,
            } => {
                let submenu = build_gtk_menu(
                    application,
                    items,
                    pending_commands,
                    command_handler,
                    action_names,
                );
                let item = gio::MenuItem::new_submenu(Some(label), &submenu);
                if !enabled {
                    let action_name = format!("zsui_menu_disabled_{}", action_names.len());
                    let detailed_action = format!("app.{action_name}");
                    let action = gio::SimpleAction::new(&action_name, None);
                    action.set_enabled(false);
                    application.add_action(&action);
                    item.set_detailed_action(&detailed_action);
                    action_names.push(action_name);
                }
                section.append_item(&item);
            }
        }
    }
    append_nonempty_gtk_section(&menu, &section);
    menu
}

fn append_nonempty_gtk_section(menu: &gio::Menu, section: &gio::Menu) {
    if section.n_items() > 0 {
        menu.append_section(None, section);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk_menu_service_implements_safe_public_contract() {
        fn assert_service<T: MenuService>() {}
        assert_service::<LinuxGtkMenuService>();
    }
}
