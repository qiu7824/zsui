use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::native_icons::NativeIconSource;
use crate::{
    CapabilityStatus, CapabilitySupport, ClipboardData, Command, DialogResponse, Dpi,
    FileDialogSpec, MenuSpec, NativeDialogSpec, PlatformName, Rect, WidgetId, WindowId, WindowSpec,
    ZsIcon, ZsuiError, ZsuiResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DesktopCapability {
    NativeWindow,
    WindowResize,
    ScaleFactor,
    KeyboardFocus,
    PointerInput,
    TextInput,
    InputMethod,
    NativeMenu,
    ClipboardText,
    OpenFileDialog,
    SaveFileDialog,
    NativeDialog,
    SystemTheme,
    NativeIcons,
}

impl DesktopCapability {
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::NativeWindow => "native_window",
            Self::WindowResize => "window_resize",
            Self::ScaleFactor => "scale_factor",
            Self::KeyboardFocus => "keyboard_focus",
            Self::PointerInput => "pointer_input",
            Self::TextInput => "text_input",
            Self::InputMethod => "input_method",
            Self::NativeMenu => "native_menu",
            Self::ClipboardText => "clipboard_text",
            Self::OpenFileDialog => "open_file_dialog",
            Self::SaveFileDialog => "save_file_dialog",
            Self::NativeDialog => "native_dialog",
            Self::SystemTheme => "system_theme",
            Self::NativeIcons => "native_icons",
        }
    }
}

pub const REQUIRED_DESKTOP_CAPABILITIES: [DesktopCapability; 14] = [
    DesktopCapability::NativeWindow,
    DesktopCapability::WindowResize,
    DesktopCapability::ScaleFactor,
    DesktopCapability::KeyboardFocus,
    DesktopCapability::PointerInput,
    DesktopCapability::TextInput,
    DesktopCapability::InputMethod,
    DesktopCapability::NativeMenu,
    DesktopCapability::ClipboardText,
    DesktopCapability::OpenFileDialog,
    DesktopCapability::SaveFileDialog,
    DesktopCapability::NativeDialog,
    DesktopCapability::SystemTheme,
    DesktopCapability::NativeIcons,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCapabilityEntry {
    pub capability: DesktopCapability,
    pub support: CapabilitySupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCapabilities {
    pub platform: PlatformName,
    pub entries: Vec<DesktopCapabilityEntry>,
}

impl DesktopCapabilities {
    pub fn all_unsupported(platform: PlatformName) -> Self {
        Self {
            platform,
            entries: REQUIRED_DESKTOP_CAPABILITIES
                .into_iter()
                .map(|capability| DesktopCapabilityEntry {
                    capability,
                    support: CapabilitySupport::unsupported(
                        "no runtime implementation has been registered by this host",
                    ),
                })
                .collect(),
        }
    }

    pub fn with_support(
        mut self,
        capability: DesktopCapability,
        support: CapabilitySupport,
    ) -> Self {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.capability == capability)
        {
            entry.support = support;
        } else {
            self.entries.push(DesktopCapabilityEntry {
                capability,
                support,
            });
        }
        self
    }

    pub fn support(&self, capability: DesktopCapability) -> Option<&CapabilitySupport> {
        self.entries
            .iter()
            .find(|entry| entry.capability == capability)
            .map(|entry| &entry.support)
    }

    pub fn require(&self, capability: DesktopCapability) -> ZsuiResult<()> {
        let Some(support) = self.support(capability) else {
            return Err(ZsuiError::unsupported(
                capability.capability_name(),
                "the host omitted this capability from its capability set",
            ));
        };
        if support.status == CapabilityStatus::Supported {
            Ok(())
        } else {
            Err(ZsuiError::unsupported(
                capability.capability_name(),
                support.detail.clone(),
            ))
        }
    }

    pub fn is_fully_supported(&self, capability: DesktopCapability) -> bool {
        self.support(capability)
            .is_some_and(CapabilitySupport::is_fully_supported)
    }

    pub fn missing_or_incomplete(&self) -> Vec<DesktopCapability> {
        REQUIRED_DESKTOP_CAPABILITIES
            .into_iter()
            .filter(|capability| !self.is_fully_supported(*capability))
            .collect()
    }

    pub fn windows_win32_current() -> Self {
        Self::all_unsupported(PlatformName::Windows)
            .with_support(
                DesktopCapability::NativeWindow,
                CapabilitySupport::supported("the Win32 HWND lifecycle is connected"),
            )
            .with_support(
                DesktopCapability::WindowResize,
                CapabilitySupport::supported("Win32 resize messages relayout and repaint the view"),
            )
            .with_support(
                DesktopCapability::ScaleFactor,
                CapabilitySupport::partial(
                    "WM_DPICHANGED relayout is connected; multi-monitor target proof is pending",
                ),
            )
            .with_support(
                DesktopCapability::KeyboardFocus,
                CapabilitySupport::supported(
                    "click and Tab focus routing plus the shared semantic focus ring are connected",
                ),
            )
            .with_support(
                DesktopCapability::PointerInput,
                CapabilitySupport::supported(
                    "pointer click, wheel and capture-backed shaped text drag selection with editor-edge viewport scrolling are connected",
                ),
            )
            .with_support(
                DesktopCapability::TextInput,
                CapabilitySupport::supported(
                    "single-line and multiline Unicode routing uses Uniscribe advances and bidirectional insertion geometry with extended-grapheme-safe editing, line/page navigation and range replacement",
                ),
            )
            .with_support(
                DesktopCapability::InputMethod,
                CapabilitySupport::partial(
                    "IMM32 result commit and shaped-caret candidate placement are connected; visual-order bidi navigation and CJK target proof are pending",
                ),
            )
            .with_support(
                DesktopCapability::NativeMenu,
                CapabilitySupport::supported(
                    "owned HMENU/HACCEL command and keyboard routing re-enter typed stateful-view updates",
                ),
            )
            .with_support(
                DesktopCapability::ClipboardText,
                if cfg!(feature = "clipboard") {
                    CapabilitySupport::supported("the optional system text clipboard is compiled")
                } else {
                    CapabilitySupport::unsupported(
                        "enable the clipboard feature to compile text clipboard support",
                    )
                },
            )
            .with_support(
                DesktopCapability::OpenFileDialog,
                CapabilitySupport::partial(
                    "the Win32 common open dialog is connected; target screenshot proof is pending",
                ),
            )
            .with_support(
                DesktopCapability::SaveFileDialog,
                CapabilitySupport::partial(
                    "the Win32 common save dialog is connected; target screenshot proof is pending",
                ),
            )
            .with_support(
                DesktopCapability::NativeDialog,
                CapabilitySupport::partial(
                    "owner-bound Win32 MessageBoxW dialogs map typed levels, buttons and responses; target interaction proof is pending",
                ),
            )
            .with_support(
                DesktopCapability::SystemTheme,
                CapabilitySupport::partial(
                    "light/dark and SPI_GETHIGHCONTRAST detection, user-selected GetSysColor pairs and repaint on system color/theme changes are connected; live OS setting-change proof is pending",
                ),
            )
            .with_support(
                DesktopCapability::NativeIcons,
                CapabilitySupport::supported(
                    "the GDI renderer detects Segoe Fluent Icons and falls back to Segoe MDL2 Assets",
                ),
            )
    }

    pub fn macos_appkit_current() -> Self {
        Self::all_unsupported(PlatformName::Macos)
            .with_support(
                DesktopCapability::NativeWindow,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSApplication/NSWindow creation, owned lifecycle, draw-plan rendering and typed pointer/keyboard routing are connected; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile the native AppKit window service",
                    )
                },
            )
            .with_support(
                DesktopCapability::KeyboardFocus,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSView first-responder focus, Tab traversal, keyboard activation and shared semantic focus rings are connected; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit keyboard focus routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::PointerInput,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSView mouse activation, mouseDragged shaped text selection with shared editor-edge viewport scrolling and scrollWheel routing are connected; richer gestures and target proof are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit pointer routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::TextInput,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "focused UTF-8 input uses Core Text advances and strong/weak bidirectional insertion geometry with extended-grapheme-safe editing and range replacement; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit text input routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::InputMethod,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSTextInputClient preedit/commit, UTF-16 ranges, replacement and Core Text shaped-caret candidate anchoring are connected; visual-order bidi navigation and CJK target proof are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile NSTextInputClient composition routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::WindowResize,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "actual NSView bounds relayout and repaint shared live/static views and refresh input geometry; target resize artifacts and public WindowResized event routing are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit window resizing",
                    )
                },
            )
            .with_support(
                DesktopCapability::ScaleFactor,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSView backing scale and resize geometry feed the shared DPI-aware layout; multi-display target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit backing-scale support",
                    )
                },
            )
            .with_support(
                DesktopCapability::ClipboardText,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSPasteboard UTF-8 text read/write is connected; AppKit host proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile the native AppKit clipboard service",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeMenu,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSMenu/NSMenuItem commands re-enter typed stateful-view updates and repaint the owned NSView; AppKit target interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile the native AppKit menu service",
                    )
                },
            )
            .with_support(
                DesktopCapability::OpenFileDialog,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSOpenPanel is connected; target interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile NSOpenPanel",
                    )
                },
            )
            .with_support(
                DesktopCapability::SaveFileDialog,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "NSSavePanel is connected; target interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile NSSavePanel",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeDialog,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "owner-bound NSAlert sheets map typed levels, platform action order and responses; target interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile native NSAlert dialogs",
                    )
                },
            )
            .with_support(
                DesktopCapability::SystemTheme,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "effective AppKit light/dark/high-contrast appearances, semantic NSColor resolution and appearance-change repaint are connected; target live-change proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit system appearance support",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeIcons,
                if cfg!(feature = "macos-appkit") {
                    CapabilitySupport::partial(
                        "SF Symbols are resolved and painted by the AppKit draw sink; target visual proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable macos-appkit to compile AppKit SF Symbols support",
                    )
                },
            )
    }

    pub fn linux_gtk_current() -> Self {
        Self::all_unsupported(PlatformName::Linux)
            .with_support(
                DesktopCapability::NativeWindow,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GtkApplication/ApplicationWindow creation, owned lifecycle, draw-plan rendering and typed pointer/keyboard routing are connected; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile the native GTK4 window service",
                    )
                },
            )
            .with_support(
                DesktopCapability::KeyboardFocus,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK4 focusable DrawingArea, Tab traversal, keyboard activation and shared semantic focus rings are connected; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 keyboard focus routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::PointerInput,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK4 GestureClick, EventControllerMotion shaped text selection with shared editor-edge viewport scrolling and scroll routing are connected; richer gestures and target proof are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 pointer routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::TextInput,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "focused UTF-8 input uses Pango advances and strong/weak bidirectional insertion geometry with extended-grapheme-safe editing and range replacement; target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 text input routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::InputMethod,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GtkIMMulticontext preedit/commit, surrounding UTF-8 text and Pango shaped-caret candidate anchoring are connected; visual-order bidi navigation and CJK target proof are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GtkIMContext composition routing",
                    )
                },
            )
            .with_support(
                DesktopCapability::WindowResize,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "actual DrawingArea allocation relayouts and repaints shared live/static views and refreshes input geometry; Wayland/X11 resize artifacts and public WindowResized event routing are pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 window resizing",
                    )
                },
            )
            .with_support(
                DesktopCapability::ScaleFactor,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK scale-factor and DrawingArea allocation changes feed the shared DPI-aware layout; Wayland/X11 target proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK scale-factor support",
                    )
                },
            )
            .with_support(
                DesktopCapability::ClipboardText,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GdkClipboard UTF-8 text read/write is connected; Wayland/X11 host proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile the native GTK4 clipboard service",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeMenu,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GMenu/SimpleAction commands re-enter typed stateful-view updates and repaint the owned DrawingArea; GTK target interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile the native GTK4 menu service",
                    )
                },
            )
            .with_support(
                DesktopCapability::OpenFileDialog,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK4 FileChooserNative open is connected; Wayland/X11 interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 FileChooserNative",
                    )
                },
            )
            .with_support(
                DesktopCapability::SaveFileDialog,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK4 FileChooserNative save is connected; Wayland/X11 interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 FileChooserNative",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeDialog,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK 4.10 AlertDialog maps typed buttons and responses with GTK-owned action order; Wayland/X11 interaction proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK AlertDialog support",
                    )
                },
            )
            .with_support(
                DesktopCapability::SystemTheme,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GTK light/dark/high-contrast theme detection, semantic theme-color lookup and settings-change repaint are connected; Wayland/X11 live-change proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK system appearance support",
                    )
                },
            )
            .with_support(
                DesktopCapability::NativeIcons,
                if cfg!(feature = "linux-gtk") {
                    CapabilitySupport::partial(
                        "GtkIconTheme lookup and bundled Fluent SVG fallback are painted by the GTK4 draw sink; target visual proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-gtk to compile GTK4 icon-theme support",
                    )
                },
            )
    }

    pub fn linux_direct_current() -> Self {
        let compiled = cfg!(feature = "linux-direct-host");
        let support = |ready: &'static str, disabled: &'static str| {
            if compiled {
                CapabilitySupport::partial(ready)
            } else {
                CapabilitySupport::unsupported(disabled)
            }
        };
        Self::all_unsupported(PlatformName::Linux)
            .with_support(
                DesktopCapability::NativeWindow,
                support(
                    "real Wayland/X11 windows, software presentation, shared draw-plan rendering and live resize are connected; target proof is required",
                    "enable linux-direct to compile the lightweight Linux host",
                ),
            )
            .with_support(
                DesktopCapability::KeyboardFocus,
                support(
                    "native focus, keyboard events, Tab traversal and semantic focus visuals are connected through the shared runtime; target proof is required",
                    "enable linux-direct to compile Linux keyboard and focus routing",
                ),
            )
            .with_support(
                DesktopCapability::PointerInput,
                support(
                    "native pointer motion, press, release and wheel events are routed into the shared runtime; richer gestures and target proof are pending",
                    "enable linux-direct to compile Linux pointer routing",
                ),
            )
            .with_support(
                DesktopCapability::TextInput,
                support(
                    "Pango-shaped UTF-8 editing, selection geometry and shared text viewport behavior are connected; target proof is required",
                    "enable linux-direct to compile Linux text shaping and input",
                ),
            )
            .with_support(
                DesktopCapability::InputMethod,
                support(
                    "the native Wayland/X11 IME event path, preedit, commit and caret rectangle are connected; candidate-window and CJK target proof are pending",
                    "enable linux-direct to compile Linux IME routing",
                ),
            )
            .with_support(
                DesktopCapability::WindowResize,
                support(
                    "native resize and scale-factor events resize the software surface and relayout the shared view; Wayland/X11 proof is pending",
                    "enable linux-direct to compile Linux resize routing",
                ),
            )
            .with_support(
                DesktopCapability::ScaleFactor,
                support(
                    "native scale-factor events resize the presentation surface and feed the shared DPI-aware layout; Wayland/X11 target proof is pending",
                    "enable linux-direct to compile Linux scale-factor routing",
                ),
            )
            .with_support(
                DesktopCapability::ClipboardText,
                if compiled && cfg!(feature = "clipboard") {
                    CapabilitySupport::partial(
                        "the system text clipboard is connected without GTK; Wayland/X11 ownership proof is pending",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-direct and clipboard to compile Linux clipboard support",
                    )
                },
            )
            .with_support(
                DesktopCapability::OpenFileDialog,
                support(
                    "the XDG desktop portal open-file dialog is connected without GTK; target interaction proof is pending",
                    "enable linux-direct to compile XDG portal file dialogs",
                ),
            )
            .with_support(
                DesktopCapability::SaveFileDialog,
                support(
                    "the XDG desktop portal save-file dialog is connected without GTK; target interaction proof is pending",
                    "enable linux-direct to compile XDG portal file dialogs",
                ),
            )
            .with_support(
                DesktopCapability::NativeDialog,
                support(
                    "the lightweight host maps typed message dialogs through the desktop-provided Zenity surface; provider and target interaction proof are required",
                    "enable linux-direct to compile Linux message-dialog support",
                ),
            )
            .with_support(
                DesktopCapability::SystemTheme,
                support(
                    "native light/dark notifications update the shared theme; desktop high-contrast integration is pending",
                    "enable linux-direct to compile Linux system appearance support",
                ),
            )
            .with_support(
                DesktopCapability::NativeIcons,
                support(
                    "the complete Cairo symbolic set is the default and optional linux-system-icons enables exact freedesktop lookup; target visual proof is pending",
                    "enable linux-direct to compile Linux native icon rendering",
                ),
            )
            .with_support(
                DesktopCapability::NativeMenu,
                if compiled {
                    CapabilitySupport::supported(
                        "the owned desktop menu bar, popup navigation, accelerators and typed command routing are connected on the direct host",
                    )
                } else {
                    CapabilitySupport::unsupported(
                        "enable linux-direct to compile the owned Linux desktop menu surface",
                    )
                },
            )
    }

    pub fn current_native_backend() -> Self {
        crate::desktop_runtime::desktop_capabilities()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopTheme {
    Light,
    Dark,
    HighContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopKey {
    Character(String),
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesktopEvent {
    WindowCloseRequested {
        window: WindowId,
    },
    WindowResized {
        window: WindowId,
        width: u32,
        height: u32,
    },
    ScaleFactorChanged {
        window: WindowId,
        dpi: Dpi,
    },
    WindowFocusChanged {
        window: WindowId,
        focused: bool,
    },
    KeyDown {
        window: WindowId,
        key: DesktopKey,
        modifiers: KeyModifiers,
    },
    TextInput {
        window: WindowId,
        text: String,
    },
    InputMethodPreedit {
        window: WindowId,
        text: String,
        selection: Option<(usize, usize)>,
    },
    InputMethodCommit {
        window: WindowId,
        text: String,
    },
    MenuCommand {
        window: WindowId,
        command: Command,
    },
    SystemThemeChanged(DesktopTheme),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveFileDialogSpec {
    pub title: String,
    pub current_path: Option<PathBuf>,
    pub suggested_name: Option<String>,
    pub filters: Vec<crate::FileDialogFilter>,
}

impl SaveFileDialogSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            current_path: None,
            suggested_name: None,
            filters: Vec::new(),
        }
    }

    pub fn current_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_path = Some(path.into());
        self
    }

    pub fn suggested_name(mut self, name: impl Into<String>) -> Self {
        self.suggested_name = Some(name.into());
        self
    }

    pub fn filter(
        mut self,
        name: impl Into<String>,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.filters
            .push(crate::FileDialogFilter::new(name, patterns));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInputRequest {
    pub window: WindowId,
    pub widget: WidgetId,
    pub caret_rect: Rect,
    pub multiline: bool,
}

pub trait WindowService {
    fn create_window(&mut self, spec: &WindowSpec) -> ZsuiResult<WindowId>;
    fn set_window_title(&mut self, window: WindowId, title: &str) -> ZsuiResult<()>;
    fn set_window_visible(&mut self, window: WindowId, visible: bool) -> ZsuiResult<()>;
    fn request_window_redraw(&mut self, window: WindowId) -> ZsuiResult<()>;
    fn close_window(&mut self, window: WindowId) -> ZsuiResult<()>;
}

pub trait ClipboardService {
    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>>;
    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()>;
}

/// Target-dispatched system text clipboard used by shared desktop controls.
/// The optional `clipboard` feature must be enabled explicitly.
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeClipboardService;

impl NativeClipboardService {
    pub const fn new() -> Self {
        Self
    }
}

impl ClipboardService for NativeClipboardService {
    fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
        crate::desktop_runtime::read_clipboard()
    }

    fn write_clipboard(&mut self, data: &ClipboardData) -> ZsuiResult<()> {
        crate::desktop_runtime::write_clipboard(data)
    }
}

pub trait FileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>>;
    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>>;
}

pub trait NativeDialogService {
    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse>;
}

/// Target-dispatched native message dialog. The selected desktop backend owns
/// its platform action order, modality and response mapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeDesktopDialogService;

impl NativeDesktopDialogService {
    pub const fn new() -> Self {
        Self
    }
}

impl NativeDialogService for NativeDesktopDialogService {
    fn show_native_dialog(&mut self, spec: &NativeDialogSpec) -> ZsuiResult<DialogResponse> {
        crate::desktop_runtime::show_native_dialog(spec)
    }
}

/// Target-dispatched file dialogs bound to the active native window when one
/// is available, with application-modal fallback when no owner exists.
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeFileDialogService;

impl NativeFileDialogService {
    pub const fn new() -> Self {
        Self
    }
}

impl FileDialogService for NativeFileDialogService {
    fn open_file_dialog(&mut self, spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
        crate::desktop_runtime::open_file_dialog_required(spec)
    }

    fn save_file_dialog(&mut self, spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
        crate::desktop_runtime::save_file_dialog(spec)
    }
}

pub trait MenuService {
    fn set_window_menu(&mut self, window: WindowId, menu: Option<&MenuSpec>) -> ZsuiResult<()>;
}

pub trait ThemeService {
    fn system_theme(&self) -> ZsuiResult<DesktopTheme>;
    fn set_theme_preference(&mut self, preference: ThemePreference) -> ZsuiResult<()>;
}

pub trait TextInputService {
    fn focus_text_input(&mut self, request: TextInputRequest) -> ZsuiResult<()>;
    fn update_text_input_caret(&mut self, window: WindowId, caret_rect: Rect) -> ZsuiResult<()>;
    fn blur_text_input(&mut self, window: WindowId) -> ZsuiResult<()>;
}

pub trait IconService {
    fn resolve_icon(&self, icon: ZsIcon) -> ZsuiResult<NativeIconSource>;
}

pub trait DesktopHost:
    WindowService
    + ClipboardService
    + FileDialogService
    + MenuService
    + ThemeService
    + TextInputService
    + IconService
{
    fn desktop_capabilities(&self) -> &DesktopCapabilities;
    fn poll_desktop_event(&mut self) -> ZsuiResult<Option<DesktopEvent>>;
    fn run_desktop_event_loop(&mut self) -> ZsuiResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_icons::native_icon_candidates;

    #[test]
    fn native_dialog_facade_implements_the_safe_service_contract() {
        fn assert_service<T: NativeDialogService>() {}
        assert_service::<NativeDesktopDialogService>();
    }

    struct ContractHost {
        capabilities: DesktopCapabilities,
        events: Vec<DesktopEvent>,
    }

    impl ContractHost {
        fn require(&self, capability: DesktopCapability) -> ZsuiResult<()> {
            self.capabilities.require(capability)
        }
    }

    impl WindowService for ContractHost {
        fn create_window(&mut self, _spec: &WindowSpec) -> ZsuiResult<WindowId> {
            self.require(DesktopCapability::NativeWindow)?;
            Ok(WindowId(1))
        }

        fn set_window_title(&mut self, _window: WindowId, _title: &str) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeWindow)
        }

        fn set_window_visible(&mut self, _window: WindowId, _visible: bool) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeWindow)
        }

        fn request_window_redraw(&mut self, _window: WindowId) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeWindow)
        }

        fn close_window(&mut self, _window: WindowId) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeWindow)
        }
    }

    impl ClipboardService for ContractHost {
        fn read_clipboard(&mut self) -> ZsuiResult<Option<ClipboardData>> {
            self.require(DesktopCapability::ClipboardText)?;
            Ok(None)
        }

        fn write_clipboard(&mut self, _data: &ClipboardData) -> ZsuiResult<()> {
            self.require(DesktopCapability::ClipboardText)
        }
    }

    impl FileDialogService for ContractHost {
        fn open_file_dialog(&mut self, _spec: &FileDialogSpec) -> ZsuiResult<Option<Vec<PathBuf>>> {
            self.require(DesktopCapability::OpenFileDialog)?;
            Ok(None)
        }

        fn save_file_dialog(&mut self, _spec: &SaveFileDialogSpec) -> ZsuiResult<Option<PathBuf>> {
            self.require(DesktopCapability::SaveFileDialog)?;
            Ok(None)
        }
    }

    impl MenuService for ContractHost {
        fn set_window_menu(
            &mut self,
            _window: WindowId,
            _menu: Option<&MenuSpec>,
        ) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeMenu)
        }
    }

    impl ThemeService for ContractHost {
        fn system_theme(&self) -> ZsuiResult<DesktopTheme> {
            self.require(DesktopCapability::SystemTheme)?;
            Ok(DesktopTheme::Light)
        }

        fn set_theme_preference(&mut self, _preference: ThemePreference) -> ZsuiResult<()> {
            self.require(DesktopCapability::SystemTheme)
        }
    }

    impl TextInputService for ContractHost {
        fn focus_text_input(&mut self, _request: TextInputRequest) -> ZsuiResult<()> {
            self.require(DesktopCapability::TextInput)?;
            self.require(DesktopCapability::InputMethod)
        }

        fn update_text_input_caret(
            &mut self,
            _window: WindowId,
            _caret_rect: Rect,
        ) -> ZsuiResult<()> {
            self.require(DesktopCapability::InputMethod)
        }

        fn blur_text_input(&mut self, _window: WindowId) -> ZsuiResult<()> {
            self.require(DesktopCapability::TextInput)
        }
    }

    impl IconService for ContractHost {
        fn resolve_icon(&self, icon: ZsIcon) -> ZsuiResult<NativeIconSource> {
            self.require(DesktopCapability::NativeIcons)?;
            native_icon_candidates(&self.capabilities.platform, icon)
                .into_iter()
                .next()
                .ok_or_else(|| {
                    ZsuiError::unsupported(
                        "native_icons",
                        "the contract host has no icon source for this platform",
                    )
                })
        }
    }

    impl DesktopHost for ContractHost {
        fn desktop_capabilities(&self) -> &DesktopCapabilities {
            &self.capabilities
        }

        fn poll_desktop_event(&mut self) -> ZsuiResult<Option<DesktopEvent>> {
            Ok(self.events.pop())
        }

        fn run_desktop_event_loop(&mut self) -> ZsuiResult<()> {
            self.require(DesktopCapability::NativeWindow)
        }
    }

    #[test]
    fn capability_set_is_complete_and_rejects_partial_runtime_claims() {
        let capabilities = DesktopCapabilities::all_unsupported(PlatformName::Linux)
            .with_support(
                DesktopCapability::NativeWindow,
                CapabilitySupport::supported("a real GTK window is connected"),
            )
            .with_support(
                DesktopCapability::InputMethod,
                CapabilitySupport::partial("preedit is not connected"),
            );

        assert_eq!(
            capabilities.entries.len(),
            REQUIRED_DESKTOP_CAPABILITIES.len()
        );
        assert!(capabilities
            .require(DesktopCapability::NativeWindow)
            .is_ok());
        assert!(matches!(
            capabilities.require(DesktopCapability::InputMethod),
            Err(ZsuiError::Unsupported { .. })
        ));
        assert_eq!(
            capabilities.missing_or_incomplete().len(),
            REQUIRED_DESKTOP_CAPABILITIES.len() - 1
        );
    }

    #[test]
    fn desktop_host_contract_is_object_safe_and_routes_typed_events() {
        let capabilities = REQUIRED_DESKTOP_CAPABILITIES.into_iter().fold(
            DesktopCapabilities::all_unsupported(PlatformName::Unknown),
            |capabilities, capability| {
                capabilities.with_support(
                    capability,
                    CapabilitySupport::supported("contract test implementation"),
                )
            },
        );
        let mut host = ContractHost {
            capabilities,
            events: vec![DesktopEvent::MenuCommand {
                window: WindowId(1),
                command: Command::CopySelection,
            }],
        };
        let host: &mut dyn DesktopHost = &mut host;

        assert!(host
            .desktop_capabilities()
            .missing_or_incomplete()
            .is_empty());
        assert_eq!(
            host.poll_desktop_event().unwrap(),
            Some(DesktopEvent::MenuCommand {
                window: WindowId(1),
                command: Command::CopySelection,
            })
        );
    }

    #[test]
    fn save_dialog_and_text_input_requests_use_owned_safe_types() {
        let open = FileDialogSpec::new("Open")
            .current_path("documents/notes.txt")
            .filter("Text", ["*.txt"]);
        let save = SaveFileDialogSpec::new("Save")
            .current_path("documents")
            .suggested_name("notes.txt")
            .filter("Text", ["*.txt"]);
        let input = TextInputRequest {
            window: WindowId(4),
            widget: WidgetId::new(9),
            caret_rect: Rect {
                x: 10,
                y: 20,
                width: 1,
                height: 18,
            },
            multiline: true,
        };

        assert_eq!(
            open.current_path,
            Some(PathBuf::from("documents/notes.txt"))
        );
        assert_eq!(save.suggested_name.as_deref(), Some("notes.txt"));
        assert!(input.multiline);
        assert_eq!(input.widget, WidgetId::new(9));
    }

    #[cfg(not(any(
        all(windows, feature = "windows-win32"),
        all(target_os = "macos", feature = "macos-appkit"),
        all(
            target_os = "linux",
            not(target_env = "ohos"),
            any(feature = "linux-direct-host", feature = "linux-gtk")
        )
    )))]
    #[test]
    fn native_desktop_service_facades_report_a_missing_backend() {
        let mut clipboard = NativeClipboardService::new();
        let mut dialogs = NativeFileDialogService::new();

        assert!(matches!(
            clipboard.read_clipboard(),
            Err(ZsuiError::Unsupported { capability, .. }) if capability == "read_clipboard"
        ));
        assert!(matches!(
            clipboard.write_clipboard(&ClipboardData::Text("test".to_string())),
            Err(ZsuiError::Unsupported { capability, .. }) if capability == "write_clipboard"
        ));
        assert!(matches!(
            dialogs.open_file_dialog(&FileDialogSpec::new("Open")),
            Err(ZsuiError::Unsupported { capability, .. }) if capability == "open_file_dialog"
        ));
        assert!(matches!(
            dialogs.save_file_dialog(&SaveFileDialogSpec::new("Save")),
            Err(ZsuiError::Unsupported { capability, .. }) if capability == "save_file_dialog"
        ));
    }

    #[test]
    fn backend_capabilities_keep_unverified_work_incomplete() {
        let windows = DesktopCapabilities::windows_win32_current();
        let macos = DesktopCapabilities::macos_appkit_current();
        let linux = DesktopCapabilities::linux_direct_current();

        assert!(windows.is_fully_supported(DesktopCapability::NativeWindow));
        assert!(windows.is_fully_supported(DesktopCapability::NativeMenu));
        assert!(windows.is_fully_supported(DesktopCapability::NativeIcons));
        assert!(!windows.is_fully_supported(DesktopCapability::InputMethod));
        assert!(!windows.is_fully_supported(DesktopCapability::OpenFileDialog));
        assert_eq!(
            macos
                .support(DesktopCapability::ClipboardText)
                .map(|support| support.status),
            Some(if cfg!(feature = "macos-appkit") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            })
        );
        assert_eq!(
            linux
                .support(DesktopCapability::ClipboardText)
                .map(|support| support.status),
            Some(
                if cfg!(all(feature = "linux-direct-host", feature = "clipboard")) {
                    CapabilityStatus::Partial
                } else {
                    CapabilityStatus::Unsupported
                }
            )
        );
        for capability in [
            DesktopCapability::KeyboardFocus,
            DesktopCapability::PointerInput,
            DesktopCapability::TextInput,
            DesktopCapability::InputMethod,
        ] {
            assert_eq!(
                macos.support(capability).map(|support| support.status),
                Some(if cfg!(feature = "macos-appkit") {
                    CapabilityStatus::Partial
                } else {
                    CapabilityStatus::Unsupported
                })
            );
            assert_eq!(
                linux.support(capability).map(|support| support.status),
                Some(if cfg!(feature = "linux-direct-host") {
                    CapabilityStatus::Partial
                } else {
                    CapabilityStatus::Unsupported
                })
            );
        }
        assert_eq!(
            macos.missing_or_incomplete().len(),
            REQUIRED_DESKTOP_CAPABILITIES.len()
        );
        assert_eq!(
            linux.missing_or_incomplete().len(),
            REQUIRED_DESKTOP_CAPABILITIES.len() - usize::from(cfg!(feature = "linux-direct-host"))
        );
    }

    #[test]
    fn backend_theme_capabilities_do_not_cross_platforms() {
        let windows = DesktopCapabilities::windows_win32_current();
        let macos = DesktopCapabilities::macos_appkit_current();
        let gtk = DesktopCapabilities::linux_gtk_current();

        let windows_theme = windows.support(DesktopCapability::SystemTheme).unwrap();
        assert!(windows_theme.detail.contains("SPI_GETHIGHCONTRAST"));
        assert!(!windows_theme.detail.contains("AppKit"));
        assert!(!windows_theme.detail.contains("GTK"));

        let macos_theme = macos.support(DesktopCapability::SystemTheme).unwrap();
        assert_eq!(
            macos_theme.status,
            if cfg!(feature = "macos-appkit") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert!(macos_theme.detail.contains("AppKit"));
        assert!(!macos_theme.detail.contains("GTK"));

        let gtk_theme = gtk.support(DesktopCapability::SystemTheme).unwrap();
        assert_eq!(
            gtk_theme.status,
            if cfg!(feature = "linux-gtk") {
                CapabilityStatus::Partial
            } else {
                CapabilityStatus::Unsupported
            }
        );
        assert!(gtk_theme.detail.contains("GTK"));
        assert!(!gtk_theme.detail.contains("AppKit"));
    }

    #[test]
    fn icon_service_returns_safe_semantic_platform_sources() {
        let host = ContractHost {
            capabilities: DesktopCapabilities::all_unsupported(PlatformName::Windows).with_support(
                DesktopCapability::NativeIcons,
                CapabilitySupport::supported("contract test implementation"),
            ),
            events: Vec::new(),
        };

        let source = host.resolve_icon(ZsIcon::Save).unwrap();
        assert_eq!(source.icon, ZsIcon::Save);
        assert_eq!(source.identifier, crate::WINDOWS_FLUENT_ICON_FONT_FAMILY);
        assert!(source.kind.is_platform_native());
    }
}
