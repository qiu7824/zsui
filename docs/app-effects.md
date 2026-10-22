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
Linux Direct uses its existing portal/Zenity provider through `rfd::pick_folder`.
All return local `PathBuf` values; cancellation is `Ok(None)`. Unsupported remains
the contract for backends without directory services. Linux Direct inherits
`rfd`'s inability to distinguish provider failures from cancellation.

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

## Native input verification

The Windows retained input path uses the same Rust glyph geometry as its paint
engine, including CJK, emoji, DPI and typography scaling. The
`desktop_reliability` example verifies a long path, Home/End, select-all, IME
commit, View rebuilding, actual window resize and an adjacent action beside an
ellipsized label. These checks complement the queue lifecycle tests.

Windows UIA object dispatch normalizes WM_GETOBJECT's 32-bit identifier before
comparison, including on 64-bit hosts; external UIA range actions are verified
against retained sidebar, horizontal/vertical panel and table column state.
