use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, HorizontalAlign, MenuItemSpec, MenuSpec, NativeDrawCommand, NativeDrawFill,
    NativeDrawIconCommand, NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, Point, Rect,
    SemanticTextStyle, ZsAccelerator, ZsAcceleratorKey, ZsIcon, ZsPlatformStyle,
};

pub type ZsMenuFlyoutPlatformStyle = ZsPlatformStyle;

/// Stable location of one visible MenuFlyout item.
///
/// `parent` is `None` for the root menu and contains the root submenu index
/// for one open child menu. Deeper submenu stacks remain an explicit 0.2 gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsMenuFlyoutPath {
    pub parent: Option<usize>,
    pub item: usize,
}

impl ZsMenuFlyoutPath {
    pub const fn root(item: usize) -> Self {
        Self { parent: None, item }
    }

    pub const fn child(parent: usize, item: usize) -> Self {
        Self {
            parent: Some(parent),
            item,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsMenuFlyoutState {
    pub open: bool,
    pub target: crate::WidgetId,
    pub highlighted: Option<ZsMenuFlyoutPath>,
    pub open_submenu: Option<usize>,
}

impl ZsMenuFlyoutState {
    pub fn first_enabled(&self, menu: &MenuSpec) -> Option<ZsMenuFlyoutPath> {
        menu_flyout_enabled_paths(menu, self.open_submenu)
            .into_iter()
            .next()
    }

    pub fn last_enabled(&self, menu: &MenuSpec) -> Option<ZsMenuFlyoutPath> {
        menu_flyout_enabled_paths(menu, self.open_submenu)
            .into_iter()
            .next_back()
    }

    pub fn relative_highlight(&self, menu: &MenuSpec, offset: isize) -> Option<ZsMenuFlyoutPath> {
        let paths = menu_flyout_enabled_paths(menu, self.open_submenu).collect::<Vec<_>>();
        if paths.is_empty() {
            return None;
        }
        let last = paths.len().saturating_sub(1);
        let current = self
            .highlighted
            .and_then(|highlighted| paths.iter().position(|path| *path == highlighted));
        let index = match (current, offset.cmp(&0)) {
            (Some(index), std::cmp::Ordering::Less) => index.saturating_sub(offset.unsigned_abs()),
            (Some(index), std::cmp::Ordering::Greater) => {
                index.saturating_add(offset as usize).min(last)
            }
            (Some(index), std::cmp::Ordering::Equal) => index,
            (None, std::cmp::Ordering::Less) => last,
            (None, _) => 0,
        };
        paths.get(index).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsMenuFlyoutMetrics {
    pub minimum_width: Dp,
    pub maximum_width: Dp,
    pub viewport_margin: Dp,
    pub target_gap: Dp,
    pub surface_padding: Dp,
    pub row_height: Dp,
    pub separator_height: Dp,
    pub horizontal_padding: Dp,
    pub indicator_width: Dp,
    pub accelerator_gap: Dp,
    pub submenu_width: Dp,
    pub icon_size: Dp,
    pub surface_radius: Dp,
    pub row_radius: Dp,
    pub shadow_offset: Dp,
    pub shadow_alpha: u8,
}

impl ZsMenuFlyoutMetrics {
    pub const fn for_platform(platform: ZsMenuFlyoutPlatformStyle) -> Self {
        crate::platform_component_profile::PlatformMenuFlyoutProfile::for_platform(platform).metrics
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsMenuFlyoutRowKind {
    Command { checked: bool },
    Separator,
    Submenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsMenuFlyoutRowRenderPlan {
    pub path: ZsMenuFlyoutPath,
    pub bounds: Rect,
    pub indicator_bounds: Rect,
    pub label_bounds: Rect,
    pub accelerator_bounds: Rect,
    pub submenu_bounds: Rect,
    pub kind: ZsMenuFlyoutRowKind,
    pub enabled: bool,
    pub highlighted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsMenuFlyoutRenderPlan {
    pub viewport: Rect,
    pub target: Rect,
    pub surfaces: Vec<Rect>,
    pub rows: Vec<ZsMenuFlyoutRowRenderPlan>,
    pub surface_radius: i32,
    pub row_radius: i32,
    pub shadow_offset: i32,
    pub shadow_alpha: u8,
    pub platform: ZsMenuFlyoutPlatformStyle,
}

pub fn zs_menu_flyout_render_plan(
    viewport: Rect,
    target: Rect,
    menu: &MenuSpec,
    highlighted: Option<ZsMenuFlyoutPath>,
    open_submenu: Option<usize>,
    platform: ZsMenuFlyoutPlatformStyle,
    dpi: Dpi,
) -> ZsMenuFlyoutRenderPlan {
    let metrics = ZsMenuFlyoutMetrics::for_platform(platform);
    let margin = metrics.viewport_margin.to_px(dpi).round_i32().max(0);
    let gap = metrics.target_gap.to_px(dpi).round_i32().max(0);
    let root_size = menu_surface_size(viewport, menu, metrics, platform, dpi);
    let root = place_root_surface(viewport, target, root_size, margin, gap);
    let mut surfaces = vec![root];
    let mut rows = menu_rows(root, menu, None, highlighted, metrics, platform, dpi);

    if let Some(parent) = open_submenu {
        if let Some(MenuItemSpec::Submenu {
            enabled: true,
            menu: submenu,
            ..
        }) = menu.items.get(parent)
        {
            if let Some(parent_row) = rows
                .iter()
                .find(|row| row.path == ZsMenuFlyoutPath::root(parent))
            {
                let child_size = menu_surface_size(viewport, submenu, metrics, platform, dpi);
                let child =
                    place_submenu_surface(viewport, parent_row.bounds, child_size, margin, gap);
                surfaces.push(child);
                rows.extend(menu_rows(
                    child,
                    submenu,
                    Some(parent),
                    highlighted,
                    metrics,
                    platform,
                    dpi,
                ));
            }
        }
    }

    ZsMenuFlyoutRenderPlan {
        viewport,
        target,
        surfaces,
        rows,
        surface_radius: metrics.surface_radius.to_px(dpi).round_i32().max(0),
        row_radius: metrics.row_radius.to_px(dpi).round_i32().max(0),
        shadow_offset: metrics.shadow_offset.to_px(dpi).round_i32().max(0),
        shadow_alpha: metrics.shadow_alpha,
        platform,
    }
}

pub fn zs_menu_flyout_native_draw_plan(
    plan: &ZsMenuFlyoutRenderPlan,
    menu: &MenuSpec,
) -> NativeDrawPlan {
    let mut output = NativeDrawPlan::default();
    for surface in &plan.surfaces {
        if plan.shadow_alpha > 0 {
            output.push(NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: surface.x.saturating_add(plan.shadow_offset),
                    y: surface.y.saturating_add(plan.shadow_offset),
                    ..*surface
                },
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::PrimaryText,
                    alpha: plan.shadow_alpha,
                },
                radius: plan.surface_radius,
            });
        }
        output.push(NativeDrawCommand::RoundRect {
            rect: *surface,
            fill: NativeDrawFill::Role(ColorRole::SurfaceRaised),
            stroke: Some(NativeDrawFill::Role(ColorRole::Border)),
            radius: plan.surface_radius,
        });
    }

    for row in &plan.rows {
        let Some(item) = menu_flyout_item(menu, row.path) else {
            continue;
        };
        if row.highlighted && row.enabled {
            output.push(NativeDrawCommand::RoundFill {
                rect: row.bounds,
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    alpha: 28,
                },
                radius: plan.row_radius,
            });
        }
        if row.kind == ZsMenuFlyoutRowKind::Separator {
            output.push(NativeDrawCommand::FillRect {
                rect: Rect {
                    x: row.label_bounds.x,
                    y: row.bounds.y.saturating_add(row.bounds.height / 2),
                    width: row
                        .submenu_bounds
                        .x
                        .saturating_add(row.submenu_bounds.width)
                        .saturating_sub(row.label_bounds.x)
                        .max(1),
                    height: 1,
                },
                fill: NativeDrawFill::Role(ColorRole::Border),
            });
            continue;
        }

        let text_color = if row.enabled {
            ColorRole::PrimaryText
        } else {
            ColorRole::DisabledText
        };
        let mut text_style = SemanticTextStyle::body();
        text_style.color = text_color;
        match item {
            MenuItemSpec::Command {
                label,
                checked,
                accelerator,
                ..
            } => {
                if *checked {
                    output.push(NativeDrawCommand::Icon(
                        NativeDrawIconCommand::new(
                            ZsIcon::Check,
                            row.indicator_bounds,
                            NativeIconColorMode::ThemeAware,
                        )
                        .with_color(text_color),
                    ));
                }
                output.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    row.label_bounds,
                    text_style,
                )));
                if let Some(accelerator) = accelerator {
                    let mut accelerator_style = text_style;
                    accelerator_style.horizontal_align = HorizontalAlign::End;
                    accelerator_style.color = if row.enabled {
                        ColorRole::SecondaryText
                    } else {
                        ColorRole::DisabledText
                    };
                    output.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                        menu_accelerator_label(*accelerator, plan.platform),
                        row.accelerator_bounds,
                        accelerator_style,
                    )));
                }
            }
            MenuItemSpec::Submenu { label, .. } => {
                output.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                    label,
                    row.label_bounds,
                    text_style,
                )));
                output.push(NativeDrawCommand::Icon(
                    NativeDrawIconCommand::new(
                        ZsIcon::ChevronRight,
                        row.submenu_bounds,
                        NativeIconColorMode::ThemeAware,
                    )
                    .with_color(text_color),
                ));
            }
            MenuItemSpec::Separator => {}
        }
    }
    output
}

pub(crate) fn menu_flyout_item(menu: &MenuSpec, path: ZsMenuFlyoutPath) -> Option<&MenuItemSpec> {
    match path.parent {
        None => menu.items.get(path.item),
        Some(parent) => match menu.items.get(parent)? {
            MenuItemSpec::Submenu { menu, .. } => menu.items.get(path.item),
            _ => None,
        },
    }
}

pub(crate) fn menu_flyout_command(
    menu: &MenuSpec,
    path: ZsMenuFlyoutPath,
) -> Option<crate::Command> {
    match menu_flyout_item(menu, path)? {
        MenuItemSpec::Command {
            command,
            enabled: true,
            ..
        } => Some(command.clone()),
        _ => None,
    }
}

pub(crate) fn menu_flyout_submenu_index(menu: &MenuSpec, path: ZsMenuFlyoutPath) -> Option<usize> {
    if path.parent.is_none()
        && matches!(
            menu.items.get(path.item),
            Some(MenuItemSpec::Submenu { enabled: true, .. })
        )
    {
        Some(path.item)
    } else {
        None
    }
}

fn menu_flyout_enabled_paths(
    menu: &MenuSpec,
    open_submenu: Option<usize>,
) -> impl DoubleEndedIterator<Item = ZsMenuFlyoutPath> + '_ {
    let (items, parent) = open_submenu
        .and_then(|parent| match menu.items.get(parent) {
            Some(MenuItemSpec::Submenu { menu, .. }) => Some((&menu.items[..], Some(parent))),
            _ => None,
        })
        .unwrap_or((&menu.items[..], None));
    items.iter().enumerate().filter_map(move |(index, item)| {
        let enabled = match item {
            MenuItemSpec::Command { enabled, .. } => *enabled,
            MenuItemSpec::Submenu { enabled, .. } => *enabled && parent.is_none(),
            MenuItemSpec::Separator => false,
        };
        enabled.then_some(ZsMenuFlyoutPath {
            parent,
            item: index,
        })
    })
}

fn menu_surface_size(
    viewport: Rect,
    menu: &MenuSpec,
    metrics: ZsMenuFlyoutMetrics,
    platform: ZsMenuFlyoutPlatformStyle,
    dpi: Dpi,
) -> Point {
    let padding = metrics.surface_padding.to_px(dpi).round_i32().max(0);
    let horizontal = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let indicator = metrics.indicator_width.to_px(dpi).round_i32().max(0);
    let accelerator_gap = metrics.accelerator_gap.to_px(dpi).round_i32().max(0);
    let submenu = metrics.submenu_width.to_px(dpi).round_i32().max(0);
    let text_unit = Dp::new(7.0).to_px(dpi).round_i32().max(1);
    let label_width = menu
        .items
        .iter()
        .filter_map(menu_item_label)
        .map(|label| crate::widget_render::zs_estimated_text_width_px(label, text_unit))
        .max()
        .unwrap_or_default();
    let accelerator_width = menu
        .items
        .iter()
        .filter_map(|item| match item {
            MenuItemSpec::Command {
                accelerator: Some(value),
                ..
            } => Some(menu_accelerator_label(*value, platform)),
            _ => None,
        })
        .map(|label| crate::widget_render::zs_estimated_text_width_px(&label, text_unit))
        .max()
        .unwrap_or_default();
    let has_submenu = menu
        .items
        .iter()
        .any(|item| matches!(item, MenuItemSpec::Submenu { .. }));
    let content_width = indicator
        .saturating_add(label_width)
        .saturating_add(
            (accelerator_width > 0)
                .then_some(accelerator_gap)
                .unwrap_or_default(),
        )
        .saturating_add(accelerator_width)
        .saturating_add(has_submenu.then_some(submenu).unwrap_or_default())
        .saturating_add(horizontal.saturating_mul(2));
    let margin = metrics.viewport_margin.to_px(dpi).round_i32().max(0);
    let minimum = metrics.minimum_width.to_px(dpi).round_i32().max(1);
    let maximum = metrics.maximum_width.to_px(dpi).round_i32().max(minimum);
    let width = content_width
        .saturating_add(padding.saturating_mul(2))
        .max(minimum)
        .min(maximum)
        .min(
            viewport
                .width
                .saturating_sub(margin.saturating_mul(2))
                .max(1),
        );
    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let separator_height = metrics.separator_height.to_px(dpi).round_i32().max(1);
    let height = menu
        .items
        .iter()
        .map(|item| {
            if matches!(item, MenuItemSpec::Separator) {
                separator_height
            } else {
                row_height
            }
        })
        .fold(padding.saturating_mul(2), i32::saturating_add)
        .min(
            viewport
                .height
                .saturating_sub(margin.saturating_mul(2))
                .max(1),
        );
    Point {
        x: width,
        y: height,
    }
}

fn menu_rows(
    surface: Rect,
    menu: &MenuSpec,
    parent: Option<usize>,
    highlighted: Option<ZsMenuFlyoutPath>,
    metrics: ZsMenuFlyoutMetrics,
    platform: ZsMenuFlyoutPlatformStyle,
    dpi: Dpi,
) -> Vec<ZsMenuFlyoutRowRenderPlan> {
    let surface_padding = metrics.surface_padding.to_px(dpi).round_i32().max(0);
    let horizontal = metrics.horizontal_padding.to_px(dpi).round_i32().max(0);
    let indicator_width = metrics.indicator_width.to_px(dpi).round_i32().max(0);
    let submenu_width = menu
        .items
        .iter()
        .any(|item| matches!(item, MenuItemSpec::Submenu { .. }))
        .then(|| metrics.submenu_width.to_px(dpi).round_i32().max(0))
        .unwrap_or_default();
    let text_unit = Dp::new(7.0).to_px(dpi).round_i32().max(1);
    let accelerator_width = menu
        .items
        .iter()
        .filter_map(|item| match item {
            MenuItemSpec::Command {
                accelerator: Some(value),
                ..
            } => Some(menu_accelerator_label(*value, platform)),
            _ => None,
        })
        .map(|label| crate::widget_render::zs_estimated_text_width_px(&label, text_unit))
        .max()
        .unwrap_or_default();
    let accelerator_gap = (accelerator_width > 0)
        .then(|| metrics.accelerator_gap.to_px(dpi).round_i32().max(0))
        .unwrap_or_default();
    let row_height = metrics.row_height.to_px(dpi).round_i32().max(1);
    let separator_height = metrics.separator_height.to_px(dpi).round_i32().max(1);
    let icon_size = metrics.icon_size.to_px(dpi).round_i32().max(1);
    let mut y = surface.y.saturating_add(surface_padding);
    menu.items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let height = if matches!(item, MenuItemSpec::Separator) {
                separator_height
            } else {
                row_height
            };
            let path = ZsMenuFlyoutPath {
                parent,
                item: index,
            };
            let bounds = Rect {
                x: surface.x.saturating_add(surface_padding),
                y,
                width: surface
                    .width
                    .saturating_sub(surface_padding.saturating_mul(2))
                    .max(1),
                height,
            };
            y = y.saturating_add(height);
            let indicator_bounds = Rect {
                x: bounds.x.saturating_add(horizontal),
                y: bounds
                    .y
                    .saturating_add((height.saturating_sub(icon_size)) / 2),
                width: icon_size,
                height: icon_size,
            };
            let label_x = bounds
                .x
                .saturating_add(horizontal)
                .saturating_add(indicator_width);
            let submenu_bounds = Rect {
                x: bounds
                    .x
                    .saturating_add(bounds.width)
                    .saturating_sub(horizontal)
                    .saturating_sub(submenu_width),
                y: bounds
                    .y
                    .saturating_add((height.saturating_sub(icon_size)) / 2),
                width: submenu_width,
                height: icon_size,
            };
            let accelerator_bounds = Rect {
                x: submenu_bounds.x.saturating_sub(accelerator_width),
                y: bounds.y,
                width: accelerator_width,
                height,
            };
            let label_bounds = Rect {
                x: label_x,
                y: bounds.y,
                width: accelerator_bounds
                    .x
                    .saturating_sub(accelerator_gap)
                    .saturating_sub(label_x)
                    .max(0),
                height,
            };
            let (kind, enabled) = match item {
                MenuItemSpec::Command {
                    checked, enabled, ..
                } => (ZsMenuFlyoutRowKind::Command { checked: *checked }, *enabled),
                MenuItemSpec::Separator => (ZsMenuFlyoutRowKind::Separator, false),
                MenuItemSpec::Submenu { enabled, .. } => {
                    (ZsMenuFlyoutRowKind::Submenu, *enabled && parent.is_none())
                }
            };
            ZsMenuFlyoutRowRenderPlan {
                path,
                bounds,
                indicator_bounds,
                label_bounds,
                accelerator_bounds,
                submenu_bounds,
                kind,
                enabled,
                highlighted: highlighted == Some(path),
            }
        })
        .collect()
}

fn place_root_surface(viewport: Rect, target: Rect, size: Point, margin: i32, gap: i32) -> Rect {
    let below = target.y.saturating_add(target.height).saturating_add(gap);
    let above = target.y.saturating_sub(gap).saturating_sub(size.y);
    let maximum_y = viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(margin)
        .saturating_sub(size.y)
        .max(viewport.y.saturating_add(margin));
    let y = if below <= maximum_y { below } else { above }
        .clamp(viewport.y.saturating_add(margin), maximum_y);
    let maximum_x = viewport
        .x
        .saturating_add(viewport.width)
        .saturating_sub(margin)
        .saturating_sub(size.x)
        .max(viewport.x.saturating_add(margin));
    Rect {
        x: target.x.clamp(viewport.x.saturating_add(margin), maximum_x),
        y,
        width: size.x,
        height: size.y,
    }
}

fn place_submenu_surface(
    viewport: Rect,
    parent_row: Rect,
    size: Point,
    margin: i32,
    gap: i32,
) -> Rect {
    let right = parent_row
        .x
        .saturating_add(parent_row.width)
        .saturating_add(gap);
    let maximum_x = viewport
        .x
        .saturating_add(viewport.width)
        .saturating_sub(margin)
        .saturating_sub(size.x)
        .max(viewport.x.saturating_add(margin));
    let left = parent_row.x.saturating_sub(gap).saturating_sub(size.x);
    let x = if right <= maximum_x { right } else { left }
        .clamp(viewport.x.saturating_add(margin), maximum_x);
    let maximum_y = viewport
        .y
        .saturating_add(viewport.height)
        .saturating_sub(margin)
        .saturating_sub(size.y)
        .max(viewport.y.saturating_add(margin));
    Rect {
        x,
        y: parent_row
            .y
            .clamp(viewport.y.saturating_add(margin), maximum_y),
        width: size.x,
        height: size.y,
    }
}

fn menu_item_label(item: &MenuItemSpec) -> Option<&str> {
    match item {
        MenuItemSpec::Command { label, .. } | MenuItemSpec::Submenu { label, .. } => Some(label),
        MenuItemSpec::Separator => None,
    }
}

fn menu_accelerator_label(
    accelerator: ZsAccelerator,
    platform: ZsMenuFlyoutPlatformStyle,
) -> String {
    let key = accelerator.key().label();
    match platform {
        ZsPlatformStyle::Macos => {
            let mut value = String::new();
            if accelerator.uses_primary() || accelerator.uses_super() {
                value.push('⌘');
            }
            if accelerator.uses_alt() {
                value.push('⌥');
            }
            if accelerator.uses_shift() {
                value.push('⇧');
            }
            value.push_str(&macos_key_label(accelerator.key(), &key));
            value
        }
        ZsPlatformStyle::Windows | ZsPlatformStyle::Gtk => {
            let mut parts = Vec::new();
            if accelerator.uses_primary() {
                parts.push("Ctrl".to_string());
            }
            if accelerator.uses_super() {
                parts.push("Super".to_string());
            }
            if accelerator.uses_alt() {
                parts.push("Alt".to_string());
            }
            if accelerator.uses_shift() {
                parts.push("Shift".to_string());
            }
            parts.push(key);
            parts.join("+")
        }
    }
}

fn macos_key_label(key: ZsAcceleratorKey, fallback: &str) -> String {
    match key {
        ZsAcceleratorKey::Enter => "↩".to_string(),
        ZsAcceleratorKey::Escape => "⎋".to_string(),
        ZsAcceleratorKey::Tab => "⇥".to_string(),
        ZsAcceleratorKey::Backspace => "⌫".to_string(),
        ZsAcceleratorKey::Delete => "⌦".to_string(),
        ZsAcceleratorKey::Up => "↑".to_string(),
        ZsAcceleratorKey::Down => "↓".to_string(),
        ZsAcceleratorKey::Left => "←".to_string(),
        ZsAcceleratorKey::Right => "→".to_string(),
        _ => fallback.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    fn menu() -> MenuSpec {
        MenuSpec::new()
            .item("Open / 打开", Command::custom("open"))
            .separator()
            .submenu(
                "Recent / 最近",
                MenuSpec::new()
                    .item("One", Command::custom("one"))
                    .item("Unavailable", Command::custom("disabled")),
            )
    }

    #[test]
    fn platform_metrics_produce_distinct_native_menu_geometry() {
        let viewport = Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let target = Rect {
            x: 40,
            y: 40,
            width: 120,
            height: 32,
        };
        let menu = menu();
        let windows = zs_menu_flyout_render_plan(
            viewport,
            target,
            &menu,
            None,
            None,
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );
        let macos = zs_menu_flyout_render_plan(
            viewport,
            target,
            &menu,
            None,
            None,
            ZsPlatformStyle::Macos,
            Dpi::standard(),
        );
        let gtk = zs_menu_flyout_render_plan(
            viewport,
            target,
            &menu,
            None,
            None,
            ZsPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_ne!(windows.rows[0].bounds.height, macos.rows[0].bounds.height);
        assert_ne!(macos.rows[0].bounds.height, gtk.rows[0].bounds.height);
        assert_ne!(windows.surface_radius, gtk.surface_radius);
    }

    #[test]
    fn submenu_opens_one_native_child_surface_and_resolves_commands() {
        let menu = menu();
        let plan = zs_menu_flyout_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 480,
            },
            Rect {
                x: 20,
                y: 20,
                width: 100,
                height: 30,
            },
            &menu,
            Some(ZsMenuFlyoutPath::child(2, 0)),
            Some(2),
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );

        assert_eq!(plan.surfaces.len(), 2);
        assert_eq!(
            menu_flyout_command(&menu, ZsMenuFlyoutPath::child(2, 0)),
            Some(Command::custom("one"))
        );
    }

    #[test]
    fn macos_accelerators_use_menu_glyphs() {
        assert_eq!(
            menu_accelerator_label(
                ZsAccelerator::primary_character('o').shifted(),
                ZsPlatformStyle::Macos,
            ),
            "⌘⇧O"
        );
    }

    #[test]
    fn accelerator_column_preserves_the_longest_bilingual_label() {
        let mut menu = MenuSpec::new();
        menu.items.push(
            MenuItemSpec::command("Save", Command::custom("save"))
                .accelerator(ZsAccelerator::primary_character('s')),
        );
        menu.items.push(
            MenuItemSpec::command("自动保存 / Auto save", Command::custom("autosave"))
                .checked(true),
        );
        menu.items.push(MenuItemSpec::Submenu {
            id: None,
            label: "更多 / More".to_string(),
            enabled: true,
            menu: MenuSpec::new().item("Copy", Command::custom("copy")),
        });
        let plan = zs_menu_flyout_render_plan(
            Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            Rect {
                x: 20,
                y: 20,
                width: 100,
                height: 32,
            },
            &menu,
            None,
            None,
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );
        let expected = crate::widget_render::zs_estimated_text_width_px(
            "自动保存 / Auto save",
            Dp::new(7.0).to_px(Dpi::standard()).round_i32(),
        );

        assert!(plan.rows[1].label_bounds.width >= expected);
        assert!(plan.rows[0].accelerator_bounds.width < 96);
    }
}
