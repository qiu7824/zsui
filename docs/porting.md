# Host Porting Contract

A ZSUI host translates framework declarations into native platform behavior.
It should not embed product business logic.
Current platform names include Windows, macOS, Linux, Android and Harmony.
Android and Harmony are capability scaffolds until dedicated mobile runtime
hosts are implemented.

Implement host surfaces in this order:

1. `ZsuiHost::capabilities`.
2. Window creation and visibility.
3. Tray/status menu creation.
4. Menu and hotkey command routing.
5. Clipboard text, then images and files.
6. File picker and native dialogs.
7. Settings page presentation.
8. Event polling and event-loop ownership.

Each backend should report real support through `HostCapabilities`.
Use `CapabilityStatus::Partial` when a declaration can be accepted but native
behavior is incomplete, session-dependent or not yet smoke-tested.

Platform handles, native widget objects and message/event-loop details belong
inside the host implementation. Product behavior belongs behind the
application's product adapter.
