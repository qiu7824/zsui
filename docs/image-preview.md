# Retained image preview

The optional `image-preview` feature provides an application-owned PNG decoder
and a self-drawn preview node:

```toml
zsui = { git = "https://github.com/qiu7824/zsui", default-features = false, features = [
    "window", "button", "label", "image-preview"
] }
```

```rust
let mut preview = ZsImagePreviewState::new(ZsImagePreviewConfig::default())?;
preview.set_png(ZsImageFrameId::new(revision), png_bytes);

let snapshot = preview.snapshot();
let content = image_preview(&snapshot)
    .image_fit(ZsImageFit::Contain)
    .image_interpolation(NativeImageInterpolation::Smooth);
```

## Frame lifecycle

- PNG parsing and pixel conversion run on one owned decoder thread.
- A pending request is replaced by the newest generation before decoding starts.
- A result is published only when its generation is still current.
- The last complete frame remains visible while the next frame is loading.
- Draw-plan clones share immutable premultiplied pixel storage through `Arc`.
- Decoded dimensions are checked against `max_decoded_bytes` before frame
  allocation.

This lifecycle avoids empty intermediate frames. On Win32, the published frame
is drawn into the existing buffered-paint target and presented with the rest of
the draw plan in one update. `WM_ERASEBKGND` remains suppressed.

`ZsImageFit` supports `Contain`, `Cover` and `Stretch`. Geometry is calculated
once in the shared View paint path, so source cropping and destination bounds
are explicit in `NativeDrawImageCommand`.

Win32 consumes premultiplied frames through GDI+, AppKit through Core Graphics
and GTK4 through Cairo. Windows has local compile and smoke coverage; AppKit and
GTK4 raster output still require target screenshots and interaction evidence.

## Verification

```powershell
cargo test --lib --no-default-features --features image-preview
cargo check --example image_preview --no-default-features --features window,button,label,image-preview
cargo run --example image_preview --no-default-features --features window,button,label,image-preview -- --smoke
```
