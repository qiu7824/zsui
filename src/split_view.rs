use serde::{Deserialize, Serialize};

use crate::{Dp, Dpi, Rect, ZsPlatformStyle};

/// Platform-neutral presentation of a pane beside application content.
///
/// `Adaptive` resolves to an inline split while both panes satisfy their
/// declared width contract and to an overlay at narrower window widths.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsSplitViewDisplayMode {
    #[default]
    Adaptive,
    Inline,
    Overlay,
}

/// Logical side occupied by the pane. The current shared layout uses the
/// left-to-right mapping; retaining logical names keeps future locale-driven
/// direction resolution out of application platform branches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsSplitViewPanePlacement {
    #[default]
    Leading,
    Trailing,
}

/// Explicit application-owned state and platform-neutral size overrides for a
/// two-pane SplitView.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZsSplitViewSpec {
    open: bool,
    display_mode: ZsSplitViewDisplayMode,
    pane_placement: ZsSplitViewPanePlacement,
    pane_width: Option<Dp>,
    minimum_content_width: Option<Dp>,
}

impl ZsSplitViewSpec {
    pub const fn new(open: bool) -> Self {
        Self {
            open,
            display_mode: ZsSplitViewDisplayMode::Adaptive,
            pane_placement: ZsSplitViewPanePlacement::Leading,
            pane_width: None,
            minimum_content_width: None,
        }
    }

    pub const fn open(self) -> bool {
        self.open
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub const fn display_mode(mut self, mode: ZsSplitViewDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    pub const fn pane_placement(mut self, placement: ZsSplitViewPanePlacement) -> Self {
        self.pane_placement = placement;
        self
    }

    pub fn pane_width(mut self, width: Dp) -> Self {
        self.pane_width = width.0.is_finite().then(|| Dp::new(width.0.max(1.0)));
        self
    }

    pub fn minimum_content_width(mut self, width: Dp) -> Self {
        self.minimum_content_width = width.0.is_finite().then(|| Dp::new(width.0.max(0.0)));
        self
    }

    pub const fn mode(self) -> ZsSplitViewDisplayMode {
        self.display_mode
    }

    pub const fn placement(self) -> ZsSplitViewPanePlacement {
        self.pane_placement
    }

    pub const fn requested_pane_width(self) -> Option<Dp> {
        self.pane_width
    }

    pub const fn requested_minimum_content_width(self) -> Option<Dp> {
        self.minimum_content_width
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsSplitViewResolvedMode {
    Inline,
    Overlay,
}

/// Deterministic geometry consumed by layout, painting and hit testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZsSplitViewLayout {
    pub bounds: Rect,
    pub pane: Option<Rect>,
    pub content: Rect,
    pub divider: Option<Rect>,
    pub scrim: Option<Rect>,
    pub mode: ZsSplitViewResolvedMode,
    pub placement: ZsSplitViewPanePlacement,
    pub platform: ZsPlatformStyle,
}

/// Resolves one shared SplitView declaration through the active platform's
/// pane, divider and adaptive-width profile.
pub fn zs_split_view_layout(
    bounds: Rect,
    spec: ZsSplitViewSpec,
    platform: ZsPlatformStyle,
    dpi: Dpi,
) -> ZsSplitViewLayout {
    let profile =
        crate::platform_component_profile::PlatformComponentProfile::for_style(platform).split_view;
    let pane_width = spec
        .requested_pane_width()
        .unwrap_or(profile.preferred_pane_width)
        .to_px(dpi)
        .round_i32()
        .max(1)
        .min(bounds.width.max(0));
    let divider_width = profile
        .divider_width
        .to_px(dpi)
        .round_i32()
        .max(1)
        .min(bounds.width.max(1));
    let minimum_content_width = spec
        .requested_minimum_content_width()
        .unwrap_or(profile.minimum_content_width)
        .to_px(dpi)
        .round_i32()
        .max(0);
    let mode = match spec.mode() {
        ZsSplitViewDisplayMode::Inline => ZsSplitViewResolvedMode::Inline,
        ZsSplitViewDisplayMode::Overlay => ZsSplitViewResolvedMode::Overlay,
        ZsSplitViewDisplayMode::Adaptive => {
            if bounds.width
                >= pane_width
                    .saturating_add(divider_width)
                    .saturating_add(minimum_content_width)
            {
                ZsSplitViewResolvedMode::Inline
            } else {
                ZsSplitViewResolvedMode::Overlay
            }
        }
    };

    if !spec.open() || bounds.width <= 0 || bounds.height <= 0 {
        return ZsSplitViewLayout {
            bounds,
            pane: None,
            content: bounds,
            divider: None,
            scrim: None,
            mode,
            placement: spec.placement(),
            platform,
        };
    }

    let pane_width = match mode {
        ZsSplitViewResolvedMode::Inline => {
            pane_width.min(bounds.width.saturating_sub(divider_width).max(0))
        }
        ZsSplitViewResolvedMode::Overlay => pane_width,
    };
    let pane = match spec.placement() {
        ZsSplitViewPanePlacement::Leading => Rect {
            x: bounds.x,
            y: bounds.y,
            width: pane_width,
            height: bounds.height,
        },
        ZsSplitViewPanePlacement::Trailing => Rect {
            x: bounds
                .x
                .saturating_add(bounds.width)
                .saturating_sub(pane_width),
            y: bounds.y,
            width: pane_width,
            height: bounds.height,
        },
    };
    let divider = (pane_width > 0).then(|| match spec.placement() {
        ZsSplitViewPanePlacement::Leading => Rect {
            x: pane.x.saturating_add(pane.width),
            y: bounds.y,
            width: divider_width.min(bounds.width.saturating_sub(pane.width).max(0)),
            height: bounds.height,
        },
        ZsSplitViewPanePlacement::Trailing => Rect {
            x: pane.x.saturating_sub(divider_width),
            y: bounds.y,
            width: divider_width.min(bounds.width.saturating_sub(pane.width).max(0)),
            height: bounds.height,
        },
    });
    let content = match (mode, spec.placement()) {
        (ZsSplitViewResolvedMode::Overlay, _) => bounds,
        (ZsSplitViewResolvedMode::Inline, ZsSplitViewPanePlacement::Leading) => Rect {
            x: pane
                .x
                .saturating_add(pane.width)
                .saturating_add(divider_width),
            y: bounds.y,
            width: bounds
                .width
                .saturating_sub(pane.width)
                .saturating_sub(divider_width)
                .max(0),
            height: bounds.height,
        },
        (ZsSplitViewResolvedMode::Inline, ZsSplitViewPanePlacement::Trailing) => Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds
                .width
                .saturating_sub(pane.width)
                .saturating_sub(divider_width)
                .max(0),
            height: bounds.height,
        },
    };
    let scrim = (mode == ZsSplitViewResolvedMode::Overlay).then(|| match spec.placement() {
        ZsSplitViewPanePlacement::Leading => Rect {
            x: pane.x.saturating_add(pane.width),
            y: bounds.y,
            width: bounds.width.saturating_sub(pane.width).max(0),
            height: bounds.height,
        },
        ZsSplitViewPanePlacement::Trailing => Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width.saturating_sub(pane.width).max(0),
            height: bounds.height,
        },
    });

    ZsSplitViewLayout {
        bounds,
        pane: Some(pane),
        content,
        divider,
        scrim,
        mode,
        placement: spec.placement(),
        platform,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: Rect = Rect {
        x: 10,
        y: 20,
        width: 800,
        height: 600,
    };

    #[test]
    fn platform_profiles_keep_native_pane_widths_and_one_shared_contract() {
        let spec = ZsSplitViewSpec::new(true);
        let windows = zs_split_view_layout(BOUNDS, spec, ZsPlatformStyle::Windows, Dpi::standard());
        let macos = zs_split_view_layout(BOUNDS, spec, ZsPlatformStyle::Macos, Dpi::standard());
        let gtk = zs_split_view_layout(BOUNDS, spec, ZsPlatformStyle::Gtk, Dpi::standard());

        assert_eq!(windows.mode, ZsSplitViewResolvedMode::Inline);
        assert_eq!(macos.mode, ZsSplitViewResolvedMode::Inline);
        assert_eq!(gtk.mode, ZsSplitViewResolvedMode::Inline);
        assert_eq!(windows.pane.unwrap().width, 296);
        assert_eq!(macos.pane.unwrap().width, 240);
        assert_eq!(gtk.pane.unwrap().width, 260);
    }

    #[test]
    fn adaptive_narrow_layout_overlays_without_reflowing_content() {
        let bounds = Rect {
            width: 520,
            ..BOUNDS
        };
        let layout = zs_split_view_layout(
            bounds,
            ZsSplitViewSpec::new(true),
            ZsPlatformStyle::Gtk,
            Dpi::standard(),
        );

        assert_eq!(layout.mode, ZsSplitViewResolvedMode::Overlay);
        assert_eq!(layout.content, bounds);
        assert_eq!(layout.scrim.unwrap().width, 260);
    }

    #[test]
    fn explicit_trailing_inline_layout_keeps_content_and_divider_disjoint() {
        let layout = zs_split_view_layout(
            BOUNDS,
            ZsSplitViewSpec::new(true)
                .display_mode(ZsSplitViewDisplayMode::Inline)
                .pane_placement(ZsSplitViewPanePlacement::Trailing)
                .pane_width(Dp::new(220.0)),
            ZsPlatformStyle::Macos,
            Dpi::standard(),
        );

        assert_eq!(layout.content.width, 579);
        assert_eq!(layout.divider.unwrap().x, 589);
        assert_eq!(layout.pane.unwrap().x, 590);
        assert!(layout.scrim.is_none());
    }

    #[test]
    fn closed_pane_leaves_content_full_size() {
        let layout = zs_split_view_layout(
            BOUNDS,
            ZsSplitViewSpec::new(false),
            ZsPlatformStyle::Windows,
            Dpi::standard(),
        );
        assert_eq!(layout.content, BOUNDS);
        assert!(layout.pane.is_none());
        assert!(layout.divider.is_none());
        assert!(layout.scrim.is_none());
    }
}
