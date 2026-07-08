use crate::mobile_host::{
    MobileRuntimeBridgeEntryPoint, MobileRuntimeCapabilityBinding, MobileRuntimeHostScaffold,
    MobileRuntimeLifecycleBinding, MobileRuntimePermission,
};
use crate::{NativeUiAdapterCapability, NativeUiBackendStatus, NativeUiPlatform, NativeUiToolkit};

pub fn android_activity_host_scaffold() -> MobileRuntimeHostScaffold {
    MobileRuntimeHostScaffold {
        platform: NativeUiPlatform::Android,
        platform_name: NativeUiPlatform::Android.platform_name(),
        toolkit: NativeUiToolkit::AndroidActivity,
        toolkit_name: NativeUiToolkit::AndroidActivity.toolkit_name(),
        status: NativeUiBackendStatus::AdapterBoundaryScaffold,
        status_name: NativeUiBackendStatus::AdapterBoundaryScaffold.status_name(),
        module_path: "src/android_activity_host.rs",
        native_library_name: "libzsui_android.so",
        application_manifest_file: "android/app/src/main/AndroidManifest.xml",
        native_window_type: "android.app.Activity surface",
        rust_entry_point: "zsui_android_activity_start",
        bridge_entry_points: android_activity_bridge_entry_points(),
        lifecycle_bindings: android_activity_lifecycle_bindings(),
        capability_bindings: android_activity_capability_bindings(),
        required_permissions: android_activity_required_permissions(),
        target_smoke_requirements: vec![
            "Android emulator or device launch",
            "visible Activity surface screenshot",
            "input method focus check",
            "clipboard service roundtrip",
            "device smoke artifacts",
        ],
        next_implementation_steps: vec![
            "add JNI or ndk glue entry point",
            "map Activity lifecycle to NativeRuntimeDriver",
            "bind Activity window surface to main_window",
            "capture device smoke artifacts with native_smoke_review",
        ],
    }
}

pub fn android_activity_bridge_entry_points() -> Vec<MobileRuntimeBridgeEntryPoint> {
    vec![
        MobileRuntimeBridgeEntryPoint::new(
            "zsui_android_activity_start",
            "Rust FFI",
            "src/android_activity_host.rs",
            "start the ZSUI runtime from the Activity bridge",
        ),
        MobileRuntimeBridgeEntryPoint::new(
            "ZsuiActivity.onCreate",
            "Kotlin or Java",
            "android/app/src/main/java/.../ZsuiActivity.kt",
            "create the native Activity surface and call into Rust",
        ),
        MobileRuntimeBridgeEntryPoint::new(
            "ZsuiActivity.onDestroy",
            "Kotlin or Java",
            "android/app/src/main/java/.../ZsuiActivity.kt",
            "request framework shutdown before Activity teardown",
        ),
    ]
}

pub fn android_activity_lifecycle_bindings() -> Vec<MobileRuntimeLifecycleBinding> {
    vec![
        MobileRuntimeLifecycleBinding::new("onCreate", "start_native_runtime", true),
        MobileRuntimeLifecycleBinding::new("onResume", "poll_native_event", true),
        MobileRuntimeLifecycleBinding::new("onPause", "poll_product_event", true),
        MobileRuntimeLifecycleBinding::new("onDestroy", "request_shutdown", true),
    ]
}

pub fn android_activity_capability_bindings() -> Vec<MobileRuntimeCapabilityBinding> {
    use NativeUiAdapterCapability::{
        Clipboard, EditDialog, FileDialog, Ime, InputDialog, MainExecutionPlanBridge,
        MainSearchControl, MainWindow, PasteTarget, PopupMenu, Renderer, SettingsDropdown,
        SettingsWindow, ShellOpen, StatusItem, TextLayout, TransientWindow, WindowIdentity,
    };

    vec![
        MobileRuntimeCapabilityBinding::new(
            MainWindow,
            "android.app.Activity surface",
            "NativeMainWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            SettingsWindow,
            "androidx.fragment.app.DialogFragment",
            "NativeSettingsWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            SettingsDropdown,
            "android.widget.PopupWindow or Spinner",
            "NativeSettingsDropdownHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            InputDialog,
            "android.app.AlertDialog with EditText",
            "NativeTextInputDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            EditDialog,
            "Activity text editor surface",
            "NativeEditTextDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            Clipboard,
            "android.content.ClipboardManager",
            "ClipboardHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            PopupMenu,
            "android.widget.PopupMenu",
            "NativePopupMenuHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            StatusItem,
            "android.app.Notification",
            "status item bridge",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            Renderer,
            "android.graphics.Canvas or Compose node",
            "Renderer",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            TextLayout,
            "android.text.StaticLayout",
            "TextLayout",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            MainSearchControl,
            "android.widget.SearchView",
            "NativeMainSearchControlHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            TransientWindow,
            "android.widget.PopupWindow",
            "NativeTransientWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            Ime,
            "android.view.inputmethod.InputMethodManager",
            "NativeImeHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            ShellOpen,
            "android.content.Intent",
            "NativeShellOpenHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            FileDialog,
            "Android Storage Access Framework",
            "NativeFileDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            PasteTarget,
            "Accessibility or focused view paste target",
            "NativePasteTargetHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            WindowIdentity,
            "Activity task and window token",
            "NativeWindowIdentityHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            MainExecutionPlanBridge,
            "Activity-to-Rust command bridge",
            "NativeRuntimeDriver",
            false,
        ),
    ]
}

pub fn android_activity_required_permissions() -> Vec<MobileRuntimePermission> {
    vec![
        MobileRuntimePermission::new(
            "android.permission.POST_NOTIFICATIONS",
            "status_item",
            false,
        ),
        MobileRuntimePermission::new(
            "android.permission.SYSTEM_ALERT_WINDOW",
            "transient_window",
            false,
        ),
    ]
}
