# Video and camera preview surface

The optional `video` feature provides a retained latest-frame surface for
decoded video and live camera previews:

```toml
zsui = { version = "0.2.0", default-features = false, features = [
    "window", "video"
] }
```

`ZsVideoSource` is cloneable and thread-safe. A capture or decoder thread
presents immutable RGBA frames while the View owns another clone:

```rust
use std::time::Duration;
use zsui::{video, ZsImageFrame, ZsImageFrameId, ZsVideoFit, ZsVideoSource};

let source = ZsVideoSource::new();

// Called by the application's camera or decoder integration.
let frame = ZsImageFrame::from_rgba8(
    ZsImageFrameId::new(sequence),
    width,
    height,
    rgba,
)?;
source.present(frame, Duration::from_millis(presentation_time_ms));

let view = video::<Msg>(source.clone()).video_fit(ZsVideoFit::Cover);
```

The source retains only the newest complete frame. If producers run faster
than the UI refresh cadence, older frames are replaced instead of accumulating
in an unbounded queue. `ZsVideoSurfaceConfig::refresh_interval` selects a
bounded 8–1000 ms repaint cadence; the default is 16 ms. Polling stops while
the source is paused, ended, failed or idle.

`ZsVideoFit` supports `Contain`, `Cover` and `Stretch`.
`video_interpolation` selects nearest-neighbor or smooth sampling. Windows
presents frames inside the existing buffered paint path; AppKit and Linux use
the same shared image draw command and their native raster presenters.

## Media integration boundary

The component is a presentation surface, not a codec or device API. Camera
enumeration, permissions, demuxing, decoding, network reconnection, audio
output, seeking, subtitles and transport controls remain application-owned or
belong in an optional media adapter. This keeps media frameworks and system
libraries out of applications that only need ordinary UI.

The Rust API accepts frames from Media Foundation, AVFoundation, GStreamer,
FFmpeg or another producer without exposing those platform handles through
ZSUI. Audio should use the same application-owned playback clock passed as the
frame presentation timestamp.

## UiDocument

UiDocument can author the surface geometry and a bounded poster frame:

```json
{
  "id": "camera-preview",
  "component": "video",
  "properties": {
    "frame": null,
    "fit": "cover",
    "interpolation": "smooth"
  }
}
```

Live frame producers remain Rust-owned and are not serialized into the
document. A `nullable_image_frame` property binding can provide a bounded
design-time or application-controlled poster frame.

## Verification

```powershell
cargo test --lib --no-default-features --features video
cargo test --lib --no-default-features --features video,ui-document-runtime
```
