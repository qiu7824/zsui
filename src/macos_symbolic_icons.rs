use super::*;
use crate::symbolic_icons::{draw_normalized_icon, SymbolicIconCanvas};

pub(super) fn draw(command: &NativeDrawIconCommand, color: crate::Color) {
    let Some(context) = NSGraphicsContext::currentContext() else {
        return;
    };
    let bounds = command.fitted_bounds(crate::Size {
        width: 16,
        height: 16,
    });
    if bounds.width <= 0 || bounds.height <= 0 {
        return;
    }
    context.saveGraphicsState();
    NSBezierPath::bezierPathWithRect(appkit_rect(bounds)).addClip();
    appkit_color(color).set();
    let path = NSBezierPath::bezierPath();
    let scale = f64::from(bounds.width) / 16.0;
    path.setLineWidth(1.35 * scale);
    path.setLineCapStyle(objc2_app_kit::NSLineCapStyle::Round);
    path.setLineJoinStyle(objc2_app_kit::NSLineJoinStyle::Round);
    let mut canvas = Canvas {
        path,
        bounds,
        scale,
        has_current: false,
    };
    draw_normalized_icon(&mut canvas, command.icon);
    context.restoreGraphicsState();
}

struct Canvas {
    path: Retained<NSBezierPath>,
    bounds: Rect,
    scale: f64,
    has_current: bool,
}
impl Canvas {
    fn point(&self, x: f64, y: f64) -> objc2_foundation::NSPoint {
        objc2_foundation::NSPoint::new(
            f64::from(self.bounds.x) + x * self.scale,
            f64::from(self.bounds.y) + y * self.scale,
        )
    }
    fn reset(&mut self) {
        self.path.removeAllPoints();
        self.has_current = false;
    }
}

impl SymbolicIconCanvas for Canvas {
    fn new_sub_path(&mut self) {
        self.has_current = false;
    }
    fn move_to(&mut self, x: f64, y: f64) {
        self.path.moveToPoint(self.point(x, y));
        self.has_current = true;
    }
    fn line_to(&mut self, x: f64, y: f64) {
        if self.has_current {
            self.path.lineToPoint(self.point(x, y));
        } else {
            self.move_to(x, y);
        }
    }
    fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.path.curveToPoint_controlPoint1_controlPoint2(
            self.point(x, y),
            self.point(x1, y1),
            self.point(x2, y2),
        );
        self.has_current = true;
    }
    fn arc(&mut self, x: f64, y: f64, radius: f64, start: f64, end: f64) {
        crate::symbolic_icons::trace_arc(self, x, y, radius, start, end);
    }
    fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close_path();
    }
    fn set_line_width(&mut self, width: f64) {
        self.path.setLineWidth(width * self.scale);
    }
    fn close_path(&mut self) {
        self.path.closePath();
    }
    fn stroke(&mut self) {
        self.path.stroke();
        self.reset();
    }
    fn fill(&mut self) {
        self.path.fill();
        self.reset();
    }
}
