use serde::{Deserialize, Serialize};

use crate::{
    zs_toggle_render_plan, ColorRole, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill,
    NativeDrawPlan, NativeDrawTextCommand, Point, Rect, SemanticTextStyle, TextRole, TextWeight,
    TextWrap, UiRect, VerticalAlign,
};

pub const ZS_SHELL_BASE_W: i32 = 1100;
pub const ZS_SHELL_BASE_H: i32 = 740;
pub const ZS_SHELL_NAV_W: i32 = 236;
pub const ZS_SHELL_TOP_H: i32 = 84;
pub const ZS_SHELL_NAV_Y: i32 = 72;
pub const ZS_SHELL_CONTENT_GAP: i32 = 28;
pub const ZS_SHELL_CONTENT_TOP_GAP: i32 = 16;
pub const ZS_SHELL_SCROLL_BAR_W: i32 = 3;
pub const ZS_SHELL_SCROLL_BAR_W_ACTIVE: i32 = 5;
pub const ZS_SHELL_SCROLL_BAR_MARGIN: i32 = 3;
pub const ZS_SHELL_FORM_HEADER_H: i32 = 52;
pub const ZS_SHELL_FORM_ROW_H: i32 = 32;
pub const ZS_SHELL_FORM_ROW_GAP: i32 = 8;
pub const ZS_SHELL_FORM_SECTION_GAP: i32 = 12;
pub const ZS_SHELL_FORM_SECTION_PAD: i32 = 18;
pub const ZS_SHELL_FORM_BOTTOM_SAFE_H: i32 = 24;
pub const ZS_SHELL_VIEWPORT_MASK_H: i32 = 14;

pub fn zs_shell_scale(value: i32, dpi: Dpi) -> i32 {
    let dpi = dpi.0.round().max(96.0) as i64;
    (((value as i64) * dpi) + 48) as i32 / 96
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellLayoutSpec {
    pub id: String,
    pub title: String,
    pub app_title: String,
    pub selected_nav_id: Option<String>,
    pub hovered_nav_id: Option<String>,
    pub nav_items: Vec<ZsShellNavItemSpec>,
    pub cards: Vec<ZsShellGroupCardSpec>,
    pub action_area: ZsShellActionAreaSpec,
    pub scroll_y: i32,
    pub scrollbar_visible: bool,
    pub scrollbar_dragging: bool,
}

impl ZsShellLayoutSpec {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            app_title: "设置".to_string(),
            selected_nav_id: None,
            hovered_nav_id: None,
            nav_items: Vec::new(),
            cards: Vec::new(),
            action_area: ZsShellActionAreaSpec::default(),
            scroll_y: 0,
            scrollbar_visible: true,
            scrollbar_dragging: false,
        }
    }

    pub fn app_title(mut self, app_title: impl Into<String>) -> Self {
        self.app_title = app_title.into();
        self
    }

    pub fn selected_nav(mut self, id: impl Into<String>) -> Self {
        self.selected_nav_id = Some(id.into());
        self
    }

    pub fn hovered_nav(mut self, id: impl Into<String>) -> Self {
        self.hovered_nav_id = Some(id.into());
        self
    }

    pub fn nav_item(mut self, item: ZsShellNavItemSpec) -> Self {
        self.nav_items.push(item);
        self
    }

    pub fn card(mut self, card: ZsShellGroupCardSpec) -> Self {
        self.cards.push(card);
        self
    }

    pub fn action_area(mut self, action_area: ZsShellActionAreaSpec) -> Self {
        self.action_area = action_area;
        self
    }

    pub fn scroll_y(mut self, scroll_y: i32) -> Self {
        self.scroll_y = scroll_y.max(0);
        self
    }

    pub fn scrollbar(mut self, visible: bool, dragging: bool) -> Self {
        self.scrollbar_visible = visible;
        self.scrollbar_dragging = dragging;
        self
    }

    pub fn scroll_layout(&self, bounds: Rect, dpi: Dpi, active: bool) -> ZsShellScrollLayout {
        let window = rect_to_ui(bounds);
        let bar_width = if active {
            zs_shell_scale(ZS_SHELL_SCROLL_BAR_W_ACTIVE, dpi)
        } else {
            zs_shell_scale(ZS_SHELL_SCROLL_BAR_W, dpi)
        };
        zs_shell_scroll_layout_for_window(window, self.content_total_h(window, dpi), bar_width, dpi)
    }

    pub fn max_scroll(&self, bounds: Rect, dpi: Dpi) -> i32 {
        self.scroll_layout(bounds, dpi, false).max_scroll()
    }

    pub fn layout_plan(&self, bounds: Rect, dpi: Dpi) -> ZsShellLayoutPlan {
        self.layout_plan_with_metrics(bounds, dpi, ZsShellLayoutMetrics::default())
    }

    pub fn layout_plan_with_metrics(
        &self,
        bounds: Rect,
        dpi: Dpi,
        _metrics: ZsShellLayoutMetrics,
    ) -> ZsShellLayoutPlan {
        let mut regions = Vec::new();
        let window = rect_to_ui(bounds);
        let chrome = zs_shell_chrome_render_plan(window, dpi);

        regions.push(ZsShellLayoutRegion::new(
            self.id.clone(),
            ZsShellLayoutRegionKind::Root,
            bounds,
        ));
        regions.push(ZsShellLayoutRegion::new(
            "navigation",
            ZsShellLayoutRegionKind::NavigationPane,
            ui_to_rect(chrome.nav_rect),
        ));
        regions.push(ZsShellLayoutRegion::new(
            "content",
            ZsShellLayoutRegionKind::ContentPane,
            ui_to_rect(zs_shell_viewport_rect_for_window(window, dpi)),
        ));
        regions.push(ZsShellLayoutRegion::new(
            "content.header",
            ZsShellLayoutRegionKind::ContentHeader,
            ui_to_rect(chrome.page_title_rect),
        ));
        regions.push(ZsShellLayoutRegion::new(
            "viewport.mask",
            ZsShellLayoutRegionKind::ViewportMask,
            ui_to_rect(chrome.viewport_mask_rect),
        ));

        for (index, item) in self.nav_items.iter().enumerate() {
            regions.push(ZsShellLayoutRegion::new(
                item.id.clone(),
                ZsShellLayoutRegionKind::NavItem,
                ui_to_rect(zs_shell_nav_item_rect(window, index, dpi)),
            ));
        }

        for section in self.sections(window, dpi) {
            regions.push(ZsShellLayoutRegion::new(
                section.id.clone(),
                ZsShellLayoutRegionKind::GroupCard,
                ui_to_rect(section.rect.offset_y(self.scroll_y)),
            ));
            let layout =
                ZsShellFormSectionLayout::from_section(section.rect.offset_y(self.scroll_y), dpi);
            if let Some(card) = self.cards.iter().find(|card| card.id == section.id) {
                for (row_index, row) in card.rows.iter().enumerate() {
                    let row_rect = layout.row_rect(row_index as i32);
                    regions.push(ZsShellLayoutRegion::new(
                        row.id.clone(),
                        ZsShellLayoutRegionKind::ContentRow,
                        ui_to_rect(row_rect),
                    ));
                    if let Some(accessory_rect) =
                        layout.accessory_rect(row_index as i32, row.accessory.width(dpi))
                    {
                        regions.push(ZsShellLayoutRegion::new(
                            format!("{}.accessory", row.id),
                            ZsShellLayoutRegionKind::RowAccessory,
                            ui_to_rect(accessory_rect),
                        ));
                    }
                }
            }
        }

        if let Some(action_rect) = zs_shell_action_area_rect(window, &self.action_area, dpi) {
            regions.push(ZsShellLayoutRegion::new(
                "actions",
                ZsShellLayoutRegionKind::ActionArea,
                ui_to_rect(action_rect),
            ));
            for (id, rect) in zs_shell_action_button_rects(window, &self.action_area, dpi) {
                regions.push(ZsShellLayoutRegion::new(
                    id,
                    ZsShellLayoutRegionKind::ActionButton,
                    ui_to_rect(rect),
                ));
            }
        }

        if let Some(scrollbar) = self.scrollbar_render_plan(window, dpi) {
            if let Some(track) = scrollbar.track_rect {
                regions.push(ZsShellLayoutRegion::new(
                    "scrollbar.track",
                    ZsShellLayoutRegionKind::ScrollbarTrack,
                    ui_to_rect(track),
                ));
            }
            regions.push(ZsShellLayoutRegion::new(
                "scrollbar.thumb",
                ZsShellLayoutRegionKind::ScrollbarThumb,
                ui_to_rect(scrollbar.thumb_rect),
            ));
        }

        ZsShellLayoutPlan { bounds, regions }
    }

    pub fn paint_plan(&self, bounds: Rect, dpi: Dpi) -> ZsShellPaintPlan {
        let (mut chrome_and_nav, content, trailing, _) = self.paint_plan_parts(bounds, dpi);
        chrome_and_nav.extend(content);
        chrome_and_nav.extend(trailing);
        chrome_and_nav
    }

    pub fn native_draw_plan(&self, bounds: Rect, dpi: Dpi) -> NativeDrawPlan {
        let (chrome_and_nav, content, trailing, content_clip_rect) =
            self.paint_plan_parts(bounds, dpi);
        let mut plan = chrome_and_nav.to_native_draw_plan();
        plan.push(NativeDrawCommand::PushClip {
            rect: ui_to_rect(content_clip_rect),
        });
        plan.commands.extend(content.to_native_draw_plan().commands);
        plan.push(NativeDrawCommand::PopClip);
        plan.commands
            .extend(trailing.to_native_draw_plan().commands);
        plan
    }

    fn paint_plan_parts(
        &self,
        bounds: Rect,
        dpi: Dpi,
    ) -> (ZsShellPaintPlan, ZsShellPaintPlan, ZsShellPaintPlan, UiRect) {
        let window = rect_to_ui(bounds);
        let chrome = zs_shell_chrome_render_plan(window, dpi);
        let mut chrome_and_nav = ZsShellPaintPlan::default();
        chrome_and_nav.extend(zs_shell_chrome_paint_plan(
            &chrome,
            self.app_title.clone(),
            self.title.clone(),
        ));

        let selected_id = self.selected_nav_id.as_deref();
        let hovered_id = self.hovered_nav_id.as_deref();
        for (index, item) in self.nav_items.iter().enumerate() {
            chrome_and_nav.extend(zs_shell_nav_item_paint_plan(
                &ZsShellNavItemRender {
                    id: item.id.clone(),
                    index,
                    label: item.label.clone(),
                    icon: item.icon,
                    rect: zs_shell_nav_item_rect(window, index, dpi),
                    selected: selected_id == Some(item.id.as_str())
                        || (selected_id.is_none() && index == 0),
                    hovered: hovered_id == Some(item.id.as_str()),
                    badge_rect: item.badge.then(|| {
                        zs_shell_nav_badge_rect(zs_shell_nav_item_rect(window, index, dpi), dpi)
                    }),
                },
                dpi,
            ));
        }

        let content = ZsShellContentRenderPlan {
            sections: self.sections(window, dpi),
            scroll_y: self.scroll_y,
        };
        let mut content_plan = zs_shell_content_paint_plan(&content, dpi);
        content_plan.extend(self.row_paint_plan(window, dpi));

        let mut trailing = ZsShellPaintPlan::default();
        if let Some(action_plan) = self.action_area_paint_plan(window, dpi) {
            trailing.extend(action_plan);
        }
        if let Some(scrollbar) = self.scrollbar_render_plan(window, dpi) {
            trailing.extend(zs_shell_scrollbar_paint_plan(&scrollbar));
        }
        trailing.extend(zs_shell_viewport_mask_paint_plan(&chrome));
        (
            chrome_and_nav,
            content_plan,
            trailing,
            chrome.content_clip_rect,
        )
    }

    pub fn native_draw_plan_with_metrics(
        &self,
        bounds: Rect,
        dpi: Dpi,
        metrics: ZsShellLayoutMetrics,
    ) -> NativeDrawPlan {
        let _ = metrics;
        self.native_draw_plan(bounds, dpi)
    }

    pub fn audit(&self) -> ZsShellLayoutAudit {
        let mut issues = Vec::new();
        if self.id.trim().is_empty() {
            issues.push("layout id is empty".to_string());
        }
        if self.title.trim().is_empty() {
            issues.push(format!("layout `{}` has an empty title", self.id));
        }
        collect_duplicate_ids(
            &mut issues,
            "navigation item",
            self.nav_items.iter().map(|item| item.id.as_str()),
        );
        collect_duplicate_ids(
            &mut issues,
            "group card",
            self.cards.iter().map(|card| card.id.as_str()),
        );
        for card in &self.cards {
            if card.title.trim().is_empty() {
                issues.push(format!("group card `{}` has an empty title", card.id));
            }
            collect_duplicate_ids(
                &mut issues,
                &format!("row in card `{}`", card.id),
                card.rows.iter().map(|row| row.id.as_str()),
            );
        }
        collect_duplicate_ids(
            &mut issues,
            "action button",
            self.action_area.buttons().map(|button| button.id.as_str()),
        );
        if let Some(selected) = &self.selected_nav_id {
            if !self.nav_items.iter().any(|item| &item.id == selected) {
                issues.push(format!(
                    "selected navigation item `{selected}` is not declared"
                ));
            }
        }

        ZsShellLayoutAudit {
            valid: issues.is_empty(),
            issue_count: issues.len(),
            issues,
        }
    }

    fn sections(&self, window: UiRect, dpi: Dpi) -> Vec<ZsShellSection> {
        let mut out = Vec::with_capacity(self.cards.len());
        let mut top = zs_shell_scale(ZS_SHELL_CONTENT_TOP_GAP, dpi);
        let gap = zs_shell_scale(ZS_SHELL_FORM_SECTION_GAP, dpi);
        let content_x = zs_shell_content_x(window, dpi);
        let content_w = zs_shell_content_w(window, dpi);
        for card in &self.cards {
            let h =
                zs_shell_form_section_height_with_extra(card.rows.len() as i32, card.extra_px, dpi);
            out.push(ZsShellSection {
                id: card.id.clone(),
                title: card.title.clone(),
                rect: UiRect::new(
                    content_x,
                    window.top + zs_shell_content_y(dpi) + top,
                    content_x + content_w,
                    window.top + zs_shell_content_y(dpi) + top + h,
                ),
            });
            top += h + gap;
        }
        out
    }

    fn content_total_h(&self, window: UiRect, dpi: Dpi) -> i32 {
        self.sections(window, dpi)
            .iter()
            .map(|section| {
                section.rect.bottom - window.top - zs_shell_content_y(dpi) + zs_shell_scale(16, dpi)
            })
            .max()
            .unwrap_or(0)
            .max(0)
    }

    fn scrollbar_render_plan(
        &self,
        window: UiRect,
        dpi: Dpi,
    ) -> Option<ZsShellScrollbarRenderPlan> {
        zs_shell_scrollbar_render_plan(
            window,
            self.content_total_h(window, dpi),
            self.scroll_y,
            self.scrollbar_visible,
            self.scrollbar_dragging,
            dpi,
        )
    }

    fn row_paint_plan(&self, window: UiRect, dpi: Dpi) -> ZsShellPaintPlan {
        let mut plan = ZsShellPaintPlan::default();
        for section in self.sections(window, dpi) {
            let Some(card) = self.cards.iter().find(|card| card.id == section.id) else {
                continue;
            };
            let layout =
                ZsShellFormSectionLayout::from_section(section.rect.offset_y(self.scroll_y), dpi);
            for (index, row) in card.rows.iter().enumerate() {
                let row_rect = layout.row_rect(index as i32);
                let accessory_width = row.accessory.width(dpi).unwrap_or(0);
                let text_right = if accessory_width > 0 {
                    row_rect.right - accessory_width - zs_shell_scale(16, dpi)
                } else {
                    row_rect.right
                };
                let title_bottom = if row.description.is_some() {
                    row_rect.top + zs_shell_scale(18, dpi)
                } else {
                    row_rect.bottom
                };
                plan.text_commands.push(ZsShellTextCommand {
                    rect: UiRect::new(row_rect.left, row_rect.top, text_right, title_bottom),
                    content: ZsShellTextContent::Label(row.title.clone()),
                    color: ZsShellThemeRole::Text,
                    size: 14,
                    bold: false,
                    font: ZsShellTextFontRole::UiText,
                    align: ZsShellTextAlign::Left,
                });
                if let Some(description) = &row.description {
                    plan.text_commands.push(ZsShellTextCommand {
                        rect: UiRect::new(
                            row_rect.left,
                            row_rect.top + zs_shell_scale(18, dpi),
                            text_right,
                            row_rect.bottom,
                        ),
                        content: ZsShellTextContent::Label(description.clone()),
                        color: ZsShellThemeRole::TextMuted,
                        size: 12,
                        bold: false,
                        font: ZsShellTextFontRole::UiText,
                        align: ZsShellTextAlign::Left,
                    });
                }
                if index + 1 < card.rows.len() {
                    plan.paint_commands.push(ZsShellPaintCommand::FillRect {
                        rect: UiRect::new(
                            row_rect.left,
                            row_rect.bottom + zs_shell_scale(3, dpi),
                            row_rect.right,
                            row_rect.bottom + zs_shell_scale(4, dpi),
                        ),
                        fill: ZsShellThemeRole::Stroke,
                    });
                }
                if let Some(accessory_rect) =
                    layout.accessory_rect(index as i32, row.accessory.width(dpi))
                {
                    plan.extend(row.accessory.paint_plan(accessory_rect, dpi));
                }
            }
        }
        plan
    }

    fn action_area_paint_plan(&self, window: UiRect, dpi: Dpi) -> Option<ZsShellPaintPlan> {
        if self.action_area.is_empty() {
            return None;
        }
        let mut plan = ZsShellPaintPlan::default();
        for (button, rect) in self.action_area.buttons().zip(
            zs_shell_action_button_rects(window, &self.action_area, dpi)
                .into_iter()
                .map(|(_, rect)| rect),
        ) {
            plan.extend(zs_shell_button_paint_plan(
                rect,
                button.label.clone(),
                button.kind.into(),
                false,
                false,
                dpi,
            ));
        }
        Some(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellNavItemSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: ZsShellNavIconKind,
    pub badge: bool,
}

impl ZsShellNavItemSpec {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            icon: ZsShellNavIconKind::General,
            badge: false,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon_name: impl AsRef<str>) -> Self {
        self.icon = ZsShellNavIconKind::from_name(icon_name.as_ref());
        self
    }

    pub fn badge(mut self, badge: bool) -> Self {
        self.badge = badge;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellNavIconKind {
    General,
    Hotkey,
    Plugin,
    Group,
    Sync,
    About,
}

impl ZsShellNavIconKind {
    pub fn from_name(value: &str) -> Self {
        match value {
            "hotkey" | "keyboard" => Self::Hotkey,
            "plugin" | "extension" => Self::Plugin,
            "group" | "folder" => Self::Group,
            "sync" | "cloud" => Self::Sync,
            "about" | "info" => Self::About,
            _ => Self::General,
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::General => "\u{E713}",
            Self::Hotkey => "\u{E76C}",
            Self::Plugin => "\u{E8D4}",
            Self::Group => "\u{E8A5}",
            Self::Sync => "\u{E753}",
            Self::About => "\u{E946}",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellGroupCardSpec {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub rows: Vec<ZsShellContentRowSpec>,
    pub extra_px: i32,
}

impl ZsShellGroupCardSpec {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            rows: Vec::new(),
            extra_px: 0,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn row(mut self, row: ZsShellContentRowSpec) -> Self {
        self.rows.push(row);
        self
    }

    pub fn extra_px(mut self, extra_px: i32) -> Self {
        self.extra_px = extra_px.max(0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellContentRowSpec {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub accessory: ZsShellRowAccessory,
}

impl ZsShellContentRowSpec {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            accessory: ZsShellRowAccessory::None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn accessory(mut self, accessory: ZsShellRowAccessory) -> Self {
        self.accessory = accessory;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellRowAccessory {
    None,
    Value {
        value: String,
    },
    Toggle {
        checked: bool,
    },
    Button {
        label: String,
        action_id: String,
    },
    AccentButton {
        label: String,
        action_id: String,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
    },
}

impl ZsShellRowAccessory {
    pub const fn toggle(checked: bool) -> Self {
        Self::Toggle { checked }
    }

    pub fn value(value: impl Into<String>) -> Self {
        Self::Value {
            value: value.into(),
        }
    }

    pub fn button(label: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self::Button {
            label: label.into(),
            action_id: action_id.into(),
        }
    }

    pub fn accent_button(label: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self::AccentButton {
            label: label.into(),
            action_id: action_id.into(),
        }
    }

    pub fn dropdown(
        selected: impl Into<String>,
        options: impl IntoIterator<Item = String>,
    ) -> Self {
        Self::Dropdown {
            selected: selected.into(),
            options: options.into_iter().collect(),
        }
    }

    fn width(&self, dpi: Dpi) -> Option<i32> {
        match self {
            Self::None => None,
            Self::Value { .. } => Some(zs_shell_scale(168, dpi)),
            Self::Toggle { .. } => Some(zs_shell_scale(44, dpi)),
            Self::Button { .. } | Self::AccentButton { .. } => Some(zs_shell_scale(128, dpi)),
            Self::Dropdown { .. } => Some(zs_shell_scale(176, dpi)),
        }
    }

    fn paint_plan(&self, rect: UiRect, dpi: Dpi) -> ZsShellPaintPlan {
        match self {
            Self::None => ZsShellPaintPlan::default(),
            Self::Value { value } => ZsShellPaintPlan {
                paint_commands: Vec::new(),
                text_commands: vec![ZsShellTextCommand {
                    rect,
                    content: ZsShellTextContent::Label(value.clone()),
                    color: ZsShellThemeRole::TextMuted,
                    size: 14,
                    bold: false,
                    font: ZsShellTextFontRole::UiText,
                    align: ZsShellTextAlign::Right,
                }],
            },
            Self::Toggle { checked } => zs_shell_toggle_paint_plan(rect, false, *checked, dpi),
            Self::Button { label, .. } => zs_shell_button_paint_plan(
                rect,
                label.clone(),
                ZsShellComponentKind::Button,
                false,
                false,
                dpi,
            ),
            Self::AccentButton { label, .. } => zs_shell_button_paint_plan(
                rect,
                label.clone(),
                ZsShellComponentKind::AccentButton,
                false,
                false,
                dpi,
            ),
            Self::Dropdown { selected, .. } => zs_shell_button_paint_plan(
                rect,
                selected.clone(),
                ZsShellComponentKind::Dropdown,
                false,
                false,
                dpi,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellActionAreaSpec {
    pub primary: Vec<ZsShellActionButtonSpec>,
    pub secondary: Vec<ZsShellActionButtonSpec>,
}

impl ZsShellActionAreaSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn primary(mut self, button: ZsShellActionButtonSpec) -> Self {
        self.primary.push(button);
        self
    }

    pub fn secondary(mut self, button: ZsShellActionButtonSpec) -> Self {
        self.secondary.push(button);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.primary.is_empty() && self.secondary.is_empty()
    }

    pub fn buttons(&self) -> impl Iterator<Item = &ZsShellActionButtonSpec> {
        self.secondary.iter().chain(self.primary.iter())
    }
}

impl Default for ZsShellActionAreaSpec {
    fn default() -> Self {
        Self {
            primary: Vec::new(),
            secondary: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellActionButtonSpec {
    pub id: String,
    pub label: String,
    pub kind: ZsShellActionButtonKind,
}

impl ZsShellActionButtonSpec {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: ZsShellActionButtonKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
        }
    }

    pub fn primary(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(id, label, ZsShellActionButtonKind::Primary)
    }

    pub fn secondary(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(id, label, ZsShellActionButtonKind::Secondary)
    }

    pub fn destructive(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(id, label, ZsShellActionButtonKind::Destructive)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellActionButtonKind {
    Primary,
    Secondary,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellComponentKind {
    Button,
    AccentButton,
    DestructiveButton,
    Dropdown,
}

impl From<ZsShellActionButtonKind> for ZsShellComponentKind {
    fn from(value: ZsShellActionButtonKind) -> Self {
        match value {
            ZsShellActionButtonKind::Primary => Self::AccentButton,
            ZsShellActionButtonKind::Secondary => Self::Button,
            ZsShellActionButtonKind::Destructive => Self::DestructiveButton,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsShellLayoutMetrics {
    pub dpi: Dpi,
}

impl Default for ZsShellLayoutMetrics {
    fn default() -> Self {
        Self {
            dpi: Dpi::standard(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellLayoutPlan {
    pub bounds: Rect,
    pub regions: Vec<ZsShellLayoutRegion>,
}

impl ZsShellLayoutPlan {
    pub fn region(&self, id: &str) -> Option<&ZsShellLayoutRegion> {
        self.regions.iter().find(|region| region.id == id)
    }

    pub fn regions_of_kind(&self, kind: ZsShellLayoutRegionKind) -> Vec<&ZsShellLayoutRegion> {
        self.regions
            .iter()
            .filter(|region| region.kind == kind)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellPointerDownTarget {
    None,
    NavigationItem {
        index: usize,
        id: String,
    },
    ScrollbarThumb {
        drag_start_y: i32,
        drag_start_scroll: i32,
    },
    ScrollbarTrack {
        scroll_y: i32,
    },
    RowAccessory {
        row_id: String,
    },
    ActionButton {
        id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellNavHoverTransition {
    pub next_hovered_nav_id: Option<String>,
    pub invalidate_rects: Vec<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellPointerMoveTransition {
    pub drag_scroll_y: Option<i32>,
    pub nav_hover: Option<ZsShellNavHoverTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellInteractionEvent {
    NavigationSelected { id: String },
    ToggleChanged { row_id: String, checked: bool },
    ActionInvoked { id: String },
    DropdownChanged { row_id: String, selected: String },
    ScrollChanged { scroll_y: i32 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellInteractionUpdate {
    pub redraw: bool,
    pub events: Vec<ZsShellInteractionEvent>,
    pub invalidate_rects: Vec<Rect>,
}

impl ZsShellInteractionUpdate {
    fn redraw() -> Self {
        Self {
            redraw: true,
            events: Vec::new(),
            invalidate_rects: Vec::new(),
        }
    }

    fn event(event: ZsShellInteractionEvent) -> Self {
        Self {
            redraw: true,
            events: vec![event],
            invalidate_rects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZsShellRuntime {
    pub spec: ZsShellLayoutSpec,
    pub bounds: Rect,
    pub dpi: Dpi,
    drag_start_y: i32,
    drag_start_scroll: i32,
}

impl ZsShellRuntime {
    pub fn new(spec: ZsShellLayoutSpec, bounds: Rect, dpi: Dpi) -> Self {
        let mut runtime = Self {
            spec,
            bounds,
            dpi,
            drag_start_y: 0,
            drag_start_scroll: 0,
        };
        runtime.clamp_scroll();
        runtime
    }

    pub fn set_surface(&mut self, bounds: Rect, dpi: Dpi) -> bool {
        if self.bounds == bounds && self.dpi == dpi {
            return false;
        }
        self.bounds = bounds;
        self.dpi = dpi;
        self.clamp_scroll();
        true
    }

    pub fn draw_plan(&self) -> NativeDrawPlan {
        self.spec.native_draw_plan(self.bounds, self.dpi)
    }

    pub fn pointer_move(&mut self, point: Point) -> ZsShellInteractionUpdate {
        let transition = zs_shell_pointer_move_transition(
            &self.spec,
            self.bounds,
            self.dpi,
            point,
            self.spec.scrollbar_dragging,
            self.drag_start_y,
            self.drag_start_scroll,
        );
        if let Some(scroll_y) = transition.drag_scroll_y {
            if scroll_y != self.spec.scroll_y {
                self.spec.scroll_y = scroll_y;
                return ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ScrollChanged {
                    scroll_y,
                });
            }
            return ZsShellInteractionUpdate::default();
        }

        let Some(hover) = transition.nav_hover else {
            return ZsShellInteractionUpdate::default();
        };
        if hover.next_hovered_nav_id == self.spec.hovered_nav_id {
            return ZsShellInteractionUpdate::default();
        }
        self.spec.hovered_nav_id = hover.next_hovered_nav_id;
        ZsShellInteractionUpdate {
            redraw: true,
            events: Vec::new(),
            invalidate_rects: hover.invalidate_rects,
        }
    }

    pub fn pointer_leave(&mut self) -> ZsShellInteractionUpdate {
        if self.spec.scrollbar_dragging || self.spec.hovered_nav_id.is_none() {
            return ZsShellInteractionUpdate::default();
        }
        let transition = zs_shell_nav_hover_transition(
            &self.spec,
            self.bounds,
            self.dpi,
            self.spec.hovered_nav_id.as_deref(),
            None,
        );
        self.spec.hovered_nav_id = None;
        ZsShellInteractionUpdate {
            redraw: true,
            events: Vec::new(),
            invalidate_rects: transition.invalidate_rects,
        }
    }

    pub fn pointer_down(&mut self, point: Point) -> ZsShellInteractionUpdate {
        match zs_shell_pointer_down_target(&self.spec, self.bounds, self.dpi, point) {
            ZsShellPointerDownTarget::None => ZsShellInteractionUpdate::default(),
            ZsShellPointerDownTarget::NavigationItem { id, .. } => {
                if self.spec.selected_nav_id.as_deref() == Some(id.as_str()) {
                    return ZsShellInteractionUpdate::default();
                }
                self.spec.selected_nav_id = Some(id.clone());
                self.spec.scroll_y = 0;
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::NavigationSelected { id })
            }
            ZsShellPointerDownTarget::ScrollbarThumb {
                drag_start_y,
                drag_start_scroll,
            } => {
                self.drag_start_y = drag_start_y;
                self.drag_start_scroll = drag_start_scroll;
                self.spec.scrollbar_dragging = true;
                ZsShellInteractionUpdate::redraw()
            }
            ZsShellPointerDownTarget::ScrollbarTrack { scroll_y } => {
                if self.spec.scroll_y == scroll_y {
                    return ZsShellInteractionUpdate::default();
                }
                self.spec.scroll_y = scroll_y;
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ScrollChanged { scroll_y })
            }
            ZsShellPointerDownTarget::RowAccessory { row_id } => {
                self.activate_row_accessory(&row_id)
            }
            ZsShellPointerDownTarget::ActionButton { id } => {
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ActionInvoked { id })
            }
        }
    }

    pub fn pointer_up(&mut self) -> ZsShellInteractionUpdate {
        if !self.spec.scrollbar_dragging {
            return ZsShellInteractionUpdate::default();
        }
        self.spec.scrollbar_dragging = false;
        ZsShellInteractionUpdate::redraw()
    }

    pub fn pointer_cancel(&mut self) -> ZsShellInteractionUpdate {
        self.pointer_up()
    }

    pub fn scroll_by(&mut self, delta_y: i32) -> ZsShellInteractionUpdate {
        let next =
            (self.spec.scroll_y + delta_y).clamp(0, self.spec.max_scroll(self.bounds, self.dpi));
        if next == self.spec.scroll_y {
            return ZsShellInteractionUpdate::default();
        }
        self.spec.scroll_y = next;
        ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ScrollChanged { scroll_y: next })
    }

    fn clamp_scroll(&mut self) {
        self.spec.scroll_y = self
            .spec
            .scroll_y
            .clamp(0, self.spec.max_scroll(self.bounds, self.dpi));
    }

    fn activate_row_accessory(&mut self, row_id: &str) -> ZsShellInteractionUpdate {
        let Some(row) = self
            .spec
            .cards
            .iter_mut()
            .flat_map(|card| card.rows.iter_mut())
            .find(|row| row.id == row_id)
        else {
            return ZsShellInteractionUpdate::default();
        };

        match &mut row.accessory {
            ZsShellRowAccessory::Toggle { checked } => {
                *checked = !*checked;
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ToggleChanged {
                    row_id: row_id.to_string(),
                    checked: *checked,
                })
            }
            ZsShellRowAccessory::Button { action_id, .. }
            | ZsShellRowAccessory::AccentButton { action_id, .. } => {
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::ActionInvoked {
                    id: action_id.clone(),
                })
            }
            ZsShellRowAccessory::Dropdown { selected, options } if !options.is_empty() => {
                let current = options
                    .iter()
                    .position(|item| item == selected)
                    .unwrap_or(0);
                *selected = options[(current + 1) % options.len()].clone();
                ZsShellInteractionUpdate::event(ZsShellInteractionEvent::DropdownChanged {
                    row_id: row_id.to_string(),
                    selected: selected.clone(),
                })
            }
            _ => ZsShellInteractionUpdate::default(),
        }
    }
}

pub fn zs_shell_nav_item_index_at(
    spec: &ZsShellLayoutSpec,
    bounds: Rect,
    dpi: Dpi,
    point: Point,
) -> Option<usize> {
    let window = rect_to_ui(bounds);
    (0..spec.nav_items.len())
        .find(|&index| zs_shell_nav_item_rect(window, index, dpi).contains(point.x, point.y))
}

pub fn zs_shell_nav_hover_transition(
    spec: &ZsShellLayoutSpec,
    bounds: Rect,
    dpi: Dpi,
    current_hovered_nav_id: Option<&str>,
    next_hovered_nav_id: Option<&str>,
) -> ZsShellNavHoverTransition {
    let next_hovered_nav_id = next_hovered_nav_id
        .filter(|id| spec.nav_items.iter().any(|item| item.id == *id))
        .map(str::to_string);
    if current_hovered_nav_id == next_hovered_nav_id.as_deref() {
        return ZsShellNavHoverTransition {
            next_hovered_nav_id,
            invalidate_rects: Vec::new(),
        };
    }

    let window = rect_to_ui(bounds);
    let mut invalidate_rects = Vec::new();
    for id in [current_hovered_nav_id, next_hovered_nav_id.as_deref()]
        .into_iter()
        .flatten()
    {
        if let Some(index) = spec.nav_items.iter().position(|item| item.id == id) {
            let rect = ui_to_rect(zs_shell_nav_item_rect(window, index, dpi));
            if !invalidate_rects.contains(&rect) {
                invalidate_rects.push(rect);
            }
        }
    }
    ZsShellNavHoverTransition {
        next_hovered_nav_id,
        invalidate_rects,
    }
}

pub fn zs_shell_pointer_move_transition(
    spec: &ZsShellLayoutSpec,
    bounds: Rect,
    dpi: Dpi,
    point: Point,
    scroll_dragging: bool,
    drag_start_y: i32,
    drag_start_scroll: i32,
) -> ZsShellPointerMoveTransition {
    if scroll_dragging {
        return ZsShellPointerMoveTransition {
            drag_scroll_y: spec.scroll_layout(bounds, dpi, true).drag_scroll_target(
                drag_start_y,
                drag_start_scroll,
                point.y,
            ),
            nav_hover: None,
        };
    }

    let next = zs_shell_nav_item_index_at(spec, bounds, dpi, point)
        .map(|index| spec.nav_items[index].id.as_str());
    ZsShellPointerMoveTransition {
        drag_scroll_y: None,
        nav_hover: Some(zs_shell_nav_hover_transition(
            spec,
            bounds,
            dpi,
            spec.hovered_nav_id.as_deref(),
            next,
        )),
    }
}

pub fn zs_shell_pointer_down_target(
    spec: &ZsShellLayoutSpec,
    bounds: Rect,
    dpi: Dpi,
    point: Point,
) -> ZsShellPointerDownTarget {
    if let Some(index) = zs_shell_nav_item_index_at(spec, bounds, dpi, point) {
        return ZsShellPointerDownTarget::NavigationItem {
            index,
            id: spec.nav_items[index].id.clone(),
        };
    }

    let scroll_layout = spec.scroll_layout(bounds, dpi, true);
    if scroll_layout
        .thumb_hit_rect(spec.scroll_y, zs_shell_scale(4, dpi))
        .map(|rect| rect.contains(point.x, point.y))
        .unwrap_or(false)
    {
        return ZsShellPointerDownTarget::ScrollbarThumb {
            drag_start_y: point.y,
            drag_start_scroll: spec.scroll_y,
        };
    }
    if scroll_layout
        .track_hit_rect(zs_shell_scale(4, dpi), zs_shell_scale(2, dpi))
        .map(|rect| rect.contains(point.x, point.y))
        .unwrap_or(false)
    {
        if let Some(scroll_y) = scroll_layout.track_click_scroll_target(point.y) {
            return ZsShellPointerDownTarget::ScrollbarTrack { scroll_y };
        }
    }

    let plan = spec.layout_plan(bounds, dpi);
    if let Some(region) = plan.regions.iter().find(|region| {
        region.kind == ZsShellLayoutRegionKind::RowAccessory && rect_contains(region.rect, point)
    }) {
        return ZsShellPointerDownTarget::RowAccessory {
            row_id: region
                .id
                .strip_suffix(".accessory")
                .unwrap_or(region.id.as_str())
                .to_string(),
        };
    }
    if let Some(region) = plan.regions.iter().find(|region| {
        region.kind == ZsShellLayoutRegionKind::ActionButton && rect_contains(region.rect, point)
    }) {
        return ZsShellPointerDownTarget::ActionButton {
            id: region.id.clone(),
        };
    }

    ZsShellPointerDownTarget::None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellLayoutRegion {
    pub id: String,
    pub kind: ZsShellLayoutRegionKind,
    pub rect: Rect,
}

impl ZsShellLayoutRegion {
    pub fn new(id: impl Into<String>, kind: ZsShellLayoutRegionKind, rect: Rect) -> Self {
        Self {
            id: id.into(),
            kind,
            rect,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellLayoutRegionKind {
    Root,
    NavigationPane,
    ContentPane,
    NavItem,
    ContentHeader,
    ViewportMask,
    GroupCard,
    ContentRow,
    RowAccessory,
    ActionArea,
    ActionButton,
    ScrollbarTrack,
    ScrollbarThumb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellLayoutAudit {
    pub valid: bool,
    pub issue_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellThemeRole {
    Background,
    NavBackground,
    NavSelectedFill,
    NavHoverFill,
    Surface,
    Accent,
    AccentHover,
    AccentPressed,
    ButtonBg,
    ButtonHover,
    ButtonPressed,
    ControlStroke,
    Stroke,
    ScrollbarTrack,
    ScrollbarThumb,
    ScrollbarThumbDragging,
    Text,
    TextMuted,
    Danger,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellTextFontRole {
    UiText,
    Display,
    FluentIcon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellTextContent {
    Label(String),
    NavIcon(ZsShellNavIconKind),
    ChromeMenuIcon,
    DropdownArrow,
}

impl ZsShellTextContent {
    fn text(&self) -> String {
        match self {
            Self::Label(label) => label.clone(),
            Self::NavIcon(icon) => icon.glyph().to_string(),
            Self::ChromeMenuIcon => "\u{E700}".to_string(),
            Self::DropdownArrow => "\u{25BE}".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellPaintCommand {
    FillRect {
        rect: UiRect,
        fill: ZsShellThemeRole,
    },
    RoundRect {
        rect: UiRect,
        fill: ZsShellThemeRole,
        stroke: ZsShellThemeRole,
        radius: i32,
    },
    RoundFill {
        rect: UiRect,
        fill: ZsShellThemeRole,
        radius: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellTextCommand {
    pub rect: UiRect,
    pub content: ZsShellTextContent,
    pub color: ZsShellThemeRole,
    pub size: i32,
    pub bold: bool,
    pub font: ZsShellTextFontRole,
    pub align: ZsShellTextAlign,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellPaintPlan {
    pub paint_commands: Vec<ZsShellPaintCommand>,
    pub text_commands: Vec<ZsShellTextCommand>,
}

impl ZsShellPaintPlan {
    pub fn extend(&mut self, other: ZsShellPaintPlan) {
        self.paint_commands.extend(other.paint_commands);
        self.text_commands.extend(other.text_commands);
    }

    pub fn command_count(&self) -> usize {
        self.paint_commands.len() + self.text_commands.len()
    }

    pub fn to_native_draw_plan(&self) -> NativeDrawPlan {
        let mut plan = NativeDrawPlan::default();
        for command in &self.paint_commands {
            match command {
                ZsShellPaintCommand::FillRect { rect, fill } => {
                    plan.push(NativeDrawCommand::FillRect {
                        rect: ui_to_rect(*rect),
                        fill: NativeDrawFill::Role(shell_role_to_color_role(*fill)),
                    });
                }
                ZsShellPaintCommand::RoundRect {
                    rect,
                    fill,
                    stroke,
                    radius,
                } => {
                    plan.push(NativeDrawCommand::RoundRect {
                        rect: ui_to_rect(*rect),
                        fill: NativeDrawFill::Role(shell_role_to_color_role(*fill)),
                        stroke: Some(NativeDrawFill::Role(shell_role_to_color_role(*stroke))),
                        radius: *radius,
                    });
                }
                ZsShellPaintCommand::RoundFill { rect, fill, radius } => {
                    plan.push(NativeDrawCommand::RoundFill {
                        rect: ui_to_rect(*rect),
                        fill: NativeDrawFill::Role(shell_role_to_color_role(*fill)),
                        radius: *radius,
                    });
                }
            }
        }
        for command in &self.text_commands {
            plan.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                command.content.text(),
                ui_to_rect(command.rect),
                shell_text_style(command),
            )));
        }
        plan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZsShellSection {
    id: String,
    title: String,
    rect: UiRect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZsShellContentRenderPlan {
    sections: Vec<ZsShellSection>,
    scroll_y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ZsShellNavItemRender {
    id: String,
    index: usize,
    label: String,
    icon: ZsShellNavIconKind,
    rect: UiRect,
    selected: bool,
    hovered: bool,
    badge_rect: Option<UiRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZsShellChromeRenderPlan {
    pub window_rect: UiRect,
    pub nav_rect: UiRect,
    pub divider_x: i32,
    pub menu_icon_rect: UiRect,
    pub app_title_rect: UiRect,
    pub page_title_rect: UiRect,
    pub content_clip_rect: UiRect,
    pub viewport_mask_rect: UiRect,
    pub viewport_mask_separator_rect: UiRect,
}

pub fn zs_shell_chrome_render_plan(window: UiRect, dpi: Dpi) -> ZsShellChromeRenderPlan {
    let viewport_mask_rect = zs_shell_viewport_mask_rect_for_window(window, dpi);
    let nav_w = zs_shell_nav_w(dpi);
    ZsShellChromeRenderPlan {
        window_rect: window,
        nav_rect: UiRect::new(window.left, window.top, window.left + nav_w, window.bottom),
        divider_x: window.left + nav_w,
        menu_icon_rect: UiRect::new(
            window.left + zs_shell_scale(22, dpi),
            window.top + zs_shell_scale(18, dpi),
            window.left + zs_shell_scale(50, dpi),
            window.top + zs_shell_scale(46, dpi),
        ),
        app_title_rect: UiRect::new(
            window.left + zs_shell_scale(56, dpi),
            window.top + zs_shell_scale(18, dpi),
            window.left + zs_shell_scale(220, dpi),
            window.top + zs_shell_scale(50, dpi),
        ),
        page_title_rect: zs_shell_title_rect(window, dpi),
        content_clip_rect: zs_shell_safe_paint_rect_for_window(window, dpi),
        viewport_mask_separator_rect: UiRect::new(
            viewport_mask_rect.left + zs_shell_scale(12, dpi),
            viewport_mask_rect.bottom - 1,
            viewport_mask_rect.right - zs_shell_scale(12, dpi),
            viewport_mask_rect.bottom,
        ),
        viewport_mask_rect,
    }
}

pub fn zs_shell_chrome_paint_plan(
    plan: &ZsShellChromeRenderPlan,
    app_title: String,
    page_title: String,
) -> ZsShellPaintPlan {
    ZsShellPaintPlan {
        paint_commands: vec![
            ZsShellPaintCommand::FillRect {
                rect: plan.nav_rect,
                fill: ZsShellThemeRole::NavBackground,
            },
            ZsShellPaintCommand::FillRect {
                rect: UiRect::new(
                    plan.divider_x,
                    plan.window_rect.top,
                    plan.divider_x + 1,
                    plan.window_rect.bottom,
                ),
                fill: ZsShellThemeRole::Stroke,
            },
        ],
        text_commands: vec![
            ZsShellTextCommand {
                rect: plan.menu_icon_rect,
                content: ZsShellTextContent::ChromeMenuIcon,
                color: ZsShellThemeRole::TextMuted,
                size: 16,
                bold: false,
                font: ZsShellTextFontRole::FluentIcon,
                align: ZsShellTextAlign::Center,
            },
            ZsShellTextCommand {
                rect: plan.app_title_rect,
                content: ZsShellTextContent::Label(app_title),
                color: ZsShellThemeRole::Text,
                size: 15,
                bold: true,
                font: ZsShellTextFontRole::UiText,
                align: ZsShellTextAlign::Left,
            },
            ZsShellTextCommand {
                rect: plan.page_title_rect,
                content: ZsShellTextContent::Label(page_title),
                color: ZsShellThemeRole::Text,
                size: 24,
                bold: true,
                font: ZsShellTextFontRole::Display,
                align: ZsShellTextAlign::Left,
            },
        ],
    }
}

fn zs_shell_nav_item_paint_plan(item: &ZsShellNavItemRender, dpi: Dpi) -> ZsShellPaintPlan {
    let mut paint_commands = Vec::new();
    if item.selected {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect: item.rect,
            fill: ZsShellThemeRole::NavSelectedFill,
            radius: zs_shell_scale(6, dpi),
        });
        let bar_h = zs_shell_scale(16, dpi);
        let bar_cy = (item.rect.top + item.rect.bottom) / 2;
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect: UiRect::new(
                item.rect.left + zs_shell_scale(3, dpi),
                bar_cy - bar_h / 2,
                item.rect.left + zs_shell_scale(6, dpi),
                bar_cy + bar_h / 2,
            ),
            fill: ZsShellThemeRole::Accent,
            radius: zs_shell_scale(2, dpi),
        });
    } else if item.hovered {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect: item.rect,
            fill: ZsShellThemeRole::NavHoverFill,
            radius: zs_shell_scale(6, dpi),
        });
    }
    if let Some(rect) = item.badge_rect {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect,
            fill: ZsShellThemeRole::Danger,
            radius: zs_shell_scale(5, dpi),
        });
    }

    let icon_color = if item.selected {
        ZsShellThemeRole::Accent
    } else if item.hovered {
        ZsShellThemeRole::Text
    } else {
        ZsShellThemeRole::TextMuted
    };
    let label_color = if item.selected || item.hovered {
        ZsShellThemeRole::Text
    } else {
        ZsShellThemeRole::TextMuted
    };
    let icon_rect = UiRect::new(
        item.rect.left + zs_shell_scale(10, dpi),
        item.rect.top,
        item.rect.left + zs_shell_scale(38, dpi),
        item.rect.bottom,
    );
    let label_rect = UiRect::new(
        item.rect.left + zs_shell_scale(40, dpi),
        item.rect.top,
        item.rect.right - zs_shell_scale(8, dpi),
        item.rect.bottom,
    );

    ZsShellPaintPlan {
        paint_commands,
        text_commands: vec![
            ZsShellTextCommand {
                rect: icon_rect,
                content: ZsShellTextContent::NavIcon(item.icon),
                color: icon_color,
                size: 16,
                bold: false,
                font: ZsShellTextFontRole::FluentIcon,
                align: ZsShellTextAlign::Center,
            },
            ZsShellTextCommand {
                rect: label_rect,
                content: ZsShellTextContent::Label(item.label.clone()),
                color: label_color,
                size: 14,
                bold: false,
                font: ZsShellTextFontRole::UiText,
                align: ZsShellTextAlign::Left,
            },
        ],
    }
}

fn zs_shell_content_paint_plan(plan: &ZsShellContentRenderPlan, dpi: Dpi) -> ZsShellPaintPlan {
    let mut paint_commands = Vec::with_capacity(plan.sections.len());
    let mut text_commands = Vec::with_capacity(plan.sections.len());
    for section in &plan.sections {
        let rect = section.rect.offset_y(plan.scroll_y);
        paint_commands.push(ZsShellPaintCommand::RoundRect {
            rect,
            fill: ZsShellThemeRole::Surface,
            stroke: ZsShellThemeRole::Stroke,
            radius: zs_shell_scale(8, dpi),
        });
        text_commands.push(ZsShellTextCommand {
            rect: UiRect::new(
                rect.left + zs_shell_scale(16, dpi),
                rect.top + zs_shell_scale(12, dpi),
                rect.right - zs_shell_scale(16, dpi),
                rect.top + zs_shell_scale(34, dpi),
            ),
            content: ZsShellTextContent::Label(section.title.clone()),
            color: ZsShellThemeRole::TextMuted,
            size: 12,
            bold: true,
            font: ZsShellTextFontRole::UiText,
            align: ZsShellTextAlign::Left,
        });
    }
    ZsShellPaintPlan {
        paint_commands,
        text_commands,
    }
}

pub fn zs_shell_viewport_mask_paint_plan(plan: &ZsShellChromeRenderPlan) -> ZsShellPaintPlan {
    ZsShellPaintPlan {
        paint_commands: vec![
            ZsShellPaintCommand::FillRect {
                rect: plan.viewport_mask_rect,
                fill: ZsShellThemeRole::Background,
            },
            ZsShellPaintCommand::FillRect {
                rect: plan.viewport_mask_separator_rect,
                fill: ZsShellThemeRole::Stroke,
            },
        ],
        text_commands: Vec::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZsShellScrollLayout {
    pub content_top: i32,
    pub viewport_bottom: i32,
    pub content_height: i32,
    pub viewport_height: i32,
    pub right: i32,
    pub margin: i32,
    pub bar_width: i32,
    pub min_thumb_height: i32,
    pub track_padding: i32,
}

impl ZsShellScrollLayout {
    pub fn new(
        content_top: i32,
        viewport_bottom: i32,
        content_height: i32,
        viewport_height: i32,
        right: i32,
        margin: i32,
        bar_width: i32,
        dpi: Dpi,
    ) -> Self {
        Self {
            content_top,
            viewport_bottom,
            content_height,
            viewport_height,
            right,
            margin,
            bar_width,
            min_thumb_height: zs_shell_scale(24, dpi),
            track_padding: zs_shell_scale(8, dpi),
        }
    }

    pub fn max_scroll(self) -> i32 {
        (self.content_height - self.viewport_height).max(0)
    }

    pub fn track_rect(self) -> Option<UiRect> {
        if self.max_scroll() <= 0 {
            return None;
        }
        let track_top = self.content_top + self.track_padding;
        let track_bottom = self.viewport_bottom - self.track_padding;
        if track_bottom <= track_top {
            return None;
        }
        let right = self.right - self.margin;
        Some(UiRect::new(
            right - self.bar_width,
            track_top,
            right,
            track_bottom,
        ))
    }

    pub fn thumb_rect(self, scroll_y: i32) -> Option<UiRect> {
        let track = self.track_rect()?;
        let track_h = (track.bottom - track.top).max(1);
        let max_scroll = self.max_scroll();
        if max_scroll <= 0 {
            return None;
        }
        let content_h = self.content_height.max(self.viewport_height + 1);
        let thumb_h = ((self.viewport_height as f32 / content_h as f32) * track_h as f32) as i32;
        let thumb_h = thumb_h.max(self.min_thumb_height).min(track_h);
        let drag_range = (track_h - thumb_h).max(1);
        let scroll_y = scroll_y.clamp(0, max_scroll);
        let thumb_top =
            track.top + ((scroll_y as f32 / max_scroll as f32) * drag_range as f32) as i32;
        Some(UiRect::new(
            track.left,
            thumb_top,
            track.right,
            thumb_top + thumb_h,
        ))
    }

    pub fn thumb_hit_rect(self, scroll_y: i32, extra_x: i32) -> Option<UiRect> {
        let thumb = self.thumb_rect(scroll_y)?;
        Some(UiRect::new(
            thumb.left - extra_x,
            thumb.top,
            thumb.right + extra_x,
            thumb.bottom,
        ))
    }

    pub fn drag_scroll_target(
        self,
        drag_start_y: i32,
        drag_start_scroll: i32,
        pointer_y: i32,
    ) -> Option<i32> {
        let track = self.track_rect()?;
        let thumb = self.thumb_rect(drag_start_scroll)?;
        let track_h = (track.bottom - track.top).max(1);
        let thumb_h = (thumb.bottom - thumb.top).max(1);
        let max_scroll = self.max_scroll();
        if max_scroll <= 0 {
            return Some(0);
        }
        let drag_range = (track_h - thumb_h).max(1);
        let dy = pointer_y - drag_start_y;
        let next = drag_start_scroll + ((dy as f32 / drag_range as f32) * max_scroll as f32) as i32;
        Some(next.clamp(0, max_scroll))
    }

    pub fn track_click_scroll_target(self, pointer_y: i32) -> Option<i32> {
        let track = self.track_rect()?;
        let max_scroll = self.max_scroll();
        if max_scroll <= 0 {
            return Some(0);
        }
        let track_h = (track.bottom - track.top).max(1);
        let pointer_pos = (pointer_y - track.top).clamp(0, track_h);
        Some(((pointer_pos as f32 / track_h as f32) * max_scroll as f32) as i32)
    }

    pub fn track_hit_rect(self, extra_left: i32, extra_right: i32) -> Option<UiRect> {
        let track = self.track_rect()?;
        Some(UiRect::new(
            track.left - extra_left,
            track.top - self.track_padding / 2,
            track.right + extra_right,
            track.bottom + self.track_padding / 2,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsShellScrollbarVisualState {
    Normal,
    Dragging,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsShellScrollbarRenderPlan {
    pub state: ZsShellScrollbarVisualState,
    pub bar_width: i32,
    pub track_rect: Option<UiRect>,
    pub thumb_rect: UiRect,
}

pub fn zs_shell_scrollbar_render_plan(
    window: UiRect,
    content_height: i32,
    scroll_y: i32,
    visible: bool,
    dragging: bool,
    dpi: Dpi,
) -> Option<ZsShellScrollbarRenderPlan> {
    if !visible {
        return None;
    }
    let state = if dragging {
        ZsShellScrollbarVisualState::Dragging
    } else {
        ZsShellScrollbarVisualState::Normal
    };
    let bar_width = if dragging {
        zs_shell_scale(ZS_SHELL_SCROLL_BAR_W_ACTIVE, dpi)
    } else {
        zs_shell_scale(ZS_SHELL_SCROLL_BAR_W, dpi)
    };
    let layout = zs_shell_scroll_layout_for_window(window, content_height, bar_width, dpi);
    let thumb_rect = layout.thumb_rect(scroll_y)?;
    Some(ZsShellScrollbarRenderPlan {
        state,
        bar_width,
        track_rect: if dragging { layout.track_rect() } else { None },
        thumb_rect,
    })
}

pub fn zs_shell_scrollbar_paint_plan(plan: &ZsShellScrollbarRenderPlan) -> ZsShellPaintPlan {
    let mut paint_commands = Vec::with_capacity(2);
    if let Some(rect) = plan.track_rect {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect,
            fill: ZsShellThemeRole::ScrollbarTrack,
            radius: plan.bar_width,
        });
    }
    let thumb_fill = if plan.state == ZsShellScrollbarVisualState::Dragging {
        ZsShellThemeRole::ScrollbarThumbDragging
    } else {
        ZsShellThemeRole::ScrollbarThumb
    };
    paint_commands.push(ZsShellPaintCommand::RoundFill {
        rect: plan.thumb_rect,
        fill: thumb_fill,
        radius: plan.bar_width,
    });
    ZsShellPaintPlan {
        paint_commands,
        text_commands: Vec::new(),
    }
}

pub fn zs_shell_form_section_height_with_extra(rows: i32, extra_px: i32, dpi: Dpi) -> i32 {
    let rows = rows.max(1);
    zs_shell_scale(ZS_SHELL_FORM_HEADER_H, dpi)
        + rows * zs_shell_scale(ZS_SHELL_FORM_ROW_H, dpi)
        + (rows - 1) * zs_shell_scale(ZS_SHELL_FORM_ROW_GAP, dpi)
        + zs_shell_scale(ZS_SHELL_FORM_SECTION_PAD, dpi)
        + zs_shell_scale(ZS_SHELL_FORM_BOTTOM_SAFE_H, dpi)
        + zs_shell_scale(extra_px.max(0), dpi)
}

#[derive(Clone, Copy)]
pub struct ZsShellFormSectionLayout {
    body: UiRect,
    dpi: Dpi,
}

impl ZsShellFormSectionLayout {
    pub fn from_section(section: UiRect, dpi: Dpi) -> Self {
        let pad = zs_shell_scale(ZS_SHELL_FORM_SECTION_PAD, dpi);
        Self {
            body: UiRect::new(
                section.left + pad,
                section.top + zs_shell_scale(ZS_SHELL_FORM_HEADER_H, dpi),
                section.right - pad,
                section.bottom - pad,
            ),
            dpi,
        }
    }

    pub fn row_y(&self, row: i32) -> i32 {
        self.body.top
            + row
                * (zs_shell_scale(ZS_SHELL_FORM_ROW_H, self.dpi)
                    + zs_shell_scale(ZS_SHELL_FORM_ROW_GAP, self.dpi))
    }

    pub fn row_rect(&self, row: i32) -> UiRect {
        let y = self.row_y(row);
        UiRect::new(
            self.body.left,
            y,
            self.body.right,
            y + zs_shell_scale(ZS_SHELL_FORM_ROW_H, self.dpi),
        )
    }

    pub fn accessory_rect(&self, row: i32, width: Option<i32>) -> Option<UiRect> {
        let width = width?;
        let row_rect = self.row_rect(row);
        Some(UiRect::new(
            row_rect.right - width,
            row_rect.top,
            row_rect.right,
            row_rect.bottom,
        ))
    }
}

fn zs_shell_toggle_paint_plan(
    rect: UiRect,
    hover: bool,
    checked: bool,
    dpi: Dpi,
) -> ZsShellPaintPlan {
    let mut paint_commands = vec![ZsShellPaintCommand::FillRect {
        rect,
        fill: ZsShellThemeRole::Surface,
    }];
    let plan = zs_toggle_render_plan(ui_to_rect(rect), hover, checked, dpi);
    let track = rect_to_ui(plan.track);
    let knob = rect_to_ui(plan.knob);

    if checked {
        paint_commands.push(ZsShellPaintCommand::RoundRect {
            rect: track,
            fill: ZsShellThemeRole::Accent,
            stroke: ZsShellThemeRole::Accent,
            radius: plan.track_radius,
        });
        paint_commands.push(ZsShellPaintCommand::RoundRect {
            rect: knob,
            fill: ZsShellThemeRole::White,
            stroke: ZsShellThemeRole::White,
            radius: plan.knob_radius,
        });
    } else {
        paint_commands.push(ZsShellPaintCommand::RoundRect {
            rect: track,
            fill: ZsShellThemeRole::ButtonBg,
            stroke: if hover {
                ZsShellThemeRole::Text
            } else {
                ZsShellThemeRole::TextMuted
            },
            radius: plan.track_radius,
        });
        paint_commands.push(ZsShellPaintCommand::RoundRect {
            rect: knob,
            fill: if hover {
                ZsShellThemeRole::Text
            } else {
                ZsShellThemeRole::TextMuted
            },
            stroke: if hover {
                ZsShellThemeRole::Text
            } else {
                ZsShellThemeRole::TextMuted
            },
            radius: plan.knob_radius,
        });
    }
    ZsShellPaintPlan {
        paint_commands,
        text_commands: Vec::new(),
    }
}

fn zs_shell_button_paint_plan(
    rect: UiRect,
    text: String,
    kind: ZsShellComponentKind,
    hover: bool,
    pressed: bool,
    dpi: Dpi,
) -> ZsShellPaintPlan {
    let rr = UiRect::new(rect.left + 1, rect.top + 1, rect.right - 1, rect.bottom - 1);
    let mut paint_commands = Vec::new();
    let mut text_commands = Vec::new();
    match kind {
        ZsShellComponentKind::Dropdown => {
            let control_h = (rr.bottom - rr.top).max(zs_shell_scale(24, dpi));
            let text_pad = (control_h * 12 / 32).max(zs_shell_scale(10, dpi));
            let arrow_w = (control_h * 20 / 32).max(zs_shell_scale(18, dpi));
            paint_commands.push(ZsShellPaintCommand::RoundRect {
                rect: rr,
                fill: if pressed {
                    ZsShellThemeRole::ButtonPressed
                } else {
                    ZsShellThemeRole::Surface
                },
                stroke: ZsShellThemeRole::ControlStroke,
                radius: zs_shell_scale(6, dpi),
            });
            text_commands.push(ZsShellTextCommand {
                rect: UiRect::new(rr.left + text_pad, rr.top, rr.right - arrow_w, rr.bottom),
                content: ZsShellTextContent::Label(text),
                color: ZsShellThemeRole::Text,
                size: 14,
                bold: false,
                font: ZsShellTextFontRole::UiText,
                align: ZsShellTextAlign::Left,
            });
            text_commands.push(ZsShellTextCommand {
                rect: UiRect::new(
                    rr.right - arrow_w,
                    rr.top,
                    rr.right - (control_h * 8 / 32).max(zs_shell_scale(6, dpi)),
                    rr.bottom,
                ),
                content: ZsShellTextContent::DropdownArrow,
                color: ZsShellThemeRole::TextMuted,
                size: 10,
                bold: false,
                font: ZsShellTextFontRole::FluentIcon,
                align: ZsShellTextAlign::Center,
            });
        }
        ZsShellComponentKind::AccentButton
        | ZsShellComponentKind::Button
        | ZsShellComponentKind::DestructiveButton => {
            let (fill, stroke, color) = match kind {
                ZsShellComponentKind::AccentButton => (
                    if pressed {
                        ZsShellThemeRole::AccentPressed
                    } else if hover {
                        ZsShellThemeRole::AccentHover
                    } else {
                        ZsShellThemeRole::Accent
                    },
                    if pressed || hover {
                        ZsShellThemeRole::Accent
                    } else {
                        ZsShellThemeRole::Accent
                    },
                    ZsShellThemeRole::White,
                ),
                ZsShellComponentKind::DestructiveButton => (
                    ZsShellThemeRole::Danger,
                    ZsShellThemeRole::Danger,
                    ZsShellThemeRole::White,
                ),
                _ => (
                    if pressed {
                        ZsShellThemeRole::ButtonPressed
                    } else if hover {
                        ZsShellThemeRole::ButtonHover
                    } else {
                        ZsShellThemeRole::ButtonBg
                    },
                    ZsShellThemeRole::ControlStroke,
                    ZsShellThemeRole::Text,
                ),
            };
            paint_commands.push(ZsShellPaintCommand::RoundRect {
                rect: rr,
                fill,
                stroke,
                radius: zs_shell_scale(4, dpi),
            });
            text_commands.push(ZsShellTextCommand {
                rect: rr,
                content: ZsShellTextContent::Label(text),
                color,
                size: 14,
                bold: false,
                font: ZsShellTextFontRole::UiText,
                align: ZsShellTextAlign::Center,
            });
        }
    }
    ZsShellPaintPlan {
        paint_commands,
        text_commands,
    }
}

fn zs_shell_action_area_rect(
    window: UiRect,
    action_area: &ZsShellActionAreaSpec,
    dpi: Dpi,
) -> Option<UiRect> {
    if action_area.is_empty() {
        return None;
    }
    let rects = zs_shell_action_button_rects(window, action_area, dpi);
    let first = rects.first()?.1;
    let last = rects.last()?.1;
    Some(UiRect::new(first.left, first.top, last.right, last.bottom))
}

fn zs_shell_action_button_rects(
    window: UiRect,
    action_area: &ZsShellActionAreaSpec,
    dpi: Dpi,
) -> Vec<(String, UiRect)> {
    let top_margin = zs_shell_scale(24, dpi);
    let btn_h = zs_shell_scale(32, dpi);
    let save_w = zs_shell_scale(72, dpi);
    let close_w = zs_shell_scale(64, dpi);
    let gap = zs_shell_scale(20, dpi);
    let right = window.right - top_margin;
    let mut rects = Vec::new();
    let mut cursor_right = right;
    for button in action_area.primary.iter().rev() {
        let rect = UiRect::new(
            cursor_right - save_w,
            window.top + top_margin,
            cursor_right,
            window.top + top_margin + btn_h,
        );
        rects.push((button.id.clone(), rect));
        cursor_right -= save_w + gap;
    }
    for button in action_area.secondary.iter().rev() {
        let rect = UiRect::new(
            cursor_right - close_w,
            window.top + top_margin,
            cursor_right,
            window.top + top_margin + btn_h,
        );
        rects.insert(0, (button.id.clone(), rect));
        cursor_right -= close_w + gap;
    }
    rects
}

fn zs_shell_nav_w(dpi: Dpi) -> i32 {
    zs_shell_scale(ZS_SHELL_NAV_W, dpi)
}

fn zs_shell_content_y(dpi: Dpi) -> i32 {
    zs_shell_scale(ZS_SHELL_TOP_H, dpi)
}

fn zs_shell_content_x(window: UiRect, dpi: Dpi) -> i32 {
    window.left + zs_shell_nav_w(dpi) + zs_shell_scale(ZS_SHELL_CONTENT_GAP, dpi)
}

fn zs_shell_content_w(window: UiRect, dpi: Dpi) -> i32 {
    (window.right
        - window.left
        - zs_shell_nav_w(dpi)
        - zs_shell_scale(ZS_SHELL_CONTENT_GAP * 2, dpi))
    .max(0)
}

fn zs_shell_title_rect(window: UiRect, dpi: Dpi) -> UiRect {
    UiRect::new(
        window.left + zs_shell_nav_w(dpi) + zs_shell_scale(36, dpi),
        window.top + zs_shell_scale(32, dpi),
        window.left + zs_shell_nav_w(dpi) + zs_shell_scale(360, dpi),
        window.top + zs_shell_scale(62, dpi),
    )
}

fn zs_shell_nav_item_rect(window: UiRect, index: usize, dpi: Dpi) -> UiRect {
    let x = window.left + zs_shell_scale(10, dpi);
    let y = window.top + zs_shell_scale(ZS_SHELL_NAV_Y + 8 + (index as i32) * 44, dpi);
    UiRect::new(
        x,
        y,
        window.left + zs_shell_nav_w(dpi) - zs_shell_scale(10, dpi),
        y + zs_shell_scale(36, dpi),
    )
}

fn zs_shell_nav_badge_rect(item_rect: UiRect, dpi: Dpi) -> UiRect {
    UiRect::new(
        item_rect.right - zs_shell_scale(22, dpi),
        item_rect.top + zs_shell_scale(14, dpi),
        item_rect.right - zs_shell_scale(12, dpi),
        item_rect.top + zs_shell_scale(24, dpi),
    )
}

fn zs_shell_viewport_rect_for_window(window: UiRect, dpi: Dpi) -> UiRect {
    UiRect::new(
        window.left + zs_shell_nav_w(dpi),
        window.top + zs_shell_content_y(dpi),
        window.right,
        window.bottom,
    )
}

fn zs_shell_viewport_mask_rect_for_window(window: UiRect, dpi: Dpi) -> UiRect {
    UiRect::new(
        window.left + zs_shell_nav_w(dpi),
        window.top + zs_shell_content_y(dpi),
        window.right,
        window.top + zs_shell_content_y(dpi) + zs_shell_scale(ZS_SHELL_VIEWPORT_MASK_H, dpi),
    )
}

fn zs_shell_safe_paint_rect_for_window(window: UiRect, dpi: Dpi) -> UiRect {
    let mask = zs_shell_viewport_mask_rect_for_window(window, dpi);
    UiRect::new(mask.left, mask.bottom, mask.right, window.bottom)
}

fn zs_shell_scroll_layout_for_window(
    window: UiRect,
    content_height: i32,
    bar_width: i32,
    dpi: Dpi,
) -> ZsShellScrollLayout {
    let content_y = window.top + zs_shell_content_y(dpi);
    let view_h = (window.bottom - window.top) - zs_shell_content_y(dpi);
    ZsShellScrollLayout::new(
        content_y,
        window.bottom,
        content_height,
        view_h,
        window.right,
        zs_shell_scale(ZS_SHELL_SCROLL_BAR_MARGIN, dpi),
        bar_width,
        dpi,
    )
}

fn rect_to_ui(rect: Rect) -> UiRect {
    UiRect::new(rect.x, rect.y, rect.x + rect.width, rect.y + rect.height)
}

fn ui_to_rect(rect: UiRect) -> Rect {
    Rect {
        x: rect.left,
        y: rect.top,
        width: (rect.right - rect.left).max(0),
        height: (rect.bottom - rect.top).max(0),
    }
}

fn rect_contains(rect: Rect, point: Point) -> bool {
    point.x >= rect.x
        && point.x < rect.x + rect.width
        && point.y >= rect.y
        && point.y < rect.y + rect.height
}

fn shell_role_to_color_role(role: ZsShellThemeRole) -> ColorRole {
    match role {
        ZsShellThemeRole::Accent
        | ZsShellThemeRole::AccentHover
        | ZsShellThemeRole::AccentPressed
        | ZsShellThemeRole::ScrollbarThumbDragging => ColorRole::Accent,
        ZsShellThemeRole::Surface | ZsShellThemeRole::White => ColorRole::Surface,
        ZsShellThemeRole::Text => ColorRole::PrimaryText,
        ZsShellThemeRole::TextMuted => ColorRole::SecondaryText,
        ZsShellThemeRole::Danger => ColorRole::Danger,
        _ => ColorRole::Control,
    }
}

fn shell_text_style(command: &ZsShellTextCommand) -> SemanticTextStyle {
    SemanticTextStyle {
        role: match command.font {
            ZsShellTextFontRole::Display => TextRole::Title,
            ZsShellTextFontRole::FluentIcon => TextRole::Icon,
            ZsShellTextFontRole::UiText => {
                if command.size <= 12 {
                    TextRole::Caption
                } else {
                    TextRole::Body
                }
            }
        },
        color: shell_role_to_color_role(command.color),
        weight: if command.bold {
            TextWeight::Bold
        } else {
            TextWeight::Regular
        },
        horizontal_align: match command.align {
            ZsShellTextAlign::Left => HorizontalAlign::Start,
            ZsShellTextAlign::Center => HorizontalAlign::Center,
            ZsShellTextAlign::Right => HorizontalAlign::End,
        },
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: true,
    }
}

fn collect_duplicate_ids<'a>(
    issues: &mut Vec<String>,
    scope: &str,
    ids: impl IntoIterator<Item = &'a str>,
) {
    let mut seen = Vec::new();
    for id in ids {
        if id.trim().is_empty() {
            issues.push(format!("{scope} id is empty"));
        } else if seen.contains(&id) {
            issues.push(format!("duplicate {scope} id `{id}`"));
        } else {
            seen.push(id);
        }
    }
}

pub type ZsNavigationScaffoldSpec = ZsShellLayoutSpec;
pub type ZsNavItemSpec = ZsShellNavItemSpec;
pub type ZsGroupCardSpec = ZsShellGroupCardSpec;
pub type ZsContentRowSpec = ZsShellContentRowSpec;
pub type ZsRowAccessory = ZsShellRowAccessory;
pub type ZsActionAreaSpec = ZsShellActionAreaSpec;
pub type ZsActionButtonSpec = ZsShellActionButtonSpec;
pub type ZsActionButtonKind = ZsShellActionButtonKind;
pub type ZsNavigationLayoutMetrics = ZsShellLayoutMetrics;
pub type ZsNavigationLayoutPlan = ZsShellLayoutPlan;
pub type ZsNavigationLayoutRegion = ZsShellLayoutRegion;
pub type ZsNavigationLayoutRegionKind = ZsShellLayoutRegionKind;
pub type ZsNavigationScaffoldAudit = ZsShellLayoutAudit;

#[cfg(test)]
mod tests {
    use super::*;

    fn shell() -> ZsShellLayoutSpec {
        ZsShellLayoutSpec::new("preferences", "Preferences")
            .app_title("设置")
            .selected_nav("general")
            .nav_item(ZsShellNavItemSpec::new("general", "General").icon("settings"))
            .nav_item(
                ZsShellNavItemSpec::new("sync", "Sync")
                    .icon("cloud")
                    .badge(true),
            )
            .card(
                ZsShellGroupCardSpec::new("clipboard", "Clipboard")
                    .row(
                        ZsShellContentRowSpec::new("capture", "Capture clipboard")
                            .description("Keep copied text from other applications")
                            .accessory(ZsShellRowAccessory::toggle(true)),
                    )
                    .row(
                        ZsShellContentRowSpec::new("ignored", "Ignored apps")
                            .description("Skip sensitive windows")
                            .accessory(ZsShellRowAccessory::button("Manage", "ignored.manage")),
                    ),
            )
            .action_area(
                ZsShellActionAreaSpec::new()
                    .secondary(ZsShellActionButtonSpec::secondary("cancel", "Cancel"))
                    .primary(ZsShellActionButtonSpec::primary("save", "Save")),
            )
    }

    #[test]
    fn shell_layout_preserves_window_nav_content_and_card_metrics() {
        let plan = shell().layout_plan(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
        );

        assert_eq!(
            plan.region("navigation").unwrap().rect,
            Rect {
                x: 0,
                y: 0,
                width: 236,
                height: 740
            }
        );
        assert_eq!(
            plan.region("content.header").unwrap().rect,
            Rect {
                x: 272,
                y: 32,
                width: 324,
                height: 30
            }
        );
        assert_eq!(
            plan.region("clipboard").unwrap().rect,
            Rect {
                x: 264,
                y: 100,
                width: 808,
                height: 166
            }
        );
        assert_eq!(plan.region("capture").unwrap().rect.y, 152);
    }

    #[test]
    fn shell_scroll_layout_keeps_thumb_track_and_drag_math() {
        let layout = ZsShellScrollLayout::new(100, 600, 1000, 500, 800, 3, 5, Dpi::standard());

        assert_eq!(layout.max_scroll(), 500);
        assert_eq!(layout.track_rect(), Some(UiRect::new(792, 108, 797, 592)));
        assert_eq!(layout.thumb_rect(0), Some(UiRect::new(792, 108, 797, 350)));
        assert_eq!(
            layout.thumb_rect(250),
            Some(UiRect::new(792, 229, 797, 471))
        );
        assert_eq!(layout.track_click_scroll_target(350), Some(250));
        assert_eq!(layout.drag_scroll_target(229, 250, 471), Some(500));
    }

    #[test]
    fn shell_paint_plan_uses_nav_card_scrollbar_command_shape() {
        let spec = shell()
            .card(
                ZsShellGroupCardSpec::new("advanced", "Advanced")
                    .extra_px(900)
                    .row(ZsShellContentRowSpec::new("extra", "Extra")),
            )
            .scroll_y(20)
            .scrollbar(true, true);
        let plan = spec.paint_plan(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
        );

        assert!(plan.paint_commands.iter().any(|command| matches!(
            command,
            ZsShellPaintCommand::RoundRect {
                radius: 8,
                fill: ZsShellThemeRole::Surface,
                stroke: ZsShellThemeRole::Stroke,
                ..
            }
        )));
        assert!(plan.paint_commands.iter().any(|command| matches!(
            command,
            ZsShellPaintCommand::RoundFill {
                fill: ZsShellThemeRole::ScrollbarThumbDragging,
                ..
            }
        )));
        assert!(plan.text_commands.iter().any(|command| {
            command.content == ZsShellTextContent::Label("Clipboard".to_string())
                && command.size == 12
                && command.bold
        }));
    }

    #[test]
    fn shell_native_draw_plan_projects_paint_plan() {
        let draw = shell().native_draw_plan(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
        );

        assert!(draw.command_count() >= 20);
        assert!(draw.text_count() >= 10);
        assert!(draw
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PushClip { .. })));
        assert!(draw
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::PopClip)));
    }

    #[test]
    fn shell_pointer_transitions_keep_nav_hover_and_selection_shape() {
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 1100,
            height: 740,
        };
        let mut runtime = ZsShellRuntime::new(shell(), bounds, Dpi::standard());

        let hover = runtime.pointer_move(Point { x: 40, y: 140 });
        assert!(hover.redraw);
        assert_eq!(runtime.spec.hovered_nav_id.as_deref(), Some("sync"));
        assert_eq!(hover.invalidate_rects.len(), 1);

        let selected = runtime.pointer_down(Point { x: 40, y: 140 });
        assert_eq!(runtime.spec.selected_nav_id.as_deref(), Some("sync"));
        assert_eq!(
            selected.events,
            vec![ZsShellInteractionEvent::NavigationSelected {
                id: "sync".to_string()
            }]
        );
    }

    #[test]
    fn shell_runtime_routes_accessories_without_product_state() {
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 1100,
            height: 740,
        };
        let mut runtime = ZsShellRuntime::new(shell(), bounds, Dpi::standard());
        let accessory = runtime
            .spec
            .layout_plan(bounds, Dpi::standard())
            .region("capture.accessory")
            .unwrap()
            .rect;
        let update = runtime.pointer_down(Point {
            x: accessory.x + accessory.width / 2,
            y: accessory.y + accessory.height / 2,
        });

        assert_eq!(
            update.events,
            vec![ZsShellInteractionEvent::ToggleChanged {
                row_id: "capture".to_string(),
                checked: false,
            }]
        );
        assert!(matches!(
            runtime.spec.cards[0].rows[0].accessory,
            ZsShellRowAccessory::Toggle { checked: false }
        ));
    }

    #[test]
    fn shell_runtime_routes_wheel_and_scrollbar_drag_with_shared_math() {
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 1100,
            height: 740,
        };
        let spec = shell().card(
            ZsShellGroupCardSpec::new("advanced", "Advanced")
                .extra_px(900)
                .row(ZsShellContentRowSpec::new("extra", "Extra")),
        );
        let mut runtime = ZsShellRuntime::new(spec, bounds, Dpi::standard());

        let wheel = runtime.scroll_by(96);
        assert_eq!(runtime.spec.scroll_y, 96);
        assert_eq!(
            wheel.events,
            vec![ZsShellInteractionEvent::ScrollChanged { scroll_y: 96 }]
        );

        let thumb = runtime
            .spec
            .scroll_layout(bounds, Dpi::standard(), true)
            .thumb_rect(runtime.spec.scroll_y)
            .unwrap();
        let start = Point {
            x: (thumb.left + thumb.right) / 2,
            y: (thumb.top + thumb.bottom) / 2,
        };
        assert!(matches!(
            zs_shell_pointer_down_target(&runtime.spec, bounds, Dpi::standard(), start),
            ZsShellPointerDownTarget::ScrollbarThumb { .. }
        ));
        runtime.pointer_down(start);
        assert!(runtime.spec.scrollbar_dragging);
        let dragged = runtime.pointer_move(Point {
            x: start.x,
            y: start.y + 80,
        });
        assert!(dragged.redraw);
        assert!(runtime.spec.scroll_y > 96);
        runtime.pointer_up();
        assert!(!runtime.spec.scrollbar_dragging);
    }

    #[test]
    fn shell_audit_rejects_duplicate_rows_and_missing_nav_selection() {
        let spec = ZsShellLayoutSpec::new("demo", "Demo")
            .selected_nav("missing")
            .nav_item(ZsShellNavItemSpec::new("general", "General"))
            .card(
                ZsShellGroupCardSpec::new("group", "Group")
                    .row(ZsShellContentRowSpec::new("same", "One"))
                    .row(ZsShellContentRowSpec::new("same", "Two")),
            );

        let audit = spec.audit();

        assert!(!audit.valid);
        assert_eq!(audit.issue_count, 2);
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.contains("duplicate row")));
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.contains("selected navigation item")));
    }
}
