use crate::mobile_host::{
    MobileRuntimeBridgeEntryPoint, MobileRuntimeCapabilityBinding, MobileRuntimeHostScaffold,
    MobileRuntimeLifecycleBinding, MobileRuntimePermission,
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
            "capture device smoke artifacts with native_smoke_review",
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

pub fn harmony_ability_lifecycle_bindings() -> Vec<MobileRuntimeLifecycleBinding> {
    vec![
        MobileRuntimeLifecycleBinding::new("onCreate", "start_native_runtime", true),
        MobileRuntimeLifecycleBinding::new("onForeground", "poll_native_event", true),
        MobileRuntimeLifecycleBinding::new("onBackground", "poll_product_event", true),
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
