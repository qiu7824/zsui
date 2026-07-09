use crate::mobile_host::{
    MobileRuntimeBridgeCallback, MobileRuntimeBridgeCallbackKind, MobileRuntimeBridgeContract,
    MobileRuntimeBridgeEntryPoint, MobileRuntimeCapabilityBinding,
    MobileRuntimeDeviceSmokeArtifact, MobileRuntimeHostScaffold, MobileRuntimeLifecycleBinding,
    MobileRuntimePermission,
};
use crate::{NativeUiAdapterCapability, NativeUiBackendStatus, NativeUiPlatform, NativeUiToolkit};

pub fn harmony_ability_host_scaffold() -> MobileRuntimeHostScaffold {
    MobileRuntimeHostScaffold {
        platform: NativeUiPlatform::Harmony,
        platform_name: NativeUiPlatform::Harmony.platform_name(),
        toolkit: NativeUiToolkit::HarmonyAbility,
        toolkit_name: NativeUiToolkit::HarmonyAbility.toolkit_name(),
        status: NativeUiBackendStatus::AdapterBoundaryScaffold,
        status_name: NativeUiBackendStatus::AdapterBoundaryScaffold.status_name(),
        module_path: "src/harmony_ability_host.rs",
        native_library_name: "libzsui_harmony.so",
        application_manifest_file: "harmony/entry/src/main/module.json5",
        native_window_type: "OpenHarmony Ability window",
        rust_entry_point: "zsui_harmony_ability_start",
        bridge_entry_points: harmony_ability_bridge_entry_points(),
        bridge_contract: harmony_ability_bridge_contract(),
        lifecycle_bindings: harmony_ability_lifecycle_bindings(),
        capability_bindings: harmony_ability_capability_bindings(),
        required_permissions: harmony_ability_required_permissions(),
        target_smoke_requirements: vec![
            "OpenHarmony device or emulator launch",
            "visible Ability window screenshot",
            "input method focus check",
            "pasteboard service roundtrip",
            "device smoke artifacts",
        ],
        next_implementation_steps: vec![
            "add N-API or native bridge entry point",
            "map Ability lifecycle to NativeRuntimeDriver",
            "bind Ability window surface to main_window",
            "capture device smoke artifacts and review them with mobile_scaffold_manifest --review harmony",
        ],
    }
}

pub fn harmony_ability_bridge_entry_points() -> Vec<MobileRuntimeBridgeEntryPoint> {
    vec![
        MobileRuntimeBridgeEntryPoint::new(
            "zsui_harmony_ability_start",
            "Rust FFI",
            "src/harmony_ability_host.rs",
            "start the ZSUI runtime from the Ability bridge",
        ),
        MobileRuntimeBridgeEntryPoint::new(
            "ZsuiAbility.onCreate",
            "ArkTS",
            "harmony/entry/src/main/ets/ZsuiAbility.ets",
            "create the Ability window and call into Rust",
        ),
        MobileRuntimeBridgeEntryPoint::new(
            "ZsuiAbility.onDestroy",
            "ArkTS",
            "harmony/entry/src/main/ets/ZsuiAbility.ets",
            "request framework shutdown before Ability teardown",
        ),
    ]
}

pub fn harmony_ability_bridge_contract() -> MobileRuntimeBridgeContract {
    MobileRuntimeBridgeContract {
        platform: NativeUiPlatform::Harmony,
        platform_name: NativeUiPlatform::Harmony.platform_name(),
        toolkit: NativeUiToolkit::HarmonyAbility,
        toolkit_name: NativeUiToolkit::HarmonyAbility.toolkit_name(),
        module_path: "src/harmony_ability_host.rs",
        native_library_name: "libzsui_harmony.so",
        rust_entry_point: "zsui_harmony_ability_start",
        foreign_language: "ArkTS",
        foreign_entry_file: "harmony/entry/src/main/ets/ZsuiAbility.ets",
        callbacks: harmony_ability_bridge_callbacks(),
        device_smoke_artifacts: harmony_ability_device_smoke_artifacts(),
        safety_rules: vec![
            "keep N-API/native raw handles inside the Harmony host module",
            "return Result-style error codes across the native bridge; do not panic through FFI",
            "map Ability lifecycle callbacks to NativeRuntimeDriver stages before creating product behavior",
            "record lifecycle, surface, input and pasteboard artifacts on a real emulator or device before completion claims",
        ],
    }
}

pub fn harmony_ability_bridge_callbacks() -> Vec<MobileRuntimeBridgeCallback> {
    use MobileRuntimeBridgeCallbackKind::{
        Bootstrap, Command, EventPoll, Input, Lifecycle, Shutdown, Surface,
    };

    vec![
        MobileRuntimeBridgeCallback::new(
            "ZsuiAbility.nativeStart",
            "zsui_harmony_ability_start",
            Bootstrap,
            "Ability context, launch Want and NativeRuntimeStartupRequest",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiAbility.nativeLifecycle",
            "zsui_harmony_ability_lifecycle",
            Lifecycle,
            "Ability lifecycle stage name: create/window-stage-create/foreground/background/destroy",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "onWindowStageCreate",
            "zsui_harmony_ability_surface_created",
            Surface,
            "WindowStage handle and current Dpi",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "onAreaChange",
            "zsui_harmony_ability_surface_resized",
            Surface,
            "surface width, height and Dpi",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "onWindowStageDestroy",
            "zsui_harmony_ability_surface_destroyed",
            Surface,
            "WindowStage identity",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ArkUI.onTouch/onKeyEvent",
            "zsui_harmony_ability_dispatch_ui_event",
            Input,
            "typed touch/key/input event translated to UiEvent",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiAbility.nativeDispatchCommand",
            "zsui_harmony_ability_dispatch_command",
            Command,
            "UiCommand id and payload from platform controls",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiAbility.nativePollAppEvent",
            "zsui_harmony_ability_poll_app_event",
            EventPoll,
            "next AppEvent from NativeRuntimeDriver",
            true,
        ),
        MobileRuntimeBridgeCallback::new(
            "ZsuiAbility.onDestroy",
            "zsui_harmony_ability_shutdown",
            Shutdown,
            "shutdown reason and Ability identity",
            true,
        ),
    ]
}

pub fn harmony_ability_lifecycle_bindings() -> Vec<MobileRuntimeLifecycleBinding> {
    vec![
        MobileRuntimeLifecycleBinding::new("onCreate", "start_native_runtime", true),
        MobileRuntimeLifecycleBinding::new("onWindowStageCreate", "bind_render_surface", true),
        MobileRuntimeLifecycleBinding::new("onForeground", "poll_native_event", true),
        MobileRuntimeLifecycleBinding::new("onAreaChange", "resize_render_surface", true),
        MobileRuntimeLifecycleBinding::new("onBackground", "poll_product_event", true),
        MobileRuntimeLifecycleBinding::new("onWindowStageDestroy", "release_render_surface", true),
        MobileRuntimeLifecycleBinding::new("onDestroy", "request_shutdown", true),
    ]
}

pub fn harmony_ability_capability_bindings() -> Vec<MobileRuntimeCapabilityBinding> {
    use NativeUiAdapterCapability::{
        Clipboard, EditDialog, FileDialog, Ime, InputDialog, MainExecutionPlanBridge,
        MainSearchControl, MainWindow, PasteTarget, PopupMenu, Renderer, SettingsDropdown,
        SettingsWindow, ShellOpen, StatusItem, TextLayout, TransientWindow, WindowIdentity,
    };

    vec![
        MobileRuntimeCapabilityBinding::new(
            MainWindow,
            "OpenHarmony Ability window",
            "NativeMainWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            SettingsWindow,
            "ArkUI settings page",
            "NativeSettingsWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            SettingsDropdown,
            "ArkUI select or menu",
            "NativeSettingsDropdownHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            InputDialog,
            "ArkUI text input dialog",
            "NativeTextInputDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            EditDialog,
            "Ability text editor surface",
            "NativeEditTextDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            Clipboard,
            "OpenHarmony pasteboard",
            "ClipboardHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(PopupMenu, "ArkUI menu", "NativePopupMenuHost", false),
        MobileRuntimeCapabilityBinding::new(
            StatusItem,
            "OpenHarmony notification",
            "status item bridge",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(Renderer, "ArkUI Canvas", "Renderer", false),
        MobileRuntimeCapabilityBinding::new(TextLayout, "ArkUI text layout", "TextLayout", false),
        MobileRuntimeCapabilityBinding::new(
            MainSearchControl,
            "ArkUI search component",
            "NativeMainSearchControlHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            TransientWindow,
            "ArkUI popup component",
            "NativeTransientWindowHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            Ime,
            "OpenHarmony input method bridge",
            "NativeImeHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            ShellOpen,
            "OpenHarmony Want launcher",
            "NativeShellOpenHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            FileDialog,
            "OpenHarmony document picker",
            "NativeFileDialogHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            PasteTarget,
            "focused ArkUI component paste target",
            "NativePasteTargetHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            WindowIdentity,
            "Ability identity and window id",
            "NativeWindowIdentityHost",
            false,
        ),
        MobileRuntimeCapabilityBinding::new(
            MainExecutionPlanBridge,
            "Ability-to-Rust command bridge",
            "NativeRuntimeDriver",
            false,
        ),
    ]
}

pub fn harmony_ability_required_permissions() -> Vec<MobileRuntimePermission> {
    vec![
        MobileRuntimePermission::new(
            "ohos.permission.NOTIFICATION_CONTROLLER",
            "status_item",
            false,
        ),
        MobileRuntimePermission::new("ohos.permission.READ_PASTEBOARD", "clipboard", false),
    ]
}

pub fn harmony_ability_device_smoke_artifacts() -> Vec<MobileRuntimeDeviceSmokeArtifact> {
    vec![
        MobileRuntimeDeviceSmokeArtifact::new(
            "mobile_manifest",
            "manifest.json",
            true,
            "serialized Harmony mobile bridge manifest",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "device_launch_log",
            "device-launch.log",
            true,
            "hdc or emulator launch output and exit status",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "device_window_screenshot",
            "device-window.png",
            true,
            "screenshot proving the Ability window is visible",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "lifecycle_trace",
            "lifecycle.json",
            true,
            "ordered Ability lifecycle callbacks observed by the Rust bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "surface_trace",
            "surface.json",
            true,
            "WindowStage create/resize/destroy callbacks observed by the Rust bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "input_trace",
            "input.json",
            true,
            "touch/key/input event dispatch observed by the typed UI event bridge",
        ),
        MobileRuntimeDeviceSmokeArtifact::new(
            "pasteboard_trace",
            "pasteboard.json",
            false,
            "OpenHarmony pasteboard roundtrip proof when clipboard capability is enabled",
        ),
    ]
}
