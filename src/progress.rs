use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

#[cfg(feature = "progress-ring")]
use crate::{ColorRole, Dp, Dpi, NativeDrawCommand, NativeDrawFill, NativeDrawPlan, Rect};

/// A finite progress range shared by the independently selectable bar and ring.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProgressRange {
    min: f32,
    max: f32,
}

impl ProgressRange {
    pub fn new(min: f32, max: f32) -> Self {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 100.0 };
        let (min, mut max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f32::EPSILON {
            max = min + 1.0;
        }
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub fn clamp(self, value: f32) -> f32 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    pub fn fraction(self, value: f32) -> f32 {
        ((self.clamp(value) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

impl From<RangeInclusive<f32>> for ProgressRange {
    fn from(range: RangeInclusive<f32>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZsProgressRingMode {
    Indeterminate,
    Determinate { value: f32, range: ProgressRange },
}

#[cfg(feature = "progress-ring")]
impl ZsProgressRingMode {
    pub fn determinate(value: f32, range: impl Into<ProgressRange>) -> Self {
        let range = range.into();
        Self::Determinate {
            value: range.clamp(value),
            range,
        }
    }

    pub fn fraction(self) -> Option<f32> {
        match self {
            Self::Indeterminate => None,
            Self::Determinate { value, range } => Some(range.fraction(value)),
        }
    }
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsProgressRingSize {
    Small,
    Medium,
    Large,
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsProgressRingSpec {
    active: bool,
    mode: ZsProgressRingMode,
    size: ZsProgressRingSize,
}

#[cfg(feature = "progress-ring")]
impl ZsProgressRingSpec {
    pub const fn indeterminate() -> Self {
        Self {
            active: true,
            mode: ZsProgressRingMode::Indeterminate,
            size: ZsProgressRingSize::Medium,
        }
    }

    pub fn determinate(value: f32, range: impl Into<ProgressRange>) -> Self {
        Self {
            active: true,
            mode: ZsProgressRingMode::determinate(value, range),
            size: ZsProgressRingSize::Medium,
        }
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn size(mut self, size: ZsProgressRingSize) -> Self {
        self.size = size;
        self
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn mode(self) -> ZsProgressRingMode {
        self.mode
    }

    pub const fn size_value(self) -> ZsProgressRingSize {
        self.size
    }

    pub const fn is_animating(self) -> bool {
        self.active && matches!(self.mode, ZsProgressRingMode::Indeterminate)
    }
}

#[cfg(feature = "progress-ring")]
impl Default for ZsProgressRingSpec {
    fn default() -> Self {
        Self::indeterminate()
    }
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZsProgressRingPlatformStyle {
    Windows,
    Macos,
    Gtk,
}

#[cfg(feature = "progress-ring")]
impl ZsProgressRingPlatformStyle {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Gtk
        } else {
            Self::Windows
        }
    }
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZsProgressRingMetrics {
    pub diameter: Dp,
    pub stroke_width: Dp,
    pub indeterminate_sweep_degrees: i16,
    pub revolution_ms: u64,
    pub indicator_role: ColorRole,
    pub track_role: ColorRole,
}

#[cfg(feature = "progress-ring")]
pub const fn zs_progress_ring_metrics(
    style: ZsProgressRingPlatformStyle,
    size: ZsProgressRingSize,
) -> ZsProgressRingMetrics {
    let (diameter, stroke_width) = match (style, size) {
        (ZsProgressRingPlatformStyle::Windows, ZsProgressRingSize::Small) => (20.0, 2.0),
        (ZsProgressRingPlatformStyle::Windows, ZsProgressRingSize::Medium) => (32.0, 3.0),
        (ZsProgressRingPlatformStyle::Windows, ZsProgressRingSize::Large) => (48.0, 4.0),
        (ZsProgressRingPlatformStyle::Macos, ZsProgressRingSize::Small) => (16.0, 2.0),
        (ZsProgressRingPlatformStyle::Macos, ZsProgressRingSize::Medium) => (20.0, 2.0),
        (ZsProgressRingPlatformStyle::Macos, ZsProgressRingSize::Large) => (32.0, 3.0),
        (ZsProgressRingPlatformStyle::Gtk, ZsProgressRingSize::Small) => (16.0, 2.0),
        (ZsProgressRingPlatformStyle::Gtk, ZsProgressRingSize::Medium) => (24.0, 2.0),
        (ZsProgressRingPlatformStyle::Gtk, ZsProgressRingSize::Large) => (32.0, 3.0),
    };
    ZsProgressRingMetrics {
        diameter: Dp::new(diameter),
        stroke_width: Dp::new(stroke_width),
        indeterminate_sweep_degrees: match style {
            ZsProgressRingPlatformStyle::Windows => 110,
            ZsProgressRingPlatformStyle::Macos => 90,
            ZsProgressRingPlatformStyle::Gtk => 80,
        },
        revolution_ms: match style {
            ZsProgressRingPlatformStyle::Windows => 1_000,
            ZsProgressRingPlatformStyle::Macos => 900,
            ZsProgressRingPlatformStyle::Gtk => 800,
        },
        indicator_role: match style {
            ZsProgressRingPlatformStyle::Windows => ColorRole::Accent,
            ZsProgressRingPlatformStyle::Macos | ZsProgressRingPlatformStyle::Gtk => {
                ColorRole::PrimaryText
            }
        },
        track_role: ColorRole::Control,
    }
}

#[cfg(feature = "progress-ring")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsProgressRingRenderPlan {
    pub active: bool,
    pub ring_bounds: Rect,
    pub stroke_width: i32,
    pub start_degrees: i16,
    pub sweep_degrees: i16,
    pub track_visible: bool,
    pub indicator_role: ColorRole,
    pub track_role: ColorRole,
    pub frame_interval_ms: Option<u64>,
}

#[cfg(feature = "progress-ring")]
pub fn zs_progress_ring_render_plan(
    spec: ZsProgressRingSpec,
    bounds: Rect,
    style: ZsProgressRingPlatformStyle,
    dpi: Dpi,
    elapsed_ms: u64,
) -> ZsProgressRingRenderPlan {
    let metrics = zs_progress_ring_metrics(style, spec.size);
    let preferred = metrics.diameter.to_px(dpi).round_i32().max(1);
    let diameter = bounds.width.min(bounds.height).min(preferred).max(1);
    let stroke_width = metrics
        .stroke_width
        .to_px(dpi)
        .round_i32()
        .clamp(1, diameter.max(1));
    let inset = (stroke_width + 1) / 2;
    let outer = Rect {
        x: bounds.x + (bounds.width - diameter) / 2,
        y: bounds.y + (bounds.height - diameter) / 2,
        width: diameter,
        height: diameter,
    };
    let ring_bounds = Rect {
        x: outer.x + inset,
        y: outer.y + inset,
        width: (outer.width - inset * 2).max(1),
        height: (outer.height - inset * 2).max(1),
    };
    let (start_degrees, sweep_degrees, track_visible, frame_interval_ms) = match spec.mode {
        ZsProgressRingMode::Indeterminate => {
            let rotation =
                ((elapsed_ms % metrics.revolution_ms) * 360 / metrics.revolution_ms) as i16;
            (
                (-90_i16).saturating_add(rotation),
                metrics.indeterminate_sweep_degrees,
                false,
                spec.active.then_some(16),
            )
        }
        ZsProgressRingMode::Determinate { value, range } => (
            -90,
            (range.fraction(value) * 360.0).round().clamp(0.0, 360.0) as i16,
            true,
            None,
        ),
    };
    ZsProgressRingRenderPlan {
        active: spec.active,
        ring_bounds,
        stroke_width,
        start_degrees,
        sweep_degrees,
        track_visible,
        indicator_role: metrics.indicator_role,
        track_role: metrics.track_role,
        frame_interval_ms,
    }
}

#[cfg(feature = "progress-ring")]
pub fn zs_progress_ring_native_draw_plan(plan: &ZsProgressRingRenderPlan) -> NativeDrawPlan {
    if !plan.active {
        return NativeDrawPlan::default();
    }
    let mut commands = Vec::with_capacity(2);
    if plan.track_visible {
        commands.push(NativeDrawCommand::StrokeArc {
            rect: plan.ring_bounds,
            stroke: NativeDrawFill::Role(plan.track_role),
            width: plan.stroke_width,
            start_degrees: -90,
            sweep_degrees: 360,
        });
    }
    if plan.sweep_degrees > 0 {
        commands.push(NativeDrawCommand::StrokeArc {
            rect: plan.ring_bounds,
            stroke: NativeDrawFill::Role(plan.indicator_role),
            width: plan.stroke_width,
            start_degrees: plan.start_degrees,
            sweep_degrees: plan.sweep_degrees,
        });
    }
    NativeDrawPlan::new(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_range_sanitizes_and_clamps_values() {
        let range = ProgressRange::new(100.0, 0.0);
        assert_eq!((range.min(), range.max()), (0.0, 100.0));
        assert_eq!(range.fraction(-10.0), 0.0);
        assert_eq!(range.fraction(125.0), 1.0);
    }

    #[cfg(feature = "progress-ring")]
    #[test]
    fn indeterminate_ring_rotates_deterministically_and_stays_noninteractive() {
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        };
        let start = zs_progress_ring_render_plan(
            ZsProgressRingSpec::indeterminate(),
            bounds,
            ZsProgressRingPlatformStyle::Windows,
            Dpi::standard(),
            0,
        );
        let half = zs_progress_ring_render_plan(
            ZsProgressRingSpec::indeterminate(),
            bounds,
            ZsProgressRingPlatformStyle::Windows,
            Dpi::standard(),
            500,
        );
        assert_eq!(start.start_degrees, -90);
        assert_eq!(half.start_degrees, 90);
        assert_eq!(start.frame_interval_ms, Some(16));
        assert_eq!(zs_progress_ring_native_draw_plan(&start).command_count(), 1);
    }

    #[cfg(feature = "progress-ring")]
    #[test]
    fn determinate_ring_draws_track_and_clamped_value_arc() {
        let plan = zs_progress_ring_render_plan(
            ZsProgressRingSpec::determinate(75.0, ProgressRange::new(0.0, 100.0)),
            Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
            ZsProgressRingPlatformStyle::Macos,
            Dpi::standard(),
            0,
        );
        assert_eq!(plan.sweep_degrees, 270);
        assert_eq!(plan.frame_interval_ms, None);
        assert_eq!(zs_progress_ring_native_draw_plan(&plan).command_count(), 2);
    }
}
