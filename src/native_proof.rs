use serde::Serialize;

use crate::{
    NativeTypographyProfile, NativeWindowSmokeRunReport, ViewHitTarget, ViewHitTargetKind,
    ZsTypographyPlatformStyle,
};

pub const NATIVE_PROOF_SCHEMA: &str = "zsui.native-proof/v1";
pub const NATIVE_PROOF_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeProofRunnerEvidence {
    pub image_os: Option<String>,
    pub image_version: Option<String>,
}

impl NativeProofRunnerEvidence {
    pub fn from_environment() -> Self {
        Self {
            image_os: std::env::var("ImageOS").ok(),
            image_version: std::env::var("ImageVersion").ok(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct NativeProofWindowEvidence {
    pub width: u32,
    pub height: u32,
    pub logical_width: u32,
    pub logical_height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeProofWidgetEvidence {
    pub id: String,
    pub role: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeProofProcessMemoryEvidence {
    pub source: &'static str,
    pub sample_point: &'static str,
    pub resident_bytes: u64,
    pub peak_resident_bytes: u64,
    pub private_bytes: Option<u64>,
    pub peak_private_bytes: Option<u64>,
    pub proportional_set_size_bytes: Option<u64>,
    pub virtual_bytes: Option<u64>,
}

impl NativeProofProcessMemoryEvidence {
    pub fn capture() -> Option<Self> {
        Self::capture_at("proof_document_serialization")
    }

    pub(crate) fn capture_at(sample_point: &'static str) -> Option<Self> {
        #[cfg(not(any(
            all(target_os = "windows", feature = "windows-gdi"),
            all(target_os = "macos", feature = "macos-appkit"),
            all(target_os = "linux", not(target_env = "ohos"))
        )))]
        let _ = sample_point;
        #[cfg(all(target_os = "windows", feature = "windows-gdi"))]
        {
            return capture_windows_process_memory(sample_point);
        }
        #[cfg(all(target_os = "macos", feature = "macos-appkit"))]
        {
            return capture_macos_process_memory(sample_point);
        }
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            return capture_linux_process_memory(sample_point);
        }
        #[allow(unreachable_code)]
        None
    }
}

impl NativeProofWidgetEvidence {
    fn from_hit_target(
        target: ViewHitTarget,
        focused_widget: Option<u64>,
        content_offset_y: i32,
    ) -> Self {
        Self {
            id: native_proof_widget_id(target.widget.0),
            role: native_proof_role(target.kind),
            x: target.bounds.x,
            y: target.bounds.y.saturating_add(content_offset_y),
            width: target.bounds.width,
            height: target.bounds.height,
            enabled: true,
            focused: focused_widget == Some(target.widget.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NativeProofDocument {
    pub schema: &'static str,
    pub schema_version: u32,
    pub application: String,
    pub scenario: String,
    pub theme: String,
    pub platform: String,
    pub backend: String,
    pub capture_backend: String,
    pub display_server: Option<String>,
    pub os_family: String,
    pub architecture: String,
    pub runner: NativeProofRunnerEvidence,
    pub scale_factor: f64,
    pub typography_scale: f32,
    pub typography: NativeTypographyProfile,
    pub process_memory: Option<NativeProofProcessMemoryEvidence>,
    pub window: NativeProofWindowEvidence,
    pub focused_widget: Option<String>,
    pub widgets: Vec<NativeProofWidgetEvidence>,
    pub messages: Vec<String>,
    pub unhandled_commands: Vec<String>,
    pub errors: Vec<String>,
    pub runtime: NativeWindowSmokeRunReport,
}

impl NativeProofDocument {
    pub fn new(
        application: impl Into<String>,
        scenario: impl Into<String>,
        theme: impl Into<String>,
        requested_width: u32,
        requested_height: u32,
        widgets: impl IntoIterator<Item = ViewHitTarget>,
        runtime: NativeWindowSmokeRunReport,
    ) -> Self {
        let capture = runtime.native_view_capture.as_ref();
        let platform = capture
            .map(|capture| capture.platform)
            .unwrap_or(std::env::consts::OS)
            .to_string();
        let capture_backend = capture
            .map(|capture| capture.backend)
            .unwrap_or("unavailable")
            .to_string();
        let logical_width = capture
            .map(|capture| capture.logical_width)
            .unwrap_or(requested_width);
        let logical_height = capture
            .map(|capture| capture.logical_height)
            .unwrap_or(requested_height);
        let scale_factor = capture.map(|capture| capture.scale_factor).unwrap_or(1.0);
        let typography_scale = capture
            .map(|capture| capture.typography_scale)
            .unwrap_or(1.0);
        let typography = native_proof_typography(capture, &platform, typography_scale);
        let focused_widget = runtime.native_view_focused_widget;
        let content_offset_y = runtime.window_menu_surface_height.min(i32::MAX as u32) as i32;
        let errors = native_proof_errors(&runtime);
        let unhandled_commands = native_proof_unhandled_commands(&runtime);

        Self {
            schema: NATIVE_PROOF_SCHEMA,
            schema_version: NATIVE_PROOF_SCHEMA_VERSION,
            application: application.into(),
            scenario: scenario.into(),
            theme: theme.into(),
            backend: native_proof_backend(&capture_backend, &platform).to_string(),
            capture_backend,
            display_server: capture
                .and_then(|capture| capture.display_server)
                .map(str::to_string),
            os_family: std::env::var("ZSUI_NATIVE_PROOF_OS_FAMILY")
                .unwrap_or_else(|_| platform.clone()),
            architecture: native_proof_architecture(std::env::consts::ARCH).to_string(),
            runner: NativeProofRunnerEvidence::from_environment(),
            scale_factor,
            typography_scale,
            typography,
            process_memory: runtime
                .process_memory_during_runtime
                .clone()
                .or_else(NativeProofProcessMemoryEvidence::capture),
            window: NativeProofWindowEvidence {
                width: logical_width,
                height: logical_height,
                logical_width,
                logical_height,
                pixel_width: capture
                    .map(|capture| capture.pixel_width)
                    .unwrap_or(logical_width),
                pixel_height: capture
                    .map(|capture| capture.pixel_height)
                    .unwrap_or(logical_height),
            },
            focused_widget: focused_widget.map(native_proof_widget_id),
            widgets: widgets
                .into_iter()
                .map(|target| {
                    NativeProofWidgetEvidence::from_hit_target(
                        target,
                        focused_widget,
                        content_offset_y,
                    )
                })
                .collect(),
            messages: Vec::new(),
            unhandled_commands,
            errors,
            platform,
            runtime,
        }
    }

    pub fn os_family(mut self, os_family: impl Into<String>) -> Self {
        self.os_family = os_family.into();
        self
    }

    pub fn runner(mut self, runner: NativeProofRunnerEvidence) -> Self {
        self.runner = runner;
        self
    }

    pub fn messages(mut self, messages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.messages = messages.into_iter().map(Into::into).collect();
        self
    }
}

fn native_proof_typography(
    capture: Option<&crate::NativeViewCaptureEvidence>,
    platform: &str,
    typography_scale: f32,
) -> NativeTypographyProfile {
    if let Some(capture) = capture {
        return capture.typography.clone();
    }
    #[cfg(all(target_os = "windows", feature = "windows-gdi"))]
    if platform == "windows" {
        return crate::windows_gdi_renderer::windows_native_typography_profile()
            .with_typography_scale(typography_scale);
    }
    NativeTypographyProfile::fallback(
        match platform {
            "macos" => ZsTypographyPlatformStyle::Macos,
            "linux" => ZsTypographyPlatformStyle::Gtk,
            _ => ZsTypographyPlatformStyle::Windows,
        },
        typography_scale,
    )
}

fn native_proof_widget_id(widget: u64) -> String {
    format!("widget-{widget}")
}

fn native_proof_role(kind: ViewHitTargetKind) -> String {
    let variant = match serde_json::to_value(kind) {
        Ok(serde_json::Value::String(variant)) => variant,
        Ok(serde_json::Value::Object(fields)) => fields
            .into_iter()
            .next()
            .map(|(variant, _)| variant)
            .unwrap_or_else(|| "Unknown".to_string()),
        _ => "Unknown".to_string(),
    };
    let mut role = String::with_capacity(variant.len() + 4);
    for (index, character) in variant.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                role.push('_');
            }
            role.push(character.to_ascii_lowercase());
        } else {
            role.push(character);
        }
    }
    role
}

fn native_proof_backend(capture_backend: &str, platform: &str) -> &'static str {
    if capture_backend.starts_with("appkit_") {
        "appkit"
    } else if capture_backend.starts_with("winit_softbuffer_") {
        "linux-direct"
    } else if capture_backend.starts_with("gtk_") {
        "gtk4"
    } else if capture_backend.starts_with("win32_") || capture_backend.starts_with("windows_") {
        "win32"
    } else {
        match platform {
            "macos" => "appkit",
            "linux" if cfg!(feature = "linux-direct-host") => "linux-direct",
            "linux" => "gtk4",
            "windows" => "win32",
            _ => "unknown",
        }
    }
}

fn native_proof_architecture(architecture: &str) -> &str {
    match architecture {
        "aarch64" => "arm64",
        value => value,
    }
}

#[cfg(all(target_os = "windows", feature = "windows-gdi"))]
fn capture_windows_process_memory(
    sample_point: &'static str,
) -> Option<NativeProofProcessMemoryEvidence> {
    use windows_sys::Win32::System::{
        ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
        },
        Threading::GetCurrentProcess,
    };

    let mut counters = unsafe { std::mem::zeroed::<PROCESS_MEMORY_COUNTERS_EX>() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let captured = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            counters.cb,
        )
    };
    (captured != 0).then(|| NativeProofProcessMemoryEvidence {
        source: "win32_get_process_memory_info",
        sample_point,
        resident_bytes: counters.WorkingSetSize as u64,
        peak_resident_bytes: counters.PeakWorkingSetSize as u64,
        private_bytes: Some(counters.PrivateUsage as u64),
        peak_private_bytes: Some(counters.PeakPagefileUsage as u64),
        proportional_set_size_bytes: None,
        virtual_bytes: None,
    })
}

#[cfg(all(target_os = "macos", feature = "macos-appkit"))]
#[allow(deprecated)]
fn capture_macos_process_memory(
    sample_point: &'static str,
) -> Option<NativeProofProcessMemoryEvidence> {
    let mut info = unsafe { std::mem::zeroed::<libc::mach_task_basic_info>() };
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    let captured = unsafe {
        libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO as libc::task_flavor_t,
            (&mut info as *mut libc::mach_task_basic_info).cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if captured != libc::KERN_SUCCESS {
        return None;
    }
    let resident_bytes = unsafe { std::ptr::addr_of!(info.resident_size).read_unaligned() };
    let peak_resident_bytes =
        unsafe { std::ptr::addr_of!(info.resident_size_max).read_unaligned() };
    let virtual_bytes = unsafe { std::ptr::addr_of!(info.virtual_size).read_unaligned() };
    Some(NativeProofProcessMemoryEvidence {
        source: "macos_mach_task_basic_info",
        sample_point,
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_bytes: None,
        peak_private_bytes: None,
        proportional_set_size_bytes: None,
        virtual_bytes: Some(virtual_bytes),
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn capture_linux_process_memory(
    sample_point: &'static str,
) -> Option<NativeProofProcessMemoryEvidence> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let resident_bytes = linux_proc_kib(&status, "VmRSS:")?;
    let peak_resident_bytes = linux_proc_kib(&status, "VmHWM:").unwrap_or(resident_bytes);
    let virtual_bytes = linux_proc_kib(&status, "VmSize:");
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").ok();
    let private_bytes = rollup.as_deref().map(|rollup| {
        ["Private_Clean:", "Private_Dirty:", "Private_Hugetlb:"]
            .into_iter()
            .filter_map(|key| linux_proc_kib(&rollup, key))
            .sum::<u64>()
    });
    let proportional_set_size_bytes = rollup
        .as_deref()
        .and_then(|rollup| linux_proc_kib(rollup, "Pss:"));
    Some(NativeProofProcessMemoryEvidence {
        source: "linux_procfs_status_smaps_rollup",
        sample_point,
        resident_bytes,
        peak_resident_bytes: peak_resident_bytes.max(resident_bytes),
        private_bytes,
        peak_private_bytes: None,
        proportional_set_size_bytes,
        virtual_bytes,
    })
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn linux_proc_kib(contents: &str, key: &str) -> Option<u64> {
    let line = contents.lines().find(|line| line.starts_with(key))?;
    let kib = line[key.len()..]
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    kib.checked_mul(1_024)
}

fn native_proof_errors(runtime: &NativeWindowSmokeRunReport) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, error) in [
        ("startup", runtime.startup_error.as_deref()),
        ("screenshot", runtime.screenshot_error.as_deref()),
        ("window_menu", runtime.window_menu_command_error.as_deref()),
        ("status_item", runtime.status_item_error.as_deref()),
        ("status_menu", runtime.status_menu_command_error.as_deref()),
        (
            "status_menu_popup",
            runtime.status_menu_popup_error.as_deref(),
        ),
    ] {
        if let Some(error) = error {
            errors.push(format!("{name}: {error}"));
        }
    }
    errors.extend(
        runtime
            .native_view_ui_command_errors
            .iter()
            .map(|error| format!("ui_command: {error}")),
    );
    errors.extend(
        runtime
            .native_view_app_command_errors
            .iter()
            .map(|error| format!("app_command: {error}")),
    );
    errors.extend(
        runtime
            .native_view_text_edit_command_errors
            .iter()
            .map(|error| format!("text_edit: {error}")),
    );
    errors
}

fn native_proof_unhandled_commands(runtime: &NativeWindowSmokeRunReport) -> Vec<String> {
    let mut unhandled = Vec::new();
    for (name, count) in [
        (
            "native_view_ui_command",
            runtime.native_view_ui_command_unhandled_count,
        ),
        (
            "native_view_app_command",
            runtime.native_view_app_command_unhandled_count,
        ),
        (
            "native_view_click",
            runtime.native_view_unhandled_click_count,
        ),
        ("native_view_key", runtime.native_view_unhandled_key_count),
        (
            "native_view_scroll",
            runtime.native_view_unhandled_scroll_count,
        ),
    ] {
        if count > 0 {
            unhandled.push(format!("{name}:{count}"));
        }
    }
    unhandled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Dp, NativeViewCaptureEvidence, NativeWindowSmokeRunOptions, Rect, ViewHitTarget,
        ViewHitTargetKind, WidgetId,
    };

    #[test]
    fn proof_document_flattens_native_capture_focus_and_widget_roles() {
        let mut runtime = NativeWindowSmokeRunReport::empty(NativeWindowSmokeRunOptions::quick());
        runtime.native_view_capture = Some(NativeViewCaptureEvidence {
            platform: "macos",
            backend: "appkit_nsview_bitmap_cache",
            display_server: None,
            logical_width: 960,
            logical_height: 640,
            pixel_width: 1920,
            pixel_height: 1280,
            scale_factor: 2.0,
            typography_scale: 1.0,
            typography: NativeTypographyProfile::fallback(ZsTypographyPlatformStyle::Macos, 1.0),
        });
        runtime.native_view_focused_widget = Some(7);

        let document = NativeProofDocument::new(
            "gallery",
            "inputs-light",
            "light",
            960,
            640,
            [ViewHitTarget::with_kind(
                WidgetId::new(7),
                Rect {
                    x: 12,
                    y: 20,
                    width: 200,
                    height: 28,
                },
                ViewHitTargetKind::Textbox,
            )],
            runtime,
        )
        .os_family("macos-15")
        .messages(["SearchFocused", "SearchChanged"]);

        assert_eq!(document.backend, "appkit");
        assert_eq!(document.capture_backend, "appkit_nsview_bitmap_cache");
        assert_eq!(
            document.architecture,
            native_proof_architecture(std::env::consts::ARCH)
        );
        assert_eq!(document.scale_factor, 2.0);
        assert_eq!(
            document.typography.platform,
            ZsTypographyPlatformStyle::Macos
        );
        assert_eq!(document.typography.body_metrics.size, 13.0);
        assert_eq!(document.window.pixel_width, 1920);
        assert_eq!(document.focused_widget.as_deref(), Some("widget-7"));
        assert_eq!(document.widgets[0].role, "textbox");
        assert!(document.widgets[0].focused);
        assert_eq!(document.messages, ["SearchFocused", "SearchChanged"]);
        assert!(document.errors.is_empty());
        assert!(document.unhandled_commands.is_empty());
    }

    #[test]
    fn proof_document_surfaces_errors_and_unhandled_counts() {
        let mut runtime = NativeWindowSmokeRunReport::empty(NativeWindowSmokeRunOptions::quick());
        runtime.screenshot_error = Some("capture failed".to_string());
        runtime.native_view_unhandled_key_count = 2;
        runtime
            .native_view_text_edit_command_errors
            .push("bad edit".to_string());

        let document =
            NativeProofDocument::new("notepad", "interaction", "system", 960, 640, [], runtime);

        assert_eq!(
            document.errors,
            ["screenshot: capture failed", "text_edit: bad edit"]
        );
        assert_eq!(document.unhandled_commands, ["native_view_key:2"]);
    }

    #[test]
    fn proof_window_falls_back_to_requested_geometry_without_capture() {
        let runtime = NativeWindowSmokeRunReport::empty(NativeWindowSmokeRunOptions::new(10));
        let document =
            NativeProofDocument::new("gallery", "catalog", "light", 1024, 640, [], runtime);

        assert_eq!(document.window.width, 1024);
        assert_eq!(document.window.height, 640);
        assert_eq!(
            document.backend,
            native_proof_backend("unavailable", std::env::consts::OS)
        );
        assert_eq!(document.scale_factor, 1.0);
        assert_eq!(Dp::new(document.typography_scale), Dp::new(1.0));
    }

    #[cfg(any(
        all(target_os = "windows", feature = "windows-gdi"),
        all(target_os = "macos", feature = "macos-appkit"),
        all(target_os = "linux", not(target_env = "ohos"))
    ))]
    #[test]
    fn process_memory_capture_reports_real_resident_bytes() {
        let memory = NativeProofProcessMemoryEvidence::capture()
            .expect("supported desktop target should expose process memory");
        assert!(memory.resident_bytes > 0);
        assert!(memory.peak_resident_bytes >= memory.resident_bytes);
    }
}
