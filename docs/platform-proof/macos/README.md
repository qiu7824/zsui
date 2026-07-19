# macOS platform proof

Backend: `macos-appkit`

The AppKit host now contains the target-side final-view capture path used by
`--native-proof`: it performs layout and display, caches the ZSUI `NSView` into
an `NSBitmapImageRep`, encodes PNG data and records logical size, pixel size and
the measured backing scale. `.github/workflows/native-proof.yml` runs the first
Gallery and Notepad proof scenes on `macos-15` ARM64 and uploads their artifacts.
No AppKit runtime proof is accepted until that target job has completed
successfully. Winit screenshots are not AppKit completion evidence.

Reviewed documentation evidence:

- `gallery-inputs-light.png` and `.json`: Gallery input page captured from the
  final AppKit `NSView` on `macos-15` ARM64.
- `notepad-interaction.png` and `.json`: the shared Notepad input, selection,
  scrolling and unsaved-close scenario captured from the final `NSView`.

The JSON records `backend=appkit`,
`capture_backend=appkit_nsview_bitmap_cache`, target typography, window geometry,
focus, messages, process memory and runtime errors. These selected documentation
artifacts do not replace the full per-run artifact uploaded by CI.

The blocking AppKit runtime, screenshot, structured-report and visual-regression
contract for ZSUI 0.3.0 is defined in
[`../../v0.3-native-proof-ci.md`](../../v0.3-native-proof-ci.md). Accepted
evidence must be produced by the real AppKit host on the fixed GitHub-hosted
`macos-15` ARM64 runner and must capture the final `NSView` rather than a shared
`NativeDrawPlan` image.
