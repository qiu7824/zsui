use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk::prelude::GdkCairoContextExt;
use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk4 as gtk;

use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, NativeStyleResolver, Rect, Size,
    TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};

#[link(name = "pangocairo-1.0")]
unsafe extern "C" {
    fn pango_cairo_show_layout(
        context: *mut gtk::cairo::ffi::cairo_t,
        layout: *mut gtk::pango::ffi::PangoLayout,
    );
}

pub(crate) fn install_linux_gtk_draw_plan(
    window: &gtk::ApplicationWindow,
    plan: NativeDrawPlan,
    runtime: crate::native::NativeViewInputRuntime,
) {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_focusable(true);
    let plan = Rc::new(RefCell::new(plan));
    let runtime = Rc::new(RefCell::new(runtime));
    let ime = gtk::IMMulticontext::new();
    ime.set_client_widget(Some(&drawing_area));
    ime.set_use_preedit(true);
    drawing_area.set_draw_func({
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |area, context, width, height| {
            let resize = runtime.borrow_mut().set_surface(
                Rect {
                    x: 0,
                    y: 0,
                    width: width.max(0),
                    height: height.max(0),
                },
                crate::Dpi::standard(),
            );
            if let Some(updated) = resize.redraw_plan {
                *plan.borrow_mut() = updated;
            }
            if resize.surface_changed {
                sync_linux_gtk_ime(area, &runtime, &ime);
            }
            let plan = plan.borrow();
            let system_prefers_dark = gtk::Settings::default()
                .map(|settings| settings.is_gtk_application_prefer_dark_theme())
                .unwrap_or(false);
            let palette = NativeDrawPalette::for_mode(plan.theme_mode, system_prefers_dark);
            let mut sink = LinuxGtkDrawSink::new(area, context, palette);
            sink.draw_plan(&plan);
        }
    });
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_PRIMARY);
    gesture.connect_pressed({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |gesture, _press_count, x, y| {
            let shift = gesture
                .current_event_state()
                .contains(gtk::gdk::ModifierType::SHIFT_MASK);
            let report = runtime.borrow_mut().dispatch_pointer_down(
                crate::Point {
                    x: gtk_coordinate(x),
                    y: gtk_coordinate(y),
                },
                shift,
            );
            if report.handled {
                area.grab_focus();
            }
            apply_linux_gtk_input_report(
                report,
                &area,
                &plan,
                &runtime,
                &ime,
                application.as_ref(),
            );
        }
    });
    gesture.connect_released({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_gesture, _press_count, x, y| {
            let report = runtime.borrow_mut().dispatch_pointer_up(crate::Point {
                x: gtk_coordinate(x),
                y: gtk_coordinate(y),
            });
            if report.handled {
                area.grab_focus();
            }
            apply_linux_gtk_input_report(
                report,
                &area,
                &plan,
                &runtime,
                &ime,
                application.as_ref(),
            );
            reset_linux_gtk_ime_if_no_text_target(&runtime, &ime);
        }
    });
    gesture.connect_cancel({
        let runtime = Rc::clone(&runtime);
        move |_gesture, _sequence| {
            runtime.borrow_mut().cancel_pointer_drag();
        }
    });
    drawing_area.add_controller(gesture);
    let motion = gtk::EventControllerMotion::new();
    motion.connect_motion({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_motion, x, y| {
            let report = runtime.borrow_mut().dispatch_pointer_move(crate::Point {
                x: gtk_coordinate(x),
                y: gtk_coordinate(y),
            });
            if report.handled {
                apply_linux_gtk_input_report(
                    report,
                    &area,
                    &plan,
                    &runtime,
                    &ime,
                    application.as_ref(),
                );
            }
        }
    });
    drawing_area.add_controller(motion);
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        move |controller, _delta_x, delta_y| {
            if delta_y.abs() < f64::EPSILON {
                return gtk::glib::Propagation::Proceed;
            }
            let (x, y) = controller
                .current_event()
                .and_then(|event| event.position())
                .unwrap_or((0.0, 0.0));
            let report = runtime.borrow_mut().dispatch_pointer_scroll(
                crate::Point {
                    x: gtk_coordinate(x),
                    y: gtk_coordinate(y),
                },
                crate::Dp::new((delta_y * 48.0) as f32),
            );
            if let Some(updated) = report.redraw_plan {
                *plan.borrow_mut() = updated;
                area.queue_draw();
            }
            if report.quit_requested {
                if let Some(application) = &application {
                    application.quit();
                }
            }
            if report.handled {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        }
    });
    drawing_area.add_controller(scroll);
    ime.connect_preedit_changed({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |context| {
            let (text, _attributes, cursor) = context.preedit_string();
            let cursor = cursor.max(0) as usize;
            let report = runtime.borrow_mut().dispatch_ime_preedit(
                text.as_str(),
                (!text.is_empty()).then_some((cursor, cursor)),
            );
            apply_linux_gtk_input_report(
                report,
                &area,
                &plan,
                &runtime,
                &ime,
                application.as_ref(),
            );
        }
    });
    ime.connect_commit({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_context, text| {
            let report = runtime.borrow_mut().dispatch_ime_commit(text);
            apply_linux_gtk_input_report(
                report,
                &area,
                &plan,
                &runtime,
                &ime,
                application.as_ref(),
            );
            reset_linux_gtk_ime_if_no_text_target(&runtime, &ime);
        }
    });
    let keyboard = gtk::EventControllerKey::new();
    keyboard.set_im_context(Some(&ime));
    keyboard.connect_key_pressed({
        let application = window.application();
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_controller, key, _keycode, modifiers| {
            let shift = modifiers.contains(gtk::gdk::ModifierType::SHIFT_MASK);
            let control = modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK);
            let command_or_control = modifiers.intersects(
                gtk::gdk::ModifierType::CONTROL_MASK
                    | gtk::gdk::ModifierType::SUPER_MASK
                    | gtk::gdk::ModifierType::META_MASK,
            );
            let mut runtime_state = runtime.borrow_mut();
            let report = match key {
                gtk::gdk::Key::Tab => {
                    runtime_state.dispatch_key_with_shift(crate::NativeViewKey::Tab, shift)
                }
                gtk::gdk::Key::ISO_Left_Tab => {
                    runtime_state.dispatch_key_with_shift(crate::NativeViewKey::Tab, true)
                }
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                    let report = runtime_state.dispatch_key(crate::NativeViewKey::Enter);
                    if report.handled {
                        report
                    } else {
                        runtime_state.dispatch_text_input("\r")
                    }
                }
                gtk::gdk::Key::space => {
                    let report = runtime_state.dispatch_key(crate::NativeViewKey::Space);
                    if report.handled || command_or_control {
                        report
                    } else {
                        runtime_state.dispatch_text_input(" ")
                    }
                }
                gtk::gdk::Key::Up => runtime_state.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Up,
                    shift,
                    control,
                ),
                gtk::gdk::Key::Down => runtime_state.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Down,
                    shift,
                    control,
                ),
                gtk::gdk::Key::Left => runtime_state.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Left,
                    shift,
                    control,
                ),
                gtk::gdk::Key::Right => runtime_state.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Right,
                    shift,
                    control,
                ),
                gtk::gdk::Key::Home => {
                    runtime_state.dispatch_key_with_shift(crate::NativeViewKey::Home, shift)
                }
                gtk::gdk::Key::End => {
                    runtime_state.dispatch_key_with_shift(crate::NativeViewKey::End, shift)
                }
                gtk::gdk::Key::BackSpace => runtime_state.dispatch_text_input("\u{8}"),
                gtk::gdk::Key::Delete => runtime_state.dispatch_text_input("\u{7f}"),
                _ if !command_or_control => key
                    .to_unicode()
                    .filter(|character| !character.is_control())
                    .map(|character| runtime_state.dispatch_text_input(&character.to_string()))
                    .unwrap_or_default(),
                _ => crate::native::NativeViewInputDispatchReport::default(),
            };
            drop(runtime_state);
            let handled = report.handled;
            apply_linux_gtk_input_report(
                report,
                &area,
                &plan,
                &runtime,
                &ime,
                application.as_ref(),
            );
            reset_linux_gtk_ime_if_no_text_target(&runtime, &ime);
            if handled {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        }
    });
    drawing_area.add_controller(keyboard);
    let focus = gtk::EventControllerFocus::new();
    focus.connect_enter({
        let area = drawing_area.clone();
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_focus| sync_linux_gtk_ime(&area, &runtime, &ime)
    });
    focus.connect_leave({
        let area = drawing_area.clone();
        let plan = Rc::clone(&plan);
        let runtime = Rc::clone(&runtime);
        let ime = ime.clone();
        move |_focus| {
            let report = runtime.borrow_mut().blur_focus();
            if let Some(updated) = report.redraw_plan {
                *plan.borrow_mut() = updated;
                area.queue_draw();
            }
            ime.reset();
            ime.focus_out();
        }
    });
    drawing_area.add_controller(focus);
    window.set_child(Some(&drawing_area));
}

fn apply_linux_gtk_input_report(
    report: crate::native::NativeViewInputDispatchReport,
    area: &gtk::DrawingArea,
    plan: &Rc<RefCell<NativeDrawPlan>>,
    runtime: &Rc<RefCell<crate::native::NativeViewInputRuntime>>,
    ime: &gtk::IMMulticontext,
    application: Option<&gtk::Application>,
) {
    if let Some(updated) = report.redraw_plan {
        *plan.borrow_mut() = updated;
        area.queue_draw();
    }
    if report.quit_requested {
        if let Some(application) = application {
            application.quit();
        }
    }
    sync_linux_gtk_ime(area, runtime, ime);
}

fn sync_linux_gtk_ime(
    area: &gtk::DrawingArea,
    runtime: &Rc<RefCell<crate::native::NativeViewInputRuntime>>,
    ime: &gtk::IMMulticontext,
) {
    let (accepts_committed_text, caret_rect, surrounding) = {
        let runtime = runtime.borrow();
        (
            runtime.accepts_committed_text_input(),
            runtime.text_input_caret_rect(),
            runtime.focused_text_input_snapshot(),
        )
    };
    if area.has_focus() && accepts_committed_text {
        if let Some(rect) = caret_rect {
            ime.set_cursor_location(&gtk::gdk::Rectangle::new(
                rect.x,
                rect.y,
                rect.width.max(1),
                rect.height.max(1),
            ));
        }
        if let Some((value, selection)) = surrounding {
            let cursor = crate::native_text_edit::char_to_byte_index(&value, selection.caret)
                .min(i32::MAX as usize) as i32;
            ime.set_surrounding(&value, cursor);
        }
        ime.focus_in();
    } else {
        ime.focus_out();
    }
}

fn reset_linux_gtk_ime_if_no_text_target(
    runtime: &Rc<RefCell<crate::native::NativeViewInputRuntime>>,
    ime: &gtk::IMMulticontext,
) {
    let accepts_committed_text = runtime.borrow().accepts_committed_text_input();
    if !accepts_committed_text {
        ime.reset();
    }
}

fn gtk_coordinate(value: f64) -> i32 {
    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

pub struct LinuxGtkTextLayout {
    context: gtk::pango::Context,
}

impl LinuxGtkTextLayout {
    pub fn new(context: gtk::pango::Context) -> Self {
        Self { context }
    }

    fn pango_layout(
        &self,
        text: &str,
        style: &TextStyle,
        bounds: Option<Rect>,
    ) -> gtk::pango::Layout {
        let layout = gtk::pango::Layout::new(&self.context);
        configure_pango_layout(&layout, text, style, bounds);
        layout
    }
}

impl TextLayout for LinuxGtkTextLayout {
    fn measure(&self, text: &str, style: &TextStyle, max_width: i32) -> Size {
        if text.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        let bounds = (max_width > 0).then(|| Rect {
            x: 0,
            y: 0,
            width: max_width,
            height: i32::MAX / 4,
        });
        let layout = self.pango_layout(text, style, bounds);
        let (width, height) = layout.pixel_size();
        Size {
            width: width.max(0),
            height: height.max(0),
        }
    }

    fn layout_runs(&self, text: &str, _style: &TextStyle, bounds: Rect) -> Vec<TextRun> {
        if text.is_empty() {
            Vec::new()
        } else {
            vec![TextRun {
                text: text.to_string(),
                bounds,
            }]
        }
    }
}

struct LinuxGtkDrawSink<'a> {
    area: &'a gtk::DrawingArea,
    context: &'a gtk::cairo::Context,
    palette: NativeDrawPalette,
    style_resolver: NativeDrawTextStyleResolver,
    text_layout: LinuxGtkTextLayout,
    clip_depth: usize,
}

impl<'a> LinuxGtkDrawSink<'a> {
    fn new(
        area: &'a gtk::DrawingArea,
        context: &'a gtk::cairo::Context,
        palette: NativeDrawPalette,
    ) -> Self {
        Self {
            area,
            context,
            palette,
            style_resolver: NativeDrawTextStyleResolver::new(
                "Cantarell",
                "Monospace",
                "Cantarell",
                palette,
            ),
            text_layout: LinuxGtkTextLayout::new(area.pango_context()),
            clip_depth: 0,
        }
    }

    fn set_source(&self, color: Color) {
        self.context.set_source_rgba(
            f64::from(color.r) / 255.0,
            f64::from(color.g) / 255.0,
            f64::from(color.b) / 255.0,
            f64::from(color.a) / 255.0,
        );
    }

    fn add_rect(&self, rect: Rect) {
        self.context.rectangle(
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.width.max(0)),
            f64::from(rect.height.max(0)),
        );
    }

    fn add_round_rect(&self, rect: Rect, radius: i32) {
        let x = f64::from(rect.x);
        let y = f64::from(rect.y);
        let width = f64::from(rect.width.max(0));
        let height = f64::from(rect.height.max(0));
        let radius = f64::from(radius.max(0)).min(width / 2.0).min(height / 2.0);
        if radius <= 0.0 {
            self.context.rectangle(x, y, width, height);
            return;
        }
        let right = x + width;
        let bottom = y + height;
        self.context.new_sub_path();
        self.context.arc(
            right - radius,
            y + radius,
            radius,
            -std::f64::consts::FRAC_PI_2,
            0.0,
        );
        self.context.arc(
            right - radius,
            bottom - radius,
            radius,
            0.0,
            std::f64::consts::FRAC_PI_2,
        );
        self.context.arc(
            x + radius,
            bottom - radius,
            radius,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
        );
        self.context.arc(
            x + radius,
            y + radius,
            radius,
            std::f64::consts::PI,
            std::f64::consts::PI * 1.5,
        );
        self.context.close_path();
    }

    fn draw_text(&self, command: &NativeDrawTextCommand) {
        let style = self.style_resolver.resolve_text_style(command.style);
        let layout = self
            .text_layout
            .pango_layout(&command.text, &style, Some(command.bounds));
        let (_, text_height) = layout.pixel_size();
        let y = match style.vertical_align {
            VerticalAlign::Start => command.bounds.y,
            VerticalAlign::Center => {
                command.bounds.y + (command.bounds.height - text_height).max(0) / 2
            }
            VerticalAlign::End => command.bounds.y + (command.bounds.height - text_height).max(0),
        };
        self.set_source(style.color);
        self.context
            .move_to(f64::from(command.bounds.x), f64::from(y));
        unsafe {
            pango_cairo_show_layout(self.context.to_raw_none(), layout.to_glib_none().0);
        }
    }

    fn draw_icon(&self, command: &NativeDrawIconCommand) {
        let size = command.bounds.width.min(command.bounds.height).max(1);
        let theme = gtk::IconTheme::for_display(&self.area.display());
        let flags = if command.color_mode == NativeIconColorMode::ThemeAware {
            gtk::IconLookupFlags::FORCE_SYMBOLIC
        } else {
            gtk::IconLookupFlags::empty()
        };
        let paintable = theme.lookup_icon(
            command.icon.gtk_symbolic_name(),
            &[],
            size,
            1,
            gtk::TextDirection::None,
            flags,
        );
        let pixbuf = paintable
            .file()
            .and_then(|file| file.path())
            .and_then(|path| {
                gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(path, size, size, true).ok()
            });
        let pixbuf = pixbuf.or_else(|| {
            let loader = gtk::gdk_pixbuf::PixbufLoader::with_type("svg").ok()?;
            loader.set_size(size, size);
            loader
                .write(crate::bundled_fluent_icon_svg(command.icon))
                .ok()?;
            loader.close().ok()?;
            loader.pixbuf()
        });
        if let Some(pixbuf) = pixbuf {
            self.context.set_source_pixbuf(
                &pixbuf,
                f64::from(command.bounds.x),
                f64::from(command.bounds.y),
            );
            let _ = self.context.paint();
        }
    }

    fn push_clip(&mut self, rect: Rect) {
        if self.context.save().is_ok() {
            self.add_rect(rect);
            self.context.clip();
            self.clip_depth += 1;
        }
    }

    fn pop_clip(&mut self) {
        if self.clip_depth > 0 {
            let _ = self.context.restore();
            self.clip_depth -= 1;
        }
    }
}

impl Drop for LinuxGtkDrawSink<'_> {
    fn drop(&mut self) {
        while self.clip_depth > 0 {
            self.pop_clip();
        }
    }
}

impl NativeDrawCommandSink for LinuxGtkDrawSink<'_> {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                self.set_source(self.palette.resolve_fill(*fill));
                self.add_rect(*rect);
                let _ = self.context.fill();
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => {
                self.set_source(self.palette.resolve_fill(*stroke));
                self.context.set_line_width(f64::from((*width).max(1)));
                self.add_rect(*rect);
                let _ = self.context.stroke();
            }
            NativeDrawCommand::RoundRect {
                rect,
                fill,
                stroke,
                radius,
            } => {
                self.add_round_rect(*rect, *radius);
                self.set_source(self.palette.resolve_fill(*fill));
                if stroke.is_some() {
                    let _ = self.context.fill_preserve();
                } else {
                    let _ = self.context.fill();
                }
                if let Some(stroke) = stroke {
                    self.set_source(self.palette.resolve_fill(*stroke));
                    self.context.set_line_width(1.0);
                    let _ = self.context.stroke();
                }
            }
            NativeDrawCommand::RoundFill { rect, fill, radius } => {
                self.add_round_rect(*rect, *radius);
                self.set_source(self.palette.resolve_fill(*fill));
                let _ = self.context.fill();
            }
            NativeDrawCommand::Text(command) => self.draw_text(command),
            NativeDrawCommand::Icon(command) => self.draw_icon(command),
            NativeDrawCommand::PushClip { rect } => self.push_clip(*rect),
            NativeDrawCommand::PopClip => self.pop_clip(),
        }
    }
}

fn configure_pango_layout(
    layout: &gtk::pango::Layout,
    text: &str,
    style: &TextStyle,
    bounds: Option<Rect>,
) {
    layout.set_text(text);
    let mut font = gtk::pango::FontDescription::new();
    font.set_family(&style.font_family);
    font.set_absolute_size(f64::from(style.size) * f64::from(gtk::pango::SCALE));
    font.set_weight(match style.weight {
        TextWeight::Regular => gtk::pango::Weight::Normal,
        TextWeight::Medium => gtk::pango::Weight::Medium,
        TextWeight::Semibold => gtk::pango::Weight::Semibold,
        TextWeight::Bold => gtk::pango::Weight::Bold,
    });
    layout.set_font_description(Some(&font));
    layout.set_alignment(match style.horizontal_align {
        HorizontalAlign::Start => gtk::pango::Alignment::Left,
        HorizontalAlign::Center => gtk::pango::Alignment::Center,
        HorizontalAlign::End => gtk::pango::Alignment::Right,
    });
    layout.set_wrap(gtk::pango::WrapMode::WordChar);
    if let Some(bounds) = bounds {
        layout.set_width(bounds.width.max(0).saturating_mul(gtk::pango::SCALE));
        if style.wrap == TextWrap::Word {
            layout.set_height(bounds.height.max(0).saturating_mul(gtk::pango::SCALE));
        }
    }
    layout.set_single_paragraph_mode(style.wrap == TextWrap::NoWrap);
    layout.set_ellipsize(if style.ellipsis {
        gtk::pango::EllipsizeMode::End
    } else {
        gtk::pango::EllipsizeMode::None
    });
}
