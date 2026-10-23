# Application effects and dispatch budgets

`AppCx` queues native file, save, directory and message dialogs. Desktop hosts
execute these services on the UI thread after releasing View and route locks.
The typed outcome returns through the same application's update function.

## Admission and cancellation

Each window has separate FIFO queues for application commands, UI commands and
native effects, each limited to 4,096 outstanding entries. The update context
receives the remaining capacity before reducing messages. `try_command`,
`try_ui_command`, `try_effect_with` and the typed `try_*_dialog` / `try_pick_directory`
helpers return `QueueFull` when the request was not accepted. Do not busy-loop on
the UI thread; retain application intent and retry on a later application event.
No accepted action is evicted to admit a newer one.

Legacy void helpers remain available. Rejections accumulate in
`AppCx::rejected_count` and native structured dispatch errors. Custom View hosts
can consume `SharedLiveViewRuntime::take_queue_rejections`. The independent
`CommandQueue` has `with_capacity`, `try_push`, `capacity` and `rejected_count`;
legacy `push` also emits a diagnostic on its first rejected admission.

Successful effect admission returns `AppEffectCancellation`. Tokens and request
clones share one lifecycle. Cancellation before execution prevents the native
call; cancellation during a modal call suppresses its eventual result. It does
not forcibly close an OS dialog. A matching result claims delivery once;
duplicates and mismatched outcome kinds cannot call the responder. A hidden
window may receive results, while destruction closes its action scope, clears
pending work and prevents further update calls. A committed delivery cannot
subsequently be cancelled.

Stable applications use the corresponding `UpdateContext::try_open_file_dialog`,
`try_save_file_dialog`, `try_pick_directory` and `try_show_dialog` helpers.

## Event-loop fairness

A native-effect turn takes at most 32 requests and checks a 4-ms deadline before
removing each next request. An already running synchronous native service cannot
be preempted. Application and UI commands are limited to 32 per kind per turn.
Remaining work stays in the owning queue and schedules a subsequent event-loop
tick. No timer remains for an empty queue. A modal nested event loop may process
input but cannot start a second dispatcher for the same window.

Text edit commands have a 32-operation per-update admission limit. Quit is an
independent state flag and does not depend on queue capacity.

## Directory services

Windows uses an owner-bound shell folder picker with an STA lifetime, initial
folder and long-path resolution. AppKit uses a directory-only `NSOpenPanel` with
the existing owner sheet contract. GTK uses `FileChooserNative::SelectFolder`.
AppKit waits through the native file-panel modal session; native alert sheets
dispatch application events while waiting for their completion callback.
Linux Direct calls the XDG FileChooser portal through a blocking native D-Bus
connection. Response `0` returns validated local file paths, `1` means explicit
user cancellation, and `2`, invalid responses, malformed selections or a lost
provider return `Err`. A missing portal may fall back to Zenity; exit status `1`
is cancellation, while other failures retain their diagnostics.
All return local `PathBuf` values; cancellation is `Ok(None)`. Unsupported remains
the contract for backends without directory services. An unavailable portal and
missing Zenity return `Unsupported`; neither is reported as a cancelled choice.

AppKit file, save and directory services also accept worker-thread callers while
the native application loop is running. They enqueue panel creation on the main
queue and return the completion result to the worker. At most 32 panel requests
are outstanding. A stopped event-loop generation terminates waiting callers and
cannot start stale requests in a subsequent loop. Without an active host, a
worker call fails immediately. Native panel proof captures composited screen
content within the panel bounds, including the system's remote view service.

## Window commands and external wakeups

`AppCx::try_window_command` admits up to 32 window operations per update.
`window_command` keeps the legacy convenience shape and records rejected
admissions in the normal diagnostics. `Minimize`, `ToggleMaximize`, `Close`,
`Hide` and `Show` use native host operations; Close follows the application's
normal veto policy. `on_window_state_change` receives changes to native maximize
state, including changes made by the window manager, without repeated identical
notifications.

`UiInvalidationHandle::request_app_command` sends an owned application command
from a worker to the owning UI thread. Its separate 32-entry FIFO also wakes a
hidden window when a repaint is already pending. Commands do not consume that
pending repaint. Custom command identifiers are limited to 256 bytes and payloads
to 65,536 bytes. Destruction clears the queue and rejects further admission.
An application maps its restore command to `ZsWindowCommand::Show` through the
normal `stateful_view_with_app_commands` callback.

Win32, AppKit and the Linux X11 hosts implement hide/restore. The Winit Wayland
profile does not expose client-driven visibility, unminimizing or forced focus;
unsupported hide/restore requests retain the active View and record diagnostics.
Window-manager activation policy remains authoritative.

## Display scale changes

Win32 uses physical pixels and the window DPI. AppKit, GTK and the Wayland/X11
direct renderer retain logical layout coordinates and apply backing scale once
at presentation. Native scale-change callbacks cancel captured gestures,
invalidate text measurements, rebuild retained content and refresh IME geometry
without replacing application-owned pane extents or text values.
Native input shapers consume the runtime's normalized typography scale, matching
the retained paint plan and semantic body font. System font preferences are
resolved before installing the Direct host's initial input backend; raw and
normalized font scales must not diverge between caret and paint paths.
`ZSUI_NATIVE_SCALE_TRACE` is an optional native-smoke trace of real host scale
notifications. It is not a mechanism for injecting scale changes.

The controlled dispatcher tests cover 1,000 queued requests, 100 chained results,
FIFO batching, admission at capacity, deadline expiry, provider failure,
cancellation, duplicate results and window destruction. Native visual and service
proof remains platform-specific; a cross-target check does not certify interaction.

[Win32, AppKit, Linux Direct, Direct Lite and GTK target proof](https://github.com/qiu7824/zsui/actions/runs/39750838016)
at `45f2c6b0d34bcff2d362119a5a26fb4666a11ac5` verifies directory selection and
cancellation through the application update loop. The selected path must match
the requested directory exactly. AppKit proof sends Return/Escape through
WindowServer to the native remote panel and captures its composited bounds;
capturing only the local panel proxy omits the remote controls.

[Desktop lifecycle and scale proof](https://github.com/qiu7824/zsui/actions/runs/39930480963)
adds window command/close-veto cases, AppKit worker panels, actual D-Bus provider
failure and missing-provider cases. Direct, Direct Lite and GTK move between
two Sway virtual outputs at 1x and 2x, retain input after the round trip, and
perform input and capture at 2x. Pixel checks compare visible text and caret
positions. Physical multi-monitor and fractional-scale behavior require
separate target-device verification.

## Native input verification

The Windows retained input path uses the same Rust glyph geometry as its paint
engine, including CJK, emoji, DPI and typography scaling. The
`desktop_reliability` example verifies a long path, Home/End, select-all, IME
commit, View rebuilding, actual window resize and an adjacent action beside an
ellipsized label. These checks complement the queue lifecycle tests.

Windows UIA object dispatch normalizes WM_GETOBJECT's 32-bit identifier before
comparison, including on 64-bit hosts; external UIA range actions are verified
against retained sidebar, horizontal/vertical panel and table column state.
