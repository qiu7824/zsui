use cairo::{Context as CairoContext, LineCap, LineJoin};

use crate::{Color, NativeDrawIconCommand, ZsIcon};

pub(crate) fn draw_symbolic_icon(
    context: &CairoContext,
    command: &NativeDrawIconCommand,
    color: Color,
) {
    let rect = command.bounds;
    if rect.width <= 0 || rect.height <= 0 || context.save().is_err() {
        return;
    }

    context.rectangle(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.width),
        f64::from(rect.height),
    );
    context.clip();
    context.translate(f64::from(rect.x), f64::from(rect.y));
    context.scale(f64::from(rect.width) / 16.0, f64::from(rect.height) / 16.0);
    context.set_source_rgba(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    );
    context.set_line_width(1.35);
    context.set_line_cap(LineCap::Round);
    context.set_line_join(LineJoin::Round);

    draw_normalized_icon(context, command.icon);
    let _ = context.restore();
}

fn draw_normalized_icon(context: &CairoContext, icon: ZsIcon) {
    match icon {
        ZsIcon::App => {
            for (x, y) in [(2.0, 2.0), (9.0, 2.0), (2.0, 9.0), (9.0, 9.0)] {
                rounded_rect(context, RectF::new(x, y, 5.0, 5.0), 1.0);
                fill(context);
            }
        }
        ZsIcon::Calculator => {
            rounded_rect(context, RectF::new(3.0, 1.5, 10.0, 13.0), 1.5);
            stroke(context);
            rounded_rect(context, RectF::new(5.0, 3.5, 6.0, 2.5), 0.5);
            stroke(context);
            for (x, y) in [
                (5.2, 8.5),
                (8.0, 8.5),
                (10.8, 8.5),
                (5.2, 11.5),
                (8.0, 11.5),
                (10.8, 11.5),
            ] {
                context.arc(x, y, 0.65, 0.0, std::f64::consts::TAU);
                fill(context);
            }
        }
        ZsIcon::History => {
            context.arc(8.0, 8.0, 5.2, -2.65, 2.55);
            stroke(context);
            path(context, &[(3.0, 3.8), (3.0, 7.0), (6.1, 7.0)]);
            stroke(context);
            path(context, &[(8.0, 4.8), (8.0, 8.0), (10.5, 9.5)]);
            stroke(context);
        }
        ZsIcon::Backspace => {
            polygon(
                context,
                &[
                    (1.5, 8.0),
                    (5.0, 3.2),
                    (14.2, 3.2),
                    (14.2, 12.8),
                    (5.0, 12.8),
                ],
            );
            stroke(context);
            path(context, &[(7.0, 6.0), (11.0, 10.0)]);
            path(context, &[(11.0, 6.0), (7.0, 10.0)]);
            stroke(context);
        }
        ZsIcon::Add => {
            path(context, &[(8.0, 3.0), (8.0, 13.0)]);
            path(context, &[(3.0, 8.0), (13.0, 8.0)]);
            stroke(context);
        }
        ZsIcon::Search => {
            context.arc(6.8, 6.8, 4.1, 0.0, std::f64::consts::TAU);
            stroke(context);
            path(context, &[(9.8, 9.8), (13.6, 13.6)]);
            stroke(context);
        }
        ZsIcon::Settings => {
            context.arc(8.0, 8.0, 2.25, 0.0, std::f64::consts::TAU);
            stroke(context);
            context.arc(8.0, 8.0, 4.65, 0.0, std::f64::consts::TAU);
            stroke(context);
            for angle in (0..8).map(|index| f64::from(index) * std::f64::consts::FRAC_PI_4) {
                let (sin, cos) = angle.sin_cos();
                path(
                    context,
                    &[
                        (8.0 + cos * 5.1, 8.0 + sin * 5.1),
                        (8.0 + cos * 6.4, 8.0 + sin * 6.4),
                    ],
                );
            }
            stroke(context);
        }
        ZsIcon::Sidebar => {
            rounded_rect(context, RectF::new(1.8, 2.3, 12.4, 11.4), 1.4);
            stroke(context);
            path(context, &[(5.6, 2.7), (5.6, 13.3)]);
            stroke(context);
        }
        ZsIcon::Inspector => {
            rounded_rect(context, RectF::new(2.0, 2.0, 12.0, 12.0), 1.2);
            stroke(context);
            path(context, &[(2.5, 5.3), (13.5, 5.3)]);
            stroke(context);
            context.arc(5.0, 3.7, 0.55, 0.0, std::f64::consts::TAU);
            fill(context);
            path(context, &[(5.0, 8.0), (11.2, 8.0)]);
            path(context, &[(5.0, 10.5), (9.5, 10.5)]);
            stroke(context);
        }
        ZsIcon::More => {
            for x in [3.5, 8.0, 12.5] {
                context.arc(x, 8.0, 1.0, 0.0, std::f64::consts::TAU);
                fill(context);
            }
        }
        ZsIcon::Attach => {
            context.move_to(10.8, 5.0);
            context.curve_to(10.8, 2.6, 7.5, 2.0, 6.0, 3.8);
            context.line_to(2.9, 7.6);
            context.curve_to(0.6, 10.4, 4.5, 14.7, 7.3, 11.8);
            context.line_to(12.8, 6.0);
            context.curve_to(14.6, 4.1, 12.0, 1.5, 10.2, 3.4);
            context.line_to(5.0, 8.9);
            context.curve_to(4.0, 9.9, 5.5, 11.2, 6.4, 10.2);
            context.line_to(10.8, 5.5);
            stroke(context);
        }
        ZsIcon::Enter => {
            path(context, &[(13.0, 3.0), (13.0, 8.2), (4.0, 8.2)]);
            path(context, &[(7.0, 5.2), (4.0, 8.2), (7.0, 11.2)]);
            stroke(context);
        }
        ZsIcon::Send => {
            polygon(context, &[(1.8, 2.2), (14.3, 8.0), (1.8, 13.8), (4.0, 8.0)]);
            stroke(context);
            path(context, &[(4.0, 8.0), (11.0, 8.0)]);
            stroke(context);
        }
        ZsIcon::Stop => {
            rounded_rect(context, RectF::new(3.0, 3.0, 10.0, 10.0), 1.5);
            fill(context);
        }
        ZsIcon::Refresh | ZsIcon::Retry => {
            context.arc(8.0, 8.0, 5.0, -2.7, 2.25);
            stroke(context);
            path(context, &[(3.1, 3.6), (3.1, 7.0), (6.4, 7.0)]);
            stroke(context);
        }
        ZsIcon::Code => {
            path(context, &[(6.2, 3.8), (2.0, 8.0), (6.2, 12.2)]);
            path(context, &[(9.8, 3.8), (14.0, 8.0), (9.8, 12.2)]);
            path(context, &[(9.2, 2.8), (6.8, 13.2)]);
            stroke(context);
        }
        ZsIcon::Tool => {
            context.arc(5.2, 5.0, 3.0, 0.7, 5.1);
            stroke(context);
            path(context, &[(7.0, 7.1), (13.5, 13.0)]);
            stroke(context);
            context.arc(13.0, 13.0, 1.0, 0.0, std::f64::consts::TAU);
            stroke(context);
        }
        ZsIcon::Check => draw_check(context, 2.5, 5.2, 13.3),
        ZsIcon::Info => {
            circle_outline(context);
            context.arc(8.0, 4.8, 0.75, 0.0, std::f64::consts::TAU);
            fill(context);
            path(context, &[(8.0, 7.2), (8.0, 11.2)]);
            stroke(context);
        }
        ZsIcon::Success => {
            circle_outline(context);
            draw_check(context, 4.0, 6.3, 12.0);
        }
        ZsIcon::Warning => {
            polygon(context, &[(8.0, 1.8), (14.2, 13.2), (1.8, 13.2)]);
            stroke(context);
            path(context, &[(8.0, 5.2), (8.0, 9.2)]);
            stroke(context);
            context.arc(8.0, 11.4, 0.65, 0.0, std::f64::consts::TAU);
            fill(context);
        }
        ZsIcon::Error => {
            circle_outline(context);
            path(context, &[(5.3, 5.3), (10.7, 10.7)]);
            path(context, &[(10.7, 5.3), (5.3, 10.7)]);
            stroke(context);
        }
        ZsIcon::Minimize => {
            path(context, &[(3.0, 11.0), (13.0, 11.0)]);
            stroke(context);
        }
        ZsIcon::Close => {
            path(context, &[(3.5, 3.5), (12.5, 12.5)]);
            path(context, &[(12.5, 3.5), (3.5, 12.5)]);
            stroke(context);
        }
        ZsIcon::Text => {
            path(context, &[(3.0, 3.0), (13.0, 3.0)]);
            path(context, &[(8.0, 3.0), (8.0, 13.0)]);
            path(context, &[(5.5, 13.0), (10.5, 13.0)]);
            stroke(context);
        }
        ZsIcon::Image => {
            rounded_rect(context, RectF::new(2.0, 2.5, 12.0, 11.0), 1.2);
            stroke(context);
            context.arc(5.2, 5.7, 1.0, 0.0, std::f64::consts::TAU);
            fill(context);
            path(
                context,
                &[
                    (3.0, 12.0),
                    (7.0, 8.0),
                    (9.2, 10.2),
                    (11.0, 8.5),
                    (13.2, 11.0),
                ],
            );
            stroke(context);
        }
        ZsIcon::File => draw_file(context),
        ZsIcon::Folder => {
            context.move_to(1.8, 5.0);
            context.line_to(6.4, 5.0);
            context.line_to(7.6, 6.3);
            context.line_to(14.2, 6.3);
            context.line_to(13.2, 13.2);
            context.line_to(2.8, 13.2);
            context.close_path();
            stroke(context);
            path(context, &[(2.2, 5.0), (2.2, 3.3), (6.2, 3.3), (7.4, 5.0)]);
            stroke(context);
        }
        ZsIcon::Save => {
            rounded_rect(context, RectF::new(2.2, 2.0, 11.6, 12.0), 1.0);
            stroke(context);
            context.rectangle(4.0, 2.2, 6.5, 4.0);
            stroke(context);
            rounded_rect(context, RectF::new(4.2, 9.0, 7.6, 5.0), 0.7);
            stroke(context);
        }
        ZsIcon::Undo => {
            context.arc(8.8, 8.5, 4.5, -2.6, 2.2);
            stroke(context);
            path(context, &[(4.6, 3.6), (4.2, 7.0), (7.5, 6.3)]);
            stroke(context);
        }
        ZsIcon::Cut => {
            for (x, y) in [(4.2, 4.0), (4.2, 12.0)] {
                context.arc(x, y, 2.0, 0.0, std::f64::consts::TAU);
                stroke(context);
            }
            path(context, &[(5.8, 5.2), (13.2, 11.8)]);
            path(context, &[(5.8, 10.8), (13.2, 4.2)]);
            stroke(context);
        }
        ZsIcon::Pin => {
            polygon(
                context,
                &[
                    (5.0, 2.0),
                    (11.0, 2.0),
                    (10.0, 6.2),
                    (12.2, 8.5),
                    (3.8, 8.5),
                    (6.0, 6.2),
                ],
            );
            stroke(context);
            path(context, &[(8.0, 8.7), (8.0, 14.0)]);
            stroke(context);
        }
        ZsIcon::Delete => {
            rounded_rect(context, RectF::new(4.0, 4.7, 8.0, 9.0), 0.8);
            stroke(context);
            path(context, &[(2.8, 4.3), (13.2, 4.3)]);
            path(context, &[(6.0, 2.3), (10.0, 2.3)]);
            path(context, &[(6.5, 7.0), (6.5, 11.5)]);
            path(context, &[(9.5, 7.0), (9.5, 11.5)]);
            stroke(context);
        }
        ZsIcon::Copy => {
            rounded_rect(context, RectF::new(5.0, 5.0, 8.5, 8.5), 1.0);
            stroke(context);
            rounded_rect(context, RectF::new(2.5, 2.5, 8.5, 8.5), 1.0);
            stroke(context);
        }
        ZsIcon::Paste => {
            rounded_rect(context, RectF::new(3.0, 3.2, 10.0, 11.0), 1.0);
            stroke(context);
            rounded_rect(context, RectF::new(5.3, 1.8, 5.4, 3.2), 0.8);
            stroke(context);
            path(context, &[(5.2, 8.0), (10.8, 8.0)]);
            path(context, &[(5.2, 10.8), (9.2, 10.8)]);
            stroke(context);
        }
        ZsIcon::Edit => {
            polygon(
                context,
                &[
                    (3.0, 10.8),
                    (10.8, 3.0),
                    (13.0, 5.2),
                    (5.2, 13.0),
                    (2.4, 13.6),
                ],
            );
            stroke(context);
            path(context, &[(9.6, 4.2), (11.8, 6.4)]);
            stroke(context);
        }
        ZsIcon::Group => {
            context.arc(8.0, 5.0, 2.2, 0.0, std::f64::consts::TAU);
            stroke(context);
            context.arc(3.7, 6.3, 1.6, 0.0, std::f64::consts::TAU);
            context.arc(12.3, 6.3, 1.6, 0.0, std::f64::consts::TAU);
            stroke(context);
            context.arc(8.0, 13.2, 4.2, 3.35, 6.08);
            stroke(context);
            context.arc(3.8, 12.6, 2.7, 3.5, 5.75);
            context.arc(12.2, 12.6, 2.7, 3.67, 5.92);
            stroke(context);
        }
        ZsIcon::Phrase => {
            rounded_rect(context, RectF::new(1.8, 2.5, 12.4, 9.0), 1.8);
            stroke(context);
            path(context, &[(5.0, 11.5), (4.0, 14.0), (8.0, 11.5)]);
            stroke(context);
            path(context, &[(4.5, 6.0), (11.5, 6.0)]);
            path(context, &[(4.5, 8.5), (9.5, 8.5)]);
            stroke(context);
        }
        ZsIcon::ChevronUp => draw_chevron(context, &[(3.5, 10.0), (8.0, 5.5), (12.5, 10.0)]),
        ZsIcon::ChevronDown => draw_chevron(context, &[(3.5, 6.0), (8.0, 10.5), (12.5, 6.0)]),
        ZsIcon::Calendar => {
            rounded_rect(context, RectF::new(2.0, 3.2, 12.0, 10.8), 1.3);
            stroke(context);
            path(context, &[(2.5, 6.3), (13.5, 6.3)]);
            path(context, &[(5.0, 1.8), (5.0, 4.5)]);
            path(context, &[(11.0, 1.8), (11.0, 4.5)]);
            stroke(context);
            for (x, y) in [
                (5.0, 9.0),
                (8.0, 9.0),
                (11.0, 9.0),
                (5.0, 12.0),
                (8.0, 12.0),
            ] {
                context.arc(x, y, 0.55, 0.0, std::f64::consts::TAU);
                fill(context);
            }
        }
        ZsIcon::ChevronLeft => draw_chevron(context, &[(10.0, 3.5), (5.5, 8.0), (10.0, 12.5)]),
        ZsIcon::ChevronRight => draw_chevron(context, &[(6.0, 3.5), (10.5, 8.0), (6.0, 12.5)]),
        ZsIcon::PasswordReveal => {
            context.move_to(1.5, 8.0);
            context.curve_to(4.0, 3.8, 12.0, 3.8, 14.5, 8.0);
            context.curve_to(12.0, 12.2, 4.0, 12.2, 1.5, 8.0);
            context.close_path();
            stroke(context);
            context.arc(8.0, 8.0, 2.2, 0.0, std::f64::consts::TAU);
            stroke(context);
        }
    }
}

fn draw_check(context: &CairoContext, left: f64, middle: f64, right: f64) {
    path(context, &[(left, 8.0), (middle, 11.0), (right, 4.5)]);
    stroke(context);
}

fn draw_file(context: &CairoContext) {
    path(
        context,
        &[
            (3.0, 1.8),
            (9.5, 1.8),
            (13.0, 5.3),
            (13.0, 14.2),
            (3.0, 14.2),
            (3.0, 1.8),
        ],
    );
    stroke(context);
    path(context, &[(9.5, 2.0), (9.5, 5.3), (12.8, 5.3)]);
    stroke(context);
}

fn circle_outline(context: &CairoContext) {
    context.arc(8.0, 8.0, 6.0, 0.0, std::f64::consts::TAU);
    stroke(context);
}

fn draw_chevron(context: &CairoContext, points: &[(f64, f64)]) {
    context.set_line_width(1.7);
    path(context, points);
    stroke(context);
}

fn path(context: &CairoContext, points: &[(f64, f64)]) {
    if let Some((first, rest)) = points.split_first() {
        context.move_to(first.0, first.1);
        for point in rest {
            context.line_to(point.0, point.1);
        }
    }
}

fn polygon(context: &CairoContext, points: &[(f64, f64)]) {
    path(context, points);
    context.close_path();
}

fn rounded_rect(context: &CairoContext, rect: RectF, radius: f64) {
    let radius = radius.min(rect.width / 2.0).min(rect.height / 2.0);
    context.new_sub_path();
    context.arc(
        rect.right() - radius,
        rect.y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    context.arc(
        rect.right() - radius,
        rect.bottom() - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    context.arc(
        rect.x + radius,
        rect.bottom() - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    context.arc(
        rect.x + radius,
        rect.y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    context.close_path();
}

fn stroke(context: &CairoContext) {
    let _ = context.stroke();
}

fn fill(context: &CairoContext) {
    let _ = context.fill();
}

#[derive(Clone, Copy)]
struct RectF {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl RectF {
    const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn right(self) -> f64 {
        self.x + self.width
    }

    fn bottom(self) -> f64 {
        self.y + self.height
    }
}
