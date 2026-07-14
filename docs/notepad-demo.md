# ZSUI Notepad Acceptance App

`examples/zsui_notepad.rs` is a framework acceptance application written once
for the target-native Win32, AppKit and GTK4 hosts. It contains no platform
`cfg`, raw handle, Objective-C object, GTK object, native event loop or WebView.

The application is deliberately small in scope. It proves that a normal ZSUI
application can combine the shared State/Msg/view/update loop with a self-drawn
multiline editor and target-native desktop services without making the demo the
framework architecture.

## Shared application path

- `native_window(...).stateful_view_with_app_commands(...).run()` is the only
  application entry path on all three desktop targets.
- `text_editor` owns the shared multiline editing surface, focus, selection,
  keyboard input and IME integration. Target renderers adapt its metrics and
  visuals to the platform theme.
- `ViewNode::text_wrap(TextWrap)` switches the shared editor between word-wrap
  and hard-line-only rendering at runtime. Paint, caret, selection and pointer
  hit geometry consume the same typed setting on all three hosts.
- Multiline Up/Down navigation consumes those same visual hard/soft rows.
  PageUp/PageDown moves by the current visible-row count and scrolls the editor
  viewport by the same page. Both retain the desired shaped x position when an intermediate
  row is shorter; Shift keeps the application-owned anchor while extending the
  caret on all three hosts.
- Long documents use an editor-owned visual-row viewport. Text paint is clipped
  to that viewport, pointer hit testing includes its first visible row, wheel
  input moves it, and edits or keyboard movement reveal the active caret again.
  With `TextWrap::NoWrap`, the same transient viewport also tracks a horizontal
  pixel offset, so caret movement, paint, selection and pointer hits stay aligned
  for proportional and bidirectional text. Wrapped modes reset that offset. A captured drag
  beyond an editor edge advances the corresponding visual row or horizontal offset by one
  step per pointer update and extends the same typed selection at the newly
  visible edge. This behavior does not enter application document state or
  require the general `scroll` feature.
- `ZsTextSelection` reports application-independent Unicode-scalar anchor and
  caret positions through `on_text_selection_change`, including edits,
  keyboard navigation and pointer drag selection. The shared runtime keeps
  those scalar indices on Unicode extended-grapheme boundaries, so combining
  sequences and joined emoji move, delete, wrap and hit-test as one unit.
- Win32 Uniscribe, AppKit Core Text and GTK4 Pango shape each visual line. Their
  proportional advances, RTL cluster boxes and primary/secondary caret positions feed
  the shared selection, hit, wrap, horizontal/vertical navigation and viewport
  geometry. Left/Right follows primary caret x order without exposing bidi run
  state to application code.
- `AppCx::text_edit_command(...)` and `text_edit_command_for(...)` queue
  strongly typed `ZsTextEditCommand` requests for the focused or explicitly
  identified editor. The host owns a bounded undo history and returns edits
  through the existing `TextEdited`/selection message path.
- Cut, copy and paste use the optional target-dispatched system clipboard:
  Win32 through the Windows clipboard provider, AppKit through `NSPasteboard`
  and GTK4 through `gdk::Clipboard`. Application code sees no platform object.
- `ZsTextDocument` owns UTF-8/UTF-16 decoding, path and encoding metadata,
  explicit dirty state and transactional UTF-8 save/save-as.
- `ZsDocumentShellCommand` converts to and from the public `Command` type, so
  the same typed commands drive buttons, native menus and accelerators.
- `NativeFileDialogService` selects Win32 open/save dialogs, AppKit
  `NSOpenPanel`/`NSSavePanel`, or GTK4 `FileChooserNative` behind one safe API.
- File dialogs and filesystem I/O execute after the live-view lock is released.
  A successful external effect refreshes the shared view before native repaint.
- `on_close_requested(ZsDocumentShellCommand::Close.to_command())` routes the
  Win32, AppKit or GTK4 title-bar close affordance into the same typed update as
  the menu/button command. Dirty state vetoes the native close until the user
  chooses Save, Discard or Cancel; clean state approves it with `AppCx::quit()`.
- The dirty-document decision is an in-view, self-drawn confirmation surface;
  it does not introduce a second platform widget tree.

ZSUI does not link WebView2, WKWebView, WebKitGTK or a browser-shell runtime.
The separately isolated Tauri comparison remains a comparison input only and
is never part of the ZSUI feature graph.

## Run and verify

Run the application with only its required feature slice:

```powershell
cargo run --example zsui_notepad --no-default-features --features notepad-demo
```

Run the auto-closing native smoke path:

```powershell
cargo run --example zsui_notepad --no-default-features --features notepad-demo -- --smoke
```

The smoke requires a visible native window, a routed native menu command, real
text input through the self-drawn editor and a typed Undo routed from the
self-drawn command bar back to that editor. It also enters 36 lines, captures a
selection drag beyond the editor edge and verifies two incremental viewport
steps, routes a native Up key through the shared visual-row navigation, scrolls
the editor one visual page with PageDown and with a real wheel message, toggles
word wrap off, enters a marked long line and routes End to prove horizontal
caret reveal while painting a mixed Latin/Hebrew/CJK proportional line. It then
enters a dedicated Latin/Hebrew line and routes Home plus four Right keys through
the shaped visual-order path. It also
commits a combining sequence plus joined emoji and uses
Left/Backspace to prove that one complete grapheme is removed, then sends a real
title-bar close request while the document is dirty, verifies that the request
is vetoed and captures the shared unsaved-confirmation surface. On
non-Windows targets the same source is compiled against the AppKit or GTK4
host; target runtime evidence is tracked separately and is not inferred from
cross-compilation.

## Current functional boundary

| Capability | Current acceptance state |
| --- | --- |
| Shared application source on Win32/AppKit/GTK4 | implemented |
| Self-drawn multiline input, focus and IME host routing | implemented |
| New/open/save/save-as and dirty decision | implemented |
| Target-native menu and primary-key accelerators | implemented |
| Target-native open/save panel facade | implemented |
| UTF-8 save and UTF-8/UTF-16 input decode | implemented |
| Caret-aware line/column, line count, character count and encoding status | implemented |
| Undo/cut/copy/paste/select-all command API | implemented |
| Runtime word-wrap toggle | implemented |
| Wrapped visual-row Up/Down, PageUp/PageDown and Shift selection | implemented |
| Long-document vertical viewport and shaped no-wrap horizontal caret reveal | implemented |
| Captured selection drag with row/column edge scrolling | implemented |
| Extended-grapheme-safe caret, deletion, wrapping and pointer hit testing | implemented |
| Proportional advances and bidirectional caret/selection/hit geometry | implemented; AppKit/GTK4 target proof pending |
| Visual-order bidirectional Left/Right navigation | implemented; AppKit/GTK4 target proof pending |
| Intercepting the operating-system window-close button | implemented on Win32/AppKit/GTK4 |
| AppKit and GTK4 physical-machine interaction evidence | pending target runners |

The current Win32 shaped-text smoke capture is stored at
[`platform-proof/windows/shaped-text.png`](platform-proof/windows/shaped-text.png).
It is diagnostic Windows evidence and does not substitute for the pending
AppKit/GTK4 target runs.

Commands still outside the shared editor contract are not placed in the menu.
This avoids claiming behavior that exists only in one platform service.

## Optional feature boundary

`notepad-demo` enables only `window`, `button`, `label`, `textbox`, `clipboard`
and `document-shell`. Cargo then selects the dependency for the current desktop
target. Editor vertical and no-wrap horizontal viewport state belongs to
`textbox` and does not enable the general `scroll` container. Clipboard support
is therefore omitted when applications do not select it. `textbox` enables the
small internal `text-input-core` slice and its Unicode segmentation dependency;
non-text input controls do not. The Windows-only
`WindowsWin32OwnedTextEditor` remains an optional framework service, but the
acceptance application does not depend on it.

## Code-volume and runtime comparison

The shared acceptance application is one source file with 743 nonblank lines,
including its tests. The former Windows-only application path used two source
files with 732 nonblank lines, so the checked-in application surface is 11
lines (1.5%) larger while replacing two platform-coupled files with one shared
cross-platform path and adding typed caret
status, native menus, shared editing commands, runtime word-wrap coverage and
visual-row/page navigation, two-axis editor caret reveal, edge-drag scrolling,
extended-grapheme acceptance and native close-request interception.

Runtime, package-count and binary-size data must be regenerated after this
rewrite; earlier Windows-only measurements are not presented as current data.
The comparison script now counts the single shared source file:

```powershell
.\scripts\measure-notepad-comparison.ps1
```

It builds each baseline in an isolated support directory and records complete
process-tree memory. Generated targets, screenshots and reports remain outside
the repository.
