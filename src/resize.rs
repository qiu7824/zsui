use serde::{Deserialize, Serialize};

use crate::{Dp, Dpi, Point, Rect, ZsuiError, ZsuiResult};

/// Axis along which a divider changes its controlled extent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZsResizeAxis {
    /// Changes width using horizontal pointer motion.
    #[default]
    Horizontal,
    /// Changes height using vertical pointer motion.
    Vertical,
}

impl ZsResizeAxis {
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
    pub(crate) const fn coordinate(self, point: Point) -> i32 {
        match self {
            Self::Horizontal => point.x,
            Self::Vertical => point.y,
        }
    }

    pub(crate) const fn extent(self, bounds: Rect) -> i32 {
        match self {
            Self::Horizontal => bounds.width,
            Self::Vertical => bounds.height,
        }
    }
}

/// Finite logical limits for a user-adjustable width or height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ZsResizeLimits {
    minimum: Dp,
    maximum: Dp,
}

impl ZsResizeLimits {
    pub fn new(minimum: Dp, maximum: Dp) -> ZsuiResult<Self> {
        if !minimum.0.is_finite()
            || !maximum.0.is_finite()
            || minimum.0 < 0.0
            || maximum.0 < minimum.0
        {
            return Err(ZsuiError::invalid_spec(
                "resize_limits",
                "limits must be finite, nonnegative and ordered",
            ));
        }
        Ok(Self { minimum, maximum })
    }

    pub const fn minimum(self) -> Dp {
        self.minimum
    }
    pub const fn maximum(self) -> Dp {
        self.maximum
    }

    pub fn clamp(self, extent: Dp) -> Dp {
        Dp::new(if extent.0.is_finite() {
            extent.0.clamp(self.minimum.0, self.maximum.0)
        } else {
            self.minimum.0
        })
    }
}

impl Default for ZsResizeLimits {
    fn default() -> Self {
        Self {
            minimum: Dp::new(48.0),
            maximum: Dp::new(4096.0),
        }
    }
}

impl<'de> Deserialize<'de> for ZsResizeLimits {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Values {
            minimum: Dp,
            maximum: Dp,
        }
        let values = Values::deserialize(deserializer)?;
        Self::new(values.minimum, values.maximum).map_err(serde::de::Error::custom)
    }
}

/// Stable owner-relative identity of a resizable surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsResizeTarget {
    Pane,
    #[cfg(feature = "table")]
    TableColumn(crate::ZsTableColumnId),
}

/// Immutable pixel geometry captured when a divider drag starts.
///
/// Keeping the initial pointer coordinate separate from the extent prevents
/// jumping when a wider hit region is pressed away from its visible divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ZsResizeHandle {
    pub(crate) owner: crate::WidgetId,
    pub(crate) target: ZsResizeTarget,
    pub(crate) axis: ZsResizeAxis,
    pub(crate) extent_px: i32,
    pub(crate) minimum_px: i32,
    pub(crate) maximum_px: i32,
    pub(crate) reversed: bool,
}

impl<'de> Deserialize<'de> for ZsResizeHandle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Values {
            owner: crate::WidgetId,
            target: ZsResizeTarget,
            axis: ZsResizeAxis,
            extent_px: i32,
            minimum_px: i32,
            maximum_px: i32,
            reversed: bool,
        }
        let value = Values::deserialize(deserializer)?;
        if value.minimum_px < 0 || value.maximum_px < value.minimum_px || value.extent_px < 0 {
            return Err(serde::de::Error::custom(
                "resize geometry must have nonnegative extents and ordered limits",
            ));
        }
        Ok(Self {
            owner: value.owner,
            target: value.target,
            axis: value.axis,
            extent_px: value.extent_px,
            minimum_px: value.minimum_px,
            maximum_px: value.maximum_px,
            reversed: value.reversed,
        })
    }
}

impl ZsResizeHandle {
    pub(crate) fn new(
        owner: crate::WidgetId,
        target: ZsResizeTarget,
        axis: ZsResizeAxis,
        extent_px: i32,
        limits: ZsResizeLimits,
        available_px: i32,
        reversed: bool,
        dpi: Dpi,
    ) -> Self {
        let maximum_px = limits
            .maximum
            .to_px(dpi)
            .round_i32()
            .min(available_px)
            .max(0);
        let minimum_px = limits.minimum.to_px(dpi).round_i32().clamp(0, maximum_px);
        Self {
            owner,
            target,
            axis,
            extent_px,
            minimum_px,
            maximum_px,
            reversed,
        }
    }

    pub const fn target(self) -> ZsResizeTarget {
        self.target
    }
    pub const fn owner(self) -> crate::WidgetId {
        self.owner
    }
    pub const fn axis(self) -> ZsResizeAxis {
        self.axis
    }
    pub const fn is_adjustable(self) -> bool {
        self.maximum_px > self.minimum_px
    }

    pub(crate) const fn widget_id(self) -> crate::WidgetId {
        let local = match self.target {
            ZsResizeTarget::Pane => 0x7265_7369_7a65_0000,
            #[cfg(feature = "table")]
            ZsResizeTarget::TableColumn(column) => {
                0x7265_7369_7a65_0001 ^ column.get().rotate_left(17)
            }
        };
        crate::WidgetId::synthetic_child(self.owner, local)
    }

    pub(crate) fn extent_at(self, start: Point, current: Point, dpi: Dpi) -> Dp {
        let delta =
            i64::from(self.axis.coordinate(current)) - i64::from(self.axis.coordinate(start));
        let next = i64::from(self.extent_px) + if self.reversed { -delta } else { delta };
        Dp::new(
            next.clamp(i64::from(self.minimum_px), i64::from(self.maximum_px)) as f32
                / dpi.scale_factor(),
        )
    }

    pub(crate) fn step(self, delta: Dp, dpi: Dpi) -> Dp {
        let delta = delta.to_px(dpi).round_i32();
        let next = self
            .extent_px
            .saturating_add(delta)
            .clamp(self.minimum_px, self.maximum_px);
        Dp::new(next as f32 / dpi.scale_factor())
    }
}

/// Wider pointer geometry around a thin divider, clipped to its owner.
pub(crate) fn resize_hit_bounds(divider: Rect, owner: Rect, axis: ZsResizeAxis, dpi: Dpi) -> Rect {
    let hit = Dp::new(8.0).to_px(dpi).round_i32().max(1);
    match axis {
        ZsResizeAxis::Horizontal => {
            let center = divider.x.saturating_add(divider.width / 2);
            let x = center.saturating_sub(hit / 2).max(owner.x);
            Rect {
                x,
                y: divider.y,
                width: hit
                    .min(owner.x.saturating_add(owner.width).saturating_sub(x))
                    .max(0),
                height: divider.height,
            }
        }
        ZsResizeAxis::Vertical => {
            let center = divider.y.saturating_add(divider.height / 2);
            let y = center.saturating_sub(hit / 2).max(owner.y);
            Rect {
                x: divider.x,
                y,
                width: divider.width,
                height: hit
                    .min(owner.y.saturating_add(owner.height).saturating_sub(y))
                    .max(0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_limits_in_builders_and_serialized_input() {
        assert!(ZsResizeLimits::new(Dp::new(f32::NAN), Dp::new(200.0)).is_err());
        assert!(ZsResizeLimits::new(Dp::new(200.0), Dp::new(100.0)).is_err());
        assert!(
            serde_json::from_str::<ZsResizeLimits>(r#"{"minimum":200,"maximum":100}"#).is_err()
        );
        assert!(serde_json::from_str::<ZsResizeHandle>(r#"{"owner":1,"target":"Pane","axis":"horizontal","extent_px":80,"minimum_px":100,"maximum_px":20,"reversed":false}"#).is_err());
    }

    #[test]
    fn captured_geometry_preserves_pointer_offset_direction_and_dpi() {
        let limits = ZsResizeLimits::new(Dp::new(100.0), Dp::new(400.0)).unwrap();
        let start = Point { x: 403, y: 80 };
        let handle = ZsResizeHandle::new(
            crate::WidgetId::new(1),
            ZsResizeTarget::Pane,
            ZsResizeAxis::Horizontal,
            400,
            limits,
            1000,
            false,
            Dpi::new(192.0),
        );
        assert_eq!(
            handle.extent_at(start, start, Dpi::new(192.0)),
            Dp::new(200.0)
        );
        assert_eq!(
            handle.extent_at(start, Point { x: 503, y: 0 }, Dpi::new(192.0)),
            Dp::new(250.0)
        );
        assert_eq!(
            handle.extent_at(start, Point { x: i32::MAX, y: 0 }, Dpi::new(192.0)),
            Dp::new(400.0)
        );
        let reversed = ZsResizeHandle {
            reversed: true,
            ..handle
        };
        assert_eq!(
            reversed.extent_at(start, Point { x: 503, y: 0 }, Dpi::new(192.0)),
            Dp::new(150.0)
        );
    }
}
