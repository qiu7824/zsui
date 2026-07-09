# Native Host Smoke Artifacts

Target smoke is the evidence layer between code-level contracts and a complete
native backend. A platform is not system-complete until a real target run stores
inspectable artifacts under:

```text
target/native-host-smoke/<platform>/
```

Generate the smoke manifest with:

```powershell
cargo run --example native_smoke_manifest -- windows
```

Use `macos`, `linux`, `android`, `harmony`, `current` or `all` for other
manifest scopes.

Record the contract-level artifact files with:

```powershell
cargo run --example native_smoke_record -- windows
```

This writes `manifest.json`, `launch.log`, `interaction.json`,
`capabilities.json` and `agent-context.json`. It intentionally does not fake
`window.png`; run the interactive native smoke command to capture or provide a
real target screenshot before target-smoke is complete.

Run the first-pass native smoke window with:

```powershell
cargo run --example native_smoke_run -- windows
```

On Windows this opens the extracted Win32/GDI native window path, closes it
automatically, then rewrites `interaction.json` and `launch.log` with the
observed window lifecycle. It also captures `window.png` into the artifact
directory through the direct Win32 `HWND`. macOS and Linux still use the
`winit_desktop` first-pass runtime and need platform screenshot capture support
before their target-smoke proof is complete.

Windows can also request a real status item during the same smoke run:

```powershell
cargo run --example native_smoke_run -- windows --tray
```

That path uses the `Shell_NotifyIconW` backed `WindowsWin32StatusItemHost` and
records status-item fields in `interaction.json`. It also exercises the
native status-menu command table and records `status_menu_command_routed`.
It creates and destroys a native popup menu and records
`status_menu_popup_destroyed`. Real user popup menu clicks are still separate
proof before the tray surface is system-complete; the Win32 host exposes the
`TrackPopupMenu` selection route, but the auto-closing smoke runner does not
block waiting for manual selection.

Windows can also attach a typed Rust view draw plan during the smoke run:

```powershell
cargo run --example native_smoke_run -- windows --view
```

That path exercises `NativeWindowBuilder::view(...)`, records draw-plan command
counts in `interaction.json`, and paints the resulting `NativeDrawPlan` through
the extracted no-flicker Win32/GDI renderer.

Review the artifact directory with:

```powershell
cargo run --example native_smoke_review -- windows
```

The review is read-only. It checks required files, rejects empty artifacts,
validates JSON artifacts, validates the `window.png` PNG header and reports
`target_smoke_complete=false` until every required target artifact is present
and valid.

Required target-smoke artifacts:

- `manifest.json`: serialized `NativeHostSmokePlan`.
- `launch.log`: native runtime launch output and exit status.
- `window.png`: screenshot proving the native window was visible.
- `interaction.json`: structured interaction record.
- `capabilities.json`: observed host capability report.
- `agent-context.json`: matching `zsui_agent_context_json()` output.

Windows currently uses the extracted `win32_gdi` runtime and is ready to attempt
target smoke. macOS and Linux use the `winit_desktop` first-pass runtime.
Android and Harmony are still scaffold/bridge-contract plans until real
Activity/Ability runtime hosts exist. Their current device-smoke contract can
be inspected with:

```powershell
cargo run --example mobile_scaffold_manifest -- --bridge android
cargo run --example mobile_scaffold_manifest -- --bridge harmony
cargo run --example mobile_scaffold_manifest -- --smoke android
cargo run --example mobile_scaffold_manifest -- --review android
```

The mobile contracts require device-side artifacts such as
`device-launch.log`, `device-window.png`, `lifecycle.json`, `surface.json` and
`input.json` before a mobile backend can move beyond scaffold status. The
review command validates artifact presence, JSON files and PNG headers; it does
not generate or fake device proof.

Current Windows proof command sequence:

```powershell
cargo run --example native_smoke_run -- windows
cargo run --example native_smoke_review -- windows
```

The Windows review should report `target_smoke_complete=true` after the run
because all six required artifacts, including `window.png`, are generated and
validated.
