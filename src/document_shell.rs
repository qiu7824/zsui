use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    ColorRole, Command, Dp, Dpi, HorizontalAlign, NativeDrawCommand, NativeDrawFill,
    NativeDrawIconCommand, NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, Point, Rect,
    SemanticTextStyle, TextRole, TextWeight, TextWrap, VerticalAlign, ZsBaseControlMetrics, ZsIcon,
    ZsuiError, ZsuiResult,
};

use crate::platform_component_profile::{PlatformComponentProfile, PlatformDocumentShellProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsTextDocumentEncoding {
    Utf8,
    Utf16LittleEndian,
    Utf16BigEndian,
}

impl ZsTextDocumentEncoding {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16LittleEndian => "UTF-16 LE",
            Self::Utf16BigEndian => "UTF-16 BE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTextCursorStatus {
    pub line: usize,
    pub column: usize,
    pub character_count: usize,
}

impl Default for ZsTextCursorStatus {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            character_count: 0,
        }
    }
}

impl ZsTextCursorStatus {
    pub fn from_character_caret(text: &str, caret: usize) -> Self {
        let mut line = 1;
        let mut column = 1;

        for character in text.chars().take(caret) {
            if character == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Self {
            line,
            column,
            character_count: text.chars().count(),
        }
    }

    pub fn from_utf16_caret(text: &str, caret_utf16: usize) -> Self {
        let mut line = 1;
        let mut column = 1;
        let mut consumed_utf16 = 0;

        for character in text.chars() {
            let character_utf16 = character.len_utf16();
            if consumed_utf16 + character_utf16 > caret_utf16 {
                break;
            }
            consumed_utf16 += character_utf16;
            if character == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Self {
            line,
            column,
            character_count: text.chars().count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsTextDocument {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    encoding: ZsTextDocumentEncoding,
}

impl Default for ZsTextDocument {
    fn default() -> Self {
        Self::untitled("")
    }
}

impl ZsTextDocument {
    pub fn untitled(initial_text: impl Into<String>) -> Self {
        Self {
            path: None,
            text: initial_text.into(),
            dirty: false,
            encoding: ZsTextDocumentEncoding::Utf8,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> ZsuiResult<Self> {
        let (text, encoding) = decode_text_document(bytes)
            .map_err(|message| ZsuiError::invalid_spec("text_document.bytes", message))?;
        Ok(Self {
            path: None,
            text,
            dirty: false,
            encoding,
        })
    }

    pub fn open(path: impl Into<PathBuf>) -> ZsuiResult<Self> {
        let path = validate_text_document_path(path.into(), "text_document.open.path")?;
        let bytes = fs::read(&path)
            .map_err(|error| ZsuiError::host("text_document.open", error.to_string()))?;
        let (text, encoding) = decode_text_document(&bytes)
            .map_err(|message| ZsuiError::host("text_document.open", message))?;
        Ok(Self {
            path: Some(path),
            text,
            dirty: false,
            encoding,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub const fn encoding(&self) -> ZsTextDocumentEncoding {
        self.encoding
    }

    pub fn replace_text(&mut self, text: impl Into<String>) -> bool {
        let text = text.into();
        if self.text == text {
            return false;
        }
        self.text = text;
        self.dirty = true;
        true
    }

    pub fn save(&mut self) -> ZsuiResult<()> {
        let path = self.path.as_deref().ok_or_else(|| {
            ZsuiError::invalid_spec(
                "text_document.path",
                "save requires an existing path or save_as",
            )
        })?;
        write_utf8_text_document(path, &self.text)?;
        self.dirty = false;
        self.encoding = ZsTextDocumentEncoding::Utf8;
        Ok(())
    }

    pub fn save_as(&mut self, path: impl Into<PathBuf>) -> ZsuiResult<()> {
        let path = validate_text_document_path(path.into(), "text_document.save_as.path")?;
        write_utf8_text_document(&path, &self.text)?;
        self.path = Some(path);
        self.dirty = false;
        self.encoding = ZsTextDocumentEncoding::Utf8;
        Ok(())
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

fn validate_text_document_path(path: PathBuf, field: &'static str) -> ZsuiResult<PathBuf> {
    if path.as_os_str().is_empty() {
        Err(ZsuiError::invalid_spec(field, "path must not be empty"))
    } else {
        Ok(path)
    }
}

fn write_utf8_text_document(path: &Path, text: &str) -> ZsuiResult<()> {
    fs::write(path, text.as_bytes())
        .map_err(|error| ZsuiError::host("text_document.save", error.to_string()))
}

fn decode_text_document(bytes: &[u8]) -> Result<(String, ZsTextDocumentEncoding), String> {
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(rest.to_vec())
            .map(|text| (text, ZsTextDocumentEncoding::Utf8))
            .map_err(|error| error.to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        return decode_utf16_text_document(rest, u16::from_le_bytes)
            .map(|text| (text, ZsTextDocumentEncoding::Utf16LittleEndian));
    }
    if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        return decode_utf16_text_document(rest, u16::from_be_bytes)
            .map(|text| (text, ZsTextDocumentEncoding::Utf16BigEndian));
    }
    String::from_utf8(bytes.to_vec())
        .map(|text| (text, ZsTextDocumentEncoding::Utf8))
        .map_err(|error| format!("the file is not valid UTF-8 or BOM-tagged UTF-16: {error}"))
}

fn decode_utf16_text_document(bytes: &[u8], decode: fn([u8; 2]) -> u16) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("UTF-16 file has an odd byte length".to_string());
    }
    let units = bytes
        .chunks_exact(2)
        .map(|pair| decode([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&units).map_err(|error| error.to_string())
}

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
    SelectAll,
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
            Self::SelectAll => "document.select-all",
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
            Self::SelectAll => "Select all",
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
            Self::SelectAll => ZsIcon::Text,
            Self::ToggleWrap => ZsIcon::Text,
            Self::ToggleStatus => ZsIcon::Inspector,
            Self::About => ZsIcon::More,
        }
    }

    pub fn to_command(self) -> Command {
        Command::custom(self.command_id())
    }

    pub fn from_command(command: &Command) -> Option<Self> {
        let Command::Custom { id, payload: None } = command else {
            return None;
        };
        Some(match id.as_str() {
            "document.new" => Self::New,
            "document.close" => Self::Close,
            "document.open" => Self::Open,
            "document.save" => Self::Save,
            "document.save-as" => Self::SaveAs,
            "document.undo" => Self::Undo,
            "document.cut" => Self::Cut,
            "document.copy" => Self::Copy,
            "document.paste" => Self::Paste,
            "document.select-all" => Self::SelectAll,
            "document.toggle-wrap" => Self::ToggleWrap,
            "document.toggle-status" => Self::ToggleStatus,
            "document.about" => Self::About,
            _ => return None,
        })
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
        let component_profile = PlatformComponentProfile::current();
        self.layout_with_profiles(
            surface,
            dpi,
            component_profile.document_shell,
            component_profile.base_control.metrics,
        )
    }

    fn layout_with_profiles(
        &self,
        surface: Rect,
        dpi: Dpi,
        profile: PlatformDocumentShellProfile,
        base_control: ZsBaseControlMetrics,
    ) -> ZsDocumentShellLayout {
        let tab_height = px_dp(profile.tab_strip_height, dpi);
        let command_bar_height = px_dp(profile.command_bar_height, dpi);
        let status_height = if self.show_status {
            px_dp(profile.status_bar_height, dpi)
        } else {
            0
        };
        let margin = px_dp(profile.surface_margin, dpi);
        let editor_inset = px_dp(profile.editor_inset, dpi);
        let compact = surface.width < px_dp(profile.compact_threshold, dpi);

        let tab_strip = Rect {
            x: surface.x,
            y: surface.y,
            width: surface.width.max(0),
            height: tab_height,
        };
        let tab_width = px_dp(
            if compact {
                profile.compact_tab_width
            } else {
                profile.regular_tab_width
            },
            dpi,
        )
        .min(
            (surface.width - margin * 3 - px_dp(profile.reserved_tab_action_width, dpi))
                .max(px_dp(profile.minimum_tab_width, dpi)),
        );
        let selected_tab = Rect {
            x: surface.x + margin,
            y: surface.y + px_dp(profile.tab_top_inset, dpi),
            width: tab_width,
            height: px_dp(profile.tab_height, dpi),
        };
        let command_bar = Rect {
            x: surface.x,
            y: surface.y + tab_height,
            width: surface.width.max(0),
            height: command_bar_height,
        };
        let editor_gap = px_dp(profile.editor_vertical_gap, dpi);
        let editor_top = command_bar.y + command_bar.height + editor_gap;
        let editor_bottom = surface.y + surface.height - status_height - editor_gap;
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
        let tab_button_size = px_dp(profile.tab_action_size, dpi);
        let tab_button_inset = px_dp(profile.tab_action_inset, dpi);
        command_regions.push(ZsDocumentShellCommandRegion {
            command: ZsDocumentShellCommand::Close,
            bounds: Rect {
                x: selected_tab.x + selected_tab.width - tab_button_size - tab_button_inset,
                y: selected_tab.y + tab_button_inset,
                width: tab_button_size,
                height: tab_button_size,
            },
            label: None,
            selected: false,
        });
        command_regions.push(ZsDocumentShellCommandRegion {
            command: ZsDocumentShellCommand::New,
            bounds: Rect {
                x: selected_tab.x + selected_tab.width + px_dp(profile.tab_action_gap, dpi),
                y: selected_tab.y + tab_button_inset,
                width: tab_button_size,
                height: tab_button_size,
            },
            label: None,
            selected: false,
        });

        let mut separators = Vec::new();
        let mut cursor = surface.x + margin;
        let command_height = px_dp(profile.command_height, dpi);
        let command_y = command_bar.y + (command_bar.height - command_height) / 2;
        let gap = px_dp(profile.command_gap, dpi);
        let group_gap = px_dp(profile.command_group_gap, dpi);

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
                profile,
                base_control,
            ) + gap;
        }
        separators.push(separator_at(
            cursor + group_gap / 2,
            command_y,
            command_height,
            px_dp(profile.separator_vertical_inset, dpi),
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
                profile,
                base_control,
            ) + gap;
        }
        separators.push(separator_at(
            cursor + group_gap / 2,
            command_y,
            command_height,
            px_dp(profile.separator_vertical_inset, dpi),
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
                profile,
                base_control,
            ) + gap;
        }

        let about_width = command_height;
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
        let component_profile = PlatformComponentProfile::current();
        let profile = component_profile.document_shell;
        let layout = self.layout_with_profiles(
            surface,
            dpi,
            profile,
            component_profile.base_control.metrics,
        );
        let mut commands = vec![fill(surface, ColorRole::Surface)];

        commands.push(round_rect(
            layout.selected_tab,
            ColorRole::SurfaceRaised,
            Some(ColorRole::Border),
            px_dp(profile.tab_radius, dpi),
        ));
        commands.push(NativeDrawCommand::Icon(
            NativeDrawIconCommand::new(
                ZsIcon::File,
                Rect {
                    x: layout.selected_tab.x + px_dp(profile.tab_icon_leading, dpi),
                    y: layout.selected_tab.y
                        + (layout.selected_tab.height - px_dp(profile.tab_icon_size, dpi)) / 2,
                    width: px_dp(profile.tab_icon_size, dpi),
                    height: px_dp(profile.tab_icon_size, dpi),
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
                x: layout.selected_tab.x + px_dp(profile.tab_label_leading, dpi),
                y: layout.selected_tab.y,
                width: (close_region.bounds.x
                    - layout.selected_tab.x
                    - px_dp(
                        if self.dirty {
                            profile.dirty_title_reserve
                        } else {
                            profile.clean_title_reserve
                        },
                        dpi,
                    ))
                .max(0),
                height: layout.selected_tab.height,
            },
            body_style(ColorRole::PrimaryText, TextWeight::Regular),
        ));
        if self.dirty {
            let indicator = px_dp(profile.dirty_indicator_size, dpi);
            commands.push(NativeDrawCommand::RoundFill {
                rect: Rect {
                    x: close_region.bounds.x - px_dp(profile.dirty_indicator_gap, dpi),
                    y: layout.selected_tab.y + (layout.selected_tab.height - indicator) / 2,
                    width: indicator,
                    height: indicator,
                },
                fill: NativeDrawFill::Role(ColorRole::Accent),
                radius: indicator / 2,
            });
        }

        for region in &layout.command_regions {
            paint_command_region(region, interaction, dpi, profile, &mut commands);
        }
        for separator in &layout.separators {
            commands.push(fill_rect(*separator, ColorRole::Border));
        }

        commands.push(round_rect(
            layout.editor_frame,
            ColorRole::SurfaceRaised,
            Some(ColorRole::Border),
            px_dp(profile.editor_radius, dpi),
        ));

        if let Some(status) = layout.status_bar {
            commands.push(text(
                format!(
                    "Ln {}, Col {}    |    {} characters",
                    self.line, self.column, self.character_count
                ),
                Rect {
                    x: status.x + px_dp(profile.status_horizontal_inset, dpi),
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
                    width: (status.width / 2 - px_dp(profile.status_horizontal_inset, dpi)).max(0),
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
    profile: PlatformDocumentShellProfile,
    base_control: ZsBaseControlMetrics,
) -> i32 {
    let width = if compact {
        px_dp(profile.command_height, dpi)
    } else {
        let content_width = profile.command_icon_leading.0
            + profile.command_icon_size.0
            + profile.command_label_gap.0
            + base_control
                .estimated_text_width_with_shaping_reserve(command.label())
                .0
            + profile.command_label_trailing.0;
        px_dp(Dp::new(content_width.max(profile.command_height.0)), dpi)
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
    profile: PlatformDocumentShellProfile,
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
            radius: px_dp(profile.command_radius, dpi),
        });
    }

    let icon_size = px_dp(profile.command_icon_size, dpi);
    let has_label = region.label.is_some();
    let icon_x = if has_label {
        region.bounds.x + px_dp(profile.command_icon_leading, dpi)
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
                x: icon_x + icon_size + px_dp(profile.command_label_gap, dpi),
                y: region.bounds.y,
                width: (region.bounds.x + region.bounds.width
                    - icon_x
                    - icon_size
                    - px_dp(profile.command_label_trailing, dpi))
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

fn separator_at(x: i32, y: i32, height: i32, vertical_inset: i32) -> Rect {
    Rect {
        x,
        y: y + vertical_inset,
        width: 1,
        height: (height - vertical_inset * 2).max(0),
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

fn px_dp(value: Dp, dpi: Dpi) -> i32 {
    value.to_px(dpi).round_i32()
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
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(1);

    struct TempFile(PathBuf);

    impl TempFile {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                "zsui-text-document-{}-{}.txt",
                std::process::id(),
                NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed)
            )))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn surface() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 960,
            height: 640,
        }
    }

    #[test]
    fn text_document_decodes_supported_unicode_encodings() {
        let utf8 = ZsTextDocument::from_bytes(b"hello").unwrap();
        assert_eq!(utf8.text(), "hello");
        assert_eq!(utf8.encoding(), ZsTextDocumentEncoding::Utf8);

        let little_endian = ZsTextDocument::from_bytes(&[0xff, 0xfe, b'h', 0, b'i', 0]).unwrap();
        assert_eq!(little_endian.text(), "hi");
        assert_eq!(
            little_endian.encoding(),
            ZsTextDocumentEncoding::Utf16LittleEndian
        );

        let big_endian = ZsTextDocument::from_bytes(&[0xfe, 0xff, 0, b'h', 0, b'i']).unwrap();
        assert_eq!(big_endian.text(), "hi");
        assert_eq!(
            big_endian.encoding(),
            ZsTextDocumentEncoding::Utf16BigEndian
        );
    }

    #[test]
    fn document_commands_round_trip_through_the_shared_command_type() {
        let commands = [
            ZsDocumentShellCommand::New,
            ZsDocumentShellCommand::Close,
            ZsDocumentShellCommand::Open,
            ZsDocumentShellCommand::Save,
            ZsDocumentShellCommand::SaveAs,
            ZsDocumentShellCommand::Undo,
            ZsDocumentShellCommand::Cut,
            ZsDocumentShellCommand::Copy,
            ZsDocumentShellCommand::Paste,
            ZsDocumentShellCommand::SelectAll,
            ZsDocumentShellCommand::ToggleWrap,
            ZsDocumentShellCommand::ToggleStatus,
            ZsDocumentShellCommand::About,
        ];

        for command in commands {
            assert_eq!(
                ZsDocumentShellCommand::from_command(&command.to_command()),
                Some(command)
            );
        }
        assert_eq!(
            ZsDocumentShellCommand::from_command(&Command::custom("application.unrelated")),
            None
        );
        assert_eq!(
            ZsDocumentShellCommand::from_command(&Command::custom_with_payload(
                ZsDocumentShellCommand::Open.command_id(),
                "unexpected"
            )),
            None
        );
    }

    #[test]
    fn text_document_tracks_dirty_state_only_when_text_changes() {
        let mut document = ZsTextDocument::untitled("draft");

        assert!(!document.replace_text("draft"));
        assert!(!document.is_dirty());
        assert!(document.replace_text("changed"));
        assert!(document.is_dirty());
        assert!(matches!(
            document.save(),
            Err(ZsuiError::InvalidSpec { field, .. }) if field == "text_document.path"
        ));
    }

    #[test]
    fn text_document_save_as_is_transactional_and_writes_utf8() {
        let target = TempFile::new();
        let mut document = ZsTextDocument::from_bytes(&[0xff, 0xfe, b'h', 0, b'i', 0]).unwrap();
        document.replace_text("保存");

        assert!(document.save_as(target.path().join("nested")).is_err());
        assert_eq!(document.path(), None);
        assert!(document.is_dirty());

        document.save_as(target.path()).unwrap();

        assert_eq!(document.path(), Some(target.path()));
        assert_eq!(
            document.display_name(),
            target.path().file_name().unwrap().to_string_lossy()
        );
        assert!(!document.is_dirty());
        assert_eq!(document.encoding(), ZsTextDocumentEncoding::Utf8);
        assert_eq!(fs::read(target.path()).unwrap(), "保存".as_bytes());
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
    fn cursor_status_uses_utf16_offsets_without_splitting_unicode_scalars() {
        let text = "a😀\n中";

        assert_eq!(
            ZsTextCursorStatus::from_utf16_caret(text, 3),
            ZsTextCursorStatus {
                line: 1,
                column: 3,
                character_count: 4,
            }
        );
        assert_eq!(
            ZsTextCursorStatus::from_utf16_caret(text, 4),
            ZsTextCursorStatus {
                line: 2,
                column: 1,
                character_count: 4,
            }
        );
        assert_eq!(
            ZsTextCursorStatus::from_utf16_caret(text, usize::MAX),
            ZsTextCursorStatus {
                line: 2,
                column: 2,
                character_count: 4,
            }
        );
        assert_eq!(
            ZsTextCursorStatus::from_character_caret(text, 3),
            ZsTextCursorStatus {
                line: 2,
                column: 1,
                character_count: 4,
            }
        );
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

    #[test]
    fn legacy_shell_layout_resolves_platform_profiles_internally() {
        let spec = ZsDocumentShellSpec::new("Editor", "notes.txt");
        let windows_profile = PlatformComponentProfile::for_style(crate::ZsPlatformStyle::Windows);
        let macos_profile = PlatformComponentProfile::for_style(crate::ZsPlatformStyle::Macos);
        let gtk_profile = PlatformComponentProfile::for_style(crate::ZsPlatformStyle::Gtk);

        let windows = spec.layout_with_profiles(
            surface(),
            Dpi::standard(),
            windows_profile.document_shell,
            windows_profile.base_control.metrics,
        );
        let macos = spec.layout_with_profiles(
            surface(),
            Dpi::standard(),
            macos_profile.document_shell,
            macos_profile.base_control.metrics,
        );
        let gtk = spec.layout_with_profiles(
            surface(),
            Dpi::standard(),
            gtk_profile.document_shell,
            gtk_profile.base_control.metrics,
        );

        assert_eq!(windows.tab_strip.height, 48);
        assert_eq!(macos.tab_strip.height, 32);
        assert_eq!(gtk.tab_strip.height, 42);
        assert_eq!(windows.command_bar.height, 48);
        assert_eq!(macos.command_bar.height, 28);
        assert_eq!(gtk.command_bar.height, 34);
        assert_ne!(windows.selected_tab.width, macos.selected_tab.width);
        assert_ne!(macos.editor_frame.x, gtk.editor_frame.x);

        for layout in [windows, macos, gtk] {
            for region in layout
                .command_regions
                .iter()
                .filter(|region| region.label.is_some())
            {
                assert!(region.bounds.width >= region.bounds.height);
            }
        }
    }
}
