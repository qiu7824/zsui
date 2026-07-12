use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBezierPath, NSColor, NSFont, NSFontAttributeName,
    NSFontWeightBold, NSFontWeightMedium, NSFontWeightRegular, NSFontWeightSemibold,
    NSForegroundColorAttributeName, NSGraphicsContext, NSImage, NSLineBreakMode,
    NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSStringDrawing,
    NSStringDrawingOptions, NSStringNSExtendedStringDrawing, NSTextAlignment, NSView,
};
use objc2_foundation::{
    NSAttributedStringKey, NSDictionary, NSMutableDictionary, NSObjectProtocol, NSPoint, NSRect,
    NSSize, NSString,
};

use crate::native_draw_support::{NativeDrawPalette, NativeDrawTextStyleResolver};
use crate::{
    Color, HorizontalAlign, NativeDrawCommand, NativeDrawCommandSink, NativeDrawIconCommand,
    NativeDrawPlan, NativeDrawTextCommand, NativeIconColorMode, NativeStyleResolver, Rect, Size,
    TextLayout, TextRun, TextStyle, TextWeight, TextWrap, VerticalAlign,
};

struct ZsuiAppKitDrawViewIvars {
    plan: NativeDrawPlan,
}

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ZsuiAppKitDrawViewIvars]
    struct ZsuiAppKitDrawView;

    unsafe impl NSObjectProtocol for ZsuiAppKitDrawView {}

    impl ZsuiAppKitDrawView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty_rect: NSRect) {
            let system_prefers_dark = appkit_system_prefers_dark(self.mtm());
            let palette =
                NativeDrawPalette::for_mode(self.ivars().plan.theme_mode, system_prefers_dark);
            let mut sink = MacosAppKitDrawSink::new(palette);
            sink.draw_plan(&self.ivars().plan);
        }
    }
);

impl ZsuiAppKitDrawView {
    fn new(mtm: MainThreadMarker, frame: NSRect, plan: NativeDrawPlan) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ZsuiAppKitDrawViewIvars { plan });
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

pub(crate) fn install_macos_appkit_draw_plan(
    window: &objc2_app_kit::NSWindow,
    plan: NativeDrawPlan,
) {
    let mtm = window.mtm();
    let frame = window
        .contentView()
        .map(|view| view.frame())
        .unwrap_or_else(|| NSRect::new(NSPoint::new(0.0, 0.0), window.frame().size));
    let view = ZsuiAppKitDrawView::new(mtm, frame, plan);
    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
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

fn appkit_system_prefers_dark(mtm: MainThreadMarker) -> bool {
    let application = objc2_app_kit::NSApplication::sharedApplication(mtm);
    application
        .effectiveAppearance()
        .name()
        .to_string()
        .contains("Dark")
}
