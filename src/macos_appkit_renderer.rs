use std::cell::{Cell, RefCell};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject, Sel};
use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBackspaceCharacter, NSBezierPath, NSCarriageReturnCharacter,
    NSColor, NSColorSpace, NSDeleteCharacter, NSDownArrowFunctionKey, NSEndFunctionKey,
    NSEnterCharacter, NSEvent, NSEventModifierFlags, NSFont, NSFontAttributeName, NSFontWeightBold,
    NSFontWeightMedium, NSFontWeightRegular, NSFontWeightSemibold, NSForegroundColorAttributeName,
    NSGraphicsContext, NSHomeFunctionKey, NSImage, NSLeftArrowFunctionKey, NSLineBreakMode,
    NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSRightArrowFunctionKey,
    NSStringDrawing, NSStringDrawingOptions, NSStringNSExtendedStringDrawing, NSTabCharacter,
    NSTextAlignment, NSTextInputClient, NSTrackingArea, NSTrackingAreaOptions,
    NSUpArrowFunctionKey, NSView,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSDictionary, NSMutableDictionary,
    NSNotFound, NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString,
};

use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, NativeStyleResolver, Rect, Size,
    TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};

struct ZsuiAppKitDrawViewIvars {
    plan: RefCell<NativeDrawPlan>,
    runtime: RefCell<crate::native::NativeViewInputRuntime>,
    marked_text: RefCell<String>,
    marked_selection: Cell<Option<(usize, usize)>>,
    ime_dispatched: Cell<bool>,
}

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ZsuiAppKitDrawViewIvars]
    struct ZsuiAppKitDrawView;

    unsafe impl NSObjectProtocol for ZsuiAppKitDrawView {}

    unsafe impl NSTextInputClient for ZsuiAppKitDrawView {
        #[unsafe(method(insertText:replacementRange:))]
        unsafe fn insert_text(&self, string: &AnyObject, _replacement_range: NSRange) {
            let text = appkit_input_string(string);
            self.ivars().marked_text.borrow_mut().clear();
            self.ivars().marked_selection.set(None);
            let report = self.ivars().runtime.borrow_mut().dispatch_ime_commit(&text);
            self.ivars().ime_dispatched.set(report.handled);
            self.apply_input_report(report);
        }

        #[unsafe(method(doCommandBySelector:))]
        unsafe fn do_command_by_selector(&self, _selector: Sel) {
            self.ivars().ime_dispatched.set(false);
        }

        #[unsafe(method(setMarkedText:selectedRange:replacementRange:))]
        unsafe fn set_marked_text(
            &self,
            string: &AnyObject,
            selected_range: NSRange,
            _replacement_range: NSRange,
        ) {
            let text = appkit_input_string(string);
            let selection = utf16_range_to_char_range(&text, selected_range);
            *self.ivars().marked_text.borrow_mut() = text.clone();
            self.ivars().marked_selection.set(selection);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_ime_preedit(&text, selection);
            let accepts_committed_text = self
                .ivars()
                .runtime
                .borrow()
                .accepts_committed_text_input();
            self.ivars()
                .ime_dispatched
                .set(report.handled || accepts_committed_text);
            self.apply_input_report(report);
        }

        #[unsafe(method(unmarkText))]
        fn unmark_text(&self) {
            self.ivars().marked_text.borrow_mut().clear();
            self.ivars().marked_selection.set(None);
            let report = self.ivars().runtime.borrow_mut().cancel_ime_preedit();
            self.ivars().ime_dispatched.set(report.handled);
            self.apply_input_report(report);
        }

        #[unsafe(method(selectedRange))]
        fn selected_range(&self) -> NSRange {
            let runtime = self.ivars().runtime.borrow();
            let Some((committed, selection)) = runtime.focused_text_input_snapshot() else {
                return NSRange::new(NSNotFound as usize, 0);
            };
            if let Some((start, end)) = self.ivars().marked_selection.get() {
                let replacement_start = runtime
                    .ime_replacement_selection()
                    .map(|selection| selection.ordered().0)
                    .unwrap_or(selection.caret);
                let base = char_index_to_utf16_offset(&committed, replacement_start);
                let marked = self.ivars().marked_text.borrow();
                let start = char_index_to_utf16_offset(&marked, start);
                let end = char_index_to_utf16_offset(&marked, end);
                NSRange::new(base.saturating_add(start), end.saturating_sub(start))
            } else {
                let (start, end) = selection.ordered();
                let start = char_index_to_utf16_offset(&committed, start);
                let end = char_index_to_utf16_offset(&committed, end);
                NSRange::new(start, end.saturating_sub(start))
            }
        }

        #[unsafe(method(markedRange))]
        fn marked_range(&self) -> NSRange {
            let marked = self.ivars().marked_text.borrow();
            if marked.is_empty() {
                return NSRange::new(NSNotFound as usize, 0);
            }
            let runtime = self.ivars().runtime.borrow();
            let start = runtime
                .focused_text_input_snapshot()
                .map(|(value, selection)| {
                    let replacement_start = runtime
                        .ime_replacement_selection()
                        .map(|selection| selection.ordered().0)
                        .unwrap_or(selection.caret);
                    char_index_to_utf16_offset(&value, replacement_start)
                })
                .unwrap_or(0);
            NSRange::new(start, marked.encode_utf16().count())
        }

        #[unsafe(method(hasMarkedText))]
        fn has_marked_text(&self) -> bool {
            !self.ivars().marked_text.borrow().is_empty()
        }

        #[unsafe(method_id(attributedSubstringForProposedRange:actualRange:))]
        unsafe fn attributed_substring_for_proposed_range(
            &self,
            _range: NSRange,
            actual_range: *mut NSRange,
        ) -> Option<Retained<NSAttributedString>> {
            if !actual_range.is_null() {
                unsafe { actual_range.write(NSRange::new(NSNotFound as usize, 0)) };
            }
            None
        }

        #[unsafe(method_id(validAttributesForMarkedText))]
        fn valid_attributes_for_marked_text(
            &self,
        ) -> Retained<NSArray<NSAttributedStringKey>> {
            NSArray::new()
        }

        #[unsafe(method(firstRectForCharacterRange:actualRange:))]
        unsafe fn first_rect_for_character_range(
            &self,
            range: NSRange,
            actual_range: *mut NSRange,
        ) -> NSRect {
            if !actual_range.is_null() {
                unsafe { actual_range.write(range) };
            }
            let local = self
                .ivars()
                .runtime
                .borrow()
                .text_input_caret_rect()
                .map(appkit_rect)
                .unwrap_or_else(|| {
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1.0, 1.0))
                });
            self.window()
                .map(|window| window.convertRectToScreen(self.convertRect_toView(local, None)))
                .unwrap_or(local)
        }

        #[unsafe(method(characterIndexForPoint:))]
        fn character_index_for_point(&self, _point: NSPoint) -> usize {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_input_snapshot()
                .map(|(value, selection)| char_index_to_utf16_offset(&value, selection.caret))
                .unwrap_or(0)
        }
    }

    impl ZsuiAppKitDrawView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(resignFirstResponder))]
        fn resign_first_responder(&self) -> bool {
            self.ivars().marked_text.borrow_mut().clear();
            self.ivars().marked_selection.set(None);
            let report = self.ivars().runtime.borrow_mut().blur_focus();
            self.apply_input_report(report);
            if let Some(context) = self.inputContext() {
                context.discardMarkedText();
            }
            unsafe { msg_send![super(self), resignFirstResponder] }
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty_rect: NSRect) {
            let bounds = self.bounds();
            let resize = self.ivars().runtime.borrow_mut().set_surface(
                Rect {
                    x: 0,
                    y: 0,
                    width: appkit_coordinate(bounds.size.width).max(0),
                    height: appkit_coordinate(bounds.size.height).max(0),
                },
                crate::Dpi::standard(),
            );
            if let Some(plan) = resize.redraw_plan {
                *self.ivars().plan.borrow_mut() = plan;
            }
            if resize.surface_changed {
                if let Some(context) = self.inputContext() {
                    context.invalidateCharacterCoordinates();
                }
            }
            let (system_prefers_dark, system_high_contrast) =
                appkit_system_appearance(self.mtm());
            let plan = self.ivars().plan.borrow();
            let palette = NativeDrawPalette::for_system_appearance(
                plan.theme_mode,
                system_prefers_dark,
                system_high_contrast,
                system_high_contrast
                    .then(appkit_semantic_high_contrast_palette)
                    .flatten(),
            );
            let mut sink = MacosAppKitDrawSink::new(palette);
            sink.draw_plan(&plan);
        }

        #[unsafe(method(viewDidChangeEffectiveAppearance))]
        fn view_did_change_effective_appearance(&self) {
            unsafe {
                let _: () = msg_send![super(self), viewDidChangeEffectiveAppearance];
            }
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            let location = self.convertPoint_fromView(event.locationInWindow(), None);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_pointer_down(
                    crate::Point {
                        x: appkit_coordinate(location.x),
                        y: appkit_coordinate(location.y),
                    },
                    event
                        .modifierFlags()
                        .contains(NSEventModifierFlags::Shift),
                );
            if report.handled {
                if let Some(window) = self.window() {
                    window.makeFirstResponder(Some(self));
                }
            }
            self.apply_input_report(report);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            let location = self.convertPoint_fromView(event.locationInWindow(), None);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_pointer_move(crate::Point {
                    x: appkit_coordinate(location.x),
                    y: appkit_coordinate(location.y),
                });
            self.apply_input_report(report);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            let location = self.convertPoint_fromView(event.locationInWindow(), None);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_pointer_move(crate::Point {
                    x: appkit_coordinate(location.x),
                    y: appkit_coordinate(location.y),
                });
            self.apply_input_report(report);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            let report = self.ivars().runtime.borrow_mut().dispatch_pointer_leave();
            self.apply_input_report(report);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let location = self.convertPoint_fromView(event.locationInWindow(), None);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_pointer_up(crate::Point {
                    x: appkit_coordinate(location.x),
                    y: appkit_coordinate(location.y),
                });
            if report.handled {
                if let Some(window) = self.window() {
                    window.makeFirstResponder(Some(self));
                }
            }
            self.apply_input_report(report);
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            let modifiers = event.modifierFlags();
            let shift = modifiers.contains(NSEventModifierFlags::Shift);
            let control = modifiers.contains(NSEventModifierFlags::Control);
            let command_or_control = modifiers
                .intersects(NSEventModifierFlags::Command | NSEventModifierFlags::Control);
            let unmodified = event
                .charactersIgnoringModifiers()
                .map(|text| text.to_string())
                .unwrap_or_default();
            let code = unmodified.chars().next().map(u32::from);
            if !command_or_control
                && self
                    .ivars()
                    .runtime
                    .borrow()
                    .accepts_committed_text_input()
            {
                self.ivars().ime_dispatched.set(false);
                let events = NSArray::from_slice(&[event]);
                self.interpretKeyEvents(&events);
                if self.ivars().ime_dispatched.get() {
                    return;
                }
            }
            let mut runtime = self.ivars().runtime.borrow_mut();
            let report = match code {
                Some(code) if code == NSTabCharacter => {
                    runtime.dispatch_key_with_modifiers(
                        crate::NativeViewKey::Tab,
                        shift,
                        command_or_control,
                    )
                }
                Some(code)
                    if code == NSCarriageReturnCharacter || code == NSEnterCharacter =>
                {
                    let report = runtime.dispatch_key(crate::NativeViewKey::Enter);
                    if report.handled {
                        report
                    } else {
                        runtime.dispatch_text_input("\r")
                    }
                }
                Some(code) if code == u32::from(' ') => {
                    let report = runtime.dispatch_key(crate::NativeViewKey::Space);
                    if report.handled || command_or_control {
                        report
                    } else {
                        runtime.dispatch_text_input(" ")
                    }
                }
                Some(code) if code == NSUpArrowFunctionKey => {
                    runtime.dispatch_key_with_modifiers(crate::NativeViewKey::Up, shift, control)
                }
                Some(code) if code == NSDownArrowFunctionKey => {
                    runtime.dispatch_key_with_modifiers(crate::NativeViewKey::Down, shift, control)
                }
                Some(code) if code == NSLeftArrowFunctionKey => runtime.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Left,
                    shift,
                    control,
                ),
                Some(code) if code == NSRightArrowFunctionKey => runtime.dispatch_key_with_modifiers(
                    crate::NativeViewKey::Right,
                    shift,
                    control,
                ),
                Some(code) if code == NSHomeFunctionKey => runtime
                    .dispatch_key_with_shift(crate::NativeViewKey::Home, shift),
                Some(code) if code == NSEndFunctionKey => runtime
                    .dispatch_key_with_shift(crate::NativeViewKey::End, shift),
                Some(code) if code == NSBackspaceCharacter => {
                    runtime.dispatch_text_input("\u{8}")
                }
                Some(code) if code == NSDeleteCharacter => runtime.dispatch_text_input("\u{7f}"),
                _ if !command_or_control => event
                    .characters()
                    .map(|text| runtime.dispatch_text_input(&text.to_string()))
                    .unwrap_or_default(),
                _ => crate::native::NativeViewInputDispatchReport::default(),
            };
            drop(runtime);
            if report.handled {
                self.apply_input_report(report);
            } else {
                unsafe {
                    let _: () = msg_send![super(self), keyDown: event];
                }
            }
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            let raw_delta = event.scrollingDeltaY() as f32;
            let delta_y = if event.hasPreciseScrollingDeltas() {
                -raw_delta
            } else {
                -raw_delta * 48.0
            };
            if delta_y.abs() < f32::EPSILON {
                return;
            }
            let location = self.convertPoint_fromView(event.locationInWindow(), None);
            let report = self
                .ivars()
                .runtime
                .borrow_mut()
                .dispatch_pointer_scroll(
                    crate::Point {
                        x: appkit_coordinate(location.x),
                        y: appkit_coordinate(location.y),
                    },
                    crate::Dp::new(delta_y),
                );
            self.apply_input_report(report);
        }
    }
);

impl ZsuiAppKitDrawView {
    fn apply_input_report(&self, report: crate::native::NativeViewInputDispatchReport) {
        if let Some(plan) = report.redraw_plan {
            *self.ivars().plan.borrow_mut() = plan;
            self.setNeedsDisplay(true);
        }
        if report.quit_requested {
            objc2_app_kit::NSApplication::sharedApplication(self.mtm()).stop(None);
        }
        let should_discard_marked_text = !self.ivars().runtime.borrow().has_focused_text_input()
            && !self.ivars().marked_text.borrow().is_empty();
        if should_discard_marked_text {
            self.ivars().marked_text.borrow_mut().clear();
            self.ivars().marked_selection.set(None);
            if let Some(context) = self.inputContext() {
                context.discardMarkedText();
            }
        }
    }

    fn new(
        mtm: MainThreadMarker,
        frame: NSRect,
        plan: NativeDrawPlan,
        runtime: crate::native::NativeViewInputRuntime,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ZsuiAppKitDrawViewIvars {
            plan: RefCell::new(plan),
            runtime: RefCell::new(runtime),
            marked_text: RefCell::new(String::new()),
            marked_selection: Cell::new(None),
            ime_dispatched: Cell::new(false),
        });
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }

    fn install_pointer_tracking(&self) {
        let options = NSTrackingAreaOptions::MouseEnteredAndExited
            | NSTrackingAreaOptions::MouseMoved
            | NSTrackingAreaOptions::ActiveInKeyWindow
            | NSTrackingAreaOptions::InVisibleRect
            | NSTrackingAreaOptions::EnabledDuringMouseDrag;
        let tracking_area = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                NSTrackingArea::alloc(),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
                options,
                Some(self),
                None,
            )
        };
        self.addTrackingArea(&tracking_area);
    }
}

pub(crate) fn install_macos_appkit_draw_plan(
    window: &objc2_app_kit::NSWindow,
    plan: NativeDrawPlan,
    runtime: crate::native::NativeViewInputRuntime,
) {
    let mtm = window.mtm();
    let frame = window
        .contentView()
        .map(|view| view.frame())
        .unwrap_or_else(|| NSRect::new(NSPoint::new(0.0, 0.0), window.frame().size));
    let view = ZsuiAppKitDrawView::new(mtm, frame, plan, runtime);
    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    view.install_pointer_tracking();
    window.setAcceptsMouseMovedEvents(true);
    window.setContentView(Some(&view));
    view.setNeedsDisplay(true);
}

pub(crate) struct MacosAppKitTextLayout;

impl TextLayout for MacosAppKitTextLayout {
    fn measure(&self, text: &str, style: &TextStyle, max_width: i32) -> Size {
        if text.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        let attributes = appkit_text_attributes(style);
        let dictionary: &NSDictionary<NSAttributedStringKey, AnyObject> = &attributes;
        let text = NSString::from_str(text);
        let measured = if style.wrap == TextWrap::Word && max_width > 0 {
            unsafe {
                text.boundingRectWithSize_options_attributes_context(
                    NSSize::new(f64::from(max_width), 32_767.0),
                    NSStringDrawingOptions::UsesLineFragmentOrigin
                        | NSStringDrawingOptions::UsesFontLeading,
                    Some(dictionary),
                    None,
                )
                .size
            }
        } else {
            unsafe { text.sizeWithAttributes(Some(dictionary)) }
        };
        let width = measured.width.ceil() as i32;
        let width = if max_width > 0 {
            width.min(max_width)
        } else {
            width
        };
        Size {
            width: width.max(0),
            height: measured.height.ceil() as i32,
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

struct MacosAppKitDrawSink {
    palette: NativeDrawPalette,
    style_resolver: NativeDrawTextStyleResolver,
    text_layout: MacosAppKitTextLayout,
    clip_depth: usize,
}

impl MacosAppKitDrawSink {
    fn new(palette: NativeDrawPalette) -> Self {
        Self {
            palette,
            style_resolver: NativeDrawTextStyleResolver::new(
                ".AppleSystemUIFont",
                "Menlo",
                ".AppleSystemUIFont",
                palette,
            ),
            text_layout: MacosAppKitTextLayout,
            clip_depth: 0,
        }
    }

    fn draw_text(&self, command: &NativeDrawTextCommand) {
        let style = self.style_resolver.resolve_text_style(command.style);
        let attributes = appkit_text_attributes(&style);
        let dictionary: &NSDictionary<NSAttributedStringKey, AnyObject> = &attributes;
        let text = NSString::from_str(&command.text);
        let measured = self
            .text_layout
            .measure(&command.text, &style, command.bounds.width);
        let y = match style.vertical_align {
            VerticalAlign::Start => command.bounds.y,
            VerticalAlign::Center => {
                command.bounds.y + (command.bounds.height - measured.height).max(0) / 2
            }
            VerticalAlign::End => {
                command.bounds.y + (command.bounds.height - measured.height).max(0)
            }
        };
        let rect = NSRect::new(
            NSPoint::new(f64::from(command.bounds.x), f64::from(y)),
            NSSize::new(
                f64::from(command.bounds.width.max(0)),
                f64::from(command.bounds.height.max(0)),
            ),
        );
        let mut options = NSStringDrawingOptions::UsesFontLeading;
        if style.wrap == TextWrap::Word {
            options |= NSStringDrawingOptions::UsesLineFragmentOrigin;
        }
        if style.ellipsis {
            options |= NSStringDrawingOptions::TruncatesLastVisibleLine;
        }
        unsafe {
            text.drawWithRect_options_attributes_context(rect, options, Some(dictionary), None)
        };
    }

    fn draw_icon(&self, command: &NativeDrawIconCommand) {
        let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(command.icon.sf_symbol_name()),
            None,
        ) else {
            return;
        };
        if command.color_mode == NativeIconColorMode::ThemeAware {
            image.setTemplate(true);
            appkit_color(self.palette.resolve(command.color)).set();
        }
        image.drawInRect(appkit_rect(command.bounds));
    }

    fn pop_clip(&mut self) {
        if self.clip_depth > 0 {
            NSGraphicsContext::restoreGraphicsState_class();
            self.clip_depth -= 1;
        }
    }
}

impl Drop for MacosAppKitDrawSink {
    fn drop(&mut self) {
        while self.clip_depth > 0 {
            self.pop_clip();
        }
    }
}

impl NativeDrawCommandSink for MacosAppKitDrawSink {
    fn draw_command(&mut self, command: &NativeDrawCommand) {
        match command {
            NativeDrawCommand::FillRect { rect, fill } => {
                appkit_color(self.palette.resolve_fill(*fill)).setFill();
                NSBezierPath::fillRect(appkit_rect(*rect));
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => {
                appkit_color(self.palette.resolve_fill(*stroke)).setStroke();
                let path = NSBezierPath::bezierPathWithRect(appkit_rect(*rect));
                path.setLineWidth(f64::from((*width).max(1)));
                path.stroke();
            }
            NativeDrawCommand::RoundRect {
                rect,
                fill,
                stroke,
                radius,
            } => {
                let radius = f64::from((*radius).max(0));
                let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    appkit_rect(*rect),
                    radius,
                    radius,
                );
                appkit_color(self.palette.resolve_fill(*fill)).setFill();
                path.fill();
                if let Some(stroke) = stroke {
                    appkit_color(self.palette.resolve_fill(*stroke)).setStroke();
                    path.setLineWidth(1.0);
                    path.stroke();
                }
            }
            NativeDrawCommand::RoundFill { rect, fill, radius } => {
                let radius = f64::from((*radius).max(0));
                let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    appkit_rect(*rect),
                    radius,
                    radius,
                );
                appkit_color(self.palette.resolve_fill(*fill)).setFill();
                path.fill();
            }
            NativeDrawCommand::Text(command) => self.draw_text(command),
            #[cfg(feature = "password-box")]
            NativeDrawCommand::SecureText(command) => {
                let rendered = command.rendered_text();
                self.draw_text(&NativeDrawTextCommand::new(
                    rendered.as_str(),
                    command.bounds,
                    command.style,
                ));
            }
            NativeDrawCommand::Icon(command) => self.draw_icon(command),
            NativeDrawCommand::PushClip { rect } => {
                NSGraphicsContext::saveGraphicsState_class();
                NSBezierPath::bezierPathWithRect(appkit_rect(*rect)).addClip();
                self.clip_depth += 1;
            }
            NativeDrawCommand::PopClip => self.pop_clip(),
        }
    }
}

fn appkit_text_attributes(
    style: &TextStyle,
) -> Retained<NSMutableDictionary<NSAttributedStringKey, AnyObject>> {
    let attributes = NSMutableDictionary::<NSAttributedStringKey, AnyObject>::new();
    let weight = unsafe {
        match style.weight {
            TextWeight::Regular => NSFontWeightRegular,
            TextWeight::Medium => NSFontWeightMedium,
            TextWeight::Semibold => NSFontWeightSemibold,
            TextWeight::Bold => NSFontWeightBold,
        }
    };
    let font = if style.font_family == "Menlo" {
        NSFont::monospacedSystemFontOfSize_weight(f64::from(style.size), weight)
    } else {
        NSFont::systemFontOfSize_weight(f64::from(style.size), weight)
    };
    let color = appkit_color(style.color);
    let paragraph = NSMutableParagraphStyle::new();
    paragraph.setAlignment(match style.horizontal_align {
        HorizontalAlign::Start => NSTextAlignment::Left,
        HorizontalAlign::Center => NSTextAlignment::Center,
        HorizontalAlign::End => NSTextAlignment::Right,
    });
    paragraph.setLineBreakMode(match (style.wrap, style.ellipsis) {
        (TextWrap::Word, _) => NSLineBreakMode::ByWordWrapping,
        (TextWrap::NoWrap, true) => NSLineBreakMode::ByTruncatingTail,
        (TextWrap::NoWrap, false) => NSLineBreakMode::ByClipping,
    });
    unsafe {
        attributes.setObject_forKey(font.as_ref(), ProtocolObject::from_ref(NSFontAttributeName));
        attributes.setObject_forKey(
            color.as_ref(),
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
        attributes.setObject_forKey(
            paragraph.as_ref(),
            ProtocolObject::from_ref(NSParagraphStyleAttributeName),
        );
    }
    attributes
}

fn appkit_color(color: Color) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    )
}

fn appkit_rect(rect: Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(f64::from(rect.x), f64::from(rect.y)),
        NSSize::new(f64::from(rect.width.max(0)), f64::from(rect.height.max(0))),
    )
}

fn appkit_coordinate(value: f64) -> i32 {
    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

fn appkit_input_string(value: &AnyObject) -> String {
    if let Some(string) = value.downcast_ref::<NSString>() {
        string.to_string()
    } else if let Some(attributed) = value.downcast_ref::<NSAttributedString>() {
        attributed.string().to_string()
    } else {
        String::new()
    }
}

fn utf16_range_to_char_range(text: &str, range: NSRange) -> Option<(usize, usize)> {
    if text.is_empty() {
        return None;
    }
    let start = utf16_offset_to_char_index(text, range.location);
    let end = utf16_offset_to_char_index(text, range.location.saturating_add(range.length));
    Some((start.min(end), start.max(end)))
}

fn utf16_offset_to_char_index(text: &str, offset: usize) -> usize {
    let mut utf16_units = 0_usize;
    for (index, character) in text.chars().enumerate() {
        if utf16_units >= offset {
            return index;
        }
        utf16_units = utf16_units.saturating_add(character.len_utf16());
    }
    text.chars().count()
}

fn char_index_to_utf16_offset(text: &str, index: usize) -> usize {
    text.chars().take(index).map(char::len_utf16).sum()
}

fn appkit_system_appearance(mtm: MainThreadMarker) -> (bool, bool) {
    let application = objc2_app_kit::NSApplication::sharedApplication(mtm);
    appkit_appearance_flags(&application.effectiveAppearance().name().to_string())
}

fn appkit_appearance_flags(name: &str) -> (bool, bool) {
    (name.contains("Dark"), name.contains("HighContrast"))
}

fn appkit_semantic_high_contrast_palette() -> Option<NativeDrawPalette> {
    let primary_text = appkit_native_color(&NSColor::labelColor())?;
    let surface = appkit_native_color(&NSColor::windowBackgroundColor())?;
    Some(NativeDrawPalette {
        primary_text,
        secondary_text: primary_text,
        disabled_text: appkit_native_color(&NSColor::disabledControlTextColor())?,
        accent: appkit_native_color(&NSColor::selectedContentBackgroundColor())?,
        accent_text: appkit_native_color(&NSColor::selectedControlTextColor())?,
        surface,
        surface_raised: appkit_native_color(&NSColor::controlBackgroundColor())?,
        control: appkit_native_color(&NSColor::controlBackgroundColor())?,
        border: appkit_native_color(&NSColor::separatorColor())?,
        success: appkit_native_color(&NSColor::systemGreenColor())?,
        warning: appkit_native_color(&NSColor::systemOrangeColor())?,
        danger: appkit_native_color(&NSColor::systemRedColor())?,
        high_contrast: true,
    })
}

fn appkit_native_color(color: &NSColor) -> Option<Color> {
    let color = color.colorUsingColorSpace(&NSColorSpace::deviceRGBColorSpace())?;
    Some(Color::rgba(
        appkit_color_channel(color.redComponent()),
        appkit_color_channel(color.greenComponent()),
        appkit_color_channel(color.blueComponent()),
        appkit_color_channel(color.alphaComponent()),
    ))
}

fn appkit_color_channel(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}
