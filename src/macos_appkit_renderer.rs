use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    path::Path,
    ptr::NonNull,
};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject, Sel};
#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
use objc2::Message;
use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
#[cfg(all(feature = "accessibility", feature = "text-input-core"))]
use objc2_app_kit::{
    NSAccessibilitySecureTextFieldSubrole, NSAccessibilityTextAreaRole,
    NSAccessibilityTextFieldRole,
};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBackspaceCharacter, NSBezierPath, NSBitmapImageFileType,
    NSBitmapImageRepPropertyKey, NSCarriageReturnCharacter, NSColor, NSColorSpace,
    NSDeleteCharacter, NSDownArrowFunctionKey, NSEndFunctionKey, NSEnterCharacter, NSEvent,
    NSEventModifierFlags, NSFont, NSFontAttributeName, NSFontTextStyle, NSFontTextStyleBody,
    NSFontTextStyleCaption1, NSFontTextStyleLargeTitle, NSFontTextStyleOptionKey,
    NSFontTextStyleTitle1, NSFontTextStyleTitle2, NSFontTextStyleTitle3, NSFontWeightBold,
    NSFontWeightMedium, NSFontWeightRegular, NSFontWeightSemibold, NSForegroundColorAttributeName,
    NSGraphicsContext, NSHomeFunctionKey, NSImage, NSLeftArrowFunctionKey, NSLineBreakMode,
    NSMutableParagraphStyle, NSPageDownFunctionKey, NSPageUpFunctionKey,
    NSParagraphStyleAttributeName, NSRightArrowFunctionKey, NSStringDrawing,
    NSStringDrawingOptions, NSStringNSExtendedStringDrawing, NSTabCharacter, NSTextAlignment,
    NSTextInputClient, NSTrackingArea, NSTrackingAreaOptions, NSUpArrowFunctionKey, NSView,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSDictionary, NSMutableDictionary,
    NSNotFound, NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString, NSTimer,
};

use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawImageCommand, NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode,
    NativeImageInterpolation, NativeStyleResolver, Rect, Size, TextLayout, TextRun, TextStyle,
    TextWeight, TextWrap, VerticalAlign,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsRect {
    origin: CoreGraphicsPoint,
    size: CoreGraphicsSize,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGDataProviderCreateWithData(
        info: *mut c_void,
        data: *const c_void,
        size: usize,
        release_data: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize)>,
    ) -> *mut c_void;
    fn CGDataProviderRelease(provider: *mut c_void);
    fn CGColorSpaceCreateDeviceRGB() -> *mut c_void;
    fn CGColorSpaceRelease(color_space: *mut c_void);
    fn CGImageCreate(
        width: usize,
        height: usize,
        bits_per_component: usize,
        bits_per_pixel: usize,
        bytes_per_row: usize,
        color_space: *mut c_void,
        bitmap_info: u32,
        provider: *mut c_void,
        decode: *const f64,
        should_interpolate: bool,
        intent: i32,
    ) -> *mut c_void;
    fn CGImageCreateWithImageInRect(image: *mut c_void, rect: CoreGraphicsRect) -> *mut c_void;
    fn CGImageRelease(image: *mut c_void);
    fn CGContextSaveGState(context: *mut c_void);
    fn CGContextRestoreGState(context: *mut c_void);
    fn CGContextTranslateCTM(context: *mut c_void, tx: f64, ty: f64);
    fn CGContextScaleCTM(context: *mut c_void, sx: f64, sy: f64);
    fn CGContextSetInterpolationQuality(context: *mut c_void, quality: i32);
    fn CGContextDrawImage(context: *mut c_void, rect: CoreGraphicsRect, image: *mut c_void);
}

const CORE_GRAPHICS_ALPHA_PREMULTIPLIED_FIRST: u32 = 2;
const CORE_GRAPHICS_BYTE_ORDER_32_LITTLE: u32 = 2 << 12;
const CORE_GRAPHICS_RENDERING_INTENT_DEFAULT: i32 = 0;
const CORE_GRAPHICS_INTERPOLATION_NONE: i32 = 1;
const CORE_GRAPHICS_INTERPOLATION_HIGH: i32 = 3;

struct ZsuiAppKitDrawViewIvars {
    plan: RefCell<NativeDrawPlan>,
    runtime: RefCell<crate::native::NativeViewInputRuntime>,
    runtime_timer: RefCell<Option<Retained<NSTimer>>>,
    marked_text: RefCell<String>,
    marked_selection: Cell<Option<(usize, usize)>>,
    ime_dispatched: Cell<bool>,
}

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ZsuiAppKitDrawViewIvars]
    struct ZsuiAppKitDrawView;

    impl ZsuiAppKitDrawView {
        #[unsafe(method(zsuiRuntimeTick:))]
        fn zsui_runtime_tick(&self, _timer: &NSTimer) {
            self.ivars().runtime_timer.borrow_mut().take();
            let report = self.ivars().runtime.borrow_mut().refresh_transient_view();
            self.apply_input_report(report);
        }
    }

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
        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(isAccessibilityElement))]
        fn is_accessibility_element(&self) -> bool {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .is_some()
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilityRole))]
        fn accessibility_role(&self) -> Option<Retained<NSString>> {
            self
                .ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| {
                    let role = if snapshot.kind().is_multiline() {
                        unsafe { NSAccessibilityTextAreaRole }
                    } else {
                        unsafe { NSAccessibilityTextFieldRole }
                    };
                    role.retain()
                })
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilitySubrole))]
        fn accessibility_subrole(&self) -> Option<Retained<NSString>> {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .filter(|snapshot| snapshot.kind().is_protected())
                .map(|_| unsafe { NSAccessibilitySecureTextFieldSubrole }.retain())
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilityIdentifier))]
        fn accessibility_identifier(&self) -> Option<Retained<NSString>> {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| NSString::from_str(&format!("zsui-widget-{}", snapshot.widget().0)))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilityValue))]
        fn accessibility_value(&self) -> Option<Retained<NSString>> {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| NSString::from_str(snapshot.exposed_text()))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(isAccessibilityProtectedContent))]
        fn is_accessibility_protected_content(&self) -> bool {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .is_some_and(|snapshot| snapshot.kind().is_protected())
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(isAccessibilityFocused))]
        fn is_accessibility_focused(&self) -> bool {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .is_some()
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(accessibilityNumberOfCharacters))]
        fn accessibility_number_of_characters(&self) -> isize {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| snapshot.utf16_offset(snapshot.character_count()).unwrap_or(0))
                .unwrap_or(0)
                .min(isize::MAX as usize) as isize
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilitySelectedText))]
        fn accessibility_selected_text(&self) -> Option<Retained<NSString>> {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| NSString::from_str(&snapshot.selected_text()))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(accessibilitySelectedTextRange))]
        fn accessibility_selected_text_range(&self) -> NSRange {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| {
                    let range = snapshot.utf16_selection();
                    NSRange::new(range.start, range.end.saturating_sub(range.start))
                })
                .unwrap_or_else(|| NSRange::new(NSNotFound as usize, 0))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(accessibilityVisibleCharacterRange))]
        fn accessibility_visible_character_range(&self) -> NSRange {
            self.ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| {
                    NSRange::new(
                        0,
                        snapshot.utf16_offset(snapshot.character_count()).unwrap_or(0),
                    )
                })
                .unwrap_or_else(|| NSRange::new(NSNotFound as usize, 0))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method_id(accessibilityStringForRange:))]
        fn accessibility_string_for_range(&self, range: NSRange) -> Option<Retained<NSString>> {
            self
                .ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .and_then(|snapshot| {
                    let scalar_range = utf16_range_to_char_range(snapshot.exposed_text(), range)?;
                    snapshot.text_in_range(scalar_range.0..scalar_range.1)
                })
                .map(|text| NSString::from_str(&text))
        }

        #[cfg(all(feature = "accessibility", feature = "text-input-core"))]
        #[unsafe(method(accessibilityFrame))]
        fn accessibility_frame(&self) -> NSRect {
            let local = self
                .ivars()
                .runtime
                .borrow()
                .focused_text_accessibility_snapshot()
                .map(|snapshot| appkit_rect(snapshot.bounds()))
                .unwrap_or_else(|| {
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0))
                });
            self.window()
                .map(|window| window.convertRectToScreen(self.convertRect_toView(local, None)))
                .unwrap_or(local)
        }

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
                appkit_semantic_palette(),
                system_high_contrast
                    .then(appkit_semantic_high_contrast_palette)
                    .flatten(),
            );
            let mut sink = MacosAppKitDrawSink::new(palette, plan.typography_scale());
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
                Some(code) if code == NSPageUpFunctionKey => runtime.dispatch_key_with_modifiers(
                    crate::NativeViewKey::PageUp,
                    shift,
                    control,
                ),
                Some(code) if code == NSPageDownFunctionKey => runtime
                    .dispatch_key_with_modifiers(crate::NativeViewKey::PageDown, shift, control),
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
    fn apply_input_report(
        &self,
        mut report: crate::native::NativeViewInputDispatchReport,
    ) -> crate::native::NativeViewInputDispatchReport {
        let (executor, commands) = self
            .ivars()
            .runtime
            .borrow_mut()
            .take_pending_app_command_dispatch();
        let effect_executed = crate::native::dispatch_deferred_native_view_app_commands(
            &mut report,
            executor,
            commands,
        );
        if effect_executed {
            self.ivars()
                .runtime
                .borrow_mut()
                .refresh_live_view_after_app_effect(&mut report);
        }
        if let Some(plan) = report.redraw_plan.clone() {
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
        self.schedule_runtime_tick();
        report
    }

    fn schedule_runtime_tick(&self) {
        if let Some(timer) = self.ivars().runtime_timer.borrow_mut().take() {
            timer.invalidate();
        }
        let Some(delay_ms) = self.ivars().runtime.borrow().transient_poll_interval_ms() else {
            return;
        };
        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                delay_ms.max(1) as f64 / 1_000.0,
                self,
                objc2::sel!(zsuiRuntimeTick:),
                None,
                false,
            )
        };
        *self.ivars().runtime_timer.borrow_mut() = Some(timer);
    }

    fn new(
        mtm: MainThreadMarker,
        frame: NSRect,
        plan: NativeDrawPlan,
        mut runtime: crate::native::NativeViewInputRuntime,
    ) -> Retained<Self> {
        runtime.defer_app_command_execution();
        let this = Self::alloc(mtm).set_ivars(ZsuiAppKitDrawViewIvars {
            plan: RefCell::new(plan),
            runtime: RefCell::new(runtime),
            runtime_timer: RefCell::new(None),
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

#[derive(Clone)]
pub(crate) struct MacosAppKitDrawViewHost {
    view: Retained<ZsuiAppKitDrawView>,
}

impl std::fmt::Debug for MacosAppKitDrawViewHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MacosAppKitDrawViewHost")
            .finish_non_exhaustive()
    }
}

impl MacosAppKitDrawViewHost {
    pub(crate) fn dispatch_proof_inputs(
        &self,
        inputs: &[crate::NativeViewSmokeInput],
    ) -> Vec<crate::native::NativeViewInputDispatchReport> {
        let mut reports = Vec::new();
        for input in inputs {
            let dispatch = |report, reports: &mut Vec<_>| {
                reports.push(self.view.apply_input_report(report));
            };
            match input {
                crate::NativeViewSmokeInput::Move(point) => {
                    let report = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_move(*point);
                    dispatch(report, &mut reports);
                }
                crate::NativeViewSmokeInput::Click(point) => {
                    let down = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_down(*point, false);
                    dispatch(down, &mut reports);
                    let up = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_up(*point);
                    dispatch(up, &mut reports);
                }
                crate::NativeViewSmokeInput::Drag { start, end } => {
                    let down = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_down(*start, false);
                    dispatch(down, &mut reports);
                    let moved = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_move(*end);
                    dispatch(moved, &mut reports);
                    let up = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_up(*end);
                    dispatch(up, &mut reports);
                }
                crate::NativeViewSmokeInput::Text(text) => {
                    let report = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_ime_commit(text);
                    dispatch(report, &mut reports);
                }
                crate::NativeViewSmokeInput::KeyDown(key) => {
                    let report = self.view.ivars().runtime.borrow_mut().dispatch_key(*key);
                    dispatch(report, &mut reports);
                }
                crate::NativeViewSmokeInput::Scroll { point, delta_y } => {
                    let report = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_pointer_scroll(*point, crate::Dp::new(*delta_y as f32));
                    dispatch(report, &mut reports);
                }
                crate::NativeViewSmokeInput::WindowCloseRequest => {
                    let report = self
                        .view
                        .ivars()
                        .runtime
                        .borrow_mut()
                        .dispatch_window_close_requested();
                    dispatch(report, &mut reports);
                }
            }
        }
        reports
    }

    pub(crate) fn capture_png(
        &self,
        path: &Path,
    ) -> Result<crate::NativeViewCaptureEvidence, String> {
        let bounds = self.view.bounds();
        if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
            return Err("the AppKit view has empty bounds".to_string());
        }

        self.view.layoutSubtreeIfNeeded();
        self.view.setNeedsDisplay(true);
        self.view.displayIfNeeded();
        let bitmap = self
            .view
            .bitmapImageRepForCachingDisplayInRect(bounds)
            .ok_or_else(|| "AppKit could not allocate an NSBitmapImageRep".to_string())?;
        self.view
            .cacheDisplayInRect_toBitmapImageRep(bounds, &bitmap);
        let properties = NSDictionary::<NSBitmapImageRepPropertyKey, AnyObject>::new();
        let data = unsafe {
            bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties)
        }
        .ok_or_else(|| "NSBitmapImageRep could not encode PNG data".to_string())?;

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("could not create AppKit capture directory: {error}"))?;
        }
        let byte_count = data.length();
        let mut bytes = vec![0_u8; byte_count];
        if let Some(buffer) = NonNull::new(bytes.as_mut_ptr().cast::<c_void>()) {
            unsafe { data.getBytes_length(buffer, byte_count) };
        }
        std::fs::write(path, bytes)
            .map_err(|error| format!("could not write AppKit PNG capture: {error}"))?;

        let logical_width = bounds.size.width.round().max(1.0) as u32;
        let logical_height = bounds.size.height.round().max(1.0) as u32;
        let pixel_width = bitmap.pixelsWide().max(1) as u32;
        let pixel_height = bitmap.pixelsHigh().max(1) as u32;
        let scale_factor = self
            .view
            .window()
            .map(|window| window.backingScaleFactor())
            .unwrap_or_else(|| f64::from(pixel_width) / f64::from(logical_width));
        Ok(crate::NativeViewCaptureEvidence {
            platform: "macos",
            backend: "appkit_nsview_bitmap_cache",
            logical_width,
            logical_height,
            pixel_width,
            pixel_height,
            scale_factor,
            typography_scale: self.view.ivars().plan.borrow().typography_scale(),
        })
    }

    pub(crate) fn set_window_suspended(&self, suspended: bool) {
        if suspended {
            if !self
                .view
                .ivars()
                .runtime
                .borrow_mut()
                .suspend_view_when_hidden()
            {
                return;
            }
            if let Some(timer) = self.view.ivars().runtime_timer.borrow_mut().take() {
                timer.invalidate();
            }
            *self.view.ivars().plan.borrow_mut() = NativeDrawPlan::default();
            self.view.ivars().marked_text.borrow_mut().clear();
            self.view.ivars().marked_selection.set(None);
            self.view.setNeedsDisplay(true);
            return;
        }

        let Some(plan) = self
            .view
            .ivars()
            .runtime
            .borrow_mut()
            .resume_view_when_visible()
        else {
            return;
        };
        *self.view.ivars().plan.borrow_mut() = plan;
        self.view.schedule_runtime_tick();
        self.view.setNeedsDisplay(true);
    }

    pub(crate) fn dispatch_app_command(&self, command: crate::Command) {
        let report = self
            .view
            .ivars()
            .runtime
            .borrow_mut()
            .dispatch_app_command(command);
        self.view.apply_input_report(report);
    }

    pub(crate) fn dispatch_window_close_requested(&self) -> bool {
        let report = self
            .view
            .ivars()
            .runtime
            .borrow_mut()
            .dispatch_window_close_requested();
        let allow = !report.handled || report.quit_requested;
        self.view.apply_input_report(report);
        allow
    }
}

pub(crate) fn install_macos_appkit_draw_plan(
    window: &objc2_app_kit::NSWindow,
    plan: NativeDrawPlan,
    #[allow(unused_mut)] mut runtime: crate::native::NativeViewInputRuntime,
) -> MacosAppKitDrawViewHost {
    #[cfg(feature = "text-input-core")]
    runtime.use_appkit_text_shaping();
    let mut plan = plan;
    if let Some(updated) = runtime.set_typography_scale(appkit_ui_font_scale()) {
        plan = updated;
    }
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
    view.schedule_runtime_tick();
    view.setNeedsDisplay(true);
    MacosAppKitDrawViewHost { view }
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

#[cfg(feature = "text-input-core")]
pub(crate) fn shape_macos_appkit_text_line(
    text: &str,
) -> Option<crate::native_input_visuals::NativeShapedTextLine> {
    use crate::native_input_visuals::{
        NativeShapedTextCaret, NativeShapedTextCluster, NativeShapedTextLine,
    };

    if text.is_empty() {
        return None;
    }
    let body = crate::TextRole::Body.metrics_for(crate::ZsTypographyPlatformStyle::Macos);
    let typography_scale = appkit_ui_font_scale();
    let mut style = TextStyle::line(
        ".AppleSystemUIFont",
        body.size * typography_scale,
        Color::rgb(0, 0, 0),
    );
    style.line_height = body.line_height * typography_scale;
    style.semantic_role = Some(crate::TextRole::Body);
    let attributes = appkit_text_attributes(&style);
    let dictionary: &NSDictionary<NSAttributedStringKey, AnyObject> = &attributes;
    let string = NSString::from_str(text);
    let attributed = unsafe {
        NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &string,
            Some(dictionary),
        )
    };
    let line = unsafe {
        CoreTextLine::from_create(CTLineCreateWithAttributedString(
            Retained::as_ptr(&attributed).cast(),
        ))?
    };
    let boundaries = crate::native_text_edit::grapheme_boundaries(text);
    let utf16_offsets = boundaries
        .iter()
        .map(|index| {
            isize::try_from(
                text.chars()
                    .take(*index)
                    .map(char::len_utf16)
                    .sum::<usize>(),
            )
            .ok()
        })
        .collect::<Option<Vec<_>>>()?;
    let carets = boundaries
        .iter()
        .copied()
        .zip(utf16_offsets.iter().copied())
        .map(|(index, utf16)| {
            let mut secondary = 0.0;
            let primary = unsafe { CTLineGetOffsetForStringIndex(line.0, utf16, &mut secondary) };
            NativeShapedTextCaret {
                index,
                primary_x: appkit_coordinate(primary),
                secondary_x: appkit_coordinate(secondary),
            }
        })
        .collect::<Vec<_>>();
    let clusters = boundaries
        .windows(2)
        .zip(carets.windows(2))
        .map(|(scalar, caret)| {
            let (start_x, end_x) = caret[0].closest_cluster_edges(caret[1]);
            NativeShapedTextCluster {
                start: scalar[0],
                end: scalar[1],
                start_x,
                end_x,
            }
        })
        .collect::<Vec<_>>();
    let width = unsafe {
        CTLineGetTypographicBounds(
            line.0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }
    .ceil();
    NativeShapedTextLine::new(appkit_coordinate(width), clusters, carets)
}

#[cfg(feature = "text-input-core")]
struct CoreTextLine(*const c_void);

#[cfg(feature = "text-input-core")]
impl CoreTextLine {
    unsafe fn from_create(line: *const c_void) -> Option<Self> {
        (!line.is_null()).then_some(Self(line))
    }
}

#[cfg(feature = "text-input-core")]
impl Drop for CoreTextLine {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CFRelease(self.0);
            }
        }
    }
}

#[cfg(feature = "text-input-core")]
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTLineCreateWithAttributedString(attributed_string: *const c_void) -> *const c_void;
    fn CTLineGetOffsetForStringIndex(
        line: *const c_void,
        char_index: isize,
        secondary_offset: *mut f64,
    ) -> f64;
    fn CTLineGetTypographicBounds(
        line: *const c_void,
        ascent: *mut f64,
        descent: *mut f64,
        leading: *mut f64,
    ) -> f64;
}

#[cfg(feature = "text-input-core")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: *const c_void);
}

struct MacosAppKitDrawSink {
    palette: NativeDrawPalette,
    style_resolver: NativeDrawTextStyleResolver,
    text_layout: MacosAppKitTextLayout,
    clip_depth: usize,
}

impl MacosAppKitDrawSink {
    fn new(palette: NativeDrawPalette, typography_scale: f32) -> Self {
        Self {
            palette,
            style_resolver: NativeDrawTextStyleResolver::new(
                ".AppleSystemUIFont",
                "Menlo",
                ".AppleSystemUIFont",
                crate::ZsTypographyPlatformStyle::Macos,
                palette,
            )
            .with_typography_scale(typography_scale),
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
        // NSString treats the rect origin as a baseline unless line-fragment
        // layout is requested. The shared draw protocol supplies a top-left
        // line box, so use line-fragment layout for single-line text too.
        let mut options = NSStringDrawingOptions::UsesFontLeading
            | NSStringDrawingOptions::UsesLineFragmentOrigin;
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

    fn draw_image(&self, command: &NativeDrawImageCommand) {
        let Some(graphics_context) = NSGraphicsContext::currentContext() else {
            return;
        };
        let context: *mut c_void = unsafe { msg_send![&*graphics_context, CGContext] };
        if context.is_null()
            || command.bounds.width <= 0
            || command.bounds.height <= 0
            || command.source.width <= 0
            || command.source.height <= 0
        {
            return;
        }
        let width = command.frame.width() as usize;
        let height = command.frame.height() as usize;
        let Some(bytes_per_row) = width.checked_mul(4) else {
            return;
        };
        if height.checked_mul(bytes_per_row) != Some(command.frame.decoded_bytes()) {
            return;
        }
        unsafe {
            let provider = CGDataProviderCreateWithData(
                std::ptr::null_mut(),
                command.frame.premultiplied_bgra8().as_ptr().cast(),
                command.frame.decoded_bytes(),
                None,
            );
            if provider.is_null() {
                return;
            }
            let color_space = CGColorSpaceCreateDeviceRGB();
            if color_space.is_null() {
                CGDataProviderRelease(provider);
                return;
            }
            let image = CGImageCreate(
                width,
                height,
                8,
                32,
                bytes_per_row,
                color_space,
                CORE_GRAPHICS_ALPHA_PREMULTIPLIED_FIRST | CORE_GRAPHICS_BYTE_ORDER_32_LITTLE,
                provider,
                std::ptr::null(),
                true,
                CORE_GRAPHICS_RENDERING_INTENT_DEFAULT,
            );
            if image.is_null() {
                CGColorSpaceRelease(color_space);
                CGDataProviderRelease(provider);
                return;
            }
            let cropped = CGImageCreateWithImageInRect(
                image,
                CoreGraphicsRect {
                    origin: CoreGraphicsPoint {
                        x: f64::from(command.source.x),
                        y: f64::from(command.source.y),
                    },
                    size: CoreGraphicsSize {
                        width: f64::from(command.source.width),
                        height: f64::from(command.source.height),
                    },
                },
            );
            if !cropped.is_null() {
                CGContextSaveGState(context);
                CGContextSetInterpolationQuality(
                    context,
                    match command.interpolation {
                        NativeImageInterpolation::Nearest => CORE_GRAPHICS_INTERPOLATION_NONE,
                        NativeImageInterpolation::Smooth => CORE_GRAPHICS_INTERPOLATION_HIGH,
                    },
                );
                CGContextTranslateCTM(
                    context,
                    f64::from(command.bounds.x),
                    f64::from(command.bounds.y + command.bounds.height),
                );
                CGContextScaleCTM(context, 1.0, -1.0);
                CGContextDrawImage(
                    context,
                    CoreGraphicsRect {
                        origin: CoreGraphicsPoint { x: 0.0, y: 0.0 },
                        size: CoreGraphicsSize {
                            width: f64::from(command.bounds.width),
                            height: f64::from(command.bounds.height),
                        },
                    },
                    cropped,
                );
                CGContextRestoreGState(context);
                CGImageRelease(cropped);
            }
            CGImageRelease(image);
            CGColorSpaceRelease(color_space);
            CGDataProviderRelease(provider);
        }
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
                appkit_color(self.palette.resolve_source_fill(*fill)).setFill();
                NSBezierPath::fillRect(appkit_rect(*rect));
            }
            NativeDrawCommand::StrokeRect {
                rect,
                stroke,
                width,
            } => {
                appkit_color(self.palette.resolve_source_fill(*stroke)).setStroke();
                let path = NSBezierPath::bezierPathWithRect(appkit_rect(*rect));
                path.setLineWidth(f64::from((*width).max(1)));
                path.stroke();
            }
            NativeDrawCommand::StrokeArc {
                rect,
                stroke,
                width,
                start_degrees,
                sweep_degrees,
            } => {
                let center = NSPoint::new(
                    f64::from(rect.x) + f64::from(rect.width) / 2.0,
                    f64::from(rect.y) + f64::from(rect.height) / 2.0,
                );
                let radius = f64::from(rect.width.min(rect.height).max(0)) / 2.0;
                let path = NSBezierPath::bezierPath();
                path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
                    center,
                    radius,
                    f64::from(*start_degrees),
                    f64::from(start_degrees.saturating_add(*sweep_degrees)),
                    true,
                );
                appkit_color(self.palette.resolve_source_fill(*stroke)).setStroke();
                path.setLineWidth(f64::from((*width).max(1)));
                path.stroke();
            }
            NativeDrawCommand::FillTriangle { points, fill } => {
                let path = NSBezierPath::bezierPath();
                path.moveToPoint(NSPoint::new(f64::from(points[0].x), f64::from(points[0].y)));
                for point in &points[1..] {
                    path.lineToPoint(NSPoint::new(f64::from(point.x), f64::from(point.y)));
                }
                path.closePath();
                appkit_color(self.palette.resolve_source_fill(*fill)).setFill();
                path.fill();
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
                appkit_color(self.palette.resolve_source_fill(*fill)).setFill();
                path.fill();
                if let Some(stroke) = stroke {
                    appkit_color(self.palette.resolve_source_fill(*stroke)).setStroke();
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
                appkit_color(self.palette.resolve_source_fill(*fill)).setFill();
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
            NativeDrawCommand::Image(command) => self.draw_image(command),
            NativeDrawCommand::PushClip { rect } => {
                NSGraphicsContext::saveGraphicsState_class();
                NSBezierPath::bezierPathWithRect(appkit_rect(*rect)).addClip();
                self.clip_depth += 1;
            }
            NativeDrawCommand::PopClip => self.pop_clip(),
        }
    }
}

pub(crate) fn appkit_ui_font_scale() -> f32 {
    let options = NSDictionary::<NSFontTextStyleOptionKey, AnyObject>::new();
    let font = unsafe { NSFont::preferredFontForTextStyle_options(NSFontTextStyleBody, &options) };
    (font.pointSize() as f32 / 13.0).clamp(0.75, 3.0)
}

fn appkit_text_attributes(
    style: &TextStyle,
) -> Retained<NSMutableDictionary<NSAttributedStringKey, AnyObject>> {
    let attributes = NSMutableDictionary::<NSAttributedStringKey, AnyObject>::new();
    let weight = unsafe {
        match style.weight {
            TextWeight::Automatic => NSFontWeightRegular,
            TextWeight::Regular => NSFontWeightRegular,
            TextWeight::Medium => NSFontWeightMedium,
            TextWeight::Semibold => NSFontWeightSemibold,
            TextWeight::Bold => NSFontWeightBold,
        }
    };
    let font = if style.semantic_role == Some(crate::TextRole::Monospace)
        || style.font_family == "Menlo"
    {
        NSFont::monospacedSystemFontOfSize_weight(f64::from(style.size), weight)
    } else if style.semantic_role == Some(crate::TextRole::Button)
        && style.weight
            == crate::TextRole::Button
                .metrics_for(crate::ZsTypographyPlatformStyle::Macos)
                .default_weight
    {
        NSFont::controlContentFontOfSize(f64::from(style.size))
    } else if let Some(text_style) = style
        .semantic_role
        .filter(|role| {
            style.weight
                == role
                    .metrics_for(crate::ZsTypographyPlatformStyle::Macos)
                    .default_weight
        })
        .and_then(appkit_preferred_text_style)
    {
        let options = NSDictionary::<NSFontTextStyleOptionKey, AnyObject>::new();
        unsafe { NSFont::preferredFontForTextStyle_options(text_style, &options) }
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
    if style.line_height > 0.0 {
        let line_height = f64::from(style.line_height.max(style.size));
        paragraph.setMinimumLineHeight(line_height);
        paragraph.setMaximumLineHeight(line_height);
    }
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

fn appkit_preferred_text_style(role: crate::TextRole) -> Option<&'static NSFontTextStyle> {
    // These AppKit text-style names are process-lifetime framework constants.
    unsafe {
        match role {
            crate::TextRole::Caption => Some(NSFontTextStyleCaption1),
            crate::TextRole::Body => Some(NSFontTextStyleBody),
            crate::TextRole::BodyLarge => Some(NSFontTextStyleTitle3),
            crate::TextRole::Subtitle => Some(NSFontTextStyleTitle2),
            crate::TextRole::Title => Some(NSFontTextStyleTitle1),
            crate::TextRole::TitleLarge | crate::TextRole::Display => {
                Some(NSFontTextStyleLargeTitle)
            }
            crate::TextRole::Button | crate::TextRole::Icon | crate::TextRole::Monospace => None,
        }
    }
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

fn appkit_semantic_palette() -> Option<NativeDrawPalette> {
    Some(NativeDrawPalette {
        primary_text: appkit_native_color(&NSColor::labelColor())?,
        secondary_text: appkit_native_color(&NSColor::secondaryLabelColor())?,
        disabled_text: appkit_native_color(&NSColor::disabledControlTextColor())?,
        accent: appkit_native_color(&NSColor::selectedContentBackgroundColor())?,
        accent_text: appkit_native_color(&NSColor::selectedControlTextColor())?,
        surface: appkit_native_color(&NSColor::windowBackgroundColor())?,
        surface_raised: appkit_native_color(&NSColor::textBackgroundColor())?,
        control: appkit_native_color(&NSColor::controlBackgroundColor())?,
        border: appkit_native_color(&NSColor::separatorColor())?,
        success: appkit_native_color(&NSColor::systemGreenColor())?,
        warning: appkit_native_color(&NSColor::systemOrangeColor())?,
        danger: appkit_native_color(&NSColor::systemRedColor())?,
        high_contrast: false,
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
