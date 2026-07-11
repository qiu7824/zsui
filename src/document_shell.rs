use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Dp, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, Point, Rect, SemanticTextStyle,
    TextRole, TextWeight, TextWrap, VerticalAlign, ZsIcon,
};

const TAB_STRIP_HEIGHT_DP: f32 = 48.0;
const COMMAND_BAR_HEIGHT_DP: f32 = 48.0;
const STATUS_BAR_HEIGHT_DP: f32 = 32.0;
const SURFACE_MARGIN_DP: f32 = 12.0;
const EDITOR_INSET_DP: f32 = 8.0;
const COMMAND_HEIGHT_DP: f32 = 32.0;
const COMMAND_GAP_DP: f32 = 4.0;
const GROUP_GAP_DP: f32 = 12.0;
const ICON_SIZE_DP: f32 = 16.0;
const COMPACT_THRESHOLD_DP: f32 = 760.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsDocumentShellCommand {
    New,
    Close,
    Open,
    Save,
    SaveAs,
    Undo,
    Cut,
    Copy,
    Paste,
    ToggleWrap,
    ToggleStatus,
    About,
}

impl ZsDocumentShellCommand {
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::New => "document.new",
            Self::Close => "document.close",
            Self::Open => "document.open",
            Self::Save => "document.save",
            Self::SaveAs => "document.save-as",
            Self::Undo => "document.undo",
            Self::Cut => "document.cut",
            Self::Copy => "document.copy",
            Self::Paste => "document.paste",
            Self::ToggleWrap => "document.toggle-wrap",
            Self::ToggleStatus => "document.toggle-status",
            Self::About => "document.about",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Close => "Close",
            Self::Open => "Open",
            Self::Save => "Save",
            Self::SaveAs => "Save as",
            Self::Undo => "Undo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::ToggleWrap => "Wrap",
            Self::ToggleStatus => "Status",
            Self::About => "About",
        }
    }

    pub const fn icon(self) -> ZsIcon {
        match self {
            Self::New => ZsIcon::Add,
            Self::Close => ZsIcon::Close,
            Self::Open => ZsIcon::Folder,
            Self::Save | Self::SaveAs => ZsIcon::Save,
            Self::Undo => ZsIcon::Undo,
            Self::Cut => ZsIcon::Cut,
            Self::Copy => ZsIcon::Copy,
            Self::Paste => ZsIcon::Paste,
            Self::ToggleWrap => ZsIcon::Text,
            Self::ToggleStatus => ZsIcon::Inspector,
            Self::About => ZsIcon::More,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDocumentShellInteraction {
    pub hovered: Option<ZsDocumentShellCommand>,
    pub pressed: Option<ZsDocumentShellCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDocumentShellCommandRegion {
    pub command: ZsDocumentShellCommand,
    pub bounds: Rect,
    pub label: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDocumentShellLayout {
    pub surface: Rect,
    pub tab_strip: Rect,
    pub selected_tab: Rect,
    pub command_bar: Rect,
    pub editor_frame: Rect,
    pub editor_content: Rect,
    pub status_bar: Option<Rect>,
    pub command_regions: Vec<ZsDocumentShellCommandRegion>,
    pub separators: Vec<Rect>,
    pub compact: bool,
}

impl ZsDocumentShellLayout {
    pub fn command_at(&self, point: Point) -> Option<ZsDocumentShellCommand> {
        self.command_regions
            .iter()
            .find(|region| region.bounds.contains(point))
            .map(|region| region.command)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsDocumentShellSpec {
    pub app_title: String,
    pub document_title: String,
    pub dirty: bool,
    pub word_wrap: bool,
    pub show_status: bool,
    pub line: usize,
    pub column: usize,
    pub character_count: usize,
    pub encoding: String,
}

impl ZsDocumentShellSpec {
    pub fn new(app_title: impl Into<String>, document_title: impl Into<String>) -> Self {
        Self {
            app_title: app_title.into(),
            document_title: document_title.into(),
            dirty: false,
            word_wrap: true,
            show_status: true,
            line: 1,
            column: 1,
            character_count: 0,
            encoding: "UTF-8".to_string(),
        }
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn word_wrap(mut self, word_wrap: bool) -> Self {
        self.word_wrap = word_wrap;
        self
    }

    pub fn show_status(mut self, show_status: bool) -> Self {
        self.show_status = show_status;
        self
    }

    pub fn status(mut self, line: usize, column: usize, character_count: usize) -> Self {
        self.line = line.max(1);
        self.column = column.max(1);
        self.character_count = character_count;
        self
    }

    pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = encoding.into();
        self
    }

    pub fn layout(&self, surface: Rect, dpi: Dpi) -> ZsDocumentShellLayout {
        let tab_height = px(TAB_STRIP_HEIGHT_DP, dpi);
        let command_bar_height = px(COMMAND_BAR_HEIGHT_DP, dpi);
        let status_height = if self.show_status {
            px(STATUS_BAR_HEIGHT_DP, dpi)
        } else {
            0
        };
        let margin = px(SURFACE_MARGIN_DP, dpi);
        let editor_inset = px(EDITOR_INSET_DP, dpi);
        let compact = surface.width < px(COMPACT_THRESHOLD_DP, dpi);

        let tab_strip = Rect {
            x: surface.x,
            y: surface.y,
            width: surface.width.max(0),
            height: tab_height,
        };
        let tab_width = px(if compact { 210.0 } else { 280.0 }, dpi)
            .min((surface.width - margin * 3 - px(36.0, dpi)).max(px(140.0, dpi)));
        let selected_tab = Rect {
            x: surface.x + margin,
            y: surface.y + px(7.0, dpi),
            width: tab_width,
            height: px(38.0, dpi),
        };
        let command_bar = Rect {
            x: surface.x,
            y: surface.y + tab_height,
            width: surface.width.max(0),
            height: command_bar_height,
        };
        let editor_top = command_bar.y + command_bar.height + px(4.0, dpi);
        let editor_bottom = surface.y + surface.height - status_height - px(4.0, dpi);
        let editor_frame = Rect {
            x: surface.x + margin,
            y: editor_top,
            width: (surface.width - margin * 2).max(0),
            height: (editor_bottom - editor_top).max(0),
        };
        let editor_content = inset_rect(editor_frame, editor_inset);
        let status_bar = self.show_status.then_some(Rect {
            x: surface.x,
            y: surface.y + surface.height - status_height,
            width: surface.width.max(0),
            height: status_height,
        });

        let mut command_regions = Vec::new();
        let tab_button_size = px(30.0, dpi);
        command_regions.push(ZsDocumentShellCommandRegion {
            command: ZsDocumentShellCommand::Close,
            bounds: Rect {
                x: selected_tab.x + selected_tab.width - tab_button_size - px(4.0, dpi),
                y: selected_tab.y + px(4.0, dpi),
                width: tab_button_size,
                height: tab_button_size,
            },
            label: None,
            selected: false,
        });
        command_regions.push(ZsDocumentShellCommandRegion {
            command: ZsDocumentShellCommand::New,
            bounds: Rect {
                x: selected_tab.x + selected_tab.width + px(6.0, dpi),
                y: selected_tab.y + px(4.0, dpi),
                width: tab_button_size,
                height: tab_button_size,
            },
            label: None,
            selected: false,
        });

        let mut separators = Vec::new();
        let mut cursor = surface.x + margin;
        let command_y = command_bar.y + (command_bar.height - px(COMMAND_HEIGHT_DP, dpi)) / 2;
        let command_height = px(COMMAND_HEIGHT_DP, dpi);
        let gap = px(COMMAND_GAP_DP, dpi);
        let group_gap = px(GROUP_GAP_DP, dpi);

        for command in [
            ZsDocumentShellCommand::Open,
            ZsDocumentShellCommand::Save,
            ZsDocumentShellCommand::SaveAs,
        ] {
            cursor = push_command_region(
                &mut command_regions,
                command,
                cursor,
                command_y,
                command_height,
                compact,
                self,
                dpi,
            ) + gap;
        }
        separators.push(separator_at(
            cursor + group_gap / 2,
            command_y,
            command_height,
            dpi,
        ));
        cursor += group_gap;

        for command in [
            ZsDocumentShellCommand::Undo,
            ZsDocumentShellCommand::Cut,
            ZsDocumentShellCommand::Copy,
            ZsDocumentShellCommand::Paste,
        ] {
            cursor = push_command_region(
                &mut command_regions,
                command,
                cursor,
                command_y,
                command_height,
                true,
                self,
                dpi,
            ) + gap;
        }
        separators.push(separator_at(
            cursor + group_gap / 2,
            command_y,
            command_height,
            dpi,
        ));
        cursor += group_gap;

        for command in [
            ZsDocumentShellCommand::ToggleWrap,
            ZsDocumentShellCommand::ToggleStatus,
        ] {
            cursor = push_command_region(
                &mut command_regions,
                command,
                cursor,
                command_y,
                command_height,
                compact,
                self,
                dpi,
            ) + gap;
        }

        let about_width = px(COMMAND_HEIGHT_DP, dpi);
        command_regions.push(ZsDocumentShellCommandRegion {
            command: ZsDocumentShellCommand::About,
            bounds: Rect {
                x: (surface.x + surface.width - margin - about_width).max(cursor),
                y: command_y,
                width: about_width,
                height: command_height,
            },
            label: None,
            selected: false,
        });

        ZsDocumentShellLayout {
            surface,
            tab_strip,
            selected_tab,
            command_bar,
            editor_frame,
            editor_content,
            status_bar,
            command_regions,
            separators,
            compact,
        }
    }

    pub fn native_draw_plan(
        &self,
        surface: Rect,
        dpi: Dpi,
        interaction: ZsDocumentShellInteraction,
    ) -> NativeDrawPlan {
        let layout = self.layout(surface, dpi);
        let mut commands = vec![fill(surface, ColorRole::Surface)];

        commands.push(round_rect(
            layout.selected_tab,
            ColorRole::SurfaceRaised,
            Some(ColorRole::Border),
            px(8.0, dpi),
        ));
        commands.push(NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                ZsIcon::File,
                Rect {
                    x: layout.selected_tab.x + px(12.0, dpi),
                    y: layout.selected_tab.y + (layout.selected_tab.height - px(16.0, dpi)) / 2,
                    width: px(16.0, dpi),
                    height: px(16.0, dpi),
                },
                NativeIconColorMode::ThemeAware,
            )
            .with_color(ColorRole::Accent),
        ));
        let close_region = layout
            .command_regions
            .iter()
            .find(|region| region.command == ZsDocumentShellCommand::Close)
            .expect("close region is part of the tab layout");
        commands.push(text(
            self.document_title.clone(),
            Rect {
                x: layout.selected_tab.x + px(36.0, dpi),
                y: layout.selected_tab.y,
                width: (close_region.bounds.x
                    - layout.selected_tab.x
                    - px(if self.dirty { 52.0 } else { 44.0 }, dpi))
                .max(0),
                height: layout.selected_tab.height,
            },
            body_style(ColorRole::PrimaryText, TextWeight::Regular),
        ));
        if self.dirty {
            let indicator = px(6.0, dpi);
            commands.push(NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: close_region.bounds.x - px(10.0, dpi),
                    y: layout.selected_tab.y + (layout.selected_tab.height - indicator) / 2,
                    width: indicator,
                    height: indicator,
                },
                fill: NativeDrawFill::Role(ColorRole::Accent),
                radius: indicator / 2,
            });
        }

        for region in &layout.command_regions {
            paint_command_region(region, interaction, dpi, &mut commands);
        }
        for separator in &layout.separators {
            commands.push(fill_rect(*separator, ColorRole::Border));
        }

        commands.push(round_rect(
            layout.editor_frame,
            ColorRole::SurfaceRaised,
            Some(ColorRole::Border),
            px(8.0, dpi),
        ));

        if let Some(status) = layout.status_bar {
            commands.push(text(
                format!(
                    "Ln {}, Col {}    |    {} characters",
                    self.line, self.column, self.character_count
                ),
                Rect {
                    x: status.x + px(16.0, dpi),
                    y: status.y,
                    width: (status.width * 2 / 3).max(0),
                    height: status.height,
                },
                caption_style(ColorRole::SecondaryText, HorizontalAlign::Start),
            ));
            commands.push(text(
                format!(
                    "{}    |    {}",
                    self.encoding,
                    if self.word_wrap { "Wrap" } else { "No wrap" }
                ),
                Rect {
                    x: status.x + status.width / 2,
                    y: status.y,
                    width: (status.width / 2 - px(16.0, dpi)).max(0),
                    height: status.height,
                },
                caption_style(ColorRole::SecondaryText, HorizontalAlign::End),
            ));
        }

        NativeDrawPlan::new(commands)
    }
}

fn push_command_region(
    regions: &mut Vec<ZsDocumentShellCommandRegion>,
    command: ZsDocumentShellCommand,
    x: i32,
    y: i32,
    height: i32,
    compact: bool,
    spec: &ZsDocumentShellSpec,
    dpi: Dpi,
) -> i32 {
    let width = if compact {
        px(COMMAND_HEIGHT_DP, dpi)
    } else {
        px(
            match command {
                ZsDocumentShellCommand::Open => 76.0,
                ZsDocumentShellCommand::Save => 70.0,
                ZsDocumentShellCommand::SaveAs => 92.0,
                ZsDocumentShellCommand::ToggleStatus => 76.0,
                ZsDocumentShellCommand::ToggleWrap => 70.0,
                _ => 68.0,
            },
            dpi,
        )
    };
    regions.push(ZsDocumentShellCommandRegion {
        command,
        bounds: Rect {
            x,
            y,
            width,
            height,
        },
        label: (!compact).then(|| command.label().to_string()),
        selected: match command {
            ZsDocumentShellCommand::ToggleWrap => spec.word_wrap,
            ZsDocumentShellCommand::ToggleStatus => spec.show_status,
            _ => false,
        },
    });
    x + width
}

fn paint_command_region(
    region: &ZsDocumentShellCommandRegion,
    interaction: ZsDocumentShellInteraction,
    dpi: Dpi,
    commands: &mut Vec<NativeDrawCommand>,
) {
    let fill = if interaction.pressed == Some(region.command) {
        Some(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Accent,
            alpha: 42,
        })
    } else if interaction.hovered == Some(region.command) {
        Some(NativeDrawFill::Role(ColorRole::Control))
    } else if region.selected {
        Some(NativeDrawFill::RoleWithAlpha {
            role: ColorRole::Accent,
            alpha: 26,
        })
    } else {
        None
    };
    if let Some(fill) = fill {
        commands.push(NativeDrawCommand::RoundFill {
            rect: region.bounds,
            fill,
            radius: px(4.0, dpi),
        });
    }

    let icon_size = px(ICON_SIZE_DP, dpi);
    let has_label = region.label.is_some();
    let icon_x = if has_label {
        region.bounds.x + px(10.0, dpi)
    } else {
        region.bounds.x + (region.bounds.width - icon_size) / 2
    };
    commands.push(NativeDrawCommand::Icon(
        NativeDrawIconCommand::new(
            region.command.icon(),
            Rect {
                x: icon_x,
                y: region.bounds.y + (region.bounds.height - icon_size) / 2,
                width: icon_size,
                height: icon_size,
            },
            NativeIconColorMode::ThemeAware,
        )
        .with_color(if region.selected {
            ColorRole::Accent
        } else {
            ColorRole::PrimaryText
        }),
    ));
    if let Some(label) = &region.label {
        commands.push(text(
            label,
            Rect {
                x: icon_x + icon_size + px(7.0, dpi),
                y: region.bounds.y,
                width: (region.bounds.x + region.bounds.width - icon_x - icon_size - px(11.0, dpi))
                    .max(0),
                height: region.bounds.height,
            },
            button_style(if region.selected {
                ColorRole::Accent
            } else {
                ColorRole::PrimaryText
            }),
        ));
    }
}

fn separator_at(x: i32, y: i32, height: i32, dpi: Dpi) -> Rect {
    Rect {
        x,
        y: y + px(7.0, dpi),
        width: 1,
        height: (height - px(14.0, dpi)).max(0),
    }
}

fn inset_rect(rect: Rect, inset: i32) -> Rect {
    Rect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2).max(0),
        height: (rect.height - inset * 2).max(0),
    }
}

fn px(value: f32, dpi: Dpi) -> i32 {
    Dp::new(value).to_px(dpi).round_i32().max(1)
}

fn fill(rect: Rect, role: ColorRole) -> NativeDrawCommand {
    NativeDrawCommand::FillRect {
        rect,
        fill: NativeDrawFill::Role(role),
    }
}

fn fill_rect(rect: Rect, role: ColorRole) -> NativeDrawCommand {
    fill(rect, role)
}

fn round_rect(
    rect: Rect,
    fill: ColorRole,
    stroke: Option<ColorRole>,
    radius: i32,
) -> NativeDrawCommand {
    NativeDrawCommand::RoundRect {
        rect,
        fill: NativeDrawFill::Role(fill),
        stroke: stroke.map(NativeDrawFill::Role),
        radius,
    }
}

fn text(value: impl Into<String>, bounds: Rect, style: SemanticTextStyle) -> NativeDrawCommand {
    NativeDrawCommand::Text(NativeDrawTextCommand::new(value, bounds, style))
}

fn body_style(color: ColorRole, weight: TextWeight) -> SemanticTextStyle {
    SemanticTextStyle {
        role: TextRole::Body,
        color,
        weight,
        horizontal_align: HorizontalAlign::Start,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: true,
    }
}

fn button_style(color: ColorRole) -> SemanticTextStyle {
    SemanticTextStyle {
        role: TextRole::Button,
        color,
        weight: TextWeight::Regular,
        horizontal_align: HorizontalAlign::Start,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: true,
    }
}

fn caption_style(color: ColorRole, align: HorizontalAlign) -> SemanticTextStyle {
    SemanticTextStyle {
        role: TextRole::Caption,
        color,
        weight: TextWeight::Regular,
        horizontal_align: align,
        vertical_align: VerticalAlign::Center,
        wrap: TextWrap::NoWrap,
        ellipsis: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 960,
            height: 640,
        }
    }

    #[test]
    fn layout_reserves_native_editor_and_status_regions() {
        let spec = ZsDocumentShellSpec::new("Editor", "notes.txt").status(4, 8, 120);
        let layout = spec.layout(surface(), Dpi::standard());

        assert!(layout.editor_content.width > 0);
        assert!(layout.editor_content.height > 0);
        assert!(layout.status_bar.is_some());
        assert!(layout.editor_frame.y > layout.command_bar.y);
        assert!(layout.command_regions.len() >= 10);
    }

    #[test]
    fn command_hit_testing_uses_stable_regions() {
        let layout =
            ZsDocumentShellSpec::new("Editor", "notes.txt").layout(surface(), Dpi::standard());
        let open = layout
            .command_regions
            .iter()
            .find(|region| region.command == ZsDocumentShellCommand::Open)
            .unwrap();

        assert_eq!(
            layout.command_at(Point {
                x: open.bounds.x + 2,
                y: open.bounds.y + 2,
            }),
            Some(ZsDocumentShellCommand::Open)
        );
    }

    #[test]
    fn draw_plan_uses_semantic_icons_and_selected_state() {
        let spec = ZsDocumentShellSpec::new("Editor", "notes.txt")
            .dirty(true)
            .word_wrap(true);
        let plan = spec.native_draw_plan(
            surface(),
            Dpi::standard(),
            ZsDocumentShellInteraction {
                hovered: Some(ZsDocumentShellCommand::Save),
                pressed: None,
            },
        );

        assert!(plan.icon_count() >= 10);
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::RoundFill {
                fill: NativeDrawFill::RoleWithAlpha {
                    role: ColorRole::Accent,
                    ..
                },
                ..
            }
        )));
        assert!(!plan.commands.iter().any(|command| matches!(
            command,
            NativeDrawCommand::Text(command) if command.style.role == TextRole::Icon
        )));
    }

    #[test]
    fn narrow_layout_switches_to_icon_only_commands() {
        let layout = ZsDocumentShellSpec::new("Editor", "notes.txt").layout(
            Rect {
                width: 620,
                ..surface()
            },
            Dpi::standard(),
        );

        assert!(layout.compact);
        assert!(layout
            .command_regions
            .iter()
            .filter(|region| region.bounds.y >= layout.command_bar.y)
            .all(|region| region.label.is_none()));
    }
}
