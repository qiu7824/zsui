use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSControlStateValueOn, NSEventModifierFlags, NSMenu, NSMenuItem,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};

use crate::native_menu::{native_menu_command_event, NativeMenuModel, NativeMenuNode};
use crate::{DesktopEvent, MenuService, MenuSpec, WindowId, ZsAccelerator, ZsuiError, ZsuiResult};

struct AppKitMenuTargetIvars {
    sender: Sender<u32>,
    command_handler: Rc<RefCell<Option<Rc<dyn Fn(u32)>>>>,
}

impl fmt::Debug for AppKitMenuTargetIvars {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppKitMenuTargetIvars")
            .finish_non_exhaustive()
    }
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppKitMenuTargetIvars]
    struct ZsuiAppKitMenuTarget;

    impl ZsuiAppKitMenuTarget {
        #[unsafe(method(zsuiMenuCommand:))]
        fn zsui_menu_command(&self, sender: &NSMenuItem) {
            let native_id = sender.tag();
            if let Ok(native_id) = u32::try_from(native_id) {
                if let Some(handler) = self.ivars().command_handler.borrow().as_ref().cloned() {
                    handler(native_id);
                } else {
                    let _ = self.ivars().sender.send(native_id);
                }
            }
        }
    }

    unsafe impl NSObjectProtocol for ZsuiAppKitMenuTarget {}
);

impl ZsuiAppKitMenuTarget {
    fn new(
        mtm: MainThreadMarker,
        sender: Sender<u32>,
        command_handler: Rc<RefCell<Option<Rc<dyn Fn(u32)>>>>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppKitMenuTargetIvars {
            sender,
            command_handler,
        });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct MacosAppKitMenuService {
    window: Option<WindowId>,
    model: Option<NativeMenuModel>,
    menu: Option<Retained<NSMenu>>,
    target: Retained<ZsuiAppKitMenuTarget>,
    command_handler: Rc<RefCell<Option<Rc<dyn Fn(u32)>>>>,
    receiver: Receiver<u32>,
    next_native_id: u32,
}

impl fmt::Debug for MacosAppKitMenuService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MacosAppKitMenuService")
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

impl MacosAppKitMenuService {
    pub fn new() -> ZsuiResult<Self> {
        let mtm = appkit_menu_main_thread_marker()?;
        let (sender, receiver) = mpsc::channel();
        let command_handler = Rc::new(RefCell::new(None));
        Ok(Self {
            window: None,
            model: None,
            menu: None,
            target: ZsuiAppKitMenuTarget::new(mtm, sender, Rc::clone(&command_handler)),
            command_handler,
            receiver,
            next_native_id: 1,
        })
    }

    pub fn poll_menu_command(&mut self) -> Option<DesktopEvent> {
        loop {
            let native_id = match self.receiver.try_recv() {
                Ok(native_id) => native_id,
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return None,
            };
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

    pub(crate) fn set_command_handler(&mut self, handler: impl Fn(crate::Command) + 'static) {
        let model = self.model.clone();
        *self.command_handler.borrow_mut() = Some(Rc::new(move |native_id| {
            if let Some(command) = model
                .as_ref()
                .and_then(|model| model.command_for_native_id(native_id))
            {
                handler(command);
            }
        }));
    }

    pub(crate) fn set_detached_menu(&mut self, menu: &MenuSpec) -> ZsuiResult<()> {
        self.set_menu(None, Some(menu), false)
    }

    pub(crate) fn native_menu(&self) -> Option<&NSMenu> {
        self.menu.as_deref()
    }

    pub(crate) fn invoke_first_enabled_command_for_proof(&self) -> bool {
        self.menu
            .as_deref()
            .is_some_and(perform_first_enabled_appkit_menu_action)
    }
}

fn perform_first_enabled_appkit_menu_action(menu: &NSMenu) -> bool {
    for index in 0..menu.numberOfItems() {
        let Some(item) = menu.itemAtIndex(index) else {
            continue;
        };
        if let Some(submenu) = item.submenu() {
            if perform_first_enabled_appkit_menu_action(&submenu) {
                return true;
            }
            continue;
        }
        if item.isEnabled() && item.tag() > 0 {
            menu.performActionForItemAtIndex(index);
            return true;
        }
    }
    false
}

impl MenuService for MacosAppKitMenuService {
    fn set_window_menu(&mut self, window: WindowId, menu: Option<&MenuSpec>) -> ZsuiResult<()> {
        self.set_menu(Some(window), menu, true)
    }
}

impl MacosAppKitMenuService {
    fn set_menu(
        &mut self,
        window: Option<WindowId>,
        menu: Option<&MenuSpec>,
        install_as_main_menu: bool,
    ) -> ZsuiResult<()> {
        let mtm = appkit_menu_main_thread_marker()?;
        let application = NSApplication::sharedApplication(mtm);
        self.command_handler.borrow_mut().take();
        detach_owned_appkit_menu(&application, self.menu.as_ref());
        let Some(menu_spec) = menu else {
            self.window = None;
            self.model = None;
            self.menu = None;
            return Ok(());
        };

        let model = NativeMenuModel::lower(menu_spec, self.next_native_id)?;
        let native_menu = build_appkit_menu(&model, &self.target, mtm);
        if install_as_main_menu {
            application.setMainMenu(Some(&native_menu));
        }
        self.next_native_id = model.next_native_id();
        self.window = window;
        self.model = Some(model);
        self.menu = Some(native_menu);
        Ok(())
    }
}

impl Drop for MacosAppKitMenuService {
    fn drop(&mut self) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let application = NSApplication::sharedApplication(mtm);
        detach_owned_appkit_menu(&application, self.menu.as_ref());
    }
}

fn appkit_menu_main_thread_marker() -> ZsuiResult<MainThreadMarker> {
    MainThreadMarker::new().ok_or_else(|| {
        ZsuiError::host(
            "macos_appkit_menu",
            "AppKit menus must be installed and polled from the macOS main thread",
        )
    })
}

fn detach_owned_appkit_menu(application: &NSApplication, owned_menu: Option<&Retained<NSMenu>>) {
    let Some(owned_menu) = owned_menu else {
        return;
    };
    let Some(current_menu) = application.mainMenu() else {
        return;
    };
    let current_menu: &NSMenu = current_menu.as_ref();
    let owned_menu: &NSMenu = owned_menu.as_ref();
    if std::ptr::eq(current_menu, owned_menu) {
        application.setMainMenu(None);
    }
}

fn build_appkit_menu(
    model: &NativeMenuModel,
    target: &ZsuiAppKitMenuTarget,
    mtm: MainThreadMarker,
) -> Retained<NSMenu> {
    let title = NSString::from_str(model.title.as_deref().unwrap_or(""));
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);
    append_appkit_menu_nodes(&menu, &model.items, target, mtm);
    menu
}

fn append_appkit_menu_nodes(
    menu: &NSMenu,
    nodes: &[NativeMenuNode],
    target: &ZsuiAppKitMenuTarget,
    mtm: MainThreadMarker,
) {
    for node in nodes {
        match node {
            NativeMenuNode::Command {
                native_id,
                label,
                enabled,
                checked,
                accelerator,
            } => {
                let title = NSString::from_str(label);
                let key_equivalent = accelerator
                    .as_ref()
                    .and_then(crate::platform_menu_accelerator::appkit_key_equivalent)
                    .unwrap_or_default();
                let key_equivalent = NSString::from_str(&key_equivalent);
                let item = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &title,
                        Some(sel!(zsuiMenuCommand:)),
                        &key_equivalent,
                    )
                };
                unsafe { item.setTarget(Some(target)) };
                item.setTag(*native_id as isize);
                item.setEnabled(*enabled);
                if *checked {
                    item.setState(NSControlStateValueOn);
                }
                if let Some(accelerator) = accelerator {
                    item.setKeyEquivalentModifierMask(appkit_modifier_flags(accelerator));
                }
                menu.addItem(&item);
            }
            NativeMenuNode::Separator => menu.addItem(&NSMenuItem::separatorItem(mtm)),
            NativeMenuNode::Submenu {
                label,
                enabled,
                items,
            } => {
                let title = NSString::from_str(label);
                let item = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &title,
                        None,
                        &NSString::new(),
                    )
                };
                item.setEnabled(*enabled);
                let submenu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);
                append_appkit_menu_nodes(&submenu, items, target, mtm);
                item.setSubmenu(Some(&submenu));
                menu.addItem(&item);
            }
        }
    }
}

fn appkit_modifier_flags(accelerator: &ZsAccelerator) -> NSEventModifierFlags {
    let mut flags = NSEventModifierFlags::empty();
    if accelerator.uses_primary() || accelerator.uses_super() {
        flags |= NSEventModifierFlags::Command;
    }
    if accelerator.uses_alt() {
        flags |= NSEventModifierFlags::Option;
    }
    if accelerator.uses_shift() {
        flags |= NSEventModifierFlags::Shift;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appkit_menu_service_implements_safe_public_contract() {
        fn assert_service<T: MenuService>() {}
        assert_service::<MacosAppKitMenuService>();
    }
}
