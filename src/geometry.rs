use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Px(pub f32);

impl Px {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn round_i32(self) -> i32 {
        self.0.round() as i32
    }

    pub fn ceil_i32(self) -> i32 {
        self.0.ceil() as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dp(pub f32);

impl Dp {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn to_px(self, dpi: Dpi) -> Px {
        Px(self.0 * dpi.scale_factor())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dpi(pub f32);

impl Dpi {
    pub fn new(value: f32) -> Self {
        Self(value.max(1.0))
    }

    pub const fn standard() -> Self {
        Self(96.0)
    }

    pub fn scale_factor(self) -> f32 {
        self.0.max(1.0) / 96.0
    }
}

impl Default for Dpi {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UiLength {
    Auto,
    Fill,
    Fixed(Dp),
}

impl UiLength {
    pub const fn fixed(value: Dp) -> Self {
        Self::Fixed(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl UiRect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn offset_y(self, dy: i32) -> Self {
        Self {
            left: self.left,
            top: self.top - dy,
            right: self.right,
            bottom: self.bottom - dy,
        }
    }

    pub const fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub const fn width(self) -> i32 {
        self.right - self.left
    }

    pub const fn height(self) -> i32 {
        self.bottom - self.top
    }

    pub const fn inflate(self, dx: i32, dy: i32) -> Self {
        Self {
            left: self.left - dx,
            top: self.top - dy,
            right: self.right + dx,
            bottom: self.bottom + dy,
        }
    }
}

#[cfg(all(windows, any(feature = "windows-win32", feature = "windows-gdi")))]
impl From<windows_sys::Win32::Foundation::RECT> for UiRect {
    fn from(value: windows_sys::Win32::Foundation::RECT) -> Self {
        Self {
            left: value.left,
            top: value.top,
            right: value.right,
            bottom: value.bottom,
        }
    }
}

#[cfg(all(windows, any(feature = "windows-win32", feature = "windows-gdi")))]
impl From<&windows_sys::Win32::Foundation::RECT> for UiRect {
    fn from(value: &windows_sys::Win32::Foundation::RECT) -> Self {
        Self::from(*value)
    }
}

#[cfg(all(windows, any(feature = "windows-win32", feature = "windows-gdi")))]
impl From<UiRect> for windows_sys::Win32::Foundation::RECT {
    fn from(value: UiRect) -> Self {
        Self {
            left: value.left,
            top: value.top,
            right: value.right,
            bottom: value.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn clamp_non_negative(self) -> Self {
        Self {
            width: if self.width < 0 { 0 } else { self.width },
            height: if self.height < 0 { 0 } else { self.height },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

pub fn clamp_window_pos_to_rect(
    x: i32,
    y: i32,
    bounds: UiRect,
    win_w: i32,
    win_h: i32,
) -> (i32, i32) {
    let max_x = bounds.left.max(bounds.right - win_w);
    let max_y = bounds.top.max(bounds.bottom - win_h);
    (bounds.left.max(x.min(max_x)), bounds.top.max(y.min(max_y)))
}

pub fn dpi_compensated_size(
    base_w: i32,
    base_h: i32,
    base_monitor_dpi: u32,
    monitor_dpi: u32,
) -> (i32, i32) {
    let base_monitor_dpi = base_monitor_dpi.max(96) as i64;
    let monitor_dpi = monitor_dpi.max(96) as i64;
    let w = (((base_w.max(1) as i64) * base_monitor_dpi) + (monitor_dpi / 2)) / monitor_dpi;
    let h = (((base_h.max(1) as i64) * base_monitor_dpi) + (monitor_dpi / 2)) / monitor_dpi;
    (w.max(1) as i32, h.max(1) as i32)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiCompensationState {
    base_w: i32,
    base_h: i32,
    base_monitor_dpi: u32,
    last_monitor_dpi: u32,
    applying: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiCompensationPlan {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub monitor_dpi: u32,
}

impl DpiCompensationState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub const fn is_applying(self) -> bool {
        self.applying
    }

    pub fn set_applying(&mut self, applying: bool) {
        self.applying = applying;
    }

    pub fn set_base(&mut self, width: i32, height: i32, monitor_dpi: u32) {
        self.base_w = width.max(1);
        self.base_h = height.max(1);
        self.base_monitor_dpi = monitor_dpi.max(96);
        self.last_monitor_dpi = self.base_monitor_dpi;
    }

    pub fn ensure_base(&mut self, width: i32, height: i32, monitor_dpi: u32) -> bool {
        if self.base_monitor_dpi == 0 || self.base_w <= 0 || self.base_h <= 0 {
            self.set_base(width, height, monitor_dpi);
            true
        } else {
            false
        }
    }

    pub fn target_size(self, monitor_dpi: u32) -> Option<(i32, i32)> {
        if self.base_monitor_dpi == 0 || self.base_w <= 0 || self.base_h <= 0 {
            None
        } else {
            Some(dpi_compensated_size(
                self.base_w,
                self.base_h,
                self.base_monitor_dpi,
                monitor_dpi,
            ))
        }
    }

    pub fn already_at_target(
        self,
        monitor_dpi: u32,
        current_w: i32,
        current_h: i32,
        target_w: i32,
        target_h: i32,
        tolerance: i32,
    ) -> bool {
        self.last_monitor_dpi == monitor_dpi.max(96)
            && (current_w - target_w).abs() <= tolerance
            && (current_h - target_h).abs() <= tolerance
    }

    pub fn finish_resize(&mut self, monitor_dpi: u32) {
        self.applying = false;
        self.last_monitor_dpi = monitor_dpi.max(96);
    }

    pub fn resize_plan(
        &mut self,
        current: UiRect,
        bounds: UiRect,
        monitor_dpi: u32,
        tolerance: i32,
    ) -> Option<DpiCompensationPlan> {
        let cur_w = current.right - current.left;
        let cur_h = current.bottom - current.top;
        if cur_w <= 0 || cur_h <= 0 {
            return None;
        }
        let monitor_dpi = monitor_dpi.max(96);
        if self.ensure_base(cur_w, cur_h, monitor_dpi) {
            return None;
        }
        let (mut target_w, mut target_h) = self.target_size(monitor_dpi)?;
        target_w = target_w.min((bounds.right - bounds.left).max(1)).max(1);
        target_h = target_h.min((bounds.bottom - bounds.top).max(1)).max(1);
        if self.already_at_target(monitor_dpi, cur_w, cur_h, target_w, target_h, tolerance) {
            return None;
        }
        let center_x = current.left + cur_w / 2;
        let center_y = current.top + cur_h / 2;
        let (x, y) = clamp_window_pos_to_rect(
            center_x - target_w / 2,
            center_y - target_h / 2,
            bounds,
            target_w,
            target_h,
        );
        Some(DpiCompensationPlan {
            x,
            y,
            width: target_w,
            height: target_h,
            monitor_dpi,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutInput {
    pub bounds: Rect,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutOutput {
    pub bounds: Rect,
    pub children: Vec<LayoutNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutNode {
    pub component: ComponentId,
    pub bounds: Rect,
}

pub trait LayoutProtocol {
    fn layout(&mut self, input: LayoutInput) -> LayoutOutput;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharedUiProtocol {
    Command,
    LayoutProtocol,
    Component,
}

impl SharedUiProtocol {
    pub const fn protocol_name(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::LayoutProtocol => "LayoutProtocol",
            Self::Component => "Component",
        }
    }
}

pub const SHARED_NON_HOST_UI_PROTOCOLS: [SharedUiProtocol; 3] = [
    SharedUiProtocol::Command,
    SharedUiProtocol::LayoutProtocol,
    SharedUiProtocol::Component,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_compensation_keeps_window_inside_bounds() {
        let mut state = DpiCompensationState::default();
        let current = UiRect::new(80, 80, 580, 380);
        let bounds = UiRect::new(0, 0, 600, 400);

        state.set_base(500, 300, 96);
        let plan = state
            .resize_plan(current, bounds, 144, 0)
            .expect("dpi change should produce a resize plan");

        assert_eq!(plan.width, 333);
        assert_eq!(plan.height, 200);
        assert!(bounds.contains(plan.x, plan.y));
        assert!(plan.x + plan.width <= bounds.right);
        assert!(plan.y + plan.height <= bounds.bottom);
    }

    #[test]
    fn dp_px_dpi_units_keep_scaling_explicit() {
        let dpi = Dpi::new(144.0);
        let padding = Dp::new(12.0);
        let px = padding.to_px(dpi);

        assert_eq!(dpi.scale_factor(), 1.5);
        assert_eq!(px.round_i32(), 18);
        assert_eq!(UiLength::fixed(padding), UiLength::Fixed(Dp::new(12.0)));
    }
}
