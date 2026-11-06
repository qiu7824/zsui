//! Optional native-window integration policies.

use serde::{Deserialize, Serialize};

/// Actual system caption reservation expressed in client-area physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionMetrics {
    /// The rectangle occupied by system caption buttons.
    pub buttons: crate::Rect,
    /// Window scale at which the rectangle was measured.
    pub dpi: u32,
    /// Whether the requested system backdrop was accepted.
    pub backdrop_active: bool,
}

impl CaptionMetrics {
    /// Width that View content must reserve in logical units.
    pub fn reserved_width(self) -> crate::Dp {
        crate::Dp::new(self.buttons.width.max(0) as f32 * 96.0 / self.dpi.max(96) as f32)
    }
}

/// Typed notifications emitted by the native window's integration services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeWindowEvent {
    /// A registered system-wide shortcut fired.
    GlobalHotkey(crate::Command),
    /// The active system/theme palette changed.
    ThemeChanged(crate::PlatformPalette),
    /// Caption geometry changed after creation, resizing, or a DPI transition.
    CaptionChanged(CaptionMetrics),
    /// Desktop integration was re-established after the system shell restarted.
    DesktopRecreated,
    /// A recoverable native service operation failed.
    Error { operation: String, message: String },
}

/// The backdrop requested for an integrated title bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowBackdrop {
    /// An opaque, theme-resolved background.
    #[default]
    Solid,
    /// The system's Mica backdrop, with an opaque fallback when unavailable.
    Mica,
}

/// A system-caption title bar whose remaining area belongs to the View.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegratedTitleBar {
    height_milli_dp: u32,
    /// The desired system backdrop.
    pub backdrop: WindowBackdrop,
}

impl IntegratedTitleBar {
    /// Creates a title bar with a finite height between 24 and 256 DP.
    pub fn new(height: crate::Dp) -> crate::ZsuiResult<Self> {
        if !height.0.is_finite() || !(24.0..=256.0).contains(&height.0) {
            return Err(crate::ZsuiError::invalid_spec(
                "title_bar.height",
                "height must be between 24 and 256 DP",
            ));
        }
        Ok(Self {
            height_milli_dp: (height.0 * 1000.0).round() as u32,
            backdrop: WindowBackdrop::Solid,
        })
    }

    /// Requests a system backdrop with an opaque fallback.
    pub const fn backdrop(mut self, backdrop: WindowBackdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// Returns the logical height of the title bar.
    pub fn height(self) -> crate::Dp {
        crate::Dp::new(self.height_milli_dp as f32 / 1000.0)
    }
}

impl Default for IntegratedTitleBar {
    fn default() -> Self {
        Self {
            height_milli_dp: 40_000,
            backdrop: WindowBackdrop::Solid,
        }
    }
}

/// How a per-pixel alpha surface participates in pointer hit testing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaHitTest {
    /// Fully transparent pixels pass pointer input to the underlying window.
    #[default]
    TransparentPixels,
    /// All pixels pass pointer input to the underlying window.
    PassThrough,
}

/// The presentation surface used by a native window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowSurface {
    /// The normal buffered, opaque window surface.
    #[default]
    Opaque,
    /// A premultiplied-alpha surface presented atomically by the host.
    PerPixelAlpha { hit_test: AlphaHitTest },
}

/// The native stacking and activation policy of a window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowLayer {
    /// A normal application window.
    #[default]
    Normal,
    /// A non-activating desktop surface, below normal application windows.
    Desktop { follow_show_desktop: bool },
}

/// Windows rasterization policy. The default preserves the GDI paint path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowRenderer {
    /// Buffered GDI/GDI+ with the shared text engine.
    #[default]
    Gdi,
    /// Direct2D's software render target.
    Direct2dSoftware,
    /// Direct2D's hardware render target, explicitly requested by the application.
    Direct2dHardware,
}

/// Declarative integration options; native handles never enter this structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowIntegration {
    /// Optional integrated system title bar.
    pub title_bar: Option<IntegratedTitleBar>,
    /// Surface alpha and hit-testing policy.
    pub surface: WindowSurface,
    /// Window stacking policy.
    pub layer: WindowLayer,
    /// Rasterization policy.
    pub renderer: WindowRenderer,
}

impl WindowIntegration {
    pub(crate) fn validate(&self, decorations: bool, always_on_top: bool) -> crate::ZsuiResult<()> {
        if let Some(title) = self.title_bar {
            IntegratedTitleBar::new(title.height())?;
            if !decorations || self.surface != WindowSurface::Opaque {
                return Err(crate::ZsuiError::invalid_spec(
                    "title_bar",
                    "integrated system captions require an opaque decorated window",
                ));
            }
        }
        if matches!(self.layer, WindowLayer::Desktop { .. }) && (decorations || always_on_top) {
            return Err(crate::ZsuiError::invalid_spec(
                "window.layer",
                "desktop windows must be undecorated and cannot be always-on-top",
            ));
        }
        if self.surface != WindowSurface::Opaque && decorations {
            return Err(crate::ZsuiError::invalid_spec(
                "window.surface",
                "per-pixel alpha windows must be undecorated",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn title_height_and_surface_combinations_are_validated() {
        for height in [f32::NAN, f32::INFINITY, -1.0, 0.0, 257.0] {
            assert!(IntegratedTitleBar::new(crate::Dp::new(height)).is_err());
        }
        let mut options = WindowIntegration {
            title_bar: Some(IntegratedTitleBar::default()),
            ..Default::default()
        };
        assert!(options.validate(true, false).is_ok());
        assert!(options.validate(false, false).is_err());
        options.surface = WindowSurface::PerPixelAlpha {
            hit_test: AlphaHitTest::TransparentPixels,
        };
        assert!(options.validate(true, false).is_err());
        options.title_bar = None;
        options.layer = WindowLayer::Desktop {
            follow_show_desktop: true,
        };
        assert!(options.validate(false, false).is_ok());
        assert!(options.validate(false, true).is_err());
    }
}
