# Host Porting Contract

A ZSUI host translates framework declarations into native platform behavior.
It should not embed product business logic.
Current platform names include Windows, macOS, Linux, Android and Harmony.
Android and Harmony are capability scaffolds until dedicated mobile runtime
hosts are implemented. Use `mobile_runtime_host_scaffold(platform)` or
`examples/mobile_scaffold_manifest.rs` to inspect the current Activity/Ability
bridge entry points, lifecycle bindings, capability bindings and target smoke
requirements.

Implement host surfaces in this order:

1. `ZsuiHost::capabilities`.
2. Window creation and visibility.
3. Tray/status menu creation.
4. Menu and hotkey command routing.
5. Clipboard text, then images and files.
6. File picker and native dialogs.
7. Settings page presentation.
8. Event polling and event-loop ownership.

For product-adapter startup, desktop/mobile runtime drivers should map
`NativeRuntimeStartupRequest.status_item` through `NativeStatusItemHost` and
`NativeRuntimeStartupRequest.settings_pages` through
`NativeSettingsPageModelHost`. This keeps status menus and settings models as
native host responsibilities while command execution and product state remain
behind `ProductAdapterHost`.

Each backend should report real support through `HostCapabilities`.
Use `CapabilityStatus::Partial` when a declaration can be accepted but native
behavior is incomplete, session-dependent or not yet smoke-tested.

Platform handles, native widget objects and message/event-loop details belong
inside the host implementation. Product behavior belongs behind the
application's product adapter.
For reusable applications, implement `ProductAdapterHost` and connect it through
`ZsuiReusableRuntimeHarness` before adding product-specific callbacks to a
platform host. Run `examples/product_adapter_smoke.rs` or call
`ProductAdapterRuntimeSmokeRequest` to prove the product boundary before wiring
the adapter to a target native runtime.
On desktop, `NativeWindowRuntimeDriver` is the current reusable driver bridge
between `ZsuiReusableRuntimeHarness` and the minimal native-window runtime.
Before claiming target-smoke readiness, generate a platform manifest with
`native_host_smoke_plan(platform)` or `examples/native_smoke_manifest.rs` and
store the required artifacts described in `docs/native-host-smoke.md`.
