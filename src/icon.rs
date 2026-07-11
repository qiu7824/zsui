use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ZsIcon {
    App,
    Calculator,
    History,
    Backspace,
    Add,
    Search,
    Settings,
    Sidebar,
    Inspector,
    More,
    Attach,
    Send,
    Stop,
    Refresh,
    Retry,
    Code,
    Tool,
    Check,
    Minimize,
    Close,
    Text,
    Image,
    File,
    Folder,
    Save,
    Undo,
    Cut,
    Pin,
    Delete,
    Copy,
    Paste,
    Edit,
    Group,
    Phrase,
}

impl ZsIcon {
    pub const fn asset_name(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Calculator => "calculator",
            Self::History => "history",
            Self::Backspace => "backspace",
            Self::Add => "add",
            Self::Search => "search",
            Self::Settings => "setting",
            Self::Sidebar => "sidebar",
            Self::Inspector => "inspector",
            Self::More => "more",
            Self::Attach => "attach",
            Self::Send => "send",
            Self::Stop => "stop",
            Self::Refresh => "refresh",
            Self::Retry => "retry",
            Self::Code => "code",
            Self::Tool => "tool",
            Self::Check => "check",
            Self::Minimize => "min",
            Self::Close => "exit",
            Self::Text | Self::Phrase => "text",
            Self::Image => "image",
            Self::File => "file",
            Self::Folder | Self::Group => "fold",
            Self::Save => "save",
            Self::Undo => "undo",
            Self::Cut => "cut",
            Self::Pin => "top",
            Self::Delete => "del",
            Self::Copy => "copy",
            Self::Paste => "paste",
            Self::Edit => "edit",
        }
    }

    pub const fn gtk_symbolic_name(self) -> &'static str {
        match self {
            Self::App => "edit-paste-symbolic",
            Self::Calculator => "accessories-calculator-symbolic",
            Self::History => "document-open-recent-symbolic",
            Self::Backspace => "edit-clear-symbolic",
            Self::Add => "list-add-symbolic",
            Self::Search => "edit-find-symbolic",
            Self::Settings => "emblem-system-symbolic",
            Self::Sidebar => "sidebar-show-symbolic",
            Self::Inspector => "document-properties-symbolic",
            Self::More => "view-more-symbolic",
            Self::Attach => "mail-attachment-symbolic",
            Self::Send => "mail-send-symbolic",
            Self::Stop => "media-playback-stop-symbolic",
            Self::Refresh => "view-refresh-symbolic",
            Self::Retry => "view-refresh-symbolic",
            Self::Code => "utilities-terminal-symbolic",
            Self::Tool => "applications-engineering-symbolic",
            Self::Check => "emblem-ok-symbolic",
            Self::Minimize => "window-minimize-symbolic",
            Self::Close => "window-close-symbolic",
            Self::Text | Self::Phrase => "text-x-generic-symbolic",
            Self::Image => "image-x-generic-symbolic",
            Self::File => "text-x-generic-symbolic",
            Self::Folder | Self::Group => "folder-symbolic",
            Self::Save => "document-save-symbolic",
            Self::Undo => "edit-undo-symbolic",
            Self::Cut => "edit-cut-symbolic",
            Self::Pin => "view-pin-symbolic",
            Self::Delete => "user-trash-symbolic",
            Self::Copy => "edit-copy-symbolic",
            Self::Paste => "edit-paste-symbolic",
            Self::Edit => "document-edit-symbolic",
        }
    }

    pub const fn windows_fluent_glyph(self) -> &'static str {
        match self {
            Self::App => "\u{E71D}",
            Self::Calculator => "\u{E8EF}",
            Self::History => "\u{E81C}",
            Self::Backspace => "\u{E750}",
            Self::Add => "\u{E710}",
            Self::Search => "\u{E721}",
            Self::Settings => "\u{E713}",
            Self::Sidebar => "\u{E700}",
            Self::Inspector => "\u{E8A0}",
            Self::More => "\u{E712}",
            Self::Attach => "\u{E723}",
            Self::Send => "\u{E724}",
            Self::Stop => "\u{E71A}",
            Self::Refresh => "\u{E72C}",
            Self::Retry => "\u{E72C}",
            Self::Code => "\u{E943}",
            Self::Tool => "\u{E90F}",
            Self::Check => "\u{E73E}",
            Self::Minimize => "\u{E921}",
            Self::Close => "\u{E8BB}",
            Self::Text | Self::Phrase => "\u{E8D2}",
            Self::Image => "\u{E8B9}",
            Self::File => "\u{E8A5}",
            Self::Folder => "\u{E8B7}",
            Self::Save => "\u{E74E}",
            Self::Undo => "\u{E7A7}",
            Self::Cut => "\u{E8C6}",
            Self::Pin => "\u{E718}",
            Self::Delete => "\u{E74D}",
            Self::Copy => "\u{E8C8}",
            Self::Paste => "\u{E77F}",
            Self::Edit => "\u{E70F}",
            Self::Group => "\u{E902}",
        }
    }

    pub const fn sf_symbol_name(self) -> &'static str {
        match self {
            Self::App => "square.grid.2x2",
            Self::Calculator => "plus.forwardslash.minus",
            Self::History => "clock.arrow.circlepath",
            Self::Backspace => "delete.backward",
            Self::Add => "plus",
            Self::Search => "magnifyingglass",
            Self::Settings => "gearshape",
            Self::Sidebar => "sidebar.left",
            Self::Inspector => "sidebar.right",
            Self::More => "ellipsis",
            Self::Attach => "paperclip",
            Self::Send => "paperplane",
            Self::Stop => "stop.fill",
            Self::Refresh => "arrow.clockwise",
            Self::Retry => "arrow.counterclockwise",
            Self::Code => "chevron.left.forwardslash.chevron.right",
            Self::Tool => "wrench.and.screwdriver",
            Self::Check => "checkmark",
            Self::Minimize => "minus",
            Self::Close => "xmark",
            Self::Text | Self::Phrase => "textformat",
            Self::Image => "photo",
            Self::File => "doc",
            Self::Folder => "folder",
            Self::Save => "square.and.arrow.down",
            Self::Undo => "arrow.uturn.backward",
            Self::Cut => "scissors",
            Self::Pin => "pin",
            Self::Delete => "trash",
            Self::Copy => "doc.on.doc",
            Self::Paste => "doc.on.clipboard",
            Self::Edit => "pencil",
            Self::Group => "person.2",
        }
    }

    pub const fn png_24_bytes(self) -> Option<&'static [u8]> {
        match self {
            Self::Search => Some(include_bytes!("../assets/icons/search/search_24x24.png")),
            Self::Settings => Some(include_bytes!("../assets/icons/setting/setting_24x24.png")),
            Self::Minimize => Some(include_bytes!("../assets/icons/min/min_24x24.png")),
            Self::Close => Some(include_bytes!("../assets/icons/exit/exit_24x24.png")),
            Self::Text | Self::Phrase => {
                Some(include_bytes!("../assets/icons/text/text_24x24.png"))
            }
            Self::Image => Some(include_bytes!("../assets/icons/image/image_24x24.png")),
            Self::File => Some(include_bytes!("../assets/icons/file/file_24x24.png")),
            Self::Folder | Self::Group => {
                Some(include_bytes!("../assets/icons/fold/fold_24x24.png"))
            }
            Self::Pin => Some(include_bytes!("../assets/icons/top/top_24x24.png")),
            Self::Delete => Some(include_bytes!("../assets/icons/del/del_24x24.png")),
            Self::App
            | Self::Calculator
            | Self::History
            | Self::Backspace
            | Self::Add
            | Self::Sidebar
            | Self::Inspector
            | Self::More
            | Self::Attach
            | Self::Send
            | Self::Stop
            | Self::Refresh
            | Self::Retry
            | Self::Code
            | Self::Tool
            | Self::Check
            | Self::Save
            | Self::Undo
            | Self::Cut
            | Self::Copy
            | Self::Paste
            | Self::Edit => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ZsIcon;

    #[test]
    fn png_24_assets_cover_shared_clipboard_row_icons() {
        for icon in [
            ZsIcon::Text,
            ZsIcon::Phrase,
            ZsIcon::Image,
            ZsIcon::File,
            ZsIcon::Folder,
            ZsIcon::Pin,
            ZsIcon::Delete,
        ] {
            let bytes = icon.png_24_bytes().expect("row icon should have PNG asset");
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }

    #[test]
    fn unavailable_png_assets_are_explicit() {
        assert!(ZsIcon::Copy.png_24_bytes().is_none());
        assert!(ZsIcon::Paste.png_24_bytes().is_none());
        assert!(ZsIcon::Edit.png_24_bytes().is_none());
        assert!(ZsIcon::Save.png_24_bytes().is_none());
        assert!(ZsIcon::Undo.png_24_bytes().is_none());
        assert!(ZsIcon::Cut.png_24_bytes().is_none());
        assert!(ZsIcon::Calculator.png_24_bytes().is_none());
        assert!(ZsIcon::History.png_24_bytes().is_none());
        assert!(ZsIcon::Backspace.png_24_bytes().is_none());
    }

    #[test]
    fn semantic_icons_have_native_symbol_mappings() {
        for icon in [
            ZsIcon::Calculator,
            ZsIcon::History,
            ZsIcon::Backspace,
            ZsIcon::Add,
            ZsIcon::Sidebar,
            ZsIcon::Inspector,
            ZsIcon::Attach,
            ZsIcon::Send,
            ZsIcon::Refresh,
            ZsIcon::Code,
            ZsIcon::Tool,
            ZsIcon::Save,
            ZsIcon::Undo,
            ZsIcon::Cut,
        ] {
            assert_eq!(icon.windows_fluent_glyph().chars().count(), 1);
            assert!(icon.gtk_symbolic_name().ends_with("-symbolic"));
            assert!(!icon.sf_symbol_name().is_empty());
        }
    }
}
