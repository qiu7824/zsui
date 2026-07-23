use serde::{Deserialize, Serialize};

use crate::platform_component_profile::{
    PlatformComponentProfile, PlatformShellNavigationComposition, PlatformShellProfile,
    PlatformShellSectionComposition,
};
#[cfg(test)]
use crate::ZsPlatformStyle;
use crate::{
    zs_toggle_render_plan_for_platform, ColorRole, Dp, Dpi, HorizontalAlign, NativeDrawCommand,
    NativeDrawFill, NativeDrawIconCommand, NativeDrawPlan, NativeDrawTextCommand,
    NativeIconColorMode, Point, Rect, SemanticTextStyle, TextRole, TextWeight, TextWrap, UiRect,
    VerticalAlign, ZsIcon,
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

fn current_shell_profile() -> PlatformShellProfile {
    PlatformComponentProfile::current().shell
}

#[cfg(test)]
fn shell_profile_for_style(style: ZsPlatformStyle) -> PlatformShellProfile {
    PlatformComponentProfile::for_style(style).shell
}

fn shell_dp(value: Dp, dpi: Dpi) -> i32 {
    value.to_px(dpi).round_i32()
}

fn shell_centered_square(rect: UiRect, size: i32, dpi: Dpi) -> UiRect {
    let available_width = (rect.right - rect.left).max(0);
    let available_height = (rect.bottom - rect.top).max(0);
    let side = zs_shell_scale(size.max(1), dpi)
        .min(available_width)
        .min(available_height);
    let left = rect.left + (available_width - side) / 2;
    let top = rect.top + (available_height - side) / 2;
    UiRect::new(left, top, left + side, top + side)
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
        self.scroll_layout_with_profile(bounds, dpi, active, current_shell_profile())
    }

    fn scroll_layout_with_profile(
        &self,
        bounds: Rect,
        dpi: Dpi,
        active: bool,
        profile: PlatformShellProfile,
    ) -> ZsShellScrollLayout {
        let window = rect_to_ui(bounds);
        let bar_width = if active {
            shell_dp(profile.active_scrollbar_width, dpi)
        } else {
            shell_dp(profile.scrollbar_width, dpi)
        };
        zs_shell_scroll_layout_for_window_with_profile(
            window,
            self.content_total_h_with_profile(window, dpi, profile),
            bar_width,
            dpi,
            profile,
        )
    }

    pub fn max_scroll(&self, bounds: Rect, dpi: Dpi) -> i32 {
        self.scroll_layout(bounds, dpi, false).max_scroll()
    }

    pub fn layout_plan(&self, bounds: Rect, dpi: Dpi) -> ZsShellLayoutPlan {
        self.layout_plan_with_profile(bounds, dpi, current_shell_profile())
    }

    pub fn layout_plan_with_metrics(
        &self,
        bounds: Rect,
        dpi: Dpi,
        _metrics: ZsShellLayoutMetrics,
    ) -> ZsShellLayoutPlan {
        self.layout_plan_with_profile(bounds, dpi, current_shell_profile())
    }

    #[cfg(test)]
    fn layout_plan_for_style(
        &self,
        bounds: Rect,
        dpi: Dpi,
        style: ZsPlatformStyle,
    ) -> ZsShellLayoutPlan {
        self.layout_plan_with_profile(bounds, dpi, shell_profile_for_style(style))
    }

    fn layout_plan_with_profile(
        &self,
        bounds: Rect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> ZsShellLayoutPlan {
        let mut regions = Vec::new();
        let window = rect_to_ui(bounds);
        let chrome = zs_shell_chrome_render_plan_with_profile(window, dpi, profile);

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
            ui_to_rect(zs_shell_viewport_rect_for_window_with_profile(
                window, dpi, profile,
            )),
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
                ui_to_rect(zs_shell_nav_item_rect_with_profile(
                    window, index, dpi, profile,
                )),
            ));
        }

        for section in self.sections_with_profile(window, dpi, profile) {
            regions.push(ZsShellLayoutRegion::new(
                section.id.clone(),
                ZsShellLayoutRegionKind::GroupCard,
                ui_to_rect(section.rect.offset_y(self.scroll_y)),
            ));
            let layout = ZsShellFormSectionLayout::from_section_with_profile(
                section.rect.offset_y(self.scroll_y),
                dpi,
                profile,
            );
            if let Some(card) = self.cards.iter().find(|card| card.id == section.id) {
                for (row_index, row) in card.rows.iter().enumerate() {
                    let row_rect = layout.row_rect(row_index as i32);
                    regions.push(ZsShellLayoutRegion::new(
                        row.id.clone(),
                        ZsShellLayoutRegionKind::ContentRow,
                        ui_to_rect(row_rect),
                    ));
                    if let Some(accessory_rect) = layout.accessory_rect(
                        row_index as i32,
                        row.accessory.width_with_profile(dpi, profile),
                    ) {
                        regions.push(ZsShellLayoutRegion::new(
                            format!("{}.accessory", row.id),
                            ZsShellLayoutRegionKind::RowAccessory,
                            ui_to_rect(accessory_rect),
                        ));
                    }
                }
            }
        }

        if let Some(action_rect) =
            zs_shell_action_area_rect_with_profile(window, &self.action_area, dpi, profile)
        {
            regions.push(ZsShellLayoutRegion::new(
                "actions",
                ZsShellLayoutRegionKind::ActionArea,
                ui_to_rect(action_rect),
            ));
            for (id, rect) in
                zs_shell_action_button_rects_with_profile(window, &self.action_area, dpi, profile)
            {
                regions.push(ZsShellLayoutRegion::new(
                    id,
                    ZsShellLayoutRegionKind::ActionButton,
                    ui_to_rect(rect),
                ));
            }
        }

        if let Some(scrollbar) = self.scrollbar_render_plan_with_profile(window, dpi, profile) {
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
        let (mut chrome_and_nav, content, trailing, _) =
            self.paint_plan_parts_with_profile(bounds, dpi, current_shell_profile());
        chrome_and_nav.extend(content);
        chrome_and_nav.extend(trailing);
        chrome_and_nav
    }

    #[cfg(test)]
    fn paint_plan_for_style(
        &self,
        bounds: Rect,
        dpi: Dpi,
        style: ZsPlatformStyle,
    ) -> ZsShellPaintPlan {
        let (mut chrome_and_nav, content, trailing, _) =
            self.paint_plan_parts_with_profile(bounds, dpi, shell_profile_for_style(style));
        chrome_and_nav.extend(content);
        chrome_and_nav.extend(trailing);
        chrome_and_nav
    }

    pub fn native_draw_plan(&self, bounds: Rect, dpi: Dpi) -> NativeDrawPlan {
        self.native_draw_plan_with_profile(bounds, dpi, current_shell_profile())
    }

    #[cfg(test)]
    fn native_draw_plan_for_style(
        &self,
        bounds: Rect,
        dpi: Dpi,
        style: ZsPlatformStyle,
    ) -> NativeDrawPlan {
        self.native_draw_plan_with_profile(bounds, dpi, shell_profile_for_style(style))
    }

    fn native_draw_plan_with_profile(
        &self,
        bounds: Rect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> NativeDrawPlan {
        let (chrome_and_nav, content, trailing, content_clip_rect) =
            self.paint_plan_parts_with_profile(bounds, dpi, profile);
        let mut plan = chrome_and_nav.to_native_draw_plan_with_dpi(dpi);
        plan.push(NativeDrawCommand::PushClip {
            rect: ui_to_rect(content_clip_rect),
        });
        plan.commands
            .extend(content.to_native_draw_plan_with_dpi(dpi).commands);
        plan.push(NativeDrawCommand::PopClip);
        plan.commands
            .extend(trailing.to_native_draw_plan_with_dpi(dpi).commands);
        plan
    }

    fn paint_plan_parts_with_profile(
        &self,
        bounds: Rect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> (ZsShellPaintPlan, ZsShellPaintPlan, ZsShellPaintPlan, UiRect) {
        let window = rect_to_ui(bounds);
        let chrome = zs_shell_chrome_render_plan_with_profile(window, dpi, profile);
        let mut chrome_and_nav = ZsShellPaintPlan::default();
        chrome_and_nav.extend(zs_shell_chrome_paint_plan_with_profile(
            &chrome,
            self.app_title.clone(),
            self.title.clone(),
            profile,
        ));

        let selected_id = self.selected_nav_id.as_deref();
        let hovered_id = self.hovered_nav_id.as_deref();
        for (index, item) in self.nav_items.iter().enumerate() {
            let item_rect = zs_shell_nav_item_rect_with_profile(window, index, dpi, profile);
            chrome_and_nav.extend(zs_shell_nav_item_paint_plan(
                &ZsShellNavItemRender {
                    id: item.id.clone(),
                    index,
                    label: item.label.clone(),
                    icon: item.icon,
                    rect: item_rect,
                    selected: selected_id == Some(item.id.as_str())
                        || (selected_id.is_none() && index == 0),
                    hovered: hovered_id == Some(item.id.as_str()),
                    badge_rect: item.badge.then(|| zs_shell_nav_badge_rect(item_rect, dpi)),
                },
                dpi,
                profile,
            ));
        }

        let content = ZsShellContentRenderPlan {
            sections: self.sections_with_profile(window, dpi, profile),
            scroll_y: self.scroll_y,
        };
        let mut content_plan = zs_shell_content_paint_plan(&content, dpi, profile);
        content_plan.extend(self.row_paint_plan_with_profile(window, dpi, profile));

        let mut trailing = ZsShellPaintPlan::default();
        if let Some(action_plan) = self.action_area_paint_plan_with_profile(window, dpi, profile) {
            trailing.extend(action_plan);
        }
        if let Some(scrollbar) = self.scrollbar_render_plan_with_profile(window, dpi, profile) {
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
        self.native_draw_plan_with_profile(bounds, dpi, current_shell_profile())
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

    fn sections_with_profile(
        &self,
        window: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> Vec<ZsShellSection> {
        let mut out = Vec::with_capacity(self.cards.len());
        let mut top = shell_dp(profile.content_top_gap, dpi);
        let gap = shell_dp(profile.section_gap, dpi);
        let content_x = zs_shell_content_x_with_profile(window, dpi, profile);
        let content_w = zs_shell_content_w_with_profile(window, dpi, profile);
        for card in &self.cards {
            let h = zs_shell_form_section_height_with_extra_profile(
                card.rows.len() as i32,
                card.extra_px,
                dpi,
                profile,
            );
            out.push(ZsShellSection {
                id: card.id.clone(),
                title: card.title.clone(),
                rect: UiRect::new(
                    content_x,
                    window.top + zs_shell_content_y_with_profile(dpi, profile) + top,
                    content_x + content_w,
                    window.top + zs_shell_content_y_with_profile(dpi, profile) + top + h,
                ),
            });
            top += h + gap;
        }
        out
    }

    fn content_total_h_with_profile(
        &self,
        window: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> i32 {
        self.sections_with_profile(window, dpi, profile)
            .iter()
            .map(|section| {
                section.rect.bottom - window.top - zs_shell_content_y_with_profile(dpi, profile)
                    + shell_dp(profile.content_top_gap, dpi)
            })
            .max()
            .unwrap_or(0)
            .max(0)
    }

    fn scrollbar_render_plan_with_profile(
        &self,
        window: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> Option<ZsShellScrollbarRenderPlan> {
        zs_shell_scrollbar_render_plan_with_profile(
            window,
            self.content_total_h_with_profile(window, dpi, profile),
            self.scroll_y,
            self.scrollbar_visible,
            self.scrollbar_dragging,
            dpi,
            profile,
        )
    }

    fn row_paint_plan_with_profile(
        &self,
        window: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> ZsShellPaintPlan {
        let mut plan = ZsShellPaintPlan::default();
        for section in self.sections_with_profile(window, dpi, profile) {
            let Some(card) = self.cards.iter().find(|card| card.id == section.id) else {
                continue;
            };
            let layout = ZsShellFormSectionLayout::from_section_with_profile(
                section.rect.offset_y(self.scroll_y),
                dpi,
                profile,
            );
            for (index, row) in card.rows.iter().enumerate() {
                let row_rect = layout.row_rect(index as i32);
                let accessory_width = row.accessory.width_with_profile(dpi, profile).unwrap_or(0);
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
                if profile.draw_row_separators && index + 1 < card.rows.len() {
                    let separator_top = row_rect.bottom
                        + if profile.sections == PlatformShellSectionComposition::FluentCards {
                            zs_shell_scale(3, dpi)
                        } else {
                            0
                        };
                    plan.paint_commands.push(ZsShellPaintCommand::FillRect {
                        rect: UiRect::new(
                            row_rect.left,
                            separator_top,
                            row_rect.right,
                            separator_top + 1,
                        ),
                        fill: ZsShellThemeRole::Stroke,
                    });
                }
                if let Some(accessory_rect) = layout
                    .accessory_rect(index as i32, row.accessory.width_with_profile(dpi, profile))
                {
                    plan.extend(row.accessory.paint_plan_with_profile(
                        accessory_rect,
                        dpi,
                        profile,
                    ));
                }
            }
        }
        plan
    }

    fn action_area_paint_plan_with_profile(
        &self,
        window: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> Option<ZsShellPaintPlan> {
        if self.action_area.is_empty() {
            return None;
        }
        let mut plan = ZsShellPaintPlan::default();
        for (button, rect) in self.action_area.buttons().zip(
            zs_shell_action_button_rects_with_profile(window, &self.action_area, dpi, profile)
                .into_iter()
                .map(|(_, rect)| rect),
        ) {
            plan.extend(zs_shell_button_paint_plan_with_profile(
                rect,
                button.label.clone(),
                button.kind.into(),
                false,
                false,
                dpi,
                profile,
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

    /// Uses a framework semantic icon while leaving the platform backend in
    /// charge of resolving Segoe Fluent Icons, SF Symbols, or the GTK theme.
    pub fn semantic_icon(mut self, icon: ZsIcon) -> Self {
        self.icon = ZsShellNavIconKind::Semantic(icon);
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
    Semantic(ZsIcon),
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
            Self::Semantic(icon) => icon.windows_fluent_glyph(),
        }
    }

    pub const fn semantic_icon(self) -> ZsIcon {
        match self {
            Self::General => ZsIcon::Settings,
            Self::Hotkey => ZsIcon::Tool,
            Self::Plugin => ZsIcon::Code,
            Self::Group => ZsIcon::Folder,
            Self::Sync => ZsIcon::Refresh,
            Self::About => ZsIcon::Info,
            Self::Semantic(icon) => icon,
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

    fn width_with_profile(&self, dpi: Dpi, profile: PlatformShellProfile) -> Option<i32> {
        let base_metrics = crate::ZsBaseControlMetrics::for_platform(profile.style);
        match self {
            Self::None => None,
            Self::Value { .. } => Some(zs_shell_scale(168, dpi)),
            Self::Toggle { .. } => Some(zs_shell_scale(44, dpi)),
            Self::Button { label, .. } | Self::AccentButton { label, .. } => Some(shell_dp(
                Dp::new(128.0_f32.max(base_metrics.button_minimum_width_for_label(label).0)),
                dpi,
            )),
            Self::Dropdown { selected, .. } => Some(shell_dp(
                Dp::new(
                    176.0_f32.max(
                        base_metrics
                            .estimated_text_width_with_shaping_reserve(selected)
                            .0
                            + base_metrics.button_padding_left.0
                            + base_metrics.button_padding_right.0
                            + 28.0,
                    ),
                ),
                dpi,
            )),
        }
    }

    fn paint_plan_with_profile(
        &self,
        rect: UiRect,
        dpi: Dpi,
        profile: PlatformShellProfile,
    ) -> ZsShellPaintPlan {
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
            Self::Toggle { checked } => {
                zs_shell_toggle_paint_plan_with_profile(rect, false, *checked, dpi, profile)
            }
            Self::Button { label, .. } => zs_shell_button_paint_plan_with_profile(
                rect,
                label.clone(),
                ZsShellComponentKind::Button,
                false,
                false,
                dpi,
                profile,
            ),
            Self::AccentButton { label, .. } => zs_shell_button_paint_plan_with_profile(
                rect,
                label.clone(),
                ZsShellComponentKind::AccentButton,
                false,
                false,
                dpi,
                profile,
            ),
            Self::Dropdown { selected, .. } => zs_shell_button_paint_plan_with_profile(
                rect,
                selected.clone(),
                ZsShellComponentKind::Dropdown,
                false,
                false,
                dpi,
                profile,
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
    Button,
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
        self.to_native_draw_plan_with_dpi(Dpi::standard())
    }

    pub fn to_native_draw_plan_with_dpi(&self, dpi: Dpi) -> NativeDrawPlan {
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
            match &command.content {
                ZsShellTextContent::Label(label) => {
                    plan.push(NativeDrawCommand::Text(NativeDrawTextCommand::new(
                        label,
                        ui_to_rect(command.rect),
                        shell_text_style(command),
                    )));
                }
                ZsShellTextContent::NavIcon(icon) => {
                    plan.push(NativeDrawCommand::Icon(
                        NativeDrawIconCommand::new(
                            icon.semantic_icon(),
                            ui_to_rect(shell_centered_square(command.rect, command.size, dpi)),
                            NativeIconColorMode::ThemeAware,
                        )
                        .with_color(shell_role_to_color_role(command.color)),
                    ));
                }
                ZsShellTextContent::ChromeMenuIcon => {
                    plan.push(NativeDrawCommand::Icon(
                        NativeDrawIconCommand::new(
                            ZsIcon::Sidebar,
                            ui_to_rect(shell_centered_square(command.rect, command.size, dpi)),
                            NativeIconColorMode::ThemeAware,
                        )
                        .with_color(shell_role_to_color_role(command.color)),
                    ));
                }
                ZsShellTextContent::DropdownArrow => {
                    plan.push(NativeDrawCommand::Icon(
                        NativeDrawIconCommand::new(
                            ZsIcon::ChevronDown,
                            ui_to_rect(shell_centered_square(command.rect, command.size, dpi)),
                            NativeIconColorMode::ThemeAware,
                        )
                        .with_color(shell_role_to_color_role(command.color)),
                    ));
                }
            }
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
    zs_shell_chrome_render_plan_with_profile(window, dpi, current_shell_profile())
}

fn zs_shell_chrome_render_plan_with_profile(
    window: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellChromeRenderPlan {
    let viewport_mask_rect =
        zs_shell_viewport_mask_rect_for_window_with_profile(window, dpi, profile);
    let nav_w = zs_shell_nav_w_with_profile(window, dpi, profile);
    let menu_left = window.left + shell_dp(profile.menu_icon_x, dpi);
    let menu_top = window.top + shell_dp(profile.menu_icon_y, dpi);
    let menu_size = shell_dp(profile.menu_icon_size, dpi);
    let app_title_left = window.left + shell_dp(profile.app_title_x, dpi);
    let app_title_top = window.top + shell_dp(profile.app_title_y, dpi);
    ZsShellChromeRenderPlan {
        window_rect: window,
        nav_rect: UiRect::new(window.left, window.top, window.left + nav_w, window.bottom),
        divider_x: window.left + nav_w,
        menu_icon_rect: UiRect::new(
            menu_left,
            menu_top,
            menu_left + menu_size,
            menu_top + menu_size,
        ),
        app_title_rect: UiRect::new(
            app_title_left,
            app_title_top,
            app_title_left + shell_dp(profile.app_title_width, dpi),
            app_title_top + shell_dp(profile.app_title_height, dpi),
        ),
        page_title_rect: zs_shell_title_rect_with_profile(window, dpi, profile),
        content_clip_rect: zs_shell_safe_paint_rect_for_window_with_profile(window, dpi, profile),
        viewport_mask_separator_rect: UiRect::new(
            viewport_mask_rect.left + shell_dp(profile.section_horizontal_padding, dpi),
            viewport_mask_rect.bottom - 1,
            viewport_mask_rect.right - shell_dp(profile.section_horizontal_padding, dpi),
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
    zs_shell_chrome_paint_plan_with_profile(plan, app_title, page_title, current_shell_profile())
}

fn zs_shell_chrome_paint_plan_with_profile(
    plan: &ZsShellChromeRenderPlan,
    app_title: String,
    page_title: String,
    profile: PlatformShellProfile,
) -> ZsShellPaintPlan {
    let mut text_commands = Vec::with_capacity(3);
    if profile.show_menu_icon {
        text_commands.push(ZsShellTextCommand {
            rect: plan.menu_icon_rect,
            content: ZsShellTextContent::ChromeMenuIcon,
            color: ZsShellThemeRole::TextMuted,
            size: 16,
            bold: false,
            font: ZsShellTextFontRole::FluentIcon,
            align: ZsShellTextAlign::Center,
        });
    }
    text_commands.push(ZsShellTextCommand {
        rect: plan.app_title_rect,
        content: ZsShellTextContent::Label(app_title),
        color: ZsShellThemeRole::Text,
        size: 15,
        bold: true,
        font: ZsShellTextFontRole::UiText,
        align: ZsShellTextAlign::Left,
    });
    text_commands.push(ZsShellTextCommand {
        rect: plan.page_title_rect,
        content: ZsShellTextContent::Label(page_title),
        color: ZsShellThemeRole::Text,
        size: 24,
        bold: true,
        font: ZsShellTextFontRole::Display,
        align: ZsShellTextAlign::Left,
    });
    ZsShellPaintPlan {
        paint_commands: vec![
            ZsShellPaintCommand::FillRect {
                rect: plan.nav_rect,
                fill: match profile.navigation {
                    PlatformShellNavigationComposition::GtkSidebar => ZsShellThemeRole::Background,
                    PlatformShellNavigationComposition::FluentPane
                    | PlatformShellNavigationComposition::AppKitSourceList => {
                        ZsShellThemeRole::NavBackground
                    }
                },
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
        text_commands,
    }
}

fn zs_shell_nav_item_paint_plan(
    item: &ZsShellNavItemRender,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellPaintPlan {
    let mut paint_commands = Vec::new();
    if item.selected {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect: item.rect,
            fill: match profile.navigation {
                PlatformShellNavigationComposition::AppKitSourceList => ZsShellThemeRole::Accent,
                PlatformShellNavigationComposition::FluentPane
                | PlatformShellNavigationComposition::GtkSidebar => {
                    ZsShellThemeRole::NavSelectedFill
                }
            },
            radius: shell_dp(profile.navigation_item_radius, dpi),
        });
        if profile.navigation == PlatformShellNavigationComposition::FluentPane {
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
        }
    } else if item.hovered {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect: item.rect,
            fill: ZsShellThemeRole::NavHoverFill,
            radius: shell_dp(profile.navigation_item_radius, dpi),
        });
    }
    if let Some(rect) = item.badge_rect {
        paint_commands.push(ZsShellPaintCommand::RoundFill {
            rect,
            fill: ZsShellThemeRole::Danger,
            radius: zs_shell_scale(5, dpi),
        });
    }

    let appkit_selected =
        item.selected && profile.navigation == PlatformShellNavigationComposition::AppKitSourceList;
    let icon_color = if appkit_selected {
        ZsShellThemeRole::White
    } else if item.selected {
        match profile.navigation {
            PlatformShellNavigationComposition::FluentPane => ZsShellThemeRole::Accent,
            PlatformShellNavigationComposition::GtkSidebar => ZsShellThemeRole::Text,
            PlatformShellNavigationComposition::AppKitSourceList => unreachable!(),
        }
    } else if item.hovered {
        ZsShellThemeRole::Text
    } else {
        ZsShellThemeRole::TextMuted
    };
    let label_color = if appkit_selected {
        ZsShellThemeRole::White
    } else if item.selected || item.hovered {
        ZsShellThemeRole::Text
    } else {
        ZsShellThemeRole::TextMuted
    };
    let icon_column = (item.rect.bottom - item.rect.top).min(zs_shell_scale(28, dpi));
    let icon_rect = UiRect::new(
        item.rect.left + zs_shell_scale(6, dpi),
        item.rect.top,
        item.rect.left + zs_shell_scale(6, dpi) + icon_column,
        item.rect.bottom,
    );
    let label_rect = UiRect::new(
        icon_rect.right + zs_shell_scale(4, dpi),
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

fn zs_shell_content_paint_plan(
    plan: &ZsShellContentRenderPlan,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellPaintPlan {
    let mut paint_commands = Vec::with_capacity(plan.sections.len());
    let mut text_commands = Vec::with_capacity(plan.sections.len());
    for section in &plan.sections {
        let rect = section.rect.offset_y(plan.scroll_y);
        match profile.sections {
            PlatformShellSectionComposition::FluentCards => {
                paint_commands.push(ZsShellPaintCommand::RoundRect {
                    rect,
                    fill: ZsShellThemeRole::Surface,
                    stroke: ZsShellThemeRole::Stroke,
                    radius: shell_dp(profile.section_radius, dpi),
                });
            }
            PlatformShellSectionComposition::AppKitForms => {}
            PlatformShellSectionComposition::GtkBoxedLists => {
                let surface_top = rect.top + shell_dp(profile.section_header_height, dpi);
                paint_commands.push(ZsShellPaintCommand::RoundRect {
                    rect: UiRect::new(rect.left, surface_top, rect.right, rect.bottom),
                    fill: ZsShellThemeRole::Surface,
                    stroke: ZsShellThemeRole::Stroke,
                    radius: shell_dp(profile.section_radius, dpi),
                });
            }
        }
        let horizontal_padding = match profile.sections {
            PlatformShellSectionComposition::FluentCards => zs_shell_scale(16, dpi),
            PlatformShellSectionComposition::AppKitForms
            | PlatformShellSectionComposition::GtkBoxedLists => 0,
        };
        let title_top = rect.top
            + match profile.sections {
                PlatformShellSectionComposition::FluentCards => zs_shell_scale(12, dpi),
                PlatformShellSectionComposition::AppKitForms
                | PlatformShellSectionComposition::GtkBoxedLists => 0,
            };
        text_commands.push(ZsShellTextCommand {
            rect: UiRect::new(
                rect.left + horizontal_padding,
                title_top,
                rect.right - horizontal_padding,
                rect.top + shell_dp(profile.section_header_height, dpi),
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
    zs_shell_scrollbar_render_plan_with_profile(
        window,
        content_height,
        scroll_y,
        visible,
        dragging,
        dpi,
        current_shell_profile(),
    )
}

fn zs_shell_scrollbar_render_plan_with_profile(
    window: UiRect,
    content_height: i32,
    scroll_y: i32,
    visible: bool,
    dragging: bool,
    dpi: Dpi,
    profile: PlatformShellProfile,
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
        shell_dp(profile.active_scrollbar_width, dpi)
    } else {
        shell_dp(profile.scrollbar_width, dpi)
    };
    let layout = zs_shell_scroll_layout_for_window_with_profile(
        window,
        content_height,
        bar_width,
        dpi,
        profile,
    );
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
    zs_shell_form_section_height_with_extra_profile(rows, extra_px, dpi, current_shell_profile())
}

fn zs_shell_form_section_height_with_extra_profile(
    rows: i32,
    extra_px: i32,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> i32 {
    let rows = rows.max(1);
    shell_dp(profile.section_header_height, dpi)
        + rows * shell_dp(profile.section_row_height, dpi)
        + (rows - 1) * shell_dp(profile.section_row_gap, dpi)
        + shell_dp(profile.section_height_extra, dpi)
        + zs_shell_scale(extra_px.max(0), dpi)
}

#[derive(Clone, Copy)]
pub struct ZsShellFormSectionLayout {
    body: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
}

impl ZsShellFormSectionLayout {
    pub fn from_section(section: UiRect, dpi: Dpi) -> Self {
        Self::from_section_with_profile(section, dpi, current_shell_profile())
    }

    fn from_section_with_profile(section: UiRect, dpi: Dpi, profile: PlatformShellProfile) -> Self {
        let pad = shell_dp(profile.section_horizontal_padding, dpi);
        Self {
            body: UiRect::new(
                section.left + pad,
                section.top + shell_dp(profile.section_header_height, dpi),
                section.right - pad,
                section.bottom - shell_dp(profile.section_body_bottom_inset, dpi),
            ),
            dpi,
            profile,
        }
    }

    pub fn row_y(&self, row: i32) -> i32 {
        self.body.top
            + row
                * (shell_dp(self.profile.section_row_height, self.dpi)
                    + shell_dp(self.profile.section_row_gap, self.dpi))
    }

    pub fn row_rect(&self, row: i32) -> UiRect {
        let y = self.row_y(row);
        UiRect::new(
            self.body.left,
            y,
            self.body.right,
            y + shell_dp(self.profile.section_row_height, self.dpi),
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

fn zs_shell_toggle_paint_plan_with_profile(
    rect: UiRect,
    hover: bool,
    checked: bool,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellPaintPlan {
    let mut paint_commands = vec![ZsShellPaintCommand::FillRect {
        rect,
        fill: ZsShellThemeRole::Surface,
    }];
    let plan =
        zs_toggle_render_plan_for_platform(ui_to_rect(rect), hover, checked, profile.style, dpi);
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

fn zs_shell_button_paint_plan_with_profile(
    rect: UiRect,
    text: String,
    kind: ZsShellComponentKind,
    hover: bool,
    pressed: bool,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellPaintPlan {
    let rr = UiRect::new(rect.left + 1, rect.top + 1, rect.right - 1, rect.bottom - 1);
    let base_metrics = crate::ZsBaseControlMetrics::for_platform(profile.style);
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
                radius: base_metrics.button_radius.to_px(dpi).round_i32(),
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
                radius: base_metrics.button_radius.to_px(dpi).round_i32(),
            });
            let padding_left = shell_dp(base_metrics.button_padding_left, dpi);
            let padding_top = shell_dp(base_metrics.button_padding_top, dpi);
            let padding_right = shell_dp(base_metrics.button_padding_right, dpi);
            let padding_bottom = shell_dp(base_metrics.button_padding_bottom, dpi);
            let content_left = (rr.left + padding_left).min(rr.right);
            let content_top = (rr.top + padding_top).min(rr.bottom);
            let content_right = (rr.right - padding_right).max(content_left);
            let content_bottom = (rr.bottom - padding_bottom).max(content_top);
            text_commands.push(ZsShellTextCommand {
                rect: UiRect::new(content_left, content_top, content_right, content_bottom),
                content: ZsShellTextContent::Label(text),
                color,
                size: 14,
                bold: false,
                font: ZsShellTextFontRole::Button,
                align: ZsShellTextAlign::Center,
            });
        }
    }
    ZsShellPaintPlan {
        paint_commands,
        text_commands,
    }
}

fn zs_shell_action_area_rect_with_profile(
    window: UiRect,
    action_area: &ZsShellActionAreaSpec,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> Option<UiRect> {
    if action_area.is_empty() {
        return None;
    }
    let rects = zs_shell_action_button_rects_with_profile(window, action_area, dpi, profile);
    let first = rects.first()?.1;
    let last = rects.last()?.1;
    Some(UiRect::new(first.left, first.top, last.right, last.bottom))
}

fn zs_shell_action_button_rects_with_profile(
    window: UiRect,
    action_area: &ZsShellActionAreaSpec,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> Vec<(String, UiRect)> {
    let top_margin = shell_dp(profile.action_margin, dpi);
    let btn_h = shell_dp(profile.action_height, dpi);
    let gap = shell_dp(profile.action_gap, dpi);
    let right = window.right - top_margin;
    let base_metrics = crate::ZsBaseControlMetrics::for_platform(profile.style);
    let mut buttons = Vec::with_capacity(action_area.primary.len() + action_area.secondary.len());
    for button in &action_area.secondary {
        let width = profile
            .secondary_action_width
            .0
            .max(base_metrics.button_minimum_width_for_label(&button.label).0);
        buttons.push((button, shell_dp(Dp::new(width), dpi)));
    }
    for button in &action_area.primary {
        let width = profile
            .primary_action_width
            .0
            .max(base_metrics.button_minimum_width_for_label(&button.label).0);
        buttons.push((button, shell_dp(Dp::new(width), dpi)));
    }
    let total_width = buttons.iter().map(|(_, width)| *width).sum::<i32>()
        + gap * buttons.len().saturating_sub(1) as i32;
    let mut cursor_left = right - total_width;
    let mut rects = Vec::with_capacity(buttons.len());
    for (button, width) in buttons {
        let rect = UiRect::new(
            cursor_left,
            window.top + top_margin,
            cursor_left + width,
            window.top + top_margin + btn_h,
        );
        rects.push((button.id.clone(), rect));
        cursor_left += width + gap;
    }
    rects
}

fn zs_shell_content_y_with_profile(dpi: Dpi, profile: PlatformShellProfile) -> i32 {
    shell_dp(profile.top_height, dpi)
}

fn zs_shell_nav_w_with_profile(window: UiRect, dpi: Dpi, profile: PlatformShellProfile) -> i32 {
    let logical_width =
        (window.right - window.left).max(0) as f32 / dpi.scale_factor().max(f32::EPSILON);
    shell_dp(profile.navigation_width(logical_width), dpi)
}

fn zs_shell_content_x_with_profile(window: UiRect, dpi: Dpi, profile: PlatformShellProfile) -> i32 {
    window.left
        + zs_shell_nav_w_with_profile(window, dpi, profile)
        + shell_dp(profile.content_gap, dpi)
}

fn zs_shell_content_w_with_profile(window: UiRect, dpi: Dpi, profile: PlatformShellProfile) -> i32 {
    (window.right
        - window.left
        - zs_shell_nav_w_with_profile(window, dpi, profile)
        - shell_dp(Dp::new(profile.content_gap.0 * 2.0), dpi))
    .max(0)
}

fn zs_shell_title_rect_with_profile(
    window: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> UiRect {
    let left = window.left
        + zs_shell_nav_w_with_profile(window, dpi, profile)
        + shell_dp(profile.title_x, dpi);
    let top = window.top + shell_dp(profile.title_y, dpi);
    UiRect::new(
        left,
        top,
        left + shell_dp(profile.title_width, dpi),
        top + shell_dp(profile.title_height, dpi),
    )
}

fn zs_shell_nav_item_rect(window: UiRect, index: usize, dpi: Dpi) -> UiRect {
    zs_shell_nav_item_rect_with_profile(window, index, dpi, current_shell_profile())
}

fn zs_shell_nav_item_rect_with_profile(
    window: UiRect,
    index: usize,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> UiRect {
    let inset = shell_dp(profile.navigation_item_inset, dpi);
    let x = window.left + inset;
    let y = window.top
        + shell_dp(profile.navigation_start, dpi)
        + index as i32 * shell_dp(profile.navigation_item_stride, dpi);
    UiRect::new(
        x,
        y,
        window.left + zs_shell_nav_w_with_profile(window, dpi, profile) - inset,
        y + shell_dp(profile.navigation_item_height, dpi),
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

fn zs_shell_viewport_rect_for_window_with_profile(
    window: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> UiRect {
    UiRect::new(
        window.left + zs_shell_nav_w_with_profile(window, dpi, profile),
        window.top + zs_shell_content_y_with_profile(dpi, profile),
        window.right,
        window.bottom,
    )
}

fn zs_shell_viewport_mask_rect_for_window_with_profile(
    window: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> UiRect {
    let top = window.top + zs_shell_content_y_with_profile(dpi, profile);
    UiRect::new(
        window.left + zs_shell_nav_w_with_profile(window, dpi, profile),
        top,
        window.right,
        top + shell_dp(profile.viewport_mask_height, dpi),
    )
}

fn zs_shell_safe_paint_rect_for_window_with_profile(
    window: UiRect,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> UiRect {
    let mask = zs_shell_viewport_mask_rect_for_window_with_profile(window, dpi, profile);
    UiRect::new(mask.left, mask.bottom, mask.right, window.bottom)
}

fn zs_shell_scroll_layout_for_window_with_profile(
    window: UiRect,
    content_height: i32,
    bar_width: i32,
    dpi: Dpi,
    profile: PlatformShellProfile,
) -> ZsShellScrollLayout {
    let content_offset = zs_shell_content_y_with_profile(dpi, profile);
    let content_y = window.top + content_offset;
    let view_h = (window.bottom - window.top) - content_offset;
    ZsShellScrollLayout::new(
        content_y,
        window.bottom,
        content_height,
        view_h,
        window.right,
        shell_dp(profile.scrollbar_margin, dpi),
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
        ZsShellThemeRole::Background | ZsShellThemeRole::ScrollbarTrack => ColorRole::Surface,
        ZsShellThemeRole::Surface => ColorRole::SurfaceRaised,
        ZsShellThemeRole::NavBackground => ColorRole::SurfaceRaised,
        ZsShellThemeRole::NavSelectedFill
        | ZsShellThemeRole::NavHoverFill
        | ZsShellThemeRole::ButtonBg
        | ZsShellThemeRole::ButtonHover
        | ZsShellThemeRole::ButtonPressed => ColorRole::Control,
        ZsShellThemeRole::ControlStroke | ZsShellThemeRole::Stroke => ColorRole::Border,
        ZsShellThemeRole::ScrollbarThumb => ColorRole::SecondaryText,
        ZsShellThemeRole::Text => ColorRole::PrimaryText,
        ZsShellThemeRole::TextMuted => ColorRole::SecondaryText,
        ZsShellThemeRole::Danger => ColorRole::Danger,
        ZsShellThemeRole::White => ColorRole::AccentText,
    }
}

fn shell_text_style(command: &ZsShellTextCommand) -> SemanticTextStyle {
    SemanticTextStyle {
        role: match command.font {
            ZsShellTextFontRole::Display => TextRole::Title,
            ZsShellTextFontRole::FluentIcon => TextRole::Icon,
            ZsShellTextFontRole::Button => TextRole::Button,
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
            TextWeight::Semibold
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
        let plan = shell().layout_plan_for_style(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
            ZsPlatformStyle::Windows,
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
    fn one_shell_spec_resolves_distinct_platform_compositions() {
        let spec = shell();
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 1100,
            height: 740,
        };
        let windows = spec.layout_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Windows);
        let macos = spec.layout_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Macos);
        let gtk = spec.layout_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Gtk);

        assert_eq!(windows.region("navigation").unwrap().rect.width, 236);
        assert_eq!(macos.region("navigation").unwrap().rect.width, 240);
        assert_eq!(gtk.region("navigation").unwrap().rect.width, 275);
        assert_ne!(
            windows.region("clipboard").unwrap().rect,
            macos.region("clipboard").unwrap().rect
        );
        assert_ne!(
            macos.region("clipboard").unwrap().rect,
            gtk.region("clipboard").unwrap().rect
        );

        let windows_paint =
            spec.paint_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Windows);
        let macos_paint =
            spec.paint_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Macos);
        let gtk_paint = spec.paint_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Gtk);
        let windows_section = rect_to_ui(windows.region("clipboard").unwrap().rect);
        let macos_section = rect_to_ui(macos.region("clipboard").unwrap().rect);
        let gtk_section = rect_to_ui(gtk.region("clipboard").unwrap().rect);

        assert!(windows_paint.paint_commands.iter().any(|command| matches!(
            command,
            ZsShellPaintCommand::RoundRect { rect, .. } if *rect == windows_section
        )));
        assert!(!macos_paint.paint_commands.iter().any(|command| matches!(
            command,
            ZsShellPaintCommand::RoundRect { rect, .. } if *rect == macos_section
        )));
        assert!(gtk_paint.paint_commands.iter().any(|command| matches!(
            command,
            ZsShellPaintCommand::RoundRect { rect, .. }
                if rect.top > gtk_section.top && rect.bottom == gtk_section.bottom
        )));

        let draw = spec.native_draw_plan_for_style(bounds, Dpi::standard(), ZsPlatformStyle::Macos);
        assert!(draw
            .commands
            .iter()
            .any(|command| matches!(command, NativeDrawCommand::Icon(_))));
        for private_glyph in [
            "\u{E700}", "\u{E713}", "\u{E76C}", "\u{E8D4}", "\u{E8A5}", "\u{E753}", "\u{E946}",
        ] {
            assert!(!draw.commands.iter().any(|command| matches!(
                command,
                NativeDrawCommand::Text(text) if text.text == private_glyph
            )));
        }
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
        let plan = spec.paint_plan_for_style(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
            ZsPlatformStyle::Windows,
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
    fn shell_theme_roles_keep_page_cards_and_strokes_visually_distinct() {
        assert_eq!(
            shell_role_to_color_role(ZsShellThemeRole::Background),
            ColorRole::Surface
        );
        assert_eq!(
            shell_role_to_color_role(ZsShellThemeRole::Surface),
            ColorRole::SurfaceRaised
        );
        assert_eq!(
            shell_role_to_color_role(ZsShellThemeRole::Stroke),
            ColorRole::Border
        );
        assert_eq!(
            shell_role_to_color_role(ZsShellThemeRole::White),
            ColorRole::AccentText
        );
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
    fn shell_native_icons_honor_their_declared_size_and_semantics() {
        let spec = ZsShellLayoutSpec::new("demo", "Demo")
            .nav_item(ZsShellNavItemSpec::new("rename", "Rename").semantic_icon(ZsIcon::Edit))
            .card(ZsShellGroupCardSpec::new("content", "Content"));
        let draw = spec.native_draw_plan_for_style(
            Rect {
                x: 0,
                y: 0,
                width: 1100,
                height: 740,
            },
            Dpi::standard(),
            ZsPlatformStyle::Windows,
        );
        let icons = draw
            .commands
            .iter()
            .filter_map(|command| match command {
                NativeDrawCommand::Icon(icon) => Some(icon),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(icons.iter().any(|icon| icon.icon == ZsIcon::Edit));
        assert!(icons
            .iter()
            .filter(|icon| matches!(icon.icon, ZsIcon::Sidebar | ZsIcon::Edit))
            .all(|icon| icon.bounds.width == 16 && icon.bounds.height == 16));
    }

    #[test]
    fn shell_actions_use_common_label_measurement_and_button_typography() {
        let action_area = ZsShellActionAreaSpec::new()
            .secondary(ZsShellActionButtonSpec::secondary("add", "添加发票"))
            .primary(ZsShellActionButtonSpec::primary("rename", "开始重命名"));
        let profile = shell_profile_for_style(ZsPlatformStyle::Windows);
        let rects = zs_shell_action_button_rects_with_profile(
            UiRect::new(0, 0, 1000, 700),
            &action_area,
            Dpi::standard(),
            profile,
        );
        let minimum = crate::ZsBaseControlMetrics::for_platform(ZsPlatformStyle::Windows)
            .button_minimum_width
            .to_px(Dpi::standard())
            .round_i32();

        assert_eq!(
            rects.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            ["add", "rename"]
        );
        assert!(rects
            .iter()
            .all(|(_, rect)| rect.right - rect.left >= minimum));

        let paint = zs_shell_button_paint_plan_with_profile(
            rects[1].1,
            "开始重命名".to_string(),
            ZsShellComponentKind::AccentButton,
            false,
            false,
            Dpi::standard(),
            profile,
        );
        let text = paint.text_commands.first().expect("button text command");
        assert_eq!(text.font, ZsShellTextFontRole::Button);
        assert!(text.rect.left > rects[1].1.left);
        assert!(text.rect.right < rects[1].1.right);
        assert_eq!(shell_text_style(text).role, TextRole::Button);
    }

    #[test]
    fn shell_emphasis_uses_platform_semibold_instead_of_heavy_bold() {
        let command = ZsShellTextCommand {
            rect: UiRect::new(0, 0, 200, 32),
            content: ZsShellTextContent::Label("Title".to_string()),
            color: ZsShellThemeRole::Text,
            size: 24,
            bold: true,
            font: ZsShellTextFontRole::Display,
            align: ZsShellTextAlign::Left,
        };

        assert_eq!(shell_text_style(&command).weight, TextWeight::Semibold);
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
        let sync = runtime
            .spec
            .layout_plan(bounds, Dpi::standard())
            .region("sync")
            .expect("sync navigation item")
            .rect;
        let sync_center = Point {
            x: sync.x + sync.width / 2,
            y: sync.y + sync.height / 2,
        };

        let hover = runtime.pointer_move(sync_center);
        assert!(hover.redraw);
        assert_eq!(runtime.spec.hovered_nav_id.as_deref(), Some("sync"));
        assert_eq!(hover.invalidate_rects.len(), 1);

        let selected = runtime.pointer_down(sync_center);
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
