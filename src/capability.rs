use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformName {
    Windows,
    Macos,
    Linux,
    Android,
    Unknown,
    Other(String),
}

impl PlatformName {
    pub fn current() -> Self {
        match crate::NativeUiPlatform::current_target() {
            Some(platform) => platform.into(),
            None => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Unknown => "unknown",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl From<crate::NativeUiPlatform> for PlatformName {
    fn from(platform: crate::NativeUiPlatform) -> Self {
        match platform {
            crate::NativeUiPlatform::Windows => Self::Windows,
            crate::NativeUiPlatform::Macos => Self::Macos,
            crate::NativeUiPlatform::Linux => Self::Linux,
            crate::NativeUiPlatform::Android => Self::Android,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySupport {
    pub status: CapabilityStatus,
    pub detail: String,
}

impl CapabilitySupport {
    pub fn supported(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Supported,
            detail: detail.into(),
        }
    }

    pub fn partial(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Partial,
            detail: detail.into(),
        }
    }

    pub fn unsupported(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Unsupported,
            detail: detail.into(),
        }
    }

    pub fn accepts_declaration(&self) -> bool {
        !matches!(self.status, CapabilityStatus::Unsupported)
    }

    pub fn is_fully_supported(&self) -> bool {
        matches!(self.status, CapabilityStatus::Supported)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub platform: PlatformName,
    pub windows: CapabilitySupport,
    pub window_resizing: CapabilitySupport,
    pub window_decorations: CapabilitySupport,
    pub window_always_on_top: CapabilitySupport,
    pub window_transparency: CapabilitySupport,
    pub tray_or_status_menu: CapabilitySupport,
    pub menus: CapabilitySupport,
    pub global_hotkeys: CapabilitySupport,
    pub clipboard_text: CapabilitySupport,
    pub clipboard_image: CapabilitySupport,
    pub clipboard_files: CapabilitySupport,
    pub file_picker: CapabilitySupport,
    pub native_dialogs: CapabilitySupport,
    pub settings_pages: CapabilitySupport,
    pub auto_paste: CapabilitySupport,
}

impl HostCapabilities {
    pub fn all_supported(platform: PlatformName) -> Self {
        Self {
            platform,
            windows: CapabilitySupport::supported("window declarations are accepted"),
            window_resizing: CapabilitySupport::supported(
                "resizable and fixed-size windows are honored",
            ),
            window_decorations: CapabilitySupport::supported(
                "native window decorations are honored",
            ),
            window_always_on_top: CapabilitySupport::supported("always-on-top windows are honored"),
            window_transparency: CapabilitySupport::supported("transparent windows are honored"),
            tray_or_status_menu: CapabilitySupport::supported(
                "tray/status menu declarations are accepted",
            ),
            menus: CapabilitySupport::supported("menu declarations are accepted"),
            global_hotkeys: CapabilitySupport::supported("global hotkeys are accepted"),
            clipboard_text: CapabilitySupport::supported("text clipboard is available"),
            clipboard_image: CapabilitySupport::supported("image clipboard is available"),
            clipboard_files: CapabilitySupport::supported("file clipboard is available"),
            file_picker: CapabilitySupport::supported("native file picker is available"),
            native_dialogs: CapabilitySupport::supported("native dialogs are available"),
            settings_pages: CapabilitySupport::supported("settings page declarations are accepted"),
            auto_paste: CapabilitySupport::supported("native auto paste is available"),
        }
    }

    pub fn all_unsupported(platform: PlatformName) -> Self {
        let unsupported = CapabilitySupport::unsupported("not implemented by this host");
        Self {
            platform,
            windows: unsupported.clone(),
            window_resizing: unsupported.clone(),
            window_decorations: unsupported.clone(),
            window_always_on_top: unsupported.clone(),
            window_transparency: unsupported.clone(),
            tray_or_status_menu: unsupported.clone(),
            menus: unsupported.clone(),
            global_hotkeys: unsupported.clone(),
            clipboard_text: unsupported.clone(),
            clipboard_image: unsupported.clone(),
            clipboard_files: unsupported.clone(),
            file_picker: unsupported.clone(),
            native_dialogs: unsupported.clone(),
            settings_pages: unsupported.clone(),
            auto_paste: unsupported,
        }
    }

    pub fn current_platform_scaffold() -> Self {
        crate::desktop_runtime::scaffold_capabilities()
    }

    pub fn current_native_window_host() -> Self {
        crate::desktop_runtime::native_host_capabilities()
    }

    pub fn degraded_capabilities(&self) -> Vec<(&'static str, &CapabilitySupport)> {
        [
            ("windows", &self.windows),
            ("window_resizing", &self.window_resizing),
            ("window_decorations", &self.window_decorations),
            ("window_always_on_top", &self.window_always_on_top),
            ("window_transparency", &self.window_transparency),
            ("tray_or_status_menu", &self.tray_or_status_menu),
            ("menus", &self.menus),
            ("global_hotkeys", &self.global_hotkeys),
            ("clipboard_text", &self.clipboard_text),
            ("clipboard_image", &self.clipboard_image),
            ("clipboard_files", &self.clipboard_files),
            ("file_picker", &self.file_picker),
            ("native_dialogs", &self.native_dialogs),
            ("settings_pages", &self.settings_pages),
            ("auto_paste", &self.auto_paste),
        ]
        .into_iter()
        .filter(|(_, support)| !support.is_fully_supported())
        .collect()
    }

    pub fn windows_scaffold() -> Self {
        Self {
            platform: PlatformName::Windows,
            windows: CapabilitySupport::partial(
                "Win32 main-window hosts are expected; generic PlatformHost currently records declarations",
            ),
            window_resizing: CapabilitySupport::partial(
                "Win32 can create standard resizable windows; generic PlatformHost does not map styles yet",
            ),
            window_decorations: CapabilitySupport::partial(
                "Win32 can create decorated windows; generic PlatformHost does not map styles yet",
            ),
            window_always_on_top: CapabilitySupport::partial(
                "Win32 topmost windows exist; ZsuiHost style mapping is not wired yet",
            ),
            window_transparency: CapabilitySupport::partial(
                "Win32 transparency exists for selected hosts; ZsuiHost mapping is not wired yet",
            ),
            tray_or_status_menu: CapabilitySupport::partial(
                "Win32 tray/status APIs exist; generic PlatformHost currently records declarations",
            ),
            menus: CapabilitySupport::partial(
                "Win32 menu APIs exist; generic PlatformHost currently records declarations",
            ),
            global_hotkeys: CapabilitySupport::partial(
                "Win32 global hotkey APIs exist; generic PlatformHost currently records declarations",
            ),
            clipboard_text: CapabilitySupport::supported("text clipboard bridge is available"),
            clipboard_image: CapabilitySupport::partial(
                "image clipboard depends on backend integration",
            ),
            clipboard_files: CapabilitySupport::partial(
                "file clipboard support requires a native Windows host backend",
            ),
            file_picker: CapabilitySupport::partial(
                "Win32 file picker exists in the platform layer",
            ),
            native_dialogs: CapabilitySupport::partial("Win32 dialogs exist in the platform layer"),
            settings_pages: CapabilitySupport::partial("settings page specs are declarative"),
            auto_paste: CapabilitySupport::partial(
                "Windows paste-target code exists outside ZsuiHost",
            ),
        }
    }

    pub fn windows_native_window_host() -> Self {
        let mut capabilities = Self::windows_scaffold();
        capabilities.windows =
            CapabilitySupport::supported("Win32 native host creates main and quick windows");
        capabilities.window_resizing =
            CapabilitySupport::supported("Win32 window styles honor resizable and fixed windows");
        capabilities.window_decorations =
            CapabilitySupport::supported("Win32 window styles honor native decorations");
        capabilities.window_always_on_top =
            CapabilitySupport::supported("Win32 extended styles honor topmost windows");
        capabilities.window_transparency = CapabilitySupport::unsupported(
            "Win32 main window transparency is not mapped by the native window host yet",
        );
        capabilities.tray_or_status_menu = CapabilitySupport::partial(
            "Win32 status items can be created by the direct native host; target tray/menu command proof is still pending",
        );
        capabilities.menus = CapabilitySupport::supported(
            "Win32 window menus and HACCEL tables are RAII-owned and route typed Command values",
        );
        capabilities.file_picker = CapabilitySupport::supported(
            "Win32 common open-file dialog is connected through the native host",
        );
        capabilities.clipboard_text = native_text_clipboard_support();
        capabilities.clipboard_image =
            CapabilitySupport::unsupported("the native image clipboard service is not connected");
        capabilities.clipboard_files =
            CapabilitySupport::unsupported("the native file clipboard service is not connected");
        capabilities
    }

    pub fn macos_scaffold() -> Self {
        Self {
            platform: PlatformName::Macos,
            windows: CapabilitySupport::partial(
                "the first-pass Winit window path exists; the AppKit backend is not connected",
            ),
            window_resizing: CapabilitySupport::partial(
                "the Winit path maps basic resize policy; AppKit verification is pending",
            ),
            window_decorations: CapabilitySupport::partial(
                "the Winit path maps native decorations; AppKit verification is pending",
            ),
            window_always_on_top: CapabilitySupport::partial(
                "the Winit path maps floating level; AppKit verification is pending",
            ),
            window_transparency: CapabilitySupport::unsupported(
                "transparent AppKit windows are not connected",
            ),
            tray_or_status_menu: CapabilitySupport::unsupported("NSStatusItem is not connected"),
            menus: CapabilitySupport::unsupported("NSMenu is not connected"),
            global_hotkeys: CapabilitySupport::unsupported(
                "global shortcut service is not wired in ZsuiHost",
            ),
            clipboard_text: CapabilitySupport::partial(
                "text clipboard requires the optional clipboard service; AppKit pasteboard is pending",
            ),
            clipboard_image: CapabilitySupport::unsupported(
                "AppKit image pasteboard support is not connected",
            ),
            clipboard_files: CapabilitySupport::unsupported(
                "AppKit file pasteboard support is not connected",
            ),
            file_picker: CapabilitySupport::unsupported(
                "NSOpenPanel and NSSavePanel are not connected",
            ),
            native_dialogs: CapabilitySupport::unsupported("NSAlert is not connected"),
            settings_pages: CapabilitySupport::partial("settings page specs are declarative"),
            auto_paste: CapabilitySupport::unsupported("auto paste requires accessibility trust"),
        }
    }

    pub fn macos_native_window_host() -> Self {
        let mut capabilities = Self::macos_scaffold();
        if cfg!(feature = "macos-appkit") {
            capabilities.windows = CapabilitySupport::partial(
                "NSApplication/NSWindow lifecycle, draw-plan rendering, typed input, semantic focus rings and resize relayout are connected; target proof is pending",
            );
            capabilities.window_resizing = CapabilitySupport::partial(
                "actual NSView bounds rebuild shared layout, draw plans and input geometry; target resize artifacts are pending",
            );
            capabilities.window_decorations = CapabilitySupport::partial(
                "NSWindow titled and borderless style declarations are connected; target proof is pending",
            );
            capabilities.window_always_on_top = CapabilitySupport::partial(
                "NSFloatingWindowLevel is connected; target interaction proof is pending",
            );
            capabilities.tray_or_status_menu = CapabilitySupport::partial(
                "NSStatusItem and detached NSMenu resources are RAII-owned and command-routed by the AppKit event loop; fixed macOS 15 runtime smoke passed, while manual menu-bar interaction remains a release gate",
            );
        }
        capabilities.clipboard_text = if cfg!(feature = "macos-appkit") {
            CapabilitySupport::partial(
                "NSPasteboard UTF-8 text read/write is connected; AppKit host proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable macos-appkit to compile the native AppKit clipboard service",
            )
        };
        capabilities.menus = if cfg!(feature = "macos-appkit") {
            CapabilitySupport::partial(
                "NSMenu/NSMenuItem installation and typed command polling are connected; AppKit host integration proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable macos-appkit to compile the native AppKit menu service",
            )
        };
        capabilities.file_picker = if cfg!(feature = "macos-appkit") {
            CapabilitySupport::partial(
                "NSOpenPanel and NSSavePanel are connected; target interaction proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable macos-appkit to compile NSOpenPanel and NSSavePanel services",
            )
        };
        capabilities
    }

    pub fn linux_scaffold() -> Self {
        Self {
            platform: PlatformName::Linux,
            windows: CapabilitySupport::partial(
                "the first-pass Winit window path exists; the GTK4 backend is not connected",
            ),
            window_resizing: CapabilitySupport::partial(
                "the Winit path maps basic resize policy; GTK4 verification is pending",
            ),
            window_decorations: CapabilitySupport::partial(
                "the Winit path maps decorations; GTK4 and compositor verification are pending",
            ),
            window_always_on_top: CapabilitySupport::unsupported(
                "GTK4 always-on-top behavior is not connected and differs between Wayland and X11",
            ),
            window_transparency: CapabilitySupport::unsupported(
                "transparent GTK4 windows are not connected",
            ),
            tray_or_status_menu: CapabilitySupport::unsupported(
                "a Linux status-item service is not connected",
            ),
            menus: CapabilitySupport::unsupported("GTK/GIO menus are not connected"),
            global_hotkeys: CapabilitySupport::unsupported(
                "global shortcut support varies by display server",
            ),
            clipboard_text: CapabilitySupport::partial(
                "text clipboard requires the optional clipboard service; GTK clipboard is pending",
            ),
            clipboard_image: CapabilitySupport::unsupported(
                "GTK image clipboard support is not connected",
            ),
            clipboard_files: CapabilitySupport::unsupported(
                "GTK file clipboard support is not connected",
            ),
            file_picker: CapabilitySupport::unsupported("GTK file chooser is not connected"),
            native_dialogs: CapabilitySupport::unsupported("GTK native dialogs are not connected"),
            settings_pages: CapabilitySupport::partial("settings page specs are declarative"),
            auto_paste: CapabilitySupport::partial(
                "xdotool/keytap path is backend and session dependent",
            ),
        }
    }

    pub fn linux_native_window_host() -> Self {
        let mut capabilities = Self::linux_scaffold();
        if cfg!(feature = "linux-direct-host") {
            capabilities.windows = CapabilitySupport::partial(
                "real Wayland/X11 windows, directly presented software rendering, typed input and resize-driven relayout are connected; target proof is pending",
            );
            capabilities.window_resizing = CapabilitySupport::partial(
                "native resize and scale-factor events rebuild shared layout, draw plans and input geometry; Wayland/X11 artifact proof is pending",
            );
            capabilities.window_decorations = CapabilitySupport::partial(
                "server/client compositor decorations and undecorated declarations are connected; target proof is pending",
            );
            capabilities.window_always_on_top = CapabilitySupport::partial(
                "native window-level declarations are connected; Wayland/X11 compositor proof is pending",
            );
            capabilities.window_transparency = CapabilitySupport::partial(
                "transparent surface declarations are connected; Wayland/X11 compositor proof is pending",
            );
        } else if cfg!(feature = "linux-gtk") {
            capabilities.windows = CapabilitySupport::partial(
                "GtkApplication/ApplicationWindow lifecycle, draw-plan rendering, typed input, semantic focus rings and allocation relayout are connected; target proof is pending",
            );
            capabilities.window_resizing = CapabilitySupport::partial(
                "actual DrawingArea allocation rebuilds shared layout, draw plans and input geometry; Wayland/X11 resize artifacts are pending",
            );
            capabilities.window_decorations = CapabilitySupport::partial(
                "GTK4 decorated and undecorated window declarations are connected; compositor proof is pending",
            );
        }
        capabilities.clipboard_text = if cfg!(all(
            feature = "linux-direct-host",
            feature = "clipboard"
        )) {
            CapabilitySupport::partial(
                "system UTF-8 text clipboard access is connected without GTK; Wayland/X11 ownership proof is pending",
            )
        } else if cfg!(feature = "linux-gtk") {
            CapabilitySupport::partial(
                "GdkClipboard UTF-8 text read/write is connected; Wayland/X11 host proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable linux-direct plus clipboard, or linux-gtk, to compile a Linux clipboard service",
            )
        };
        capabilities.menus = if cfg!(feature = "linux-direct-host") {
            CapabilitySupport::supported(
                "the owned desktop menu bar, popup navigation, accelerators and typed command routing are connected on the direct host",
            )
        } else if cfg!(all(
            feature = "linux-gtk",
            not(feature = "linux-direct-host")
        )) {
            CapabilitySupport::partial(
                "GMenu/SimpleAction installation and typed command polling are connected; GTK host integration proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable linux-direct or linux-gtk to compile a Linux desktop menu surface",
            )
        };
        capabilities.file_picker = if cfg!(feature = "linux-direct-host") {
            CapabilitySupport::partial(
                "XDG desktop portal open/save services are connected without GTK; target interaction proof is pending",
            )
        } else if cfg!(feature = "linux-gtk") {
            CapabilitySupport::partial(
                "GTK4 FileChooserNative open/save services are connected; Wayland/X11 interaction proof is pending",
            )
        } else {
            CapabilitySupport::unsupported(
                "enable linux-direct or linux-gtk to compile Linux file dialog services",
            )
        };
        capabilities
    }

    pub fn android_scaffold() -> Self {
        Self {
            platform: PlatformName::Android,
            windows: CapabilitySupport::partial(
                "Android Activity/native surface host is planned; generic PlatformHost records declarations",
            ),
            window_resizing: CapabilitySupport::unsupported(
                "Android phone/tablet surfaces do not map to desktop resize policy",
            ),
            window_decorations: CapabilitySupport::unsupported(
                "Android app chrome is owned by Activity/theme/system bars",
            ),
            window_always_on_top: CapabilitySupport::unsupported(
                "always-on-top requires Android overlay permissions and is not a normal app window",
            ),
            window_transparency: CapabilitySupport::partial(
                "transparent Activity surfaces depend on theme and compositor support",
            ),
            tray_or_status_menu: CapabilitySupport::unsupported(
                "Android has notifications/quick settings instead of a desktop tray",
            ),
            menus: CapabilitySupport::partial(
                "Android menu/action surfaces need a dedicated mobile host",
            ),
            global_hotkeys: CapabilitySupport::unsupported(
                "global shortcuts are not available to normal Android apps",
            ),
            clipboard_text: CapabilitySupport::partial(
                "Android ClipboardManager host is planned",
            ),
            clipboard_image: CapabilitySupport::partial(
                "Android image clipboard depends on URI/content-provider integration",
            ),
            clipboard_files: CapabilitySupport::partial(
                "Android file clipboard depends on content URI integration",
            ),
            file_picker: CapabilitySupport::partial(
                "Android Storage Access Framework host is planned",
            ),
            native_dialogs: CapabilitySupport::partial(
                "Android dialog host is planned",
            ),
            settings_pages: CapabilitySupport::partial(
                "settings page specs can be mapped to Android screens",
            ),
            auto_paste: CapabilitySupport::unsupported(
                "auto paste requires accessibility/input-method integration",
            ),
        }
    }

    pub fn android_native_window_host() -> Self {
        let mut capabilities = Self::android_scaffold();
        capabilities.windows = CapabilitySupport::unsupported(
            "NativeWindowHost does not yet own an Android Activity event loop",
        );
        capabilities
    }
}

fn native_text_clipboard_support() -> CapabilitySupport {
    if cfg!(feature = "clipboard") {
        CapabilitySupport::supported("the optional native text clipboard service is compiled")
    } else {
        CapabilitySupport::unsupported(
            "enable the clipboard feature to compile the native text clipboard service",
        )
    }
}
