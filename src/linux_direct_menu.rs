#[cfg(feature = "linux-direct")]
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

pub(crate) trait LinuxMenuTextMetrics {
    fn measure_menu_text(&self, text: &str) -> (i32, i32);
}

#[derive(Clone, Copy)]
pub(crate) enum LinuxMenuTextPlacement {
    Start,
    Center,
    End,
}

pub(crate) trait LinuxMenuCanvas {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn fill_round_rect(&mut self, rect: Rect, radius: f64, color: Color);
    fn stroke_round_rect(&mut self, rect: Rect, radius: f64, color: Color, width: f32);
    fn draw_text(
        &mut self,
        text: &str,
        bounds: Rect,
        color: Color,
        placement: LinuxMenuTextPlacement,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinuxMenuInputResult {
    Ignored,
    Redraw,
    Command(Command),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "accessibility")]
pub(crate) enum LinuxMenuAccessibilityTarget {
    Root(usize),
    Row(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "accessibility")]
pub(crate) enum LinuxMenuAccessibilityRole {
    Menu,
    MenuItem,
    CheckedMenuItem,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "accessibility")]
pub(crate) struct LinuxMenuAccessibilityItem {
    pub target: Option<LinuxMenuAccessibilityTarget>,
    pub author_id: String,
    pub label: String,
    pub bounds: Rect,
    pub role: LinuxMenuAccessibilityRole,
    pub enabled: bool,
    pub expanded: Option<bool>,
    pub checked: Option<bool>,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "accessibility")]
pub(crate) struct LinuxMenuAccessibilitySnapshot {
    pub bar_bounds: Rect,
    pub roots: Vec<LinuxMenuAccessibilityItem>,
    pub rows: Vec<LinuxMenuAccessibilityItem>,
    pub open_root: Option<usize>,
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

    pub(crate) const fn spec(&self) -> &MenuSpec {
        &self.menu
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_snapshot(&self) -> LinuxMenuAccessibilitySnapshot {
        let roots = self
            .roots
            .iter()
            .enumerate()
            .filter_map(|(root_index, root)| {
                let item = self.menu.items.get(root.item_index)?;
                let (label, enabled) = menu_item_label_and_enabled(item)?;
                Some(LinuxMenuAccessibilityItem {
                    target: Some(LinuxMenuAccessibilityTarget::Root(root_index)),
                    author_id: menu_item_author_id(item, "root", root.item_index),
                    label: label.to_string(),
                    bounds: root.bounds,
                    role: LinuxMenuAccessibilityRole::Menu,
                    enabled,
                    expanded: Some(self.open_root == Some(root_index)),
                    checked: None,
                    focused: self.open_root == Some(root_index) && self.keyboard_row.is_none(),
                })
            })
            .collect();
        let rows = self
            .current_menu()
            .map(|menu| {
                self.popup_rows
                    .iter()
                    .enumerate()
                    .filter_map(|(row_index, row)| {
                        let item = menu.items.get(row.item_index)?;
                        let (label, enabled, role, checked, expanded) = match item {
                            MenuItemSpec::Command {
                                label,
                                enabled,
                                checked,
                                ..
                            } => (
                                label.as_str(),
                                *enabled,
                                if *checked {
                                    LinuxMenuAccessibilityRole::CheckedMenuItem
                                } else {
                                    LinuxMenuAccessibilityRole::MenuItem
                                },
                                (*checked).then_some(true),
                                None,
                            ),
                            MenuItemSpec::Submenu { label, enabled, .. } => (
                                label.as_str(),
                                *enabled,
                                LinuxMenuAccessibilityRole::Menu,
                                None,
                                Some(false),
                            ),
                            MenuItemSpec::Separator => {
                                ("", false, LinuxMenuAccessibilityRole::Separator, None, None)
                            }
                        };
                        Some(LinuxMenuAccessibilityItem {
                            target: (!matches!(item, MenuItemSpec::Separator))
                                .then_some(LinuxMenuAccessibilityTarget::Row(row_index)),
                            author_id: menu_item_author_id(item, "row", row.item_index),
                            label: label.to_string(),
                            bounds: row.bounds,
                            role,
                            enabled,
                            expanded,
                            checked,
                            focused: self.keyboard_row == Some(row_index),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        LinuxMenuAccessibilitySnapshot {
            bar_bounds: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: LINUX_MENU_BAR_HEIGHT,
            },
            roots,
            rows,
            open_root: self.open_root,
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_focus_root(
        &mut self,
        root_index: usize,
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> bool {
        if !self.roots.get(root_index).is_some_and(|root| root.enabled) {
            return false;
        }
        self.open_root = Some(root_index);
        self.submenu_path.clear();
        self.keyboard_row = None;
        self.layout_popup(text_metrics);
        true
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_focus_row(&mut self, row_index: usize) -> bool {
        let enabled = self
            .popup_rows
            .get(row_index)
            .and_then(|row| self.current_menu()?.items.get(row.item_index))
            .is_some_and(menu_item_enabled);
        if !enabled {
            return false;
        }
        self.keyboard_row = Some(row_index);
        true
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_activate_root(
        &mut self,
        root_index: usize,
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> LinuxMenuInputResult {
        if self.open_root == Some(root_index) {
            self.close();
            LinuxMenuInputResult::Redraw
        } else if self.accessibility_focus_root(root_index, text_metrics) {
            self.keyboard_row = first_enabled_row(self.current_menu());
            LinuxMenuInputResult::Redraw
        } else {
            LinuxMenuInputResult::Ignored
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_activate_row(
        &mut self,
        row_index: usize,
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> LinuxMenuInputResult {
        self.activate_row(row_index, text_metrics)
    }

    pub(crate) fn layout(&mut self, width: i32, text_metrics: &dyn LinuxMenuTextMetrics) {
        self.width = width.max(1);
        self.roots.clear();
        let mut x = 6;
        for (item_index, item) in self.menu.items.iter().enumerate() {
            let Some((label, enabled)) = menu_item_label_and_enabled(item) else {
                continue;
            };
            let label_width = text_metrics.measure_menu_text(label).0;
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
        self.layout_popup(text_metrics);
    }

    pub(crate) fn captures_pointer(&self, point: Point) -> bool {
        point.y < LINUX_MENU_BAR_HEIGHT
            || self
                .popup_bounds
                .is_some_and(|bounds| bounds.contains(point))
            || self.is_open()
    }

    pub(crate) fn pointer_move(
        &mut self,
        point: Point,
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> bool {
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
                    self.layout_popup(text_metrics);
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
        text_metrics: &dyn LinuxMenuTextMetrics,
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
                self.layout_popup(text_metrics);
            }
            return LinuxMenuInputResult::Redraw;
        }

        if let Some(row_index) = self
            .popup_rows
            .iter()
            .position(|row| row.bounds.contains(point))
        {
            return self.activate_row(row_index, text_metrics);
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
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> LinuxMenuInputResult {
        if matches!(key, Key::Named(NamedKey::F10)) {
            if self.is_open() {
                self.close();
            } else if let Some(root) = self.roots.iter().position(|root| root.enabled) {
                self.open_root = Some(root);
                self.submenu_path.clear();
                self.keyboard_row = first_enabled_row(self.current_menu());
                self.layout_popup(text_metrics);
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
                    self.layout_popup(text_metrics);
                }
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.move_root(-1, text_metrics);
                LinuxMenuInputResult::Redraw
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.move_root(1, text_metrics);
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
                .map(|row| self.activate_row(row, text_metrics))
                .unwrap_or(LinuxMenuInputResult::Redraw),
            _ => LinuxMenuInputResult::Redraw,
        }
    }

    pub(crate) fn proof_command(
        &mut self,
        text_metrics: &dyn LinuxMenuTextMetrics,
    ) -> Option<Command> {
        let root = self.roots.iter().position(|root| root.enabled)?;
        self.open_root = Some(root);
        self.submenu_path.clear();
        self.layout_popup(text_metrics);
        let row = first_enabled_row(self.current_menu())?;
        let result = self.activate_row(row, text_metrics);
        match result {
            LinuxMenuInputResult::Command(command) => Some(command),
            _ => None,
        }
    }

    pub(crate) fn open_for_capture(&mut self, text_metrics: &dyn LinuxMenuTextMetrics) -> bool {
        let Some(root) = self.roots.iter().position(|root| root.enabled) else {
            return false;
        };
        self.open_root = Some(root);
        self.submenu_path.clear();
        self.keyboard_row = first_enabled_row(self.current_menu());
        self.layout_popup(text_metrics);
        true
    }

    pub(crate) fn draw(&self, canvas: &mut dyn LinuxMenuCanvas, palette: NativeDrawPalette) {
        canvas.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: LINUX_MENU_BAR_HEIGHT,
            },
            palette.surface_raised,
        );
        canvas.fill_rect(
            Rect {
                x: 0,
                y: LINUX_MENU_BAR_HEIGHT - 1,
                width: self.width,
                height: 1,
            },
            palette.border,
        );

        for (root_index, root) in self.roots.iter().enumerate() {
            if self.open_root == Some(root_index) || self.hovered_root == Some(root_index) {
                canvas.fill_round_rect(
                    root.bounds,
                    6.0,
                    if self.open_root == Some(root_index) {
                        palette.control
                    } else {
                        palette.surface
                    },
                );
            }
            let label = menu_item_label_and_enabled(&self.menu.items[root.item_index])
                .map(|(label, _)| label)
                .unwrap_or_default();
            canvas.draw_text(
                label,
                root.bounds,
                if root.enabled {
                    palette.primary_text
                } else {
                    palette.disabled_text
                },
                LinuxMenuTextPlacement::Center,
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
        canvas.fill_round_rect(shadow, POPUP_RADIUS, Color::rgba(0, 0, 0, 56));
        canvas.fill_round_rect(popup, POPUP_RADIUS, palette.surface_raised);
        canvas.stroke_round_rect(popup, POPUP_RADIUS, palette.border, 1.0);

        for (row_index, row) in self.popup_rows.iter().enumerate() {
            let Some(item) = menu.items.get(row.item_index) else {
                continue;
            };
            if matches!(item, MenuItemSpec::Separator) {
                canvas.fill_rect(
                    Rect {
                        x: row.bounds.x + 10,
                        y: row.bounds.y + row.bounds.height / 2,
                        width: (row.bounds.width - 20).max(1),
                        height: 1,
                    },
                    palette.border,
                );
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
                canvas.fill_round_rect(highlight, 6.0, palette.control);
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
            canvas.draw_text(
                label,
                label_bounds,
                text_color,
                LinuxMenuTextPlacement::Start,
            );
            match item {
                MenuItemSpec::Command {
                    checked,
                    accelerator,
                    ..
                } => {
                    if *checked {
                        canvas.draw_text(
                            "✓",
                            Rect {
                                x: row.bounds.x + 8,
                                y: row.bounds.y,
                                width: 22,
                                height: row.bounds.height,
                            },
                            text_color,
                            LinuxMenuTextPlacement::Center,
                        );
                    }
                    if let Some(accelerator) = accelerator {
                        canvas.draw_text(
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
                            LinuxMenuTextPlacement::End,
                        );
                    }
                }
                MenuItemSpec::Submenu { .. } => {
                    canvas.draw_text(
                        "›",
                        Rect {
                            x: row.bounds.x + row.bounds.width - 28,
                            y: row.bounds.y,
                            width: 18,
                            height: row.bounds.height,
                        },
                        text_color,
                        LinuxMenuTextPlacement::Center,
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

    fn layout_popup(&mut self, text_metrics: &dyn LinuxMenuTextMetrics) {
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
                let label_width = text_metrics.measure_menu_text(label).0;
                let accelerator_width = match item {
                    MenuItemSpec::Command {
                        accelerator: Some(accelerator),
                        ..
                    } => {
                        text_metrics
                            .measure_menu_text(&linux_accelerator_label(*accelerator))
                            .0
                    }
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
        text_metrics: &dyn LinuxMenuTextMetrics,
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
                self.layout_popup(text_metrics);
                LinuxMenuInputResult::Redraw
            }
            _ => LinuxMenuInputResult::Redraw,
        }
    }

    fn move_root(&mut self, offset: isize, text_metrics: &dyn LinuxMenuTextMetrics) {
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
        self.layout_popup(text_metrics);
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

#[cfg(feature = "accessibility")]
fn menu_item_author_id(item: &MenuItemSpec, prefix: &str, index: usize) -> String {
    let declared = match item {
        MenuItemSpec::Command { id, .. } | MenuItemSpec::Submenu { id, .. } => id.as_deref(),
        MenuItemSpec::Separator => None,
    };
    declared
        .map(|id| format!("zsui-menu-{id}"))
        .unwrap_or_else(|| format!("zsui-menu-{prefix}-{index}"))
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

#[cfg(feature = "linux-direct")]
fn menu_font_description() -> pango::FontDescription {
    let configured = std::env::var("ZSUI_LINUX_UI_FONT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Ubuntu Sans 11".to_string());
    pango::FontDescription::from_string(&configured)
}

#[cfg(feature = "linux-direct")]
impl LinuxMenuTextMetrics for pango::Context {
    fn measure_menu_text(&self, text: &str) -> (i32, i32) {
        use pango::prelude::*;
        let layout = pango::Layout::new(self);
        layout.set_font_description(Some(&menu_font_description()));
        layout.set_text(text);
        layout.pixel_size()
    }
}

#[cfg(feature = "linux-direct")]
pub(crate) struct LinuxCairoMenuCanvas<'a> {
    context: &'a Context,
    pango_context: &'a pango::Context,
}

#[cfg(feature = "linux-direct")]
impl<'a> LinuxCairoMenuCanvas<'a> {
    pub(crate) const fn new(context: &'a Context, pango_context: &'a pango::Context) -> Self {
        Self {
            context,
            pango_context,
        }
    }
}

#[cfg(feature = "linux-direct")]
impl LinuxMenuCanvas for LinuxCairoMenuCanvas<'_> {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        set_source(self.context, color);
        self.context.rectangle(
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.width.max(0)),
            f64::from(rect.height.max(0)),
        );
        let _ = self.context.fill();
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: f64, color: Color) {
        draw_round_rect(self.context, rect, radius);
        set_source(self.context, color);
        let _ = self.context.fill();
    }

    fn stroke_round_rect(&mut self, rect: Rect, radius: f64, color: Color, width: f32) {
        draw_round_rect(self.context, rect, radius);
        set_source(self.context, color);
        self.context.set_line_width(f64::from(width.max(0.5)));
        let _ = self.context.stroke();
    }

    fn draw_text(
        &mut self,
        text: &str,
        bounds: Rect,
        color: Color,
        placement: LinuxMenuTextPlacement,
    ) {
        use pango::prelude::*;
        let layout = pango::Layout::new(self.pango_context);
        layout.set_font_description(Some(&menu_font_description()));
        layout.set_text(text);
        let (width, height) = layout.pixel_size();
        let x = match placement {
            LinuxMenuTextPlacement::Start => bounds.x,
            LinuxMenuTextPlacement::Center => bounds.x + (bounds.width - width).max(0) / 2,
            LinuxMenuTextPlacement::End => bounds.x + (bounds.width - width).max(0),
        };
        let y = bounds.y + (bounds.height - height).max(0) / 2;
        set_source(self.context, color);
        self.context.move_to(f64::from(x), f64::from(y));
        pangocairo::functions::show_layout(self.context, &layout);
    }
}

#[cfg(feature = "linux-direct")]
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

#[cfg(feature = "linux-direct")]
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
