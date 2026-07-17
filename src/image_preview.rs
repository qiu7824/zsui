use std::{
    io::Cursor,
    sync::{Arc, Condvar, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    NativeDrawImageCommand, NativeImageInterpolation, Rect, ZsImageFrame, ZsImageFrameId,
    ZsuiError, ZsuiResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsImagePreviewConfig {
    pub max_decoded_bytes: usize,
}

impl Default for ZsImagePreviewConfig {
    fn default() -> Self {
        Self {
            max_decoded_bytes: 128 * 1024 * 1024,
        }
    }
}

impl ZsImagePreviewConfig {
    pub fn max_decoded_bytes(mut self, bytes: usize) -> Self {
        self.max_decoded_bytes = bytes.max(4);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZsImagePreviewSnapshot {
    pub generation: u64,
    pub frame: Option<ZsImageFrame>,
    pub loading: bool,
    pub last_error: Option<ZsuiError>,
}

#[derive(Debug)]
struct DecodeJob {
    generation: u64,
    frame_id: ZsImageFrameId,
    png: Arc<[u8]>,
}

#[derive(Debug)]
struct PreviewInner {
    config: ZsImagePreviewConfig,
    generation: u64,
    frame: Option<ZsImageFrame>,
    loading: bool,
    last_error: Option<ZsuiError>,
    pending: Option<DecodeJob>,
    shutdown: bool,
}

#[derive(Debug)]
struct PreviewShared {
    inner: Mutex<PreviewInner>,
    wake: Condvar,
}

pub struct ZsImagePreviewState {
    shared: Arc<PreviewShared>,
    worker: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for ZsImagePreviewState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.lock();
        formatter
            .debug_struct("ZsImagePreviewState")
            .field("generation", &inner.generation)
            .field("frame", &inner.frame)
            .field("loading", &inner.loading)
            .field("last_error", &inner.last_error)
            .finish_non_exhaustive()
    }
}

impl ZsImagePreviewState {
    pub fn new(config: ZsImagePreviewConfig) -> ZsuiResult<Self> {
        let shared = Arc::new(PreviewShared {
            inner: Mutex::new(PreviewInner {
                config,
                generation: 0,
                frame: None,
                loading: false,
                last_error: None,
                pending: None,
                shutdown: false,
            }),
            wake: Condvar::new(),
        });
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("zsui-image-preview".to_string())
            .spawn(move || image_preview_worker(worker_shared))
            .map_err(|error| ZsuiError::host("spawn_image_preview_worker", error.to_string()))?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }

    pub fn set_png(&mut self, frame_id: ZsImageFrameId, png: impl Into<Arc<[u8]>>) -> u64 {
        let mut inner = self.lock();
        inner.generation = inner.generation.saturating_add(1);
        let generation = inner.generation;
        inner.pending = Some(DecodeJob {
            generation,
            frame_id,
            png: png.into(),
        });
        inner.loading = true;
        inner.last_error = None;
        drop(inner);
        self.shared.wake.notify_one();
        generation
    }

    pub fn clear(&mut self) {
        let mut inner = self.lock();
        inner.generation = inner.generation.saturating_add(1);
        inner.frame = None;
        inner.loading = false;
        inner.last_error = None;
        inner.pending = None;
    }

    pub fn snapshot(&self) -> ZsImagePreviewSnapshot {
        let inner = self.lock();
        ZsImagePreviewSnapshot {
            generation: inner.generation,
            frame: inner.frame.clone(),
            loading: inner.loading,
            last_error: inner.last_error.clone(),
        }
    }

    pub fn wait_for_idle(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if !self.snapshot().loading {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn lock(&self) -> MutexGuard<'_, PreviewInner> {
        self.shared
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for ZsImagePreviewState {
    fn default() -> Self {
        Self::new(ZsImagePreviewConfig::default())
            .expect("the image preview decoder worker should start")
    }
}

impl Drop for ZsImagePreviewState {
    fn drop(&mut self) {
        {
            let mut inner = self.lock();
            inner.shutdown = true;
            inner.pending = None;
        }
        self.shared.wake.notify_one();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn image_preview_worker(shared: Arc<PreviewShared>) {
    loop {
        let (job, max_decoded_bytes) = {
            let mut inner = shared
                .inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            while inner.pending.is_none() && !inner.shutdown {
                inner = shared
                    .wake
                    .wait(inner)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
            if inner.shutdown {
                return;
            }
            (
                inner.pending.take().expect("pending decode job"),
                inner.config.max_decoded_bytes,
            )
        };

        let result = decode_png_frame(job.frame_id, &job.png, max_decoded_bytes);
        let mut inner = shared
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if job.generation != inner.generation {
            continue;
        }
        match result {
            Ok(frame) => {
                inner.frame = Some(frame);
                inner.last_error = None;
            }
            Err(error) => inner.last_error = Some(error),
        }
        inner.loading = inner.pending.is_some();
    }
}

fn decode_png_frame(
    frame_id: ZsImageFrameId,
    png: &[u8],
    max_decoded_bytes: usize,
) -> ZsuiResult<ZsImageFrame> {
    let mut decoder = png::Decoder::new(Cursor::new(png));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|error| ZsuiError::invalid_spec("image_preview.png", error.to_string()))?;
    let header = reader.info();
    let decoded_bytes = usize::try_from(header.width)
        .ok()
        .and_then(|width| {
            usize::try_from(header.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            ZsuiError::invalid_spec("image_preview.png", "decoded dimensions overflow")
        })?;
    if decoded_bytes > max_decoded_bytes {
        return Err(ZsuiError::invalid_spec(
            "image_preview.png",
            format!("decoded image requires {decoded_bytes} bytes; limit is {max_decoded_bytes}"),
        ));
    }

    let output_size = reader.output_buffer_size().ok_or_else(|| {
        ZsuiError::invalid_spec("image_preview.png", "PNG output size is unavailable")
    })?;
    let mut decoded = vec![0; output_size];
    let info = reader
        .next_frame(&mut decoded)
        .map_err(|error| ZsuiError::invalid_spec("image_preview.png", error.to_string()))?;
    let bytes = &decoded[..info.buffer_size()];
    let mut rgba = Vec::with_capacity(decoded_bytes);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(bytes),
        png::ColorType::Rgb => {
            for pixel in bytes.chunks_exact(3) {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for value in bytes {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for pixel in bytes.chunks_exact(2) {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        png::ColorType::Indexed => {
            return Err(ZsuiError::invalid_spec(
                "image_preview.png",
                "indexed PNG data was not expanded by the decoder",
            ));
        }
    }
    ZsImageFrame::from_rgba8(frame_id, info.width, info.height, rgba)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZsImageFit {
    Contain,
    Cover,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsImageRenderGeometry {
    pub source: Rect,
    pub bounds: Rect,
}

pub fn zs_image_render_geometry(
    frame: &ZsImageFrame,
    bounds: Rect,
    fit: ZsImageFit,
) -> Option<ZsImageRenderGeometry> {
    if bounds.width <= 0 || bounds.height <= 0 || frame.width() == 0 || frame.height() == 0 {
        return None;
    }
    let source_width = i32::try_from(frame.width()).ok()?;
    let source_height = i32::try_from(frame.height()).ok()?;
    let full_source = Rect {
        x: 0,
        y: 0,
        width: source_width,
        height: source_height,
    };
    match fit {
        ZsImageFit::Stretch => Some(ZsImageRenderGeometry {
            source: full_source,
            bounds,
        }),
        ZsImageFit::Contain => {
            let scale = (f64::from(bounds.width) / f64::from(source_width))
                .min(f64::from(bounds.height) / f64::from(source_height));
            let width = (f64::from(source_width) * scale).round().max(1.0) as i32;
            let height = (f64::from(source_height) * scale).round().max(1.0) as i32;
            Some(ZsImageRenderGeometry {
                source: full_source,
                bounds: Rect {
                    x: bounds.x + (bounds.width - width) / 2,
                    y: bounds.y + (bounds.height - height) / 2,
                    width,
                    height,
                },
            })
        }
        ZsImageFit::Cover => {
            let destination_ratio = f64::from(bounds.width) / f64::from(bounds.height);
            let source_ratio = f64::from(source_width) / f64::from(source_height);
            let source = if source_ratio > destination_ratio {
                let width = (f64::from(source_height) * destination_ratio)
                    .round()
                    .clamp(1.0, f64::from(source_width)) as i32;
                Rect {
                    x: (source_width - width) / 2,
                    y: 0,
                    width,
                    height: source_height,
                }
            } else {
                let height = (f64::from(source_width) / destination_ratio)
                    .round()
                    .clamp(1.0, f64::from(source_height)) as i32;
                Rect {
                    x: 0,
                    y: (source_height - height) / 2,
                    width: source_width,
                    height,
                }
            };
            Some(ZsImageRenderGeometry { source, bounds })
        }
    }
}

pub fn zs_image_native_draw_command(
    frame: ZsImageFrame,
    bounds: Rect,
    fit: ZsImageFit,
    interpolation: NativeImageInterpolation,
) -> Option<NativeDrawImageCommand> {
    let geometry = zs_image_render_geometry(&frame, bounds, fit)?;
    Some(
        NativeDrawImageCommand::new(frame, geometry.source, geometry.bounds)
            .interpolation(interpolation),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32, rgba: &[u8]) -> Arc<[u8]> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(rgba).unwrap();
        }
        Arc::from(bytes)
    }

    #[test]
    fn loading_snapshot_never_exposes_a_partial_frame_during_atomic_swap() {
        let mut preview = ZsImagePreviewState::default();
        preview.set_png(ZsImageFrameId::new(1), png(1, 1, &[255, 0, 0, 255]));
        assert!(preview.wait_for_idle(Duration::from_secs(1)));
        assert_eq!(
            preview.snapshot().frame.unwrap().id(),
            ZsImageFrameId::new(1)
        );

        preview.set_png(ZsImageFrameId::new(2), png(1, 1, &[0, 255, 0, 255]));
        let transition = preview.snapshot();
        // The decoder is intentionally asynchronous. A one-pixel PNG can
        // finish before this thread reacquires the mutex, so scheduling is
        // not part of the contract: while loading we retain frame 1, and if
        // the atomic swap already completed we expose the complete frame 2.
        assert_eq!(
            transition.frame.unwrap().id(),
            if transition.loading {
                ZsImageFrameId::new(1)
            } else {
                ZsImageFrameId::new(2)
            }
        );
        assert!(transition.last_error.is_none());
        assert!(preview.wait_for_idle(Duration::from_secs(1)));
        assert_eq!(
            preview.snapshot().frame.unwrap().id(),
            ZsImageFrameId::new(2)
        );
    }

    #[test]
    fn cover_and_contain_geometry_preserve_aspect_ratio() {
        let frame =
            ZsImageFrame::from_rgba8(ZsImageFrameId::new(1), 400, 200, vec![255; 400 * 200 * 4])
                .unwrap();
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let contain = zs_image_render_geometry(&frame, bounds, ZsImageFit::Contain).unwrap();
        assert_eq!(contain.bounds.height, 50);
        assert_eq!(contain.bounds.y, 25);
        let cover = zs_image_render_geometry(&frame, bounds, ZsImageFit::Cover).unwrap();
        assert_eq!(cover.source.width, 200);
        assert_eq!(cover.source.x, 100);
    }

    #[test]
    fn decoded_byte_limit_rejects_large_frames_before_allocation() {
        let bytes = png(2, 2, &[255; 16]);
        let error = decode_png_frame(ZsImageFrameId::new(1), &bytes, 8).unwrap_err();
        assert!(matches!(error, ZsuiError::InvalidSpec { .. }));
    }
}
