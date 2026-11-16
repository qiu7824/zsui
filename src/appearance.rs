//! Application appearance overrides layered over the native platform preset.
use crate::{Color, ColorRole, Dp, TextStyle, TextWeight, ZsFontSize};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Typography shared by labels, controls, measurement and native text input.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontSpec {
    /// Preferred families, in order. An empty list preserves the platform family.
    pub families: Vec<String>,
    /// Logical size, independent of the display scale.
    pub size: Option<ZsFontSize>,
    /// Logical line height. Omission preserves the semantic role's ratio.
    pub line_height: Option<ZsFontSize>,
    /// Requested font weight, with platform fallback for unavailable faces.
    pub weight: Option<TextWeight>,
}
impl FontSpec {
    /// Creates a primary family followed by the host's normal glyph fallback.
    pub fn family(family: impl Into<String>) -> Self {
        Self {
            families: vec![family.into()],
            ..Self::default()
        }
    }
    /// Adds an explicit fallback family.
    pub fn fallback(mut self, family: impl Into<String>) -> Self {
        self.families.push(family.into());
        self
    }
    /// Sets a logical font size.
    pub fn size(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.size = Some(ZsFontSize::from_dip(value.0));
        self
    }
    /// Sets logical line spacing.
    pub fn line_height(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.line_height = Some(ZsFontSize::from_dip(value.0));
        self
    }
    /// Sets a requested weight.
    pub fn weight(mut self, value: TextWeight) -> Self {
        self.weight = Some(value);
        self
    }
    pub(crate) fn cascade(&self, local: &Self) -> Self {
        Self {
            families: if local.families.is_empty() {
                self.families.clone()
            } else {
                local.families.clone()
            },
            size: local.size.or(self.size),
            line_height: local.line_height.or(self.line_height),
            weight: local.weight.or(self.weight),
        }
    }
    pub(crate) fn resolved_metrics(&self, base_size: f32, base_line_height: f32) -> (f32, f32) {
        let size = self.size.map_or(base_size, ZsFontSize::dip).max(1.0);
        let height = self.line_height.map_or_else(
            || {
                if self.size.is_some() {
                    size * base_line_height / base_size.max(1.0)
                } else {
                    base_line_height
                }
            },
            ZsFontSize::dip,
        );
        (size, height.max(size))
    }
    /// Applies this declaration to a resolved native role style.
    pub fn apply(&self, style: &mut TextStyle) {
        if let Some(primary) = self.families.first() {
            style.font_family = primary.clone();
            style.fallback_families = self.families[1..].to_vec();
            style.semantic_role = None;
        }
        if self.size.is_some() || self.line_height.is_some() {
            (style.size, style.line_height) = self.resolved_metrics(style.size, style.line_height);
        }
        if let Some(weight) = self.weight {
            style.weight = weight;
        }
    }
}

/// Optional colors and geometry for a control state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStyle {
    /// Semantic surface override, used when an explicit color is absent.
    pub background_role: Option<ColorRole>,
    /// Semantic outline override, used when an explicit color is absent.
    pub border_role: Option<ColorRole>,
    /// Surface fill. Its semantic role is retained for high-contrast mode.
    pub background: Option<Color>,
    /// Text and icon foreground.
    pub foreground: Option<Color>,
    /// Outline color.
    pub border: Option<Color>,
    /// Outline width in logical units.
    pub border_width: Option<ZsFontSize>,
    /// Corner radius in logical units.
    pub radius: Option<ZsFontSize>,
}
impl ControlStyle {
    /// Overrides the surface color.
    pub fn background(mut self, value: Color) -> Self {
        self.background = Some(value);
        self
    }
    /// Overrides text and icon color.
    pub fn foreground(mut self, value: Color) -> Self {
        self.foreground = Some(value);
        self
    }
    /// Overrides the outline color and logical width.
    pub fn border(mut self, value: Color, width: impl Into<Dp>) -> Self {
        let width = width.into();
        self.border = Some(value);
        self.border_width = Some(ZsFontSize::from_dip(width.0));
        self
    }
    /// Overrides the corner radius.
    pub fn radius(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.radius = Some(ZsFontSize::from_dip(value.0));
        self
    }
    pub(crate) fn cascade(&self, other: &Self) -> Self {
        Self {
            background_role: other.background_role.or(self.background_role),
            border_role: other.border_role.or(self.border_role),
            background: other.background.or(self.background),
            foreground: other.foreground.or(self.foreground),
            border: other.border.or(self.border),
            border_width: other.border_width.or(self.border_width),
            radius: other.radius.or(self.radius),
        }
    }
}

/// State-specific overrides; later states have explicit, deterministic priority.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateStyles {
    pub normal: ControlStyle,
    pub hover: ControlStyle,
    pub pressed: ControlStyle,
    pub selected: ControlStyle,
    pub focus_visible: ControlStyle,
    pub invalid: ControlStyle,
    pub disabled: ControlStyle,
}
impl StateStyles {
    pub(crate) fn cascade(&self, other: &Self) -> Self {
        Self {
            normal: self.normal.cascade(&other.normal),
            hover: self.hover.cascade(&other.hover),
            pressed: self.pressed.cascade(&other.pressed),
            selected: self.selected.cascade(&other.selected),
            focus_visible: self.focus_visible.cascade(&other.focus_visible),
            invalid: self.invalid.cascade(&other.invalid),
            disabled: self.disabled.cascade(&other.disabled),
        }
    }
    pub(crate) fn resolve(&self, state: ControlState) -> ControlStyle {
        let mut style = self.normal.clone();
        for (enabled, rule) in [
            (state.selected, &self.selected),
            (state.hover, &self.hover),
            (state.pressed, &self.pressed),
            (state.focus_visible, &self.focus_visible),
            (state.invalid, &self.invalid),
            (state.disabled, &self.disabled),
        ] {
            if enabled {
                style = style.cascade(rule);
            }
        }
        style
    }
}

/// Explicit visual state. Native hover, press and keyboard focus are merged at paint time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlState {
    pub hover: bool,
    pub pressed: bool,
    pub selected: bool,
    pub focus_visible: bool,
    pub disabled: bool,
    pub invalid: bool,
}

/// Tab chrome independent of platform, preserving native default behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabAppearance {
    #[default]
    Platform,
    Underline,
    Filled,
    Pill,
    Segmented,
}
/// Focus rendering independent of input routing or accessibility focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusAppearance {
    #[default]
    Platform,
    Outline,
    Underline,
    None,
}
/// Logical control density; accessibility hit regions retain their own geometry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlDensity {
    Compact,
    #[default]
    Comfortable,
    Touch,
}

/// Reusable appearance declaration. Subtree inheritance is opt-in through a theme.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appearance {
    pub font: FontSpec,
    pub colors: Vec<(ColorRole, Color)>,
    pub states: StateStyles,
    pub tabs: Option<TabAppearance>,
    pub focus: Option<FocusAppearance>,
    pub density: Option<ControlDensity>,
    pub icon_size: Option<ZsFontSize>,
    pub padding: Option<[ZsFontSize; 4]>,
    pub gap: Option<ZsFontSize>,
}
impl Appearance {
    /// Sets default subtree padding; explicit View padding has higher priority.
    pub fn padding(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.padding = Some([ZsFontSize::from_dip(value.0); 4]);
        self
    }
    /// Sets the default gap between children; explicit View gap has higher priority.
    pub fn gap(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.gap = Some(ZsFontSize::from_dip(value.0));
        self
    }
    /// Assigns a semantic palette entry.
    pub fn color(mut self, role: ColorRole, value: Color) -> Self {
        self.colors.retain(|(key, _)| *key != role);
        self.colors.push((role, value));
        self
    }
    /// Assigns default subtree typography.
    pub fn font(mut self, font: FontSpec) -> Self {
        self.font = font;
        self
    }
    /// Assigns control state rules.
    pub fn states(mut self, states: StateStyles) -> Self {
        self.states = states;
        self
    }
    /// Chooses tab rendering while retaining the same selection model.
    pub fn tabs(mut self, value: TabAppearance) -> Self {
        self.tabs = Some(value);
        self
    }
    /// Chooses keyboard focus chrome.
    pub fn focus(mut self, value: FocusAppearance) -> Self {
        self.focus = Some(value);
        self
    }
    /// Chooses logical control density.
    pub fn density(mut self, value: ControlDensity) -> Self {
        self.density = Some(value);
        self
    }
    /// Sets icon artwork size without changing its hit slot.
    pub fn icon_size(mut self, value: impl Into<Dp>) -> Self {
        let value = value.into();
        self.icon_size = Some(ZsFontSize::from_dip(value.0));
        self
    }
    pub(crate) fn cascade(&self, local: &Self) -> Self {
        let mut result = self.clone();
        result.font = self.font.cascade(&local.font);
        result.states = self.states.cascade(&local.states);
        for &(role, color) in &local.colors {
            result = result.color(role, color);
        }
        result.tabs = local.tabs.or(result.tabs);
        result.focus = local.focus.or(result.focus);
        result.density = local.density.or(result.density);
        result.icon_size = local.icon_size.or(result.icon_size);
        result.padding = local.padding.or(result.padding);
        result.gap = local.gap.or(result.gap);
        result
    }
    pub(crate) fn color_for(&self, role: ColorRole) -> Option<Color> {
        self.colors
            .iter()
            .rev()
            .find(|(key, _)| *key == role)
            .map(|(_, color)| *color)
    }
}

/// An application/subtree theme shared without per-node palette copies.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppTheme(pub Arc<Appearance>);
impl AppTheme {
    /// Creates a theme layered over platform defaults.
    pub fn new(appearance: Appearance) -> Self {
        Self(Arc::new(appearance))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaintAppearance {
    #[cfg(feature = "custom-icons")]
    pub icon_source: Option<crate::IconSource>,
    pub widget: Option<crate::WidgetId>,
    pub bounds: crate::Rect,
    pub fixed_icon_slots: bool,
    pub style: Arc<Appearance>,
    pub state: ControlState,
    pub commands: Vec<usize>,
}

impl PaintAppearance {
    pub(crate) fn style_bounds(&self, command: &crate::NativeDrawCommand) -> crate::Rect {
        if self.fixed_icon_slots {
            match command {
                crate::NativeDrawCommand::Icon(icon) => return icon.bounds,
                #[cfg(feature = "custom-icons")]
                crate::NativeDrawCommand::VectorIcon(icon) => return icon.bounds,
                _ => {}
            }
        }
        self.bounds
    }
}

fn themed_fill(
    value: &mut crate::NativeDrawFill,
    appearance: &Appearance,
    override_color: Option<Color>,
) {
    use crate::NativeDrawFill::*;
    if matches!(*value, RoleOverride { .. }) && override_color.is_none() {
        return;
    }
    let (role, alpha) = match *value {
        Role(role) => (role, 255),
        RoleWithAlpha { role, alpha } => (role, alpha),
        RoleOverride { role, alpha, .. } => (role, alpha),
        Color(_) => return,
    };
    if let Some(color) = override_color.or_else(|| appearance.color_for(role)) {
        *value = RoleOverride {
            role,
            color,
            alpha: if override_color.is_some() { 255 } else { alpha },
        };
    }
}

#[allow(dead_code)] // Used by the optional native rendering backends.
pub(crate) const fn override_color(
    color: Color,
    semantic: Color,
    alpha: u8,
    high_contrast: bool,
) -> Color {
    let color = if high_contrast { semantic } else { color };
    let alpha = if high_contrast {
        match alpha {
            0 => 0,
            1..=20 => 64,
            21..=63 => 112,
            value => value,
        }
    } else {
        alpha
    };
    Color::rgba(
        color.r,
        color.g,
        color.b,
        ((color.a as u32 * alpha as u32 + 127) / 255) as u8,
    )
}

pub(crate) fn style_command(
    command: &mut crate::NativeDrawCommand,
    appearance: &Appearance,
    style: &ControlStyle,
    bounds: crate::Rect,
    dpi: crate::Dpi,
) {
    use crate::NativeDrawCommand::*;
    let outer = |rect: crate::Rect| {
        rect.width >= bounds.width.saturating_sub(8)
            && rect.height >= bounds.height.saturating_sub(8)
    };
    match command {
        FillRect { rect, fill } => {
            if outer(*rect) {
                if let Some(role) = style.background_role {
                    *fill = crate::NativeDrawFill::Role(role);
                }
            }
            themed_fill(
                fill,
                appearance,
                outer(*rect).then_some(style.background).flatten(),
            );
            if outer(*rect) {
                if let Some(radius) = style.radius {
                    *command = RoundRect {
                        rect: *rect,
                        fill: *fill,
                        stroke: style
                            .border
                            .map(|color| crate::NativeDrawFill::RoleOverride {
                                role: ColorRole::Border,
                                color,
                                alpha: 255,
                            }),
                        radius: Dp::new(radius.dip()).to_px(dpi).round_i32(),
                    };
                }
            }
        }
        RoundFill { rect, fill, radius } => {
            if outer(*rect) {
                if let Some(role) = style.background_role {
                    *fill = crate::NativeDrawFill::Role(role);
                }
            }
            themed_fill(
                fill,
                appearance,
                outer(*rect).then_some(style.background).flatten(),
            );
            if outer(*rect) {
                if let Some(value) = style.radius {
                    *radius = Dp::new(value.dip()).to_px(dpi).round_i32();
                }
            }
        }
        RoundRect {
            rect,
            fill,
            stroke,
            radius,
        } => {
            let surface = outer(*rect);
            if surface {
                if let Some(role) = style.background_role {
                    *fill = crate::NativeDrawFill::Role(role);
                }
                if let Some(role) = style.border_role {
                    *stroke = Some(crate::NativeDrawFill::Role(role));
                }
            }
            themed_fill(
                fill,
                appearance,
                surface.then_some(style.background).flatten(),
            );
            if let Some(stroke) = stroke {
                themed_fill(
                    stroke,
                    appearance,
                    surface.then_some(style.border).flatten(),
                );
            } else if surface {
                *stroke = style
                    .border
                    .map(|color| crate::NativeDrawFill::RoleOverride {
                        role: ColorRole::Border,
                        color,
                        alpha: 255,
                    });
            }
            if surface {
                if let Some(value) = style.radius {
                    *radius = Dp::new(value.dip()).to_px(dpi).round_i32();
                }
            }
        }
        StrokeRect {
            rect,
            stroke,
            width,
        } => {
            themed_fill(
                stroke,
                appearance,
                outer(*rect).then_some(style.border).flatten(),
            );
            if outer(*rect) {
                if let Some(value) = style.border_width {
                    *width = Dp::new(value.dip()).to_px(dpi).round_i32().max(1);
                }
            }
        }
        StrokeArc { stroke, .. } => themed_fill(stroke, appearance, None),
        StyledRoundRect {
            rect,
            fill,
            stroke,
            width,
            radius,
        } => {
            if let Some(role) = style.background_role {
                *fill = crate::NativeDrawFill::Role(role);
            }
            themed_fill(fill, appearance, style.background);
            themed_fill(stroke, appearance, style.border);
            if let Some(value) = style.border_width {
                *width = Dp::new(value.dip())
                    .to_px(dpi)
                    .round_i32()
                    .max(1)
                    .min(rect.width.min(rect.height).max(1));
            }
            if let Some(value) = style.radius {
                *radius = Dp::new(value.dip()).to_px(dpi).round_i32();
            }
        }
        FillTriangle { fill, .. } => themed_fill(fill, appearance, None),
        Text(text) => {
            if text.style.role != crate::TextRole::Icon && appearance.font != FontSpec::default() {
                text.font = Some(Arc::new(appearance.font.clone()));
            }
            if let Some(color) = style.foreground.or_else(|| {
                text.color
                    .is_none()
                    .then(|| appearance.color_for(text.style.color))
                    .flatten()
            }) {
                text.color = Some(color);
            }
        }
        #[cfg(feature = "password-box")]
        SecureText(text) => {
            if appearance.font != FontSpec::default() {
                text.font = Some(Arc::new(appearance.font.clone()));
            }
        }
        Icon(icon) => {
            if let Some(color) = style.foreground.or_else(|| {
                icon.custom_color
                    .is_none()
                    .then(|| appearance.color_for(icon.color))
                    .flatten()
            }) {
                icon.custom_color = Some(color);
            }
            if let Some(size) = appearance.icon_size {
                let side = Dp::new(size.dip())
                    .to_px(dpi)
                    .round_i32()
                    .max(1)
                    .min(bounds.width.min(bounds.height).max(1));
                icon.bounds = crate::Rect {
                    x: icon.bounds.x + (icon.bounds.width - side) / 2,
                    y: icon.bounds.y + (icon.bounds.height - side) / 2,
                    width: side,
                    height: side,
                };
            }
        }
        #[cfg(feature = "custom-icons")]
        VectorIcon(icon) => {
            if let Some(color) = style.foreground.or_else(|| {
                icon.custom_color
                    .is_none()
                    .then(|| appearance.color_for(icon.color))
                    .flatten()
            }) {
                icon.custom_color = Some(color);
            }
        }
        _ => {}
    }
    if let (
        Some(width),
        RoundRect {
            rect,
            fill,
            stroke: Some(stroke),
            radius,
        },
    ) = (style.border_width, &*command)
    {
        if outer(*rect) {
            let width = Dp::new(width.dip())
                .to_px(dpi)
                .round_i32()
                .max(1)
                .min(rect.width.min(rect.height).max(1));
            if width > 1 {
                *command = StyledRoundRect {
                    rect: *rect,
                    fill: *fill,
                    stroke: *stroke,
                    width,
                    radius: *radius,
                };
            }
        }
    }
}

impl crate::NativeDrawPlan {
    pub(crate) fn scoped_fill(
        &self,
        widget: crate::WidgetId,
        role: ColorRole,
        alpha: u8,
    ) -> crate::NativeDrawFill {
        self.appearance
            .iter()
            .rev()
            .find(|record| record.widget == Some(widget))
            .and_then(|record| record.style.color_for(role))
            .map_or_else(
                || {
                    if alpha == 255 {
                        crate::NativeDrawFill::Role(role)
                    } else {
                        crate::NativeDrawFill::RoleWithAlpha { role, alpha }
                    }
                },
                |color| crate::NativeDrawFill::RoleOverride { role, color, alpha },
            )
    }
    #[allow(dead_code)] // Used when pointer-capable controls are enabled.
    pub(crate) fn custom_pointer_style(&self, widget: crate::WidgetId) -> bool {
        self.appearance.iter().any(|record| {
            record.widget == Some(widget)
                && (record.style.states.hover != ControlStyle::default()
                    || record.style.states.pressed != ControlStyle::default())
        })
    }
    pub(crate) fn custom_focus_style(&self, widget: crate::WidgetId) -> Option<FocusAppearance> {
        self.appearance
            .iter()
            .rev()
            .find(|record| record.widget == Some(widget))
            .and_then(|record| record.style.focus)
            .filter(|mode| *mode != FocusAppearance::Platform)
    }
    #[allow(dead_code)] // Used by the optional native View input runtime.
    pub(crate) fn apply_appearance_states(
        &mut self,
        hover: Option<crate::WidgetId>,
        pressed: Option<crate::WidgetId>,
        focus: Option<crate::WidgetId>,
        focus_visible: bool,
        dpi: crate::Dpi,
    ) {
        for record in &mut self.appearance {
            let mut state = record.state;
            if !state.disabled {
                state.hover |= record.widget.is_some() && record.widget == hover;
                state.pressed |=
                    record.widget.is_some() && record.widget == pressed && pressed == hover;
                state.focus_visible |=
                    focus_visible && record.widget.is_some() && record.widget == focus;
            }
            record.state = state;
            let style = record.style.states.resolve(state);
            for &index in &record.commands {
                if let Some(command) = self.commands.get_mut(index) {
                    let bounds = record.style_bounds(command);
                    style_command(command, &record.style, &style, bounds, dpi);
                }
            }
        }
    }
}
