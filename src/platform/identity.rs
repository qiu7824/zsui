use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiPlatform {
    Windows,
    Macos,
    Linux,
    Android,
}

impl NativeUiPlatform {
    pub const fn platform_name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
        }
    }

    pub const fn current_target() -> Option<Self> {
        #[cfg(target_os = "windows")]
        {
            return Some(Self::Windows);
        }
        #[cfg(target_os = "macos")]
        {
            return Some(Self::Macos);
        }
        #[cfg(target_os = "android")]
        {
            return Some(Self::Android);
        }
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            return Some(Self::Linux);
        }
        #[allow(unreachable_code)]
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiToolkit {
    WinitDesktop,
    Win32Gdi,
    AppKit,
    LinuxDirect,
    Gtk4Libadwaita,
    AndroidActivity,
}

impl NativeUiToolkit {
    pub const fn toolkit_name(self) -> &'static str {
        match self {
            Self::WinitDesktop => "winit_desktop",
            Self::Win32Gdi => "win32_gdi",
            Self::AppKit => "appkit",
            Self::LinuxDirect => "linux_direct",
            Self::Gtk4Libadwaita => "gtk4_libadwaita",
            Self::AndroidActivity => "android_activity",
        }
    }
}

pub const SUPPORTED_NATIVE_UI_PLATFORMS: [NativeUiPlatform; 4] = [
    NativeUiPlatform::Windows,
    NativeUiPlatform::Macos,
    NativeUiPlatform::Linux,
    NativeUiPlatform::Android,
];

pub const SUPPORTED_NATIVE_UI_TOOLKITS: [NativeUiToolkit; 5] = [
    NativeUiToolkit::Win32Gdi,
    NativeUiToolkit::AppKit,
    NativeUiToolkit::LinuxDirect,
    NativeUiToolkit::Gtk4Libadwaita,
    NativeUiToolkit::AndroidActivity,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUiBackendStatus {
    NativeHostIntegrated,
    NativeHostFirstPass,
    AdapterBoundaryScaffold,
}

impl NativeUiBackendStatus {
    pub const fn status_name(self) -> &'static str {
        match self {
            Self::NativeHostIntegrated => "native_host_integrated",
            Self::NativeHostFirstPass => "native_host_first_pass",
            Self::AdapterBoundaryScaffold => "adapter_boundary_scaffold",
        }
    }

    pub const fn is_native_runtime_ready(self) -> bool {
        matches!(self, Self::NativeHostIntegrated)
    }

    pub const fn is_scaffold(self) -> bool {
        matches!(self, Self::AdapterBoundaryScaffold)
    }

    pub const fn is_first_pass_native_host(self) -> bool {
        matches!(self, Self::NativeHostFirstPass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NativeUiBackendDescriptor {
    pub platform: NativeUiPlatform,
    pub toolkit: NativeUiToolkit,
    pub status: NativeUiBackendStatus,
    pub adapter_boundary: &'static str,
    pub module_path: &'static str,
}

impl NativeUiBackendDescriptor {
    pub const fn platform_name(&self) -> &'static str {
        self.platform.platform_name()
    }

    pub const fn toolkit_name(&self) -> &'static str {
        self.toolkit.toolkit_name()
    }

    pub const fn status_name(&self) -> &'static str {
        self.status.status_name()
    }
}
