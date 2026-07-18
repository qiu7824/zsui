use cairo::Context;
use winit::keyboard::{Key, NamedKey};

use crate::native_draw_support::NativeDrawPalette;
use crate::{Color, Command, MenuItemSpec, MenuSpec, Point, Rect};

pub(crate) const LINUX_MENU_BAR_HEIGHT: i32 = 36;
const ROOT_HORIZONTAL_PADDING: i32 = 12;
const ROOT_GAP: i32 = 2;
const POPUP_MIN_WIDTH: i32 = 240;
const POPUP_HORIZONTAL_PADDING: i32 = 14;
const POPUP_ROW_HEIGHT: i32 = 34;
const POPUP_SEPARATOR_HEIGHT: i32 = 9;
const POPUP_RADIUS: f64 = 8.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinuxMenuInputResult {
    Ignored,
    Redraw,
    Command(Command),
}

#[derive(Debug, Clone)]
struct RootLayout {
    item_index: usize,
    bounds: Rect,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct PopupRowLayout {
    item_index: usize,
    bounds: Rect,
}

#[derive(Debug, Clone)]
pub(crate) struct LinuxDirectMenuSurface {
    menu: MenuSpec,
    width: i32,
    roots: Vec<RootLayout>,
    popup_rows: Vec<PopupRowLayout>,
    popup_bounds: Option<Rect>,
    open_root: Option<usize>,
    submenu_path: Vec<usize>,
    hovered_root: Option<usize>,
    hovered_row: Option<usize>,
    keyboard_row: Option<usize>,
    pressed_in_surface: bool,
}

impl LinuxDirectMenuSurface {
    pub(crate) fn new(menu: MenuSpec) -> Self {
        Self {
            menu,
            width: 1,
            roots: Vec::new(),
            popup_rows: Vec::new(),
            popup_bounds: None,
            open_root: None,
            submenu_path: Vec::new(),
            hovered_root: None,
            hovered_row: None,
            keyboard_row: None,
            pressed_in_surface: false,
        }
    }

    pub(crate) const fn content_offset_y(&self) -> i32 {
        LINUX_MENU_BAR_HEIGHT
    }

    pub(crate) const fn is_open(&self) -> bool {
        self.open_root.is_some()
    }

    pub(crate) fn layout(&mut self, width: i32, pango_context: &pango::Context) {
        self.width = width.max(1);
        self.roots.clear();
        let mut x = 6;
        for (item_index, item) in self.menu.items.iter().enumerate() {
            let Some((label, enabled)) = menu_item_label_and_enabled(item) else {
                continue;
            };
            let label_width = measure_text(pango_context, label).0;
            let root_width = label_width
                .saturating_add(ROOT_HORIZONTAL_PADDING.saturating_mul(2))
                .max(44);
            self.roots.push(RootLayout {
                item_index,
                bounds: Rect {
                    x,
                    y: 3,
                    width: root_width,
                    height: LINUX_MENU_BAR_HEIGHT - 6,
                },
                enabled,
            });
            x = x.saturating_add(root_width).saturating_add(ROOT_GAP);
        }
        self.layout_popup(pango_context);
    }

    pub(crate) fn captures_pointer(&self, point: Point) -> bool {
        point.y < LINUX_MENU_BAR_HEIGHT
            || self
                .popup_bounds
                .is_some_and(|bounds| bounds.contains(point))
            || self.is_open()
    }

    pub(crate) fn pointer_move(&mut self, point: Point, pango_context: &pango::Context) -> bool {
        let hovered_root = self
            .roots
            .iter()
            .position(|root| root.bounds.contains(point));
        let hovered_row = self
            .popup_rows
            .iter()
            .position(|row| row.bounds.contains(point));
        let changed = self.hovered_root != hovered_root || self.hovered_row != hovered_row;
        self.hovered_root = hovered_root;
        self.hovered_row = hovered_row;
        if self.is_open() {
            if let Some(root) = hovered_root
                .filter(|root| self.roots.get(*root).is_some_and(|layout| layout.enabled))
            {
                if self.open_root != Some(root) {
                    self.open_root = Some(root);
                    self.submenu_path.clear();
                    self.keyboard_row = None;
                    self.layout_popup(pango_context);
                    return true;
                }
            }
        }
        changed
    }

    pub(crate) fn pointer_down(&mut self, point: Point) -> bool {
        let in_surface = point.y < LINUX_MENU_BAR_HEIGHT
            || self
                .popup_bounds
                .is_some_and(|bounds| bounds.contains(point));
        self.pressed_in_surface = in_surface || self.is_open();
        self.pressed_in_surface
    }

    pub(crate) fn pointer_up(
        &mut self,
        point: Point,
        pango_context: &pango::Context,
    ) -> LinuxMenuInputResult {
        let consumed = self.pressed_in_surface || self.is_open();
        self.pressed_in_surface = false;

        if let Some(root_index) = self
            .roots
            .iter()
            .position(|root| root.bounds.contains(point) && root.enabled)
        {
            if self.open_root == Some(root_index) {
                self.close();
            } else {
                self.open_root = Some(root_index);
                self.submenu_path.clear();
                self.keyboard_row = first_enabled_row(self.current_menu());
                self.layout_popup(pango_context);
            }
            return LinuxMenuInputResult::Redraw;
        }

        if let Some(row_index) = self
            .popup_rows
            .iter()
            .position(|row| row.bounds.contains(point))
        {
            return self.activate_row(row_index, pango_context);
        }

        if self.is_open() {
            self.close();
            return LinuxMenuInputResult::Redraw;
        }
        if consumed {
            LinuxMenuInputResult::Redraw
        } else {
            LinuxMenuInputResult::Ignored
        }
    }

    pub(crate) fn key(
        &mut self,
        key: &Key,
        pango_context: &pango::Context,
    ) -> LinuxMenuInputResult {
        if matches!(key, Key::Named(NamedKey::F10)) {
            if self.is_open() {
                self.close();
            } else if let Some(root) = self.roots.iter().position(|root| root.enabled) {
                self.open_root = Some(root);
                self.submenu_path.clear();
                self.keyboard_row = first_enabled_row(self.current_menu());
                self.layout_popup(pango_context);
            }
            return LinuxMenuInputResult::Redraw;
        }
        if !self.is_open() {
            return LinuxMenuInputResult::Ignored;
        }
        match key {
            Key::Named(NamedKey::Escape) => {
                if self.submenu_path.pop().is_none() {
                    self.close();
                } else {
                    self.keyboard_row = first_enabled_row(self.current_menu());
                    self.layout_popup(pango_context);
                }
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.move_root(-1, pango_context);
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.move_root(1, pango_context);
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.move_row(-1);
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.move_row(1);
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => self
                .keyboard_row
                .map(|row| self.activate_row(row, pango_context))
                .unwrap_or(LinuxMenuInputResult::Redraw),
            _ => LinuxMenuInputResult::Redraw,
        }
    }

    pub(crate) fn proof_command(&mut self, pango_context: &pango::Context) -> Option<Command> {
        let root = self.roots.iter().position(|root| root.enabled)?;
        self.open_root = Some(root);
        self.submenu_path.clear();
        self.layout_popup(pango_context);
        let row = first_enabled_row(self.current_menu())?;
        let result = self.activate_row(row, pango_context);
        match result {
            LinuxMenuInputResult::Command(command) => Some(command),
            _ => None,
        }
    }

    pub(crate) fn open_for_capture(&mut self, pango_context: &pango::Context) -> bool {
        let Some(root) = self.roots.iter().position(|root| root.enabled) else {
            return false;
        };
        self.open_root = Some(root);
        self.submenu_path.clear();
        self.keyboard_row = first_enabled_row(self.current_menu());
        self.layout_popup(pango_context);
        true
    }

    pub(crate) fn draw(
        &self,
        context: &Context,
        pango_context: &pango::Context,
        palette: NativeDrawPalette,
    ) {
        set_source(context, palette.surface_raised);
        context.rectangle(
            0.0,
            0.0,
            f64::from(self.width),
            f64::from(LINUX_MENU_BAR_HEIGHT),
        );
        let _ = context.fill();
        set_source(context, palette.border);
        context.rectangle(
            0.0,
            f64::from(LINUX_MENU_BAR_HEIGHT - 1),
            f64::from(self.width),
            1.0,
        );
        let _ = context.fill();

        for (root_index, root) in self.roots.iter().enumerate() {
            if self.open_root == Some(root_index) || self.hovered_root == Some(root_index) {
                draw_round_rect(context, root.bounds, 6.0);
                set_source(
                    context,
                    if self.open_root == Some(root_index) {
                        palette.control
                    } else {
                        palette.surface
                    },
                );
                let _ = context.fill();
            }
            let label = menu_item_label_and_enabled(&self.menu.items[root.item_index])
                .map(|(label, _)| label)
                .unwrap_or_default();
            draw_text(
                context,
                pango_context,
                label,
                root.bounds,
                if root.enabled {
                    palette.primary_text
                } else {
                    palette.disabled_text
                },
                TextPlacement::Center,
            );
        }

        let (Some(popup), Some(menu)) = (self.popup_bounds, self.current_menu()) else {
            return;
        };
        let shadow = Rect {
            x: popup.x + 2,
            y: popup.y + 3,
            width: popup.width,
            height: popup.height,
        };
        draw_round_rect(context, shadow, POPUP_RADIUS);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.22);
        let _ = context.fill();
        draw_round_rect(context, popup, POPUP_RADIUS);
        set_source(context, palette.surface_raised);
        let _ = context.fill_preserve();
        set_source(context, palette.border);
        context.set_line_width(1.0);
        let _ = context.stroke();

        for (row_index, row) in self.popup_rows.iter().enumerate() {
            let Some(item) = menu.items.get(row.item_index) else {
                continue;
            };
            if matches!(item, MenuItemSpec::Separator) {
                set_source(context, palette.border);
                context.rectangle(
                    f64::from(row.bounds.x + 10),
                    f64::from(row.bounds.y + row.bounds.height / 2),
                    f64::from((row.bounds.width - 20).max(1)),
                    1.0,
                );
                let _ = context.fill();
                continue;
            }
            let selected =
                self.hovered_row == Some(row_index) || self.keyboard_row == Some(row_index);
            if selected {
                let highlight = Rect {
                    x: row.bounds.x + 4,
                    y: row.bounds.y + 2,
                    width: row.bounds.width - 8,
                    height: row.bounds.height - 4,
                };
                draw_round_rect(context, highlight, 6.0);
                set_source(context, palette.control);
                let _ = context.fill();
            }
            let Some((label, enabled)) = menu_item_label_and_enabled(item) else {
                continue;
            };
            let text_color = if enabled {
                palette.primary_text
            } else {
                palette.disabled_text
            };
            let label_bounds = Rect {
                x: row.bounds.x + POPUP_HORIZONTAL_PADDING + 18,
                y: row.bounds.y,
                width: row.bounds.width - POPUP_HORIZONTAL_PADDING * 2 - 60,
                height: row.bounds.height,
            };
            draw_text(
                context,
                pango_context,
                label,
                label_bounds,
                text_color,
                TextPlacement::Start,
            );
            match item {
                MenuItemSpec::Command {
                    checked,
                    accelerator,
                    ..
                } => {
                    if *checked {
                        draw_text(
                            context,
                            pango_context,
                            "✓",
                            Rect {
                                x: row.bounds.x + 8,
                                y: row.bounds.y,
                                width: 22,
                                height: row.bounds.height,
                            },
                            text_color,
                            TextPlacement::Center,
                        );
                    }
                    if let Some(accelerator) = accelerator {
                        draw_text(
                            context,
                            pango_context,
                            &linux_accelerator_label(*accelerator),
                            Rect {
                                x: row.bounds.x + row.bounds.width - 112,
                                y: row.bounds.y,
                                width: 94,
                                height: row.bounds.height,
                            },
                            if enabled {
                                palette.secondary_text
                            } else {
                                palette.disabled_text
                            },
                            TextPlacement::End,
                        );
                    }
                }
                MenuItemSpec::Submenu { .. } => {
                    draw_text(
                        context,
                        pango_context,
                        "›",
                        Rect {
                            x: row.bounds.x + row.bounds.width - 28,
                            y: row.bounds.y,
                            width: 18,
                            height: row.bounds.height,
                        },
                        text_color,
                        TextPlacement::Center,
                    );
                }
                MenuItemSpec::Separator => {}
            }
        }
    }

    fn close(&mut self) {
        self.open_root = None;
        self.submenu_path.clear();
        self.popup_rows.clear();
        self.popup_bounds = None;
        self.hovered_row = None;
        self.keyboard_row = None;
    }

    fn current_menu(&self) -> Option<&MenuSpec> {
        let root = self.open_root.and_then(|index| self.roots.get(index))?;
        let mut menu = match self.menu.items.get(root.item_index)? {
            MenuItemSpec::Submenu { menu, .. } => menu,
            _ => return None,
        };
        for index in &self.submenu_path {
            menu = match menu.items.get(*index)? {
                MenuItemSpec::Submenu { menu, .. } => menu,
                _ => return None,
            };
        }
        Some(menu)
    }

    fn layout_popup(&mut self, pango_context: &pango::Context) {
        self.popup_rows.clear();
        self.popup_bounds = None;
        let Some(root_index) = self.open_root else {
            return;
        };
        let Some(root) = self.roots.get(root_index) else {
            return;
        };
        let Some(menu) = self.current_menu().cloned() else {
            return;
        };
        let mut popup_width = POPUP_MIN_WIDTH;
        let mut popup_height: i32 = 8;
        for item in &menu.items {
            let row_height = if matches!(item, MenuItemSpec::Separator) {
                POPUP_SEPARATOR_HEIGHT
            } else {
                POPUP_ROW_HEIGHT
            };
            popup_height = popup_height.saturating_add(row_height);
            if let Some((label, _)) = menu_item_label_and_enabled(item) {
                let label_width = measure_text(pango_context, label).0;
                let accelerator_width = match item {
                    MenuItemSpec::Command {
                        accelerator: Some(accelerator),
                        ..
                    } => measure_text(pango_context, &linux_accelerator_label(*accelerator)).0,
                    MenuItemSpec::Submenu { .. } => 16,
                    _ => 0,
                };
                popup_width = popup_width.max(
                    label_width
                        .saturating_add(accelerator_width)
                        .saturating_add(86),
                );
            }
        }
        popup_height = popup_height.saturating_add(8);
        let x = root.bounds.x.min((self.width - popup_width - 6).max(6));
        let popup = Rect {
            x,
            y: LINUX_MENU_BAR_HEIGHT + 2,
            width: popup_width.min((self.width - 12).max(120)),
            height: popup_height,
        };
        let mut y = popup.y + 4;
        for (item_index, item) in menu.items.iter().enumerate() {
            let row_height = if matches!(item, MenuItemSpec::Separator) {
                POPUP_SEPARATOR_HEIGHT
            } else {
                POPUP_ROW_HEIGHT
            };
            self.popup_rows.push(PopupRowLayout {
                item_index,
                bounds: Rect {
                    x: popup.x,
                    y,
                    width: popup.width,
                    height: row_height,
                },
            });
            y = y.saturating_add(row_height);
        }
        self.popup_bounds = Some(popup);
    }

    fn activate_row(
        &mut self,
        row_index: usize,
        pango_context: &pango::Context,
    ) -> LinuxMenuInputResult {
        let Some(row) = self.popup_rows.get(row_index) else {
            return LinuxMenuInputResult::Redraw;
        };
        let item_index = row.item_index;
        let Some(item) = self
            .current_menu()
            .and_then(|menu| menu.items.get(item_index))
            .cloned()
        else {
            return LinuxMenuInputResult::Redraw;
        };
        match item {
            MenuItemSpec::Command {
                command,
                enabled: true,
                ..
            } => {
                self.close();
                LinuxMenuInputResult::Command(command)
            }
            MenuItemSpec::Submenu { enabled: true, .. } => {
                self.submenu_path.push(item_index);
                self.keyboard_row = first_enabled_row(self.current_menu());
                self.layout_popup(pango_context);
                LinuxMenuInputResult::Redraw
            }
            _ => LinuxMenuInputResult::Redraw,
        }
    }

    fn move_root(&mut self, offset: isize, pango_context: &pango::Context) {
        let enabled = self
            .roots
            .iter()
            .enumerate()
            .filter_map(|(index, root)| root.enabled.then_some(index))
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return;
        }
        let current = self
            .open_root
            .and_then(|root| enabled.iter().position(|candidate| *candidate == root))
            .unwrap_or(0);
        let next = (current as isize + offset).rem_euclid(enabled.len() as isize) as usize;
        self.open_root = Some(enabled[next]);
        self.submenu_path.clear();
        self.keyboard_row = first_enabled_row(self.current_menu());
        self.layout_popup(pango_context);
    }

    fn move_row(&mut self, offset: isize) {
        let Some(menu) = self.current_menu() else {
            return;
        };
        let enabled = menu
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| menu_item_enabled(item).then_some(index))
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return;
        }
        let current_item = self
            .keyboard_row
            .and_then(|row| self.popup_rows.get(row))
            .map(|row| row.item_index);
        let current = current_item
            .and_then(|item| enabled.iter().position(|candidate| *candidate == item))
            .unwrap_or(0);
        let next = (current as isize + offset).rem_euclid(enabled.len() as isize) as usize;
        self.keyboard_row = self
            .popup_rows
            .iter()
            .position(|row| row.item_index == enabled[next]);
    }
}

fn menu_item_label_and_enabled(item: &MenuItemSpec) -> Option<(&str, bool)> {
    match item {
        MenuItemSpec::Command { label, enabled, .. }
        | MenuItemSpec::Submenu { label, enabled, .. } => Some((label, *enabled)),
        MenuItemSpec::Separator => None,
    }
}

fn menu_item_enabled(item: &MenuItemSpec) -> bool {
    match item {
        MenuItemSpec::Command { enabled, .. } | MenuItemSpec::Submenu { enabled, .. } => *enabled,
        MenuItemSpec::Separator => false,
    }
}

fn first_enabled_row(menu: Option<&MenuSpec>) -> Option<usize> {
    menu?.items.iter().position(menu_item_enabled)
}

fn linux_accelerator_label(accelerator: crate::ZsAccelerator) -> String {
    accelerator
        .to_string()
        .replace("Primary+", "Ctrl+")
        .replace("Super+", "Super+")
}

fn measure_text(context: &pango::Context, text: &str) -> (i32, i32) {
    let layout = pango::Layout::new(context);
    layout.set_font_description(Some(&menu_font_description()));
    layout.set_text(text);
    layout.pixel_size()
}

#[derive(Clone, Copy)]
enum TextPlacement {
    Start,
    Center,
    End,
}

fn draw_text(
    context: &Context,
    pango_context: &pango::Context,
    text: &str,
    bounds: Rect,
    color: Color,
    placement: TextPlacement,
) {
    let layout = pango::Layout::new(pango_context);
    layout.set_font_description(Some(&menu_font_description()));
    layout.set_text(text);
    let (width, height) = layout.pixel_size();
    let x = match placement {
        TextPlacement::Start => bounds.x,
        TextPlacement::Center => bounds.x + (bounds.width - width).max(0) / 2,
        TextPlacement::End => bounds.x + (bounds.width - width).max(0),
    };
    let y = bounds.y + (bounds.height - height).max(0) / 2;
    set_source(context, color);
    context.move_to(f64::from(x), f64::from(y));
    pangocairo::functions::show_layout(context, &layout);
}

fn menu_font_description() -> pango::FontDescription {
    let configured = std::env::var("ZSUI_LINUX_UI_FONT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Ubuntu Sans 11".to_string());
    pango::FontDescription::from_string(&configured)
}

fn draw_round_rect(context: &Context, rect: Rect, radius: f64) {
    let x = f64::from(rect.x);
    let y = f64::from(rect.y);
    let width = f64::from(rect.width.max(0));
    let height = f64::from(rect.height.max(0));
    let radius = radius.min(width / 2.0).min(height / 2.0);
    context.new_sub_path();
    context.arc(
        x + width - radius,
        y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    context.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    context.arc(
        x + radius,
        y + height - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    context.arc(
        x + radius,
        y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    context.close_path();
}

fn set_source(context: &Context, color: Color) {
    context.set_source_rgba(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_accelerator_uses_linux_ctrl_label() {
        assert_eq!(
            linux_accelerator_label(crate::ZsAccelerator::primary_character('s')),
            "Ctrl+S"
        );
    }

    #[test]
    fn first_enabled_row_skips_separator_and_disabled_item() {
        let mut menu = MenuSpec::new().separator();
        menu.items
            .push(MenuItemSpec::command("Disabled", Command::Quit).disabled());
        let menu = menu.item("Open", Command::custom("open"));
        assert_eq!(first_enabled_row(Some(&menu)), Some(2));
    }
}
