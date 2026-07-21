use std::rc::Rc;

use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{NSImage, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::NSString;

use crate::{Command, TraySpec, ZsuiError, ZsuiResult};

pub(crate) struct MacosAppKitStatusItemHost {
    status_bar: Retained<NSStatusBar>,
    items: Vec<Retained<NSStatusItem>>,
    menus: Vec<crate::macos_appkit_menu::MacosAppKitMenuService>,
}

impl MacosAppKitStatusItemHost {
    pub(crate) fn create(
        trays: &[TraySpec],
        command_handler: Option<Rc<dyn Fn(Command)>>,
    ) -> ZsuiResult<Self> {
        let _mtm = MainThreadMarker::new().ok_or_else(|| {
            ZsuiError::host(
                "macos_status_item",
                "AppKit status items must be created on the macOS main thread",
            )
        })?;
        let mut host = Self {
            status_bar: NSStatusBar::systemStatusBar(),
            items: Vec::with_capacity(trays.len()),
            menus: Vec::with_capacity(trays.len()),
        };
        for tray in trays {
            host.create_item(tray, command_handler.clone())?;
        }
        Ok(host)
    }

    pub(crate) fn item_count(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn native_command_count(&self) -> usize {
        self.menus
            .iter()
            .map(crate::macos_appkit_menu::MacosAppKitMenuService::native_command_count)
            .sum()
    }

    pub(crate) fn invoke_first_enabled_command_for_proof(&self) -> bool {
        self.menus
            .iter()
            .any(|menu| menu.invoke_first_enabled_command_for_proof())
    }

    fn create_item(
        &mut self,
        tray: &TraySpec,
        command_handler: Option<Rc<dyn Fn(Command)>>,
    ) -> ZsuiResult<()> {
        let mut menu = crate::macos_appkit_menu::MacosAppKitMenuService::new()?;
        menu.set_detached_menu(&tray.menu)?;
        if let Some(command_handler) = command_handler {
            menu.set_command_handler(move |command| command_handler(command));
        }

        let item = self
            .status_bar
            .statusItemWithLength(NSVariableStatusItemLength);
        self.items.push(item);
        let item = self
            .items
            .last()
            .expect("the AppKit status item was retained before configuration");

        #[allow(deprecated)]
        if let Some(icon_path) = tray.icon_path.as_deref() {
            let image =
                NSImage::initWithContentsOfFile(NSImage::alloc(), &NSString::from_str(icon_path))
                    .ok_or_else(|| {
                    ZsuiError::host(
                        "macos_status_item_icon",
                        format!("NSImage could not load status item icon `{icon_path}`"),
                    )
                })?;
            image.setTemplate(true);
            item.setImage(Some(&image));
            item.setTitle(None);
        } else {
            let title = tray.tooltip.as_deref().unwrap_or("ZSUI");
            item.setTitle(Some(&NSString::from_str(title)));
        }

        #[allow(deprecated)]
        item.setToolTip(tray.tooltip.as_deref().map(NSString::from_str).as_deref());
        item.setMenu(menu.native_menu());
        item.setVisible(true);
        self.menus.push(menu);
        Ok(())
    }
}

impl Drop for MacosAppKitStatusItemHost {
    fn drop(&mut self) {
        for item in self.items.drain(..) {
            item.setMenu(None);
            self.status_bar.removeStatusItem(&item);
        }
        self.menus.clear();
    }
}
