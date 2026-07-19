use crate::platform_identity::NativeUiToolkit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeHostProfile {
    Win32,
    AppKit,
    LinuxDirect,
    AndroidActivity,
    HarmonyAbility,
}

impl NativeHostProfile {
    pub(crate) const fn module_path(self) -> &'static str {
        match self {
            Self::Win32 => "src/platform/windows/mod.rs",
            Self::AppKit => "src/macos_appkit_services.rs",
            Self::LinuxDirect => "src/linux_direct.rs",
            Self::AndroidActivity => "src/android_activity_host.rs",
            Self::HarmonyAbility => "src/harmony_ability_host.rs",
        }
    }

    pub(crate) const fn native_application_type(self) -> &'static str {
        match self {
            Self::Win32 => "Win32 message loop with GDI no-flicker paint on Windows",
            Self::AppKit => "NSApplication event loop on macOS",
            Self::LinuxDirect => "Wayland/X11 native event loop on Linux",
            Self::AndroidActivity => "Android Activity host",
            Self::HarmonyAbility => "Harmony Ability host",
        }
    }

    pub(crate) const fn native_window_type(self) -> &'static str {
        match self {
            Self::Win32 => "Win32 HWND main/quick windows",
            Self::AppKit => "AppKit NSWindow",
            Self::LinuxDirect => "Wayland/X11 native window with directly presented surface",
            Self::AndroidActivity => "android.app.Activity surface",
            Self::HarmonyAbility => "OpenHarmony Ability window",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeTextProfile {
    Uniscribe,
    CoreText,
    Pango,
    #[cfg(any(
        test,
        all(
            target_os = "linux",
            feature = "linux-direct-lite",
            not(feature = "linux-direct")
        )
    ))]
    CosmicText,
    AndroidText,
    HarmonyText,
}

impl NativeTextProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Uniscribe => "uniscribe",
            Self::CoreText => "core_text",
            Self::Pango => "pango",
            #[cfg(any(
                test,
                all(
                    target_os = "linux",
                    feature = "linux-direct-lite",
                    not(feature = "linux-direct")
                )
            ))]
            Self::CosmicText => "cosmic_text",
            Self::AndroidText => "android_text",
            Self::HarmonyText => "harmony_text",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeRasterProfile {
    GdiPlus,
    CoreGraphics,
    Cairo,
    #[cfg(any(
        test,
        all(
            target_os = "linux",
            feature = "linux-direct-lite",
            not(feature = "linux-direct")
        )
    ))]
    TinySkia,
    AndroidCanvas,
    HarmonyDrawing,
}

impl NativeRasterProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::GdiPlus => "gdi_plus",
            Self::CoreGraphics => "core_graphics",
            Self::Cairo => "cairo",
            #[cfg(any(
                test,
                all(
                    target_os = "linux",
                    feature = "linux-direct-lite",
                    not(feature = "linux-direct")
                )
            ))]
            Self::TinySkia => "tiny_skia",
            Self::AndroidCanvas => "android_canvas",
            Self::HarmonyDrawing => "harmony_drawing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativePresenterProfile {
    BufferedDib,
    AppKitView,
    Softbuffer,
    AndroidSurface,
    HarmonySurface,
}

impl NativePresenterProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::BufferedDib => "buffered_dib",
            Self::AppKitView => "appkit_view",
            Self::Softbuffer => "softbuffer",
            Self::AndroidSurface => "android_surface",
            Self::HarmonySurface => "harmony_surface",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeServicesProfile {
    Win32,
    AppKit,
    XdgDesktop,
    Android,
    Harmony,
}

impl NativeServicesProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Win32 => "win32",
            Self::AppKit => "appkit",
            Self::XdgDesktop => "xdg_desktop",
            Self::Android => "android",
            Self::Harmony => "harmony",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackendProfile {
    toolkit: NativeUiToolkit,
    host: NativeHostProfile,
    text: NativeTextProfile,
    raster: NativeRasterProfile,
    presenter: NativePresenterProfile,
    services: NativeServicesProfile,
    real_runtime: bool,
}

impl BackendProfile {
    pub(crate) const fn windows() -> Self {
        Self {
            toolkit: NativeUiToolkit::Win32Gdi,
            host: NativeHostProfile::Win32,
            text: NativeTextProfile::Uniscribe,
            raster: NativeRasterProfile::GdiPlus,
            presenter: NativePresenterProfile::BufferedDib,
            services: NativeServicesProfile::Win32,
            real_runtime: true,
        }
    }

    pub(crate) const fn macos() -> Self {
        Self {
            toolkit: NativeUiToolkit::AppKit,
            host: NativeHostProfile::AppKit,
            text: NativeTextProfile::CoreText,
            raster: NativeRasterProfile::CoreGraphics,
            presenter: NativePresenterProfile::AppKitView,
            services: NativeServicesProfile::AppKit,
            real_runtime: true,
        }
    }

    pub(crate) const fn linux() -> Self {
        Self {
            toolkit: NativeUiToolkit::LinuxDirect,
            host: NativeHostProfile::LinuxDirect,
            text: NativeTextProfile::Pango,
            raster: NativeRasterProfile::Cairo,
            presenter: NativePresenterProfile::Softbuffer,
            services: NativeServicesProfile::XdgDesktop,
            real_runtime: true,
        }
    }

    #[cfg(any(
        test,
        all(
            target_os = "linux",
            feature = "linux-direct-lite",
            not(feature = "linux-direct")
        )
    ))]
    pub(crate) const fn linux_lite() -> Self {
        Self {
            text: NativeTextProfile::CosmicText,
            raster: NativeRasterProfile::TinySkia,
            ..Self::linux()
        }
    }

    pub(crate) const fn android() -> Self {
        Self {
            toolkit: NativeUiToolkit::AndroidActivity,
            host: NativeHostProfile::AndroidActivity,
            text: NativeTextProfile::AndroidText,
            raster: NativeRasterProfile::AndroidCanvas,
            presenter: NativePresenterProfile::AndroidSurface,
            services: NativeServicesProfile::Android,
            real_runtime: false,
        }
    }

    pub(crate) const fn harmony() -> Self {
        Self {
            toolkit: NativeUiToolkit::HarmonyAbility,
            host: NativeHostProfile::HarmonyAbility,
            text: NativeTextProfile::HarmonyText,
            raster: NativeRasterProfile::HarmonyDrawing,
            presenter: NativePresenterProfile::HarmonySurface,
            services: NativeServicesProfile::Harmony,
            real_runtime: false,
        }
    }

    pub(crate) const fn toolkit(self) -> NativeUiToolkit {
        self.toolkit
    }

    pub(crate) const fn host(self) -> NativeHostProfile {
        self.host
    }

    pub(crate) const fn text(self) -> NativeTextProfile {
        self.text
    }

    pub(crate) const fn raster(self) -> NativeRasterProfile {
        self.raster
    }

    pub(crate) const fn presenter(self) -> NativePresenterProfile {
        self.presenter
    }

    pub(crate) const fn services(self) -> NativeServicesProfile {
        self.services
    }

    pub(crate) const fn has_real_runtime(self) -> bool {
        self.real_runtime
    }
}
