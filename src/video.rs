use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{NativeDrawImageCommand, NativeImageInterpolation, Rect, ZsImageFrame, ZsuiError};

const DEFAULT_REFRESH_INTERVAL_MS: u64 = 16;
const MIN_REFRESH_INTERVAL_MS: u64 = 8;
const MAX_REFRESH_INTERVAL_MS: u64 = 1_000;

/// Rendering cadence for a [`ZsVideoSource`].
///
/// The source retains only the newest complete frame. Decoding, camera capture,
/// network transport and audio playback stay application-owned so applications
/// can connect the media stack appropriate for their deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsVideoSurfaceConfig {
    refresh_interval_ms: u64,
}

impl ZsVideoSurfaceConfig {
    pub const fn refresh_interval_ms(self) -> u64 {
        self.refresh_interval_ms
    }

    pub fn refresh_interval(mut self, interval: Duration) -> Self {
        let millis = u64::try_from(interval.as_millis()).unwrap_or(u64::MAX);
        self.refresh_interval_ms = millis.clamp(MIN_REFRESH_INTERVAL_MS, MAX_REFRESH_INTERVAL_MS);
        self
    }
}

impl Default for ZsVideoSurfaceConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: DEFAULT_REFRESH_INTERVAL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsVideoPlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Ended,
    Failed,
}

impl ZsVideoPlaybackState {
    pub const fn needs_refresh(self) -> bool {
        matches!(self, Self::Buffering | Self::Playing)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZsVideoSnapshot {
    pub revision: u64,
    pub frame: Option<ZsImageFrame>,
    pub state: ZsVideoPlaybackState,
    pub position: Duration,
    pub presented_frame_count: u64,
    pub last_error: Option<ZsuiError>,
}

#[derive(Debug)]
struct ZsVideoSourceInner {
    revision: u64,
    frame: Option<ZsImageFrame>,
    state: ZsVideoPlaybackState,
    position: Duration,
    presented_frame_count: u64,
    last_error: Option<ZsuiError>,
}

/// A cloneable latest-frame bridge for camera previews and decoded video.
///
/// Producer threads call [`present`](Self::present). The View owns another
/// clone and paints only the newest immutable frame, so a slow UI does not grow
/// an unbounded frame queue.
#[derive(Clone)]
pub struct ZsVideoSource {
    config: ZsVideoSurfaceConfig,
    inner: Arc<Mutex<ZsVideoSourceInner>>,
}

impl ZsVideoSource {
    pub fn new() -> Self {
        Self::with_config(ZsVideoSurfaceConfig::default())
    }

    pub fn with_config(config: ZsVideoSurfaceConfig) -> Self {
        Self {
            config,
            inner: Arc::new(Mutex::new(ZsVideoSourceInner {
                revision: 0,
                frame: None,
                state: ZsVideoPlaybackState::Idle,
                position: Duration::ZERO,
                presented_frame_count: 0,
                last_error: None,
            })),
        }
    }

    pub fn from_frame(frame: ZsImageFrame) -> Self {
        let source = Self::new();
        source.present(frame, Duration::ZERO);
        source.pause();
        source
    }

    pub fn present(&self, frame: ZsImageFrame, position: Duration) -> u64 {
        let mut inner = self.lock();
        inner.revision = inner.revision.saturating_add(1);
        inner.presented_frame_count = inner.presented_frame_count.saturating_add(1);
        inner.frame = Some(frame);
        inner.state = ZsVideoPlaybackState::Playing;
        inner.position = position;
        inner.last_error = None;
        inner.revision
    }

    pub fn set_buffering(&self) {
        let mut inner = self.lock();
        inner.revision = inner.revision.saturating_add(1);
        inner.state = ZsVideoPlaybackState::Buffering;
        inner.last_error = None;
    }

    pub fn resume(&self) {
        let mut inner = self.lock();
        inner.revision = inner.revision.saturating_add(1);
        inner.state = if inner.frame.is_some() {
            ZsVideoPlaybackState::Playing
        } else {
            ZsVideoPlaybackState::Buffering
        };
        inner.last_error = None;
    }

    pub fn pause(&self) {
        self.set_terminal_state(ZsVideoPlaybackState::Paused, None);
    }

    pub fn finish(&self) {
        self.set_terminal_state(ZsVideoPlaybackState::Ended, None);
    }

    pub fn fail(&self, error: ZsuiError) {
        self.set_terminal_state(ZsVideoPlaybackState::Failed, Some(error));
    }

    pub fn clear(&self) {
        let mut inner = self.lock();
        inner.revision = inner.revision.saturating_add(1);
        inner.frame = None;
        inner.state = ZsVideoPlaybackState::Idle;
        inner.position = Duration::ZERO;
        inner.presented_frame_count = 0;
        inner.last_error = None;
    }

    pub fn snapshot(&self) -> ZsVideoSnapshot {
        let inner = self.lock();
        ZsVideoSnapshot {
            revision: inner.revision,
            frame: inner.frame.clone(),
            state: inner.state,
            position: inner.position,
            presented_frame_count: inner.presented_frame_count,
            last_error: inner.last_error.clone(),
        }
    }

    pub const fn refresh_interval_ms(&self) -> u64 {
        self.config.refresh_interval_ms()
    }

    pub fn needs_refresh(&self) -> bool {
        self.lock().state.needs_refresh()
    }

    fn set_terminal_state(&self, state: ZsVideoPlaybackState, error: Option<ZsuiError>) {
        let mut inner = self.lock();
        inner.revision = inner.revision.saturating_add(1);
        inner.state = state;
        inner.last_error = error;
    }

    fn lock(&self) -> MutexGuard<'_, ZsVideoSourceInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for ZsVideoSource {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ZsVideoSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZsVideoSource")
            .field("config", &self.config)
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsVideoFit {
    Contain,
    Cover,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZsVideoRenderGeometry {
    pub source: Rect,
    pub bounds: Rect,
}

pub fn zs_video_render_geometry(
    frame: &ZsImageFrame,
    bounds: Rect,
    fit: ZsVideoFit,
) -> Option<ZsVideoRenderGeometry> {
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
        ZsVideoFit::Stretch => Some(ZsVideoRenderGeometry {
            source: full_source,
            bounds,
        }),
        ZsVideoFit::Contain => {
            let scale = (f64::from(bounds.width) / f64::from(source_width))
                .min(f64::from(bounds.height) / f64::from(source_height));
            let width = (f64::from(source_width) * scale).round().max(1.0) as i32;
            let height = (f64::from(source_height) * scale).round().max(1.0) as i32;
            Some(ZsVideoRenderGeometry {
                source: full_source,
                bounds: Rect {
                    x: bounds.x + (bounds.width - width) / 2,
                    y: bounds.y + (bounds.height - height) / 2,
                    width,
                    height,
                },
            })
        }
        ZsVideoFit::Cover => {
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
            Some(ZsVideoRenderGeometry { source, bounds })
        }
    }
}

pub fn zs_video_native_draw_command(
    frame: ZsImageFrame,
    bounds: Rect,
    fit: ZsVideoFit,
    interpolation: NativeImageInterpolation,
) -> Option<NativeDrawImageCommand> {
    let geometry = zs_video_render_geometry(&frame, bounds, fit)?;
    Some(
        NativeDrawImageCommand::new(frame, geometry.source, geometry.bounds)
            .interpolation(interpolation),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZsImageFrameId;

    fn frame(id: u64, width: u32, height: u32) -> ZsImageFrame {
        ZsImageFrame::from_rgba8(
            ZsImageFrameId::new(id),
            width,
            height,
            vec![255; width as usize * height as usize * 4],
        )
        .unwrap()
    }

    #[test]
    fn source_retains_only_the_latest_complete_frame() {
        let source = ZsVideoSource::new();
        source.present(frame(1, 2, 1), Duration::from_millis(10));
        source.present(frame(2, 2, 1), Duration::from_millis(20));

        let snapshot = source.snapshot();
        assert_eq!(snapshot.frame.unwrap().id(), ZsImageFrameId::new(2));
        assert_eq!(snapshot.position, Duration::from_millis(20));
        assert_eq!(snapshot.presented_frame_count, 2);
        assert_eq!(snapshot.state, ZsVideoPlaybackState::Playing);
        assert!(source.needs_refresh());

        source.pause();
        assert!(!source.needs_refresh());
    }

    #[test]
    fn render_geometry_supports_contain_cover_and_stretch() {
        let frame = frame(3, 200, 100);
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 100,
        };

        assert_eq!(
            zs_video_render_geometry(&frame, bounds, ZsVideoFit::Contain),
            Some(ZsVideoRenderGeometry {
                source: Rect {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 100,
                },
                bounds: Rect {
                    x: 10,
                    y: 45,
                    width: 100,
                    height: 50,
                },
            })
        );
        assert_eq!(
            zs_video_render_geometry(&frame, bounds, ZsVideoFit::Cover)
                .unwrap()
                .source,
            Rect {
                x: 50,
                y: 0,
                width: 100,
                height: 100,
            }
        );
        assert_eq!(
            zs_video_render_geometry(&frame, bounds, ZsVideoFit::Stretch)
                .unwrap()
                .bounds,
            bounds
        );
    }
}
