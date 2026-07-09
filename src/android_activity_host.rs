use crate::mobile_host::{
    MobileRuntimeBridgeCallback, MobileRuntimeBridgeCallbackKind, MobileRuntimeBridgeContract,
    MobileRuntimeBridgeEntryPoint, MobileRuntimeCapabilityBinding,
    MobileRuntimeDeviceSmokeArtifact, MobileRuntimeHostScaffold, MobileRuntimeLifecycleBinding,
    MobileRuntimePermission,
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
        bridge_contract: android_activity_bridge_contract(),
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
            "capture device smoke artifacts and review them with mobile_scaffold_manifest --review android",
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

pub fn android_activity_bridge_contract() -> MobileRuntimeBridgeContract {
    MobileRuntimeBridgeContract {
        platform: NativeUiPlatform::Android,
        platform_name: NativeUiPlatform::Android.platform_name(),
        toolkit: NativeUiToolkit::AndroidActivity,
        toolkit_name: NativeUiToolkit::AndroidActivity.toolkit_name(),
        module_path: "src/android_activity_host.rs",
        native_library_name: "libzsui_android.so",
        rust_entry_point: "zsui_android_activity_start",
        foreign_language: "Kotlin or Java",
        foreign_entry_file: "android/app/src/main/java/.../ZsuiActivity.kt",
        callbacks: android_activity_bridge_callbacks(),
        device_smoke_artifacts: android_activity_device_smoke_artifacts(),
        safety_rules: vec![
            "keep JNI/NDK raw handles inside the Android host module",
            "return Result-style error codes across the FFI boundary; do not panic through FFI",
            "map Activity lifecycle callbacks to NativeRuntimeDriver stages before creating product behavior",
            "record lifecycle, surface, input and clipboard artifacts on a real emulator or device before completion claims",
        ],
    }
}

pub fn android_activity_bridge_callbacks() -> Vec<MobileRuntimeBridgeCallback> {
    use MobileRuntimeBridgeCallbackKind::{
        Bootstrap, Command, EventPoll, Input, Lifecycle, Shutdown, Surface,
    };

    vec![
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.nativeStart",
            "zsui_android_activity_start",
            Bootstrap,
            "Activity handle, saved state pointer and NativeRuntimeStartupRequest",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.nativeLifecycle",
            "zsui_android_activity_lifecycle",
            Lifecycle,
            "Android lifecycle stage name: create/start/resume/pause/stop/destroy",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "SurfaceHolder.Callback.surfaceCreated",
            "zsui_android_activity_surface_created",
            Surface,
            "Android Surface handle and current Dpi",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "SurfaceHolder.Callback.surfaceChanged",
            "zsui_android_activity_surface_resized",
            Surface,
            "surface width, height and Dpi",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "SurfaceHolder.Callback.surfaceDestroyed",
            "zsui_android_activity_surface_destroyed",
            Surface,
            "surface identity",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.dispatchTouchEvent",
            "zsui_android_activity_dispatch_ui_event",
            Input,
            "typed touch/key/input event translated to UiEvent",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.nativeDispatchCommand",
            "zsui_android_activity_dispatch_command",
            Command,
            "UiCommand id and payload from platform controls",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.nativePollAppEvent",
            "zsui_android_activity_poll_app_event",
            EventPoll,
            "next AppEvent from NativeRuntimeDriver",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiActivity.onDestroy",
            "zsui_android_activity_shutdown",
            Shutdown,
            "shutdown reason and Activity identity",
            true,
        ),
    ]
}

pub fn android_activity_lifecycle_bindings() -> Vec<MobileRuntimeLifecycleBinding> {
    vec![
        MobileRuntimeLifecycleBinding::new("onCreate", "start_native_runtime", true),
        MobileRuntimeLifecycleBinding::new("onStart", "create_main_surface", true),
        MobileRuntimeLifecycleBinding::new("onResume", "poll_native_event", true),
        MobileRuntimeLifecycleBinding::new("surfaceCreated", "bind_render_surface", true),
        MobileRuntimeLifecycleBinding::new("surfaceChanged", "resize_render_surface", true),
        MobileRuntimeLifecycleBinding::new("surfaceDestroyed", "release_render_surface", true),
        MobileRuntimeLifecycleBinding::new("onPause", "poll_product_event", true),
        MobileRuntimeLifecycleBinding::new("onStop", "suspend_native_runtime", true),
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

pub fn android_activity_device_smoke_artifacts() -> Vec<MobileRuntimeDeviceSmokeArtifact> {
    vec![
        MobileRuntimeDeviceSmokeArtifact::new(
            "mobile_manifest",
            "manifest.json",
            true,
            "serialized Android mobile bridge manifest",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "device_launch_log",
            "device-launch.log",
            true,
            "adb or emulator launch output and exit status",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "device_window_screenshot",
            "device-window.png",
            true,
            "screenshot proving the Activity surface is visible",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "lifecycle_trace",
            "lifecycle.json",
            true,
            "ordered Activity lifecycle callbacks observed by the Rust bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "surface_trace",
            "surface.json",
            true,
            "surface create/resize/destroy callbacks observed by the Rust bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "input_trace",
            "input.json",
            true,
            "touch/key/input event dispatch observed by the typed UI event bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "clipboard_trace",
            "clipboard.json",
            false,
            "ClipboardManager roundtrip proof when clipboard capability is enabled",
        ),
    ]
}
